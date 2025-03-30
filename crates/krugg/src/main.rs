#![deny(clippy::all, clippy::nursery, rust_2018_idioms)]
#![forbid(unsafe_code)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    krugg_lib::run();
}
