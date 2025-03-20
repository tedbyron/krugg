use serde::{Serialize, ser::Serializer};
use tauri_plugin_http::reqwest::StatusCode;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Utf8(#[from] std::str::Utf8Error),
    // #[error(transparent)]
    // ParseInt(#[from] std::num::ParseIntError),
    #[error(transparent)]
    Shell(#[from] tauri_plugin_shell::Error),
    #[error("command output status code {0:?}")]
    Command(Option<i32>),
    #[error("parsing command output")]
    ParseCommand,
    #[error(transparent)]
    Tauri(#[from] tauri::Error),
    #[error(transparent)]
    Store(#[from] tauri_plugin_store::Error),
    #[error(transparent)]
    UrlParse(#[from] url::ParseError),
    #[error(transparent)]
    Reqwest(#[from] tauri_plugin_http::reqwest::Error),
    #[error(transparent)]
    InvalidHeaderValue(#[from] tauri_plugin_http::reqwest::header::InvalidHeaderValue),
    #[error("request failed with status {status}: {text}")]
    StatusCode { status: StatusCode, text: String },
    #[error("not connected to the LCU")]
    Disconnected,
    #[error("{0}")]
    Custom(&'static str),
}

impl Serialize for Error {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.to_string().as_ref())
    }
}
