//! Bevy ECS Event Types
//!
//! This module defines typed Bevy events for all EventType variants,
//! replacing the Arc<Mutex<EventRegistry>> pattern with proper ECS events.

use bevy::prelude::*;
use scarab_plugin_api::object_model::ObjectHandle;

// ==================== Window Events ====================

/// Fired when a new window is created
#[derive(Event, Debug, Clone)]
pub struct WindowCreatedEvent {
    pub window: ObjectHandle,
}

/// Fired when a window is closed
#[derive(Event, Debug, Clone)]
pub struct WindowClosedEvent {
    pub window: ObjectHandle,
}

/// Fired when window focus changes (gained or lost)
#[derive(Event, Debug, Clone)]
pub struct WindowFocusChangedEvent {
    pub window: ObjectHandle,
    pub is_focused: bool,
}

/// Fired when a window is resized
#[derive(Event, Debug, Clone)]
pub struct WindowResizedEvent {
    pub window: ObjectHandle,
    pub cols: u16,
    pub rows: u16,
}

/// Fired when configuration is reloaded
#[derive(Event, Debug, Clone)]
pub struct WindowConfigReloadedEvent {
    pub window: ObjectHandle,
}

/// Fired once when the GUI starts up (before first window)
#[derive(Event, Debug, Clone)]
pub struct GuiStartupEvent;

// ==================== Tab Events ====================

/// Fired when a new tab is created
#[derive(Event, Debug, Clone)]
pub struct TabCreatedEvent {
    pub window: ObjectHandle,
    pub tab: ObjectHandle,
}

/// Fired when a tab is closed
#[derive(Event, Debug, Clone)]
pub struct TabClosedEvent {
    pub window: ObjectHandle,
    pub tab: ObjectHandle,
}

/// Fired when the active tab changes
#[derive(Event, Debug, Clone)]
pub struct TabSwitchedEvent {
    pub window: ObjectHandle,
    pub old_tab: Option<ObjectHandle>,
    pub new_tab: ObjectHandle,
}

/// Fired when the user clicks the "new tab" button
#[derive(Event, Debug, Clone)]
pub struct NewTabButtonClickEvent {
    pub window: ObjectHandle,
}

// ==================== Pane Events ====================

/// Fired when a new pane is created (split)
#[derive(Event, Debug, Clone)]
pub struct PaneCreatedEvent {
    pub window: ObjectHandle,
    pub tab: ObjectHandle,
    pub pane: ObjectHandle,
}

/// Fired when a pane is closed
#[derive(Event, Debug, Clone)]
pub struct PaneClosedEvent {
    pub window: ObjectHandle,
    pub tab: ObjectHandle,
    pub pane: ObjectHandle,
}

/// Fired when a pane gains focus
#[derive(Event, Debug, Clone)]
pub struct PaneFocusedEvent {
    pub window: ObjectHandle,
    pub tab: ObjectHandle,
    pub pane: ObjectHandle,
}

/// Fired when a pane's title changes
#[derive(Event, Debug, Clone)]
pub struct PaneTitleChangedEvent {
    pub pane: ObjectHandle,
    pub old_title: String,
    pub new_title: String,
}

// ==================== Terminal Events ====================

/// Fired when the terminal bell is rung
#[derive(Event, Debug, Clone)]
pub struct BellEvent {
    pub pane: ObjectHandle,
}

/// Fired when the text selection changes
#[derive(Event, Debug, Clone)]
pub struct SelectionChangedEvent {
    pub pane: ObjectHandle,
    pub text: String,
    pub start: (u16, u16),
    pub end: (u16, u16),
}

/// Fired when a user-defined variable changes (OSC 1337)
#[derive(Event, Debug, Clone)]
pub struct UserVarChangedEvent {
    pub pane: ObjectHandle,
    pub name: String,
    pub value: String,
}

/// Fired when a clickable URI is detected
#[derive(Event, Debug, Clone)]
pub struct OpenUriEvent {
    pub pane: ObjectHandle,
    pub uri: String,
}

/// Fired when the scrollback buffer is cleared
#[derive(Event, Debug, Clone)]
pub struct ScrollbackClearedEvent {
    pub pane: ObjectHandle,
}

// ==================== Status Events ====================

/// Request to update the status bar
#[derive(Event, Debug, Clone)]
pub struct UpdateStatusEvent {
    pub window: ObjectHandle,
}

/// Request to update the right status section
#[derive(Event, Debug, Clone)]
pub struct UpdateRightStatusEvent {
    pub window: ObjectHandle,
}

/// Request to update the left status section
#[derive(Event, Debug, Clone)]
pub struct UpdateLeftStatusEvent {
    pub window: ObjectHandle,
}

/// Request to format a tab title (return formatted string)
#[derive(Event, Debug, Clone)]
pub struct FormatTabTitleEvent {
    pub tab: ObjectHandle,
    pub default_title: String,
}

/// Request to format the window title (return formatted string)
#[derive(Event, Debug, Clone)]
pub struct FormatWindowTitleEvent {
    pub window: ObjectHandle,
    pub default_title: String,
}

// ==================== Legacy Hook Events ====================

/// Terminal output (maps to HookType::PreOutput)
#[derive(Event, Debug, Clone)]
pub struct OutputEvent {
    pub pane: ObjectHandle,
    pub data: Vec<u8>,
}

/// User input (maps to HookType::PostInput)
#[derive(Event, Debug, Clone)]
pub struct InputEvent {
    pub pane: ObjectHandle,
    pub data: Vec<u8>,
}

/// Before command execution (maps to HookType::PreCommand)
#[derive(Event, Debug, Clone)]
pub struct PreCommandEvent {
    pub pane: ObjectHandle,
    pub command: String,
}

/// After command execution (maps to HookType::PostCommand)
#[derive(Event, Debug, Clone)]
pub struct PostCommandEvent {
    pub pane: ObjectHandle,
    pub command: String,
    pub exit_code: i32,
}

/// Terminal resize (maps to HookType::OnResize)
#[derive(Event, Debug, Clone)]
pub struct ResizeEvent {
    pub pane: ObjectHandle,
    pub cols: u16,
    pub rows: u16,
}

/// Client attached (maps to HookType::OnAttach)
#[derive(Event, Debug, Clone)]
pub struct AttachEvent {
    pub window: ObjectHandle,
}

/// Client detached (maps to HookType::OnDetach)
#[derive(Event, Debug, Clone)]
pub struct DetachEvent {
    pub window: ObjectHandle,
}

// ==================== Custom Events ====================

/// User-defined custom event
///
/// Allows plugins to emit and listen for custom events with arbitrary names.
#[derive(Event, Debug, Clone)]
pub struct CustomEvent {
    pub name: String,
    pub data: Vec<u8>,
    pub window: Option<ObjectHandle>,
    pub pane: Option<ObjectHandle>,
    pub tab: Option<ObjectHandle>,
}

// ==================== Daemon Events ====================

/// Bevy event wrapper for daemon-forwarded events
///
/// This is a generic event type used to receive events from the daemon via IPC.
/// These are then converted to typed events above.
#[derive(Event, Debug, Clone)]
pub struct DaemonEvent {
    pub event_type: String,
    pub window_id: Option<u64>,
    pub pane_id: Option<u64>,
    pub tab_id: Option<u64>,
    pub data: Vec<u8>,
    pub timestamp_micros: u64,
}
