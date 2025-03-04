use std::ffi::OsStr;

use sysinfo::{ProcessRefreshKind, RefreshKind, System, UpdateKind};
use tauri::{Result, State};

use crate::state::AppState;

/// Get the LeagueClientUx process info.
#[tauri::command]
pub async fn league_client_process(state: State<'_, AppState>) -> Result<()> {
    if state.game_process.read().await.is_none() {
        let sys = System::new_with_specifics(
            RefreshKind::nothing().with_processes(
                ProcessRefreshKind::nothing()
                    .with_cmd(UpdateKind::Never)
                    .with_exe(UpdateKind::Never),
            ),
        );
        if let Some(proc) = sys
            .processes_by_exact_name(OsStr::new("LeagueClientUx.exe"))
            .next()
        {
            dbg!(proc);
            // state.game_process.write().await.replace(proc);
        }
    }

    Ok(())
}
