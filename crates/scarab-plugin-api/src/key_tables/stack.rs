//! Key Table Stack
//!
//! Manages a stack of active key tables for modal keyboard configurations.

use super::{ActivateKeyTableMode, KeyAction, KeyCombo, KeyTable};
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};

/// A stack of active key tables
#[derive(Clone, Debug)]
pub struct KeyTableStack {
    /// Stack of active table activations (top = most recent)
    stack: Vec<KeyTableActivation>,
    /// Default key table (always at bottom of stack)
    default_table: KeyTable,
}

impl KeyTableStack {
    /// Create a new key table stack with a default table
    pub fn new(default_table: KeyTable) -> Self {
        Self {
            stack: Vec::new(),
            default_table,
        }
    }

    /// Push a new key table activation onto the stack
    pub fn push(&mut self, activation: KeyTableActivation) {
        self.stack.push(activation);
    }

    /// Pop the top key table from the stack
    pub fn pop(&mut self) -> Option<KeyTableActivation> {
        self.stack.pop()
    }

    /// Get the current (top) key table activation
    pub fn current(&self) -> Option<&KeyTableActivation> {
        self.stack.last()
    }

    /// Get the name of the current key table, or "default" if stack is empty
    pub fn current_name(&self) -> &str {
        self.current()
            .map(|a| a.name.as_str())
            .unwrap_or("default")
    }

    /// Clear the entire stack
    pub fn clear(&mut self) {
        self.stack.clear();
    }

    /// Check if the stack is empty
    pub fn is_empty(&self) -> bool {
        self.stack.is_empty()
    }

    /// Get the number of tables on the stack (not including default)
    pub fn len(&self) -> usize {
        self.stack.len()
    }

    /// Resolve a key combination by searching the stack from top to bottom
    ///
    /// Returns the action if found, None otherwise
    pub fn resolve(&self, combo: &KeyCombo) -> Option<&KeyAction> {
        // Search from top of stack downward
        for activation in self.stack.iter().rev() {
            if let Some(action) = activation.table.get(combo) {
                return Some(action);
            }
        }

        // Fall through to default table
        self.default_table.get(combo)
    }

    /// Handle a key press, resolving it and managing one-shot/timeout behavior
    ///
    /// Returns the action if found, None otherwise
    pub fn handle_key(&mut self, combo: KeyCombo, now: Instant) -> Option<KeyAction> {
        // First, expire any timed-out tables
        self.expire_timeouts(now);

        // Try to resolve the key
        let action = self.resolve(&combo).cloned();

        // Handle one-shot mode: pop after any keypress
        if let Some(top) = self.stack.last() {
            if matches!(top.mode, ActivateKeyTableMode::OneShot) {
                self.stack.pop();
            }
        }

        action
    }

    /// Remove expired tables from the stack
    fn expire_timeouts(&mut self, now: Instant) {
        self.stack.retain(|activation| {
            activation
                .timeout
                .map(|timeout| now < timeout)
                .unwrap_or(true)
        });
    }

    /// Check if any tables have timeouts and return the earliest timeout
    pub fn next_timeout(&self) -> Option<Instant> {
        self.stack
            .iter()
            .filter_map(|a| a.timeout)
            .min()
    }

    /// Get a reference to the default table
    pub fn default_table(&self) -> &KeyTable {
        &self.default_table
    }

    /// Get a mutable reference to the default table
    pub fn default_table_mut(&mut self) -> &mut KeyTable {
        &mut self.default_table
    }
}

impl Default for KeyTableStack {
    fn default() -> Self {
        Self::new(KeyTable::new("default"))
    }
}

/// An activation of a key table on the stack
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct KeyTableActivation {
    /// Name of the activated table
    pub name: String,
    /// The key table itself
    pub table: KeyTable,
    /// Activation mode
    pub mode: ActivateKeyTableMode,
    /// Optional timeout (absolute time when this activation expires)
    #[serde(skip)]
    pub timeout: Option<Instant>,
    /// Whether this activation replaces the previous one
    pub replace_current: bool,
}

impl KeyTableActivation {
    /// Create a new persistent activation
    pub fn persistent(name: String, table: KeyTable) -> Self {
        Self {
            name,
            table,
            mode: ActivateKeyTableMode::Persistent,
            timeout: None,
            replace_current: false,
        }
    }

    /// Create a new one-shot activation
    pub fn one_shot(name: String, table: KeyTable) -> Self {
        Self {
            name,
            table,
            mode: ActivateKeyTableMode::OneShot,
            timeout: None,
            replace_current: false,
        }
    }

    /// Create a new timed activation
    pub fn timed(name: String, table: KeyTable, duration: Duration, now: Instant) -> Self {
        Self {
            name,
            table,
            mode: ActivateKeyTableMode::Timeout(duration),
            timeout: Some(now + duration),
            replace_current: false,
        }
    }

    /// Set whether this activation should replace the current one
    pub fn with_replace(mut self, replace: bool) -> Self {
        self.replace_current = replace;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::key_tables::{KeyCode, KeyModifiers};

    fn create_test_table(name: &str) -> KeyTable {
        let mut table = KeyTable::new(name);
        table.bind(
            KeyCombo::new(KeyCode::KeyH, KeyModifiers::NONE),
            KeyAction::Noop,
        );
        table
    }

    #[test]
    fn test_stack_push_pop() {
        let mut stack = KeyTableStack::default();
        assert_eq!(stack.len(), 0);
        assert!(stack.is_empty());

        let activation = KeyTableActivation::persistent("test".into(), create_test_table("test"));
        stack.push(activation);

        assert_eq!(stack.len(), 1);
        assert!(!stack.is_empty());
        assert_eq!(stack.current_name(), "test");

        stack.pop();
        assert_eq!(stack.len(), 0);
        assert!(stack.is_empty());
    }

    #[test]
    fn test_key_resolution_stack() {
        let mut stack = KeyTableStack::default();

        // Add a table with H bound
        let table1 = create_test_table("test1");
        stack.push(KeyTableActivation::persistent("test1".into(), table1));

        // H should resolve to the table's action
        let combo = KeyCombo::new(KeyCode::KeyH, KeyModifiers::NONE);
        assert!(stack.resolve(&combo).is_some());

        // J should not resolve (not in any table)
        let combo = KeyCombo::new(KeyCode::KeyJ, KeyModifiers::NONE);
        assert!(stack.resolve(&combo).is_none());
    }

    #[test]
    fn test_one_shot_pop() {
        let mut stack = KeyTableStack::default();
        let now = Instant::now();

        // Push a one-shot table
        let table = create_test_table("oneshot");
        stack.push(KeyTableActivation::one_shot("oneshot".into(), table));

        assert_eq!(stack.len(), 1);

        // Any keypress should pop the table
        let combo = KeyCombo::new(KeyCode::KeyX, KeyModifiers::NONE);
        stack.handle_key(combo, now);

        assert_eq!(stack.len(), 0);
    }

    #[test]
    fn test_timeout_expiration() {
        let mut stack = KeyTableStack::default();
        let now = Instant::now();

        // Push a table with 100ms timeout
        let table = create_test_table("timed");
        stack.push(KeyTableActivation::timed(
            "timed".into(),
            table,
            Duration::from_millis(100),
            now,
        ));

        assert_eq!(stack.len(), 1);

        // Should still be there immediately
        stack.expire_timeouts(now);
        assert_eq!(stack.len(), 1);

        // Should be gone after timeout
        let future = now + Duration::from_millis(101);
        stack.expire_timeouts(future);
        assert_eq!(stack.len(), 0);
    }

    #[test]
    fn test_multiple_tables_resolution() {
        let mut stack = KeyTableStack::default();

        // Bottom table: H -> Noop
        let mut table1 = KeyTable::new("bottom");
        table1.bind(
            KeyCombo::new(KeyCode::KeyH, KeyModifiers::NONE),
            KeyAction::Noop,
        );
        stack.push(KeyTableActivation::persistent("bottom".into(), table1));

        // Top table: H -> PopKeyTable (overrides bottom)
        let mut table2 = KeyTable::new("top");
        table2.bind(
            KeyCombo::new(KeyCode::KeyH, KeyModifiers::NONE),
            KeyAction::PopKeyTable,
        );
        stack.push(KeyTableActivation::persistent("top".into(), table2));

        // H should resolve to PopKeyTable (from top table)
        let combo = KeyCombo::new(KeyCode::KeyH, KeyModifiers::NONE);
        let action = stack.resolve(&combo);
        assert_eq!(action, Some(&KeyAction::PopKeyTable));
    }

    #[test]
    fn test_clear_stack() {
        let mut stack = KeyTableStack::default();

        stack.push(KeyTableActivation::persistent(
            "test1".into(),
            create_test_table("test1"),
        ));
        stack.push(KeyTableActivation::persistent(
            "test2".into(),
            create_test_table("test2"),
        ));

        assert_eq!(stack.len(), 2);

        stack.clear();
        assert_eq!(stack.len(), 0);
        assert!(stack.is_empty());
    }

    #[test]
    fn test_next_timeout() {
        let mut stack = KeyTableStack::default();
        let now = Instant::now();

        // No timeouts initially
        assert!(stack.next_timeout().is_none());

        // Add table with timeout
        let table = create_test_table("timed");
        stack.push(KeyTableActivation::timed(
            "timed".into(),
            table,
            Duration::from_millis(100),
            now,
        ));

        let next = stack.next_timeout();
        assert!(next.is_some());
        assert!(next.unwrap() > now);
    }

    #[test]
    fn test_default_table_access() {
        let mut stack = KeyTableStack::default();

        // Modify default table
        stack.default_table_mut().bind(
            KeyCombo::new(KeyCode::KeyA, KeyModifiers::CTRL),
            KeyAction::Noop,
        );

        // Should be able to resolve from default table
        let combo = KeyCombo::new(KeyCode::KeyA, KeyModifiers::CTRL);
        assert!(stack.resolve(&combo).is_some());
    }
}
