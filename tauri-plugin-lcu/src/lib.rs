#![warn(clippy::all, clippy::nursery, rust_2018_idioms)]

use krugg_model::LockFile;
use tauri::{
    AppHandle, Manager, Runtime, async_runtime,
    plugin::{Builder, TauriPlugin},
};
use tauri_plugin_shell::ShellExt;
use tauri_plugin_store::StoreExt;

mod commands;
mod error;

pub use error::{Error, Result};

/// Access to the lcu APIs.
pub struct Lcu<R: Runtime>(AppHandle<R>);

/// Extensions to [`tauri::App`], [`tauri::AppHandle`] and [`tauri::Window`] to access the lcu APIs.
pub trait LcuExt<R: Runtime> {
    fn lcu(&self) -> &Lcu<R>;
}

impl<R: Runtime, T: Manager<R>> LcuExt<R> for T {
    fn lcu(&self) -> &Lcu<R> {
        self.state::<Lcu<R>>().inner()
    }
}

/// Initializes the plugin.
pub fn init<R: Runtime>() -> TauriPlugin<R> {
    Builder::new("lcu")
        .invoke_handler(tauri::generate_handler![])
        .setup(|app, _api| {
            let lcu = Lcu(app.clone());
            app.manage(lcu);

            Ok(())
        })
        .on_window_ready(|window| {
            let lock = find_lockfile(window.app_handle()).unwrap();
            dbg!(lock);
        })
        .build()
}

#[cfg(target_os = "windows")]
pub fn find_lockfile<R: Runtime>(app: &AppHandle<R>) -> Result<LockFile> {
    use std::{fs, io, net::Ipv4Addr, path::Path, str};

    use krugg_model::STORE_FILE;

    let shell = app.shell();
    let output = async_runtime::block_on(async move {
        shell
            .command("WMIC.exe")
            .args([
                "process",
                "WHERE",
                "Name='LeagueClientUx.exe'",
                "GET",
                "CommandLine",
            ])
            .output()
            .await
    })?;

    if output.status.code() != Some(0) {
        return Err(Error::Command(output.status.code()));
    }

    let cmd = str::from_utf8(&output.stdout)?;

    let quote_positions = cmd
        .chars()
        .enumerate()
        .filter_map(|(i, c)| if c == '"' { Some(i) } else { None })
        .collect::<Box<[_]>>();
    let cmd = quote_positions
        .chunks_exact(2)
        .map(|chunk| &cmd[chunk[0] + 1..chunk[1]])
        .collect::<Box<[_]>>();

    let exe_path = Path::new(&cmd[0]);
    let arg_port = "--app-port=";
    let port = cmd.iter().find_map(|arg| {
        if arg.starts_with(arg_port) {
            Some(arg.strip_prefix(arg_port).unwrap().parse::<u16>().ok()?)
        } else {
            None
        }
    });
    let arg_token = "--remoting-auth-token=";
    let token = cmd.iter().find_map(|arg| {
        if arg.starts_with(arg_token) {
            Some(arg.strip_prefix(arg_token).unwrap().to_owned())
        } else {
            None
        }
    });

    let lockfile_path = exe_path.parent().unwrap().join("lockfile");
    let Ok(lockfile) = fs::read_to_string(&lockfile_path) else {
        return Err(Error::Io(io::Error::last_os_error()));
    };
    let lock = lockfile.split(':').collect::<Box<[_]>>();
    let pid = lock[1].parse::<u32>().ok();
    let protocol = lock[4].to_owned();

    let username = "riot".to_owned();
    let token = token.unwrap_or_else(|| lock[3].to_owned());

    let store = app.store(STORE_FILE)?;
    store.set("lockfile_path", lockfile_path.to_string_lossy());

    Ok(LockFile {
        path: lockfile_path,
        name: exe_path.file_name().unwrap().to_owned(),
        pid,
        port: port.or_else(|| lock[2].parse::<u16>().ok()),
        token,
        protocol,
        username,
        address: Ipv4Addr::LOCALHOST.to_string(),
        b64_auth: String::new(),
    })
}

#[cfg(target_os = "macos")]
pub fn find_lockfile() -> Result<LockFile> {
    // pgrep -f LeagueClientUx
}
