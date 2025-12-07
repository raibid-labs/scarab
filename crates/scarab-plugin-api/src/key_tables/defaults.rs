//! Default Key Tables
//!
//! This module provides default key bindings for various modal editing modes,
//! matching WezTerm's behavior where appropriate.

use super::{CopyModeAction, Direction, KeyAction, KeyCode, KeyCombo, KeyTable, SearchAction};
use std::collections::HashMap;

/// Create the default copy mode key table
///
/// Copy mode is vim-like and allows for selecting and copying text from the terminal buffer.
/// This table includes:
/// - hjkl navigation
/// - w/b/e word movement
/// - 0/$/^ line movement
/// - g/G document movement
/// - Ctrl+u/d half page movement
/// - v/V/Ctrl+v selection modes
/// - / and ? for search
/// - n/N for next/previous match
/// - y yank (copy), Escape/q exit
pub fn default_copy_mode_table() -> KeyTable {
    let mut table = KeyTable::new("copy_mode");

    // Basic movement - hjkl (vim-style)
    table.bind(
        KeyCombo::key(KeyCode::KeyH),
        KeyAction::CopyMode(CopyModeAction::MoveLeft),
    );
    table.bind(
        KeyCombo::key(KeyCode::KeyJ),
        KeyAction::CopyMode(CopyModeAction::MoveDown),
    );
    table.bind(
        KeyCombo::key(KeyCode::KeyK),
        KeyAction::CopyMode(CopyModeAction::MoveUp),
    );
    table.bind(
        KeyCombo::key(KeyCode::KeyL),
        KeyAction::CopyMode(CopyModeAction::MoveRight),
    );

    // Arrow keys for movement (alternative to hjkl)
    table.bind(
        KeyCombo::key(KeyCode::Left),
        KeyAction::CopyMode(CopyModeAction::MoveLeft),
    );
    table.bind(
        KeyCombo::key(KeyCode::Down),
        KeyAction::CopyMode(CopyModeAction::MoveDown),
    );
    table.bind(
        KeyCombo::key(KeyCode::Up),
        KeyAction::CopyMode(CopyModeAction::MoveUp),
    );
    table.bind(
        KeyCombo::key(KeyCode::Right),
        KeyAction::CopyMode(CopyModeAction::MoveRight),
    );

    // Word movement
    table.bind(
        KeyCombo::key(KeyCode::KeyW),
        KeyAction::CopyMode(CopyModeAction::MoveWordForward),
    );
    table.bind(
        KeyCombo::key(KeyCode::KeyB),
        KeyAction::CopyMode(CopyModeAction::MoveWordBackward),
    );
    table.bind(
        KeyCombo::key(KeyCode::KeyE),
        KeyAction::CopyMode(CopyModeAction::MoveWordForward),
    );

    // Line movement
    table.bind(
        KeyCombo::key(KeyCode::Digit0),
        KeyAction::CopyMode(CopyModeAction::MoveToLineStart),
    );
    table.bind(
        KeyCombo::shift(KeyCode::Digit4), // $ (Shift+4)
        KeyAction::CopyMode(CopyModeAction::MoveToLineEnd),
    );
    table.bind(
        KeyCombo::shift(KeyCode::Digit6), // ^ (Shift+6)
        KeyAction::CopyMode(CopyModeAction::MoveToLineStart),
    );
    table.bind(
        KeyCombo::key(KeyCode::Home),
        KeyAction::CopyMode(CopyModeAction::MoveToLineStart),
    );
    table.bind(
        KeyCombo::key(KeyCode::End),
        KeyAction::CopyMode(CopyModeAction::MoveToLineEnd),
    );

    // Document movement
    table.bind(
        KeyCombo::key(KeyCode::KeyG),
        KeyAction::CopyMode(CopyModeAction::MoveToTop),
    );
    table.bind(
        KeyCombo::shift(KeyCode::KeyG), // G (Shift+g)
        KeyAction::CopyMode(CopyModeAction::MoveToBottom),
    );

    // Page movement
    table.bind(
        KeyCombo::ctrl(KeyCode::KeyU),
        KeyAction::ScrollByPage(-1), // Half page up
    );
    table.bind(
        KeyCombo::ctrl(KeyCode::KeyD),
        KeyAction::ScrollByPage(1), // Half page down
    );
    table.bind(
        KeyCombo::ctrl(KeyCode::KeyB),
        KeyAction::ScrollByPage(-2), // Full page up
    );
    table.bind(
        KeyCombo::ctrl(KeyCode::KeyF),
        KeyAction::ScrollByPage(2), // Full page down
    );
    table.bind(KeyCombo::key(KeyCode::PageUp), KeyAction::ScrollByPage(-1));
    table.bind(KeyCombo::key(KeyCode::PageDown), KeyAction::ScrollByPage(1));

    // Selection modes
    table.bind(
        KeyCombo::key(KeyCode::KeyV),
        KeyAction::CopyMode(CopyModeAction::ToggleSelection),
    );
    table.bind(
        KeyCombo::shift(KeyCode::KeyV), // V (Shift+v)
        KeyAction::CopyMode(CopyModeAction::ToggleLineSelection),
    );
    table.bind(
        KeyCombo::ctrl(KeyCode::KeyV),
        KeyAction::CopyMode(CopyModeAction::ToggleBlockSelection),
    );

    // Search
    table.bind(
        KeyCombo::key(KeyCode::Slash), // /
        KeyAction::CopyMode(CopyModeAction::SearchForward),
    );
    table.bind(
        KeyCombo::shift(KeyCode::Slash), // ? (Shift+/)
        KeyAction::CopyMode(CopyModeAction::SearchBackward),
    );
    table.bind(
        KeyCombo::key(KeyCode::KeyN),
        KeyAction::CopyMode(CopyModeAction::NextMatch),
    );
    table.bind(
        KeyCombo::shift(KeyCode::KeyN), // N (Shift+n)
        KeyAction::CopyMode(CopyModeAction::PrevMatch),
    );

    // Copy and exit
    table.bind(
        KeyCombo::key(KeyCode::KeyY),
        KeyAction::CopyMode(CopyModeAction::CopyAndExit),
    );

    // Exit copy mode
    table.bind(
        KeyCombo::key(KeyCode::Escape),
        KeyAction::CopyMode(CopyModeAction::Exit),
    );
    table.bind(
        KeyCombo::key(KeyCode::KeyQ),
        KeyAction::CopyMode(CopyModeAction::Exit),
    );
    table.bind(
        KeyCombo::ctrl(KeyCode::KeyC),
        KeyAction::CopyMode(CopyModeAction::Exit),
    );

    table
}

/// Create the default search mode key table
///
/// Search mode allows for searching and navigating through search results.
/// This table includes:
/// - n/N for next/previous match
/// - Enter to accept search
/// - Escape to cancel search
pub fn default_search_mode_table() -> KeyTable {
    let mut table = KeyTable::new("search_mode");

    // Navigate matches
    table.bind(
        KeyCombo::key(KeyCode::KeyN),
        KeyAction::Search(SearchAction::NextMatch),
    );
    table.bind(
        KeyCombo::shift(KeyCode::KeyN), // N (Shift+n)
        KeyAction::Search(SearchAction::PrevMatch),
    );

    // Accept/confirm search
    table.bind(
        KeyCombo::key(KeyCode::Enter),
        KeyAction::Search(SearchAction::Confirm),
    );

    // Cancel search
    table.bind(
        KeyCombo::key(KeyCode::Escape),
        KeyAction::Search(SearchAction::Cancel),
    );
    table.bind(
        KeyCombo::ctrl(KeyCode::KeyC),
        KeyAction::Search(SearchAction::Cancel),
    );

    // Arrow key navigation
    table.bind(
        KeyCombo::key(KeyCode::Down),
        KeyAction::Search(SearchAction::NextMatch),
    );
    table.bind(
        KeyCombo::key(KeyCode::Up),
        KeyAction::Search(SearchAction::PrevMatch),
    );

    // Ctrl+n/p for next/previous (emacs-style)
    table.bind(
        KeyCombo::ctrl(KeyCode::KeyN),
        KeyAction::Search(SearchAction::NextMatch),
    );
    table.bind(
        KeyCombo::ctrl(KeyCode::KeyP),
        KeyAction::Search(SearchAction::PrevMatch),
    );

    table
}

/// Create the default resize mode key table
///
/// Resize mode allows for resizing panes using keyboard shortcuts.
/// This table includes:
/// - hjkl to resize pane in direction
/// - Arrow keys as alternative
/// - Enter/Escape to exit resize mode
pub fn default_resize_mode_table() -> KeyTable {
    let mut table = KeyTable::new("resize_pane");

    // hjkl resizing (vim-style)
    // Each keypress adjusts the pane boundary in that direction
    table.bind(
        KeyCombo::key(KeyCode::KeyH),
        KeyAction::AdjustPaneSize {
            direction: Direction::Left,
            amount: 2,
        },
    );
    table.bind(
        KeyCombo::key(KeyCode::KeyJ),
        KeyAction::AdjustPaneSize {
            direction: Direction::Down,
            amount: 2,
        },
    );
    table.bind(
        KeyCombo::key(KeyCode::KeyK),
        KeyAction::AdjustPaneSize {
            direction: Direction::Up,
            amount: 2,
        },
    );
    table.bind(
        KeyCombo::key(KeyCode::KeyL),
        KeyAction::AdjustPaneSize {
            direction: Direction::Right,
            amount: 2,
        },
    );

    // Arrow keys for resizing
    table.bind(
        KeyCombo::key(KeyCode::Left),
        KeyAction::AdjustPaneSize {
            direction: Direction::Left,
            amount: 2,
        },
    );
    table.bind(
        KeyCombo::key(KeyCode::Down),
        KeyAction::AdjustPaneSize {
            direction: Direction::Down,
            amount: 2,
        },
    );
    table.bind(
        KeyCombo::key(KeyCode::Up),
        KeyAction::AdjustPaneSize {
            direction: Direction::Up,
            amount: 2,
        },
    );
    table.bind(
        KeyCombo::key(KeyCode::Right),
        KeyAction::AdjustPaneSize {
            direction: Direction::Right,
            amount: 2,
        },
    );

    // Shift + hjkl for larger adjustments
    table.bind(
        KeyCombo::shift(KeyCode::KeyH),
        KeyAction::AdjustPaneSize {
            direction: Direction::Left,
            amount: 5,
        },
    );
    table.bind(
        KeyCombo::shift(KeyCode::KeyJ),
        KeyAction::AdjustPaneSize {
            direction: Direction::Down,
            amount: 5,
        },
    );
    table.bind(
        KeyCombo::shift(KeyCode::KeyK),
        KeyAction::AdjustPaneSize {
            direction: Direction::Up,
            amount: 5,
        },
    );
    table.bind(
        KeyCombo::shift(KeyCode::KeyL),
        KeyAction::AdjustPaneSize {
            direction: Direction::Right,
            amount: 5,
        },
    );

    // Exit resize mode
    table.bind(KeyCombo::key(KeyCode::Enter), KeyAction::PopKeyTable);
    table.bind(KeyCombo::key(KeyCode::Escape), KeyAction::PopKeyTable);
    table.bind(KeyCombo::ctrl(KeyCode::KeyC), KeyAction::PopKeyTable);
    table.bind(KeyCombo::key(KeyCode::KeyQ), KeyAction::PopKeyTable);

    table
}

/// Key table registry for managing named key tables
///
/// This registry stores all available key tables and provides lookup functionality.
pub struct KeyTableRegistry {
    tables: HashMap<String, KeyTable>,
}

impl KeyTableRegistry {
    /// Create a new registry with default key tables registered
    pub fn new() -> Self {
        let mut registry = Self {
            tables: HashMap::new(),
        };

        // Register default tables
        registry.register_table(default_copy_mode_table());
        registry.register_table(default_search_mode_table());
        registry.register_table(default_resize_mode_table());

        registry
    }

    /// Get a key table by name
    pub fn get(&self, name: &str) -> Option<&KeyTable> {
        self.tables.get(name)
    }

    /// Register a key table with the given name
    pub fn register(&mut self, name: impl Into<String>, table: KeyTable) {
        self.tables.insert(name.into(), table);
    }

    /// Register a key table using its own name
    fn register_table(&mut self, table: KeyTable) {
        let name = table.name.clone();
        self.tables.insert(name, table);
    }

    /// Get a mutable reference to a key table
    pub fn get_mut(&mut self, name: &str) -> Option<&mut KeyTable> {
        self.tables.get_mut(name)
    }

    /// Check if a table exists
    pub fn contains(&self, name: &str) -> bool {
        self.tables.contains_key(name)
    }

    /// Get all table names
    pub fn table_names(&self) -> Vec<&str> {
        self.tables.keys().map(|s| s.as_str()).collect()
    }
}

impl Default for KeyTableRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_copy_mode_table() {
        let table = default_copy_mode_table();
        assert_eq!(table.name, "copy_mode");

        // Test basic movement
        let h_action = table.get(&KeyCombo::key(KeyCode::KeyH));
        assert!(matches!(
            h_action,
            Some(KeyAction::CopyMode(CopyModeAction::MoveLeft))
        ));

        let j_action = table.get(&KeyCombo::key(KeyCode::KeyJ));
        assert!(matches!(
            j_action,
            Some(KeyAction::CopyMode(CopyModeAction::MoveDown))
        ));

        // Test exit bindings
        let esc_action = table.get(&KeyCombo::key(KeyCode::Escape));
        assert!(matches!(
            esc_action,
            Some(KeyAction::CopyMode(CopyModeAction::Exit))
        ));

        let q_action = table.get(&KeyCombo::key(KeyCode::KeyQ));
        assert!(matches!(
            q_action,
            Some(KeyAction::CopyMode(CopyModeAction::Exit))
        ));

        // Test selection modes
        let v_action = table.get(&KeyCombo::key(KeyCode::KeyV));
        assert!(matches!(
            v_action,
            Some(KeyAction::CopyMode(CopyModeAction::ToggleSelection))
        ));

        // Test copy and exit
        let y_action = table.get(&KeyCombo::key(KeyCode::KeyY));
        assert!(matches!(
            y_action,
            Some(KeyAction::CopyMode(CopyModeAction::CopyAndExit))
        ));

        // Test search bindings
        let slash_action = table.get(&KeyCombo::key(KeyCode::Slash));
        assert!(matches!(
            slash_action,
            Some(KeyAction::CopyMode(CopyModeAction::SearchForward))
        ));

        let shift_slash_action = table.get(&KeyCombo::shift(KeyCode::Slash));
        assert!(matches!(
            shift_slash_action,
            Some(KeyAction::CopyMode(CopyModeAction::SearchBackward))
        ));

        let n_action = table.get(&KeyCombo::key(KeyCode::KeyN));
        assert!(matches!(
            n_action,
            Some(KeyAction::CopyMode(CopyModeAction::NextMatch))
        ));

        let shift_n_action = table.get(&KeyCombo::shift(KeyCode::KeyN));
        assert!(matches!(
            shift_n_action,
            Some(KeyAction::CopyMode(CopyModeAction::PrevMatch))
        ));
    }

    #[test]
    fn test_default_search_mode_table() {
        let table = default_search_mode_table();
        assert_eq!(table.name, "search_mode");

        // Test next/previous match
        let n_action = table.get(&KeyCombo::key(KeyCode::KeyN));
        assert!(matches!(
            n_action,
            Some(KeyAction::Search(SearchAction::NextMatch))
        ));

        let shift_n_action = table.get(&KeyCombo::shift(KeyCode::KeyN));
        assert!(matches!(
            shift_n_action,
            Some(KeyAction::Search(SearchAction::PrevMatch))
        ));

        // Test confirm
        let enter_action = table.get(&KeyCombo::key(KeyCode::Enter));
        assert!(matches!(
            enter_action,
            Some(KeyAction::Search(SearchAction::Confirm))
        ));

        // Test cancel
        let esc_action = table.get(&KeyCombo::key(KeyCode::Escape));
        assert!(matches!(
            esc_action,
            Some(KeyAction::Search(SearchAction::Cancel))
        ));
    }

    #[test]
    fn test_default_resize_mode_table() {
        let table = default_resize_mode_table();
        assert_eq!(table.name, "resize_pane");

        // Test hjkl resizing
        let h_action = table.get(&KeyCombo::key(KeyCode::KeyH));
        assert!(matches!(
            h_action,
            Some(KeyAction::AdjustPaneSize {
                direction: Direction::Left,
                amount: 2
            })
        ));

        // Test larger adjustments with Shift
        let shift_h_action = table.get(&KeyCombo::shift(KeyCode::KeyH));
        assert!(matches!(
            shift_h_action,
            Some(KeyAction::AdjustPaneSize {
                direction: Direction::Left,
                amount: 5
            })
        ));

        // Test exit
        let enter_action = table.get(&KeyCombo::key(KeyCode::Enter));
        assert_eq!(enter_action, Some(&KeyAction::PopKeyTable));

        let esc_action = table.get(&KeyCombo::key(KeyCode::Escape));
        assert_eq!(esc_action, Some(&KeyAction::PopKeyTable));
    }

    #[test]
    fn test_key_table_registry_creation() {
        let registry = KeyTableRegistry::new();

        // Verify default tables are registered
        assert!(registry.contains("copy_mode"));
        assert!(registry.contains("search_mode"));
        assert!(registry.contains("resize_pane"));
    }

    #[test]
    fn test_key_table_registry_lookup() {
        let registry = KeyTableRegistry::new();

        // Test lookup
        let copy_table = registry.get("copy_mode");
        assert!(copy_table.is_some());
        assert_eq!(copy_table.unwrap().name, "copy_mode");

        let nonexistent = registry.get("nonexistent");
        assert!(nonexistent.is_none());
    }

    #[test]
    fn test_key_table_registry_custom_registration() {
        let mut registry = KeyTableRegistry::new();

        let mut custom_table = KeyTable::new("custom");
        custom_table.bind(KeyCombo::key(KeyCode::KeyA), KeyAction::Noop);

        registry.register("custom", custom_table);

        assert!(registry.contains("custom"));
        let retrieved = registry.get("custom");
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().name, "custom");
    }

    #[test]
    fn test_key_table_registry_table_names() {
        let registry = KeyTableRegistry::new();
        let names = registry.table_names();

        assert_eq!(names.len(), 3);
        assert!(names.contains(&"copy_mode"));
        assert!(names.contains(&"search_mode"));
        assert!(names.contains(&"resize_pane"));
    }

    #[test]
    fn test_copy_mode_arrow_keys() {
        let table = default_copy_mode_table();

        // Test arrow key alternatives
        let left_action = table.get(&KeyCombo::key(KeyCode::Left));
        assert!(matches!(
            left_action,
            Some(KeyAction::CopyMode(CopyModeAction::MoveLeft))
        ));

        let down_action = table.get(&KeyCombo::key(KeyCode::Down));
        assert!(matches!(
            down_action,
            Some(KeyAction::CopyMode(CopyModeAction::MoveDown))
        ));
    }

    #[test]
    fn test_copy_mode_word_movement() {
        let table = default_copy_mode_table();

        let w_action = table.get(&KeyCombo::key(KeyCode::KeyW));
        assert!(matches!(
            w_action,
            Some(KeyAction::CopyMode(CopyModeAction::MoveWordForward))
        ));

        let b_action = table.get(&KeyCombo::key(KeyCode::KeyB));
        assert!(matches!(
            b_action,
            Some(KeyAction::CopyMode(CopyModeAction::MoveWordBackward))
        ));
    }

    #[test]
    fn test_copy_mode_page_movement() {
        let table = default_copy_mode_table();

        let ctrl_u = table.get(&KeyCombo::ctrl(KeyCode::KeyU));
        assert!(matches!(ctrl_u, Some(KeyAction::ScrollByPage(-1))));

        let ctrl_d = table.get(&KeyCombo::ctrl(KeyCode::KeyD));
        assert!(matches!(ctrl_d, Some(KeyAction::ScrollByPage(1))));
    }
}
