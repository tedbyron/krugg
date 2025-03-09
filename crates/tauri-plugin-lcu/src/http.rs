use krugg_model::{
    client_runepage::{NewRunePage, RunePage, RunePages},
    client_summoner::ClientSummoner,
};
use serde::{Serialize, de::DeserializeOwned};
use tauri::{Manager, Runtime};
use tauri_plugin_http::reqwest::{
    Certificate, Client, ClientBuilder, Response, Url,
    header::{self, HeaderMap, HeaderValue},
};

use crate::{Lcu, LcuState, Result, lockfile::LockFile};

const ROOT_CERT: &[u8] = include_bytes!("./riotgames.pem");

/// Build a new HTTP client with auth from a lockfile.
pub fn client(lockfile: &LockFile) -> Result<Client> {
    let mut headers = HeaderMap::new();
    headers.insert(
        header::AUTHORIZATION,
        HeaderValue::from_str(&lockfile.auth_header)?,
    );

    Ok(ClientBuilder::new()
        .https_only(true)
        .add_root_certificate(Certificate::from_pem(ROOT_CERT)?)
        .tls_built_in_root_certs(false)
        .default_headers(headers)
        .build()?)
}

impl<R: Runtime> Lcu<R> {
    /// Returns the HTTP client if available.
    async fn client(&self) -> Option<Client> {
        let state = self.0.state::<LcuState>();
        let lock = state.client.lock().await;
        // Reqwest client is wrapped in an Arc, so it's cheap to clone.
        lock.clone()
    }

    /// Add a path to the LCU API base URL.
    pub fn url(&self, path: &str) -> Option<Url> {
        let state = self.0.state::<LcuState>();
        let mut url = {
            let lock = state.lockfile.blocking_read();
            match &*lock {
                Some(lockfile) => lockfile.base_url.clone(),
                None => {
                    return None;
                }
            }
        };
        url.set_path(path);
        Some(url)
    }

    /// Send a GET request.
    pub async fn get<T: DeserializeOwned>(&self, path: &str) -> Option<T> {
        match self.client().await?.get(self.url(path)?).send().await {
            Ok(res) if res.status().is_success() => res.json().await.ok(),
            _ => None,
        }
    }

    /// Send a POST request.
    pub async fn post<T: Serialize>(&self, path: &str, body: T) -> Option<Response> {
        if let Some(client) = self.client().await {
            client.post(self.url(path)?).json(&body).send().await.ok()
        } else {
            None
        }
    }

    /// Send a PUT request.
    pub async fn put<T: Serialize>(&self, path: &str, body: T) -> Option<Response> {
        if let Some(client) = self.client().await {
            client.put(self.url(path)?).json(&body).send().await.ok()
        } else {
            None
        }
    }

    /// Send a DELETE request.
    pub async fn delete(&self, path: &str) -> Option<Response> {
        match self.client().await?.delete(self.url(path)?).send().await {
            Ok(res) if res.status().is_success() => Some(res),
            _ => None,
        }
    }

    /// Get the current summoner.
    ///
    /// - GET [/lol-summoner/v1/current-summoner](https://www.mingweisamuel.com/lcu-schema/tool/#/Plugin%20lol-summoner/GetLolSummonerV1CurrentSummoner)
    pub async fn get_summoner(&self) -> Option<ClientSummoner> {
        self.get::<ClientSummoner>("/lol-summoner/v1/current-summoner")
            .await
    }

    /// Get the current rune page.
    ///
    /// - GET [/lol-perks/v1/pages](https://www.mingweisamuel.com/lcu-schema/tool/#/Plugin%20lol-perks/GetLolPerksV1Currentpage)
    pub async fn get_rune_page(&self) -> Option<RunePage> {
        match self.get::<RunePages>("/lol-perks/v1/pages").await {
            Some(ref pages) => {
                for page in pages {
                    if page.name.starts_with("krugg:") && page.is_deletable {
                        return Some(page.clone());
                    }
                }
                for page in pages {
                    if page.current && page.is_deletable {
                        return Some(page.clone());
                    }
                }
                None
            }
            None => None,
        }
    }

    /// Deletes the rune page with `page_id`, and adds `rune_page`.
    ///
    /// - DELETE [/lol-perks/v1/pages/{page_id}](https://www.mingweisamuel.com/lcu-schema/tool/#/Plugin%20lol-perks/DeleteLolPerksV1PagesById)
    /// - POST [/lol-perks/v1/pages](https://www.mingweisamuel.com/lcu-schema/tool/#/Plugin%20lol-perks/PostLolPerksV1Pages)
    pub async fn update_rune_page(
        &self,
        page_id: i64,
        rune_page: &NewRunePage,
    ) -> Option<Response> {
        self.delete(&format!("lol-perks/v1/pages/{page_id}"))
            .await?;
        self.post("/lol-perks/v1/pages", rune_page).await
    }
}
