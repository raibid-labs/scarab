//! Integration tests for core mouse types

use scarab_mouse::types::{
    ClickType, Modifiers, MouseButton, MouseEvent, MouseEventKind, MouseMode, Position,
};

#[test]
fn test_position_creation() {
    let pos = Position::new(10, 20);
    assert_eq!(pos.x, 10);
    assert_eq!(pos.y, 20);
}

#[test]
fn test_position_equality() {
    let pos1 = Position::new(5, 10);
    let pos2 = Position::new(5, 10);
    let pos3 = Position::new(5, 11);

    assert_eq!(pos1, pos2);
    assert_ne!(pos1, pos3);
}

#[test]
fn test_position_distance_horizontal() {
    let pos1 = Position::new(0, 0);
    let pos2 = Position::new(3, 0);

    assert_eq!(pos1.distance_to(&pos2), 3.0);
}

#[test]
fn test_position_distance_vertical() {
    let pos1 = Position::new(0, 0);
    let pos2 = Position::new(0, 4);

    assert_eq!(pos1.distance_to(&pos2), 4.0);
}

#[test]
fn test_position_distance_diagonal() {
    let pos1 = Position::new(0, 0);
    let pos2 = Position::new(3, 4);

    assert_eq!(pos1.distance_to(&pos2), 5.0); // 3-4-5 triangle
}

#[test]
fn test_position_distance_same_point() {
    let pos = Position::new(10, 10);
    assert_eq!(pos.distance_to(&pos), 0.0);
}

#[test]
fn test_position_distance_symmetry() {
    let pos1 = Position::new(5, 10);
    let pos2 = Position::new(15, 20);

    assert_eq!(pos1.distance_to(&pos2), pos2.distance_to(&pos1));
}

#[test]
fn test_modifiers_none() {
    let mods = Modifiers::none();
    assert!(!mods.shift);
    assert!(!mods.ctrl);
    assert!(!mods.alt);
    assert!(!mods.meta);
    assert!(mods.is_empty());
}

#[test]
fn test_modifiers_default() {
    let mods = Modifiers::default();
    assert!(mods.is_empty());
}

#[test]
fn test_modifiers_is_empty() {
    let empty = Modifiers {
        shift: false,
        ctrl: false,
        alt: false,
        meta: false,
    };
    assert!(empty.is_empty());

    let with_shift = Modifiers {
        shift: true,
        ..Default::default()
    };
    assert!(!with_shift.is_empty());

    let with_ctrl = Modifiers {
        ctrl: true,
        ..Default::default()
    };
    assert!(!with_ctrl.is_empty());

    let with_alt = Modifiers {
        alt: true,
        ..Default::default()
    };
    assert!(!with_alt.is_empty());

    let with_meta = Modifiers {
        meta: true,
        ..Default::default()
    };
    assert!(!with_meta.is_empty());
}

#[test]
fn test_modifiers_multiple() {
    let mods = Modifiers {
        shift: true,
        ctrl: true,
        alt: false,
        meta: false,
    };

    assert!(mods.shift);
    assert!(mods.ctrl);
    assert!(!mods.alt);
    assert!(!mods.meta);
    assert!(!mods.is_empty());
}

#[test]
fn test_modifiers_equality() {
    let mods1 = Modifiers {
        shift: true,
        ctrl: false,
        alt: false,
        meta: false,
    };

    let mods2 = Modifiers {
        shift: true,
        ctrl: false,
        alt: false,
        meta: false,
    };

    let mods3 = Modifiers {
        shift: false,
        ctrl: true,
        alt: false,
        meta: false,
    };

    assert_eq!(mods1, mods2);
    assert_ne!(mods1, mods3);
}

#[test]
fn test_mouse_button_variants() {
    let _ = MouseButton::Left;
    let _ = MouseButton::Right;
    let _ = MouseButton::Middle;
    let _ = MouseButton::ScrollUp;
    let _ = MouseButton::ScrollDown;
}

#[test]
fn test_mouse_button_equality() {
    assert_eq!(MouseButton::Left, MouseButton::Left);
    assert_ne!(MouseButton::Left, MouseButton::Right);
}

#[test]
fn test_mouse_event_kind_variants() {
    let _ = MouseEventKind::Press;
    let _ = MouseEventKind::Release;
    let _ = MouseEventKind::Move;
    let _ = MouseEventKind::Scroll;
}

#[test]
fn test_mouse_event_creation() {
    let event = MouseEvent {
        kind: MouseEventKind::Press,
        position: Position::new(10, 20),
        button: Some(MouseButton::Left),
        modifiers: Modifiers::none(),
    };

    assert_eq!(event.kind, MouseEventKind::Press);
    assert_eq!(event.position, Position::new(10, 20));
    assert_eq!(event.button, Some(MouseButton::Left));
    assert!(event.modifiers.is_empty());
}

#[test]
fn test_mouse_event_with_modifiers() {
    let event = MouseEvent {
        kind: MouseEventKind::Press,
        position: Position::new(0, 0),
        button: Some(MouseButton::Left),
        modifiers: Modifiers {
            shift: true,
            ctrl: true,
            ..Default::default()
        },
    };

    assert!(event.modifiers.shift);
    assert!(event.modifiers.ctrl);
}

#[test]
fn test_mouse_event_no_button() {
    let event = MouseEvent {
        kind: MouseEventKind::Move,
        position: Position::new(5, 5),
        button: None,
        modifiers: Modifiers::none(),
    };

    assert!(event.button.is_none());
}

#[test]
fn test_click_type_variants() {
    let _ = ClickType::Single;
    let _ = ClickType::Double;
    let _ = ClickType::Triple;
}

#[test]
fn test_click_type_equality() {
    assert_eq!(ClickType::Single, ClickType::Single);
    assert_eq!(ClickType::Double, ClickType::Double);
    assert_eq!(ClickType::Triple, ClickType::Triple);

    assert_ne!(ClickType::Single, ClickType::Double);
    assert_ne!(ClickType::Double, ClickType::Triple);
}

#[test]
fn test_mouse_mode_normal() {
    let mode = MouseMode::Normal;
    assert!(mode.is_normal());
    assert!(!mode.is_application());
}

#[test]
fn test_mouse_mode_application() {
    let mode = MouseMode::Application;
    assert!(mode.is_application());
    assert!(!mode.is_normal());
}

#[test]
fn test_mouse_mode_equality() {
    assert_eq!(MouseMode::Normal, MouseMode::Normal);
    assert_eq!(MouseMode::Application, MouseMode::Application);
    assert_ne!(MouseMode::Normal, MouseMode::Application);
}

#[test]
fn test_mouse_event_equality() {
    let event1 = MouseEvent {
        kind: MouseEventKind::Press,
        position: Position::new(10, 20),
        button: Some(MouseButton::Left),
        modifiers: Modifiers::none(),
    };

    let event2 = MouseEvent {
        kind: MouseEventKind::Press,
        position: Position::new(10, 20),
        button: Some(MouseButton::Left),
        modifiers: Modifiers::none(),
    };

    let event3 = MouseEvent {
        kind: MouseEventKind::Release,
        position: Position::new(10, 20),
        button: Some(MouseButton::Left),
        modifiers: Modifiers::none(),
    };

    assert_eq!(event1, event2);
    assert_ne!(event1, event3);
}

#[test]
fn test_position_copy() {
    let pos1 = Position::new(5, 10);
    let pos2 = pos1;

    assert_eq!(pos1, pos2);
    // Both should be usable (Copy trait)
    assert_eq!(pos1.x, 5);
    assert_eq!(pos2.x, 5);
}

#[test]
fn test_modifiers_copy() {
    let mods1 = Modifiers {
        ctrl: true,
        ..Default::default()
    };
    let mods2 = mods1;

    assert_eq!(mods1, mods2);
    assert!(mods1.ctrl);
    assert!(mods2.ctrl);
}

#[test]
fn test_mouse_button_copy() {
    let btn1 = MouseButton::Left;
    let btn2 = btn1;

    assert_eq!(btn1, btn2);
}

#[test]
fn test_click_type_copy() {
    let ct1 = ClickType::Double;
    let ct2 = ct1;

    assert_eq!(ct1, ct2);
}

#[test]
fn test_mouse_mode_copy() {
    let mode1 = MouseMode::Application;
    let mode2 = mode1;

    assert_eq!(mode1, mode2);
}

#[test]
fn test_position_hash() {
    use std::collections::HashSet;

    let mut set = HashSet::new();
    set.insert(Position::new(1, 2));
    set.insert(Position::new(3, 4));
    set.insert(Position::new(1, 2)); // Duplicate

    assert_eq!(set.len(), 2);
    assert!(set.contains(&Position::new(1, 2)));
    assert!(set.contains(&Position::new(3, 4)));
}

#[test]
fn test_mouse_button_hash() {
    use std::collections::HashSet;

    let mut set = HashSet::new();
    set.insert(MouseButton::Left);
    set.insert(MouseButton::Right);
    set.insert(MouseButton::Left); // Duplicate

    assert_eq!(set.len(), 2);
}

#[test]
fn test_serialization_roundtrip_position() {
    use serde_json;

    let pos = Position::new(42, 84);
    let json = serde_json::to_string(&pos).unwrap();
    let deserialized: Position = serde_json::from_str(&json).unwrap();

    assert_eq!(pos, deserialized);
}

#[test]
fn test_serialization_roundtrip_modifiers() {
    use serde_json;

    let mods = Modifiers {
        shift: true,
        ctrl: true,
        alt: false,
        meta: false,
    };

    let json = serde_json::to_string(&mods).unwrap();
    let deserialized: Modifiers = serde_json::from_str(&json).unwrap();

    assert_eq!(mods, deserialized);
}

#[test]
fn test_serialization_roundtrip_mouse_button() {
    use serde_json;

    let button = MouseButton::Left;
    let json = serde_json::to_string(&button).unwrap();
    let deserialized: MouseButton = serde_json::from_str(&json).unwrap();

    assert_eq!(button, deserialized);
}

#[test]
fn test_serialization_roundtrip_mouse_mode() {
    use serde_json;

    let mode = MouseMode::Application;
    let json = serde_json::to_string(&mode).unwrap();
    let deserialized: MouseMode = serde_json::from_str(&json).unwrap();

    assert_eq!(mode, deserialized);
}

#[test]
fn test_serialization_roundtrip_click_type() {
    use serde_json;

    let click = ClickType::Double;
    let json = serde_json::to_string(&click).unwrap();
    let deserialized: ClickType = serde_json::from_str(&json).unwrap();

    assert_eq!(click, deserialized);
}

#[test]
fn test_debug_formatting() {
    let pos = Position::new(10, 20);
    let debug_str = format!("{:?}", pos);
    assert!(debug_str.contains("10"));
    assert!(debug_str.contains("20"));

    let mods = Modifiers {
        ctrl: true,
        ..Default::default()
    };
    let debug_str = format!("{:?}", mods);
    assert!(debug_str.contains("ctrl"));

    let button = MouseButton::Left;
    let debug_str = format!("{:?}", button);
    assert!(debug_str.contains("Left"));
}

#[test]
fn test_large_position_values() {
    let pos = Position::new(u16::MAX, u16::MAX);
    assert_eq!(pos.x, 65535);
    assert_eq!(pos.y, 65535);
}

#[test]
fn test_position_distance_max_values() {
    let pos1 = Position::new(0, 0);
    let pos2 = Position::new(u16::MAX, u16::MAX);

    let distance = pos1.distance_to(&pos2);
    assert!(distance > 0.0);
    assert!(distance.is_finite());
}
