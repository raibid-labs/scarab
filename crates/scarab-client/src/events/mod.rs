//! Client-side event handling
//!
//! Handles events originating from the client (window focus, resize, etc.) and
//! processes events forwarded from the daemon.

mod plugin;

pub use plugin::EventsPlugin;

use bevy::prelude::*;

/// Bevy event for window focus changes
#[derive(Event, Debug, Clone)]
pub struct WindowFocusChangedEvent {
    pub is_focused: bool,
}

/// Bevy event for window resizes
#[derive(Event, Debug, Clone)]
pub struct WindowResizedEvent {
    pub cols: u16,
    pub rows: u16,
}

/// Bevy event wrapper for daemon-forwarded events
#[derive(Event, Debug, Clone)]
pub struct DaemonEvent {
    pub event_type: String,
    pub window_id: Option<u64>,
    pub pane_id: Option<u64>,
    pub tab_id: Option<u64>,
    pub data: Vec<u8>,
    pub timestamp_micros: u64,
}
