//! Plugin action and response events for ECS integration
//!
//! This module defines the event types used for plugin-to-ECS communication.
//! Plugins queue PluginActions which are processed by ECS systems, and receive
//! PluginResponses back when operations complete.

use bevy::prelude::*;

/// Actions that plugins can request from the ECS layer
#[derive(Event, Debug, Clone)]
pub enum PluginAction {
    /// Spawn a UI overlay at specified position
    SpawnOverlay {
        plugin_id: String,
        x: u16,
        y: u16,
        width: u16,
        height: u16,
        content: String,
        z_index: f32,
    },

    /// Remove a previously spawned overlay
    DespawnOverlay {
        plugin_id: String,
        overlay_id: u64,
    },

    /// Show a notification to the user
    ShowNotification {
        plugin_id: String,
        title: String,
        message: String,
        level: NotificationLevel,
        duration_ms: u64,
    },

    /// Add an item to the status bar
    AddStatusItem {
        plugin_id: String,
        side: StatusSide,
        content: String,
        priority: i32,
    },

    /// Remove a status bar item
    RemoveStatusItem {
        plugin_id: String,
        item_id: u64,
    },

    /// Register a keybinding for the plugin
    RegisterKeybinding {
        plugin_id: String,
        key: String,
        modifiers: Vec<String>,
        action_id: String,
    },

    /// Send input data to the terminal
    SendInput {
        plugin_id: String,
        data: Vec<u8>,
    },

    /// Request terminal content from specified rows
    RequestTerminalContent {
        plugin_id: String,
        start_row: u16,
        end_row: u16,
    },

    /// Update theme colors
    UpdateTheme {
        plugin_id: String,
        theme_json: String,
    },

    /// Show a modal dialog
    ShowModal {
        plugin_id: String,
        title: String,
        items: Vec<ModalItem>,
    },

    /// Navigation: Enter hint mode
    NavEnterHintMode {
        plugin_id: String,
    },

    /// Navigation: Exit navigation mode
    NavExitMode {
        plugin_id: String,
    },

    /// Navigation: Register a custom focusable region
    NavRegisterFocusable {
        plugin_id: String,
        x: u16,
        y: u16,
        width: u16,
        height: u16,
        label: String,
        action: NavFocusableAction,
    },

    /// Navigation: Unregister a focusable region
    NavUnregisterFocusable {
        plugin_id: String,
        focusable_id: u64,
    },
}

/// Responses from ECS systems back to plugins
#[derive(Event, Debug, Clone)]
pub enum PluginResponse {
    /// Overlay was successfully spawned
    OverlaySpawned {
        plugin_id: String,
        overlay_id: u64,
    },

    /// Terminal content requested by plugin
    TerminalContent {
        plugin_id: String,
        rows: Vec<TerminalRow>,
    },

    /// Keybinding was triggered
    KeybindingTriggered {
        plugin_id: String,
        action_id: String,
    },

    /// Error occurred while processing action
    Error {
        plugin_id: String,
        action: String,
        message: String,
    },

    /// Navigation: Focusable was registered
    NavFocusableRegistered {
        plugin_id: String,
        focusable_id: u64,
    },

    /// Navigation: Focusable was unregistered
    NavFocusableUnregistered {
        plugin_id: String,
        focusable_id: u64,
    },

    /// Navigation: Mode was entered
    NavModeEntered {
        plugin_id: String,
    },

    /// Navigation: Mode was exited
    NavModeExited {
        plugin_id: String,
    },
}

/// Navigation focusable action
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NavFocusableAction {
    /// Open a URL in the default browser
    OpenUrl(String),
    /// Open a file in the configured editor
    OpenFile(String),
    /// Custom plugin-defined action
    Custom(String),
}

/// Notification severity level
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NotificationLevel {
    Info,
    Success,
    Warning,
    Error,
}

/// Status bar side
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatusSide {
    Left,
    Right,
}

/// Modal item for dialogs
#[derive(Debug, Clone)]
pub struct ModalItem {
    pub label: String,
    pub value: String,
    pub description: Option<String>,
}

/// Terminal row data
#[derive(Debug, Clone)]
pub struct TerminalRow {
    pub row_index: u16,
    pub text: String,
    pub cells: Vec<TerminalCell>,
}

/// Terminal cell with styling
#[derive(Debug, Clone)]
pub struct TerminalCell {
    pub char: char,
    pub fg_color: (u8, u8, u8),
    pub bg_color: (u8, u8, u8),
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
}

impl Default for TerminalCell {
    fn default() -> Self {
        Self {
            char: ' ',
            fg_color: (255, 255, 255),
            bg_color: (0, 0, 0),
            bold: false,
            italic: false,
            underline: false,
        }
    }
}
