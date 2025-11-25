//! Scarab Configuration System
//!
//! Provides Fusabi-based configuration with:
//! - Global config (~/.config/scarab/config.fsx)
//! - Per-directory overrides (.scarab.fsx)
//! - Hot-reload support (<100ms)
//! - Programmable F# DSL for dynamic configuration
//! - Validation with helpful errors
//! - Sensible defaults (zero-config startup)
//! - Plugin registry and marketplace
//!
//! Legacy TOML support is maintained for backwards compatibility.

pub mod config;
pub mod error;
pub mod fusabi_loader;
pub mod loader;
pub mod registry;
pub mod validation;
pub mod watcher;

pub use config::*;
pub use error::{ConfigError, Result};
pub use fusabi_loader::FusabiConfigLoader;
pub use loader::ConfigLoader;
pub use registry::{
    InstalledPlugin, PluginEntry, PluginFilter, RegistryCache, RegistryClient, RegistryConfig,
    RegistryManager, RegistryManifest, SecurityConfig,
};
pub use validation::ConfigValidator;
pub use watcher::ConfigWatcher;

/// Prelude for convenient imports
pub mod prelude {
    pub use crate::{
        config::*,
        error::{ConfigError, Result},
        loader::ConfigLoader,
        registry::{
            InstalledPlugin, PluginEntry, PluginFilter, RegistryCache, RegistryClient,
            RegistryConfig, RegistryManager, RegistryManifest, SecurityConfig,
        },
        validation::ConfigValidator,
        watcher::ConfigWatcher,
    };
}
