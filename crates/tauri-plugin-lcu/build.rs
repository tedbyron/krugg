#![deny(clippy::all, clippy::nursery, rust_2018_idioms)]

#[cfg(not(feature = "ugg-types"))]
const COMMANDS: &[&str] = &["connected"];
#[cfg(feature = "ugg-types")]
const COMMANDS: &[&str] = &[
    "connected",
    "get_current_summoner",
    "get_current_rune_page",
    "update_rune_page",
];

fn main() {
    tauri_plugin::Builder::new(COMMANDS).build();
}
