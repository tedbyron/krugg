use tauri::{AppHandle, Manager, Window, ipc::Channel};

use crate::{State, channel::KruggMessage};

#[tauri::command]
pub fn show_main_window(window: Window) -> crate::Result<()> {
    if matches!(window.is_visible(), Ok(false)) {
        window
            .get_webview_window("main")
            .ok_or(tauri::Error::WindowNotFound)?
            .show()?;
    }

    Ok(())
}

#[tauri::command]
pub async fn get_champions(app: AppHandle, channel: Channel<KruggMessage>) -> crate::Result<()> {
    let state = app.state::<State>();
    let ddragon = state.client.ddragon();
    let champs = ddragon.get_champions().await?;
    channel.send(KruggMessage::Champions(champs))?;

    Ok(())
}
