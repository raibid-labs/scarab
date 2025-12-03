//! Navigation API for plugins
//!
//! This module provides the navigation extension API that allows plugins to:
//! - Enter and exit navigation modes (hint mode)
//! - Register custom focusable regions in terminal content
//! - Trigger navigation actions
//!
//! # Example
//!
//! ```ignore
//! use scarab_plugin_api::navigation::{NavigationExt, PluginFocusable, PluginFocusableAction};
//!
//! fn my_plugin_hook(ctx: &PluginContext) -> Result<()> {
//!     // Register a focusable URL in terminal content
//!     ctx.register_focusable(PluginFocusable {
//!         x: 10,
//!         y: 5,
//!         width: 20,
//!         height: 1,
//!         label: "GitHub".to_string(),
//!         action: PluginFocusableAction::OpenUrl("https://github.com".to_string()),
//!     })?;
//!
//!     // Enter hint mode programmatically
//!     ctx.enter_hint_mode()?;
//!
//!     Ok(())
//! }
//! ```

use crate::error::Result;

/// Navigation extension trait for plugin contexts
///
/// Provides navigation-related operations that plugins can perform,
/// such as entering/exiting navigation modes and registering focusable regions.
///
/// This trait is automatically implemented for `PluginContext` when the
/// navigation feature is enabled.
pub trait NavigationExt {
    /// Enter hint mode to display navigation hints
    ///
    /// This triggers the hint mode UI, displaying labels for all focusable
    /// elements in the terminal (URLs, file paths, registered regions, etc.).
    ///
    /// # Example
    /// ```ignore
    /// ctx.enter_hint_mode()?;
    /// ```
    fn enter_hint_mode(&self) -> Result<()>;

    /// Exit navigation mode and return to normal mode
    ///
    /// Clears all hint labels and returns input handling to normal mode.
    ///
    /// # Example
    /// ```ignore
    /// ctx.exit_nav_mode()?;
    /// ```
    fn exit_nav_mode(&self) -> Result<()>;

    /// Register a custom focusable region
    ///
    /// Allows plugins to register custom navigation targets that will appear
    /// in hint mode alongside auto-detected URLs, file paths, etc.
    ///
    /// Returns a unique ID for this focusable that can be used to unregister it later.
    ///
    /// # Arguments
    /// * `region` - The focusable region to register
    ///
    /// # Returns
    /// Unique ID for this focusable region
    ///
    /// # Example
    /// ```ignore
    /// let id = ctx.register_focusable(PluginFocusable {
    ///     x: 10,
    ///     y: 5,
    ///     width: 20,
    ///     height: 1,
    ///     label: "Click me".to_string(),
    ///     action: PluginFocusableAction::Custom("my_action".to_string()),
    /// })?;
    /// ```
    fn register_focusable(&self, region: PluginFocusable) -> Result<u64>;

    /// Unregister a previously registered focusable region
    ///
    /// Removes a focusable region from the navigation system using its ID.
    ///
    /// # Arguments
    /// * `id` - The ID returned from `register_focusable`
    ///
    /// # Example
    /// ```ignore
    /// ctx.unregister_focusable(focusable_id)?;
    /// ```
    fn unregister_focusable(&self, id: u64) -> Result<()>;
}

/// A plugin-registered focusable region
///
/// Represents a rectangular area in the terminal grid that can be
/// focused and activated via hint mode. Plugins can register these
/// to make custom UI elements or terminal content navigable.
#[derive(Debug, Clone, PartialEq)]
pub struct PluginFocusable {
    /// Column position in terminal grid (0-based)
    pub x: u16,

    /// Row position in terminal grid (0-based)
    pub y: u16,

    /// Width in terminal cells
    pub width: u16,

    /// Height in terminal cells
    pub height: u16,

    /// Label to display for this focusable (used in hint mode)
    pub label: String,

    /// Action to perform when this focusable is activated
    pub action: PluginFocusableAction,
}

/// Action to perform when a plugin focusable is activated
///
/// Defines what happens when a user activates a plugin-registered
/// focusable region through hint mode.
#[derive(Debug, Clone, PartialEq)]
pub enum PluginFocusableAction {
    /// Open a URL in the default browser
    OpenUrl(String),

    /// Open a file in the configured editor
    OpenFile(String),

    /// Custom plugin-defined action
    ///
    /// The plugin will receive a callback when this action is triggered,
    /// allowing custom behavior to be implemented.
    Custom(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plugin_focusable_creation() {
        let focusable = PluginFocusable {
            x: 10,
            y: 5,
            width: 20,
            height: 1,
            label: "Test".to_string(),
            action: PluginFocusableAction::OpenUrl("https://example.com".to_string()),
        };

        assert_eq!(focusable.x, 10);
        assert_eq!(focusable.y, 5);
        assert_eq!(focusable.width, 20);
        assert_eq!(focusable.height, 1);
        assert_eq!(focusable.label, "Test");
    }

    #[test]
    fn test_focusable_action_equality() {
        let action1 = PluginFocusableAction::OpenUrl("https://example.com".to_string());
        let action2 = PluginFocusableAction::OpenUrl("https://example.com".to_string());
        let action3 = PluginFocusableAction::OpenFile("/path/to/file".to_string());

        assert_eq!(action1, action2);
        assert_ne!(action1, action3);
    }

    #[test]
    fn test_focusable_clone() {
        let focusable = PluginFocusable {
            x: 10,
            y: 5,
            width: 20,
            height: 1,
            label: "Test".to_string(),
            action: PluginFocusableAction::Custom("my_action".to_string()),
        };

        let cloned = focusable.clone();
        assert_eq!(focusable, cloned);
    }
}
