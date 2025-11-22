//! Core plugin trait and metadata definitions

use crate::{context::PluginContext, error::Result, types::Action};
use async_trait::async_trait;

/// Main plugin trait that all plugins must implement
///
/// Plugins can hook into various events in the terminal lifecycle.
/// All methods have default implementations that do nothing.
#[async_trait]
pub trait Plugin: Send + Sync {
    /// Get plugin metadata
    fn metadata(&self) -> &PluginMetadata;

    /// Called when the plugin is loaded
    ///
    /// This is where plugins should initialize their state and resources.
    async fn on_load(&mut self, _ctx: &mut PluginContext) -> Result<()> {
        Ok(())
    }

    /// Called when the plugin is being unloaded
    ///
    /// Plugins should clean up resources here.
    async fn on_unload(&mut self) -> Result<()> {
        Ok(())
    }

    /// Hook called before output is displayed to the terminal
    ///
    /// Plugins can modify, block, or pass through the output.
    async fn on_output(&mut self, _line: &str, _ctx: &PluginContext) -> Result<Action> {
        Ok(Action::Continue)
    }

    /// Hook called after input is received from the user
    ///
    /// Plugins can intercept and modify input before it reaches the PTY.
    async fn on_input(&mut self, _input: &[u8], _ctx: &PluginContext) -> Result<Action> {
        Ok(Action::Continue)
    }

    /// Hook called before a command is executed
    async fn on_pre_command(&mut self, _command: &str, _ctx: &PluginContext) -> Result<Action> {
        Ok(Action::Continue)
    }

    /// Hook called after a command completes
    async fn on_post_command(
        &mut self,
        _command: &str,
        _exit_code: i32,
        _ctx: &PluginContext,
    ) -> Result<()> {
        Ok(())
    }

    /// Hook called when terminal is resized
    async fn on_resize(&mut self, _cols: u16, _rows: u16, _ctx: &PluginContext) -> Result<()> {
        Ok(())
    }

    /// Hook called when a client attaches to the session
    async fn on_attach(&mut self, _client_id: u64, _ctx: &PluginContext) -> Result<()> {
        Ok(())
    }

    /// Hook called when a client detaches from the session
    async fn on_detach(&mut self, _client_id: u64, _ctx: &PluginContext) -> Result<()> {
        Ok(())
    }
}

/// Plugin metadata
#[derive(Debug, Clone)]
pub struct PluginMetadata {
    /// Plugin name (must be unique)
    pub name: String,
    /// Plugin version (semver)
    pub version: String,
    /// Short description
    pub description: String,
    /// Author name
    pub author: String,
    /// Homepage URL
    pub homepage: Option<String>,
    /// API version this plugin was built against
    pub api_version: String,
    /// Minimum Scarab version required
    pub min_scarab_version: String,
}

impl PluginMetadata {
    /// Create new plugin metadata
    pub fn new(
        name: impl Into<String>,
        version: impl Into<String>,
        description: impl Into<String>,
        author: impl Into<String>,
    ) -> Self {
        Self {
            name: name.into(),
            version: version.into(),
            description: description.into(),
            author: author.into(),
            homepage: None,
            api_version: crate::API_VERSION.to_string(),
            min_scarab_version: "0.1.0".to_string(),
        }
    }

    /// Set homepage URL
    pub fn with_homepage(mut self, homepage: impl Into<String>) -> Self {
        self.homepage = Some(homepage.into());
        self
    }

    /// Set API version
    pub fn with_api_version(mut self, version: impl Into<String>) -> Self {
        self.api_version = version.into();
        self
    }

    /// Set minimum Scarab version
    pub fn with_min_scarab_version(mut self, version: impl Into<String>) -> Self {
        self.min_scarab_version = version.into();
        self
    }

    /// Check if this plugin is compatible with the current API version
    pub fn is_compatible(&self, current_api_version: &str) -> bool {
        use semver::Version;

        let Ok(plugin_version) = Version::parse(&self.api_version) else {
            return false;
        };

        let Ok(current_version) = Version::parse(current_api_version) else {
            return false;
        };

        // Compatible if major versions match and plugin minor <= current minor
        plugin_version.major == current_version.major
            && plugin_version.minor <= current_version.minor
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_compatibility() {
        let meta = PluginMetadata::new("test", "1.0.0", "Test plugin", "Test Author")
            .with_api_version("0.1.0");

        assert!(meta.is_compatible("0.1.0"));
        assert!(meta.is_compatible("0.2.0"));
        assert!(!meta.is_compatible("1.0.0"));
        assert!(!meta.is_compatible("0.0.1"));
    }
}
