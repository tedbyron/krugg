#![deny(clippy::all, clippy::nursery, rust_2018_idioms)]
#![doc = include_str!("../README.md")]

use tauri::{
    AppHandle, Manager, Runtime,
    async_runtime::{self, RwLock},
    plugin::{Builder, TauriPlugin},
};
use tauri_plugin_http::reqwest::{Client, Url};
use tokio_util::{sync::CancellationToken, task::TaskTracker};

pub mod commands;
mod error;
mod http;
mod lockfile;

pub use error::{Error, Result};
use lockfile::LockFile;

/// Access to the LCU APIs.
pub struct Lcu<R: Runtime>(AppHandle<R>);

/// Extensions to [`tauri::App`], [`tauri::AppHandle`] and [`tauri::Window`] to
/// access the LCU APIs.
pub trait LcuExt<R: Runtime> {
    fn lcu(&self) -> &Lcu<R>;
}

impl<R: Runtime, T: Manager<R>> LcuExt<R> for T {
    fn lcu(&self) -> &Lcu<R> {
        self.state::<Lcu<R>>().inner()
    }
}

#[derive(Debug)]
struct LcuState {
    /// Persistent store file.
    store_file: Option<String>,
    /// LCU lockfile.
    lockfile: RwLock<Option<LockFile>>,
    /// LCU API base URL, including protocol, hostname, and port.
    base_url: RwLock<Option<Url>>,
    /// HTTP client.
    client: RwLock<Option<Client>>,
    /// Used to cancel all tasks when the plugin is dropped.
    cancel_token: CancellationToken,
    /// Used to wait for all tasks to complete before dropping the plugin.
    tracker: TaskTracker,
}

/// Initialize the plugin.
///
/// `store_file` is an optional path to a `tauri-plugin-store` store. This
/// requires the `tauri-plugin-store` feature. If provided, the LCU lockfile
/// path will be saved in the store under the key `"lockfile_path"`.
pub fn init<R: Runtime, S: ToString>(store_file: Option<S>) -> TauriPlugin<R> {
    let store_file = store_file.map(|s| s.to_string());

    Builder::new("lcu")
        .invoke_handler(tauri::generate_handler![])
        .setup(|app, _| {
            let lcu = Lcu(app.clone());
            app.manage(lcu);
            app.manage(LcuState {
                store_file,
                lockfile: RwLock::new(None),
                base_url: RwLock::new(None),
                client: RwLock::new(None),
                cancel_token: CancellationToken::new(),
                tracker: TaskTracker::new(),
            });

            LockFile::watch(app)?;

            Ok(())
        })
        .on_drop(|app| {
            // Cancel all tasks and wait for them to complete.
            let state = app.state::<LcuState>();
            state.cancel_token.cancel();
            state.tracker.close();

            tokio::task::block_in_place(move || {
                async_runtime::block_on(async {
                    state.tracker.wait().await;
                });
            });
        })
        .build()
}
