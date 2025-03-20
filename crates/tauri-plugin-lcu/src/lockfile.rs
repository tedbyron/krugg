use std::{
    fs,
    net::Ipv4Addr,
    path::{Path, PathBuf},
};

use base64ct::{Base64, Encoding};
use notify_debouncer_full::{
    DebounceEventResult, Debouncer,
    notify::{self, EventKind, RecursiveMode},
};
use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager, Runtime, async_runtime};
use tauri_plugin_http::reqwest::Url;
use tauri_plugin_shell::ShellExt;
#[cfg(feature = "tauri-plugin-store")]
use tauri_plugin_store::{JsonValue, StoreExt};
use tokio::time::{self, Duration};

use crate::LcuState;

/// LCU API username.
const USERNAME: &str = "riot";

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LockFile {
    /// Path to the lockfile.
    pub path: PathBuf,
    /// League client process ID.
    pub pid: u32,
    /// HTTP port.
    pub port: u16,
    /// HTTP auth password.
    pub token: String,
    /// HTTP basic auth header value.
    pub auth_header: String,
}

type Watcher = Debouncer<notify::RecommendedWatcher, notify_debouncer_full::RecommendedCache>;
type Receiver = async_runtime::Receiver<Option<(LockFile, Url)>>;

impl LockFile {
    /// Retrieve the lockfile path from the store.
    fn path_from_store<R: Runtime>(app: &AppHandle<R>) -> Option<PathBuf> {
        #[cfg(feature = "tauri-plugin-store")]
        {
            let state = app.state::<LcuState>();
            if let Ok(store) = app.store(&state.store_file) {
                if let Some(JsonValue::String(path)) = store.get("lockfile_path") {
                    return Some(path.into());
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
    async fn path<R: Runtime>(app: &AppHandle<R>) -> crate::Result<PathBuf> {
        let shell = app.shell();
        let output = shell.command("ps").args(["-xo", "args="]).output().await?;
        if output.status.code() != Some(0) {
            return Err(crate::Error::Command(output.status.code()));
        }

        let cmd = str::from_utf8(&output.stdout)?
            .lines()
            .filter_map(|line| {
                if line.contains("LeagueClientUx") {
                    Some(line.trim_ascii())
                } else {
                    None
                }
            })
            .collect::<Box<[_]>>();
        let position = cmd
            .first()
            .ok_or(crate::Error::ParseCommand)?
            .find("LeagueClientUx")
            .ok_or(crate::Error::ParseCommand)?;
        let exe_dir = Path::new(&cmd[0][..position]);
        dbg!(&exe_dir);

        Ok(exe_dir.join("lockfile"))
    }

    /// Parse the lockfile contents. Saves the lockfile path to the store if
    /// the `tauri-plugin-store` feature is enabled.
    fn parse<R: Runtime>(app: &AppHandle<R>, path: impl AsRef<Path>) -> Option<(Self, Url)> {
        let path = path.as_ref();
        let lockfile = fs::read_to_string(path).ok()?;
        let parts = lockfile.split(':').collect::<Box<[_]>>();
        let pid = parts[1].parse::<u32>().ok()?;
        let port = parts[2].parse::<u16>().ok()?;
        let token = parts[3].to_owned();

        let mut base_url = Url::parse(&format!("https://{}", Ipv4Addr::LOCALHOST)).unwrap();
        base_url.set_port(Some(port)).ok()?;
        let auth_header = format!(
            "Basic {}",
            Base64::encode_string(format!("{USERNAME}:{token}").as_bytes())
        );

        #[cfg(feature = "tauri-plugin-store")]
        {
            let state = app.state::<LcuState>();
            if let Ok(store) = app.store(&state.store_file) {
                store.set("lockfile_path", path.to_str().unwrap());
            }
        }

        Some((
            Self {
                path: path.to_owned(),
                pid,
                port,
                token,
                auth_header,
            },
            base_url,
        ))
    }

    /// Watch for changes to the lockfile using the debounced watcher so the
    /// lockfile isn't read until it has contents. The channel and watcher need
    /// to live for the duration of the task.
    fn watcher<R: Runtime>(app: &AppHandle<R>, path: &Path) -> notify::Result<(Watcher, Receiver)> {
        let app = app.clone();
        let path = path.to_owned();
        let (tx, rx) = async_runtime::channel(1);
        let watcher = notify_debouncer_full::new_debouncer(
            Duration::from_secs(1),
            None,
            move |res: DebounceEventResult| {
                async_runtime::block_on(async {
                    if let Ok(events) = res {
                        for evt in events {
                            match evt.kind {
                                EventKind::Modify(_) => {
                                    _ = tx.send(Self::parse(&app, &path)).await;
                                    break;
                                }
                                EventKind::Remove(_) => {
                                    _ = tx.send(None).await;
                                    break;
                                }
                                _ => (),
                            }
                        }
                    }
                });
            },
        )?;

        Ok((watcher, rx))
    }

    /// Update `LcuState` `lockfile`, `base_url`, and `client` fields.
    async fn update_state<R: Runtime>(app: &AppHandle<R>, lockfile: Self, url: Url) {
        let state = app.state::<LcuState>();

        if let Ok(client) = crate::http::client(&lockfile) {
            {
                let mut lock = state.client.write().await;
                *lock = Some(client);
            }
            _ = app.emit("lcu-connected", ());
        }

        {
            _ = app.emit("lcu-base-url", &url.as_str());
            let mut lock = state.base_url.write().await;
            *lock = Some(url);
        }

        {
            _ = app.emit("lcu-lockfile", &lockfile);
            let mut lock = state.lockfile.write().await;
            *lock = Some(lockfile);
        }
    }

    /// Watch for file system changes to the LCU lockfile.
    pub fn watch<R: Runtime>(app: &AppHandle<R>) {
        // Get the lockfile path from the store or call Self::path every 5
        // seconds until it returns a path.
        let state = app.state::<LcuState>();
        let path = Self::path_from_store(app).unwrap_or_else(|| {
            // TODO: don't block main thread lol.
            tokio::task::block_in_place(move || {
                async_runtime::block_on(state.tracker.track_future(async {
                    let mut interval = time::interval(Duration::from_secs(5));

                    loop {
                        if let Ok(lockfile_path) = Self::path(app).await {
                            break lockfile_path;
                        }
                        interval.tick().await;
                    }
                }))
            })
        });

        // Update state if possible before starting the file watcher.
        let state = app.state::<LcuState>();
        if let Some((lockfile, url)) = Self::parse(app, &path) {
            async_runtime::block_on(state.tracker.track_future(async {
                Self::update_state(app, lockfile, url).await;
            }));
        }

        // Spawn a background task to update state when the lockfile changes.
        let cancel_token = state.cancel_token.clone();
        let app = app.clone();
        async_runtime::spawn(state.tracker.track_future(async move {
            let (mut watcher, mut rx) = Self::watcher(&app, &path).unwrap();
            // TODO: panics if the lockfile path is set but file doesn't exist
            //       yet (client not open).
            watcher.watch(&path, RecursiveMode::NonRecursive).unwrap();

            let state = app.state::<LcuState>();
            loop {
                tokio::select! {
                    biased;
                    // Unwatch the path and close the channel when canceled.
                    () = cancel_token.cancelled() => {
                        _ = watcher.unwatch(&path);
                        rx.close();
                        while rx.recv().await.is_some() {}
                        break;
                    }
                    // Update state when the lockfile is modified.
                    Some(msg) = rx.recv() => {
                        if let Some((lockfile, url)) = msg {
                            if Some(&lockfile) != state.lockfile.read().await.as_ref() {
                                Self::update_state(&app, lockfile, url).await;
                            }
                        } else {
                            _ = app.emit("lcu-disconnected", ());
                        }
                    }
                }
            }
        }));
    }
}
