//! Scarab Plugin API
//!
//! This crate provides the core plugin API for the Scarab terminal emulator.
//! It defines traits, types, and utilities for building 3rd-party plugins.
//!
//! # ECS-Safe Host Bindings
//!
//! Fusabi plugins interact with Scarab's ECS through the [`host_bindings`] module,
//! which provides safe APIs with capability checks, quotas, and rate limiting.
//!
//! See [`host_bindings::HostBindings`] for the main entry point.

pub mod config;
pub mod context;
pub mod copy_mode;
pub mod delight;
pub mod error;
pub mod events;
pub mod host_bindings;
pub mod key_tables;
pub mod manifest;
pub mod menu;
pub mod navigation;
pub mod object_model;
pub mod plugin;
pub mod status_bar;
pub mod types;

pub use config::{PluginConfig, PluginDiscovery};
pub use context::PluginContext;
pub use copy_mode::{
    get_selection_bounds, normalize_selection, CopyModeCursor, CopyModeState, SearchDirection,
    SearchMatch, SearchState, Selection, SelectionMode,
};
pub use delight::{Achievement, PluginMood};
pub use error::{PluginError, Result};

// Note: EventRegistry is deprecated for client-side use. See events module docs for migration guide.
#[allow(deprecated)]
pub use events::{EventArgs, EventData, EventHandler, EventRegistry, EventResult, EventType, HandlerEntry};
pub use key_tables::{
    ActivateKeyTableMode, ClipboardKind, CopyModeAction, Direction, KeyAction, KeyCode, KeyCombo,
    KeyModifiers, KeyTable, KeyTableActivation, KeyTableStack, LeaderKeyConfig, LeaderKeyState,
    SearchAction, SplitDirection,
};
pub use manifest::{Capability, FusabiModule, ManifestError, PluginManifest};
pub use menu::{MenuAction, MenuItem};
pub use navigation::{
    validate_focusable, NavigationExt, PluginFocusable, PluginFocusableAction,
    PluginNavCapabilities, ValidationError,
};
pub use host_bindings::{
    HostBindings, HostBindingLimits, NavStyle, NavKeymap, ResourceUsage,
    DEFAULT_RATE_LIMIT, DEFAULT_MAX_FOCUSABLES, DEFAULT_MAX_OVERLAYS, DEFAULT_MAX_STATUS_ITEMS,
};
pub use object_model::{ObjectError, ObjectHandle, ObjectRegistry, ObjectType, RegistryEntry};
pub use plugin::{Plugin, PluginMetadata};
pub use status_bar::{
    AnsiColor, Color, RenderItem, StatusBarSide, StatusBarUpdate, UnderlineStyle,
};
pub use types::{Action, HookType, PluginInfo};

/// Current plugin API version
pub const API_VERSION: &str = "0.1.0";
