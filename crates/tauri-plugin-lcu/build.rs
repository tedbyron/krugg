#![warn(clippy::all, clippy::nursery, rust_2018_idioms)]

const COMMANDS: &[&str] = &[];

fn main() {
    tauri_plugin::Builder::new(COMMANDS).build();
}
