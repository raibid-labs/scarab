//! Plugin manifest schema and validation
//!
//! This module defines the plugin manifest format that plugins must provide
//! to declare their capabilities, dependencies, and requirements.

use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use thiserror::Error;

/// Plugin manifest validation errors
#[derive(Debug, Error)]
pub enum ManifestError {
    #[error("Missing required field: {0}")]
    MissingField(&'static str),

    #[error("Invalid API version: {0}")]
    InvalidApiVersion(String),

    #[error("Unsupported capability: {0}")]
    UnsupportedCapability(String),

    #[error("Missing required module: {0}")]
    MissingModule(String),

    #[error("Manifest validation failed: {0}")]
    ValidationFailed(String),
}

/// Plugin manifest that declares plugin capabilities and requirements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginManifest {
    /// Plugin name (must be unique)
    pub name: String,

    /// Plugin version (semver)
    pub version: String,

    /// Short description
    pub description: String,

    /// Author name
    pub author: String,

    /// Homepage URL
    #[serde(skip_serializing_if = "Option::is_none")]
    pub homepage: Option<String>,

    /// API version this plugin requires
    #[serde(rename = "api-version")]
    pub api_version: String,

    /// Minimum Scarab version required
    #[serde(rename = "min-scarab-version")]
    pub min_scarab_version: String,

    /// Required capabilities
    #[serde(default)]
    pub capabilities: HashSet<Capability>,

    /// Required modules from fusabi-stdlib-ext
    #[serde(default, rename = "required-modules")]
    pub required_modules: HashSet<FusabiModule>,

    /// Optional visual metadata
    #[serde(skip_serializing_if = "Option::is_none")]
    pub emoji: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub catchphrase: Option<String>,
}

/// Plugin capabilities that must be declared
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Capability {
    /// Can intercept and modify terminal output
    OutputFiltering,

    /// Can intercept and modify user input
    InputFiltering,

    /// Can execute shell commands
    ShellExecution,

    /// Can read/write files
    FileSystem,

    /// Can make network requests
    Network,

    /// Can access clipboard
    Clipboard,

    /// Can spawn processes
    ProcessSpawn,

    /// Can modify terminal state
    TerminalControl,

    /// Can draw overlays on client UI
    UiOverlay,

    /// Can register menu items
    MenuRegistration,

    /// Can register commands in command palette
    CommandRegistration,
}

/// Fusabi stdlib modules that plugins can depend on
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FusabiModule {
    /// Terminal I/O operations
    Terminal,

    /// GPU rendering utilities
    Gpu,

    /// File system operations
    Fs,

    /// Network operations
    Net,

    /// Process management
    Process,

    /// Text processing utilities
    Text,

    /// JSON/TOML parsing
    Config,
}

impl PluginManifest {
    /// Validate the manifest against current API version and available capabilities
    pub fn validate(&self, current_api_version: &str) -> Result<(), ManifestError> {
        // Check API version compatibility
        use semver::Version;

        let plugin_version = Version::parse(&self.api_version)
            .map_err(|_| ManifestError::InvalidApiVersion(self.api_version.clone()))?;

        let current_version = Version::parse(current_api_version)
            .map_err(|_| ManifestError::InvalidApiVersion(current_api_version.to_string()))?;

        // Major version must match
        if plugin_version.major != current_version.major {
            return Err(ManifestError::ValidationFailed(format!(
                "API major version mismatch: plugin requires {}, current is {}",
                plugin_version.major, current_version.major
            )));
        }

        // Plugin minor version must not exceed current
        if plugin_version.minor > current_version.minor {
            return Err(ManifestError::ValidationFailed(format!(
                "Plugin requires API version {}.{}, but current is {}.{}",
                plugin_version.major,
                plugin_version.minor,
                current_version.major,
                current_version.minor
            )));
        }

        Ok(())
    }

    /// Check if the plugin declares a specific capability
    pub fn has_capability(&self, capability: &Capability) -> bool {
        self.capabilities.contains(capability)
    }

    /// Check if the plugin requires a specific module
    pub fn requires_module(&self, module: &FusabiModule) -> bool {
        self.required_modules.contains(module)
    }

    /// Get all required capabilities as a sorted list
    pub fn capabilities_list(&self) -> Vec<Capability> {
        let mut caps: Vec<_> = self.capabilities.iter().cloned().collect();
        caps.sort_by(|a, b| format!("{:?}", a).cmp(&format!("{:?}", b)));
        caps
    }

    /// Get all required modules as a sorted list
    pub fn modules_list(&self) -> Vec<FusabiModule> {
        let mut mods: Vec<_> = self.required_modules.iter().cloned().collect();
        mods.sort_by(|a, b| format!("{:?}", a).cmp(&format!("{:?}", b)));
        mods
    }
}

impl Default for PluginManifest {
    fn default() -> Self {
        Self {
            name: String::new(),
            version: "0.1.0".to_string(),
            description: String::new(),
            author: String::new(),
            homepage: None,
            api_version: crate::API_VERSION.to_string(),
            min_scarab_version: "0.1.0".to_string(),
            capabilities: HashSet::new(),
            required_modules: HashSet::new(),
            emoji: None,
            color: None,
            catchphrase: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_manifest_validation_compatible() {
        let manifest = PluginManifest {
            name: "test-plugin".to_string(),
            version: "1.0.0".to_string(),
            description: "Test".to_string(),
            author: "Test Author".to_string(),
            homepage: None,
            api_version: "0.1.0".to_string(),
            min_scarab_version: "0.1.0".to_string(),
            capabilities: HashSet::new(),
            required_modules: HashSet::new(),
            emoji: None,
            color: None,
            catchphrase: None,
        };

        assert!(manifest.validate("0.1.0").is_ok());
        assert!(manifest.validate("0.2.0").is_ok());
    }

    #[test]
    fn test_manifest_validation_incompatible() {
        let manifest = PluginManifest {
            api_version: "1.0.0".to_string(),
            ..Default::default()
        };

        assert!(manifest.validate("0.1.0").is_err());
    }

    #[test]
    fn test_capability_checking() {
        let mut manifest = PluginManifest::default();
        manifest.capabilities.insert(Capability::OutputFiltering);
        manifest.capabilities.insert(Capability::FileSystem);

        assert!(manifest.has_capability(&Capability::OutputFiltering));
        assert!(manifest.has_capability(&Capability::FileSystem));
        assert!(!manifest.has_capability(&Capability::Network));
    }

    #[test]
    fn test_module_requirements() {
        let mut manifest = PluginManifest::default();
        manifest.required_modules.insert(FusabiModule::Terminal);
        manifest.required_modules.insert(FusabiModule::Fs);

        assert!(manifest.requires_module(&FusabiModule::Terminal));
        assert!(manifest.requires_module(&FusabiModule::Fs));
        assert!(!manifest.requires_module(&FusabiModule::Net));
    }

    #[test]
    fn test_toml_serialization() {
        let mut manifest = PluginManifest {
            name: "example-plugin".to_string(),
            version: "1.0.0".to_string(),
            description: "An example plugin".to_string(),
            author: "Example Author".to_string(),
            homepage: Some("https://example.com".to_string()),
            api_version: "0.1.0".to_string(),
            min_scarab_version: "0.1.0".to_string(),
            capabilities: HashSet::new(),
            required_modules: HashSet::new(),
            emoji: Some("🔌".to_string()),
            color: Some("#FF5733".to_string()),
            catchphrase: Some("Power to the plugins!".to_string()),
        };

        manifest.capabilities.insert(Capability::OutputFiltering);
        manifest.capabilities.insert(Capability::UiOverlay);
        manifest.required_modules.insert(FusabiModule::Terminal);

        let toml = toml::to_string_pretty(&manifest).unwrap();
        let deserialized: PluginManifest = toml::from_str(&toml).unwrap();

        assert_eq!(manifest.name, deserialized.name);
        assert_eq!(manifest.version, deserialized.version);
        assert_eq!(manifest.capabilities, deserialized.capabilities);
        assert_eq!(manifest.required_modules, deserialized.required_modules);
    }
}
