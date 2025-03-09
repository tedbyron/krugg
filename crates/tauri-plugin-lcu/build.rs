#![warn(clippy::all, clippy::nursery, rust_2018_idioms)]

#[cfg(not(feature = "ugg-types"))]
const COMMANDS: &[&str] = &["get", "head", "post", "put", "delete", "patch"];
#[cfg(feature = "ugg-types")]
const COMMANDS: &[&str] = &[
    "get",
    "head",
    "post",
    "put",
    "delete",
    "patch",
    "get_current_summoner",
    "get_current_rune_page",
    "update_rune_page",
];

fn main() {
    tauri_plugin::Builder::new(COMMANDS).build();
}
