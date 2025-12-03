//! Context Menu Action Dispatch
//!
//! This module handles the execution of actions triggered by context menu selections.
//! Actions include:
//! - Copy/Paste via clipboard
//! - Open URL in browser
//! - Open file in editor
//! - Split pane operations
//! - Search activation
//! - Custom plugin actions

use bevy::prelude::*;

use super::ContextMenuItemSelected;

/// Context menu action types
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ContextMenuAction {
    /// Copy selected text to clipboard
    Copy,
    /// Paste from clipboard
    Paste,
    /// Select all text
    SelectAll,
    /// Clear current selection
    ClearSelection,
    /// Open search interface
    Search,
    /// Create new tab
    NewTab,
    /// Split pane horizontally
    SplitHorizontal,
    /// Split pane vertically
    SplitVertical,
    /// Open URL in browser
    OpenUrl(String),
    /// Copy URL to clipboard
    CopyUrl(String),
    /// Open file in editor
    OpenFile(String),
    /// Copy file path to clipboard
    CopyPath(String),
    /// Custom plugin action
    PluginAction(String),
}

impl ContextMenuAction {
    /// Parse action from menu item ID
    pub fn from_id(id: &str, data: Option<&str>) -> Option<Self> {
        match id {
            "copy" => Some(Self::Copy),
            "paste" => Some(Self::Paste),
            "select_all" => Some(Self::SelectAll),
            "clear_selection" => Some(Self::ClearSelection),
            "search" => Some(Self::Search),
            "new_tab" => Some(Self::NewTab),
            "split_horizontal" => Some(Self::SplitHorizontal),
            "split_vertical" => Some(Self::SplitVertical),
            "open_url" => data.map(|d| Self::OpenUrl(d.to_string())),
            "copy_url" => data.map(|d| Self::CopyUrl(d.to_string())),
            "open_file" => data.map(|d| Self::OpenFile(d.to_string())),
            "copy_path" => data.map(|d| Self::CopyPath(d.to_string())),
            _ => {
                // Check if it's a plugin action
                if id.starts_with("plugin.") {
                    Some(Self::PluginAction(id.to_string()))
                } else {
                    None
                }
            }
        }
    }
}

/// Event for dispatching context menu actions
#[derive(Event)]
pub struct DispatchContextMenuAction {
    pub action: ContextMenuAction,
}

/// System to convert menu item selections into action events
pub fn handle_context_menu_actions(
    mut selection_events: EventReader<ContextMenuItemSelected>,
    mut action_events: EventWriter<DispatchContextMenuAction>,
) {
    for event in selection_events.read() {
        if let Some(action) = ContextMenuAction::from_id(&event.item_id, event.data.as_deref()) {
            info!("Context menu action: {:?}", action);
            action_events.send(DispatchContextMenuAction { action });
        } else {
            warn!("Unknown context menu action: {}", event.item_id);
        }
    }
}

/// System to execute context menu actions
pub fn dispatch_action(
    mut action_events: EventReader<DispatchContextMenuAction>,
    // TODO: Add resources for clipboard, IPC, etc.
) {
    for event in action_events.read() {
        match &event.action {
            ContextMenuAction::Copy => {
                info!("Executing Copy action");
                // TODO: Integrate with clipboard system
                // This should trigger the existing copy functionality
            }

            ContextMenuAction::Paste => {
                info!("Executing Paste action");
                // TODO: Integrate with clipboard system
            }

            ContextMenuAction::SelectAll => {
                info!("Executing Select All action");
                // TODO: Send select all command to selection system
            }

            ContextMenuAction::ClearSelection => {
                info!("Executing Clear Selection action");
                // TODO: Clear current selection
            }

            ContextMenuAction::Search => {
                info!("Executing Search action");
                // TODO: Open search UI
            }

            ContextMenuAction::NewTab => {
                info!("Executing New Tab action");
                // TODO: Send new tab command to daemon via IPC
            }

            ContextMenuAction::SplitHorizontal => {
                info!("Executing Split Horizontal action");
                // TODO: Send split command to daemon
            }

            ContextMenuAction::SplitVertical => {
                info!("Executing Split Vertical action");
                // TODO: Send split command to daemon
            }

            ContextMenuAction::OpenUrl(url) => {
                info!("Opening URL: {}", url);
                // Use the 'open' crate to open URL in default browser
                if let Err(e) = open::that(url) {
                    error!("Failed to open URL {}: {}", url, e);
                } else {
                    info!("Opened URL in browser: {}", url);
                }
            }

            ContextMenuAction::CopyUrl(url) => {
                info!("Copying URL to clipboard: {}", url);
                // TODO: Copy to clipboard
                // For now, use arboard directly
                use arboard::Clipboard;
                match Clipboard::new() {
                    Ok(mut clipboard) => {
                        if let Err(e) = clipboard.set_text(url) {
                            error!("Failed to copy URL to clipboard: {}", e);
                        } else {
                            info!("URL copied to clipboard");
                        }
                    }
                    Err(e) => {
                        error!("Failed to access clipboard: {}", e);
                    }
                }
            }

            ContextMenuAction::OpenFile(path) => {
                info!("Opening file: {}", path);
                // Open file in default application
                if let Err(e) = open::that(path) {
                    error!("Failed to open file {}: {}", path, e);
                } else {
                    info!("Opened file: {}", path);
                }
            }

            ContextMenuAction::CopyPath(path) => {
                info!("Copying path to clipboard: {}", path);
                use arboard::Clipboard;
                match Clipboard::new() {
                    Ok(mut clipboard) => {
                        if let Err(e) = clipboard.set_text(path) {
                            error!("Failed to copy path to clipboard: {}", e);
                        } else {
                            info!("Path copied to clipboard");
                        }
                    }
                    Err(e) => {
                        error!("Failed to access clipboard: {}", e);
                    }
                }
            }

            ContextMenuAction::PluginAction(action_id) => {
                info!("Executing plugin action: {}", action_id);
                // TODO: Route to plugin system
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_action_from_id_basic() {
        assert_eq!(
            ContextMenuAction::from_id("copy", None),
            Some(ContextMenuAction::Copy)
        );
        assert_eq!(
            ContextMenuAction::from_id("paste", None),
            Some(ContextMenuAction::Paste)
        );
        assert_eq!(
            ContextMenuAction::from_id("search", None),
            Some(ContextMenuAction::Search)
        );
    }

    #[test]
    fn test_action_from_id_with_data() {
        let url = "https://example.com";
        assert_eq!(
            ContextMenuAction::from_id("open_url", Some(url)),
            Some(ContextMenuAction::OpenUrl(url.to_string()))
        );
        assert_eq!(
            ContextMenuAction::from_id("copy_url", Some(url)),
            Some(ContextMenuAction::CopyUrl(url.to_string()))
        );

        let path = "/home/user/file.txt";
        assert_eq!(
            ContextMenuAction::from_id("open_file", Some(path)),
            Some(ContextMenuAction::OpenFile(path.to_string()))
        );
    }

    #[test]
    fn test_action_from_id_plugin() {
        let plugin_action = ContextMenuAction::from_id("plugin.custom_action", None);
        assert!(plugin_action.is_some());
        match plugin_action {
            Some(ContextMenuAction::PluginAction(id)) => {
                assert_eq!(id, "plugin.custom_action");
            }
            _ => panic!("Expected PluginAction"),
        }
    }

    #[test]
    fn test_action_from_id_unknown() {
        assert_eq!(ContextMenuAction::from_id("unknown_action", None), None);
        assert_eq!(ContextMenuAction::from_id("", None), None);
    }

    #[test]
    fn test_split_actions() {
        assert_eq!(
            ContextMenuAction::from_id("split_horizontal", None),
            Some(ContextMenuAction::SplitHorizontal)
        );
        assert_eq!(
            ContextMenuAction::from_id("split_vertical", None),
            Some(ContextMenuAction::SplitVertical)
        );
    }
}
