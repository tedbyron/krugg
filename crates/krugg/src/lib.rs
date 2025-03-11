#![deny(clippy::all, clippy::nursery, rust_2018_idioms)]
#![doc = include_str!("../../../README.md")]

use std::{path::PathBuf, time::Duration};

use mimalloc::MiMalloc;
use tauri::{Listener, Manager, tray::TrayIconBuilder};
use tauri_plugin_autostart::MacosLauncher;
use tauri_plugin_store::StoreExt;

mod commands;

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
pub struct AppState {
    store_path: PathBuf,
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
            // Set up persistent store.
            app.store_builder(STORE_FILE)
                .auto_save(Duration::from_secs(60))
                .build()?;

            // Set up app state.
            let store_path = tauri_plugin_store::resolve_store_path(app.handle(), STORE_FILE)?
                .parent()
                .unwrap()
                .to_owned();
            app.manage(AppState { store_path });

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
                for evt in EVENTS {
                    let w = win.clone();
                    win.listen(evt, move |evt| {
                        _ = w.eval(&format!(
                            "console.log('id: {}, payload: {}')",
                            evt.id(),
                            evt.payload()
                        ));
                    });
                }
            }

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running the application");
}
