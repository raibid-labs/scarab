//! Scarab Plugin API
//!
//! This crate provides the core plugin API for the Scarab terminal emulator.
//! It defines traits, types, and utilities for building 3rd-party plugins.

pub mod config;
pub mod context;
pub mod delight;
pub mod error;
pub mod menu;
pub mod object_model;
pub mod plugin;
pub mod status_bar;
pub mod types;

pub use config::{PluginConfig, PluginDiscovery};
pub use context::PluginContext;
pub use delight::{Achievement, PluginMood};
pub use error::{PluginError, Result};
pub use menu::{MenuAction, MenuItem};
pub use object_model::{ObjectError, ObjectHandle, ObjectRegistry, ObjectType, RegistryEntry};
pub use plugin::{Plugin, PluginMetadata};
pub use status_bar::{
    AnsiColor, Color, RenderItem, StatusBarSide, StatusBarUpdate, UnderlineStyle,
};
pub use types::{Action, HookType, PluginInfo};

/// Current plugin API version
pub const API_VERSION: &str = "0.1.0";
