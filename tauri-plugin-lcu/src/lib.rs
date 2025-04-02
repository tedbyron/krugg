#![deny(clippy::all, clippy::nursery, rust_2018_idioms)]
#![forbid(unsafe_code)]
#![doc = include_str!("../README.md")]

use tauri::{
    AppHandle, Manager, Runtime,
    async_runtime::{self, RwLock},
    plugin::{Builder, TauriPlugin},
};
use tokio::task;
use tokio_util::{sync::CancellationToken, task::TaskTracker};

mod commands;
mod error;
mod http;
mod lockfile;
mod state;

pub use error::{Error, Result};
use lockfile::LockFile;
use state::LcuState;

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

/// Initialize the plugin.
#[cfg(not(feature = "tauri-plugin-store"))]
pub fn init<R: Runtime>() -> TauriPlugin<R> {
    _init(None)
}

/// Initialize the plugin.
///
/// `store_file` is a path to a `tauri-plugin-store` store. The LCU lockfile
/// path will be saved in the store under the key `lockfile_path`.
#[cfg(feature = "tauri-plugin-store")]
pub fn init<R: Runtime, S: ToString>(store_file: S) -> TauriPlugin<R> {
    _init(Some(store_file))
}

fn _init<R: Runtime, S: ToString>(store_file: Option<S>) -> TauriPlugin<R> {
    #[cfg(feature = "tauri-plugin-store")]
    let store_file = store_file.unwrap().to_string();

    Builder::new("lcu")
        .invoke_handler(tauri::generate_handler![
            commands::connected,
            #[cfg(feature = "ugg-types")]
            commands::get_current_summoner,
            #[cfg(feature = "ugg-types")]
            commands::get_current_rune_page,
            #[cfg(feature = "ugg-types")]
            commands::update_rune_page,
        ])
        .setup(|app, _| {
            let lcu = Lcu(app.clone());
            app.manage(lcu);
            app.manage(LcuState {
                #[cfg(feature = "tauri-plugin-store")]
                store_file,
                lockfile: RwLock::new(None),
                base_url: RwLock::new(None),
                client: RwLock::new(None),
                cancel_token: CancellationToken::new(),
                tracker: TaskTracker::new(),
            });

            LockFile::watch(app);

            Ok(())
        })
        .on_drop(|app| {
            // Cancel all tasks and wait for them to complete.
            let state = app.state::<LcuState>();
            state.cancel_token.cancel();
            state.tracker.close();

            task::block_in_place(move || {
                async_runtime::block_on(async {
                    state.tracker.wait().await;
                });
            });
        })
        .build()
}
