use serde_json::Value;
use tauri::{AppHandle, Runtime};
use tauri_plugin_http::reqwest::Response;
#[cfg(feature = "ugg-types")]
use ugg_types::{
    client_runepage::{NewRunePage, RunePage},
    client_summoner::ClientSummoner,
};

use crate::LcuExt;

/// Send a GET request and return the raw response.
#[tauri::command]
pub async fn get<R: Runtime>(app: AppHandle<R>, path: &str) -> crate::Result<Response> {
    app.lcu().get_raw(path).await
}

/// Send a HEAD request.
#[tauri::command]
pub async fn head<R: Runtime>(app: AppHandle<R>, path: &str) -> crate::Result<Response> {
    app.lcu().head(path).await
}

/// Send a POST request.
#[tauri::command]
pub async fn post<R: Runtime>(
    app: AppHandle<R>,
    path: &str,
    body: Value,
) -> crate::Result<Response> {
    app.lcu().post(path, body).await
}

/// Send a PUT request.
#[tauri::command]
pub async fn put<R: Runtime>(
    app: AppHandle<R>,
    path: &str,
    body: Value,
) -> crate::Result<Response> {
    app.lcu().put(path, body).await
}

/// Send a DELETE request.
#[tauri::command]
pub async fn delete<R: Runtime>(app: AppHandle<R>, path: &str) -> crate::Result<Response> {
    app.lcu().delete(path).await
}

/// Send a PATCH request.
#[tauri::command]
pub async fn patch<R: Runtime>(
    app: AppHandle<R>,
    path: &str,
    body: Value,
) -> crate::Result<Response> {
    app.lcu().patch(path, body).await
}

/// Get the current summoner.
///
/// - GET [/lol-summoner/v1/current-summoner](https://www.mingweisamuel.com/lcu-schema/tool/#/Plugin%20lol-summoner/GetLolSummonerV1CurrentSummoner)
#[cfg(feature = "ugg-types")]
#[tauri::command]
pub async fn get_current_summoner<R: Runtime>(app: AppHandle<R>) -> crate::Result<ClientSummoner> {
    app.lcu().get_current_summoner().await
}

/// Get the current rune page.
///
/// - GET [/lol-perks/v1/pages](https://www.mingweisamuel.com/lcu-schema/tool/#/Plugin%20lol-perks/GetLolPerksV1Currentpage)
#[cfg(feature = "ugg-types")]
#[tauri::command]
pub async fn get_current_rune_page<R: Runtime>(app: AppHandle<R>) -> crate::Result<RunePage> {
    app.lcu().get_current_rune_page().await
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
    rune_page: &NewRunePage,
) -> crate::Result<Response> {
    app.lcu().update_rune_page(page_id, rune_page).await
}
