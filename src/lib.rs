use std::path::PathBuf;

use thiserror::Error;

pub mod config;
pub mod ui;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Configuration error: {0}")]
    CfgError(#[from] ConfigError),
}

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Failed to read config file: {source}: {path}")]
    Io {
        #[source]
        source: std::io::Error,
        path: PathBuf,
    },
    #[error("Failed to store config file: {source}: {path}")]
    SerializeToml {
        #[source]
        source: toml::ser::Error,
        path: PathBuf,
    },
    #[error("Failed to parse config file: {source}: {path}")]
    DeserializeToml {
        #[source]
        source: toml::de::Error,
        path: PathBuf,
    },
    #[error("Missing field in config file: {0}: {1}")]
    MissingField(&'static str, PathBuf),
}

pub type AppResult<T> = Result<T, AppError>;
