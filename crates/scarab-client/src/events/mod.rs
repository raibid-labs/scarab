//! Client-side event handling
//!
//! This module provides pure ECS event handling using Bevy events.
//! All events are typed and use Bevy's EventReader/EventWriter pattern
//! instead of Arc<Mutex<EventRegistry>>.

mod bevy_events;
mod plugin;
mod plugin_actions;

// Re-export all event types
pub use bevy_events::*;

// Re-export the plugin and receiver
pub use plugin::{DaemonEventReceiver, EventsPlugin};

// Re-export plugin actions (for UI integration)
pub use plugin_actions::{
    ModalItem, NavFocusableAction, NotificationLevel, PluginAction, PluginResponse, StatusSide,
    TerminalCell, TerminalRow,
};
