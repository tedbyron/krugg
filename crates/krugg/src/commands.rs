use tauri::{Error, Manager, Window};

#[tauri::command]
pub fn show_main_window(window: Window) -> tauri::Result<()> {
    if matches!(window.is_visible(), Ok(false)) {
        window
            .get_webview_window("main")
            .ok_or(Error::WindowNotFound)?
            .show()
    } else {
        Ok(())
    }
}
