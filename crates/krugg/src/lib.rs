#![deny(clippy::all, clippy::nursery, rust_2018_idioms)]
#![feature(const_vec_string_slice)]
#![doc = include_str!("../../../README.md")]

use std::{borrow::Cow, time::Duration};

use mimalloc::MiMalloc;
use tauri::{Listener, Manager, RunEvent, async_runtime, tray::TrayIconBuilder};
use tauri_plugin_autostart::MacosLauncher;
use tauri_plugin_store::StoreExt;
use tokio_util::{sync::CancellationToken, task::TaskTracker};

mod commands;
mod ddragon;
mod ugg;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;
const STORE_FILE: &str = "app_data.json";
const EVENTS: [&str; 4] = [
    "lcu-lockfile",
    "lcu-base-url",
    "lcu-connected",
    "lcu-disconnected",
];

#[derive(Debug)]
pub struct State {
    /// Used to cancel all tasks before the app exits.
    cancel_token: CancellationToken,
    /// Used to wait for all tasks to complete before the app exits.
    tracker: TaskTracker,
}

pub fn run() {
    #[allow(clippy::large_stack_frames)] // generate_context macro is scuffed
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![commands::show_main_window])
        .plugin(tauri_plugin_autostart::init(
            MacosLauncher::LaunchAgent,
            None,
        ))
        .plugin(tauri_plugin_positioner::init())
        .plugin(tauri_plugin_single_instance::init(|app, _, _| {
            // Focus the main window if the user tries to launch a new instance.
            let _ = app
                .get_webview_window("main")
                .expect("no main window")
                .set_focus();
        }))
        .plugin(tauri_plugin_http::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_store::Builder::default().build())
        .plugin(tauri_plugin_lcu::init(Some(STORE_FILE)))
        .setup(|app| {
            // Set up app state.
            app.manage(State {
                cancel_token: CancellationToken::new(),
                tracker: TaskTracker::new(),
            });

            // Set up persistent store.
            app.store_builder(STORE_FILE)
                .auto_save(Duration::from_secs(60))
                .build()?;

            // Tray-relative window positioning.
            TrayIconBuilder::new()
                .on_tray_icon_event(|tray_handle, evt| {
                    tauri_plugin_positioner::on_tray_event(tray_handle.app_handle(), &evt);
                })
                .build(app)?;

            #[cfg(debug_assertions)]
            if let Some(win) = app.get_webview_window("main") {
                // win.open_devtools();

                // Log tauri events to main window console.
                // TODO: emit messages from plugin after main window ready?
                for evt in EVENTS {
                    let w = win.clone();
                    win.listen(evt, move |evt| {
                        _ = w.eval(&format!(
                            "console.log('Event: id: {:?}, payload: {}')",
                            EVENTS
                                .get(evt.id() as usize)
                                .map_or_else(|| Cow::from(evt.id().to_string()), |e| Cow::from(*e)),
                            evt.payload()
                        ));
                    });
                }
            }

            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("error while running the application")
        .run(|app, evt| {
            if let RunEvent::ExitRequested { .. } = evt {
                // Save store, cancel all tasks, and wait for tasks to complete.
                if let Ok(store) = app.store(STORE_FILE) {
                    _ = store.save();
                }

                let state = app.state::<State>();
                state.cancel_token.cancel();
                state.tracker.close();

                tokio::task::block_in_place(move || {
                    async_runtime::block_on(async {
                        state.tracker.wait().await;
                    });
                });
            }
        });
}
