use serde::{Serialize, de::DeserializeOwned};
use tauri::{Manager, Runtime};
use tauri_plugin_http::reqwest::{
    Certificate, Client, ClientBuilder, Method, Response, Url,
    header::{self, HeaderMap, HeaderValue},
};
#[cfg(feature = "ugg-types")]
use ugg_types::{
    client_runepage::{NewRunePage, RunePage, RunePages},
    client_summoner::ClientSummoner,
};

use crate::{Lcu, LcuState, lockfile::LockFile};

const ROOT_CERT: &[u8] = include_bytes!("./riotgames.pem");

trait ResultExt<T> {
    async fn check_status(self) -> crate::Result<T>;
}

impl ResultExt<Response> for tauri_plugin_http::reqwest::Result<Response> {
    async fn check_status(self) -> crate::Result<Response> {
        match self {
            Ok(res) if res.status().is_success() => Ok(res),
            Ok(res) => Err(crate::Error::StatusCode {
                status: res.status(),
                text: res.text().await?,
            }),
            Err(err) => Err(err.into()),
        }
    }
}

/// Build a new HTTP client with auth from a lockfile.
pub fn client(lockfile: &LockFile) -> crate::Result<Client> {
    let headers = HeaderMap::from_iter([(
        header::AUTHORIZATION,
        HeaderValue::from_str(&lockfile.auth_header)?,
    )]);

    Ok(ClientBuilder::new()
        .https_only(true)
        .tls_built_in_root_certs(false)
        .add_root_certificate(Certificate::from_pem(ROOT_CERT)?)
        .default_headers(headers)
        .build()?)
}

impl<R: Runtime> Lcu<R> {
    /// Returns the HTTP client if available.
    async fn client(&self) -> crate::Result<Client> {
        let state = self.0.state::<LcuState>();
        let lock = state.client.read().await;
        lock.as_ref()
            .map_or(Err(crate::Error::Disconnected), |client| Ok(client.clone()))
    }

    /// Add a path to the LCU API base URL.
    async fn url(&self, path: &str) -> crate::Result<Url> {
        let state = self.0.state::<LcuState>();
        let lock = state.base_url.read().await;
        match &*lock {
            Some(url) => Ok(url.join(path)?),
            None => Err(crate::Error::Disconnected),
        }
    }

    /// Send a request to the LCU API.
    async fn request(&self, method: Method, path: &str) -> crate::Result<Response> {
        self.client()
            .await?
            .request(method, self.url(path).await?)
            .send()
            .await
            .check_status()
            .await
    }

    /// Send a request to the LCU API with a JSON body.
    async fn request_with_body<T: Serialize + ?Sized + Sync>(
        &self,
        method: Method,
        path: &str,
        body: &T,
    ) -> crate::Result<Response> {
        self.client()
            .await?
            .request(method, self.url(path).await?)
            .json(&body)
            .send()
            .await
            .check_status()
            .await
    }

    /// Send a GET request and deserialize the response body as JSON.
    pub async fn get<T: DeserializeOwned>(&self, path: &str) -> crate::Result<T> {
        Ok(self.request(Method::GET, path).await?.json().await?)
    }

    /// Send a GET request and return the raw response.
    pub async fn get_raw(&self, path: &str) -> crate::Result<Response> {
        self.request(Method::GET, path).await
    }

    /// Send a HEAD request.
    pub async fn head(&self, path: &str) -> crate::Result<Response> {
        self.request(Method::HEAD, path).await
    }

    /// Send a POST request.
    pub async fn post<T: Serialize + ?Sized + Sync>(
        &self,
        path: &str,
        body: &T,
    ) -> crate::Result<Response> {
        self.request_with_body(Method::POST, path, body).await
    }

    /// Send a PUT request.
    pub async fn put<T: Serialize + ?Sized + Sync>(
        &self,
        path: &str,
        body: &T,
    ) -> crate::Result<Response> {
        self.request_with_body(Method::PUT, path, body).await
    }

    /// Send a DELETE request.
    pub async fn delete(&self, path: &str) -> crate::Result<Response> {
        self.request(Method::DELETE, path).await
    }

    /// Send a PATCH request.
    pub async fn patch<T: Serialize + ?Sized + Sync>(
        &self,
        path: &str,
        body: &T,
    ) -> crate::Result<Response> {
        self.request_with_body(Method::PATCH, path, body).await
    }

    /// Get the current summoner.
    ///
    /// - GET [/lol-summoner/v1/current-summoner](https://www.mingweisamuel.com/lcu-schema/tool/#/Plugin%20lol-summoner/GetLolSummonerV1CurrentSummoner)
    #[cfg(feature = "ugg-types")]
    pub async fn get_current_summoner(&self) -> crate::Result<ClientSummoner> {
        self.get::<ClientSummoner>("/lol-summoner/v1/current-summoner")
            .await
    }

    /// Get the current rune page, unless `prefix` is provided. If `prefix` is
    /// provided, returns the first page with a name starting with `prefix`.
    ///
    /// - GET [/lol-perks/v1/pages](https://www.mingweisamuel.com/lcu-schema/tool/#/Plugin%20lol-perks/GetLolPerksV1Currentpage)
    #[cfg(feature = "ugg-types")]
    pub async fn get_current_rune_page(
        &self,
        prefix: Option<impl AsRef<str>>,
    ) -> crate::Result<RunePage> {
        let pages = self.get::<RunePages>("/lol-perks/v1/pages").await?;

        if let Some(p) = prefix {
            let p = p.as_ref();
            for page in pages.into_iter() {
                if page.name.starts_with(p) && page.is_deletable {
                    return Ok(page);
                }
            }
        } else {
            for page in pages.into_iter() {
                if page.current && page.is_deletable {
                    return Ok(page);
                }
            }
        }

        Err(crate::Error::Custom("No deletable rune page found"))
    }

    /// Deletes the rune page with `page_id`, and adds `rune_page`.
    ///
    /// - DELETE [/lol-perks/v1/pages/{page_id}](https://www.mingweisamuel.com/lcu-schema/tool/#/Plugin%20lol-perks/DeleteLolPerksV1PagesById)
    /// - POST [/lol-perks/v1/pages](https://www.mingweisamuel.com/lcu-schema/tool/#/Plugin%20lol-perks/PostLolPerksV1Pages)
    #[cfg(feature = "ugg-types")]
    pub async fn update_rune_page(
        &self,
        page_id: i64,
        rune_page: &NewRunePage,
    ) -> crate::Result<Response> {
        self.delete(&format!("lol-perks/v1/pages/{page_id}"))
            .await?;
        self.post("/lol-perks/v1/pages", rune_page).await
    }
}
