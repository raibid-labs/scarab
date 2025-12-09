//! Integration tests for click detection and handling

use scarab_mouse::click_handler::{generate_cursor_position_sequence, generate_mouse_sequence, ClickDetector};
use scarab_mouse::types::{ClickType, Modifiers, MouseButton, MouseEvent, MouseEventKind, Position};
use std::thread::sleep;
use std::time::Duration;

#[test]
fn test_single_click_detection() {
    let mut detector = ClickDetector::new();

    let event = MouseEvent {
        kind: MouseEventKind::Press,
        position: Position::new(10, 5),
        button: Some(MouseButton::Left),
        modifiers: Modifiers::none(),
    };

    let click_type = detector.handle_press(&event);
    assert_eq!(click_type, ClickType::Single);
}

#[test]
fn test_double_click_detection() {
    let mut detector = ClickDetector::new();

    let event = MouseEvent {
        kind: MouseEventKind::Press,
        position: Position::new(10, 5),
        button: Some(MouseButton::Left),
        modifiers: Modifiers::none(),
    };

    // First click
    let click_type = detector.handle_press(&event);
    assert_eq!(click_type, ClickType::Single);

    // Second click at same position within threshold
    let click_type = detector.handle_press(&event);
    assert_eq!(click_type, ClickType::Double);
}

#[test]
fn test_triple_click_detection() {
    let mut detector = ClickDetector::new();

    let event = MouseEvent {
        kind: MouseEventKind::Press,
        position: Position::new(10, 5),
        button: Some(MouseButton::Left),
        modifiers: Modifiers::none(),
    };

    // Three rapid clicks
    assert_eq!(detector.handle_press(&event), ClickType::Single);
    assert_eq!(detector.handle_press(&event), ClickType::Double);
    assert_eq!(detector.handle_press(&event), ClickType::Triple);
}

#[test]
fn test_click_detection_resets_after_movement() {
    let mut detector = ClickDetector::new();

    let event1 = MouseEvent {
        kind: MouseEventKind::Press,
        position: Position::new(10, 5),
        button: Some(MouseButton::Left),
        modifiers: Modifiers::none(),
    };

    let event2 = MouseEvent {
        kind: MouseEventKind::Press,
        position: Position::new(20, 10), // Far away
        button: Some(MouseButton::Left),
        modifiers: Modifiers::none(),
    };

    assert_eq!(detector.handle_press(&event1), ClickType::Single);
    // Click far away should reset to single
    assert_eq!(detector.handle_press(&event2), ClickType::Single);
}

#[test]
fn test_click_detection_timeout() {
    let mut detector = ClickDetector::new();

    let event = MouseEvent {
        kind: MouseEventKind::Press,
        position: Position::new(10, 5),
        button: Some(MouseButton::Left),
        modifiers: Modifiers::none(),
    };

    assert_eq!(detector.handle_press(&event), ClickType::Single);

    // Wait longer than double-click threshold
    sleep(Duration::from_millis(600));

    // Should be treated as new single click
    assert_eq!(detector.handle_press(&event), ClickType::Single);
}

#[test]
fn test_click_detector_reset() {
    let mut detector = ClickDetector::new();

    let event = MouseEvent {
        kind: MouseEventKind::Press,
        position: Position::new(10, 5),
        button: Some(MouseButton::Left),
        modifiers: Modifiers::none(),
    };

    detector.handle_press(&event);
    detector.reset();

    // After reset, should be single click again
    assert_eq!(detector.handle_press(&event), ClickType::Single);
}

#[test]
fn test_cursor_position_sequence_generation() {
    let seq = generate_cursor_position_sequence(Position::new(0, 0));
    assert_eq!(seq, b"\x1b[1;1H");

    let seq = generate_cursor_position_sequence(Position::new(10, 20));
    assert_eq!(seq, b"\x1b[21;11H"); // +1 for 1-based indexing

    let seq = generate_cursor_position_sequence(Position::new(79, 23));
    assert_eq!(seq, b"\x1b[24;80H");
}

#[test]
fn test_mouse_sequence_left_press() {
    let event = MouseEvent {
        kind: MouseEventKind::Press,
        position: Position::new(5, 10),
        button: Some(MouseButton::Left),
        modifiers: Modifiers::none(),
    };

    let seq = generate_mouse_sequence(&event).unwrap();
    assert_eq!(seq, b"\x1b[<0;6;11M");
}

#[test]
fn test_mouse_sequence_left_release() {
    let event = MouseEvent {
        kind: MouseEventKind::Release,
        position: Position::new(5, 10),
        button: Some(MouseButton::Left),
        modifiers: Modifiers::none(),
    };

    let seq = generate_mouse_sequence(&event).unwrap();
    assert_eq!(seq, b"\x1b[<0;6;11m"); // lowercase 'm' for release
}

#[test]
fn test_mouse_sequence_middle_button() {
    let event = MouseEvent {
        kind: MouseEventKind::Press,
        position: Position::new(0, 0),
        button: Some(MouseButton::Middle),
        modifiers: Modifiers::none(),
    };

    let seq = generate_mouse_sequence(&event).unwrap();
    assert_eq!(seq, b"\x1b[<1;1;1M"); // button code 1 for middle
}

#[test]
fn test_mouse_sequence_right_button() {
    let event = MouseEvent {
        kind: MouseEventKind::Press,
        position: Position::new(0, 0),
        button: Some(MouseButton::Right),
        modifiers: Modifiers::none(),
    };

    let seq = generate_mouse_sequence(&event).unwrap();
    assert_eq!(seq, b"\x1b[<2;1;1M"); // button code 2 for right
}

#[test]
fn test_mouse_sequence_with_shift_modifier() {
    let event = MouseEvent {
        kind: MouseEventKind::Press,
        position: Position::new(0, 0),
        button: Some(MouseButton::Left),
        modifiers: Modifiers {
            shift: true,
            ..Default::default()
        },
    };

    let seq = generate_mouse_sequence(&event).unwrap();
    assert_eq!(seq, b"\x1b[<4;1;1M"); // +4 for shift
}

#[test]
fn test_mouse_sequence_with_alt_modifier() {
    let event = MouseEvent {
        kind: MouseEventKind::Press,
        position: Position::new(0, 0),
        button: Some(MouseButton::Left),
        modifiers: Modifiers {
            alt: true,
            ..Default::default()
        },
    };

    let seq = generate_mouse_sequence(&event).unwrap();
    assert_eq!(seq, b"\x1b[<8;1;1M"); // +8 for alt
}

#[test]
fn test_mouse_sequence_with_ctrl_modifier() {
    let event = MouseEvent {
        kind: MouseEventKind::Press,
        position: Position::new(0, 0),
        button: Some(MouseButton::Left),
        modifiers: Modifiers {
            ctrl: true,
            ..Default::default()
        },
    };

    let seq = generate_mouse_sequence(&event).unwrap();
    assert_eq!(seq, b"\x1b[<16;1;1M"); // +16 for ctrl
}

#[test]
fn test_mouse_sequence_with_multiple_modifiers() {
    let event = MouseEvent {
        kind: MouseEventKind::Press,
        position: Position::new(0, 0),
        button: Some(MouseButton::Left),
        modifiers: Modifiers {
            shift: true,
            ctrl: true,
            alt: true,
            ..Default::default()
        },
    };

    let seq = generate_mouse_sequence(&event).unwrap();
    assert_eq!(seq, b"\x1b[<28;1;1M"); // 4 + 8 + 16 = 28
}

#[test]
fn test_mouse_sequence_scroll_up() {
    let event = MouseEvent {
        kind: MouseEventKind::Scroll,
        position: Position::new(10, 5),
        button: Some(MouseButton::ScrollUp),
        modifiers: Modifiers::none(),
    };

    let seq = generate_mouse_sequence(&event).unwrap();
    assert_eq!(seq, b"\x1b[<64;11;6M");
}

#[test]
fn test_mouse_sequence_scroll_down() {
    let event = MouseEvent {
        kind: MouseEventKind::Scroll,
        position: Position::new(10, 5),
        button: Some(MouseButton::ScrollDown),
        modifiers: Modifiers::none(),
    };

    let seq = generate_mouse_sequence(&event).unwrap();
    assert_eq!(seq, b"\x1b[<65;11;6M");
}

#[test]
fn test_mouse_sequence_no_button() {
    let event = MouseEvent {
        kind: MouseEventKind::Press,
        position: Position::new(0, 0),
        button: None,
        modifiers: Modifiers::none(),
    };

    let seq = generate_mouse_sequence(&event);
    assert!(seq.is_none());
}

#[test]
fn test_multiple_double_clicks() {
    let mut detector = ClickDetector::new();

    let event = MouseEvent {
        kind: MouseEventKind::Press,
        position: Position::new(10, 5),
        button: Some(MouseButton::Left),
        modifiers: Modifiers::none(),
    };

    // First double-click
    assert_eq!(detector.handle_press(&event), ClickType::Single);
    assert_eq!(detector.handle_press(&event), ClickType::Double);

    // Reset
    detector.reset();

    // Second double-click
    assert_eq!(detector.handle_press(&event), ClickType::Single);
    assert_eq!(detector.handle_press(&event), ClickType::Double);
}
