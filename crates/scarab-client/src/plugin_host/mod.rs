//! Plugin host system for Scarab terminal emulator
//!
//! Provides ECS-safe plugin hosting capabilities, allowing plugins to interact
//! with the client through events and resources. This system bridges the gap
//! between Fusabi plugins and Bevy's ECS architecture.
//!
//! # Architecture
//!
//! Plugins communicate with the client by emitting `PluginAction` events and
//! receiving `PluginResponse` events. The plugin host system processes these
//! events and manages plugin resources (overlays, status items, keybindings).
//!
//! # Example
//!
//! ```ignore
//! use scarab_client::plugin_host::ScarabPluginHostPlugin;
//! use scarab_client::events::{PluginAction, NotificationLevel};
//! use bevy::prelude::*;
//!
//! fn setup(app: &mut App) {
//!     app.add_plugins(ScarabPluginHostPlugin);
//! }
//!
//! fn plugin_system(mut actions: EventWriter<PluginAction>) {
//!     actions.send(PluginAction::ShowNotification {
//!         plugin_id: "my.plugin".to_string(),
//!         title: "Hello".to_string(),
//!         message: "World".to_string(),
//!         level: NotificationLevel::Info,
//!         duration_ms: 3000,
//!     });
//! }
//! ```

mod registry;

pub use registry::*;

use bevy::prelude::*;
use crate::events::{NavFocusableAction, NotificationLevel, PluginAction, PluginResponse};
use crate::navigation::{EnterHintModeEvent, ExitHintModeEvent, FocusableRegion, FocusableType, FocusableSource, NavAction};
use crate::ratatui_bridge::RatatuiSurface;

/// Plugin providing ECS-safe plugin hosting capabilities
///
/// This plugin initializes the plugin registry, sets up event channels,
/// and runs systems to process plugin actions.
///
/// # Systems
///
/// - `process_plugin_actions`: Processes PluginAction events and spawns
///   resources (overlays, notifications, status items)
/// - `cleanup_expired_notifications`: Removes notification entities after
///   their expiration time
/// - `cleanup_removed_overlays`: Removes overlay entities when plugins
///   are unloaded or overlays are despawned
///
/// # Resources
///
/// - `PluginRegistry`: Tracks all registered plugins and their resources
///
/// # Events
///
/// - `PluginAction`: Actions requested by plugins (input) - defined in events module
/// - `PluginResponse`: Responses sent to plugins (output) - defined in events module
pub struct ScarabPluginHostPlugin;

impl Plugin for ScarabPluginHostPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PluginRegistry>()
            .add_systems(
                Update,
                (
                    process_plugin_actions,
                    cleanup_expired_notifications,
                    cleanup_removed_overlays,
                ),
            );

        info!("ScarabPluginHostPlugin initialized");
    }
}

/// Process plugin action events and spawn/manage resources
///
/// This is the main system that handles all plugin requests. It validates
/// that plugins are enabled before processing actions, and emits appropriate
/// responses.
fn process_plugin_actions(
    mut commands: Commands,
    mut actions: EventReader<PluginAction>,
    mut responses: EventWriter<PluginResponse>,
    mut registry: ResMut<PluginRegistry>,
    time: Res<Time>,
) {
    for action in actions.read() {
        match action {
            PluginAction::SpawnOverlay {
                plugin_id,
                x,
                y,
                width,
                height,
                content: _,
                z_index,
            } => {
                // Validate plugin is enabled
                if !registry.is_enabled(plugin_id) {
                    responses.send(PluginResponse::Error {
                        plugin_id: plugin_id.clone(),
                        action: "SpawnOverlay".to_string(),
                        message: "Plugin is disabled".into(),
                    });
                    continue;
                }

                // Allocate overlay ID
                let overlay_id = registry.next_overlay_id();

                info!(
                    plugin_id = %plugin_id,
                    overlay_id = overlay_id,
                    x = x,
                    y = y,
                    width = width,
                    height = height,
                    z_index = z_index,
                    "Spawning plugin overlay"
                );

                // Spawn overlay entity using Ratatui bridge
                commands.spawn((
                    RatatuiSurface::new(*x, *y, *width, *height).with_z_index(*z_index),
                    PluginOverlay {
                        plugin_id: plugin_id.clone(),
                        overlay_id,
                    },
                    // The content will be rendered by a separate system that
                    // queries for PluginOverlay components and renders their
                    // content to the RatatuiSurface buffer
                ));

                // Track in registry
                registry.add_overlay(plugin_id, overlay_id);

                // Send success response
                responses.send(PluginResponse::OverlaySpawned {
                    plugin_id: plugin_id.clone(),
                    overlay_id,
                });
            }

            PluginAction::DespawnOverlay {
                plugin_id,
                overlay_id,
            } => {
                info!(
                    plugin_id = %plugin_id,
                    overlay_id = overlay_id,
                    "Despawning plugin overlay"
                );

                // Find and despawn the overlay entity
                // This will be handled by cleanup_removed_overlays system
                registry.remove_overlay(plugin_id, *overlay_id);
            }

            PluginAction::ShowNotification {
                plugin_id,
                title,
                message,
                level,
                duration_ms,
            } => {
                let expires_at = time.elapsed_secs_f64() + (*duration_ms as f64 / 1000.0);

                info!(
                    plugin_id = %plugin_id,
                    title = %title,
                    level = ?level,
                    duration_ms = duration_ms,
                    "Showing plugin notification"
                );

                // Spawn notification entity
                commands.spawn(PluginNotification {
                    plugin_id: plugin_id.clone(),
                    expires_at,
                });

                // TODO: Integrate with actual notification UI system
                // For now, just log the notification
                match level {
                    NotificationLevel::Info => {
                        info!("[{}] {}: {}", plugin_id, title, message);
                    }
                    NotificationLevel::Warning => {
                        warn!("[{}] {}: {}", plugin_id, title, message);
                    }
                    NotificationLevel::Error => {
                        error!("[{}] {}: {}", plugin_id, title, message);
                    }
                    NotificationLevel::Success => {
                        info!("[{}] {}: {}", plugin_id, title, message);
                    }
                }
            }

            PluginAction::AddStatusItem {
                plugin_id,
                side,
                content,
                priority,
            } => {
                if !registry.is_enabled(plugin_id) {
                    responses.send(PluginResponse::Error {
                        plugin_id: plugin_id.clone(),
                        action: "AddStatusItem".to_string(),
                        message: "Plugin is disabled".into(),
                    });
                    continue;
                }

                let item_id = registry.next_status_item_id();

                info!(
                    plugin_id = %plugin_id,
                    item_id = item_id,
                    side = ?side,
                    priority = priority,
                    content = %content,
                    "Adding plugin status item"
                );

                // Spawn status item entity
                commands.spawn(PluginStatusItem {
                    plugin_id: plugin_id.clone(),
                    item_id,
                    side: *side,
                    priority: *priority,
                });

                // Track in registry
                registry.add_status_item(plugin_id, item_id);

                // TODO: Integrate with status bar system
                // This will require updating the status bar to query for
                // PluginStatusItem components
            }

            PluginAction::RemoveStatusItem {
                plugin_id,
                item_id,
            } => {
                info!(
                    plugin_id = %plugin_id,
                    item_id = item_id,
                    "Removing plugin status item"
                );

                registry.remove_status_item(plugin_id, *item_id);

                // The actual entity cleanup will be handled by a separate system
                // that queries for despawned PluginStatusItem components
            }

            PluginAction::RegisterKeybinding {
                plugin_id,
                key,
                modifiers,
                action_id,
            } => {
                if !registry.is_enabled(plugin_id) {
                    responses.send(PluginResponse::Error {
                        plugin_id: plugin_id.clone(),
                        action: "RegisterKeybinding".to_string(),
                        message: "Plugin is disabled".into(),
                    });
                    continue;
                }

                info!(
                    plugin_id = %plugin_id,
                    key = %key,
                    modifiers = ?modifiers,
                    action_id = %action_id,
                    "Registering plugin keybinding"
                );

                // Track in registry
                registry.add_keybinding(plugin_id, action_id.clone());

                // TODO: Integrate with keybinding system
                // This will require a global keybinding registry that can
                // dispatch KeybindingTriggered responses
            }

            PluginAction::RequestTerminalContent {
                plugin_id,
                start_row,
                end_row,
            } => {
                debug!(
                    plugin_id = %plugin_id,
                    start_row = start_row,
                    end_row = end_row,
                    "Plugin requesting terminal content"
                );

                // TODO: Extract terminal content from SharedMemory or chunk system
                // For now, send empty response
                responses.send(PluginResponse::TerminalContent {
                    plugin_id: plugin_id.clone(),
                    rows: vec![],
                });
            }

            PluginAction::SendInput { plugin_id, data } => {
                debug!(
                    plugin_id = %plugin_id,
                    bytes = data.len(),
                    "Plugin sending input to terminal"
                );

                // TODO: Forward input to daemon via IPC
                // This requires access to the IPC channel
            }

            PluginAction::UpdateTheme { plugin_id, theme_json: _ } => {
                debug!(
                    plugin_id = %plugin_id,
                    "Plugin updating theme"
                );

                // TODO: Parse theme JSON and apply to renderer
            }

            PluginAction::ShowModal { plugin_id, title, items } => {
                debug!(
                    plugin_id = %plugin_id,
                    title = %title,
                    item_count = items.len(),
                    "Plugin showing modal"
                );

                // TODO: Spawn modal UI entity
            }

            PluginAction::NavEnterHintMode { plugin_id } => {
                if !registry.is_enabled(plugin_id) {
                    responses.send(PluginResponse::Error {
                        plugin_id: plugin_id.clone(),
                        action: "NavEnterHintMode".to_string(),
                        message: "Plugin is disabled".into(),
                    });
                    continue;
                }

                info!(
                    plugin_id = %plugin_id,
                    "Plugin entering hint mode"
                );

                // Emit EnterHintModeEvent to trigger hint mode
                commands.add(|world: &mut World| {
                    world.send_event(EnterHintModeEvent);
                });

                // Send success response
                responses.send(PluginResponse::NavModeEntered {
                    plugin_id: plugin_id.clone(),
                });
            }

            PluginAction::NavExitMode { plugin_id } => {
                info!(
                    plugin_id = %plugin_id,
                    "Plugin exiting navigation mode"
                );

                // Emit ExitHintModeEvent to exit hint mode
                commands.add(|world: &mut World| {
                    world.send_event(ExitHintModeEvent);
                });

                // Send success response
                responses.send(PluginResponse::NavModeExited {
                    plugin_id: plugin_id.clone(),
                });
            }

            PluginAction::NavRegisterFocusable {
                plugin_id,
                x,
                y,
                width,
                height,
                label,
                action,
            } => {
                if !registry.is_enabled(plugin_id) {
                    responses.send(PluginResponse::Error {
                        plugin_id: plugin_id.clone(),
                        action: "NavRegisterFocusable".to_string(),
                        message: "Plugin is disabled".into(),
                    });
                    continue;
                }

                // Allocate a unique focusable ID
                let focusable_id = registry.next_overlay_id(); // Reuse overlay ID counter

                info!(
                    plugin_id = %plugin_id,
                    focusable_id = focusable_id,
                    x = x,
                    y = y,
                    width = width,
                    height = height,
                    label = %label,
                    "Plugin registering focusable region"
                );

                // Convert plugin action to navigation action
                let nav_action = match action {
                    NavFocusableAction::OpenUrl(url) => NavAction::Open(url.clone()),
                    NavFocusableAction::OpenFile(path) => NavAction::Open(path.clone()),
                    NavFocusableAction::Custom(action_name) => {
                        // For custom actions, we'll use Cancel as placeholder
                        // In production, this would trigger a plugin callback
                        warn!(
                            plugin_id = %plugin_id,
                            action_name = %action_name,
                            "Custom navigation actions not yet implemented, using Cancel"
                        );
                        NavAction::Cancel
                    }
                };

                // Spawn FocusableRegion entity
                commands.spawn(FocusableRegion {
                    region_type: FocusableType::Widget, // Plugin-registered focusables are widgets
                    grid_start: (*x, *y),
                    grid_end: (*x + *width, *y + *height),
                    content: label.clone(),
                    source: FocusableSource::Ratatui, // Mark as plugin/UI source
                    screen_position: None, // Will be calculated by bounds_to_world_coords
                    pane_id: None, // TODO: Track pane_id for plugin focusables
                    generation: 0, // Plugin focusables don't use generation tracking
                });

                // Track in registry (store as overlay for lifecycle management)
                registry.add_overlay(plugin_id, focusable_id);

                // Send success response
                responses.send(PluginResponse::NavFocusableRegistered {
                    plugin_id: plugin_id.clone(),
                    focusable_id,
                });
            }

            PluginAction::NavUnregisterFocusable {
                plugin_id,
                focusable_id,
            } => {
                info!(
                    plugin_id = %plugin_id,
                    focusable_id = focusable_id,
                    "Plugin unregistering focusable region"
                );

                // Remove from registry
                registry.remove_overlay(plugin_id, *focusable_id);

                // The actual entity cleanup will happen via cleanup_removed_overlays system
                // which already handles despawning entities not in the registry

                // Send success response
                responses.send(PluginResponse::NavFocusableUnregistered {
                    plugin_id: plugin_id.clone(),
                    focusable_id: *focusable_id,
                });
            }
        }
    }
}

/// Clean up expired notification entities
///
/// Notifications have a finite lifetime specified by their `expires_at` field.
/// This system runs every frame and despawns any notifications that have
/// exceeded their lifetime.
fn cleanup_expired_notifications(
    mut commands: Commands,
    time: Res<Time>,
    notifications: Query<(Entity, &PluginNotification)>,
) {
    let now = time.elapsed_secs_f64();

    for (entity, notification) in notifications.iter() {
        if now >= notification.expires_at {
            debug!(
                plugin_id = %notification.plugin_id,
                "Cleaning up expired notification"
            );
            commands.entity(entity).despawn();
        }
    }
}

/// Clean up overlay entities that have been removed from the registry
///
/// When a plugin is unloaded or calls DespawnOverlay, the overlay ID is
/// removed from the registry. This system finds and despawns the corresponding
/// entity.
fn cleanup_removed_overlays(
    mut commands: Commands,
    registry: Res<PluginRegistry>,
    overlays: Query<(Entity, &PluginOverlay)>,
) {
    for (entity, overlay) in overlays.iter() {
        // Check if this overlay is still tracked in the registry
        if let Some(plugin) = registry.get(&overlay.plugin_id) {
            if !plugin.overlay_ids.contains(&overlay.overlay_id) {
                debug!(
                    plugin_id = %overlay.plugin_id,
                    overlay_id = overlay.overlay_id,
                    "Cleaning up removed overlay"
                );
                commands.entity(entity).despawn();
            }
        } else {
            // Plugin was unregistered, clean up all its overlays
            debug!(
                plugin_id = %overlay.plugin_id,
                overlay_id = overlay.overlay_id,
                "Cleaning up overlay from unregistered plugin"
            );
            commands.entity(entity).despawn();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::events::PluginAction;

    #[test]
    fn test_plugin_host_plugin_build() {
        let mut app = App::new();
        app.add_plugins(ScarabPluginHostPlugin);

        // Check that resources are registered
        assert!(app.world().get_resource::<PluginRegistry>().is_some());
    }

    #[test]
    fn test_process_overlay_action() {
        let mut app = App::new();
        app.add_plugins(ScarabPluginHostPlugin);
        app.add_plugins(MinimalPlugins);

        // Add PluginAction and PluginResponse events
        app.add_event::<PluginAction>();
        app.add_event::<PluginResponse>();

        // Register a test plugin
        let mut registry = app.world_mut().resource_mut::<PluginRegistry>();
        registry.register(
            "test.plugin".to_string(),
            "Test Plugin".to_string(),
            "1.0.0".to_string(),
        );
        drop(registry);

        // Send spawn overlay action
        app.world_mut()
            .send_event(PluginAction::SpawnOverlay {
                plugin_id: "test.plugin".to_string(),
                x: 10,
                y: 20,
                width: 80,
                height: 24,
                content: "Test overlay".to_string(),
                z_index: 100.0,
            });

        // Run systems
        app.update();

        // Check that overlay was spawned
        let overlays = app
            .world_mut()
            .query::<&PluginOverlay>()
            .iter(app.world())
            .count();
        assert_eq!(overlays, 1);

        // Check that response was sent
        let mut response_reader = app.world_mut().resource_mut::<Events<PluginResponse>>();
        let mut reader = response_reader.get_reader();
        let responses: Vec<_> = reader.read(&response_reader).cloned().collect();
        assert_eq!(responses.len(), 1);

        match &responses[0] {
            PluginResponse::OverlaySpawned {
                plugin_id,
                overlay_id,
            } => {
                assert_eq!(plugin_id, "test.plugin");
                assert_eq!(*overlay_id, 0);
            }
            _ => panic!("Expected OverlaySpawned response"),
        }
    }

    #[test]
    fn test_disabled_plugin_rejection() {
        let mut app = App::new();
        app.add_plugins(ScarabPluginHostPlugin);
        app.add_plugins(MinimalPlugins);

        // Add events
        app.add_event::<PluginAction>();
        app.add_event::<PluginResponse>();

        // Register a test plugin and disable it
        let mut registry = app.world_mut().resource_mut::<PluginRegistry>();
        registry.register(
            "test.plugin".to_string(),
            "Test Plugin".to_string(),
            "1.0.0".to_string(),
        );
        registry.set_enabled("test.plugin", false);
        drop(registry);

        // Send spawn overlay action (should be rejected)
        app.world_mut()
            .send_event(PluginAction::SpawnOverlay {
                plugin_id: "test.plugin".to_string(),
                x: 10,
                y: 20,
                width: 80,
                height: 24,
                content: "Test overlay".to_string(),
                z_index: 100.0,
            });

        // Run systems
        app.update();

        // Check that overlay was NOT spawned
        let overlays = app
            .world_mut()
            .query::<&PluginOverlay>()
            .iter(app.world())
            .count();
        assert_eq!(overlays, 0);

        // Check that error response was sent
        let mut response_reader = app.world_mut().resource_mut::<Events<PluginResponse>>();
        let mut reader = response_reader.get_reader();
        let responses: Vec<_> = reader.read(&response_reader).cloned().collect();
        assert_eq!(responses.len(), 1);

        match &responses[0] {
            PluginResponse::Error { plugin_id, message, .. } => {
                assert_eq!(plugin_id, "test.plugin");
                assert_eq!(message, "Plugin is disabled");
            }
            _ => panic!("Expected Error response"),
        }
    }

    #[test]
    fn test_notification_expiration() {
        let mut app = App::new();
        app.add_plugins(ScarabPluginHostPlugin);
        app.add_plugins(MinimalPlugins);

        // Add events
        app.add_event::<PluginAction>();
        app.add_event::<PluginResponse>();

        // Manually spawn a notification that's already expired
        let mut time = app.world_mut().resource_mut::<Time>();
        let now = time.elapsed_secs_f64();
        drop(time);

        app.world_mut().spawn(PluginNotification {
            plugin_id: "test.plugin".to_string(),
            expires_at: now - 1.0, // Expired 1 second ago
        });

        // Run systems
        app.update();

        // Check that notification was cleaned up
        let notifications = app
            .world_mut()
            .query::<&PluginNotification>()
            .iter(app.world())
            .count();
        assert_eq!(notifications, 0);
    }

    #[test]
    fn test_nav_enter_hint_mode() {
        let mut app = App::new();
        app.add_plugins(ScarabPluginHostPlugin);
        app.add_plugins(MinimalPlugins);

        // Add events
        app.add_event::<PluginAction>();
        app.add_event::<PluginResponse>();
        app.add_event::<EnterHintModeEvent>();

        // Register a test plugin
        let mut registry = app.world_mut().resource_mut::<PluginRegistry>();
        registry.register(
            "test.plugin".to_string(),
            "Test Plugin".to_string(),
            "1.0.0".to_string(),
        );
        drop(registry);

        // Send NavEnterHintMode action
        app.world_mut()
            .send_event(PluginAction::NavEnterHintMode {
                plugin_id: "test.plugin".to_string(),
            });

        // Run systems
        app.update();

        // Check that response was sent
        let mut response_reader = app.world_mut().resource_mut::<Events<PluginResponse>>();
        let mut reader = response_reader.get_reader();
        let responses: Vec<_> = reader.read(&response_reader).cloned().collect();
        assert_eq!(responses.len(), 1);

        match &responses[0] {
            PluginResponse::NavModeEntered { plugin_id } => {
                assert_eq!(plugin_id, "test.plugin");
            }
            _ => panic!("Expected NavModeEntered response"),
        }

        // Check that EnterHintModeEvent was emitted
        let mut hint_events = app.world_mut().resource_mut::<Events<EnterHintModeEvent>>();
        let mut hint_reader = hint_events.get_reader();
        let hint_events_list: Vec<_> = hint_reader.read(&hint_events).collect();
        assert_eq!(hint_events_list.len(), 1);
    }

    #[test]
    fn test_nav_register_focusable() {
        let mut app = App::new();
        app.add_plugins(ScarabPluginHostPlugin);
        app.add_plugins(MinimalPlugins);

        // Add events
        app.add_event::<PluginAction>();
        app.add_event::<PluginResponse>();

        // Register a test plugin
        let mut registry = app.world_mut().resource_mut::<PluginRegistry>();
        registry.register(
            "test.plugin".to_string(),
            "Test Plugin".to_string(),
            "1.0.0".to_string(),
        );
        drop(registry);

        // Send NavRegisterFocusable action
        app.world_mut()
            .send_event(PluginAction::NavRegisterFocusable {
                plugin_id: "test.plugin".to_string(),
                x: 10,
                y: 5,
                width: 20,
                height: 1,
                label: "Test Focusable".to_string(),
                action: NavFocusableAction::OpenUrl("https://example.com".to_string()),
            });

        // Run systems
        app.update();

        // Check that FocusableRegion entity was spawned
        let focusables = app
            .world_mut()
            .query::<&FocusableRegion>()
            .iter(app.world())
            .count();
        assert_eq!(focusables, 1);

        // Check the focusable properties
        let mut query = app.world_mut().query::<&FocusableRegion>();
        let region = query.single(app.world());
        assert_eq!(region.grid_start, (10, 5));
        assert_eq!(region.grid_end, (30, 6));
        assert_eq!(region.content, "Test Focusable");
        assert_eq!(region.region_type, FocusableType::Widget);
        assert_eq!(region.source, FocusableSource::Ratatui);

        // Check that response was sent
        let mut response_reader = app.world_mut().resource_mut::<Events<PluginResponse>>();
        let mut reader = response_reader.get_reader();
        let responses: Vec<_> = reader.read(&response_reader).cloned().collect();
        assert_eq!(responses.len(), 1);

        match &responses[0] {
            PluginResponse::NavFocusableRegistered {
                plugin_id,
                focusable_id,
            } => {
                assert_eq!(plugin_id, "test.plugin");
                assert_eq!(*focusable_id, 0);
            }
            _ => panic!("Expected NavFocusableRegistered response"),
        }
    }

    #[test]
    fn test_nav_exit_mode() {
        let mut app = App::new();
        app.add_plugins(ScarabPluginHostPlugin);
        app.add_plugins(MinimalPlugins);

        // Add events
        app.add_event::<PluginAction>();
        app.add_event::<PluginResponse>();
        app.add_event::<ExitHintModeEvent>();

        // Register a test plugin
        let mut registry = app.world_mut().resource_mut::<PluginRegistry>();
        registry.register(
            "test.plugin".to_string(),
            "Test Plugin".to_string(),
            "1.0.0".to_string(),
        );
        drop(registry);

        // Send NavExitMode action
        app.world_mut()
            .send_event(PluginAction::NavExitMode {
                plugin_id: "test.plugin".to_string(),
            });

        // Run systems
        app.update();

        // Check that response was sent
        let mut response_reader = app.world_mut().resource_mut::<Events<PluginResponse>>();
        let mut reader = response_reader.get_reader();
        let responses: Vec<_> = reader.read(&response_reader).cloned().collect();
        assert_eq!(responses.len(), 1);

        match &responses[0] {
            PluginResponse::NavModeExited { plugin_id } => {
                assert_eq!(plugin_id, "test.plugin");
            }
            _ => panic!("Expected NavModeExited response"),
        }

        // Check that ExitHintModeEvent was emitted
        let mut exit_events = app.world_mut().resource_mut::<Events<ExitHintModeEvent>>();
        let mut exit_reader = exit_events.get_reader();
        let exit_events_list: Vec<_> = exit_reader.read(&exit_events).collect();
        assert_eq!(exit_events_list.len(), 1);
    }

    #[test]
    fn test_nav_disabled_plugin_rejection() {
        let mut app = App::new();
        app.add_plugins(ScarabPluginHostPlugin);
        app.add_plugins(MinimalPlugins);

        // Add events
        app.add_event::<PluginAction>();
        app.add_event::<PluginResponse>();

        // Register a test plugin and disable it
        let mut registry = app.world_mut().resource_mut::<PluginRegistry>();
        registry.register(
            "test.plugin".to_string(),
            "Test Plugin".to_string(),
            "1.0.0".to_string(),
        );
        registry.set_enabled("test.plugin", false);
        drop(registry);

        // Send NavEnterHintMode action (should be rejected)
        app.world_mut()
            .send_event(PluginAction::NavEnterHintMode {
                plugin_id: "test.plugin".to_string(),
            });

        // Run systems
        app.update();

        // Check that error response was sent
        let mut response_reader = app.world_mut().resource_mut::<Events<PluginResponse>>();
        let mut reader = response_reader.get_reader();
        let responses: Vec<_> = reader.read(&response_reader).cloned().collect();
        assert_eq!(responses.len(), 1);

        match &responses[0] {
            PluginResponse::Error { plugin_id, message, .. } => {
                assert_eq!(plugin_id, "test.plugin");
                assert_eq!(message, "Plugin is disabled");
            }
            _ => panic!("Expected Error response"),
        }
    }
}
