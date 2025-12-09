//! Daemon event dispatcher
//!
//! Dispatches events through the event registry and forwards them to clients via IPC.
//!
//! # Note on EventRegistry Usage
//!
//! This dispatcher wraps the **legacy** `EventRegistry` for daemon-side plugin compatibility.
//! The registry itself is deprecated for client-side use, but remains necessary on the daemon
//! for Fusabi plugin event handling.
//!
//! **Event Flow:**
//! ```text
//! Daemon Plugin → EventRegistry → DaemonEventDispatcher → IPC → Client
//!                                                                  ↓
//!                                                          Bevy Events (typed)
//! ```
//!
//! Client-side code should use typed Bevy events from `scarab-client/src/events/bevy_events.rs`
//! instead of directly accessing the registry.

// Allow use of deprecated EventRegistry - it's still needed for daemon-side plugin compatibility
#[allow(deprecated)]
use scarab_plugin_api::events::{EventArgs, EventData, EventRegistry, EventResult, EventType};
use scarab_plugin_api::object_model::ObjectHandle;
use scarab_protocol::{DaemonMessage, EventMessage};
use std::sync::{Arc, Mutex};
use tokio::sync::broadcast;

/// Daemon-side event dispatcher
///
/// Wraps an EventRegistry and provides IPC forwarding capabilities.
/// Events are dispatched locally to plugins and forwarded to connected clients.
///
/// # Architecture Note
///
/// This dispatcher continues to use `Arc<Mutex<EventRegistry>>` for daemon-side plugin
/// compatibility. Events are:
/// 1. Dispatched to local daemon plugins via the registry
/// 2. Forwarded to clients via IPC
/// 3. Converted to typed Bevy events on the client side
///
/// **Do not use this pattern in client code** - use Bevy events instead.
#[allow(deprecated)]
pub struct DaemonEventDispatcher {
    /// Event registry for local plugin handlers (daemon-side only)
    ///
    /// Note: This uses the legacy EventRegistry pattern for daemon plugin compatibility.
    /// Client-side code uses typed Bevy events instead.
    registry: Arc<Mutex<EventRegistry>>,

    /// Broadcast channel for forwarding events to clients
    ipc_sender: Option<broadcast::Sender<DaemonMessage>>,
}

#[allow(deprecated)]
impl DaemonEventDispatcher {
    /// Create a new daemon event dispatcher
    pub fn new(registry: Arc<Mutex<EventRegistry>>) -> Self {
        Self {
            registry,
            ipc_sender: None,
        }
    }

    /// Set the IPC sender for forwarding events to clients
    pub fn set_ipc_sender(&mut self, sender: broadcast::Sender<DaemonMessage>) {
        self.ipc_sender = Some(sender);
    }

    /// Dispatch an event to local handlers and forward to clients
    ///
    /// # Arguments
    ///
    /// * `args` - Event arguments containing event type and context
    ///
    /// # Returns
    ///
    /// Results from all local handlers that were executed
    pub fn dispatch(&self, args: &EventArgs) -> Vec<EventResult> {
        // Dispatch to local handlers first
        let results = {
            let registry = match self.registry.lock() {
                Ok(guard) => guard,
                Err(poisoned) => {
                    log::warn!("Event registry lock poisoned, recovering");
                    poisoned.into_inner()
                }
            };
            registry.dispatch(args)
        };

        // Forward to clients if IPC is configured
        if let Some(sender) = &self.ipc_sender {
            if let Some(event_msg) = self.create_event_message(args) {
                // Best-effort send - ignore if no receivers
                let _ = sender.send(DaemonMessage::Event(event_msg));
            }
        }

        results
    }

    /// Dispatch an event by type with minimal context
    ///
    /// Convenience method for simple events without object handles.
    pub fn dispatch_simple(&self, event_type: EventType) {
        let args = EventArgs::new(event_type);
        self.dispatch(&args);
    }

    /// Dispatch a bell event
    ///
    /// # Arguments
    ///
    /// * `pane` - Handle to the pane that rang the bell
    pub fn dispatch_bell(&self, pane: ObjectHandle) {
        let args = EventArgs::new(EventType::Bell).with_pane(pane);
        self.dispatch(&args);
    }

    /// Dispatch a pane title changed event
    ///
    /// # Arguments
    ///
    /// * `pane` - Handle to the pane whose title changed
    /// * `old_title` - Previous title
    /// * `new_title` - New title
    pub fn dispatch_pane_title_changed(
        &self,
        pane: ObjectHandle,
        old_title: String,
        new_title: String,
    ) {
        let args = EventArgs::new(EventType::PaneTitleChanged)
            .with_pane(pane)
            .with_data(EventData::TitleChange {
                old: old_title,
                new: new_title,
            });
        self.dispatch(&args);
    }

    /// Create an IPC event message from event args
    fn create_event_message(&self, args: &EventArgs) -> Option<EventMessage> {
        // Serialize event data
        let data = self.serialize_event_data(&args.data);

        // Extract object IDs
        let window_id = args.window.map(|h| h.id());
        let pane_id = args.pane.map(|h| h.id());
        let tab_id = args.tab.map(|h| h.id());

        // Get timestamp in microseconds
        let timestamp_micros = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .ok()?
            .as_micros() as u64;

        Some(EventMessage {
            event_type: args.event_type.name().into(),
            window_id,
            pane_id,
            tab_id,
            data,
            timestamp_micros,
        })
    }

    /// Serialize event data to bytes
    fn serialize_event_data(&self, data: &EventData) -> Vec<u8> {
        // Use simple JSON serialization for now
        // In production, consider using bincode or msgpack for efficiency
        match data {
            EventData::None => Vec::new(),
            EventData::Text(s) => s.as_bytes().to_vec(),
            EventData::Uri(s) => s.as_bytes().to_vec(),
            EventData::FocusState { is_focused } => {
                serde_json::to_vec(&serde_json::json!({ "is_focused": is_focused }))
                    .unwrap_or_default()
            }
            EventData::Dimensions { cols, rows } => {
                serde_json::to_vec(&serde_json::json!({ "cols": cols, "rows": rows }))
                    .unwrap_or_default()
            }
            EventData::TitleChange { old, new } => {
                serde_json::to_vec(&serde_json::json!({ "old": old, "new": new }))
                    .unwrap_or_default()
            }
            EventData::UserVar { name, value } => {
                serde_json::to_vec(&serde_json::json!({ "name": name, "value": value }))
                    .unwrap_or_default()
            }
            EventData::Selection { text, start, end } => serde_json::to_vec(&serde_json::json!({
                "text": text,
                "start": start,
                "end": end
            }))
            .unwrap_or_default(),
            EventData::ExitCode(code) => {
                serde_json::to_vec(&serde_json::json!({ "exit_code": code })).unwrap_or_default()
            }
            EventData::Binary(bytes) => bytes.clone(),
        }
    }

    /// Register a new event handler
    ///
    /// # Arguments
    ///
    /// * `event_type` - Type of event to listen for
    /// * `priority` - Handler priority (higher = called first)
    /// * `plugin_name` - Name of the plugin registering this handler
    /// * `handler` - The handler function
    ///
    /// # Returns
    ///
    /// A unique handler ID
    pub fn register_handler<F>(
        &self,
        event_type: EventType,
        priority: i32,
        plugin_name: &str,
        handler: F,
    ) -> u64
    where
        F: Fn(&EventArgs) -> EventResult + Send + Sync + 'static,
    {
        let mut registry = match self.registry.lock() {
            Ok(guard) => guard,
            Err(poisoned) => {
                log::warn!("Event registry lock poisoned during handler registration, recovering");
                poisoned.into_inner()
            }
        };
        registry.register(event_type, priority, plugin_name, Box::new(handler))
    }

    /// Unregister a handler by ID
    pub fn unregister_handler(&self, handler_id: u64) -> bool {
        let mut registry = match self.registry.lock() {
            Ok(guard) => guard,
            Err(poisoned) => {
                log::warn!("Event registry lock poisoned during handler unregistration, recovering");
                poisoned.into_inner()
            }
        };
        registry.unregister(handler_id)
    }

    /// Get access to the underlying registry
    pub fn registry(&self) -> Arc<Mutex<EventRegistry>> {
        Arc::clone(&self.registry)
    }
}

#[allow(deprecated)]
impl Default for DaemonEventDispatcher {
    fn default() -> Self {
        Self::new(Arc::new(Mutex::new(EventRegistry::new())))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use scarab_plugin_api::object_model::ObjectType;
    use std::sync::atomic::{AtomicBool, Ordering};

    #[test]
    fn test_dispatcher_creation() {
        let dispatcher = DaemonEventDispatcher::default();
        assert!(dispatcher.ipc_sender.is_none());
    }

    #[test]
    fn test_dispatch_simple() {
        let dispatcher = DaemonEventDispatcher::default();
        let called = Arc::new(AtomicBool::new(false));
        let called_clone = Arc::clone(&called);

        dispatcher.register_handler(EventType::Bell, 100, "test", move |_| {
            called_clone.store(true, Ordering::SeqCst);
            EventResult::Continue
        });

        dispatcher.dispatch_simple(EventType::Bell);
        assert!(called.load(Ordering::SeqCst));
    }

    #[test]
    fn test_dispatch_bell() {
        let dispatcher = DaemonEventDispatcher::default();
        let called = Arc::new(AtomicBool::new(false));
        let called_clone = Arc::clone(&called);

        dispatcher.register_handler(EventType::Bell, 100, "test", move |args| {
            assert!(args.has_pane());
            called_clone.store(true, Ordering::SeqCst);
            EventResult::Continue
        });

        let pane = ObjectHandle::new(ObjectType::Pane, 1, 0);
        dispatcher.dispatch_bell(pane);
        assert!(called.load(Ordering::SeqCst));
    }

    #[test]
    fn test_dispatch_title_changed() {
        let dispatcher = DaemonEventDispatcher::default();
        let called = Arc::new(AtomicBool::new(false));
        let called_clone = Arc::clone(&called);

        dispatcher.register_handler(EventType::PaneTitleChanged, 100, "test", move |args| {
            let (old, new) = args.data.as_title_change().unwrap();
            assert_eq!(old, "old");
            assert_eq!(new, "new");
            called_clone.store(true, Ordering::SeqCst);
            EventResult::Continue
        });

        let pane = ObjectHandle::new(ObjectType::Pane, 1, 0);
        dispatcher.dispatch_pane_title_changed(pane, "old".to_string(), "new".to_string());
        assert!(called.load(Ordering::SeqCst));
    }

    #[test]
    fn test_event_message_creation() {
        let dispatcher = DaemonEventDispatcher::default();
        let pane = ObjectHandle::new(ObjectType::Pane, 42, 0);

        let args = EventArgs::new(EventType::Bell)
            .with_pane(pane)
            .with_data(EventData::Text("test".to_string()));

        let msg = dispatcher.create_event_message(&args).unwrap();

        assert_eq!(msg.event_type, "bell");
        assert_eq!(msg.pane_id, Some(42));
        assert_eq!(msg.data, b"test");
    }

    #[test]
    fn test_register_unregister() {
        let dispatcher = DaemonEventDispatcher::default();

        let id =
            dispatcher.register_handler(EventType::Bell, 100, "test", |_| EventResult::Continue);

        assert!(dispatcher.unregister_handler(id));
        assert!(!dispatcher.unregister_handler(id)); // Already removed
    }
}
