//! Error types for configuration system

use std::path::PathBuf;
use thiserror::Error;

pub type Result<T> = std::result::Result<T, ConfigError>;

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("TOML parsing error: {0}")]
    TomlParse(#[from] toml::de::Error),

    #[error("TOML serialization error: {0}")]
    TomlSerialize(#[from] toml::ser::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Invalid font size: {0} (must be between 6.0 and 72.0)")]
    InvalidFontSize(f32),

    #[error("Invalid scrollback lines: {0} (must be between 100 and 100,000)")]
    InvalidScrollback(u32),

    #[error("Invalid color format: {0} (expected #RRGGBB)")]
    InvalidColor(String),

    #[error("Invalid line height: {0} (must be between 0.5 and 3.0)")]
    InvalidLineHeight(f32),

    #[error("Config file not found at {0}")]
    FileNotFound(PathBuf),

    #[error("Invalid theme name: {0}")]
    InvalidTheme(String),

    #[error("Invalid shell command: {0}")]
    InvalidShell(String),

    #[error("Watch error: {0}")]
    Watch(#[from] notify::Error),

    #[error("Plugin config error: {0}")]
    PluginConfig(String),

    #[error("Migration error: {0}")]
    Migration(String),

    #[error("Validation error: {0}")]
    Validation(String),
}

impl ConfigError {
    /// Create a helpful error message with suggestions
    pub fn help_text(&self) -> String {
        match self {
            ConfigError::InvalidFontSize(size) => {
                format!(
                    "Font size {} is out of range.\n\
                     Valid range: 6.0 to 72.0\n\
                     Suggestion: Try 12.0 or 14.0 for readability",
                    size
                )
            }
            ConfigError::InvalidColor(color) => {
                format!(
                    "Color '{}' is not valid.\n\
                     Expected format: #RRGGBB (e.g., #FF5555)\n\
                     You can use color names in theme presets instead",
                    color
                )
            }
            ConfigError::InvalidScrollback(lines) => {
                format!(
                    "Scrollback {} lines is out of range.\n\
                     Valid range: 100 to 100,000\n\
                     Suggestion: Try 10,000 for good balance",
                    lines
                )
            }
            ConfigError::FileNotFound(path) => {
                format!(
                    "Config file not found: {}\n\
                     Run 'scarab config init' to create a default config\n\
                     Or create it manually at ~/.config/scarab/config.toml",
                    path.display()
                )
            }
            _ => format!("{}", self),
        }
    }
}
