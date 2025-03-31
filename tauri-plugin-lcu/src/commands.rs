use tauri::{AppHandle, Runtime};
#[cfg(feature = "ugg-types")]
use ugg_types::{
    client_runepage::{NewRunePage, RunePage},
    client_summoner::ClientSummoner,
};

use crate::LcuExt;

/// Check if the plugin is connected to the LCU API.
#[tauri::command]
pub async fn connected<R: Runtime>(app: AppHandle<R>) -> bool {
    app.lcu().connected().await
}

/// Get the current summoner.
///
/// - GET [/lol-summoner/v1/current-summoner](https://www.mingweisamuel.com/lcu-schema/tool/#/Plugin%20lol-summoner/GetLolSummonerV1CurrentSummoner)
#[cfg(feature = "ugg-types")]
#[tauri::command]
pub async fn get_current_summoner<R: Runtime>(app: AppHandle<R>) -> crate::Result<ClientSummoner> {
    app.lcu().get_current_summoner().await
}

/// Get the current rune page, unless `prefix` is provided. In the latter case,
/// the first page with a name starting with `prefix` will be returned.
///
/// - GET [/lol-perks/v1/pages](https://www.mingweisamuel.com/lcu-schema/tool/#/Plugin%20lol-perks/GetLolPerksV1Currentpage)
#[cfg(feature = "ugg-types")]
#[tauri::command]
pub async fn get_current_rune_page<R: Runtime>(
    app: AppHandle<R>,
    prefix: Option<&str>,
) -> crate::Result<RunePage> {
    app.lcu().get_current_rune_page(prefix).await
}

/// Deletes the rune page with `page_id`, and adds `rune_page`.
///
/// - DELETE [/lol-perks/v1/pages/{page_id}](https://www.mingweisamuel.com/lcu-schema/tool/#/Plugin%20lol-perks/DeleteLolPerksV1PagesById)
/// - POST [/lol-perks/v1/pages](https://www.mingweisamuel.com/lcu-schema/tool/#/Plugin%20lol-perks/PostLolPerksV1Pages)
#[cfg(feature = "ugg-types")]
#[tauri::command]
pub async fn update_rune_page<R: Runtime>(
    app: AppHandle<R>,
    page_id: i64,
    rune_page: NewRunePage,
) -> crate::Result<()> {
    app.lcu().update_rune_page(page_id, &rune_page).await?;
    Ok(())
}
