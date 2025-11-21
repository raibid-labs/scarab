//! Error types for plugin operations

use thiserror::Error;

#[derive(Error, Debug)]
pub enum PluginError {
    #[error("Plugin failed to load: {0}")]
    LoadError(String),

    #[error("Plugin version incompatible: required {required}, got {actual}")]
    VersionIncompatible { required: String, actual: String },

    #[error("Plugin hook execution timed out after {0}ms")]
    Timeout(u64),

    #[error("Plugin panic: {0}")]
    Panic(String),

    #[error("Plugin disabled due to repeated failures")]
    Disabled,

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("TOML parse error: {0}")]
    TomlError(#[from] toml::de::Error),

    #[error("Invalid plugin metadata: {0}")]
    InvalidMetadata(String),

    #[error("Plugin not found: {0}")]
    NotFound(String),

    #[error("Other error: {0}")]
    Other(#[from] anyhow::Error),
}

pub type Result<T> = std::result::Result<T, PluginError>;
