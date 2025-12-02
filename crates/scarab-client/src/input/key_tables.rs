//! Bevy integration for Key Tables
//!
//! This module provides Bevy resources and systems for managing key table stacks
//! and leader key state in the Scarab client.

use bevy::prelude::*;
use scarab_plugin_api::key_tables::{KeyTableStack, LeaderKeyState, KeyCombo, KeyCode as ApiKeyCode, KeyModifiers as ApiKeyModifiers};
use bevy::input::keyboard::KeyCode as BevyKeyCode;

/// Bevy resource wrapping KeyTableStack
#[derive(Resource, Debug)]
pub struct KeyTableStackResource {
    /// The underlying key table stack
    stack: KeyTableStack,
}

impl KeyTableStackResource {
    /// Create a new key table stack resource
    pub fn new(stack: KeyTableStack) -> Self {
        Self { stack }
    }

    /// Get a reference to the underlying stack
    pub fn stack(&self) -> &KeyTableStack {
        &self.stack
    }

    /// Get a mutable reference to the underlying stack
    pub fn stack_mut(&mut self) -> &mut KeyTableStack {
        &mut self.stack
    }
}

impl Default for KeyTableStackResource {
    fn default() -> Self {
        Self {
            stack: KeyTableStack::default(),
        }
    }
}

/// Bevy resource wrapping LeaderKeyState
#[derive(Resource, Debug)]
pub struct LeaderKeyResource {
    /// The underlying leader key state
    state: LeaderKeyState,
}

impl LeaderKeyResource {
    /// Create a new leader key resource
    pub fn new(state: LeaderKeyState) -> Self {
        Self { state }
    }

    /// Get a reference to the underlying state
    pub fn state(&self) -> &LeaderKeyState {
        &self.state
    }

    /// Get a mutable reference to the underlying state
    pub fn state_mut(&mut self) -> &mut LeaderKeyState {
        &mut self.state
    }

    /// Check if the leader key is currently active
    pub fn is_active(&self) -> bool {
        self.state.is_active()
    }
}

impl Default for LeaderKeyResource {
    fn default() -> Self {
        // Default: Ctrl+A with 1000ms timeout
        Self {
            state: LeaderKeyState::new(
                KeyCombo::new(ApiKeyCode::KeyA, ApiKeyModifiers::CTRL),
                1000,
            ),
        }
    }
}

/// Convert Bevy KeyCode to API KeyCode
pub fn bevy_to_api_keycode(bevy_key: BevyKeyCode) -> Option<ApiKeyCode> {
    match bevy_key {
        // Letters
        BevyKeyCode::KeyA => Some(ApiKeyCode::KeyA),
        BevyKeyCode::KeyB => Some(ApiKeyCode::KeyB),
        BevyKeyCode::KeyC => Some(ApiKeyCode::KeyC),
        BevyKeyCode::KeyD => Some(ApiKeyCode::KeyD),
        BevyKeyCode::KeyE => Some(ApiKeyCode::KeyE),
        BevyKeyCode::KeyF => Some(ApiKeyCode::KeyF),
        BevyKeyCode::KeyG => Some(ApiKeyCode::KeyG),
        BevyKeyCode::KeyH => Some(ApiKeyCode::KeyH),
        BevyKeyCode::KeyI => Some(ApiKeyCode::KeyI),
        BevyKeyCode::KeyJ => Some(ApiKeyCode::KeyJ),
        BevyKeyCode::KeyK => Some(ApiKeyCode::KeyK),
        BevyKeyCode::KeyL => Some(ApiKeyCode::KeyL),
        BevyKeyCode::KeyM => Some(ApiKeyCode::KeyM),
        BevyKeyCode::KeyN => Some(ApiKeyCode::KeyN),
        BevyKeyCode::KeyO => Some(ApiKeyCode::KeyO),
        BevyKeyCode::KeyP => Some(ApiKeyCode::KeyP),
        BevyKeyCode::KeyQ => Some(ApiKeyCode::KeyQ),
        BevyKeyCode::KeyR => Some(ApiKeyCode::KeyR),
        BevyKeyCode::KeyS => Some(ApiKeyCode::KeyS),
        BevyKeyCode::KeyT => Some(ApiKeyCode::KeyT),
        BevyKeyCode::KeyU => Some(ApiKeyCode::KeyU),
        BevyKeyCode::KeyV => Some(ApiKeyCode::KeyV),
        BevyKeyCode::KeyW => Some(ApiKeyCode::KeyW),
        BevyKeyCode::KeyX => Some(ApiKeyCode::KeyX),
        BevyKeyCode::KeyY => Some(ApiKeyCode::KeyY),
        BevyKeyCode::KeyZ => Some(ApiKeyCode::KeyZ),

        // Numbers
        BevyKeyCode::Digit0 => Some(ApiKeyCode::Digit0),
        BevyKeyCode::Digit1 => Some(ApiKeyCode::Digit1),
        BevyKeyCode::Digit2 => Some(ApiKeyCode::Digit2),
        BevyKeyCode::Digit3 => Some(ApiKeyCode::Digit3),
        BevyKeyCode::Digit4 => Some(ApiKeyCode::Digit4),
        BevyKeyCode::Digit5 => Some(ApiKeyCode::Digit5),
        BevyKeyCode::Digit6 => Some(ApiKeyCode::Digit6),
        BevyKeyCode::Digit7 => Some(ApiKeyCode::Digit7),
        BevyKeyCode::Digit8 => Some(ApiKeyCode::Digit8),
        BevyKeyCode::Digit9 => Some(ApiKeyCode::Digit9),

        // Function keys
        BevyKeyCode::F1 => Some(ApiKeyCode::F1),
        BevyKeyCode::F2 => Some(ApiKeyCode::F2),
        BevyKeyCode::F3 => Some(ApiKeyCode::F3),
        BevyKeyCode::F4 => Some(ApiKeyCode::F4),
        BevyKeyCode::F5 => Some(ApiKeyCode::F5),
        BevyKeyCode::F6 => Some(ApiKeyCode::F6),
        BevyKeyCode::F7 => Some(ApiKeyCode::F7),
        BevyKeyCode::F8 => Some(ApiKeyCode::F8),
        BevyKeyCode::F9 => Some(ApiKeyCode::F9),
        BevyKeyCode::F10 => Some(ApiKeyCode::F10),
        BevyKeyCode::F11 => Some(ApiKeyCode::F11),
        BevyKeyCode::F12 => Some(ApiKeyCode::F12),

        // Special keys
        BevyKeyCode::Escape => Some(ApiKeyCode::Escape),
        BevyKeyCode::Enter => Some(ApiKeyCode::Enter),
        BevyKeyCode::Tab => Some(ApiKeyCode::Tab),
        BevyKeyCode::Backspace => Some(ApiKeyCode::Backspace),
        BevyKeyCode::Space => Some(ApiKeyCode::Space),

        // Arrow keys
        BevyKeyCode::ArrowLeft => Some(ApiKeyCode::Left),
        BevyKeyCode::ArrowRight => Some(ApiKeyCode::Right),
        BevyKeyCode::ArrowUp => Some(ApiKeyCode::Up),
        BevyKeyCode::ArrowDown => Some(ApiKeyCode::Down),

        // Navigation
        BevyKeyCode::Home => Some(ApiKeyCode::Home),
        BevyKeyCode::End => Some(ApiKeyCode::End),
        BevyKeyCode::PageUp => Some(ApiKeyCode::PageUp),
        BevyKeyCode::PageDown => Some(ApiKeyCode::PageDown),
        BevyKeyCode::Insert => Some(ApiKeyCode::Insert),
        BevyKeyCode::Delete => Some(ApiKeyCode::Delete),

        // Modifiers
        BevyKeyCode::ControlLeft => Some(ApiKeyCode::ControlLeft),
        BevyKeyCode::ControlRight => Some(ApiKeyCode::ControlRight),
        BevyKeyCode::AltLeft => Some(ApiKeyCode::AltLeft),
        BevyKeyCode::AltRight => Some(ApiKeyCode::AltRight),
        BevyKeyCode::ShiftLeft => Some(ApiKeyCode::ShiftLeft),
        BevyKeyCode::ShiftRight => Some(ApiKeyCode::ShiftRight),
        BevyKeyCode::SuperLeft => Some(ApiKeyCode::SuperLeft),
        BevyKeyCode::SuperRight => Some(ApiKeyCode::SuperRight),

        _ => None,
    }
}

/// Build modifiers from current keyboard state
pub fn build_modifiers(keyboard: &ButtonInput<BevyKeyCode>, leader_active: bool) -> ApiKeyModifiers {
    let mut mods = ApiKeyModifiers::NONE;

    if keyboard.any_pressed([BevyKeyCode::ControlLeft, BevyKeyCode::ControlRight]) {
        mods |= ApiKeyModifiers::CTRL;
    }
    if keyboard.any_pressed([BevyKeyCode::AltLeft, BevyKeyCode::AltRight]) {
        mods |= ApiKeyModifiers::ALT;
    }
    if keyboard.any_pressed([BevyKeyCode::ShiftLeft, BevyKeyCode::ShiftRight]) {
        mods |= ApiKeyModifiers::SHIFT;
    }
    if keyboard.any_pressed([BevyKeyCode::SuperLeft, BevyKeyCode::SuperRight]) {
        mods |= ApiKeyModifiers::SUPER;
    }
    if leader_active {
        mods |= ApiKeyModifiers::LEADER;
    }

    mods
}

/// System to check leader key timeout
pub fn check_leader_timeout_system(mut leader: ResMut<LeaderKeyResource>) {
    leader.state_mut().check_timeout();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keycode_conversion() {
        assert_eq!(
            bevy_to_api_keycode(BevyKeyCode::KeyA),
            Some(ApiKeyCode::KeyA)
        );
        assert_eq!(
            bevy_to_api_keycode(BevyKeyCode::Escape),
            Some(ApiKeyCode::Escape)
        );
        assert_eq!(
            bevy_to_api_keycode(BevyKeyCode::F1),
            Some(ApiKeyCode::F1)
        );
    }

    #[test]
    fn test_resource_default() {
        let stack_resource = KeyTableStackResource::default();
        assert!(stack_resource.stack().is_empty());

        let leader_resource = LeaderKeyResource::default();
        assert!(!leader_resource.is_active());
    }

    #[test]
    fn test_leader_resource_activation() {
        let mut leader = LeaderKeyResource::default();
        assert!(!leader.is_active());

        leader.state_mut().activate();
        assert!(leader.is_active());

        leader.state_mut().deactivate();
        assert!(!leader.is_active());
    }
}
