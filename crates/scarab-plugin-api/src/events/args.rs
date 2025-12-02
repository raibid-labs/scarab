//! Event argument types
//!
//! Defines the data structures passed to event handlers, including context
//! about the terminal objects involved and event-specific data.

use super::EventType;
use crate::object_model::ObjectHandle;
use std::time::Instant;

/// Event arguments passed to handlers
///
/// Contains all contextual information about an event, including:
/// - The event type
/// - Handles to affected terminal objects (window, pane, tab)
/// - Event-specific data payload
/// - Timestamp when the event occurred
#[derive(Clone, Debug)]
pub struct EventArgs {
    /// Type of event being fired
    pub event_type: EventType,

    /// Handle to the window associated with this event (if any)
    pub window: Option<ObjectHandle>,

    /// Handle to the pane associated with this event (if any)
    pub pane: Option<ObjectHandle>,

    /// Handle to the tab associated with this event (if any)
    pub tab: Option<ObjectHandle>,

    /// Event-specific data payload
    pub data: EventData,

    /// When this event was created
    pub timestamp: Instant,
}

impl EventArgs {
    /// Create new event arguments
    pub fn new(event_type: EventType) -> Self {
        Self {
            event_type,
            window: None,
            pane: None,
            tab: None,
            data: EventData::None,
            timestamp: Instant::now(),
        }
    }

    /// Set the window handle (builder pattern)
    pub fn with_window(mut self, handle: ObjectHandle) -> Self {
        self.window = Some(handle);
        self
    }

    /// Set the pane handle (builder pattern)
    pub fn with_pane(mut self, handle: ObjectHandle) -> Self {
        self.pane = Some(handle);
        self
    }

    /// Set the tab handle (builder pattern)
    pub fn with_tab(mut self, handle: ObjectHandle) -> Self {
        self.tab = Some(handle);
        self
    }

    /// Set the event data (builder pattern)
    pub fn with_data(mut self, data: EventData) -> Self {
        self.data = data;
        self
    }

    /// Get a reference to the event data
    pub fn data(&self) -> &EventData {
        &self.data
    }

    /// Check if this event has window context
    pub fn has_window(&self) -> bool {
        self.window.is_some()
    }

    /// Check if this event has pane context
    pub fn has_pane(&self) -> bool {
        self.pane.is_some()
    }

    /// Check if this event has tab context
    pub fn has_tab(&self) -> bool {
        self.tab.is_some()
    }

    /// Get the age of this event
    pub fn age(&self) -> std::time::Duration {
        self.timestamp.elapsed()
    }
}

/// Event-specific data payload
///
/// Different events carry different types of data. This enum provides
/// type-safe access to event payloads.
#[derive(Clone, Debug, Default)]
pub enum EventData {
    /// No data
    #[default]
    None,

    /// Plain text data (for output, input, etc.)
    Text(String),

    /// URI/URL data (for OpenUri events)
    Uri(String),

    /// Focus state change
    FocusState {
        /// Whether the object now has focus
        is_focused: bool,
    },

    /// Dimension change (for resize events)
    Dimensions {
        /// New width in columns
        cols: u16,
        /// New height in rows
        rows: u16,
    },

    /// Title change event
    TitleChange {
        /// Previous title
        old: String,
        /// New title
        new: String,
    },

    /// User variable change (OSC 1337)
    UserVar {
        /// Variable name
        name: String,
        /// Variable value
        value: String,
    },

    /// Text selection data
    Selection {
        /// Selected text content
        text: String,
        /// Selection start position (col, row)
        start: (u16, u16),
        /// Selection end position (col, row)
        end: (u16, u16),
    },

    /// Process exit code
    ExitCode(i32),

    /// Raw binary data (for legacy hooks)
    Binary(Vec<u8>),
}

impl EventData {
    /// Try to extract text data
    pub fn as_text(&self) -> Option<&str> {
        match self {
            EventData::Text(s) => Some(s),
            _ => None,
        }
    }

    /// Try to extract URI data
    pub fn as_uri(&self) -> Option<&str> {
        match self {
            EventData::Uri(s) => Some(s),
            _ => None,
        }
    }

    /// Try to extract focus state
    pub fn as_focus_state(&self) -> Option<bool> {
        match self {
            EventData::FocusState { is_focused } => Some(*is_focused),
            _ => None,
        }
    }

    /// Try to extract dimensions
    pub fn as_dimensions(&self) -> Option<(u16, u16)> {
        match self {
            EventData::Dimensions { cols, rows } => Some((*cols, *rows)),
            _ => None,
        }
    }

    /// Try to extract title change data
    pub fn as_title_change(&self) -> Option<(&str, &str)> {
        match self {
            EventData::TitleChange { old, new } => Some((old, new)),
            _ => None,
        }
    }

    /// Try to extract user variable data
    pub fn as_user_var(&self) -> Option<(&str, &str)> {
        match self {
            EventData::UserVar { name, value } => Some((name, value)),
            _ => None,
        }
    }

    /// Try to extract selection data
    pub fn as_selection(&self) -> Option<(&str, (u16, u16), (u16, u16))> {
        match self {
            EventData::Selection { text, start, end } => Some((text, *start, *end)),
            _ => None,
        }
    }

    /// Try to extract exit code
    pub fn as_exit_code(&self) -> Option<i32> {
        match self {
            EventData::ExitCode(code) => Some(*code),
            _ => None,
        }
    }

    /// Try to extract binary data
    pub fn as_binary(&self) -> Option<&[u8]> {
        match self {
            EventData::Binary(data) => Some(data),
            _ => None,
        }
    }

    /// Check if this is empty/none data
    pub fn is_none(&self) -> bool {
        matches!(self, EventData::None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::object_model::ObjectType;

    #[test]
    fn test_event_args_builder() {
        let window = ObjectHandle::new(ObjectType::Window, 1, 0);
        let pane = ObjectHandle::new(ObjectType::Pane, 2, 0);

        let args = EventArgs::new(EventType::PaneFocused)
            .with_window(window)
            .with_pane(pane)
            .with_data(EventData::FocusState { is_focused: true });

        assert_eq!(args.event_type, EventType::PaneFocused);
        assert_eq!(args.window, Some(window));
        assert_eq!(args.pane, Some(pane));
        assert!(args.has_window());
        assert!(args.has_pane());
        assert!(!args.has_tab());
        assert!(args.data.as_focus_state().unwrap());
    }

    #[test]
    fn test_event_data_text() {
        let data = EventData::Text("hello".to_string());
        assert_eq!(data.as_text(), Some("hello"));
        assert!(data.as_uri().is_none());
        assert!(!data.is_none());
    }

    #[test]
    fn test_event_data_dimensions() {
        let data = EventData::Dimensions { cols: 80, rows: 24 };
        assert_eq!(data.as_dimensions(), Some((80, 24)));
        assert!(data.as_text().is_none());
    }

    #[test]
    fn test_event_data_selection() {
        let data = EventData::Selection {
            text: "selected".to_string(),
            start: (0, 0),
            end: (8, 0),
        };

        let (text, start, end) = data.as_selection().unwrap();
        assert_eq!(text, "selected");
        assert_eq!(start, (0, 0));
        assert_eq!(end, (8, 0));
    }

    #[test]
    fn test_event_data_user_var() {
        let data = EventData::UserVar {
            name: "CWD".to_string(),
            value: "/home/user".to_string(),
        };

        let (name, value) = data.as_user_var().unwrap();
        assert_eq!(name, "CWD");
        assert_eq!(value, "/home/user");
    }

    #[test]
    fn test_event_data_title_change() {
        let data = EventData::TitleChange {
            old: "old title".to_string(),
            new: "new title".to_string(),
        };

        let (old, new) = data.as_title_change().unwrap();
        assert_eq!(old, "old title");
        assert_eq!(new, "new title");
    }

    #[test]
    fn test_event_data_exit_code() {
        let data = EventData::ExitCode(42);
        assert_eq!(data.as_exit_code(), Some(42));
    }

    #[test]
    fn test_event_data_binary() {
        let bytes = vec![1, 2, 3, 4];
        let data = EventData::Binary(bytes.clone());
        assert_eq!(data.as_binary(), Some(bytes.as_slice()));
    }

    #[test]
    fn test_event_args_age() {
        let args = EventArgs::new(EventType::Bell);
        std::thread::sleep(std::time::Duration::from_millis(10));
        assert!(args.age().as_millis() >= 10);
    }
}
