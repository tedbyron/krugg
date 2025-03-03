#![warn(clippy::all, clippy::cargo, clippy::nursery, rust_2018_idioms)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    league_thingy_lib::run();
}
