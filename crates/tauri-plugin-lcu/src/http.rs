use tauri::Runtime;
use tauri_plugin_http::reqwest::{
    Certificate, Client, ClientBuilder,
    header::{self, HeaderMap, HeaderValue},
};

use crate::{Lcu, LockFile, Result};

const ROOT_CERT: &[u8] = include_bytes!("./riotgames.pem");

/// Build a new HTTP client with auth from the lockfile.
pub fn client(lockfile: &LockFile) -> Result<Client> {
    let mut headers = HeaderMap::new();
    headers.insert(
        header::AUTHORIZATION,
        HeaderValue::from_str(&lockfile.b64_auth)?,
    );

    Ok(ClientBuilder::new()
        .https_only(true)
        .add_root_certificate(Certificate::from_pem(ROOT_CERT)?)
        .tls_built_in_root_certs(false)
        .default_headers(headers)
        .build()?)
}

impl<R: Runtime> Lcu<R> {
    //
}
