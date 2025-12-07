//! Unified Navigation Input Routing System
//!
//! This module provides centralized input routing for all navigation modes,
//! eliminating duplication between plugin_host and legacy EventsPlugin.
//!
//! Architecture:
//! - NavInputRouter: Stores keymap configurations for different navigation styles
//! - ModeStack: Stack-based navigation mode management (allows nested modes)
//! - route_nav_input system: Routes keyboard input to appropriate handlers
//! - NavAction events: Unified action events for navigation

use bevy::input::keyboard::KeyCode;
use bevy::prelude::*;
use std::collections::HashMap;

/// Navigation mode defining how input is interpreted
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NavMode {
    /// Normal terminal mode (input passed through to PTY)
    Normal,
    /// Vimium-style link hints mode (f for hints, letter keys for selection)
    Hints,
    /// Copy mode for text selection and navigation
    Copy,
    /// Search mode for finding text in terminal output
    Search,
    /// Command palette for executing commands
    CommandPalette,
    /// Prompt navigation mode (jumping between shell prompts)
    PromptNav,
}

/// Navigation style defining the overall keymap philosophy
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NavStyle {
    /// Vimium-style: f for hints, Esc to cancel, letter keys for hint selection
    VimiumStyle,
    /// Cosmos-style: space as leader key, then navigation submodes
    CosmosStyle,
    /// Spacemacs-style: SPC prefix for commands
    SpacemacsStyle,
}

/// Actions that can be triggered by navigation input
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Event)]
pub enum NavAction {
    // Mode transitions
    /// Enter hint mode for link selection
    EnterHintMode,
    /// Enter copy mode for text selection
    EnterCopyMode,
    /// Enter search mode
    EnterSearchMode,
    /// Enter command palette
    EnterCommandPalette,
    /// Exit current mode (return to parent or Normal)
    ExitCurrentMode,
    /// Cancel all modes and return to Normal
    CancelAllModes,

    // Hint mode actions
    /// Activate the currently selected hint
    ActivateHint,
    /// Input a character for hint selection
    HintChar(char),

    // Prompt navigation
    /// Jump to previous shell prompt
    JumpToPrevPrompt,
    /// Jump to next shell prompt
    JumpToNextPrompt,

    // Copy mode actions (delegated to CopyModeAction)
    /// Toggle visual selection in copy mode
    CopyModeToggleSelection,
    /// Exit copy mode and copy selection
    CopyModeExit,

    // Search actions
    /// Search forward for pattern
    SearchForward,
    /// Search backward for pattern
    SearchBackward,
    /// Move to next search match
    NextSearchMatch,
    /// Move to previous search match
    PrevSearchMatch,

    // Command palette actions
    /// Execute selected command
    ExecuteCommand,
    /// Filter command palette
    FilterCommands,
}

/// Key binding mapping a key combo to a navigation action
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct KeyBinding {
    /// The primary key
    pub key: KeyCode,
    /// Required modifiers (all must be pressed)
    pub modifiers: Vec<Modifier>,
    /// The action to trigger
    pub action: NavAction,
    /// Optional mode restriction (None = active in all modes)
    pub active_in_mode: Option<NavMode>,
}

/// Keyboard modifiers
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Modifier {
    Ctrl,
    Alt,
    Shift,
    Super,
}

impl KeyBinding {
    /// Create a new key binding
    pub fn new(key: KeyCode, action: NavAction) -> Self {
        Self {
            key,
            modifiers: Vec::new(),
            action,
            active_in_mode: None,
        }
    }

    /// Parse a key binding from a string like "Ctrl+F" or "Alt+Shift+Enter"
    pub fn from_string(s: &str, action: NavAction) -> Result<Self, String> {
        let parts: Vec<&str> = s.split('+').map(|p| p.trim()).collect();

        if parts.is_empty() {
            return Err("Empty key binding string".to_string());
        }

        let mut modifiers = Vec::new();
        let mut key_part = None;

        for part in parts {
            match part {
                "Ctrl" | "Control" => modifiers.push(Modifier::Ctrl),
                "Alt" => modifiers.push(Modifier::Alt),
                "Shift" => modifiers.push(Modifier::Shift),
                "Super" | "Meta" | "Win" | "Cmd" => modifiers.push(Modifier::Super),
                key => {
                    if key_part.is_some() {
                        return Err(format!(
                            "Multiple keys specified: {} and {}",
                            key_part.unwrap(),
                            key
                        ));
                    }
                    key_part = Some(key);
                }
            }
        }

        let key = match key_part {
            Some(k) => Self::parse_key(k)?,
            None => return Err("No key specified, only modifiers".to_string()),
        };

        Ok(Self {
            key,
            modifiers,
            action,
            active_in_mode: None,
        })
    }

    /// Parse a key string into a KeyCode
    fn parse_key(s: &str) -> Result<KeyCode, String> {
        match s {
            // Function keys
            "F1" => Ok(KeyCode::F1),
            "F2" => Ok(KeyCode::F2),
            "F3" => Ok(KeyCode::F3),
            "F4" => Ok(KeyCode::F4),
            "F5" => Ok(KeyCode::F5),
            "F6" => Ok(KeyCode::F6),
            "F7" => Ok(KeyCode::F7),
            "F8" => Ok(KeyCode::F8),
            "F9" => Ok(KeyCode::F9),
            "F10" => Ok(KeyCode::F10),
            "F11" => Ok(KeyCode::F11),
            "F12" => Ok(KeyCode::F12),
            // Special keys
            "Escape" | "Esc" => Ok(KeyCode::Escape),
            "Enter" | "Return" => Ok(KeyCode::Enter),
            "Space" => Ok(KeyCode::Space),
            "Tab" => Ok(KeyCode::Tab),
            "Backspace" => Ok(KeyCode::Backspace),
            "Delete" | "Del" => Ok(KeyCode::Delete),
            "Insert" | "Ins" => Ok(KeyCode::Insert),
            "Home" => Ok(KeyCode::Home),
            "End" => Ok(KeyCode::End),
            "PageUp" | "PgUp" => Ok(KeyCode::PageUp),
            "PageDown" | "PgDown" => Ok(KeyCode::PageDown),
            // Arrow keys
            "ArrowUp" | "Up" => Ok(KeyCode::ArrowUp),
            "ArrowDown" | "Down" => Ok(KeyCode::ArrowDown),
            "ArrowLeft" | "Left" => Ok(KeyCode::ArrowLeft),
            "ArrowRight" | "Right" => Ok(KeyCode::ArrowRight),
            // Letters
            "A" | "a" => Ok(KeyCode::KeyA),
            "B" | "b" => Ok(KeyCode::KeyB),
            "C" | "c" => Ok(KeyCode::KeyC),
            "D" | "d" => Ok(KeyCode::KeyD),
            "E" | "e" => Ok(KeyCode::KeyE),
            "F" | "f" => Ok(KeyCode::KeyF),
            "G" | "g" => Ok(KeyCode::KeyG),
            "H" | "h" => Ok(KeyCode::KeyH),
            "I" | "i" => Ok(KeyCode::KeyI),
            "J" | "j" => Ok(KeyCode::KeyJ),
            "K" | "k" => Ok(KeyCode::KeyK),
            "L" | "l" => Ok(KeyCode::KeyL),
            "M" | "m" => Ok(KeyCode::KeyM),
            "N" | "n" => Ok(KeyCode::KeyN),
            "O" | "o" => Ok(KeyCode::KeyO),
            "P" | "p" => Ok(KeyCode::KeyP),
            "Q" | "q" => Ok(KeyCode::KeyQ),
            "R" | "r" => Ok(KeyCode::KeyR),
            "S" | "s" => Ok(KeyCode::KeyS),
            "T" | "t" => Ok(KeyCode::KeyT),
            "U" | "u" => Ok(KeyCode::KeyU),
            "V" | "v" => Ok(KeyCode::KeyV),
            "W" | "w" => Ok(KeyCode::KeyW),
            "X" | "x" => Ok(KeyCode::KeyX),
            "Y" | "y" => Ok(KeyCode::KeyY),
            "Z" | "z" => Ok(KeyCode::KeyZ),
            // Numbers
            "0" => Ok(KeyCode::Digit0),
            "1" => Ok(KeyCode::Digit1),
            "2" => Ok(KeyCode::Digit2),
            "3" => Ok(KeyCode::Digit3),
            "4" => Ok(KeyCode::Digit4),
            "5" => Ok(KeyCode::Digit5),
            "6" => Ok(KeyCode::Digit6),
            "7" => Ok(KeyCode::Digit7),
            "8" => Ok(KeyCode::Digit8),
            "9" => Ok(KeyCode::Digit9),
            // Special characters
            "/" | "Slash" => Ok(KeyCode::Slash),
            "-" | "Minus" => Ok(KeyCode::Minus),
            "=" | "Equals" => Ok(KeyCode::Equal),
            _ => Err(format!("Unknown key: {}", s)),
        }
    }

    /// Add Ctrl modifier
    pub fn with_ctrl(mut self) -> Self {
        self.modifiers.push(Modifier::Ctrl);
        self
    }

    /// Add Alt modifier
    pub fn with_alt(mut self) -> Self {
        self.modifiers.push(Modifier::Alt);
        self
    }

    /// Add Shift modifier
    pub fn with_shift(mut self) -> Self {
        self.modifiers.push(Modifier::Shift);
        self
    }

    /// Add Super modifier
    pub fn with_super(mut self) -> Self {
        self.modifiers.push(Modifier::Super);
        self
    }

    /// Restrict this binding to a specific mode
    pub fn in_mode(mut self, mode: NavMode) -> Self {
        self.active_in_mode = Some(mode);
        self
    }

    /// Check if this binding matches the current keyboard state
    pub fn matches(&self, keyboard: &ButtonInput<KeyCode>) -> bool {
        // Check primary key is pressed
        if !keyboard.just_pressed(self.key) {
            return false;
        }

        // Check all required modifiers are pressed
        for modifier in &self.modifiers {
            let pressed = match modifier {
                Modifier::Ctrl => {
                    keyboard.any_pressed([KeyCode::ControlLeft, KeyCode::ControlRight])
                }
                Modifier::Alt => keyboard.any_pressed([KeyCode::AltLeft, KeyCode::AltRight]),
                Modifier::Shift => keyboard.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight]),
                Modifier::Super => keyboard.any_pressed([KeyCode::SuperLeft, KeyCode::SuperRight]),
            };

            if !pressed {
                return false;
            }
        }

        true
    }

    /// Check if this binding is active in the given mode
    pub fn is_active_in(&self, mode: NavMode) -> bool {
        self.active_in_mode.map_or(true, |m| m == mode)
    }
}

/// Navigation input router managing keymaps for different styles
#[derive(Resource)]
pub struct NavInputRouter {
    /// Currently active navigation style
    pub current_style: NavStyle,
    /// Keybindings indexed by style
    bindings_by_style: HashMap<NavStyle, Vec<KeyBinding>>,
}

impl NavInputRouter {
    /// Create a new router with default keymaps
    pub fn new(style: NavStyle) -> Self {
        let mut router = Self {
            current_style: style,
            bindings_by_style: HashMap::new(),
        };

        router.register_vimium_bindings();
        router.register_cosmos_bindings();
        router.register_spacemacs_bindings();

        router
    }

    /// Create a router from configuration
    #[allow(dead_code)]
    pub fn from_config(config: &scarab_config::NavConfig) -> Self {
        let style = match config.style {
            scarab_config::NavStyle::Vimium => NavStyle::VimiumStyle,
            scarab_config::NavStyle::Cosmos => NavStyle::CosmosStyle,
            scarab_config::NavStyle::Spacemacs => NavStyle::SpacemacsStyle,
        };

        let mut router = Self::new(style);
        router.apply_custom_bindings(&config.keybindings);
        router
    }

    /// Apply custom keybinding overrides from config
    fn apply_custom_bindings(&mut self, custom: &HashMap<String, String>) {
        let current_style = self.current_style;
        let bindings = self.bindings_by_style.get_mut(&current_style);

        if let Some(bindings) = bindings {
            for (action_name, key_string) in custom {
                if let Some(action) = Self::parse_action_name(action_name) {
                    match KeyBinding::from_string(key_string, action) {
                        Ok(new_binding) => {
                            // Remove any existing binding for this action
                            bindings.retain(|b| b.action != action);
                            // Add the new custom binding
                            bindings.push(new_binding);
                            info!(
                                "Applied custom keybinding: {} -> {}",
                                action_name, key_string
                            );
                        }
                        Err(e) => {
                            warn!(
                                "Failed to parse keybinding '{}' for action '{}': {}",
                                key_string, action_name, e
                            );
                        }
                    }
                } else {
                    warn!("Unknown navigation action: {}", action_name);
                }
            }
        }
    }

    /// Parse an action name string into a NavAction
    fn parse_action_name(name: &str) -> Option<NavAction> {
        match name {
            // Mode transitions
            "enter_hints" | "enter_hint_mode" => Some(NavAction::EnterHintMode),
            "enter_copy" | "enter_copy_mode" => Some(NavAction::EnterCopyMode),
            "enter_search" | "enter_search_mode" => Some(NavAction::EnterSearchMode),
            "enter_command_palette" => Some(NavAction::EnterCommandPalette),
            "exit_mode" | "exit_current_mode" => Some(NavAction::ExitCurrentMode),
            "cancel" | "cancel_all_modes" => Some(NavAction::CancelAllModes),

            // Hint mode actions
            "activate_hint" => Some(NavAction::ActivateHint),

            // Prompt navigation
            "prev_prompt" | "jump_to_prev_prompt" => Some(NavAction::JumpToPrevPrompt),
            "next_prompt" | "jump_to_next_prompt" => Some(NavAction::JumpToNextPrompt),

            // Copy mode actions
            "copy_mode_toggle_selection" => Some(NavAction::CopyModeToggleSelection),
            "copy_mode_exit" => Some(NavAction::CopyModeExit),

            // Search actions
            "search_forward" => Some(NavAction::SearchForward),
            "search_backward" => Some(NavAction::SearchBackward),
            "next_search_match" => Some(NavAction::NextSearchMatch),
            "prev_search_match" => Some(NavAction::PrevSearchMatch),

            // Command palette actions
            "execute_command" => Some(NavAction::ExecuteCommand),
            "filter_commands" => Some(NavAction::FilterCommands),

            _ => None,
        }
    }

    /// Get the current active keybindings
    pub fn current_bindings(&self) -> &[KeyBinding] {
        self.bindings_by_style
            .get(&self.current_style)
            .map(|v| v.as_slice())
            .unwrap_or(&[])
    }

    /// Register Vimium-style keybindings
    fn register_vimium_bindings(&mut self) {
        let bindings = vec![
            // Hint mode
            KeyBinding::new(KeyCode::KeyF, NavAction::EnterHintMode),
            KeyBinding::new(KeyCode::KeyF, NavAction::EnterHintMode)
                .with_ctrl()
                .in_mode(NavMode::Normal),
            // Escape to cancel
            KeyBinding::new(KeyCode::Escape, NavAction::CancelAllModes),
            // Prompt navigation
            KeyBinding::new(KeyCode::ArrowUp, NavAction::JumpToPrevPrompt).with_ctrl(),
            KeyBinding::new(KeyCode::ArrowDown, NavAction::JumpToNextPrompt).with_ctrl(),
            // Copy mode
            KeyBinding::new(KeyCode::KeyV, NavAction::EnterCopyMode).with_ctrl(),
            KeyBinding::new(KeyCode::Escape, NavAction::CopyModeExit).in_mode(NavMode::Copy),
            // Search
            KeyBinding::new(KeyCode::Slash, NavAction::EnterSearchMode).with_ctrl(),
            KeyBinding::new(KeyCode::KeyN, NavAction::NextSearchMatch).in_mode(NavMode::Search),
            KeyBinding::new(KeyCode::KeyN, NavAction::PrevSearchMatch)
                .with_shift()
                .in_mode(NavMode::Search),
            // Command palette
            KeyBinding::new(KeyCode::KeyP, NavAction::EnterCommandPalette).with_ctrl(),
        ];

        self.bindings_by_style
            .insert(NavStyle::VimiumStyle, bindings);
    }

    /// Register Cosmos-style keybindings (Space as leader)
    fn register_cosmos_bindings(&mut self) {
        let bindings = vec![
            // TODO: Implement leader key pattern for Space
            // For now, use similar bindings to Vimium
            KeyBinding::new(KeyCode::KeyF, NavAction::EnterHintMode),
            KeyBinding::new(KeyCode::Escape, NavAction::CancelAllModes),
            KeyBinding::new(KeyCode::ArrowUp, NavAction::JumpToPrevPrompt).with_ctrl(),
            KeyBinding::new(KeyCode::ArrowDown, NavAction::JumpToNextPrompt).with_ctrl(),
        ];

        self.bindings_by_style
            .insert(NavStyle::CosmosStyle, bindings);
    }

    /// Register Spacemacs-style keybindings (SPC prefix)
    fn register_spacemacs_bindings(&mut self) {
        let bindings = vec![
            // TODO: Implement SPC prefix pattern
            // For now, use similar bindings to Vimium
            KeyBinding::new(KeyCode::KeyF, NavAction::EnterHintMode),
            KeyBinding::new(KeyCode::Escape, NavAction::CancelAllModes),
            KeyBinding::new(KeyCode::ArrowUp, NavAction::JumpToPrevPrompt).with_ctrl(),
            KeyBinding::new(KeyCode::ArrowDown, NavAction::JumpToNextPrompt).with_ctrl(),
        ];

        self.bindings_by_style
            .insert(NavStyle::SpacemacsStyle, bindings);
    }

    /// Change the active navigation style
    pub fn set_style(&mut self, style: NavStyle) {
        self.current_style = style;
    }
}

impl Default for NavInputRouter {
    fn default() -> Self {
        Self::new(NavStyle::VimiumStyle)
    }
}

/// Stack of navigation modes allowing nested mode entry
#[derive(Resource, Debug)]
pub struct ModeStack {
    /// Stack of active modes (top = current mode)
    modes: Vec<NavMode>,
}

impl ModeStack {
    /// Create a new mode stack starting in Normal mode
    pub fn new() -> Self {
        Self {
            modes: vec![NavMode::Normal],
        }
    }

    /// Get the current active mode
    pub fn current(&self) -> NavMode {
        *self.modes.last().unwrap_or(&NavMode::Normal)
    }

    /// Push a new mode onto the stack
    pub fn push(&mut self, mode: NavMode) {
        info!("Entering navigation mode: {:?}", mode);
        self.modes.push(mode);
    }

    /// Pop the current mode and return to parent
    pub fn pop(&mut self) -> Option<NavMode> {
        if self.modes.len() > 1 {
            let popped = self.modes.pop();
            info!(
                "Exiting navigation mode: {:?}, returning to {:?}",
                popped,
                self.current()
            );
            popped
        } else {
            warn!("Cannot pop Normal mode from stack");
            None
        }
    }

    /// Clear all modes and return to Normal
    pub fn clear(&mut self) {
        info!("Clearing all navigation modes, returning to Normal");
        self.modes.clear();
        self.modes.push(NavMode::Normal);
    }

    /// Check if a specific mode is active
    pub fn is_in(&self, mode: NavMode) -> bool {
        self.current() == mode
    }

    /// Check if we're in Normal mode
    pub fn is_normal(&self) -> bool {
        self.is_in(NavMode::Normal)
    }

    /// Get the depth of the mode stack
    pub fn depth(&self) -> usize {
        self.modes.len()
    }
}

impl Default for ModeStack {
    fn default() -> Self {
        Self::new()
    }
}

/// System: Route navigation input to appropriate handlers
///
/// This system runs in PreUpdate and:
/// 1. Checks current NavMode from ModeStack
/// 2. Routes keyboard input to appropriate handler based on mode
/// 3. Emits NavAction events based on keymap configuration
/// 4. Handles mode transitions (entering/exiting modes)
pub fn route_nav_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    router: Res<NavInputRouter>,
    mut mode_stack: ResMut<ModeStack>,
    mut action_writer: EventWriter<NavAction>,
) {
    let current_mode = mode_stack.current();

    // Check each binding in the current style
    for binding in router.current_bindings() {
        // Skip bindings not active in current mode
        if !binding.is_active_in(current_mode) {
            continue;
        }

        // Check if binding matches current keyboard state
        if binding.matches(&keyboard) {
            info!(
                "Navigation input matched: {:?} -> {:?} (mode: {:?})",
                binding.key, binding.action, current_mode
            );

            // Handle mode transitions
            match binding.action {
                NavAction::EnterHintMode => {
                    mode_stack.push(NavMode::Hints);
                    action_writer.send(binding.action);
                }
                NavAction::EnterCopyMode => {
                    mode_stack.push(NavMode::Copy);
                    action_writer.send(binding.action);
                }
                NavAction::EnterSearchMode => {
                    mode_stack.push(NavMode::Search);
                    action_writer.send(binding.action);
                }
                NavAction::EnterCommandPalette => {
                    mode_stack.push(NavMode::CommandPalette);
                    action_writer.send(binding.action);
                }
                NavAction::ExitCurrentMode => {
                    mode_stack.pop();
                    action_writer.send(binding.action);
                }
                NavAction::CancelAllModes => {
                    mode_stack.clear();
                    action_writer.send(binding.action);
                }
                // All other actions are just passed through
                _ => {
                    action_writer.send(binding.action);
                }
            }

            // Only process the first matching binding
            return;
        }
    }

    // Handle letter key input in Hints mode
    if mode_stack.is_in(NavMode::Hints) {
        for key_code in keyboard.get_just_pressed() {
            if let Some(ch) = keycode_to_char(*key_code) {
                action_writer.send(NavAction::HintChar(ch));
            }
        }
    }
}

/// Convert KeyCode to lowercase character (for hint selection)
fn keycode_to_char(keycode: KeyCode) -> Option<char> {
    match keycode {
        KeyCode::KeyA => Some('a'),
        KeyCode::KeyB => Some('b'),
        KeyCode::KeyC => Some('c'),
        KeyCode::KeyD => Some('d'),
        KeyCode::KeyE => Some('e'),
        KeyCode::KeyF => Some('f'),
        KeyCode::KeyG => Some('g'),
        KeyCode::KeyH => Some('h'),
        KeyCode::KeyI => Some('i'),
        KeyCode::KeyJ => Some('j'),
        KeyCode::KeyK => Some('k'),
        KeyCode::KeyL => Some('l'),
        KeyCode::KeyM => Some('m'),
        KeyCode::KeyN => Some('n'),
        KeyCode::KeyO => Some('o'),
        KeyCode::KeyP => Some('p'),
        KeyCode::KeyQ => Some('q'),
        KeyCode::KeyR => Some('r'),
        KeyCode::KeyS => Some('s'),
        KeyCode::KeyT => Some('t'),
        KeyCode::KeyU => Some('u'),
        KeyCode::KeyV => Some('v'),
        KeyCode::KeyW => Some('w'),
        KeyCode::KeyX => Some('x'),
        KeyCode::KeyY => Some('y'),
        KeyCode::KeyZ => Some('z'),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mode_stack_basic() {
        let mut stack = ModeStack::new();
        assert_eq!(stack.current(), NavMode::Normal);
        assert!(stack.is_normal());

        stack.push(NavMode::Hints);
        assert_eq!(stack.current(), NavMode::Hints);
        assert!(!stack.is_normal());

        stack.pop();
        assert_eq!(stack.current(), NavMode::Normal);
    }

    #[test]
    fn test_mode_stack_nested() {
        let mut stack = ModeStack::new();

        stack.push(NavMode::CommandPalette);
        stack.push(NavMode::Hints);
        assert_eq!(stack.depth(), 3); // Normal + CommandPalette + Hints

        stack.pop();
        assert_eq!(stack.current(), NavMode::CommandPalette);

        stack.clear();
        assert_eq!(stack.current(), NavMode::Normal);
        assert_eq!(stack.depth(), 1);
    }

    #[test]
    fn test_mode_stack_cannot_pop_normal() {
        let mut stack = ModeStack::new();
        assert_eq!(stack.pop(), None);
        assert_eq!(stack.current(), NavMode::Normal);
    }

    #[test]
    fn test_key_binding_creation() {
        let binding = KeyBinding::new(KeyCode::KeyF, NavAction::EnterHintMode)
            .with_ctrl()
            .in_mode(NavMode::Normal);

        assert_eq!(binding.key, KeyCode::KeyF);
        assert!(binding.modifiers.contains(&Modifier::Ctrl));
        assert_eq!(binding.active_in_mode, Some(NavMode::Normal));
        assert!(binding.is_active_in(NavMode::Normal));
        assert!(!binding.is_active_in(NavMode::Hints));
    }

    #[test]
    fn test_router_default_style() {
        let router = NavInputRouter::default();
        assert_eq!(router.current_style, NavStyle::VimiumStyle);
        assert!(!router.current_bindings().is_empty());
    }

    #[test]
    fn test_router_style_switching() {
        let mut router = NavInputRouter::default();
        assert_eq!(router.current_style, NavStyle::VimiumStyle);

        router.set_style(NavStyle::CosmosStyle);
        assert_eq!(router.current_style, NavStyle::CosmosStyle);
    }

    #[test]
    fn test_keycode_to_char() {
        assert_eq!(keycode_to_char(KeyCode::KeyA), Some('a'));
        assert_eq!(keycode_to_char(KeyCode::KeyZ), Some('z'));
        assert_eq!(keycode_to_char(KeyCode::Escape), None);
        assert_eq!(keycode_to_char(KeyCode::Digit1), None);
    }

    #[test]
    fn test_nav_action_event_equality() {
        let action1 = NavAction::EnterHintMode;
        let action2 = NavAction::EnterHintMode;
        let action3 = NavAction::EnterCopyMode;

        assert_eq!(action1, action2);
        assert_ne!(action1, action3);
    }

    // KeyBinding::from_string() tests
    #[test]
    fn test_key_binding_from_string_simple() {
        let binding = KeyBinding::from_string("F", NavAction::EnterHintMode).unwrap();
        assert_eq!(binding.key, KeyCode::KeyF);
        assert!(binding.modifiers.is_empty());
        assert_eq!(binding.action, NavAction::EnterHintMode);
    }

    #[test]
    fn test_key_binding_from_string_with_ctrl() {
        let binding = KeyBinding::from_string("Ctrl+F", NavAction::EnterHintMode).unwrap();
        assert_eq!(binding.key, KeyCode::KeyF);
        assert_eq!(binding.modifiers.len(), 1);
        assert!(binding.modifiers.contains(&Modifier::Ctrl));
    }

    #[test]
    fn test_key_binding_from_string_with_multiple_modifiers() {
        let binding =
            KeyBinding::from_string("Ctrl+Alt+Shift+Enter", NavAction::EnterSearchMode).unwrap();
        assert_eq!(binding.key, KeyCode::Enter);
        assert_eq!(binding.modifiers.len(), 3);
        assert!(binding.modifiers.contains(&Modifier::Ctrl));
        assert!(binding.modifiers.contains(&Modifier::Alt));
        assert!(binding.modifiers.contains(&Modifier::Shift));
    }

    #[test]
    fn test_key_binding_from_string_escape() {
        let binding = KeyBinding::from_string("Escape", NavAction::CancelAllModes).unwrap();
        assert_eq!(binding.key, KeyCode::Escape);
        assert!(binding.modifiers.is_empty());
    }

    #[test]
    fn test_key_binding_from_string_arrow_keys() {
        let up = KeyBinding::from_string("ArrowUp", NavAction::JumpToPrevPrompt).unwrap();
        assert_eq!(up.key, KeyCode::ArrowUp);

        let down = KeyBinding::from_string("Down", NavAction::JumpToNextPrompt).unwrap();
        assert_eq!(down.key, KeyCode::ArrowDown);
    }

    #[test]
    fn test_key_binding_from_string_function_keys() {
        let f1 = KeyBinding::from_string("F1", NavAction::EnterCommandPalette).unwrap();
        assert_eq!(f1.key, KeyCode::F1);

        let f12 = KeyBinding::from_string("F12", NavAction::EnterCommandPalette).unwrap();
        assert_eq!(f12.key, KeyCode::F12);
    }

    #[test]
    fn test_key_binding_from_string_case_insensitive() {
        let lower = KeyBinding::from_string("f", NavAction::EnterHintMode).unwrap();
        let upper = KeyBinding::from_string("F", NavAction::EnterHintMode).unwrap();
        assert_eq!(lower.key, upper.key);
        assert_eq!(lower.key, KeyCode::KeyF);
    }

    #[test]
    fn test_key_binding_from_string_error_empty() {
        let result = KeyBinding::from_string("", NavAction::EnterHintMode);
        assert!(result.is_err());
    }

    #[test]
    fn test_key_binding_from_string_error_unknown_key() {
        let result = KeyBinding::from_string("UnknownKey", NavAction::EnterHintMode);
        assert!(result.is_err());
    }

    // NavInputRouter::from_config() tests
    #[test]
    fn test_router_from_config_vimium() {
        let config = scarab_config::NavConfig {
            style: scarab_config::NavStyle::Vimium,
            keybindings: std::collections::HashMap::new(),
            allow_plugin_hint_mode: true,
            allow_plugin_focusables: true,
        };

        let router = NavInputRouter::from_config(&config);
        assert_eq!(router.current_style, NavStyle::VimiumStyle);
        assert!(!router.current_bindings().is_empty());
    }

    #[test]
    fn test_router_from_config_cosmos() {
        let config = scarab_config::NavConfig {
            style: scarab_config::NavStyle::Cosmos,
            keybindings: std::collections::HashMap::new(),
            allow_plugin_hint_mode: true,
            allow_plugin_focusables: true,
        };

        let router = NavInputRouter::from_config(&config);
        assert_eq!(router.current_style, NavStyle::CosmosStyle);
    }

    #[test]
    fn test_router_from_config_with_custom_bindings() {
        let mut custom_bindings = std::collections::HashMap::new();
        custom_bindings.insert("enter_hints".to_string(), "Ctrl+H".to_string());
        custom_bindings.insert("cancel".to_string(), "Ctrl+C".to_string());

        let config = scarab_config::NavConfig {
            style: scarab_config::NavStyle::Vimium,
            keybindings: custom_bindings,
            allow_plugin_hint_mode: true,
            allow_plugin_focusables: true,
        };

        let router = NavInputRouter::from_config(&config);

        // Verify custom bindings were applied
        let has_ctrl_h = router
            .current_bindings()
            .iter()
            .any(|b| b.key == KeyCode::KeyH && b.modifiers.contains(&Modifier::Ctrl));
        assert!(has_ctrl_h, "Custom Ctrl+H binding should be present");

        let has_ctrl_c = router
            .current_bindings()
            .iter()
            .any(|b| b.key == KeyCode::KeyC && b.modifiers.contains(&Modifier::Ctrl));
        assert!(has_ctrl_c, "Custom Ctrl+C binding should be present");
    }

    #[test]
    fn test_parse_action_name() {
        assert_eq!(
            NavInputRouter::parse_action_name("enter_hints"),
            Some(NavAction::EnterHintMode)
        );
        assert_eq!(
            NavInputRouter::parse_action_name("cancel"),
            Some(NavAction::CancelAllModes)
        );
        assert_eq!(
            NavInputRouter::parse_action_name("prev_prompt"),
            Some(NavAction::JumpToPrevPrompt)
        );
        assert_eq!(NavInputRouter::parse_action_name("unknown_action"), None);
    }
}
