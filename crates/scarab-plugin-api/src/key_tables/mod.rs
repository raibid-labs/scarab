//! Key Tables & Modal Editing
//!
//! This module implements WezTerm-style key tables for modal keyboard configurations.
//! Different key bindings can be active in different contexts (resize mode, copy mode, etc).

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

pub mod defaults;
pub mod leader;
pub mod stack;

pub use defaults::{
    default_copy_mode_table, default_resize_mode_table, default_search_mode_table, KeyTableRegistry,
};
pub use leader::{LeaderKeyConfig, LeaderKeyState};
pub use stack::{KeyTableActivation, KeyTableStack};

/// A named key table containing key bindings
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct KeyTable {
    /// Name of the key table (e.g., "resize_pane", "copy_mode")
    pub name: String,
    /// Map of key combinations to actions
    pub bindings: HashMap<KeyCombo, KeyAction>,
}

impl KeyTable {
    /// Create a new empty key table
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            bindings: HashMap::new(),
        }
    }

    /// Add a key binding to this table
    pub fn bind(&mut self, combo: KeyCombo, action: KeyAction) {
        self.bindings.insert(combo, action);
    }

    /// Look up an action for a key combination
    pub fn get(&self, combo: &KeyCombo) -> Option<&KeyAction> {
        self.bindings.get(combo)
    }
}

/// A key combination: key code + modifiers
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct KeyCombo {
    /// The key code
    pub key: KeyCode,
    /// Modifier keys
    pub mods: KeyModifiers,
}

impl KeyCombo {
    /// Create a new key combination
    pub fn new(key: KeyCode, mods: KeyModifiers) -> Self {
        Self { key, mods }
    }

    /// Create a key combination with no modifiers
    pub fn key(key: KeyCode) -> Self {
        Self {
            key,
            mods: KeyModifiers::NONE,
        }
    }

    /// Create a key combination with Ctrl modifier
    pub fn ctrl(key: KeyCode) -> Self {
        Self {
            key,
            mods: KeyModifiers::CTRL,
        }
    }

    /// Create a key combination with Shift modifier
    pub fn shift(key: KeyCode) -> Self {
        Self {
            key,
            mods: KeyModifiers::SHIFT,
        }
    }

    /// Create a key combination with Alt modifier
    pub fn alt(key: KeyCode) -> Self {
        Self {
            key,
            mods: KeyModifiers::ALT,
        }
    }

    /// Create a key combination with Super modifier
    pub fn super_key(key: KeyCode) -> Self {
        Self {
            key,
            mods: KeyModifiers::SUPER,
        }
    }
}

/// Key codes - simplified representation
/// In practice, this would integrate with the windowing system's key codes
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum KeyCode {
    // Letters
    KeyA,
    KeyB,
    KeyC,
    KeyD,
    KeyE,
    KeyF,
    KeyG,
    KeyH,
    KeyI,
    KeyJ,
    KeyK,
    KeyL,
    KeyM,
    KeyN,
    KeyO,
    KeyP,
    KeyQ,
    KeyR,
    KeyS,
    KeyT,
    KeyU,
    KeyV,
    KeyW,
    KeyX,
    KeyY,
    KeyZ,

    // Numbers
    Digit0,
    Digit1,
    Digit2,
    Digit3,
    Digit4,
    Digit5,
    Digit6,
    Digit7,
    Digit8,
    Digit9,

    // Function keys
    F1,
    F2,
    F3,
    F4,
    F5,
    F6,
    F7,
    F8,
    F9,
    F10,
    F11,
    F12,

    // Special keys
    Escape,
    Enter,
    Tab,
    Backspace,
    Space,
    Slash,

    // Arrow keys
    Left,
    Right,
    Up,
    Down,

    // Navigation
    Home,
    End,
    PageUp,
    PageDown,
    Insert,
    Delete,

    // Modifiers (when pressed alone)
    ControlLeft,
    ControlRight,
    AltLeft,
    AltRight,
    ShiftLeft,
    ShiftRight,
    SuperLeft,
    SuperRight,
}

bitflags::bitflags! {
    /// Modifier key flags
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
    pub struct KeyModifiers: u8 {
        /// No modifiers
        const NONE = 0;
        /// Control key
        const CTRL = 1 << 0;
        /// Alt/Option key
        const ALT = 1 << 1;
        /// Shift key
        const SHIFT = 1 << 2;
        /// Super/Windows/Command key
        const SUPER = 1 << 3;
        /// Leader key (virtual modifier)
        const LEADER = 1 << 4;
    }
}

impl KeyModifiers {
    /// Check if Ctrl is pressed
    pub fn ctrl(self) -> bool {
        self.contains(KeyModifiers::CTRL)
    }

    /// Check if Alt is pressed
    pub fn alt(self) -> bool {
        self.contains(KeyModifiers::ALT)
    }

    /// Check if Shift is pressed
    pub fn shift(self) -> bool {
        self.contains(KeyModifiers::SHIFT)
    }

    /// Check if Super is pressed
    pub fn super_key(self) -> bool {
        self.contains(KeyModifiers::SUPER)
    }

    /// Check if Leader is active
    pub fn leader(self) -> bool {
        self.contains(KeyModifiers::LEADER)
    }
}

/// How to activate a key table
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ActivateKeyTableMode {
    /// Table stays active until explicitly popped
    Persistent,
    /// Table pops after any keypress (one-shot mode)
    OneShot,
    /// Table has a timeout
    Timeout(Duration),
    /// Table stays until a specific action is triggered
    UntilAction(Box<KeyAction>),
}

/// Actions that can be triggered by key bindings
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum KeyAction {
    // Table management
    /// Activate a key table by name
    ActivateKeyTable {
        name: String,
        mode: ActivateKeyTableMode,
        replace_current: bool,
    },
    /// Pop the current key table from the stack
    PopKeyTable,
    /// Clear the entire key table stack
    ClearKeyTableStack,

    // Pane actions
    /// Activate pane in a direction
    ActivatePaneDirection(Direction),
    /// Adjust pane size
    AdjustPaneSize { direction: Direction, amount: i32 },
    /// Split pane
    SplitPane { direction: SplitDirection },
    /// Close current pane
    ClosePane,
    /// Zoom/maximize current pane
    ZoomPane,

    // Tab actions
    /// Activate tab by index (0-based)
    ActivateTab(i32),
    /// Activate tab relative to current (+1, -1, etc)
    ActivateTabRelative(i32),
    /// Create new tab
    SpawnTab,
    /// Close current tab
    CloseTab,
    /// Move tab to index
    MoveTab(i32),

    // Window actions
    /// Create new window
    SpawnWindow,
    /// Close current window
    CloseWindow,
    /// Toggle fullscreen
    ToggleFullscreen,

    // Terminal actions
    /// Send a string to the terminal
    SendString(String),
    /// Send a specific key combination
    SendKey { key: KeyCode, mods: KeyModifiers },
    /// Scroll by pages
    ScrollByPage(i32),
    /// Scroll by lines
    ScrollByLine(i32),
    /// Scroll to top
    ScrollToTop,
    /// Scroll to bottom
    ScrollToBottom,
    /// Clear scrollback buffer
    ClearScrollback,

    // Clipboard
    /// Copy selection
    Copy,
    /// Paste from clipboard
    Paste,
    /// Copy to specific clipboard
    CopyTo(ClipboardKind),
    /// Paste from specific clipboard
    PasteFrom(ClipboardKind),

    // Mode actions
    /// Enter copy mode
    ActivateCopyMode,
    /// Enter search mode
    ActivateSearchMode,
    /// Copy mode specific actions
    CopyMode(CopyModeAction),
    /// Search mode specific actions
    Search(SearchAction),

    // Custom actions
    /// Emit a custom event
    EmitEvent { event: String, args: Vec<String> },
    /// Run a shell command
    RunCommand(String),
    /// No operation
    Noop,
}

/// Directional navigation
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Direction {
    Left,
    Right,
    Up,
    Down,
}

/// Split direction
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SplitDirection {
    Horizontal,
    Vertical,
}

/// Clipboard kinds
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ClipboardKind {
    /// Primary selection (X11)
    Primary,
    /// System clipboard
    Clipboard,
}

/// Copy mode specific actions
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum CopyModeAction {
    // Movement
    MoveLeft,
    MoveRight,
    MoveUp,
    MoveDown,
    MoveWordForward,
    MoveWordBackward,
    MoveToLineStart,
    MoveToLineEnd,
    MoveToTop,
    MoveToBottom,

    // Selection
    ToggleSelection,
    ToggleLineSelection,
    ToggleBlockSelection,

    // Search
    SearchForward,
    SearchBackward,
    NextMatch,
    PrevMatch,

    // Actions
    CopyAndExit,
    Exit,
}

/// Search mode specific actions
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SearchAction {
    /// Confirm search and find first match
    Confirm,
    /// Cancel search
    Cancel,
    /// Go to next match
    NextMatch,
    /// Go to previous match
    PrevMatch,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_modifiers_flags() {
        let mods = KeyModifiers::CTRL | KeyModifiers::SHIFT;
        assert!(mods.ctrl());
        assert!(mods.shift());
        assert!(!mods.alt());
        assert!(!mods.super_key());
        assert!(!mods.leader());
    }

    #[test]
    fn test_key_modifiers_none() {
        let mods = KeyModifiers::NONE;
        assert!(!mods.ctrl());
        assert!(!mods.alt());
        assert!(!mods.shift());
        assert!(!mods.super_key());
        assert!(!mods.leader());
    }

    #[test]
    fn test_key_combo_equality() {
        let combo1 = KeyCombo::new(KeyCode::KeyH, KeyModifiers::CTRL);
        let combo2 = KeyCombo::new(KeyCode::KeyH, KeyModifiers::CTRL);
        let combo3 = KeyCombo::new(KeyCode::KeyH, KeyModifiers::ALT);

        assert_eq!(combo1, combo2);
        assert_ne!(combo1, combo3);
    }

    #[test]
    fn test_key_table_binding() {
        let mut table = KeyTable::new("test");
        let combo = KeyCombo::new(KeyCode::KeyH, KeyModifiers::NONE);
        let action = KeyAction::Noop;

        table.bind(combo.clone(), action.clone());

        assert_eq!(table.get(&combo), Some(&action));
    }

    #[test]
    fn test_key_combo_hash() {
        use std::collections::HashSet;

        let mut set = HashSet::new();
        let combo = KeyCombo::new(KeyCode::KeyA, KeyModifiers::CTRL);

        set.insert(combo.clone());
        assert!(set.contains(&combo));
    }
}
