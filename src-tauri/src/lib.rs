#![warn(clippy::all, clippy::nursery, rust_2018_idioms)]

use std::sync::Mutex;

use krugg_model::{AppConfig, AppState};
use tauri::{Manager, tray::TrayIconBuilder};
use tauri_plugin_autostart::MacosLauncher;

mod commands;

pub fn run() {
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
        .plugin(tauri_plugin_lcu::init())
        .setup(|app| {
            // Read config file.
            let config = serde_json::from_str::<AppConfig>(include_str!("../config.json"))?;
            dbg!(&config);

            // Setup app state.
            app.manage(AppState {
                config: Mutex::new(config),
            });

            // Tray-relative window positioning.
            TrayIconBuilder::new()
                .on_tray_icon_event(|tray_handle, evt| {
                    tauri_plugin_positioner::on_tray_event(tray_handle.app_handle(), &evt);
                })
                .build(app)?;

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running the application");
}
