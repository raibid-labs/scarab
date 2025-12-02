//! Event handler types
//!
//! Defines the callback function types and result enums for event handlers.

use super::EventArgs;

/// Event handler function type
///
/// Handlers are called when events are dispatched. They receive event arguments
/// and return a result indicating how to proceed with event processing.
///
/// Handlers must be `Send + Sync` to allow safe concurrent access across threads.
pub type EventHandler = Box<dyn Fn(&EventArgs) -> EventResult + Send + Sync>;

/// Result returned by event handlers
///
/// Determines how the event dispatch should proceed after a handler executes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EventResult {
    /// Continue processing with the next handler
    ///
    /// This is the default behavior - the event will be passed to the next
    /// handler in the priority chain.
    Continue,

    /// Stop processing and don't call remaining handlers
    ///
    /// Use this when an event has been fully handled and no further processing
    /// is needed. Subsequent handlers will not be called.
    Stop,

    /// Modify the event data and continue
    ///
    /// Used primarily for legacy hook compatibility where handlers can transform
    /// data (e.g., filtering output, modifying input). The modified data will be
    /// passed to subsequent handlers.
    Modified(Vec<u8>),
}

impl EventResult {
    /// Check if this result indicates continuation
    pub fn is_continue(&self) -> bool {
        matches!(self, EventResult::Continue)
    }

    /// Check if this result indicates stopping
    pub fn is_stop(&self) -> bool {
        matches!(self, EventResult::Stop)
    }

    /// Check if this result contains modified data
    pub fn is_modified(&self) -> bool {
        matches!(self, EventResult::Modified(_))
    }

    /// Extract modified data if present
    pub fn into_modified(self) -> Option<Vec<u8>> {
        match self {
            EventResult::Modified(data) => Some(data),
            _ => None,
        }
    }

    /// Get a reference to modified data if present
    pub fn as_modified(&self) -> Option<&[u8]> {
        match self {
            EventResult::Modified(data) => Some(data),
            _ => None,
        }
    }
}

/// Entry in the event handler registry
///
/// Stores handler metadata and the handler function itself.
pub struct HandlerEntry {
    /// Unique handler ID
    pub id: u64,

    /// Name of the plugin that registered this handler
    pub plugin_name: String,

    /// Handler priority (higher = called first)
    ///
    /// Handlers are sorted by priority in descending order. Typical priorities:
    /// - 1000+: Critical system handlers
    /// - 100-999: High priority plugin handlers
    /// - 0-99: Normal priority handlers
    /// - Negative: Low priority background handlers
    pub priority: i32,

    /// The handler function
    pub handler: EventHandler,
}

impl HandlerEntry {
    /// Create a new handler entry
    pub fn new(
        id: u64,
        plugin_name: impl Into<String>,
        priority: i32,
        handler: EventHandler,
    ) -> Self {
        Self {
            id,
            plugin_name: plugin_name.into(),
            priority,
            handler,
        }
    }

    /// Call this handler with event arguments
    pub fn call(&self, args: &EventArgs) -> EventResult {
        (self.handler)(args)
    }
}

impl std::fmt::Debug for HandlerEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HandlerEntry")
            .field("id", &self.id)
            .field("plugin_name", &self.plugin_name)
            .field("priority", &self.priority)
            .field("handler", &"<function>")
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::events::EventType;

    #[test]
    fn test_event_result_checks() {
        assert!(EventResult::Continue.is_continue());
        assert!(!EventResult::Continue.is_stop());
        assert!(!EventResult::Continue.is_modified());

        assert!(EventResult::Stop.is_stop());
        assert!(!EventResult::Stop.is_continue());

        let modified = EventResult::Modified(vec![1, 2, 3]);
        assert!(modified.is_modified());
        assert!(!modified.is_stop());
        assert_eq!(modified.as_modified(), Some(&[1, 2, 3][..]));
    }

    #[test]
    fn test_event_result_into_modified() {
        let result = EventResult::Modified(vec![1, 2, 3]);
        assert_eq!(result.into_modified(), Some(vec![1, 2, 3]));

        let result = EventResult::Continue;
        assert_eq!(result.into_modified(), None);
    }

    #[test]
    fn test_handler_entry_creation() {
        let handler: EventHandler = Box::new(|_| EventResult::Continue);
        let entry = HandlerEntry::new(1, "test-plugin", 100, handler);

        assert_eq!(entry.id, 1);
        assert_eq!(entry.plugin_name, "test-plugin");
        assert_eq!(entry.priority, 100);
    }

    #[test]
    fn test_handler_entry_call() {
        let handler: EventHandler = Box::new(|args| {
            if args.event_type == EventType::Bell {
                EventResult::Stop
            } else {
                EventResult::Continue
            }
        });

        let entry = HandlerEntry::new(1, "test", 0, handler);

        let bell_args = EventArgs::new(EventType::Bell);
        assert_eq!(entry.call(&bell_args), EventResult::Stop);

        let other_args = EventArgs::new(EventType::TabCreated);
        assert_eq!(entry.call(&other_args), EventResult::Continue);
    }

    #[test]
    fn test_handler_entry_debug() {
        let handler: EventHandler = Box::new(|_| EventResult::Continue);
        let entry = HandlerEntry::new(42, "debug-test", 50, handler);
        let debug_str = format!("{:?}", entry);

        assert!(debug_str.contains("42"));
        assert!(debug_str.contains("debug-test"));
        assert!(debug_str.contains("50"));
        assert!(debug_str.contains("<function>"));
    }
}
