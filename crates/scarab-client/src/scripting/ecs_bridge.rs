//! Bridge between Fusabi runtime and ECS event system
//!
//! This module provides the connection layer between Fusabi script native calls
//! and the Bevy ECS plugin host. It uses thread-safe channels to queue actions
//! from scripts and collect responses from ECS systems.

use bevy::prelude::*;
use std::sync::{Arc, Mutex};

use crate::events::{
    ModalItem, NotificationLevel, PluginAction, PluginResponse, StatusSide,
};

/// Channel for bidirectional communication between Fusabi and ECS
#[derive(Resource, Clone)]
pub struct FusabiActionChannel {
    /// Actions waiting to be processed by ECS
    pub pending_actions: Arc<Mutex<Vec<PluginAction>>>,
    /// Responses from ECS waiting to be consumed by scripts
    pub pending_responses: Arc<Mutex<Vec<PluginResponse>>>,
}

impl Default for FusabiActionChannel {
    fn default() -> Self {
        Self::new()
    }
}

impl FusabiActionChannel {
    /// Create a new action channel
    pub fn new() -> Self {
        Self {
            pending_actions: Arc::new(Mutex::new(Vec::new())),
            pending_responses: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Queue an action from Fusabi runtime to be processed by ECS
    pub fn send_action(&self, action: PluginAction) {
        if let Ok(mut actions) = self.pending_actions.lock() {
            actions.push(action);
        } else {
            error!("Failed to lock pending_actions mutex");
        }
    }

    /// Retrieve all pending responses for a specific plugin
    pub fn take_responses(&self, plugin_id: &str) -> Vec<PluginResponse> {
        if let Ok(mut responses) = self.pending_responses.lock() {
            let (matching, remaining): (Vec<_>, Vec<_>) = responses.drain(..).partition(|r| {
                match r {
                    PluginResponse::OverlaySpawned {
                        plugin_id: id, ..
                    } => id == plugin_id,
                    PluginResponse::TerminalContent {
                        plugin_id: id, ..
                    } => id == plugin_id,
                    PluginResponse::KeybindingTriggered {
                        plugin_id: id, ..
                    } => id == plugin_id,
                    PluginResponse::Error {
                        plugin_id: id, ..
                    } => id == plugin_id,
                    PluginResponse::NavFocusableRegistered {
                        plugin_id: id, ..
                    } => id == plugin_id,
                    PluginResponse::NavFocusableUnregistered {
                        plugin_id: id, ..
                    } => id == plugin_id,
                    PluginResponse::NavModeEntered {
                        plugin_id: id, ..
                    } => id == plugin_id,
                    PluginResponse::NavModeExited {
                        plugin_id: id, ..
                    } => id == plugin_id,
                }
            });
            *responses = remaining;
            matching
        } else {
            error!("Failed to lock pending_responses mutex");
            Vec::new()
        }
    }

    /// Peek at responses without consuming them
    pub fn peek_responses(&self, plugin_id: &str) -> Vec<PluginResponse> {
        if let Ok(responses) = self.pending_responses.lock() {
            responses
                .iter()
                .filter(|r| match r {
                    PluginResponse::OverlaySpawned {
                        plugin_id: id, ..
                    } => id == plugin_id,
                    PluginResponse::TerminalContent {
                        plugin_id: id, ..
                    } => id == plugin_id,
                    PluginResponse::KeybindingTriggered {
                        plugin_id: id, ..
                    } => id == plugin_id,
                    PluginResponse::Error {
                        plugin_id: id, ..
                    } => id == plugin_id,
                    PluginResponse::NavFocusableRegistered {
                        plugin_id: id, ..
                    } => id == plugin_id,
                    PluginResponse::NavFocusableUnregistered {
                        plugin_id: id, ..
                    } => id == plugin_id,
                    PluginResponse::NavModeEntered {
                        plugin_id: id, ..
                    } => id == plugin_id,
                    PluginResponse::NavModeExited {
                        plugin_id: id, ..
                    } => id == plugin_id,
                })
                .cloned()
                .collect()
        } else {
            Vec::new()
        }
    }
}

/// System that flushes pending Fusabi actions to ECS event writers
///
/// This runs every frame and drains the pending_actions queue, converting
/// each action into a Bevy event that can be processed by other systems.
pub fn flush_fusabi_actions(
    channel: Res<FusabiActionChannel>,
    mut events: EventWriter<PluginAction>,
) {
    if let Ok(mut actions) = channel.pending_actions.lock() {
        for action in actions.drain(..) {
            events.send(action);
        }
    } else {
        error!("Failed to lock pending_actions mutex in flush_fusabi_actions");
    }
}

/// System that collects ECS responses and queues them for Fusabi scripts
///
/// This runs every frame and reads PluginResponse events, storing them in
/// the pending_responses queue so scripts can retrieve them.
pub fn collect_fusabi_responses(
    channel: Res<FusabiActionChannel>,
    mut events: EventReader<PluginResponse>,
) {
    if let Ok(mut responses) = channel.pending_responses.lock() {
        for response in events.read() {
            responses.push(response.clone());
        }
    } else {
        error!("Failed to lock pending_responses mutex in collect_fusabi_responses");
    }
}

/// Native function implementations for Fusabi scripts
///
/// This provides the actual implementation of native functions that Fusabi
/// scripts can call. Each method queues a PluginAction to be processed by ECS.
pub struct FusabiNatives {
    /// Reference to the action channel for sending actions
    channel: Arc<Mutex<Vec<PluginAction>>>,
    /// Plugin ID for this script instance
    plugin_id: String,
}

impl FusabiNatives {
    /// Create a new natives instance for a specific plugin
    pub fn new(channel: &FusabiActionChannel, plugin_id: String) -> Self {
        Self {
            channel: channel.pending_actions.clone(),
            plugin_id,
        }
    }

    /// Native: ui_spawn_overlay(x, y, width, height, content, z_index) -> overlay_id
    ///
    /// Spawns a UI overlay at the specified terminal cell position.
    pub fn ui_spawn_overlay(
        &self,
        x: u16,
        y: u16,
        width: u16,
        height: u16,
        content: String,
        z_index: f32,
    ) {
        if let Ok(mut actions) = self.channel.lock() {
            actions.push(PluginAction::SpawnOverlay {
                plugin_id: self.plugin_id.clone(),
                x,
                y,
                width,
                height,
                content,
                z_index,
            });
        } else {
            error!("Failed to lock channel in ui_spawn_overlay");
        }
    }

    /// Native: ui_despawn_overlay(overlay_id)
    ///
    /// Removes a previously spawned overlay by ID.
    pub fn ui_despawn_overlay(&self, overlay_id: u64) {
        if let Ok(mut actions) = self.channel.lock() {
            actions.push(PluginAction::DespawnOverlay {
                plugin_id: self.plugin_id.clone(),
                overlay_id,
            });
        } else {
            error!("Failed to lock channel in ui_despawn_overlay");
        }
    }

    /// Native: notify(title, message, level)
    ///
    /// Shows a notification to the user with the specified severity level.
    pub fn notify(&self, title: String, message: String, level: &str) {
        let level = match level {
            "info" => NotificationLevel::Info,
            "warning" => NotificationLevel::Warning,
            "error" => NotificationLevel::Error,
            "success" => NotificationLevel::Success,
            _ => NotificationLevel::Info,
        };

        if let Ok(mut actions) = self.channel.lock() {
            actions.push(PluginAction::ShowNotification {
                plugin_id: self.plugin_id.clone(),
                title,
                message,
                level,
                duration_ms: 5000,
            });
        } else {
            error!("Failed to lock channel in notify");
        }
    }

    /// Native: status_add(side, content, priority) -> item_id
    ///
    /// Adds an item to the status bar on the specified side with given priority.
    pub fn status_add(&self, side: &str, content: String, priority: i32) {
        let side = match side {
            "left" => StatusSide::Left,
            "right" => StatusSide::Right,
            _ => StatusSide::Right,
        };

        if let Ok(mut actions) = self.channel.lock() {
            actions.push(PluginAction::AddStatusItem {
                plugin_id: self.plugin_id.clone(),
                side,
                content,
                priority,
            });
        } else {
            error!("Failed to lock channel in status_add");
        }
    }

    /// Native: status_remove(item_id)
    ///
    /// Removes a status bar item by ID.
    pub fn status_remove(&self, item_id: u64) {
        if let Ok(mut actions) = self.channel.lock() {
            actions.push(PluginAction::RemoveStatusItem {
                plugin_id: self.plugin_id.clone(),
                item_id,
            });
        } else {
            error!("Failed to lock channel in status_remove");
        }
    }

    /// Native: register_keybinding(key, modifiers, action_id)
    ///
    /// Registers a keybinding that will trigger the specified action_id when pressed.
    pub fn register_keybinding(&self, key: String, modifiers: Vec<String>, action_id: String) {
        if let Ok(mut actions) = self.channel.lock() {
            actions.push(PluginAction::RegisterKeybinding {
                plugin_id: self.plugin_id.clone(),
                key,
                modifiers,
                action_id,
            });
        } else {
            error!("Failed to lock channel in register_keybinding");
        }
    }

    /// Native: send_input(data)
    ///
    /// Sends raw input data to the terminal.
    pub fn send_input(&self, data: Vec<u8>) {
        if let Ok(mut actions) = self.channel.lock() {
            actions.push(PluginAction::SendInput {
                plugin_id: self.plugin_id.clone(),
                data,
            });
        } else {
            error!("Failed to lock channel in send_input");
        }
    }

    /// Native: get_terminal_rows(start_row, end_row)
    ///
    /// Requests terminal content from the specified row range.
    /// Results will be available via PluginResponse::TerminalContent.
    pub fn get_terminal_rows(&self, start_row: u16, end_row: u16) {
        if let Ok(mut actions) = self.channel.lock() {
            actions.push(PluginAction::RequestTerminalContent {
                plugin_id: self.plugin_id.clone(),
                start_row,
                end_row,
            });
        } else {
            error!("Failed to lock channel in get_terminal_rows");
        }
    }

    /// Native: update_theme(theme_json)
    ///
    /// Updates the terminal theme with JSON-formatted theme data.
    pub fn update_theme(&self, theme_json: String) {
        if let Ok(mut actions) = self.channel.lock() {
            actions.push(PluginAction::UpdateTheme {
                plugin_id: self.plugin_id.clone(),
                theme_json,
            });
        } else {
            error!("Failed to lock channel in update_theme");
        }
    }

    /// Native: show_modal(title, items)
    ///
    /// Shows a modal dialog with the specified items.
    pub fn show_modal(&self, title: String, items: Vec<ModalItem>) {
        if let Ok(mut actions) = self.channel.lock() {
            actions.push(PluginAction::ShowModal {
                plugin_id: self.plugin_id.clone(),
                title,
                items,
            });
        } else {
            error!("Failed to lock channel in show_modal");
        }
    }
}

/// Plugin that integrates the Fusabi ECS bridge into Bevy
///
/// This plugin adds the necessary resources and systems to enable
/// bidirectional communication between Fusabi scripts and ECS.
pub struct FusabiEcsBridgePlugin;

impl Plugin for FusabiEcsBridgePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<FusabiActionChannel>()
            .add_event::<PluginAction>()
            .add_event::<PluginResponse>()
            .add_systems(
                Update,
                (
                    flush_fusabi_actions.before(collect_fusabi_responses),
                    collect_fusabi_responses,
                ),
            );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_channel_creation() {
        let channel = FusabiActionChannel::new();
        assert!(channel.pending_actions.lock().unwrap().is_empty());
        assert!(channel.pending_responses.lock().unwrap().is_empty());
    }

    #[test]
    fn test_send_action() {
        let channel = FusabiActionChannel::new();
        channel.send_action(PluginAction::ShowNotification {
            plugin_id: "test".to_string(),
            title: "Test".to_string(),
            message: "Test message".to_string(),
            level: NotificationLevel::Info,
            duration_ms: 5000,
        });

        let actions = channel.pending_actions.lock().unwrap();
        assert_eq!(actions.len(), 1);
    }

    #[test]
    fn test_take_responses() {
        let channel = FusabiActionChannel::new();

        // Add responses for different plugins
        {
            let mut responses = channel.pending_responses.lock().unwrap();
            responses.push(PluginResponse::OverlaySpawned {
                plugin_id: "plugin1".to_string(),
                overlay_id: 1,
            });
            responses.push(PluginResponse::OverlaySpawned {
                plugin_id: "plugin2".to_string(),
                overlay_id: 2,
            });
            responses.push(PluginResponse::Error {
                plugin_id: "plugin1".to_string(),
                action: "test".to_string(),
                message: "error".to_string(),
            });
        }

        // Take responses for plugin1
        let plugin1_responses = channel.take_responses("plugin1");
        assert_eq!(plugin1_responses.len(), 2);

        // Verify remaining responses
        let remaining = channel.pending_responses.lock().unwrap();
        assert_eq!(remaining.len(), 1);
    }

    #[test]
    fn test_natives_creation() {
        let channel = FusabiActionChannel::new();
        let natives = FusabiNatives::new(&channel, "test_plugin".to_string());
        assert_eq!(natives.plugin_id, "test_plugin");
    }

    #[test]
    fn test_natives_notify() {
        let channel = FusabiActionChannel::new();
        let natives = FusabiNatives::new(&channel, "test".to_string());

        natives.notify(
            "Title".to_string(),
            "Message".to_string(),
            "warning",
        );

        let actions = channel.pending_actions.lock().unwrap();
        assert_eq!(actions.len(), 1);

        if let PluginAction::ShowNotification { level, .. } = &actions[0] {
            assert_eq!(*level, NotificationLevel::Warning);
        } else {
            panic!("Expected ShowNotification action");
        }
    }
}
