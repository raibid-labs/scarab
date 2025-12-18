//! Modal system for Scarab terminal emulator
//!
//! Provides vim-like modal interaction modes inspired by wezterm.
//! Each mode has distinct keybindings and UI behavior.

use bevy::input::keyboard::KeyCode;
use bevy::prelude::*;
use std::time::{Duration, Instant};

/// Plugin for modal system functionality
pub struct ModesPlugin;

impl Plugin for ModesPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ModeState>()
            .add_event::<ModeChangeEvent>()
            .add_event::<ModeActionEvent>()
            .add_systems(
                Update,
                (
                    handle_mode_switching_system,
                    handle_mode_timeout_system,
                    handle_mode_exit_system,
                )
                    .chain(),
            );
    }
}

/// Available modes in Scarab
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum ScarabMode {
    /// Normal terminal interaction mode
    Normal,
    /// Vim-like text selection and copy mode
    Copy,
    /// Pattern search in terminal buffer
    Search,
    /// Pane management (split, resize, navigate)
    Window,
    /// Font size adjustment mode
    Font,
    /// Picker interface (colorscheme, font, etc.)
    Pick,
    /// Link/nav hint mode (already exists, integrated here)
    Hint,
}

impl ScarabMode {
    /// Get human-readable name
    pub fn name(&self) -> &str {
        match self {
            Self::Normal => "NORMAL",
            Self::Copy => "COPY",
            Self::Search => "SEARCH",
            Self::Window => "WINDOW",
            Self::Font => "FONT",
            Self::Pick => "PICK",
            Self::Hint => "HINT",
        }
    }

    /// Get icon/emoji for this mode
    pub fn icon(&self) -> &str {
        match self {
            Self::Normal => "➤",
            Self::Copy => "📋",
            Self::Search => "🔍",
            Self::Window => "🪟",
            Self::Font => "🔤",
            Self::Pick => "📌",
            Self::Hint => "🎯",
        }
    }

    /// Get status bar color for this mode
    pub fn color(&self) -> Color {
        match self {
            Self::Normal => Color::srgb(0.66, 0.87, 0.35), // Slime green
            Self::Copy => Color::srgb(0.4, 0.7, 1.0),      // Blue
            Self::Search => Color::srgb(1.0, 0.8, 0.0),    // Yellow
            Self::Window => Color::srgb(0.9, 0.4, 0.6),    // Pink
            Self::Font => Color::srgb(0.7, 0.5, 1.0),      // Purple
            Self::Pick => Color::srgb(0.0, 0.9, 0.7),      // Cyan
            Self::Hint => Color::srgb(1.0, 0.5, 0.0),      // Orange
        }
    }

    /// Get help text for status bar
    pub fn help_text(&self) -> &str {
        match self {
            Self::Normal => "",
            Self::Copy => "h/j/k/l: move | v: select | y: copy | Esc: exit",
            Self::Search => "type to search | n: next | N: prev | Esc: exit",
            Self::Window => "s: split | v: vsplit | hjkl: navigate | Esc: exit",
            Self::Font => "+/-: adjust size | 0: reset | Esc: exit",
            Self::Pick => "↑↓: navigate | Enter: select | Esc: exit",
            Self::Hint => "type hint | Enter: activate | Esc: exit",
        }
    }

    /// Whether this mode has a timeout
    pub fn has_timeout(&self) -> bool {
        matches!(self, Self::Font)
    }

    /// Get timeout duration for this mode
    pub fn timeout_duration(&self) -> Duration {
        match self {
            Self::Font => Duration::from_secs(3),
            _ => Duration::from_secs(30), // Fallback
        }
    }
}

/// Resource tracking current mode state
#[derive(Resource)]
pub struct ModeState {
    /// Current active mode
    pub current: ScarabMode,
    /// Previous mode (for mode stack)
    pub previous: Option<ScarabMode>,
    /// When the current mode was entered
    pub entered_at: Option<Instant>,
    /// Whether mode indicator is visible in status bar
    pub show_indicator: bool,
}

impl Default for ModeState {
    fn default() -> Self {
        Self {
            current: ScarabMode::Normal,
            previous: None,
            entered_at: None,
            show_indicator: true,
        }
    }
}

impl ModeState {
    /// Switch to a new mode
    pub fn enter_mode(&mut self, mode: ScarabMode) {
        if self.current != mode {
            self.previous = Some(self.current);
            self.current = mode;
            self.entered_at = Some(Instant::now());
            info!("Entered mode: {:?}", mode);
        }
    }

    /// Return to Normal mode
    pub fn exit_to_normal(&mut self) {
        if self.current != ScarabMode::Normal {
            self.previous = Some(self.current);
            self.current = ScarabMode::Normal;
            self.entered_at = None;
            info!("Returned to Normal mode");
        }
    }

    /// Return to previous mode
    pub fn return_to_previous(&mut self) {
        if let Some(prev) = self.previous {
            self.current = prev;
            self.previous = None;
            self.entered_at = Some(Instant::now());
            info!("Returned to previous mode: {:?}", prev);
        }
    }

    /// Check if mode has timed out
    pub fn is_timed_out(&self) -> bool {
        if !self.current.has_timeout() {
            return false;
        }

        if let Some(entered) = self.entered_at {
            entered.elapsed() >= self.current.timeout_duration()
        } else {
            false
        }
    }
}

/// Event fired when mode changes
#[derive(Event)]
pub struct ModeChangeEvent {
    pub from: ScarabMode,
    pub to: ScarabMode,
}

/// Event fired for mode-specific actions
#[derive(Event)]
pub struct ModeActionEvent {
    pub mode: ScarabMode,
    pub action: String,
}

/// System to handle mode switching via keyboard
fn handle_mode_switching_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut mode_state: ResMut<ModeState>,
    mut event_writer: EventWriter<ModeChangeEvent>,
) {
    // Only handle mode switching from Normal mode or via Escape
    let previous_mode = mode_state.current;

    // Global escape to Normal
    if keyboard.just_pressed(KeyCode::Escape) {
        if mode_state.current != ScarabMode::Normal {
            mode_state.exit_to_normal();
            event_writer.send(ModeChangeEvent {
                from: previous_mode,
                to: ScarabMode::Normal,
            });
        }
        return;
    }

    // Mode switching only from Normal mode
    if mode_state.current != ScarabMode::Normal {
        return;
    }

    // Check for mode switching keys
    // Ctrl+Shift+C -> Copy mode
    if keyboard.just_pressed(KeyCode::KeyC)
        && keyboard.any_pressed([KeyCode::ControlLeft, KeyCode::ControlRight])
        && keyboard.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight])
    {
        mode_state.enter_mode(ScarabMode::Copy);
        event_writer.send(ModeChangeEvent {
            from: previous_mode,
            to: ScarabMode::Copy,
        });
    }

    // Ctrl+Shift+F -> Search mode
    if keyboard.just_pressed(KeyCode::KeyF)
        && keyboard.any_pressed([KeyCode::ControlLeft, KeyCode::ControlRight])
        && keyboard.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight])
    {
        mode_state.enter_mode(ScarabMode::Search);
        event_writer.send(ModeChangeEvent {
            from: previous_mode,
            to: ScarabMode::Search,
        });
    }

    // Ctrl+Shift+W -> Window mode
    if keyboard.just_pressed(KeyCode::KeyW)
        && keyboard.any_pressed([KeyCode::ControlLeft, KeyCode::ControlRight])
        && keyboard.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight])
    {
        mode_state.enter_mode(ScarabMode::Window);
        event_writer.send(ModeChangeEvent {
            from: previous_mode,
            to: ScarabMode::Window,
        });
    }

    // Ctrl+Shift+P -> Pick mode
    if keyboard.just_pressed(KeyCode::KeyP)
        && keyboard.any_pressed([KeyCode::ControlLeft, KeyCode::ControlRight])
        && keyboard.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight])
    {
        mode_state.enter_mode(ScarabMode::Pick);
        event_writer.send(ModeChangeEvent {
            from: previous_mode,
            to: ScarabMode::Pick,
        });
    }
}

/// System to handle mode timeouts
fn handle_mode_timeout_system(
    mut mode_state: ResMut<ModeState>,
    mut event_writer: EventWriter<ModeChangeEvent>,
) {
    if mode_state.is_timed_out() {
        let previous_mode = mode_state.current;
        mode_state.exit_to_normal();
        event_writer.send(ModeChangeEvent {
            from: previous_mode,
            to: ScarabMode::Normal,
        });
        info!("Mode {:?} timed out", previous_mode);
    }
}

/// System to handle explicit mode exit requests
fn handle_mode_exit_system(
    mut events: EventReader<ModeActionEvent>,
    mut mode_state: ResMut<ModeState>,
    mut mode_change_writer: EventWriter<ModeChangeEvent>,
) {
    for event in events.read() {
        if event.action == "exit" {
            let previous_mode = mode_state.current;
            mode_state.exit_to_normal();
            mode_change_writer.send(ModeChangeEvent {
                from: previous_mode,
                to: ScarabMode::Normal,
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mode_transitions() {
        let mut state = ModeState::default();
        assert_eq!(state.current, ScarabMode::Normal);

        state.enter_mode(ScarabMode::Copy);
        assert_eq!(state.current, ScarabMode::Copy);
        assert_eq!(state.previous, Some(ScarabMode::Normal));

        state.exit_to_normal();
        assert_eq!(state.current, ScarabMode::Normal);
    }

    #[test]
    fn test_mode_properties() {
        assert_eq!(ScarabMode::Copy.name(), "COPY");
        assert_eq!(ScarabMode::Normal.icon(), "➤");
        assert!(ScarabMode::Font.has_timeout());
        assert!(!ScarabMode::Normal.has_timeout());
    }

    #[test]
    fn test_mode_timeout() {
        let mut state = ModeState::default();
        state.enter_mode(ScarabMode::Font);

        // Should not be timed out immediately
        assert!(!state.is_timed_out());
    }

    #[test]
    fn test_return_to_previous() {
        let mut state = ModeState::default();
        state.enter_mode(ScarabMode::Copy);
        state.enter_mode(ScarabMode::Search);

        state.return_to_previous();
        assert_eq!(state.current, ScarabMode::Copy);
    }
}
