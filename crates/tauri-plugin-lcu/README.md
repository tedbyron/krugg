# tauri-plugin-lcu

League of Legends client (LCU) API client for tauri.

- Requires `tauri-plugin-http` and `tauri-plugin-shell`
- `tauri-plugin-store` is optional and used to store LCU lockfile path

## Usage

- Available commands: see [`build.rs`](./build.rs)
- Emitted messages:
  - `lcu-connected` with a boolean payload, true if the game client is running and an http client
    was successfully created to connect to it
  - `lcu-lockfile` with a [`crate::lockfile::LockFile`](https://docs.rs/tauri-plugin-lcu/latest/tauri-plugin-lcu/lockfile/struct.LockFile.html)
    payload if the game client is running
  - `lcu-base-url` with a [`url::Url`](https://docs.rs/url/latest/url/struct.Url.html) payload,
    including protocol, hostname, and port, if the game client is running

### Setup

```jsonc
// src-tauri/capabilities/default.json
{
  // ...
  "permissions": [
    "http:default",
    {
      "identifier": "shell:allow-execute",
      "allow": [
        // Windows
        {
          "name": "wmic",
          "cmd": "WMIC.exe",
          "args": ["process", "WHERE", "Name='LeagueClientUx.exe'", "GET", "CommandLine"],
        },
        // macOS
        {
          "name": "ps",
          "cmd": "ps",
          "args": ["-xo", "args="],
        },
      ],
    },
  ],
}
```

```rs
//! src-tauri/lib.rs
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_http::init()) // Required
        .plugin(tauri_plugin_shell::init()) // Required
        .plugin(tauri_plugin_store::Builder::default().build()) // Optional
        // Initialize LCU plugin after other plugins.
        .plugin(tauri_plugin_lcu::init(Some("my_store_path.json")))
        .run(tauri::generate_context!())
        .expect("error while running the application");
}
```

### Call LCU APIs

#### Rust

```rs
//! src-tauri/my_command.rs
use tauri_plugin_lcu::LcuExt;

#[tauri::command]
// TODO
```

#### JS

<!-- TODO -->
