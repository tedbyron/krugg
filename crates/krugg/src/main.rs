#![deny(clippy::all, clippy::nursery, rust_2018_idioms)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    krugg_lib::run();
}
