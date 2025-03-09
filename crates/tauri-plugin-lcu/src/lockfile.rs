use std::{
    fs,
    net::Ipv4Addr,
    path::{Path, PathBuf},
    sync::LazyLock,
};

use base64ct::{Base64, Encoding};
use notify_debouncer_full::{
    DebounceEventResult,
    notify::{EventKind, RecursiveMode},
};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, Manager, Runtime, async_runtime};
use tauri_plugin_http::reqwest::Url;
use tauri_plugin_shell::ShellExt;
#[cfg(feature = "tauri-plugin-store")]
use tauri_plugin_store::{JsonValue, StoreExt};
use tokio::{
    task,
    time::{self, Duration},
};

use crate::LcuState;

/// LCU API username.
const USERNAME: &str = "riot";
/// LCU API base URL without port.
static BASE_URL: LazyLock<Url> =
    LazyLock::new(|| Url::parse(&format!("https://{}", Ipv4Addr::LOCALHOST)).unwrap());

/// LCU lockfile.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LockFile {
    /// Path to the lockfile.
    pub path: PathBuf,
    /// League client process ID.
    pub pid: u32,
    /// HTTP port.
    pub port: u16,
    /// LCU API base URL, including protocol, hostname, and port.
    pub base_url: Url,
    /// HTTP auth password.
    pub token: String,
    /// HTTP basic auth header value.
    pub auth_header: String,
}

impl LockFile {
    /// Retrieve the lockfile path from the store.
    fn path_from_store<R: Runtime>(app: &AppHandle<R>) -> Option<PathBuf> {
        #[cfg(feature = "tauri-plugin-store")]
        {
            let state = app.state::<LcuState>();
            if let Some(store_file) = &state.store_file {
                if let Ok(store) = app.store(store_file) {
                    if let Some(JsonValue::String(path)) = store.get("lockfile_path") {
                        return Some(path.into());
                    }
                }
            }
        }

        None
    }

    /// Retrieve the lockfile path from the running League client.
    #[cfg(target_os = "windows")]
    async fn path<R: Runtime>(app: &AppHandle<R>) -> crate::Result<PathBuf> {
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
            return Err(crate::Error::Command(output.status.code()));
        }

        let cmd = str::from_utf8(&output.stdout)?;
        if cmd.trim_ascii().is_empty() {
            return Err(crate::Error::ParseCommand);
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
        let exe_path = Path::new(argv.first().ok_or(crate::Error::ParseCommand)?);
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

    /// Parse the lockfile contents. Saves the lockfile path to the store if
    /// the plugin state's `store_file` field is set.
    fn parse<R: Runtime>(app: &AppHandle<R>, path: impl AsRef<Path>) -> Option<Self> {
        let path = path.as_ref();
        let lockfile = fs::read_to_string(path).ok()?;
        let parts = lockfile.split(':').collect::<Box<[_]>>();
        let pid = parts[1].parse::<u32>().ok()?;
        let port = parts[2].parse::<u16>().ok()?;
        let token = parts[3].to_owned();

        let mut base_url = BASE_URL.clone();
        base_url.set_port(Some(port)).ok()?;
        let auth_header = format!(
            "Basic {}",
            Base64::encode_string(format!("{USERNAME}:{token}").as_bytes())
        );

        #[cfg(feature = "tauri-plugin-store")]
        {
            let state = app.state::<LcuState>();
            if let Some(store_file) = &state.store_file {
                if let Ok(store) = app.store(store_file) {
                    store.set("lockfile_path", path.to_str().unwrap());
                }
            }
        }

        Some(Self {
            path: path.to_owned(),
            pid,
            port,
            base_url,
            token,
            auth_header,
        })
    }

    /// Watch for file system changes to the LCU lockfile.
    pub fn watch<R: Runtime>(app: &AppHandle<R>) -> crate::Result<()> {
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
        if let Some(lockfile) = Self::parse(app, &path) {
            _ = app.emit("lcu-lockfile", lockfile.clone());
            let state = app.state::<LcuState>();
            {
                let mut lock = state.client.blocking_lock();
                *lock = Some(crate::http::client(&lockfile)?);
            }
            {
                let mut lock = state.lockfile.blocking_write();
                *lock = Some(lockfile);
            }
        }

        // Spawn a background task to update state when the lockfile changes.
        let state = app.state::<LcuState>();
        let cancel_token = state.cancel_token.clone();
        let app = app.clone();
        async_runtime::spawn(state.tracker.track_future(async move {
            // Watch for changes to the lockfile using the debounced watcher
            // so the lockfile isn't read until it has contents. The channel
            // and watcher need to live for the duration of the task.
            let (tx, mut rx) = async_runtime::channel(1);
            let p = path.clone();
            let handle = app.clone();
            let mut watcher = notify_debouncer_full::new_debouncer(
                Duration::from_secs(1),
                None,
                move |res: DebounceEventResult| {
                    if let Ok(events) = res {
                        if let Some(evt) = events.first() {
                            match evt.kind {
                                EventKind::Modify(_) => {
                                    _ = tx.blocking_send(Self::parse(&handle, &p));
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

            let state = app.state::<LcuState>();
            loop {
                tokio::select! {
                    biased;
                    // Unwatch the path, close and drain the channel, and break
                    // from the loop.
                    () = cancel_token.cancelled() => {
                        _ = watcher.unwatch(&path);
                        rx.close();
                        while rx.recv().await.is_some() {}
                        break;
                    }
                    // Update state and emit the lockfile to the frontend
                    // on change.
                    Some(ref msg) = rx.recv() => {
                        if let Some(lockfile) = msg {
                            if Some(lockfile) != state.lockfile.read().await.as_ref() {
                                {
                                    let mut lock = state.lockfile.write().await;
                                    (*lock).clone_from(msg);
                                }
                                _ = app.emit("lcu-lockfile", lockfile);
                                if let Ok(client) = crate::http::client(lockfile) {
                                    {
                                        let mut lock = state.client.lock().await;
                                        *lock = Some(client);
                                    }
                                    _ = app.emit("lcu-connected", ());
                                }
                            }
                        } else {
                            _ = app.emit("lcu-lockfile", ());
                            _ = app.emit("lcu-disconnected", ());
                        }
                    }
                }
            }
        }));

        Ok(())
    }
}
