#![warn(clippy::all, clippy::nursery, rust_2018_idioms)]

use std::{ffi::OsString, path::PathBuf, sync::Mutex};

use serde::Deserialize;

pub const STORE_FILE: &str = "app_data.json";

/// Tauri managed app state.
#[derive(Debug)]
pub struct AppState {
    pub config: Mutex<AppConfig>,
}

/// App config from JSON.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppConfig {
    pub riot_dev_key: String,
}

/// League client lockfile.
#[derive(Debug)]
pub struct LockFile {
    pub path: PathBuf,
    pub name: OsString,
    pub pid: Option<u32>,
    pub port: Option<u16>,
    pub token: String,
    pub protocol: String,
    pub username: String,
    pub address: String,
    pub b64_auth: String,
}
