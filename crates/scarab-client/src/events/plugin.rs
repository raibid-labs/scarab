//! Events plugin for Bevy integration

use super::{DaemonEvent, WindowFocusChangedEvent, WindowResizedEvent};
use bevy::prelude::*;
use bevy::window::WindowFocused;
use scarab_plugin_api::events::{EventArgs, EventData, EventRegistry, EventType};
use scarab_plugin_api::object_model::{ObjectHandle, ObjectType};
use scarab_protocol::{DaemonMessage, EventMessage};
use std::sync::{Arc, Mutex};
use tokio::sync::broadcast;

/// Bevy plugin for event handling
///
/// Integrates the event system with Bevy's ECS, handling both client-originated
/// events (window focus, resize) and daemon-forwarded events.
pub struct EventsPlugin {
    /// Event registry for local client-side handlers
    registry: Arc<Mutex<EventRegistry>>,
}

impl EventsPlugin {
    /// Create a new events plugin with default registry
    pub fn new() -> Self {
        Self {
            registry: Arc::new(Mutex::new(EventRegistry::new())),
        }
    }

    /// Create events plugin with existing registry
    pub fn with_registry(registry: Arc<Mutex<EventRegistry>>) -> Self {
        Self {
            registry,
        }
    }
}

impl Default for EventsPlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl Plugin for EventsPlugin {
    fn build(&self, app: &mut App) {
        // Insert the event registry as a resource
        app.insert_resource(ClientEventRegistry(Arc::clone(&self.registry)));

        // Add custom event types
        app.add_event::<WindowFocusChangedEvent>();
        app.add_event::<WindowResizedEvent>();
        app.add_event::<DaemonEvent>();

        // Add systems
        app.add_systems(
            Update,
            (
                handle_window_focus_changed,
                handle_window_resized,
                receive_daemon_events,
            ),
        );
    }
}

/// Resource wrapper for event registry
#[derive(Resource, Clone)]
pub struct ClientEventRegistry(pub Arc<Mutex<EventRegistry>>);

/// Resource wrapper for daemon event receiver
#[derive(Resource)]
pub struct DaemonEventReceiver(pub Arc<Mutex<broadcast::Receiver<DaemonMessage>>>);

/// System to handle window focus changes
fn handle_window_focus_changed(
    mut focus_events: EventReader<WindowFocused>,
    mut custom_events: EventWriter<WindowFocusChangedEvent>,
    registry: Option<Res<ClientEventRegistry>>,
) {
    for event in focus_events.read() {
        // Emit custom event for other systems
        custom_events.send(WindowFocusChangedEvent {
            is_focused: event.focused,
        });

        // Dispatch through event registry if available
        if let Some(registry) = &registry {
            let window_handle = ObjectHandle::new(ObjectType::Window, 0, 0); // TODO: Use actual window ID
            let args = EventArgs::new(EventType::WindowFocusChanged)
                .with_window(window_handle)
                .with_data(EventData::FocusState {
                    is_focused: event.focused,
                });

            let registry = registry.0.lock().unwrap();
            registry.dispatch(&args);
        }
    }
}

/// System to handle window resize events
fn handle_window_resized(
    mut resize_events: EventReader<bevy::window::WindowResized>,
    mut custom_events: EventWriter<WindowResizedEvent>,
    registry: Option<Res<ClientEventRegistry>>,
) {
    for event in resize_events.read() {
        // Convert physical pixels to terminal cells
        // TODO: Use actual cell dimensions from rendering system
        let cell_width = 9.0;
        let cell_height = 18.0;
        let cols = (event.width / cell_width).floor() as u16;
        let rows = (event.height / cell_height).floor() as u16;

        // Emit custom event
        custom_events.send(WindowResizedEvent { cols, rows });

        // Dispatch through event registry if available
        if let Some(registry) = &registry {
            let window_handle = ObjectHandle::new(ObjectType::Window, 0, 0); // TODO: Use actual window ID
            let args = EventArgs::new(EventType::WindowResized)
                .with_window(window_handle)
                .with_data(EventData::Dimensions { cols, rows });

            let registry = registry.0.lock().unwrap();
            registry.dispatch(&args);
        }
    }
}

/// System to receive and process events from the daemon
fn receive_daemon_events(
    receiver: Option<Res<DaemonEventReceiver>>,
    mut daemon_events: EventWriter<DaemonEvent>,
    registry: Option<Res<ClientEventRegistry>>,
) {
    let Some(receiver) = receiver else {
        return;
    };

    let mut receiver = receiver.0.lock().unwrap();

    // Process all available messages without blocking
    loop {
        match receiver.try_recv() {
            Ok(DaemonMessage::Event(event_msg)) => {
                // Emit Bevy event for other systems
                daemon_events.send(DaemonEvent {
                    event_type: event_msg.event_type.to_string(),
                    window_id: event_msg.window_id,
                    pane_id: event_msg.pane_id,
                    tab_id: event_msg.tab_id,
                    data: event_msg.data.to_vec(),
                    timestamp_micros: event_msg.timestamp_micros,
                });

                // Dispatch through local registry if available
                if let Some(registry) = &registry {
                    if let Some(args) = convert_event_message_to_args(&event_msg) {
                        let registry = registry.0.lock().unwrap();
                        registry.dispatch(&args);
                    }
                }
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

/// Convert an EventMessage from the daemon to EventArgs
fn convert_event_message_to_args(msg: &EventMessage) -> Option<EventArgs> {
    // Parse event type from name
    let event_type = match msg.event_type.as_str() {
        "bell" => EventType::Bell,
        "pane-title-changed" => EventType::PaneTitleChanged,
        "window-created" => EventType::WindowCreated,
        "window-closed" => EventType::WindowClosed,
        "window-focus-changed" => EventType::WindowFocusChanged,
        "window-resized" => EventType::WindowResized,
        "tab-created" => EventType::TabCreated,
        "tab-closed" => EventType::TabClosed,
        "pane-created" => EventType::PaneCreated,
        "pane-closed" => EventType::PaneClosed,
        _ => {
            // Unknown or custom event
            warn!("Unknown event type from daemon: {}", msg.event_type);
            return None;
        }
    };

    // Create object handles
    let window = msg.window_id.map(|id| ObjectHandle::new(ObjectType::Window, id, 0));
    let pane = msg.pane_id.map(|id| ObjectHandle::new(ObjectType::Pane, id, 0));
    let tab = msg.tab_id.map(|id| ObjectHandle::new(ObjectType::Tab, id, 0));

    // Parse event data
    let data = parse_event_data(&event_type, &msg.data);

    let mut args = EventArgs::new(event_type).with_data(data);

    if let Some(w) = window {
        args = args.with_window(w);
    }
    if let Some(p) = pane {
        args = args.with_pane(p);
    }
    if let Some(t) = tab {
        args = args.with_tab(t);
    }

    Some(args)
}

/// Parse event data from bytes
fn parse_event_data(event_type: &EventType, data: &[u8]) -> EventData {
    if data.is_empty() {
        return EventData::None;
    }

    match event_type {
        EventType::PaneTitleChanged => {
            // Parse JSON: {"old": "...", "new": "..."}
            if let Ok(json) = serde_json::from_slice::<serde_json::Value>(data) {
                if let (Some(old), Some(new)) = (json.get("old"), json.get("new")) {
                    if let (Some(old_str), Some(new_str)) = (old.as_str(), new.as_str()) {
                        return EventData::TitleChange {
                            old: old_str.to_string(),
                            new: new_str.to_string(),
                        };
                    }
                }
            }
            EventData::None
        }
        EventType::WindowFocusChanged => {
            // Parse JSON: {"is_focused": bool}
            if let Ok(json) = serde_json::from_slice::<serde_json::Value>(data) {
                if let Some(is_focused) = json.get("is_focused").and_then(|v| v.as_bool()) {
                    return EventData::FocusState { is_focused };
                }
            }
            EventData::None
        }
        EventType::WindowResized => {
            // Parse JSON: {"cols": u16, "rows": u16}
            if let Ok(json) = serde_json::from_slice::<serde_json::Value>(data) {
                if let (Some(cols), Some(rows)) = (
                    json.get("cols").and_then(|v| v.as_u64()),
                    json.get("rows").and_then(|v| v.as_u64()),
                ) {
                    return EventData::Dimensions {
                        cols: cols as u16,
                        rows: rows as u16,
                    };
                }
            }
            EventData::None
        }
        _ => {
            // Default to binary data
            EventData::Binary(data.to_vec())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plugin_creation() {
        let _plugin = EventsPlugin::new();
        // Plugin created successfully
    }

    #[test]
    fn test_convert_event_message() {
        let msg = EventMessage {
            event_type: "bell".to_string().into(),
            window_id: Some(1),
            pane_id: Some(2),
            tab_id: None,
            data: vec![],
            timestamp_micros: 123456,
        };

        let args = convert_event_message_to_args(&msg).unwrap();
        assert_eq!(args.event_type, EventType::Bell);
        assert_eq!(args.window.unwrap().id(), 1);
        assert_eq!(args.pane.unwrap().id(), 2);
    }

    #[test]
    fn test_parse_title_change_data() {
        let json = r#"{"old": "old title", "new": "new title"}"#;
        let data = json.as_bytes();

        let event_data = parse_event_data(&EventType::PaneTitleChanged, data);

        if let EventData::TitleChange { old, new } = event_data {
            assert_eq!(old, "old title");
            assert_eq!(new, "new title");
        } else {
            panic!("Expected TitleChange data");
        }
    }
}
