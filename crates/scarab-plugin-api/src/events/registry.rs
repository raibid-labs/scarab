//! Event registry for managing and dispatching events
//!
//! **DEPRECATED**: This legacy mutex-based registry is being replaced by pure Bevy ECS events.
//!
//! # Migration Path
//!
//! The `EventRegistry` pattern using `Arc<Mutex<EventRegistry>>` is deprecated in favor
//! of typed Bevy events defined in `scarab-client/src/events/bevy_events.rs`.
//!
//! **Old pattern (deprecated):**
//! ```ignore
//! let registry = Arc::new(Mutex::new(EventRegistry::new()));
//! registry.lock().unwrap().register(EventType::Bell, 100, "plugin", handler);
//! registry.lock().unwrap().dispatch(&args);
//! ```
//!
//! **New pattern (Bevy ECS):**
//! ```ignore
//! // In plugin setup:
//! app.add_event::<BellEvent>();
//!
//! // In handler system:
//! fn handle_bell(mut events: EventReader<BellEvent>) {
//!     for event in events.read() {
//!         // Handle bell event
//!     }
//! }
//! ```
//!
//! See `crates/scarab-client/src/events/bevy_events.rs` for all available typed events.

use super::{EventArgs, EventHandler, EventResult, EventType, HandlerEntry};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};

/// Central registry for event handlers
///
/// **DEPRECATED**: Use Bevy ECS events instead. See module documentation for migration guide.
///
/// This registry uses `Arc<Mutex<>>` for thread synchronization, which is incompatible
/// with Bevy's lock-free ECS architecture. The daemon still uses this for plugin-side
/// event handling, but client-side code should use typed Bevy events.
///
/// # Migration Path
///
/// - **Daemon plugins**: Continue using `DaemonEventDispatcher` which wraps this registry
/// - **Client code**: Use typed events from `scarab-client/src/events/bevy_events.rs`
/// - **New code**: Always prefer Bevy events over this registry
///
/// # Thread Safety
///
/// The registry is NOT thread-safe by default. Wrap it in a `Mutex` or `RwLock`
/// for concurrent access from multiple threads.
#[deprecated(
    since = "0.1.0",
    note = "Use Bevy ECS events (scarab-client/src/events/bevy_events.rs) instead of Arc<Mutex<EventRegistry>>. \
            See crates/scarab-plugin-api/src/events/registry.rs module docs for migration guide."
)]
pub struct EventRegistry {
    /// Handlers for standard event types
    handlers: HashMap<EventType, Vec<HandlerEntry>>,

    /// Handlers for custom event types (by custom name)
    custom_handlers: HashMap<String, Vec<HandlerEntry>>,

    /// Next available handler ID (atomic for thread-safe ID generation)
    next_handler_id: AtomicU64,
}

#[allow(deprecated)]
impl EventRegistry {
    /// Create a new empty event registry
    pub fn new() -> Self {
        Self {
            handlers: HashMap::new(),
            custom_handlers: HashMap::new(),
            next_handler_id: AtomicU64::new(1),
        }
    }

    /// Register a new event handler
    ///
    /// # Arguments
    ///
    /// * `event_type` - Type of event to listen for
    /// * `priority` - Handler priority (higher values = called first)
    /// * `plugin_name` - Name of the plugin registering this handler
    /// * `handler` - The handler function
    ///
    /// # Returns
    ///
    /// A unique handler ID that can be used to unregister the handler later.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let id = registry.register(
    ///     EventType::Bell,
    ///     100,
    ///     "my-plugin",
    ///     Box::new(|args| {
    ///         println!("Bell rang!");
    ///         EventResult::Continue
    ///     })
    /// );
    /// ```
    pub fn register(
        &mut self,
        event_type: EventType,
        priority: i32,
        plugin_name: &str,
        handler: EventHandler,
    ) -> u64 {
        let id = self.next_handler_id.fetch_add(1, Ordering::SeqCst);
        let entry = HandlerEntry::new(id, plugin_name, priority, handler);

        match event_type {
            EventType::Custom(ref name) => {
                let name = name.clone();
                let handlers = self.custom_handlers.entry(name).or_insert_with(Vec::new);
                handlers.push(entry);

                // Sort by priority (descending)
                handlers.sort_by(|a, b| b.priority.cmp(&a.priority));
            }
            _ => {
                let handlers = self.handlers.entry(event_type).or_insert_with(Vec::new);
                handlers.push(entry);

                // Sort by priority (descending)
                handlers.sort_by(|a, b| b.priority.cmp(&a.priority));
            }
        }

        id
    }

    /// Unregister a handler by ID
    ///
    /// # Returns
    ///
    /// `true` if a handler was removed, `false` if no handler with that ID was found.
    pub fn unregister(&mut self, handler_id: u64) -> bool {
        // Search in standard handlers
        for handlers in self.handlers.values_mut() {
            if let Some(pos) = handlers.iter().position(|h| h.id == handler_id) {
                handlers.remove(pos);
                return true;
            }
        }

        // Search in custom handlers
        for handlers in self.custom_handlers.values_mut() {
            if let Some(pos) = handlers.iter().position(|h| h.id == handler_id) {
                handlers.remove(pos);
                return true;
            }
        }

        false
    }

    /// Unregister all handlers for a specific plugin
    ///
    /// # Returns
    ///
    /// The number of handlers removed.
    pub fn unregister_plugin(&mut self, plugin_name: &str) -> usize {
        let mut count = 0;

        // Remove from standard handlers
        for handlers in self.handlers.values_mut() {
            let before = handlers.len();
            handlers.retain(|h| h.plugin_name != plugin_name);
            count += before - handlers.len();
        }

        // Remove from custom handlers
        for handlers in self.custom_handlers.values_mut() {
            let before = handlers.len();
            handlers.retain(|h| h.plugin_name != plugin_name);
            count += before - handlers.len();
        }

        count
    }

    /// Dispatch an event to all registered handlers
    ///
    /// Handlers are called in priority order (highest first). If any handler
    /// returns `EventResult::Stop`, subsequent handlers are not called.
    ///
    /// # Returns
    ///
    /// A vector of all results returned by handlers (may be empty if no handlers
    /// are registered or if an early handler stopped processing).
    pub fn dispatch(&self, args: &EventArgs) -> Vec<EventResult> {
        let handlers = match &args.event_type {
            EventType::Custom(name) => self.custom_handlers.get(name).map(|h| h.as_slice()),
            _ => self.handlers.get(&args.event_type).map(|h| h.as_slice()),
        };

        let Some(handlers) = handlers else {
            return Vec::new();
        };

        let mut results = Vec::with_capacity(handlers.len());

        for handler_entry in handlers {
            let result = handler_entry.call(args);

            // Check if we should stop processing
            let should_stop = result.is_stop();
            results.push(result);

            if should_stop {
                break;
            }
        }

        results
    }

    /// Get all handlers for a specific event type
    ///
    /// Returns an empty slice if no handlers are registered for that event.
    pub fn get_handlers(&self, event_type: &EventType) -> &[HandlerEntry] {
        match event_type {
            EventType::Custom(name) => self
                .custom_handlers
                .get(name)
                .map(|h| h.as_slice())
                .unwrap_or(&[]),
            _ => self
                .handlers
                .get(event_type)
                .map(|h| h.as_slice())
                .unwrap_or(&[]),
        }
    }

    /// Get the number of registered handlers for an event type
    pub fn handler_count(&self, event_type: &EventType) -> usize {
        self.get_handlers(event_type).len()
    }

    /// Get the total number of registered handlers across all events
    pub fn total_handler_count(&self) -> usize {
        let standard_count: usize = self.handlers.values().map(|v| v.len()).sum();
        let custom_count: usize = self.custom_handlers.values().map(|v| v.len()).sum();
        standard_count + custom_count
    }

    /// Clear all handlers for a specific event type
    pub fn clear_event(&mut self, event_type: &EventType) {
        match event_type {
            EventType::Custom(name) => {
                self.custom_handlers.remove(name);
            }
            _ => {
                self.handlers.remove(event_type);
            }
        }
    }

    /// Clear all handlers from the registry
    pub fn clear_all(&mut self) {
        self.handlers.clear();
        self.custom_handlers.clear();
    }

    /// Get a list of all event types that have registered handlers
    pub fn registered_events(&self) -> Vec<EventType> {
        let mut events: Vec<EventType> = self.handlers.keys().cloned().collect();

        for custom_name in self.custom_handlers.keys() {
            events.push(EventType::Custom(custom_name.clone()));
        }

        events
    }
}

#[allow(deprecated)]
impl Default for EventRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};

    #[test]
    fn test_registry_creation() {
        let registry = EventRegistry::new();
        assert_eq!(registry.total_handler_count(), 0);
    }

    #[test]
    fn test_register_and_dispatch() {
        let mut registry = EventRegistry::new();
        let called = Arc::new(Mutex::new(false));
        let called_clone = Arc::clone(&called);

        let id = registry.register(
            EventType::Bell,
            100,
            "test-plugin",
            Box::new(move |_| {
                *called_clone.lock().unwrap() = true;
                EventResult::Continue
            }),
        );

        assert_eq!(id, 1);
        assert_eq!(registry.handler_count(&EventType::Bell), 1);

        let args = EventArgs::new(EventType::Bell);
        let results = registry.dispatch(&args);

        assert_eq!(results.len(), 1);
        assert!(results[0].is_continue());
        assert!(*called.lock().unwrap());
    }

    #[test]
    fn test_priority_ordering() {
        let mut registry = EventRegistry::new();
        let order = Arc::new(Mutex::new(Vec::new()));

        for (i, priority) in [10, 50, 30].iter().enumerate() {
            let order_clone = Arc::clone(&order);
            registry.register(
                EventType::Bell,
                *priority,
                &format!("plugin-{}", i),
                Box::new(move |_| {
                    order_clone.lock().unwrap().push(*priority);
                    EventResult::Continue
                }),
            );
        }

        let args = EventArgs::new(EventType::Bell);
        registry.dispatch(&args);

        // Should be called in descending priority order: 50, 30, 10
        let order = order.lock().unwrap();
        assert_eq!(*order, vec![50, 30, 10]);
    }

    #[test]
    fn test_stop_processing() {
        let mut registry = EventRegistry::new();
        let call_count = Arc::new(Mutex::new(0));

        for i in 0..3 {
            let call_count_clone = Arc::clone(&call_count);
            registry.register(
                EventType::Bell,
                100 - i,
                &format!("plugin-{}", i),
                Box::new(move |_| {
                    *call_count_clone.lock().unwrap() += 1;
                    if i == 1 {
                        EventResult::Stop
                    } else {
                        EventResult::Continue
                    }
                }),
            );
        }

        let args = EventArgs::new(EventType::Bell);
        let results = registry.dispatch(&args);

        // Only first two handlers should be called (second one stops)
        assert_eq!(*call_count.lock().unwrap(), 2);
        assert_eq!(results.len(), 2);
        assert!(results[1].is_stop());
    }

    #[test]
    fn test_unregister() {
        let mut registry = EventRegistry::new();

        let id = registry.register(
            EventType::Bell,
            100,
            "test",
            Box::new(|_| EventResult::Continue),
        );

        assert_eq!(registry.handler_count(&EventType::Bell), 1);
        assert!(registry.unregister(id));
        assert_eq!(registry.handler_count(&EventType::Bell), 0);
        assert!(!registry.unregister(id)); // Already removed
    }

    #[test]
    fn test_unregister_plugin() {
        let mut registry = EventRegistry::new();

        registry.register(
            EventType::Bell,
            100,
            "plugin-a",
            Box::new(|_| EventResult::Continue),
        );
        registry.register(
            EventType::Bell,
            90,
            "plugin-b",
            Box::new(|_| EventResult::Continue),
        );
        registry.register(
            EventType::TabCreated,
            100,
            "plugin-a",
            Box::new(|_| EventResult::Continue),
        );

        assert_eq!(registry.total_handler_count(), 3);

        let removed = registry.unregister_plugin("plugin-a");
        assert_eq!(removed, 2);
        assert_eq!(registry.total_handler_count(), 1);
    }

    #[test]
    fn test_custom_events() {
        let mut registry = EventRegistry::new();
        let called = Arc::new(Mutex::new(false));
        let called_clone = Arc::clone(&called);

        registry.register(
            EventType::Custom("my-event".to_string()),
            100,
            "test",
            Box::new(move |_| {
                *called_clone.lock().unwrap() = true;
                EventResult::Continue
            }),
        );

        let args = EventArgs::new(EventType::Custom("my-event".to_string()));
        registry.dispatch(&args);

        assert!(*called.lock().unwrap());
    }

    #[test]
    fn test_clear_operations() {
        let mut registry = EventRegistry::new();

        registry.register(
            EventType::Bell,
            100,
            "test",
            Box::new(|_| EventResult::Continue),
        );
        registry.register(
            EventType::TabCreated,
            100,
            "test",
            Box::new(|_| EventResult::Continue),
        );

        assert_eq!(registry.total_handler_count(), 2);

        registry.clear_event(&EventType::Bell);
        assert_eq!(registry.total_handler_count(), 1);

        registry.clear_all();
        assert_eq!(registry.total_handler_count(), 0);
    }

    #[test]
    fn test_registered_events() {
        let mut registry = EventRegistry::new();

        registry.register(
            EventType::Bell,
            100,
            "test",
            Box::new(|_| EventResult::Continue),
        );
        registry.register(
            EventType::Custom("test".to_string()),
            100,
            "test",
            Box::new(|_| EventResult::Continue),
        );

        let events = registry.registered_events();
        assert_eq!(events.len(), 2);
        assert!(events.contains(&EventType::Bell));
        assert!(events.contains(&EventType::Custom("test".to_string())));
    }

    #[test]
    fn test_modified_result() {
        let mut registry = EventRegistry::new();

        registry.register(
            EventType::Output,
            100,
            "test",
            Box::new(|_| EventResult::Modified(vec![1, 2, 3])),
        );

        let args = EventArgs::new(EventType::Output);
        let results = registry.dispatch(&args);

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].as_modified(), Some(&[1, 2, 3][..]));
    }
}
