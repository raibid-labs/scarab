//! Events plugin for Bevy integration
//!
//! This module provides pure ECS event handling, replacing the Arc<Mutex<EventRegistry>>
//! pattern with typed Bevy events.

use super::bevy_events::*;
use bevy::prelude::*;
use bevy::window::WindowFocused;
use scarab_plugin_api::object_model::{ObjectHandle, ObjectType};
use scarab_protocol::{DaemonMessage, EventMessage};
use std::sync::{Arc, Mutex};
use tokio::sync::broadcast;

/// Bevy plugin for event handling
///
/// Integrates the event system with Bevy's ECS, handling both client-originated
/// events (window focus, resize) and daemon-forwarded events.
///
/// This plugin uses pure Bevy events (no Arc<Mutex<EventRegistry>>) for lock-free event handling.
pub struct EventsPlugin;

impl Plugin for EventsPlugin {
    fn build(&self, app: &mut App) {
        // Register all window events
        app.add_event::<WindowCreatedEvent>()
            .add_event::<WindowClosedEvent>()
            .add_event::<WindowFocusChangedEvent>()
            .add_event::<WindowResizedEvent>()
            .add_event::<WindowConfigReloadedEvent>()
            .add_event::<GuiStartupEvent>();

        // Register all tab events
        app.add_event::<TabCreatedEvent>()
            .add_event::<TabClosedEvent>()
            .add_event::<TabSwitchedEvent>()
            .add_event::<NewTabButtonClickEvent>();

        // Register all pane events
        app.add_event::<PaneCreatedEvent>()
            .add_event::<PaneClosedEvent>()
            .add_event::<PaneFocusedEvent>()
            .add_event::<PaneTitleChangedEvent>();

        // Register all terminal events
        app.add_event::<BellEvent>()
            .add_event::<SelectionChangedEvent>()
            .add_event::<UserVarChangedEvent>()
            .add_event::<OpenUriEvent>()
            .add_event::<ScrollbackClearedEvent>();

        // Register all status events
        app.add_event::<UpdateStatusEvent>()
            .add_event::<UpdateRightStatusEvent>()
            .add_event::<UpdateLeftStatusEvent>()
            .add_event::<FormatTabTitleEvent>()
            .add_event::<FormatWindowTitleEvent>();

        // Register all legacy hook events
        app.add_event::<OutputEvent>()
            .add_event::<InputEvent>()
            .add_event::<PreCommandEvent>()
            .add_event::<PostCommandEvent>()
            .add_event::<ResizeEvent>()
            .add_event::<AttachEvent>()
            .add_event::<DetachEvent>();

        // Register custom and daemon events
        app.add_event::<CustomEvent>()
            .add_event::<DaemonEvent>();

        // Add systems for handling Bevy window events
        app.add_systems(
            Update,
            (
                handle_bevy_window_focus,
                handle_bevy_window_resize,
                receive_daemon_events,
            ),
        );
    }
}

impl Default for EventsPlugin {
    fn default() -> Self {
        Self
    }
}

/// Resource wrapper for daemon event receiver
#[derive(Resource)]
pub struct DaemonEventReceiver(pub Arc<Mutex<broadcast::Receiver<DaemonMessage>>>);

/// System to handle Bevy window focus events and convert to typed events
fn handle_bevy_window_focus(
    mut bevy_focus_events: EventReader<WindowFocused>,
    mut focus_events: EventWriter<WindowFocusChangedEvent>,
) {
    for event in bevy_focus_events.read() {
        // TODO: Use actual window ID instead of hardcoded 0
        let window_handle = ObjectHandle::new(ObjectType::Window, 0, 0);

        focus_events.send(WindowFocusChangedEvent {
            window: window_handle,
            is_focused: event.focused,
        });
    }
}

/// System to handle Bevy window resize events and convert to typed events
fn handle_bevy_window_resize(
    mut bevy_resize_events: EventReader<bevy::window::WindowResized>,
    mut resize_events: EventWriter<WindowResizedEvent>,
) {
    for event in bevy_resize_events.read() {
        // Convert physical pixels to terminal cells
        // TODO: Use actual cell dimensions from rendering system
        let cell_width = 9.0;
        let cell_height = 18.0;
        let cols = (event.width / cell_width).floor() as u16;
        let rows = (event.height / cell_height).floor() as u16;

        // TODO: Use actual window ID instead of hardcoded 0
        let window_handle = ObjectHandle::new(ObjectType::Window, 0, 0);

        resize_events.send(WindowResizedEvent {
            window: window_handle,
            cols,
            rows,
        });
    }
}

/// System to receive and process events from the daemon
fn receive_daemon_events(
    receiver: Option<Res<DaemonEventReceiver>>,
    mut daemon_events: EventWriter<DaemonEvent>,
    // Event writers for all typed events
    mut bell_events: EventWriter<BellEvent>,
    mut title_changed_events: EventWriter<PaneTitleChangedEvent>,
    mut window_created_events: EventWriter<WindowCreatedEvent>,
    mut window_closed_events: EventWriter<WindowClosedEvent>,
    mut window_focus_events: EventWriter<WindowFocusChangedEvent>,
    mut window_resize_events: EventWriter<WindowResizedEvent>,
    mut tab_created_events: EventWriter<TabCreatedEvent>,
    mut tab_closed_events: EventWriter<TabClosedEvent>,
    mut pane_created_events: EventWriter<PaneCreatedEvent>,
    mut pane_closed_events: EventWriter<PaneClosedEvent>,
) {
    let Some(receiver) = receiver else {
        return;
    };

    let mut receiver = match receiver.0.lock() {
        Ok(r) => r,
        Err(e) => {
            error!("Failed to lock daemon event receiver: {}", e);
            return;
        }
    };

    // Process all available messages without blocking
    loop {
        match receiver.try_recv() {
            Ok(DaemonMessage::Event(event_msg)) => {
                // Emit generic daemon event for debugging/logging
                daemon_events.send(DaemonEvent {
                    event_type: event_msg.event_type.to_string(),
                    window_id: event_msg.window_id,
                    pane_id: event_msg.pane_id,
                    tab_id: event_msg.tab_id,
                    data: event_msg.data.to_vec(),
                    timestamp_micros: event_msg.timestamp_micros,
                });

                // Convert to typed event and dispatch
                convert_and_dispatch_event(
                    &event_msg,
                    &mut bell_events,
                    &mut title_changed_events,
                    &mut window_created_events,
                    &mut window_closed_events,
                    &mut window_focus_events,
                    &mut window_resize_events,
                    &mut tab_created_events,
                    &mut tab_closed_events,
                    &mut pane_created_events,
                    &mut pane_closed_events,
                );
            }
            Ok(_other_message) => {
                // Other daemon messages (TabCreated, etc.) - not event system messages
                // These are handled elsewhere in the system
            }
            Err(broadcast::error::TryRecvError::Empty) => {
                // No more messages available
                break;
            }
            Err(broadcast::error::TryRecvError::Lagged(skipped)) => {
                warn!("Daemon event receiver lagged, skipped {} messages", skipped);
                // Continue processing available messages
            }
            Err(broadcast::error::TryRecvError::Closed) => {
                warn!("Daemon event channel closed");
                break;
            }
        }
    }
}

/// Convert an EventMessage from the daemon to typed Bevy events
#[allow(clippy::too_many_arguments)]
fn convert_and_dispatch_event(
    msg: &EventMessage,
    bell_events: &mut EventWriter<BellEvent>,
    title_changed_events: &mut EventWriter<PaneTitleChangedEvent>,
    window_created_events: &mut EventWriter<WindowCreatedEvent>,
    window_closed_events: &mut EventWriter<WindowClosedEvent>,
    window_focus_events: &mut EventWriter<WindowFocusChangedEvent>,
    window_resize_events: &mut EventWriter<WindowResizedEvent>,
    tab_created_events: &mut EventWriter<TabCreatedEvent>,
    tab_closed_events: &mut EventWriter<TabClosedEvent>,
    pane_created_events: &mut EventWriter<PaneCreatedEvent>,
    pane_closed_events: &mut EventWriter<PaneClosedEvent>,
) {
    // Create object handles
    let window = msg
        .window_id
        .map(|id| ObjectHandle::new(ObjectType::Window, id, 0));
    let pane = msg
        .pane_id
        .map(|id| ObjectHandle::new(ObjectType::Pane, id, 0));
    let tab = msg
        .tab_id
        .map(|id| ObjectHandle::new(ObjectType::Tab, id, 0));

    // Dispatch to appropriate typed event
    match msg.event_type.as_str() {
        "bell" => {
            if let Some(pane) = pane {
                bell_events.send(BellEvent { pane });
            }
        }
        "pane-title-changed" => {
            if let Some(pane) = pane {
                // Parse JSON: {"old": "...", "new": "..."}
                if let Ok(json) = serde_json::from_slice::<serde_json::Value>(&msg.data) {
                    if let (Some(old), Some(new)) = (json.get("old"), json.get("new")) {
                        if let (Some(old_str), Some(new_str)) = (old.as_str(), new.as_str()) {
                            title_changed_events.send(PaneTitleChangedEvent {
                                pane,
                                old_title: old_str.to_string(),
                                new_title: new_str.to_string(),
                            });
                        }
                    }
                }
            }
        }
        "window-created" => {
            if let Some(window) = window {
                window_created_events.send(WindowCreatedEvent { window });
            }
        }
        "window-closed" => {
            if let Some(window) = window {
                window_closed_events.send(WindowClosedEvent { window });
            }
        }
        "window-focus-changed" => {
            if let Some(window) = window {
                // Parse JSON: {"is_focused": bool}
                let is_focused = if let Ok(json) =
                    serde_json::from_slice::<serde_json::Value>(&msg.data)
                {
                    json.get("is_focused")
                        .and_then(|v| v.as_bool())
                        .unwrap_or(false)
                } else {
                    false
                };

                window_focus_events.send(WindowFocusChangedEvent {
                    window,
                    is_focused,
                });
            }
        }
        "window-resized" => {
            if let Some(window) = window {
                // Parse JSON: {"cols": u16, "rows": u16}
                if let Ok(json) = serde_json::from_slice::<serde_json::Value>(&msg.data) {
                    if let (Some(cols), Some(rows)) = (
                        json.get("cols").and_then(|v| v.as_u64()),
                        json.get("rows").and_then(|v| v.as_u64()),
                    ) {
                        window_resize_events.send(WindowResizedEvent {
                            window,
                            cols: cols as u16,
                            rows: rows as u16,
                        });
                    }
                }
            }
        }
        "tab-created" => {
            if let (Some(window), Some(tab)) = (window, tab) {
                tab_created_events.send(TabCreatedEvent { window, tab });
            }
        }
        "tab-closed" => {
            if let (Some(window), Some(tab)) = (window, tab) {
                tab_closed_events.send(TabClosedEvent { window, tab });
            }
        }
        "pane-created" => {
            if let (Some(window), Some(tab), Some(pane)) = (window, tab, pane) {
                pane_created_events.send(PaneCreatedEvent { window, tab, pane });
            }
        }
        "pane-closed" => {
            if let (Some(window), Some(tab), Some(pane)) = (window, tab, pane) {
                pane_closed_events.send(PaneClosedEvent { window, tab, pane });
            }
        }
        _ => {
            // Unknown event type - logged by daemon event
            debug!("Unknown event type from daemon: {}", msg.event_type);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plugin_creation() {
        let _plugin = EventsPlugin;
        // Plugin created successfully
    }

    #[test]
    fn test_plugin_registration() {
        let mut app = App::new();
        app.add_plugins(EventsPlugin);

        // Verify events are registered by checking that we can send them
        app.world_mut().send_event(BellEvent {
            pane: ObjectHandle::new(ObjectType::Pane, 1, 0),
        });

        app.world_mut().send_event(WindowFocusChangedEvent {
            window: ObjectHandle::new(ObjectType::Window, 1, 0),
            is_focused: true,
        });

        // If we get here without panic, events are registered
    }
}
