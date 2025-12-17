//! Host function registration for Fusabi scripts
//!
//! This module registers all Scarab native functions with the fusabi-host
//! registry, enabling scripts to call Scarab APIs using the `Scarab.*` namespace.

use std::sync::Arc;

use bevy::prelude::*;
use fusabi_host::{HostRegistry, Value};

use crate::events::{ModalItem, NotificationLevel, PluginAction, StatusSide};
use crate::scripting::ecs_bridge::FusabiActionChannel;

/// Register all Scarab host functions with the given registry.
///
/// This enables scripts to call functions like:
/// - `Scarab.status_add("left", "content", 100)`
/// - `Scarab.notify("Title", "Message", "info")`
/// - `Scarab.spawn_overlay(0, 0, 10, 5, "content", 1.0)`
pub fn register_scarab_functions(registry: &mut HostRegistry, channel: Arc<FusabiActionChannel>) {
    // Store plugin ID for now - in future this would be per-script
    let default_plugin_id = "script".to_string();

    // =========================================================================
    // Status Bar Functions
    // =========================================================================

    // Scarab.status_add(side: string, content: string, priority: int) -> unit
    {
        let channel = channel.clone();
        let plugin_id = default_plugin_id.clone();
        registry.register_module("Scarab", "status_add", move |args, _ctx| {
            let side = args
                .get(0)
                .and_then(|v| v.as_str())
                .unwrap_or("right")
                .to_string();
            let content = args
                .get(1)
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let priority = args.get(2).and_then(|v| v.as_int()).unwrap_or(0) as i32;

            let side = match side.as_str() {
                "left" => StatusSide::Left,
                _ => StatusSide::Right,
            };

            channel.send_action(PluginAction::AddStatusItem {
                plugin_id: plugin_id.clone(),
                side,
                content,
                priority,
            });

            Ok(Value::Null)
        });
    }

    // Scarab.status_remove(item_id: int) -> unit
    {
        let channel = channel.clone();
        let plugin_id = default_plugin_id.clone();
        registry.register_module("Scarab", "status_remove", move |args, _ctx| {
            let item_id = args.get(0).and_then(|v| v.as_int()).unwrap_or(0) as u64;

            channel.send_action(PluginAction::RemoveStatusItem {
                plugin_id: plugin_id.clone(),
                item_id,
            });

            Ok(Value::Null)
        });
    }

    // =========================================================================
    // Notification Functions
    // =========================================================================

    // Scarab.notify(title: string, message: string, level: string) -> unit
    {
        let channel = channel.clone();
        let plugin_id = default_plugin_id.clone();
        registry.register_module("Scarab", "notify", move |args, _ctx| {
            let title = args
                .get(0)
                .and_then(|v| v.as_str())
                .unwrap_or("Notification")
                .to_string();
            let message = args
                .get(1)
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let level_str = args.get(2).and_then(|v| v.as_str()).unwrap_or("info");

            let level = match level_str {
                "info" => NotificationLevel::Info,
                "warning" => NotificationLevel::Warning,
                "error" => NotificationLevel::Error,
                "success" => NotificationLevel::Success,
                _ => NotificationLevel::Info,
            };

            channel.send_action(PluginAction::ShowNotification {
                plugin_id: plugin_id.clone(),
                title,
                message,
                level,
                duration_ms: 5000,
            });

            Ok(Value::Null)
        });
    }

    // =========================================================================
    // Overlay Functions
    // =========================================================================

    // Scarab.spawn_overlay(x: int, y: int, width: int, height: int, content: string, z_index: float) -> unit
    {
        let channel = channel.clone();
        let plugin_id = default_plugin_id.clone();
        registry.register_module("Scarab", "spawn_overlay", move |args, _ctx| {
            let x = args.get(0).and_then(|v| v.as_int()).unwrap_or(0) as u16;
            let y = args.get(1).and_then(|v| v.as_int()).unwrap_or(0) as u16;
            let width = args.get(2).and_then(|v| v.as_int()).unwrap_or(10) as u16;
            let height = args.get(3).and_then(|v| v.as_int()).unwrap_or(5) as u16;
            let content = args
                .get(4)
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let z_index = args.get(5).and_then(|v| v.as_float()).unwrap_or(1.0) as f32;

            channel.send_action(PluginAction::SpawnOverlay {
                plugin_id: plugin_id.clone(),
                x,
                y,
                width,
                height,
                content,
                z_index,
            });

            Ok(Value::Null)
        });
    }

    // Scarab.despawn_overlay(overlay_id: int) -> unit
    {
        let channel = channel.clone();
        let plugin_id = default_plugin_id.clone();
        registry.register_module("Scarab", "despawn_overlay", move |args, _ctx| {
            let overlay_id = args.get(0).and_then(|v| v.as_int()).unwrap_or(0) as u64;

            channel.send_action(PluginAction::DespawnOverlay {
                plugin_id: plugin_id.clone(),
                overlay_id,
            });

            Ok(Value::Null)
        });
    }

    // =========================================================================
    // Terminal Functions
    // =========================================================================

    // Scarab.send_input(data: string) -> unit
    {
        let channel = channel.clone();
        let plugin_id = default_plugin_id.clone();
        registry.register_module("Scarab", "send_input", move |args, _ctx| {
            let data = args
                .get(0)
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .as_bytes()
                .to_vec();

            channel.send_action(PluginAction::SendInput {
                plugin_id: plugin_id.clone(),
                data,
            });

            Ok(Value::Null)
        });
    }

    // Scarab.get_terminal_rows(start_row: int, end_row: int) -> unit
    // Note: Results come back via PluginResponse::TerminalContent
    {
        let channel = channel.clone();
        let plugin_id = default_plugin_id.clone();
        registry.register_module("Scarab", "get_terminal_rows", move |args, _ctx| {
            let start_row = args.get(0).and_then(|v| v.as_int()).unwrap_or(0) as u16;
            let end_row = args.get(1).and_then(|v| v.as_int()).unwrap_or(24) as u16;

            channel.send_action(PluginAction::RequestTerminalContent {
                plugin_id: plugin_id.clone(),
                start_row,
                end_row,
            });

            Ok(Value::Null)
        });
    }

    // =========================================================================
    // Theme Functions
    // =========================================================================

    // Scarab.update_theme(theme_json: string) -> unit
    {
        let channel = channel.clone();
        let plugin_id = default_plugin_id.clone();
        registry.register_module("Scarab", "update_theme", move |args, _ctx| {
            let theme_json = args
                .get(0)
                .and_then(|v| v.as_str())
                .unwrap_or("{}")
                .to_string();

            channel.send_action(PluginAction::UpdateTheme {
                plugin_id: plugin_id.clone(),
                theme_json,
            });

            Ok(Value::Null)
        });
    }

    // =========================================================================
    // Keybinding Functions
    // =========================================================================

    // Scarab.register_keybinding(key: string, modifiers: list, action_id: string) -> unit
    {
        let channel = channel.clone();
        let plugin_id = default_plugin_id.clone();
        registry.register_module("Scarab", "register_keybinding", move |args, _ctx| {
            let key = args
                .get(0)
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();

            let modifiers: Vec<String> = args
                .get(1)
                .and_then(|v| v.as_list())
                .map(|list| {
                    list.iter()
                        .filter_map(|v| v.as_str().map(String::from))
                        .collect()
                })
                .unwrap_or_default();

            let action_id = args
                .get(2)
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();

            channel.send_action(PluginAction::RegisterKeybinding {
                plugin_id: plugin_id.clone(),
                key,
                modifiers,
                action_id,
            });

            Ok(Value::Null)
        });
    }

    // =========================================================================
    // Modal Functions
    // =========================================================================

    // Scarab.show_modal(title: string, items: list) -> unit
    {
        let channel = channel.clone();
        let plugin_id = default_plugin_id.clone();
        registry.register_module("Scarab", "show_modal", move |args, _ctx| {
            let title = args
                .get(0)
                .and_then(|v| v.as_str())
                .unwrap_or("Modal")
                .to_string();

            // Parse items from list of maps
            let items: Vec<ModalItem> = args
                .get(1)
                .and_then(|v| v.as_list())
                .map(|list| {
                    list.iter()
                        .filter_map(|item| {
                            let map = item.as_map()?;
                            let label = map.get("label")?.as_str()?.to_string();
                            let value = map.get("value")?.as_str()?.to_string();
                            let description = map
                                .get("description")
                                .and_then(|v| v.as_str())
                                .map(String::from);
                            Some(ModalItem {
                                label,
                                value,
                                description,
                            })
                        })
                        .collect()
                })
                .unwrap_or_default();

            channel.send_action(PluginAction::ShowModal {
                plugin_id: plugin_id.clone(),
                title,
                items,
            });

            Ok(Value::Null)
        });
    }

    // =========================================================================
    // Utility Functions
    // =========================================================================

    // Scarab.log(level: string, message: string) -> unit
    registry.register_module("Scarab", "log", |args, _ctx| {
        let level = args.get(0).and_then(|v| v.as_str()).unwrap_or("info");
        let message = args.get(1).and_then(|v| v.as_str()).unwrap_or("");

        match level {
            "debug" => debug!("[Script] {}", message),
            "info" => info!("[Script] {}", message),
            "warn" => warn!("[Script] {}", message),
            "error" => error!("[Script] {}", message),
            _ => info!("[Script] {}", message),
        }

        Ok(Value::Null)
    });

    // Scarab.version() -> string
    registry.register_module("Scarab", "version", |_args, _ctx| {
        Ok(Value::String(env!("CARGO_PKG_VERSION").to_string()))
    });

    info!(
        "Registered Scarab host functions: {:?}",
        registry.module_names().collect::<Vec<_>>()
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_register_functions() {
        let mut registry = HostRegistry::new();
        let channel = Arc::new(FusabiActionChannel::new());

        register_scarab_functions(&mut registry, channel);

        // Verify functions are registered
        assert!(registry.get_module("Scarab", "status_add").is_some());
        assert!(registry.get_module("Scarab", "notify").is_some());
        assert!(registry.get_module("Scarab", "spawn_overlay").is_some());
        assert!(registry.get_module("Scarab", "log").is_some());
        assert!(registry.get_module("Scarab", "version").is_some());
    }
}
