# tauri-plugin-lcu

League of Legends client (LCU) API client for tauri.

- Tauri plugin dependencies
  - `tauri-plugin-http`
  - `tauri-plugin-shell`
    - Uses `WMIC.exe` on Windows or `ps` on macOS to find the League install dir.
  - `tauri-plugin-store`
    - optional, used to store LCU lockfile path

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
        // Only specify a store path if using the `tauri-plugin-store` feature.
        .plugin(tauri_plugin_lcu::init("my_store_path.json"))
        .run(tauri::generate_context!())
        .expect("error while running the application");
}
```

### Call LCU APIs

Example to get the user from the client

#### Rust

```rs
use serde::{Serialize, Deserialize};
use tauri_plugin_lcu::LcuExt;

#[tauri::command]
pub async fn get_current_summoner(
    app: AppHandle,
    channel: Channel<ChannelMsg>
) -> crate::Result<()> {
    let summoner = app.lcu().get::<Summoner>("/lol-summoner/v1/current-summoner").await?;
    channel.send(ChannelMsg::Summoner(summoner))?;
    Ok(())
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase", tag = "type", content = "data")]
enum ChannelMsg {
  Summoner(Summoner)
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Summoner {
    pub account_id: i64,
    pub display_name: String,
    pub internal_name: String,
    pub name_change_flag: bool,
    pub percent_complete_for_next_level: i64,
    pub profile_icon_id: i64,
    pub puuid: String,
    pub reroll_points: RerollPoints,
    pub summoner_id: i64,
    pub summoner_level: i64,
    pub unnamed: bool,
    pub xp_since_last_level: i64,
    pub xp_until_next_level: i64,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RerollPoints {
    pub current_points: i64,
    pub max_rolls: i64,
    pub number_of_rolls: i64,
    pub points_cost_to_roll: i64,
    pub points_to_reroll: i64,
}
```

#### JS

```ts
import { invoke, Channel } from '@tauri-apps/api/core'

const channel = new Channel<ChannelMsg>()
let summoner: Summoner | undefined
channel.onmessage = ({ type, data }) => {
  if (type === 'summoner') {
    summoner = data
  }
}

await invoke('get_current_summoner', { channel })

type ChannelMsg = {
  type: 'summoner'
  data: Summoner
}

interface Summoner {
  accountId: number
  displayName: string
  internalName: string
  nameChangeFlag: boolean
  percentCompleteForNextLevel: number
  profileIconId: number
  puuid: string
  rerollPoints: RerollPoints
  summonerId: number
  summonerLevel: number
  unnamed: boolean
  xpSinceLastLevel: number
  xpUntilNextLevel: number
}

interface RerollPoints {
  currentPoints: number
  maxRolls: number
  numberOfRolls: number
  pointsCostToRoll: number
  pointsToReroll: number
}
```
