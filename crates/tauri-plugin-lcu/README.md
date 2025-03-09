# tauri-plugin-lcu

League of Legends client (LCU) API client for tauri.

- Requires `tauri-plugin-http` and `tauri-plugin-shell`
- `tauri-plugin-store` is optional and used to store LCU lockfile path

## Usage

- Available commands: see [`build.rs`](./build.rs)
- Emitted messages:
  - `lcu-lockfile` with value of `lockfile::LockFile` or `null` if the game
    client isn't running
  - `lcu-connected` if the game client is running and an http client was
    successfully created to connect to it
  - `lcu-disconnected` if the game client isn't running

### Setup

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
