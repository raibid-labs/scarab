//! Leader Key State
//!
//! Manages the leader key - a special modifier that becomes temporarily active
//! after being pressed, similar to tmux's prefix key.

use super::{KeyCode, KeyCombo, KeyModifiers};
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};

/// State of the leader key
#[derive(Clone, Debug)]
pub struct LeaderKeyState {
    /// The key combination that activates the leader
    pub key: KeyCombo,
    /// Whether the leader is currently active
    is_active: bool,
    /// When the leader was activated (for timeout)
    activated_at: Option<Instant>,
    /// Timeout duration in milliseconds
    timeout_ms: u64,
    /// Optional prefix sequence (for multi-key leaders like "Ctrl+A, A")
    prefix_sequence: Vec<KeyCombo>,
    /// Keys pressed so far in the sequence
    sequence_progress: Vec<KeyCombo>,
}

impl LeaderKeyState {
    /// Create a new leader key state
    pub fn new(key: KeyCombo, timeout_ms: u64) -> Self {
        Self {
            key,
            is_active: false,
            activated_at: None,
            timeout_ms,
            prefix_sequence: Vec::new(),
            sequence_progress: Vec::new(),
        }
    }

    /// Create a leader key state with a multi-key prefix sequence
    pub fn with_sequence(sequence: Vec<KeyCombo>, timeout_ms: u64) -> Self {
        let key = sequence
            .first()
            .cloned()
            .unwrap_or_else(|| KeyCombo::new(KeyCode::KeyA, KeyModifiers::CTRL));

        Self {
            key,
            is_active: false,
            activated_at: None,
            timeout_ms,
            prefix_sequence: sequence,
            sequence_progress: Vec::new(),
        }
    }

    /// Check if the leader is currently active
    pub fn is_active(&self) -> bool {
        self.is_active
    }

    /// Get the timeout duration
    pub fn timeout(&self) -> Duration {
        Duration::from_millis(self.timeout_ms)
    }

    /// Activate the leader key
    pub fn activate(&mut self) {
        self.is_active = true;
        self.activated_at = Some(Instant::now());
    }

    /// Deactivate the leader key
    pub fn deactivate(&mut self) {
        self.is_active = false;
        self.activated_at = None;
        self.sequence_progress.clear();
    }

    /// Reset the leader state (same as deactivate)
    pub fn reset(&mut self) {
        self.deactivate();
    }

    /// Feed a key to the leader state machine
    ///
    /// Returns true if the leader should be activated
    pub fn feed_key(&mut self, combo: &KeyCombo) -> bool {
        // If we have a prefix sequence, handle it
        if !self.prefix_sequence.is_empty() {
            return self.feed_sequence_key(combo);
        }

        // Simple single-key leader
        if combo == &self.key {
            self.activate();
            true
        } else {
            false
        }
    }

    /// Feed a key when using a prefix sequence
    fn feed_sequence_key(&mut self, combo: &KeyCombo) -> bool {
        // Add to progress
        self.sequence_progress.push(combo.clone());

        // Check if we match the sequence so far
        if self.sequence_progress.len() <= self.prefix_sequence.len() {
            let matches = self
                .prefix_sequence
                .iter()
                .take(self.sequence_progress.len())
                .zip(self.sequence_progress.iter())
                .all(|(expected, actual)| expected == actual);

            if !matches {
                // Mismatch - reset
                self.sequence_progress.clear();
                return false;
            }

            // Check if we completed the sequence
            if self.sequence_progress.len() == self.prefix_sequence.len() {
                self.activate();
                self.sequence_progress.clear();
                return true;
            }

            // Still in progress
            false
        } else {
            // Too many keys - reset
            self.sequence_progress.clear();
            false
        }
    }

    /// Check if the leader has timed out
    ///
    /// Returns true if timeout occurred, false otherwise
    pub fn check_timeout(&mut self) -> bool {
        if let Some(activated) = self.activated_at {
            if activated.elapsed().as_millis() as u64 > self.timeout_ms {
                self.deactivate();
                return true;
            }
        }
        false
    }

    /// Get the time remaining before timeout (if active)
    pub fn time_remaining(&self) -> Option<Duration> {
        if let Some(activated) = self.activated_at {
            let elapsed = activated.elapsed();
            let timeout = Duration::from_millis(self.timeout_ms);
            timeout.checked_sub(elapsed)
        } else {
            None
        }
    }

    /// Get when the leader will timeout (if active)
    pub fn timeout_at(&self) -> Option<Instant> {
        self.activated_at
            .map(|t| t + Duration::from_millis(self.timeout_ms))
    }

    /// Check if the sequence is in progress (for multi-key leaders)
    pub fn is_sequence_in_progress(&self) -> bool {
        !self.sequence_progress.is_empty()
    }

    /// Get the current sequence progress
    pub fn sequence_progress(&self) -> &[KeyCombo] {
        &self.sequence_progress
    }

    /// Update the leader key configuration
    pub fn update_config(&mut self, key: KeyCombo, timeout_ms: u64) {
        self.key = key;
        self.timeout_ms = timeout_ms;
        self.deactivate();
    }
}

/// Configuration for leader key
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LeaderKeyConfig {
    /// The key combination
    pub key: KeyCombo,
    /// Timeout in milliseconds
    pub timeout_ms: u64,
    /// Optional multi-key sequence
    pub sequence: Option<Vec<KeyCombo>>,
}

impl Default for LeaderKeyConfig {
    fn default() -> Self {
        Self {
            key: KeyCombo::new(KeyCode::KeyA, KeyModifiers::CTRL),
            timeout_ms: 1000,
            sequence: None,
        }
    }
}

impl From<LeaderKeyConfig> for LeaderKeyState {
    fn from(config: LeaderKeyConfig) -> Self {
        if let Some(sequence) = config.sequence {
            LeaderKeyState::with_sequence(sequence, config.timeout_ms)
        } else {
            LeaderKeyState::new(config.key, config.timeout_ms)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_leader_activation() {
        let mut leader =
            LeaderKeyState::new(KeyCombo::new(KeyCode::KeyA, KeyModifiers::CTRL), 1000);

        assert!(!leader.is_active());

        leader.activate();
        assert!(leader.is_active());

        leader.deactivate();
        assert!(!leader.is_active());
    }

    #[test]
    fn test_leader_timeout() {
        let mut leader = LeaderKeyState::new(
            KeyCombo::new(KeyCode::KeyA, KeyModifiers::CTRL),
            100, // 100ms timeout
        );

        leader.activate();
        assert!(leader.is_active());

        // Should not timeout immediately
        assert!(!leader.check_timeout());
        assert!(leader.is_active());

        // Simulate passage of time by sleeping
        std::thread::sleep(Duration::from_millis(150));

        // Should timeout now
        assert!(leader.check_timeout());
        assert!(!leader.is_active());
    }

    #[test]
    fn test_leader_feed_key() {
        let mut leader =
            LeaderKeyState::new(KeyCombo::new(KeyCode::KeyA, KeyModifiers::CTRL), 1000);

        let correct_key = KeyCombo::new(KeyCode::KeyA, KeyModifiers::CTRL);
        let wrong_key = KeyCombo::new(KeyCode::KeyB, KeyModifiers::CTRL);

        // Feeding the correct key should activate
        assert!(leader.feed_key(&correct_key));
        assert!(leader.is_active());

        leader.deactivate();

        // Feeding wrong key should not activate
        assert!(!leader.feed_key(&wrong_key));
        assert!(!leader.is_active());
    }

    #[test]
    fn test_leader_sequence() {
        let sequence = vec![
            KeyCombo::new(KeyCode::KeyA, KeyModifiers::CTRL),
            KeyCombo::new(KeyCode::KeyA, KeyModifiers::NONE),
        ];

        let mut leader = LeaderKeyState::with_sequence(sequence, 1000);

        // First key in sequence
        let key1 = KeyCombo::new(KeyCode::KeyA, KeyModifiers::CTRL);
        assert!(!leader.feed_key(&key1)); // Not complete yet
        assert!(!leader.is_active());

        // Second key completes sequence
        let key2 = KeyCombo::new(KeyCode::KeyA, KeyModifiers::NONE);
        assert!(leader.feed_key(&key2));
        assert!(leader.is_active());
    }

    #[test]
    fn test_leader_sequence_mismatch() {
        let sequence = vec![
            KeyCombo::new(KeyCode::KeyA, KeyModifiers::CTRL),
            KeyCombo::new(KeyCode::KeyA, KeyModifiers::NONE),
        ];

        let mut leader = LeaderKeyState::with_sequence(sequence, 1000);

        // First key correct
        let key1 = KeyCombo::new(KeyCode::KeyA, KeyModifiers::CTRL);
        assert!(!leader.feed_key(&key1));

        // Second key wrong
        let key2 = KeyCombo::new(KeyCode::KeyB, KeyModifiers::NONE);
        assert!(!leader.feed_key(&key2));
        assert!(!leader.is_active());

        // Should reset and not be in progress
        assert!(!leader.is_sequence_in_progress());
    }

    #[test]
    fn test_time_remaining() {
        let mut leader =
            LeaderKeyState::new(KeyCombo::new(KeyCode::KeyA, KeyModifiers::CTRL), 1000);

        // Not active, no time remaining
        assert!(leader.time_remaining().is_none());

        leader.activate();

        // Should have some time remaining
        let remaining = leader.time_remaining();
        assert!(remaining.is_some());
        assert!(remaining.unwrap().as_millis() > 0);
        assert!(remaining.unwrap().as_millis() <= 1000);
    }

    #[test]
    fn test_reset() {
        let mut leader =
            LeaderKeyState::new(KeyCombo::new(KeyCode::KeyA, KeyModifiers::CTRL), 1000);

        leader.activate();
        assert!(leader.is_active());

        leader.reset();
        assert!(!leader.is_active());
        assert!(leader.activated_at.is_none());
    }

    #[test]
    fn test_update_config() {
        let mut leader =
            LeaderKeyState::new(KeyCombo::new(KeyCode::KeyA, KeyModifiers::CTRL), 1000);

        leader.activate();
        assert!(leader.is_active());

        // Update config should deactivate
        let new_key = KeyCombo::new(KeyCode::KeyB, KeyModifiers::CTRL);
        leader.update_config(new_key.clone(), 2000);

        assert!(!leader.is_active());
        assert_eq!(leader.key, new_key);
        assert_eq!(leader.timeout_ms, 2000);
    }

    #[test]
    fn test_leader_config_default() {
        let config = LeaderKeyConfig::default();
        assert_eq!(config.timeout_ms, 1000);
        assert_eq!(config.key.key, KeyCode::KeyA);
        assert!(config.key.mods.ctrl());
    }

    #[test]
    fn test_leader_from_config() {
        let config = LeaderKeyConfig {
            key: KeyCombo::new(KeyCode::KeyB, KeyModifiers::ALT),
            timeout_ms: 500,
            sequence: None,
        };

        let leader: LeaderKeyState = config.into();
        assert_eq!(leader.key.key, KeyCode::KeyB);
        assert!(leader.key.mods.alt());
        assert_eq!(leader.timeout_ms, 500);
    }
}
