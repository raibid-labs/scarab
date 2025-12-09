//! Provides Fusabi-based configuration with:
//! - .fsx script loading
//! - Direct host function integration
//! - Type-safe configuration structs
//! - Bevy plugin for asset-based hot-reloading

pub mod config;
pub mod error;
pub mod fusabi_loader;
pub mod loader;
pub mod plugin;
pub mod registry;
pub mod theme_resolver;
pub mod validation;
pub mod watcher;

pub use config::{
    ColorConfig, ColorPalette, CursorStyle, EffectsConfig, FontConfig, KeyBindings, NavConfig,
    NavStyle, PluginConfig, ScarabConfig, SessionConfig, SshAuthConfig, SshDomainConfig,
    TabPosition, TerminalConfig, UiConfig,
};
pub use error::{ConfigError, Result};
pub use fusabi_loader::FusabiConfigLoader;
pub use loader::ConfigLoader;
pub use plugin::{ConfigHandle, ScarabConfigPlugin};
pub use registry::{PluginFilter, RegistryManager};
pub use theme_resolver::ThemeResolver;
pub use validation::ConfigValidator;
pub use watcher::ConfigWatcher;

pub mod prelude {
    pub use crate::config::*;
    pub use crate::error::*;
    pub use crate::fusabi_loader::*;
    pub use crate::loader::*;
    pub use crate::plugin::*;
    pub use crate::registry::*;
    pub use crate::theme_resolver::*;
    pub use crate::validation::*;
    pub use crate::watcher::*;
}
