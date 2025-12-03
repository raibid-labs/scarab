// Input mapping from Bevy events to Ratatui events
//
// This module handles conversion of Bevy keyboard and mouse events to Ratatui's
// crossterm event format, enabling proper interaction with Ratatui widgets.

use bevy::input::keyboard::KeyboardInput;
use bevy::input::mouse::{MouseButton, MouseButtonInput};
use bevy::prelude::*;
use ratatui::crossterm::event::{
    Event as RatEvent, KeyCode as RatKeyCode, KeyEvent, KeyModifiers, MouseButton as RatMouseButton,
    MouseEvent, MouseEventKind,
};
use scarab_protocol::TerminalMetrics;

use super::surface::RatatuiSurface;

/// Resource for managing surface focus
///
/// Maintains a stack of focused surfaces, with the top being the currently focused one.
/// This enables proper event routing for overlays and modal dialogs.
///
/// # Focus Stack Behavior
/// - New surfaces can push themselves onto the focus stack
/// - Surfaces are automatically removed when despawned
/// - Events are routed only to the topmost (currently focused) surface
#[derive(Resource, Default)]
pub struct SurfaceFocus {
    /// Stack of focused surfaces (top is current focus)
    pub focus_stack: Vec<Entity>,
}

impl SurfaceFocus {
    /// Push a surface to focus (becomes the new current focus)
    ///
    /// If the surface is already in the stack, it's moved to the top.
    pub fn push(&mut self, entity: Entity) {
        // Remove if already present to avoid duplicates
        self.focus_stack.retain(|&e| e != entity);
        self.focus_stack.push(entity);
    }

    /// Pop the top focused surface
    ///
    /// Returns the entity that was focused, or None if the stack is empty.
    pub fn pop(&mut self) -> Option<Entity> {
        self.focus_stack.pop()
    }

    /// Get currently focused surface
    ///
    /// Returns None if no surface is focused.
    pub fn current(&self) -> Option<Entity> {
        self.focus_stack.last().copied()
    }

    /// Check if a specific surface is currently focused
    pub fn is_focused(&self, entity: Entity) -> bool {
        self.current() == Some(entity)
    }

    /// Remove a specific surface from the focus stack
    ///
    /// Used when surfaces are despawned or manually unfocused.
    pub fn remove(&mut self, entity: Entity) {
        self.focus_stack.retain(|&e| e != entity);
    }

    /// Clear the entire focus stack
    pub fn clear(&mut self) {
        self.focus_stack.clear();
    }
}

/// Event sent when a Ratatui event is available for a surface
///
/// This event is sent to the currently focused surface and can be consumed
/// by widget-specific systems to handle user interaction.
#[derive(Event)]
pub struct SurfaceInputEvent {
    /// The surface entity that should receive this event
    pub surface: Entity,
    /// The Ratatui event to process
    pub event: RatEvent,
}

/// Convert Bevy KeyCode to Ratatui KeyCode
///
/// Maps Bevy's keyboard representation to Ratatui's crossterm format.
/// Returns None for keys that don't have a direct Ratatui equivalent.
pub fn bevy_to_ratatui_key(key: KeyCode) -> Option<RatKeyCode> {
    Some(match key {
        // Special keys
        KeyCode::Backspace => RatKeyCode::Backspace,
        KeyCode::Enter => RatKeyCode::Enter,
        KeyCode::ArrowLeft => RatKeyCode::Left,
        KeyCode::ArrowRight => RatKeyCode::Right,
        KeyCode::ArrowUp => RatKeyCode::Up,
        KeyCode::ArrowDown => RatKeyCode::Down,
        KeyCode::Home => RatKeyCode::Home,
        KeyCode::End => RatKeyCode::End,
        KeyCode::PageUp => RatKeyCode::PageUp,
        KeyCode::PageDown => RatKeyCode::PageDown,
        KeyCode::Tab => RatKeyCode::Tab,
        KeyCode::Delete => RatKeyCode::Delete,
        KeyCode::Insert => RatKeyCode::Insert,
        KeyCode::Escape => RatKeyCode::Esc,

        // Function keys
        KeyCode::F1 => RatKeyCode::F(1),
        KeyCode::F2 => RatKeyCode::F(2),
        KeyCode::F3 => RatKeyCode::F(3),
        KeyCode::F4 => RatKeyCode::F(4),
        KeyCode::F5 => RatKeyCode::F(5),
        KeyCode::F6 => RatKeyCode::F(6),
        KeyCode::F7 => RatKeyCode::F(7),
        KeyCode::F8 => RatKeyCode::F(8),
        KeyCode::F9 => RatKeyCode::F(9),
        KeyCode::F10 => RatKeyCode::F(10),
        KeyCode::F11 => RatKeyCode::F(11),
        KeyCode::F12 => RatKeyCode::F(12),

        // Space
        KeyCode::Space => RatKeyCode::Char(' '),

        // Letter keys (lowercase by default, shift handled by modifiers)
        KeyCode::KeyA => RatKeyCode::Char('a'),
        KeyCode::KeyB => RatKeyCode::Char('b'),
        KeyCode::KeyC => RatKeyCode::Char('c'),
        KeyCode::KeyD => RatKeyCode::Char('d'),
        KeyCode::KeyE => RatKeyCode::Char('e'),
        KeyCode::KeyF => RatKeyCode::Char('f'),
        KeyCode::KeyG => RatKeyCode::Char('g'),
        KeyCode::KeyH => RatKeyCode::Char('h'),
        KeyCode::KeyI => RatKeyCode::Char('i'),
        KeyCode::KeyJ => RatKeyCode::Char('j'),
        KeyCode::KeyK => RatKeyCode::Char('k'),
        KeyCode::KeyL => RatKeyCode::Char('l'),
        KeyCode::KeyM => RatKeyCode::Char('m'),
        KeyCode::KeyN => RatKeyCode::Char('n'),
        KeyCode::KeyO => RatKeyCode::Char('o'),
        KeyCode::KeyP => RatKeyCode::Char('p'),
        KeyCode::KeyQ => RatKeyCode::Char('q'),
        KeyCode::KeyR => RatKeyCode::Char('r'),
        KeyCode::KeyS => RatKeyCode::Char('s'),
        KeyCode::KeyT => RatKeyCode::Char('t'),
        KeyCode::KeyU => RatKeyCode::Char('u'),
        KeyCode::KeyV => RatKeyCode::Char('v'),
        KeyCode::KeyW => RatKeyCode::Char('w'),
        KeyCode::KeyX => RatKeyCode::Char('x'),
        KeyCode::KeyY => RatKeyCode::Char('y'),
        KeyCode::KeyZ => RatKeyCode::Char('z'),

        // Number keys
        KeyCode::Digit0 => RatKeyCode::Char('0'),
        KeyCode::Digit1 => RatKeyCode::Char('1'),
        KeyCode::Digit2 => RatKeyCode::Char('2'),
        KeyCode::Digit3 => RatKeyCode::Char('3'),
        KeyCode::Digit4 => RatKeyCode::Char('4'),
        KeyCode::Digit5 => RatKeyCode::Char('5'),
        KeyCode::Digit6 => RatKeyCode::Char('6'),
        KeyCode::Digit7 => RatKeyCode::Char('7'),
        KeyCode::Digit8 => RatKeyCode::Char('8'),
        KeyCode::Digit9 => RatKeyCode::Char('9'),

        // Unmapped keys
        _ => return None,
    })
}

/// Get current key modifiers from ButtonInput state
///
/// Checks for Shift, Control, and Alt keys and returns the corresponding
/// Ratatui KeyModifiers flags.
pub fn get_modifiers(keys: &ButtonInput<KeyCode>) -> KeyModifiers {
    let mut mods = KeyModifiers::NONE;

    if keys.pressed(KeyCode::ShiftLeft) || keys.pressed(KeyCode::ShiftRight) {
        mods |= KeyModifiers::SHIFT;
    }
    if keys.pressed(KeyCode::ControlLeft) || keys.pressed(KeyCode::ControlRight) {
        mods |= KeyModifiers::CONTROL;
    }
    if keys.pressed(KeyCode::AltLeft) || keys.pressed(KeyCode::AltRight) {
        mods |= KeyModifiers::ALT;
    }

    mods
}

/// System to handle keyboard input for the focused surface
///
/// This system:
/// 1. Reads keyboard input events from Bevy
/// 2. Converts them to Ratatui key events
/// 3. Sends them to the currently focused surface (if any)
///
/// Only visible surfaces receive input events.
pub fn handle_keyboard_input(
    mut events: EventReader<KeyboardInput>,
    keys: Res<ButtonInput<KeyCode>>,
    focus: Res<SurfaceFocus>,
    surfaces: Query<&RatatuiSurface>,
    mut surface_events: EventWriter<SurfaceInputEvent>,
) {
    // Get the currently focused surface
    let Some(focused) = focus.current() else {
        return;
    };

    // Verify surface exists and is visible
    let Ok(surface) = surfaces.get(focused) else {
        return;
    };
    if !surface.visible {
        return;
    }

    // Process keyboard events
    for event in events.read() {
        // Only process key press events (not releases)
        if !event.state.is_pressed() {
            continue;
        }

        // Convert Bevy key to Ratatui key
        if let Some(rat_key) = bevy_to_ratatui_key(event.key_code) {
            let modifiers = get_modifiers(&keys);

            // Apply shift modifier to uppercase letters
            let rat_key = if let RatKeyCode::Char(c) = rat_key {
                if modifiers.contains(KeyModifiers::SHIFT) && c.is_ascii_lowercase() {
                    RatKeyCode::Char(c.to_ascii_uppercase())
                } else {
                    RatKeyCode::Char(c)
                }
            } else {
                rat_key
            };

            let key_event = KeyEvent::new(rat_key, modifiers);

            surface_events.send(SurfaceInputEvent {
                surface: focused,
                event: RatEvent::Key(key_event),
            });
        }
    }
}

/// System to handle mouse input
///
/// This system:
/// 1. Reads mouse button events
/// 2. Determines which surface (if any) is under the cursor
/// 3. Updates focus if a surface is clicked
/// 4. Sends mouse events to the surface in surface-local coordinates
///
/// Surfaces with higher z-index take priority for mouse events.
pub fn handle_mouse_input(
    mut mouse_button: EventReader<MouseButtonInput>,
    windows: Query<&Window>,
    metrics: Res<TerminalMetrics>,
    mut focus: ResMut<SurfaceFocus>,
    surfaces: Query<(Entity, &RatatuiSurface)>,
    mut surface_events: EventWriter<SurfaceInputEvent>,
) {
    let Ok(window) = windows.get_single() else {
        return;
    };
    let Some(cursor_pos) = window.cursor_position() else {
        return;
    };

    // Convert screen coordinates to grid coordinates
    let (col, row) = metrics.screen_to_grid(cursor_pos.x, cursor_pos.y);

    for event in mouse_button.read() {
        // Find the topmost surface under the cursor
        let mut target: Option<(Entity, f32)> = None;
        for (entity, surface) in surfaces.iter() {
            if !surface.visible {
                continue;
            }
            if surface.contains_point(col, row) {
                // Select surface with highest z-index
                if target.is_none() || surface.z_index > target.unwrap().1 {
                    target = Some((entity, surface.z_index));
                }
            }
        }

        if let Some((entity, _)) = target {
            // Update focus on mouse down
            if event.state.is_pressed() {
                focus.push(entity);
            }

            // Get surface to calculate local coordinates
            let (_, surface) = surfaces.get(entity).unwrap();
            let local_col = col.saturating_sub(surface.x);
            let local_row = row.saturating_sub(surface.y);

            // Convert Bevy mouse button to Ratatui
            let button = match event.button {
                MouseButton::Left => RatMouseButton::Left,
                MouseButton::Right => RatMouseButton::Right,
                MouseButton::Middle => RatMouseButton::Middle,
                _ => continue,
            };

            // Convert press/release to mouse event kind
            let kind = if event.state.is_pressed() {
                MouseEventKind::Down(button)
            } else {
                MouseEventKind::Up(button)
            };

            let mouse_event = MouseEvent {
                kind,
                column: local_col,
                row: local_row,
                modifiers: KeyModifiers::NONE, // TODO: Track mouse modifiers
            };

            surface_events.send(SurfaceInputEvent {
                surface: entity,
                event: RatEvent::Mouse(mouse_event),
            });
        }
    }
}

/// System to clean up focus when surfaces are removed
///
/// This system automatically removes despawned surfaces from the focus stack,
/// preventing focus from being held by non-existent entities.
pub fn cleanup_focus(
    mut focus: ResMut<SurfaceFocus>,
    mut removed: RemovedComponents<RatatuiSurface>,
) {
    for entity in removed.read() {
        focus.remove(entity);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_mapping_special_keys() {
        assert_eq!(
            bevy_to_ratatui_key(KeyCode::Enter),
            Some(RatKeyCode::Enter)
        );
        assert_eq!(
            bevy_to_ratatui_key(KeyCode::Escape),
            Some(RatKeyCode::Esc)
        );
        assert_eq!(
            bevy_to_ratatui_key(KeyCode::Backspace),
            Some(RatKeyCode::Backspace)
        );
        assert_eq!(bevy_to_ratatui_key(KeyCode::Tab), Some(RatKeyCode::Tab));
    }

    #[test]
    fn test_key_mapping_arrows() {
        assert_eq!(
            bevy_to_ratatui_key(KeyCode::ArrowUp),
            Some(RatKeyCode::Up)
        );
        assert_eq!(
            bevy_to_ratatui_key(KeyCode::ArrowDown),
            Some(RatKeyCode::Down)
        );
        assert_eq!(
            bevy_to_ratatui_key(KeyCode::ArrowLeft),
            Some(RatKeyCode::Left)
        );
        assert_eq!(
            bevy_to_ratatui_key(KeyCode::ArrowRight),
            Some(RatKeyCode::Right)
        );
    }

    #[test]
    fn test_key_mapping_function_keys() {
        assert_eq!(bevy_to_ratatui_key(KeyCode::F1), Some(RatKeyCode::F(1)));
        assert_eq!(bevy_to_ratatui_key(KeyCode::F5), Some(RatKeyCode::F(5)));
        assert_eq!(bevy_to_ratatui_key(KeyCode::F12), Some(RatKeyCode::F(12)));
    }

    #[test]
    fn test_key_mapping_letters() {
        assert_eq!(
            bevy_to_ratatui_key(KeyCode::KeyA),
            Some(RatKeyCode::Char('a'))
        );
        assert_eq!(
            bevy_to_ratatui_key(KeyCode::KeyZ),
            Some(RatKeyCode::Char('z'))
        );
    }

    #[test]
    fn test_key_mapping_numbers() {
        assert_eq!(
            bevy_to_ratatui_key(KeyCode::Digit0),
            Some(RatKeyCode::Char('0'))
        );
        assert_eq!(
            bevy_to_ratatui_key(KeyCode::Digit9),
            Some(RatKeyCode::Char('9'))
        );
    }

    #[test]
    fn test_key_mapping_unmapped() {
        // Some keys don't have direct Ratatui equivalents
        assert_eq!(bevy_to_ratatui_key(KeyCode::NumpadAdd), None);
    }

    #[test]
    fn test_modifier_detection() {
        let mut keys = ButtonInput::<KeyCode>::default();

        // No modifiers
        assert_eq!(get_modifiers(&keys), KeyModifiers::NONE);

        // Shift
        keys.press(KeyCode::ShiftLeft);
        assert!(get_modifiers(&keys).contains(KeyModifiers::SHIFT));
        keys.clear();

        // Control
        keys.press(KeyCode::ControlRight);
        assert!(get_modifiers(&keys).contains(KeyModifiers::CONTROL));
        keys.clear();

        // Alt
        keys.press(KeyCode::AltLeft);
        assert!(get_modifiers(&keys).contains(KeyModifiers::ALT));
        keys.clear();

        // Multiple modifiers
        keys.press(KeyCode::ControlLeft);
        keys.press(KeyCode::ShiftRight);
        let mods = get_modifiers(&keys);
        assert!(mods.contains(KeyModifiers::CONTROL));
        assert!(mods.contains(KeyModifiers::SHIFT));
    }

    #[test]
    fn test_focus_stack_operations() {
        let mut focus = SurfaceFocus::default();
        let e1 = Entity::from_raw(1);
        let e2 = Entity::from_raw(2);
        let e3 = Entity::from_raw(3);

        // Empty stack
        assert_eq!(focus.current(), None);

        // Push operations
        focus.push(e1);
        assert_eq!(focus.current(), Some(e1));

        focus.push(e2);
        assert_eq!(focus.current(), Some(e2));

        focus.push(e3);
        assert_eq!(focus.current(), Some(e3));

        // Pop operations
        assert_eq!(focus.pop(), Some(e3));
        assert_eq!(focus.current(), Some(e2));

        assert_eq!(focus.pop(), Some(e2));
        assert_eq!(focus.current(), Some(e1));

        assert_eq!(focus.pop(), Some(e1));
        assert_eq!(focus.current(), None);

        assert_eq!(focus.pop(), None);
    }

    #[test]
    fn test_focus_stack_remove() {
        let mut focus = SurfaceFocus::default();
        let e1 = Entity::from_raw(1);
        let e2 = Entity::from_raw(2);
        let e3 = Entity::from_raw(3);

        focus.push(e1);
        focus.push(e2);
        focus.push(e3);

        // Remove middle element
        focus.remove(e2);
        assert_eq!(focus.focus_stack, vec![e1, e3]);

        // Remove current focus
        focus.remove(e3);
        assert_eq!(focus.current(), Some(e1));
    }

    #[test]
    fn test_focus_stack_duplicate_push() {
        let mut focus = SurfaceFocus::default();
        let e1 = Entity::from_raw(1);
        let e2 = Entity::from_raw(2);

        focus.push(e1);
        focus.push(e2);
        focus.push(e1); // Push e1 again

        // e1 should be moved to top, not duplicated
        assert_eq!(focus.focus_stack, vec![e2, e1]);
        assert_eq!(focus.current(), Some(e1));
    }

    #[test]
    fn test_focus_is_focused() {
        let mut focus = SurfaceFocus::default();
        let e1 = Entity::from_raw(1);
        let e2 = Entity::from_raw(2);

        focus.push(e1);
        assert!(focus.is_focused(e1));
        assert!(!focus.is_focused(e2));

        focus.push(e2);
        assert!(!focus.is_focused(e1));
        assert!(focus.is_focused(e2));
    }

    #[test]
    fn test_focus_clear() {
        let mut focus = SurfaceFocus::default();
        focus.push(Entity::from_raw(1));
        focus.push(Entity::from_raw(2));
        focus.push(Entity::from_raw(3));

        focus.clear();
        assert_eq!(focus.current(), None);
        assert!(focus.focus_stack.is_empty());
    }
}
