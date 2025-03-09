#![warn(clippy::all, clippy::nursery, rust_2018_idioms)]
#![doc = include_str!("../README.md")]

use tauri::{
    AppHandle, Manager, Runtime,
    async_runtime::{self, RwLock},
    plugin::{Builder, TauriPlugin},
};
use tauri_plugin_http::reqwest::Client;
use tokio::task;
use tokio_util::{sync::CancellationToken, task::TaskTracker};

mod commands;
mod error;
mod http;
mod lockfile;

pub use error::{Error, Result};
pub use lockfile::LockFile;

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

struct LcuState {
    /// LCU lockfile.
    lockfile: RwLock<Option<LockFile>>,
    /// Reusable HTTP client.
    client: RwLock<Option<Client>>,
    /// Used to cancel all tasks when the plugin is dropped.
    cancel_token: CancellationToken,
    /// Used to wait for all tasks to complete before dropping the plugin.
    tracker: TaskTracker,
}

/// Initialize the plugin.
pub fn init<R: Runtime>() -> TauriPlugin<R> {
    Builder::new("lcu")
        .invoke_handler(tauri::generate_handler![])
        .setup(|app, _api| {
            let lcu = Lcu(app.clone());
            app.manage(lcu);
            app.manage(LcuState {
                lockfile: RwLock::new(None),
                client: RwLock::new(None),
                cancel_token: CancellationToken::new(),
                tracker: TaskTracker::new(),
            });

            LockFile::watch(app)?;

            Ok(())
        })
        .on_drop(|app| {
            let state = app.state::<LcuState>();
            state.cancel_token.cancel();

            task::block_in_place(move || {
                async_runtime::block_on(async move {
                    state.tracker.wait().await;
                })
            });
        })
        .build()
}
