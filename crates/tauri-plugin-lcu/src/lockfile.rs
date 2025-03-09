use std::{
    fs,
    net::Ipv4Addr,
    path::{Path, PathBuf},
};

use base64ct::{Base64, Encoding};
use krugg_model::STORE_FILE;
use notify_debouncer_full::{
    DebounceEventResult,
    notify::{EventKind, RecursiveMode},
};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, Manager, Runtime, async_runtime};
use tauri_plugin_shell::ShellExt;
use tauri_plugin_store::{JsonValue, StoreExt};
use tokio::{
    task,
    time::{self, Duration},
};

use crate::{Error, LcuState, Result, http};

/// LCU lockfile.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LockFile {
    pub path: PathBuf,
    pub pid: u32,
    pub port: u16,
    pub address: String,
    pub protocol: String,
    pub username: String,
    pub token: String,
    pub b64_auth: String,
}

impl LockFile {
    /// Retrieve the lockfile path from the store.
    fn path_from_store<R: Runtime>(app: &AppHandle<R>) -> Option<PathBuf> {
        if let Ok(store) = app.store(STORE_FILE) {
            if let Some(JsonValue::String(path)) = store.get("lockfile_path") {
                return Some(path.into());
            }
        }

        None
    }

    /// Retrieve the lockfile path from the running League client.
    #[cfg(target_os = "windows")]
    async fn path<R: Runtime>(app: &AppHandle<R>) -> Result<PathBuf> {
        let shell = app.shell();
        let output = shell
            .command("WMIC.exe")
            .args([
                "process",
                "WHERE",
                "Name='LeagueClientUx.exe'",
                "GET",
                "CommandLine",
            ])
            .output()
            .await?;
        if output.status.code() != Some(0) {
            return Err(Error::Command(output.status.code()));
        }

        let cmd = str::from_utf8(&output.stdout)?;
        if cmd.trim_ascii().is_empty() {
            return Err(Error::ParseCommand);
        }
        let quote_positions = cmd
            .chars()
            .enumerate()
            .filter_map(|(i, c)| if c == '"' { Some(i) } else { None })
            .collect::<Box<[_]>>();
        let argv = quote_positions
            .chunks_exact(2)
            .map(|chunk| &cmd[chunk[0] + 1..chunk[1]])
            .collect::<Box<[_]>>();
        let exe_path = Path::new(argv.first().ok_or(Error::ParseCommand)?);
        // let arg_port = "--app-port=";
        // let port = cmd.iter().find_map(|arg| {
        //     if arg.starts_with(arg_port) {
        //         Some(arg.strip_prefix(arg_port).unwrap().parse::<u16>().ok()?)
        //     } else {
        //         None
        //     }
        // });
        // let arg_token = "--remoting-auth-token=";
        // let token = cmd.iter().find_map(|arg| {
        //     if arg.starts_with(arg_token) {
        //         Some(arg.strip_prefix(arg_token).unwrap().to_owned())
        //     } else {
        //         None
        //     }
        // });

        Ok(exe_path.parent().unwrap().join("lockfile"))
    }

    /// Retrieve the lockfile path from the running League client.
    #[cfg(target_os = "macos")]
    async fn path<R: Runtime>(app: &AppHandle<R>) -> Result<PathBuf> {
        use tauri_plugin_shell::process::CommandEvent;

        let shell = app.shell();
        let output = shell.command("ps").args(["-xo", "args="]).output().await?;
        if output.status.code() != Some(0) {
            return Err(Error::Command(output.status.code()));
        }
        dbg!(str::from_utf8(&output.stdout));
        // let (mut rx, mut child) = shell.command("grep").arg("LeagueClientUx").spawn()?;
        // child.write(&output.stdout)?;
        // let mut buf = vec![];
        // loop {
        //     match rx.recv().await {
        //         Some(CommandEvent::Stdout(output)) => {
        //             buf.extend_from_slice(&output);
        //         }
        //         Some(CommandEvent::Terminated(_)) | None => {
        //             break;
        //         }
        //         Some(_) => (),
        //     }
        // }

        // let cmd = str::from_utf8(&buf)?;
        // dbg!(&cmd);
        // let quote_positions = cmd
        //     .chars()
        //     .enumerate()
        //     .filter_map(|(i, c)| if c == '"' { Some(i) } else { None })
        //     .collect::<Box<[_]>>();
        // let argv = quote_positions
        //     .chunks_exact(2)
        //     .map(|chunk| &cmd[chunk[0] + 1..chunk[1]])
        //     .collect::<Box<[_]>>();
        // let exe_path = Path::new(&argv[0]);

        Ok(PathBuf::new())
    }

    /// Parse the lockfile contents.
    fn parse(path: impl AsRef<Path>) -> Option<Self> {
        let path = path.as_ref();
        let Ok(lock) = fs::read_to_string(path) else {
            return None;
        };
        let parts = lock.split(':').collect::<Box<[_]>>();
        let Ok(pid) = parts[1].parse::<u32>() else {
            return None;
        };
        let Ok(port) = parts[2].parse::<u16>() else {
            return None;
        };
        let username = "riot".to_owned();
        let token = parts[3].to_owned();
        let b64_auth = format!(
            "Basic {}",
            Base64::encode_string(format!("{username}:{token}").as_bytes())
        );

        Some(Self {
            path: path.to_owned(),
            pid,
            port,
            address: Ipv4Addr::LOCALHOST.to_string(),
            protocol: parts[4].to_owned(),
            username,
            token,
            b64_auth,
        })
    }

    /// Watch for file system changes to the LCU lockfile.
    pub fn watch<R: Runtime>(app: &AppHandle<R>) -> Result<()> {
        // Get the lockfile path from the store or call Self::path every 5
        // seconds until it returns a path.
        let path = Self::path_from_store(app).unwrap_or_else(|| {
            // TODO: don't block main thread lol.
            task::block_in_place(move || {
                async_runtime::block_on(async move {
                    let mut interval = time::interval(Duration::from_secs(5));

                    loop {
                        if let Ok(lockfile_path) = Self::path(app).await {
                            break lockfile_path;
                        }
                        interval.tick().await;
                    }
                })
            })
        });

        // Update state if possible before starting the file watcher.
        if let Some(lockfile) = Self::parse(&path) {
            _ = app.emit("lcu-lockfile", lockfile.clone());
            let state = app.state::<LcuState>();
            {
                let mut lock = state.client.blocking_write();
                *lock = Some(http::client(&lockfile)?);
            }
            {
                let mut lock = state.lockfile.blocking_write();
                *lock = Some(lockfile);
            }
        }

        // Spawn a background task to update state when the file changes.
        let state = app.state::<LcuState>();
        let cancel_token = state.cancel_token.clone();
        let handle = app.clone();
        async_runtime::spawn(state.tracker.track_future(async move {
            // Watch for changes to the lockfile. The channel and watcher need
            // to live for the duration of the task.
            let (tx, mut rx) = async_runtime::channel(1);
            let p = path.clone();
            let mut watcher = notify_debouncer_full::new_debouncer(
                Duration::from_secs(1),
                None,
                move |res: DebounceEventResult| {
                    if let Ok(events) = res {
                        if let Some(evt) = events.first() {
                            match evt.kind {
                                EventKind::Create(_) | EventKind::Modify(_) => {
                                    _ = tx.blocking_send(Self::parse(&p));
                                }
                                EventKind::Remove(_) => {
                                    _ = tx.blocking_send(None);
                                }
                                _ => (),
                            }
                        }
                    }
                },
            )
            .unwrap();
            watcher.watch(&path, RecursiveMode::NonRecursive).unwrap();
            rx.close();

            let state = handle.state::<LcuState>();
            loop {
                tokio::select! {
                    biased;
                    // Unwatch the path, close and drain the channel, and break
                    // from the loop.
                    _ = cancel_token.cancelled() => {
                        _ = watcher.unwatch(&path);
                        rx.close();
                        while rx.recv().await.is_some() {}
                        break;
                    }
                    // Update state and emit the lockfile to the frontend
                    // on change.
                    Some(msg) = rx.recv() => {
                        if msg != *state.lockfile.read().await {
                            if let Some(ref lockfile) = msg {
                                let mut lock = state.client.write().await;
                                if let Ok(client) = http::client(lockfile) {
                                    *lock = Some(client);
                                }
                            }
                            {
                            let mut lock = state.lockfile.write().await;
                            *lock = msg.clone();
                                }
                        }

                        match msg {
                            Some(lockfile) => {
                                _ = handle.emit("lcu-lockfile", lockfile);
                            }
                            None => {
                                _ = handle.emit("lcu-lockfile", ());
                            }
                        }
                    }
                }
            }
        }));

        Ok(())
    }
}
