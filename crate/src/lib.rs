#![warn(clippy::all, clippy::cargo, clippy::nursery, rust_2018_idioms)]

use tauri::{Manager, tray::TrayIconBuilder};
use tauri_plugin_autostart::{MacosLauncher, ManagerExt};
use tauri_plugin_store::StoreExt;

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_autostart::init(
            MacosLauncher::LaunchAgent,
            None,
        ))
        .plugin(tauri_plugin_http::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_single_instance::init(|app, _, _| {
            let _ = app
                .get_webview_window("main")
                .expect("no main window")
                .set_focus();
        }))
        .plugin(tauri_plugin_positioner::init())
        .plugin(tauri_plugin_store::Builder::default().build())
        .setup(|app| {
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
