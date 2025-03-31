use tauri::{AppHandle, Emitter, Runtime, async_runtime::RwLock};
use tauri_plugin_http::reqwest::{Client, Url};
use tokio_util::{sync::CancellationToken, task::TaskTracker};

use crate::LockFile;

#[derive(Debug)]
pub struct LcuState {
    /// Persistent store file.
    #[cfg(feature = "tauri-plugin-store")]
    pub store_file: String,
    /// LCU lockfile.
    pub lockfile: RwLock<Option<LockFile>>,
    /// LCU API base URL, including protocol, hostname, and port.
    pub base_url: RwLock<Option<Url>>,
    /// HTTP client.
    pub client: RwLock<Option<Client>>,
    /// Used to cancel all tasks when the plugin is dropped.
    pub cancel_token: CancellationToken,
    /// Used to wait for all tasks to complete before dropping the plugin.
    pub tracker: TaskTracker,
}

impl LcuState {
    /// Update `lockfile`, `base_url`, and `client` fields.
    pub async fn update<R: Runtime>(&self, app: &AppHandle<R>, lockfile: LockFile, url: Url) {
        if let Ok(client) = crate::http::client(&lockfile) {
            {
                let mut lock = self.client.write().await;
                *lock = Some(client);
            }
            _ = app.emit("lcu-connected", true);
        }
        {
            _ = app.emit("lcu-base-url", &url.as_str());
            let mut lock = self.base_url.write().await;
            *lock = Some(url);
        }
        {
            _ = app.emit("lcu-lockfile", &lockfile);
            let mut lock = self.lockfile.write().await;
            *lock = Some(lockfile);
        }
    }

    /// Reset `lockfile`, `base_url`, and `client` fields.
    pub async fn reset<R: Runtime>(&self, app: &AppHandle<R>) {
        {
            _ = app.emit("lcu-connected", false);
            let mut lock = self.client.write().await;
            *lock = None;
        }
        {
            _ = app.emit("lcu-base-url", ());
            let mut lock = self.base_url.write().await;
            *lock = None;
        }
        {
            _ = app.emit("lcu-lockfile", ());
            let mut lock = self.lockfile.write().await;
            *lock = None;
        }
    }
}
