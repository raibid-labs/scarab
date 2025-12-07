//! Plugin menu system for the Scarab Dock
//!
//! This module provides types for plugins to define contextual menus that
//! appear in the Scarab Dock interface. Menu items can trigger commands,
//! remote actions, or spawn submenus for hierarchical navigation.
//!
//! ## Example
//!
//! ```rust
//! use scarab_plugin_api::menu::{MenuItem, MenuAction};
//!
//! // Create a simple command menu item
//! let item = MenuItem::new("Run Tests", MenuAction::Command("cargo test".to_string()))
//!     .with_icon("🧪")
//!     .with_shortcut("Ctrl+T");
//!
//! // Create a submenu with multiple actions
//! let build_menu = MenuItem::new(
//!     "Build",
//!     MenuAction::SubMenu(vec![
//!         MenuItem::new("Debug", MenuAction::Command("cargo build".to_string())),
//!         MenuItem::new("Release", MenuAction::Command("cargo build --release".to_string())),
//!     ])
//! );
//! ```

use serde::{Deserialize, Serialize};

/// A menu item that can be displayed in the Scarab Dock
///
/// Menu items represent actions that users can trigger from the dock interface.
/// They support icons, keyboard shortcuts, and hierarchical organization through submenus.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MenuItem {
    /// Display label for the menu item
    ///
    /// This is the text shown to the user in the dock interface.
    /// Keep it concise and descriptive (e.g., "Run Tests", "Open Settings").
    pub label: String,

    /// Optional icon or emoji for visual identification
    ///
    /// Can be a single emoji (e.g., "🚀", "⚙️") or an icon identifier.
    /// Icons help users quickly identify menu items at a glance.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,

    /// The action to perform when this menu item is selected
    ///
    /// Defines what happens when the user clicks or activates this menu item.
    pub action: MenuAction,

    /// Optional keyboard shortcut hint
    ///
    /// Display-only hint showing the keyboard shortcut (e.g., "Ctrl+T", "Alt+B").
    /// Note: The actual shortcut handling must be implemented separately.
    /// This field is purely informational for the UI.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub shortcut: Option<String>,
}

impl MenuItem {
    /// Create a new menu item with a label and action
    ///
    /// # Arguments
    ///
    /// * `label` - Display text for the menu item
    /// * `action` - Action to perform when selected
    ///
    /// # Example
    ///
    /// ```rust
    /// use scarab_plugin_api::menu::{MenuItem, MenuAction};
    ///
    /// let item = MenuItem::new(
    ///     "Deploy",
    ///     MenuAction::Command("./deploy.sh".to_string())
    /// );
    /// ```
    pub fn new(label: impl Into<String>, action: MenuAction) -> Self {
        Self {
            label: label.into(),
            icon: None,
            action,
            shortcut: None,
        }
    }

    /// Add an icon or emoji to this menu item
    ///
    /// # Arguments
    ///
    /// * `icon` - Icon identifier or emoji character
    ///
    /// # Example
    ///
    /// ```rust
    /// use scarab_plugin_api::menu::{MenuItem, MenuAction};
    ///
    /// let item = MenuItem::new("Build", MenuAction::Command("make".to_string()))
    ///     .with_icon("🔨");
    /// ```
    pub fn with_icon(mut self, icon: impl Into<String>) -> Self {
        self.icon = Some(icon.into());
        self
    }

    /// Add a keyboard shortcut hint to this menu item
    ///
    /// Note: This is display-only. The shortcut must be registered separately
    /// through the appropriate input handling mechanism.
    ///
    /// # Arguments
    ///
    /// * `shortcut` - Shortcut text (e.g., "Ctrl+B", "Alt+Shift+T")
    ///
    /// # Example
    ///
    /// ```rust
    /// use scarab_plugin_api::menu::{MenuItem, MenuAction};
    ///
    /// let item = MenuItem::new("Save", MenuAction::Command("save".to_string()))
    ///     .with_shortcut("Ctrl+S");
    /// ```
    pub fn with_shortcut(mut self, shortcut: impl Into<String>) -> Self {
        self.shortcut = Some(shortcut.into());
        self
    }
}

/// Action to perform when a menu item is selected
///
/// Menu actions define what happens when a user interacts with a menu item.
/// They can execute terminal commands, trigger plugin callbacks, or open submenus.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum MenuAction {
    /// Execute a terminal command
    ///
    /// When selected, this action sends the command to the active PTY.
    /// The command is executed as if the user typed it into the terminal.
    ///
    /// # Example
    ///
    /// ```rust
    /// use scarab_plugin_api::menu::MenuAction;
    ///
    /// let action = MenuAction::Command("ls -la".to_string());
    /// ```
    Command(String),

    /// Trigger a plugin remote action
    ///
    /// When selected, this action calls the plugin's `on_remote_command` hook
    /// with the specified identifier. This allows plugins to implement custom
    /// actions beyond simple command execution.
    ///
    /// # Example
    ///
    /// ```rust
    /// use scarab_plugin_api::menu::MenuAction;
    ///
    /// // Will call on_remote_command("refresh_cache", ctx)
    /// let action = MenuAction::Remote("refresh_cache".to_string());
    /// ```
    Remote(String),

    /// Open a submenu with additional items
    ///
    /// When selected, this action displays a nested menu with the provided items.
    /// Submenus can be nested arbitrarily deep, but keep hierarchies shallow
    /// for better user experience (2-3 levels maximum recommended).
    ///
    /// # Example
    ///
    /// ```rust
    /// use scarab_plugin_api::menu::{MenuItem, MenuAction};
    ///
    /// let submenu = MenuAction::SubMenu(vec![
    ///     MenuItem::new("Option 1", MenuAction::Command("cmd1".to_string())),
    ///     MenuItem::new("Option 2", MenuAction::Command("cmd2".to_string())),
    /// ]);
    /// ```
    SubMenu(Vec<MenuItem>),
}

impl MenuAction {
    /// Check if this action is a submenu
    ///
    /// # Returns
    ///
    /// `true` if this action opens a submenu, `false` otherwise
    pub fn is_submenu(&self) -> bool {
        matches!(self, MenuAction::SubMenu(_))
    }

    /// Check if this action executes a command
    ///
    /// # Returns
    ///
    /// `true` if this action executes a terminal command, `false` otherwise
    pub fn is_command(&self) -> bool {
        matches!(self, MenuAction::Command(_))
    }

    /// Check if this action triggers a remote callback
    ///
    /// # Returns
    ///
    /// `true` if this action calls a plugin remote handler, `false` otherwise
    pub fn is_remote(&self) -> bool {
        matches!(self, MenuAction::Remote(_))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_menu_item_builder() {
        let item = MenuItem::new("Test", MenuAction::Command("test".to_string()))
            .with_icon("🧪")
            .with_shortcut("Ctrl+T");

        assert_eq!(item.label, "Test");
        assert_eq!(item.icon, Some("🧪".to_string()));
        assert_eq!(item.shortcut, Some("Ctrl+T".to_string()));
    }

    #[test]
    fn test_menu_action_checks() {
        let cmd = MenuAction::Command("ls".to_string());
        assert!(cmd.is_command());
        assert!(!cmd.is_remote());
        assert!(!cmd.is_submenu());

        let remote = MenuAction::Remote("action_id".to_string());
        assert!(!remote.is_command());
        assert!(remote.is_remote());
        assert!(!remote.is_submenu());

        let submenu = MenuAction::SubMenu(vec![]);
        assert!(!submenu.is_command());
        assert!(!submenu.is_remote());
        assert!(submenu.is_submenu());
    }

    #[test]
    fn test_nested_submenu() {
        let nested = MenuItem::new(
            "Root",
            MenuAction::SubMenu(vec![MenuItem::new(
                "Child",
                MenuAction::SubMenu(vec![MenuItem::new(
                    "Grandchild",
                    MenuAction::Command("echo nested".to_string()),
                )]),
            )]),
        );

        if let MenuAction::SubMenu(items) = &nested.action {
            assert_eq!(items.len(), 1);
            assert_eq!(items[0].label, "Child");
        } else {
            panic!("Expected submenu");
        }
    }

    #[test]
    fn test_menu_item_with_remote_action() {
        let item =
            MenuItem::new("Test", MenuAction::Remote("test_action".to_string())).with_icon("🎯");

        assert_eq!(item.label, "Test");
        assert_eq!(item.icon, Some("🎯".to_string()));
        assert!(item.action.is_remote());

        if let MenuAction::Remote(id) = &item.action {
            assert_eq!(id, "test_action");
        } else {
            panic!("Expected Remote action");
        }
    }
}
