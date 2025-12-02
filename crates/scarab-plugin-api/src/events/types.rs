//! Event type definitions
//!
//! Defines all available event types in the Scarab event system, matching
//! WezTerm's event capabilities with 20+ granular events.

use serde::{Deserialize, Serialize};

/// Event type enumeration
///
/// Defines all events that plugins can subscribe to. Events are organized into
/// categories: window, tab, pane, terminal, and status events.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EventType {
    // ==================== Window Events ====================
    /// Fired when a new window is created
    WindowCreated,

    /// Fired when a window is closed
    WindowClosed,

    /// Fired when window focus changes (gained or lost)
    WindowFocusChanged,

    /// Fired when a window is resized
    WindowResized,

    /// Fired when configuration is reloaded
    WindowConfigReloaded,

    /// Fired once when the GUI starts up (before first window)
    GuiStartup,

    // ==================== Tab Events ====================
    /// Fired when a new tab is created
    TabCreated,

    /// Fired when a tab is closed
    TabClosed,

    /// Fired when the active tab changes
    TabSwitched,

    /// Fired when the user clicks the "new tab" button
    NewTabButtonClick,

    // ==================== Pane Events ====================
    /// Fired when a new pane is created (split)
    PaneCreated,

    /// Fired when a pane is closed
    PaneClosed,

    /// Fired when a pane gains focus
    PaneFocused,

    /// Fired when a pane's title changes
    PaneTitleChanged,

    // ==================== Terminal Events ====================
    /// Fired when the terminal bell is rung
    Bell,

    /// Fired when the text selection changes
    SelectionChanged,

    /// Fired when a user-defined variable changes (OSC 1337)
    UserVarChanged,

    /// Fired when a clickable URI is detected
    OpenUri,

    /// Fired when the scrollback buffer is cleared
    ScrollbackCleared,

    // ==================== Status Events ====================
    /// Request to update the status bar
    UpdateStatus,

    /// Request to update the right status section
    UpdateRightStatus,

    /// Request to update the left status section
    UpdateLeftStatus,

    /// Request to format a tab title (return formatted string)
    FormatTabTitle,

    /// Request to format the window title (return formatted string)
    FormatWindowTitle,

    // ==================== Legacy Hook Events ====================
    /// Terminal output (maps to HookType::PreOutput)
    Output,

    /// User input (maps to HookType::PostInput)
    Input,

    /// Before command execution (maps to HookType::PreCommand)
    PreCommand,

    /// After command execution (maps to HookType::PostCommand)
    PostCommand,

    /// Terminal resize (maps to HookType::OnResize)
    Resize,

    /// Client attached (maps to HookType::OnAttach)
    Attach,

    /// Client detached (maps to HookType::OnDetach)
    Detach,

    // ==================== Custom Events ====================
    /// User-defined custom event
    ///
    /// Allows plugins to emit and listen for custom events with arbitrary names.
    /// For example, `Custom("git-status-changed")` or `Custom("test-completed")`.
    Custom(String),
}

impl EventType {
    /// Check if this is a window-related event
    pub fn is_window_event(&self) -> bool {
        matches!(
            self,
            EventType::WindowCreated
                | EventType::WindowClosed
                | EventType::WindowFocusChanged
                | EventType::WindowResized
                | EventType::WindowConfigReloaded
                | EventType::GuiStartup
        )
    }

    /// Check if this is a tab-related event
    pub fn is_tab_event(&self) -> bool {
        matches!(
            self,
            EventType::TabCreated
                | EventType::TabClosed
                | EventType::TabSwitched
                | EventType::NewTabButtonClick
        )
    }

    /// Check if this is a pane-related event
    pub fn is_pane_event(&self) -> bool {
        matches!(
            self,
            EventType::PaneCreated
                | EventType::PaneClosed
                | EventType::PaneFocused
                | EventType::PaneTitleChanged
        )
    }

    /// Check if this is a terminal interaction event
    pub fn is_terminal_event(&self) -> bool {
        matches!(
            self,
            EventType::Bell
                | EventType::SelectionChanged
                | EventType::UserVarChanged
                | EventType::OpenUri
                | EventType::ScrollbackCleared
        )
    }

    /// Check if this is a status/formatting event
    pub fn is_status_event(&self) -> bool {
        matches!(
            self,
            EventType::UpdateStatus
                | EventType::UpdateRightStatus
                | EventType::UpdateLeftStatus
                | EventType::FormatTabTitle
                | EventType::FormatWindowTitle
        )
    }

    /// Check if this is a legacy hook event
    pub fn is_legacy_event(&self) -> bool {
        matches!(
            self,
            EventType::Output
                | EventType::Input
                | EventType::PreCommand
                | EventType::PostCommand
                | EventType::Resize
                | EventType::Attach
                | EventType::Detach
        )
    }

    /// Check if this is a custom event
    pub fn is_custom_event(&self) -> bool {
        matches!(self, EventType::Custom(_))
    }

    /// Get a human-readable name for this event
    pub fn name(&self) -> String {
        match self {
            EventType::WindowCreated => "window-created".to_string(),
            EventType::WindowClosed => "window-closed".to_string(),
            EventType::WindowFocusChanged => "window-focus-changed".to_string(),
            EventType::WindowResized => "window-resized".to_string(),
            EventType::WindowConfigReloaded => "window-config-reloaded".to_string(),
            EventType::GuiStartup => "gui-startup".to_string(),
            EventType::TabCreated => "tab-created".to_string(),
            EventType::TabClosed => "tab-closed".to_string(),
            EventType::TabSwitched => "tab-switched".to_string(),
            EventType::NewTabButtonClick => "new-tab-button-click".to_string(),
            EventType::PaneCreated => "pane-created".to_string(),
            EventType::PaneClosed => "pane-closed".to_string(),
            EventType::PaneFocused => "pane-focused".to_string(),
            EventType::PaneTitleChanged => "pane-title-changed".to_string(),
            EventType::Bell => "bell".to_string(),
            EventType::SelectionChanged => "selection-changed".to_string(),
            EventType::UserVarChanged => "user-var-changed".to_string(),
            EventType::OpenUri => "open-uri".to_string(),
            EventType::ScrollbackCleared => "scrollback-cleared".to_string(),
            EventType::UpdateStatus => "update-status".to_string(),
            EventType::UpdateRightStatus => "update-right-status".to_string(),
            EventType::UpdateLeftStatus => "update-left-status".to_string(),
            EventType::FormatTabTitle => "format-tab-title".to_string(),
            EventType::FormatWindowTitle => "format-window-title".to_string(),
            EventType::Output => "output".to_string(),
            EventType::Input => "input".to_string(),
            EventType::PreCommand => "pre-command".to_string(),
            EventType::PostCommand => "post-command".to_string(),
            EventType::Resize => "resize".to_string(),
            EventType::Attach => "attach".to_string(),
            EventType::Detach => "detach".to_string(),
            EventType::Custom(name) => format!("custom:{}", name),
        }
    }

    /// Get all standard (non-custom) event types
    pub fn all_standard() -> Vec<EventType> {
        vec![
            // Window events
            EventType::WindowCreated,
            EventType::WindowClosed,
            EventType::WindowFocusChanged,
            EventType::WindowResized,
            EventType::WindowConfigReloaded,
            EventType::GuiStartup,
            // Tab events
            EventType::TabCreated,
            EventType::TabClosed,
            EventType::TabSwitched,
            EventType::NewTabButtonClick,
            // Pane events
            EventType::PaneCreated,
            EventType::PaneClosed,
            EventType::PaneFocused,
            EventType::PaneTitleChanged,
            // Terminal events
            EventType::Bell,
            EventType::SelectionChanged,
            EventType::UserVarChanged,
            EventType::OpenUri,
            EventType::ScrollbackCleared,
            // Status events
            EventType::UpdateStatus,
            EventType::UpdateRightStatus,
            EventType::UpdateLeftStatus,
            EventType::FormatTabTitle,
            EventType::FormatWindowTitle,
            // Legacy events
            EventType::Output,
            EventType::Input,
            EventType::PreCommand,
            EventType::PostCommand,
            EventType::Resize,
            EventType::Attach,
            EventType::Detach,
        ]
    }

    /// Convert from legacy HookType to EventType
    pub fn from_hook_type(hook_type: crate::types::HookType) -> Self {
        match hook_type {
            crate::types::HookType::PreOutput => EventType::Output,
            crate::types::HookType::PostInput => EventType::Input,
            crate::types::HookType::PreCommand => EventType::PreCommand,
            crate::types::HookType::PostCommand => EventType::PostCommand,
            crate::types::HookType::OnResize => EventType::Resize,
            crate::types::HookType::OnAttach => EventType::Attach,
            crate::types::HookType::OnDetach => EventType::Detach,
        }
    }

    /// Try to convert EventType to legacy HookType (returns None for new events)
    pub fn to_hook_type(&self) -> Option<crate::types::HookType> {
        match self {
            EventType::Output => Some(crate::types::HookType::PreOutput),
            EventType::Input => Some(crate::types::HookType::PostInput),
            EventType::PreCommand => Some(crate::types::HookType::PreCommand),
            EventType::PostCommand => Some(crate::types::HookType::PostCommand),
            EventType::Resize => Some(crate::types::HookType::OnResize),
            EventType::Attach => Some(crate::types::HookType::OnAttach),
            EventType::Detach => Some(crate::types::HookType::OnDetach),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_categorization() {
        assert!(EventType::WindowCreated.is_window_event());
        assert!(!EventType::WindowCreated.is_tab_event());

        assert!(EventType::TabSwitched.is_tab_event());
        assert!(!EventType::TabSwitched.is_pane_event());

        assert!(EventType::PaneFocused.is_pane_event());
        assert!(!EventType::PaneFocused.is_terminal_event());

        assert!(EventType::Bell.is_terminal_event());
        assert!(!EventType::Bell.is_status_event());

        assert!(EventType::UpdateStatus.is_status_event());
        assert!(!EventType::UpdateStatus.is_legacy_event());

        assert!(EventType::Output.is_legacy_event());
    }

    #[test]
    fn test_custom_event() {
        let custom = EventType::Custom("my-event".to_string());
        assert!(custom.is_custom_event());
        assert_eq!(custom.name(), "custom:my-event");
    }

    #[test]
    fn test_event_names() {
        assert_eq!(EventType::WindowCreated.name(), "window-created");
        assert_eq!(EventType::Bell.name(), "bell");
        assert_eq!(EventType::FormatTabTitle.name(), "format-tab-title");
    }

    #[test]
    fn test_hook_type_conversion() {
        use crate::types::HookType;

        assert_eq!(
            EventType::from_hook_type(HookType::PreOutput),
            EventType::Output
        );
        assert_eq!(
            EventType::from_hook_type(HookType::OnResize),
            EventType::Resize
        );

        assert_eq!(
            EventType::Output.to_hook_type(),
            Some(HookType::PreOutput)
        );
        assert_eq!(EventType::Bell.to_hook_type(), None);
    }

    #[test]
    fn test_all_standard_events() {
        let events = EventType::all_standard();
        assert_eq!(events.len(), 31); // 6 window + 4 tab + 4 pane + 5 terminal + 5 status + 7 legacy + 2 custom

        // Verify no duplicates
        use std::collections::HashSet;
        let unique: HashSet<_> = events.iter().collect();
        assert_eq!(unique.len(), events.len());
    }
}
