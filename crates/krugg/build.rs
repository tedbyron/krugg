#![deny(clippy::all, clippy::nursery, rust_2018_idioms)]
#![forbid(unsafe_code)]

fn main() {
    tauri_build::build();
}
