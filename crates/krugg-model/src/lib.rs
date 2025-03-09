#![warn(clippy::all, clippy::nursery, rust_2018_idioms)]
//! Shared data and types.

pub use ugg_types::*;

pub const STORE_FILE: &str = "app_data.json";

/// Tauri managed app state.
#[derive(Debug)]
pub struct AppState {}
