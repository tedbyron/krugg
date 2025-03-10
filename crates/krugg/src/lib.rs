#![warn(clippy::all, clippy::nursery, rust_2018_idioms)]
#![doc = include_str!("../../../README.md")]

use std::time::Duration;

use tauri::{Manager, tray::TrayIconBuilder};
use tauri_plugin_autostart::MacosLauncher;
use tauri_plugin_store::StoreExt;

mod commands;

/// Persistant store file.
const STORE_FILE: &str = "app_data.json";

/// Tauri managed app state.
#[derive(Debug)]
pub struct AppState {}

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
            // Setup app state.
            app.manage(AppState {});

            // Setup persistent store.
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
            app.get_webview_window("main").unwrap().open_devtools();

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running the application");
}
