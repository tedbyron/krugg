#![warn(clippy::all, clippy::nursery, rust_2018_idioms)]

use tauri::{Manager, async_runtime::RwLock, tray::TrayIconBuilder};
use tauri_plugin_autostart::{MacosLauncher, ManagerExt};
use tauri_plugin_store::StoreExt;

mod commands;
mod state;

pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            commands::league_client_process,
            commands::show_main_window,
        ])
        .plugin(tauri_plugin_autostart::init(
            MacosLauncher::LaunchAgent,
            None,
        ))
        .plugin(tauri_plugin_http::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_single_instance::init(|app, _, _| {
            // Focus the main window if the user tries to launch a new instance.
            let _ = app
                .get_webview_window("main")
                .expect("no main window")
                .set_focus();
        }))
        .plugin(tauri_plugin_positioner::init())
        .plugin(tauri_plugin_store::Builder::default().build())
        .setup(|app| {
            // Read config file.
            let config = serde_json::from_str::<state::Config>(include_str!("../config.json"))?;
            dbg!(&config);

            // App state.
            app.manage(state::AppState {
                config: RwLock::new(config),
                game_process: RwLock::new(None),
            });

            // Persistent store.
            let store = app.store("app_data.json")?;

            // Start app on boot if there's no autostart setting.
            if store.get("autostart").is_none() {
                store.set("autostart", true);
                let autostart_mgr = app.autolaunch();
                autostart_mgr.enable()?;
            }

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
