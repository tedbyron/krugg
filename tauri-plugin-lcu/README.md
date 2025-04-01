# tauri-plugin-lcu

League of Legends client (LCU) API client for tauri.

- Requires `tauri-plugin-http` and `tauri-plugin-shell`
- `tauri-plugin-store` is optional and used to store LCU lockfile path

## Usage

- Available commands: see [`build.rs`](./build.rs)
- Events:

  | name            | payload                                |
  | --------------- | -------------------------------------- |
  | `lcu-connected` | `boolean`                              |
  | `lcu-lockfile`  | [`LockFile`](./lib/index.ts) \| `null` |
  | `lcu-base-url`  | `string` \| `null`                     |

### Setup

```jsonc
// src-tauri/capabilities/default.json
{
  // ...
  "permissions": ["lcu:default"],
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

<!-- TODO -->

```rs
//! src-tauri/my_command.rs
use tauri_plugin_lcu::LcuExt;

#[tauri::command]
```

#### JS

<!-- TODO -->
