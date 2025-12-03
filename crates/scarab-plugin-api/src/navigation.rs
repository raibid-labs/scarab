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

use thiserror::Error;

use crate::error::Result;

/// Security capabilities for plugin navigation APIs
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PluginNavCapabilities {
    pub can_enter_hint_mode: bool,
    pub can_register_focusables: bool,
    pub max_focusables: usize,
    pub can_trigger_actions: bool,
}

impl Default for PluginNavCapabilities {
    fn default() -> Self {
        Self {
            can_enter_hint_mode: true,
            can_register_focusables: true,
            max_focusables: 50,
            can_trigger_actions: true,
        }
    }
}

/// Errors that can occur during navigation API validation
#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum ValidationError {
    #[error("Focusable coordinates out of bounds: x={x}, y={y} (max: {max})")]
    CoordinatesOutOfBounds { x: u16, y: u16, max: u16 },
    #[error("Invalid focusable dimensions: width={width}, height={height}")]
    InvalidDimensions { width: u16, height: u16 },
    #[error("Dangerous URL protocol detected: {protocol}")]
    DangerousProtocol { protocol: String },
    #[error("Malformed URL: {url}")]
    MalformedUrl { url: String },
    #[error("Dangerous file path pattern: {path}")]
    DangerousPath { path: String },
    #[error("Invalid label: {reason}")]
    InvalidLabel { reason: String },
}

pub fn validate_focusable(region: &PluginFocusable) -> std::result::Result<(), ValidationError> {
    const MAX_COORDINATE: u16 = 1000;
    if region.x >= MAX_COORDINATE || region.y >= MAX_COORDINATE {
        return Err(ValidationError::CoordinatesOutOfBounds {
            x: region.x,
            y: region.y,
            max: MAX_COORDINATE,
        });
    }
    if region.width == 0 || region.height == 0 {
        return Err(ValidationError::InvalidDimensions {
            width: region.width,
            height: region.height,
        });
    }
    if region.width > MAX_COORDINATE || region.height > MAX_COORDINATE {
        return Err(ValidationError::InvalidDimensions {
            width: region.width,
            height: region.height,
        });
    }
    if region.label.is_empty() {
        return Err(ValidationError::InvalidLabel {
            reason: "Label cannot be empty".to_string(),
        });
    }
    if region.label.len() > 256 {
        return Err(ValidationError::InvalidLabel {
            reason: format!("Label too long: {} chars (max: 256)", region.label.len()),
        });
    }
    match &region.action {
        PluginFocusableAction::OpenUrl(url) => validate_url(url)?,
        PluginFocusableAction::OpenFile(path) => validate_file_path(path)?,
        PluginFocusableAction::Custom(_) => {}
    }
    Ok(())
}

fn validate_url(url: &str) -> std::result::Result<(), ValidationError> {
    let url_lower = url.to_lowercase();
    const DANGEROUS_PROTOCOLS: &[&str] = &["javascript:", "data:", "vbscript:", "about:", "blob:"];
    for protocol in DANGEROUS_PROTOCOLS {
        if url_lower.starts_with(protocol) {
            return Err(ValidationError::DangerousProtocol {
                protocol: protocol.to_string(),
            });
        }
    }
    if !url_lower.starts_with("http://") && !url_lower.starts_with("https://") && !url_lower.starts_with("file://") {
        return Err(ValidationError::MalformedUrl {
            url: url.to_string(),
        });
    }
    if url.len() < 10 {
        return Err(ValidationError::MalformedUrl {
            url: url.to_string(),
        });
    }
    Ok(())
}

fn validate_file_path(path: &str) -> std::result::Result<(), ValidationError> {
    if path.contains("..") {
        return Err(ValidationError::DangerousPath {
            path: path.to_string(),
        });
    }
    if path.is_empty() {
        return Err(ValidationError::DangerousPath {
            path: "empty path".to_string(),
        });
    }
    let path_lower = path.to_lowercase();
    const SENSITIVE_PATTERNS: &[&str] = &["/etc/passwd", "/etc/shadow", "/proc/", "/sys/", "\\.ssh", "/root/"];
    for pattern in SENSITIVE_PATTERNS {
        if path_lower.contains(pattern) {
            return Err(ValidationError::DangerousPath {
                path: path.to_string(),
            });
        }
    }
    Ok(())
}

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

    #[test]
    fn test_validate_focusable_valid() {
        let focusable = PluginFocusable {
            x: 10,
            y: 5,
            width: 20,
            height: 1,
            label: "GitHub".to_string(),
            action: PluginFocusableAction::OpenUrl("https://github.com".to_string()),
        };
        assert!(validate_focusable(&focusable).is_ok());
    }

    #[test]
    fn test_validate_focusable_out_of_bounds() {
        let focusable = PluginFocusable {
            x: 1000,
            y: 5,
            width: 20,
            height: 1,
            label: "Test".to_string(),
            action: PluginFocusableAction::OpenUrl("https://example.com".to_string()),
        };
        assert!(matches!(
            validate_focusable(&focusable).unwrap_err(),
            ValidationError::CoordinatesOutOfBounds { .. }
        ));
    }

    #[test]
    fn test_validate_url_dangerous_protocol() {
        assert!(validate_url("javascript:alert('xss')").is_err());
        assert!(validate_url("data:text/html,<script>alert('xss')</script>").is_err());
    }

    #[test]
    fn test_validate_file_path_traversal() {
        assert!(validate_file_path("../../../etc/passwd").is_err());
    }
}
