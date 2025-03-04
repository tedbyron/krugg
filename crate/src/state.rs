use serde::Deserialize;
use sysinfo::Process;
use tauri::async_runtime::RwLock;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    pub riot_dev_key: String,
}

#[derive(Debug)]
pub struct AppState {
    pub config: RwLock<Config>,
    pub game_process: RwLock<Option<Process>>,
}
