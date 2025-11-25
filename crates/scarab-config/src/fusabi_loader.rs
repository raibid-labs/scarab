//! Fusabi-based configuration loader
//!
//! Loads configuration from `.fsx` files using the Fusabi frontend.
//! This replaces the TOML-based configuration system with a programmable
//! F# DSL that allows dynamic configuration, hooks, and validation.

use crate::{config::*, error::*};
use fusabi_frontend::compile_program_from_source;
use std::path::Path;

/// Fusabi configuration loader
pub struct FusabiConfigLoader;

impl FusabiConfigLoader {
    /// Load configuration from a Fusabi script file
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<ScarabConfig> {
        let source = std::fs::read_to_string(path.as_ref())?;
        Self::from_source(&source)
    }

    /// Load configuration from Fusabi source code
    pub fn from_source(source: &str) -> Result<ScarabConfig> {
        // Compile the Fusabi source
        let _chunk = compile_program_from_source(source).map_err(|e| {
            ConfigError::FusabiCompileError(format!("Failed to compile config: {:?}", e))
        })?;

        // TODO: Execute the compiled config and extract configuration values
        // For now, return default config with a note that this is WIP
        println!("⚠️  Fusabi config loader is WIP - using defaults");
        println!("📝 Config file compiled successfully, but extraction not yet implemented");

        Ok(ScarabConfig::default())
    }

    /// Load configuration with fallback chain:
    /// 1. Try ~/.config/scarab/config.fsx
    /// 2. Fall back to default config
    pub fn load_with_fallback() -> Result<ScarabConfig> {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        let config_path = std::path::PathBuf::from(home)
            .join(".config/scarab/config.fsx");

        if config_path.exists() {
            println!("Loading Fusabi config from: {}", config_path.display());
            Self::from_file(config_path)
        } else {
            println!("No Fusabi config found, using defaults");
            println!("Create {} to customize your terminal", config_path.display());
            Ok(ScarabConfig::default())
        }
    }

    /// Extract terminal config from compiled Fusabi module
    fn extract_terminal_config(_module: &FusabiModule) -> Result<TerminalConfig> {
        // TODO: Query the module for [<TerminalConfig>] attribute values
        Ok(TerminalConfig::default())
    }

    /// Extract font config from compiled Fusabi module
    fn extract_font_config(_module: &FusabiModule) -> Result<FontConfig> {
        // TODO: Query the module for [<FontConfig>] attribute values
        Ok(FontConfig::default())
    }

    /// Extract color config from compiled Fusabi module
    fn extract_color_config(_module: &FusabiModule) -> Result<ColorConfig> {
        // TODO: Query the module for [<ColorConfig>] attribute values
        Ok(ColorConfig::default())
    }

    /// Extract keybindings from compiled Fusabi module
    fn extract_keybindings(_module: &FusabiModule) -> Result<KeyBindings> {
        // TODO: Query the module for [<KeyBindings>] attribute values
        Ok(KeyBindings::default())
    }

    /// Extract UI config from compiled Fusabi module
    fn extract_ui_config(_module: &FusabiModule) -> Result<UiConfig> {
        // TODO: Query the module for [<UiConfig>] attribute values
        Ok(UiConfig::default())
    }

    /// Extract plugin config from compiled Fusabi module
    fn extract_plugin_config(_module: &FusabiModule) -> Result<PluginConfig> {
        // TODO: Query the module for [<PluginConfig>] attribute values
        Ok(PluginConfig::default())
    }

    /// Extract session config from compiled Fusabi module
    fn extract_session_config(_module: &FusabiModule) -> Result<SessionConfig> {
        // TODO: Query the module for [<SessionConfig>] attribute values
        Ok(SessionConfig::default())
    }
}

/// Placeholder for Fusabi module representation
/// TODO: Replace with actual fusabi-vm module type
struct FusabiModule;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compile_simple_config() {
        let source = r#"
            module ScarabConfig
            open Scarab.Config

            [<TerminalConfig>]
            let terminal = {
                DefaultShell = "/bin/bash"
                ScrollbackLines = 5000
                Columns = 100
                Rows = 30
            }
        "#;

        let config = FusabiConfigLoader::from_source(source);
        assert!(config.is_ok(), "Config should compile successfully");
    }

    #[test]
    fn test_load_nonexistent_file() {
        let result = FusabiConfigLoader::from_file("/nonexistent/config.fsx");
        assert!(result.is_err(), "Should fail on nonexistent file");
    }

    #[test]
    fn test_load_with_fallback() {
        let config = FusabiConfigLoader::load_with_fallback();
        assert!(config.is_ok(), "Fallback should always succeed");
    }
}
