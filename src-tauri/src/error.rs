use serde::{Serialize, ser::Serializer};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Image(#[from] image::ImageError),
    #[error(transparent)]
    ReqwestMiddleware(#[from] reqwest_middleware::Error),
    #[error(transparent)]
    Tauri(#[from] tauri::Error),
    #[error(transparent)]
    Lcu(#[from] tauri_plugin_lcu::Error),
    #[error(transparent)]
    Reqwest(#[from] tauri_plugin_http::reqwest::Error),
    #[error(transparent)]
    Store(#[from] tauri_plugin_store::Error),
    #[error(transparent)]
    UrlParse(#[from] url::ParseError),

    #[error(transparent)]
    Ddragon(#[from] DdragonError),
    #[error(transparent)]
    Ugg(#[from] UggError),
}

#[derive(Debug, thiserror::Error)]
pub enum DdragonError {
    #[error("Failed to get the latest API version")]
    NoLatestVersion,
    #[error("Failed to get champion data for id: {0}")]
    NoChampionData(String),
}

#[derive(Debug, thiserror::Error)]
pub enum UggError {
    #[error("Failed to find a supported game version")]
    NoSupportedVersions,
    #[error("Missing region or rank entry")]
    MissingRegionOrRank,
    #[error("Missing role entry")]
    MissingRole,
}

macro_rules! impl_serialize_err {
    ( $($ty:ty),* $(,)? ) => {
        $(
            impl Serialize for $ty {
                fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
                where
                    S: Serializer,
                {
                    serializer.serialize_str(&self.to_string())
                }
            }
        )*
    };
}

impl_serialize_err![Error, DdragonError, UggError];
