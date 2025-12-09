//! Comprehensive integration tests for mouse functionality
//!
//! Tests the interaction between multiple modules and real-world scenarios

use scarab_mouse::click_handler::{generate_cursor_position_sequence, generate_mouse_sequence, ClickDetector};
use scarab_mouse::context_menu::ContextMenu;
use scarab_mouse::mode::{AppHeuristics, ModeDetector};
use scarab_mouse::selection::{find_word_at, Selection, SelectionKind};
use scarab_mouse::types::{ClickType, Modifiers, MouseButton, MouseEvent, MouseEventKind, MouseMode, Position};

// Mock terminal grid for testing
struct MockTerminalGrid {
    data: Vec<Vec<char>>,
}

impl MockTerminalGrid {
    fn new(lines: Vec<&str>) -> Self {
        let data = lines
            .iter()
            .map(|line| line.chars().collect())
            .collect();
        Self { data }
    }

    fn char_at(&self, pos: Position) -> Option<char> {
        self.data
            .get(pos.y as usize)
            .and_then(|row| row.get(pos.x as usize))
            .copied()
    }

    fn dimensions(&self) -> (u16, u16) {
        let rows = self.data.len() as u16;
        let cols = self.data.first().map_or(0, |row| row.len() as u16);
        (cols, rows)
    }
}

#[test]
fn test_complete_click_to_select_workflow() {
    let mut detector = ClickDetector::new();
    let grid = MockTerminalGrid::new(vec!["Hello World", "This is a test"]);

    // First click on "World"
    let event1 = MouseEvent {
        kind: MouseEventKind::Press,
        position: Position::new(7, 0),
        button: Some(MouseButton::Left),
        modifiers: Modifiers::none(),
    };

    let click_type = detector.handle_press(&event1);
    assert_eq!(click_type, ClickType::Single);

    // Double-click to select word
    let click_type = detector.handle_press(&event1);
    assert_eq!(click_type, ClickType::Double);

    // Find word at position
    let word_bounds = find_word_at(event1.position, |pos| grid.char_at(pos));
    assert!(word_bounds.is_some());

    let (start, end) = word_bounds.unwrap();
    let selection = Selection::word(start, end);

    // Extract text
    let text = selection.get_text(|pos| grid.char_at(pos), grid.dimensions().0);
    assert_eq!(text, "World");
}

#[test]
fn test_drag_selection_workflow() {
    let grid = MockTerminalGrid::new(vec!["Hello World"]);

    // Start selection
    let start_pos = Position::new(0, 0);
    let mut selection = Selection::character(start_pos, start_pos);

    // Simulate drag to end position
    let end_pos = Position::new(4, 0);
    selection.update_end(end_pos);

    // Check selection contains expected positions
    assert!(selection.contains(Position::new(0, 0)));
    assert!(selection.contains(Position::new(2, 0)));
    assert!(selection.contains(Position::new(4, 0)));
    assert!(!selection.contains(Position::new(5, 0)));

    // Extract selected text
    let text = selection.get_text(|pos| grid.char_at(pos), grid.dimensions().0);
    assert_eq!(text, "Hello");
}

#[test]
fn test_context_menu_with_selection() {
    let grid = MockTerminalGrid::new(vec!["Hello World"]);

    // Create selection
    let selection = Selection::character(Position::new(0, 0), Position::new(4, 0));
    let has_selection = true;

    // Right-click to open context menu
    let click_pos = Position::new(3, 0);
    let menu = ContextMenu::standard(click_pos, has_selection);

    // Verify copy is enabled
    let copy_item = menu.get_item("copy").unwrap();
    assert!(copy_item.enabled);

    // Extract text from selection
    let text = selection.get_text(|pos| grid.char_at(pos), grid.dimensions().0);
    assert_eq!(text, "Hello");
}

#[test]
fn test_mouse_mode_change_with_vim() {
    let mut detector = ModeDetector::new();

    // Check that vim is detected as mouse app
    assert!(AppHeuristics::detect_mouse_app("vim"));

    // Simulate vim enabling mouse mode
    let result = detector.scan_sequence(b"\x1b[?1000h");
    assert_eq!(result, Some(MouseMode::Application));

    // In application mode, mouse events should be sent to terminal
    let event = MouseEvent {
        kind: MouseEventKind::Press,
        position: Position::new(10, 5),
        button: Some(MouseButton::Left),
        modifiers: Modifiers::none(),
    };

    let sequence = generate_mouse_sequence(&event).unwrap();
    assert!(!sequence.is_empty());

    // Simulate vim exiting and disabling mouse mode
    detector.scan_sequence(b"\x1b[?1000l");
    assert_eq!(detector.mode(), MouseMode::Normal);
}

#[test]
fn test_ctrl_click_on_url() {
    let _grid = MockTerminalGrid::new(vec!["Visit https://example.com for info"]);

    // Ctrl+Click event
    let event = MouseEvent {
        kind: MouseEventKind::Press,
        position: Position::new(10, 0), // On the URL
        button: Some(MouseButton::Left),
        modifiers: Modifiers {
            ctrl: true,
            ..Default::default()
        },
    };

    // Generate mouse sequence with modifiers
    let sequence = generate_mouse_sequence(&event).unwrap();
    assert!(sequence.len() > 0);

    // Create URL context menu
    let menu = ContextMenu::url_menu(event.position, "https://example.com".to_string());

    assert!(menu.get_item("open_url").is_some());
    assert!(menu.get_item("copy_url").is_some());
}

#[test]
fn test_triple_click_line_selection() {
    let mut detector = ClickDetector::new();
    let grid = MockTerminalGrid::new(vec!["Hello World", "Second line", "Third line"]);

    let event = MouseEvent {
        kind: MouseEventKind::Press,
        position: Position::new(5, 0),
        button: Some(MouseButton::Left),
        modifiers: Modifiers::none(),
    };

    // Triple-click
    detector.handle_press(&event);
    detector.handle_press(&event);
    let click_type = detector.handle_press(&event);
    assert_eq!(click_type, ClickType::Triple);

    // Select entire line
    let (cols, _) = grid.dimensions();
    let selection = Selection::line(0, 0, cols);

    let text = selection.get_text(|pos| grid.char_at(pos), cols);
    assert_eq!(text, "Hello World");
}

#[test]
fn test_scroll_wheel_in_application_mode() {
    let mut detector = ModeDetector::new();

    // Enable application mode (like less or vim)
    detector.scan_sequence(b"\x1b[?1000h");
    assert_eq!(detector.mode(), MouseMode::Application);

    // Scroll up event
    let scroll_up = MouseEvent {
        kind: MouseEventKind::Scroll,
        position: Position::new(10, 5),
        button: Some(MouseButton::ScrollUp),
        modifiers: Modifiers::none(),
    };

    let sequence = generate_mouse_sequence(&scroll_up).unwrap();
    assert!(sequence.len() > 0);
    assert!(sequence.contains(&b'M')); // SGR format

    // Scroll down event
    let scroll_down = MouseEvent {
        kind: MouseEventKind::Scroll,
        position: Position::new(10, 5),
        button: Some(MouseButton::ScrollDown),
        modifiers: Modifiers::none(),
    };

    let sequence = generate_mouse_sequence(&scroll_down).unwrap();
    assert!(sequence.len() > 0);
}

#[test]
fn test_multi_line_selection() {
    let grid = MockTerminalGrid::new(vec![
        "First line here",
        "Second line here",
        "Third line here",
    ]);

    let (cols, _) = grid.dimensions();

    // Select from middle of first line to middle of third line
    let selection = Selection::character(Position::new(6, 0), Position::new(6, 2));

    let text = selection.get_text(|pos| grid.char_at(pos), cols);

    // Should include parts of all three lines
    // From position 6 on first line ("line here")
    // Through entire second line
    // To position 6 on third line ("Third ")
    assert!(text.contains("line here"));
    assert!(text.contains("\n")); // Should span multiple lines
    assert!(text.contains("Third"));
}

#[test]
fn test_block_selection_scenario() {
    let grid = MockTerminalGrid::new(vec![
        "Column1 Column2 Column3",
        "Data1   Data2   Data3",
        "Value1  Value2  Value3",
    ]);

    // Select a rectangular block (middle column)
    let selection = Selection::new(
        Position::new(8, 0),
        Position::new(14, 2),
        SelectionKind::Block,
    );

    let (cols, _) = grid.dimensions();
    let text = selection.get_text(|pos| grid.char_at(pos), cols);

    let lines: Vec<&str> = text.lines().collect();
    assert_eq!(lines.len(), 3);

    // Each line should have same width (rectangular selection)
    for line in lines {
        assert_eq!(line.len(), 7); // 8 to 14 inclusive
    }
}

#[test]
fn test_word_expansion_on_double_click() {
    let grid = MockTerminalGrid::new(vec!["foo-bar_baz test"]);

    // Click in middle of compound word
    let mut selection = Selection::character(Position::new(5, 0), Position::new(5, 0));

    selection.expand_to_word(|pos| grid.char_at(pos));

    // Should expand to entire compound word
    let (cols, _) = grid.dimensions();
    let text = selection.get_text(|pos| grid.char_at(pos), cols);
    assert_eq!(text, "foo-bar_baz");
    assert_eq!(selection.kind, SelectionKind::Word);
}

#[test]
fn test_cursor_positioning_sequence() {
    // Test cursor positioning for click-to-position
    let positions = vec![
        Position::new(0, 0),
        Position::new(10, 20),
        Position::new(79, 23),
        Position::new(100, 50),
    ];

    for pos in positions {
        let seq = generate_cursor_position_sequence(pos);
        assert!(!seq.is_empty());
        assert!(seq.starts_with(b"\x1b["));
        assert!(seq.ends_with(b"H"));
    }
}

#[test]
fn test_complex_modifier_combinations() {
    let test_cases = vec![
        (Modifiers { shift: true, ..Default::default() }, 4),
        (Modifiers { alt: true, ..Default::default() }, 8),
        (Modifiers { ctrl: true, ..Default::default() }, 16),
        (Modifiers { shift: true, alt: true, ..Default::default() }, 12),
        (Modifiers { shift: true, ctrl: true, ..Default::default() }, 20),
        (Modifiers { alt: true, ctrl: true, ..Default::default() }, 24),
        (Modifiers { shift: true, alt: true, ctrl: true, meta: false }, 28),
    ];

    for (modifiers, expected_offset) in test_cases {
        let event = MouseEvent {
            kind: MouseEventKind::Press,
            position: Position::new(0, 0),
            button: Some(MouseButton::Left),
            modifiers,
        };

        let seq = generate_mouse_sequence(&event).unwrap();
        let seq_str = String::from_utf8_lossy(&seq);

        // Extract button code from sequence
        // Format is \x1b[<CODE;x;yM
        assert!(seq_str.contains(&format!("{};", expected_offset)));
    }
}

#[test]
fn test_menu_navigation_with_selection() {
    let mut menu = ContextMenu::standard(Position::new(10, 10), true);

    // Navigate to copy command
    while menu.selected_item().map(|i| i.id.as_str()) != Some("copy") {
        menu.select_next();
    }

    let selected = menu.selected_item().unwrap();
    assert_eq!(selected.id, "copy");
    assert!(selected.enabled);
}

#[test]
fn test_app_mode_detection_workflow() {
    let apps_requiring_mouse = vec!["vim", "nvim", "tmux", "htop", "less"];
    let apps_not_requiring_mouse = vec!["bash", "zsh", "cat", "ls"];

    for app in apps_requiring_mouse {
        assert!(AppHeuristics::detect_mouse_app(app));
        assert!(AppHeuristics::get_app_description(app).is_some());
    }

    for app in apps_not_requiring_mouse {
        assert!(!AppHeuristics::detect_mouse_app(app));
    }
}

#[test]
fn test_selection_with_empty_cells() {
    let grid = MockTerminalGrid::new(vec!["Hello     World"]);

    let selection = Selection::character(Position::new(0, 0), Position::new(14, 0));

    let (cols, _) = grid.dimensions();
    let text = selection.get_text(|pos| grid.char_at(pos), cols);

    // Should include spaces
    assert_eq!(text, "Hello     World");
    assert_eq!(text.len(), 15);
}

#[test]
fn test_mouse_event_with_all_buttons() {
    let buttons = vec![
        MouseButton::Left,
        MouseButton::Right,
        MouseButton::Middle,
        MouseButton::ScrollUp,
        MouseButton::ScrollDown,
    ];

    for button in buttons {
        let event = MouseEvent {
            kind: MouseEventKind::Press,
            position: Position::new(5, 5),
            button: Some(button),
            modifiers: Modifiers::none(),
        };

        let sequence = generate_mouse_sequence(&event);
        assert!(sequence.is_some());
    }
}

#[test]
fn test_complete_selection_and_copy_workflow() {
    let grid = MockTerminalGrid::new(vec![
        "$ cargo build",
        "   Compiling scarab v0.1.0",
        "    Finished dev [unoptimized] target(s)",
    ]);

    // User selects "cargo build" by double-clicking
    let mut detector = ClickDetector::new();
    let click_pos = Position::new(3, 0);

    let event = MouseEvent {
        kind: MouseEventKind::Press,
        position: click_pos,
        button: Some(MouseButton::Left),
        modifiers: Modifiers::none(),
    };

    detector.handle_press(&event);
    let click_type = detector.handle_press(&event);
    assert_eq!(click_type, ClickType::Double);

    // Find word at position
    let word_bounds = find_word_at(click_pos, |pos| grid.char_at(pos));
    assert!(word_bounds.is_some());

    let (start, end) = word_bounds.unwrap();
    let selection = Selection::word(start, end);

    // Extract text for clipboard
    let (cols, _) = grid.dimensions();
    let text = selection.get_text(|pos| grid.char_at(pos), cols);
    assert_eq!(text, "cargo");

    // Open context menu for copy action
    let menu = ContextMenu::standard(click_pos, true);
    let copy_item = menu.get_item("copy").unwrap();
    assert!(copy_item.enabled);
}

#[test]
fn test_backward_selection_normalization() {
    let grid = MockTerminalGrid::new(vec!["Hello World"]);

    // User drags from right to left (backward selection)
    let selection = Selection::character(Position::new(10, 0), Position::new(0, 0));

    // Should still work correctly when normalized
    let (start, end) = selection.normalized();
    assert!(start.x < end.x || start.y < end.y || (start.x == end.x && start.y == end.y));

    let (cols, _) = grid.dimensions();
    let text = selection.get_text(|pos| grid.char_at(pos), cols);
    assert_eq!(text, "Hello World");
}

#[test]
fn test_click_distance_threshold() {
    let mut detector = ClickDetector::new();

    let pos1 = Position::new(10, 10);
    let pos2 = Position::new(15, 10); // 5 units away (beyond threshold)

    let event1 = MouseEvent {
        kind: MouseEventKind::Press,
        position: pos1,
        button: Some(MouseButton::Left),
        modifiers: Modifiers::none(),
    };

    let event2 = MouseEvent {
        kind: MouseEventKind::Press,
        position: pos2,
        button: Some(MouseButton::Left),
        modifiers: Modifiers::none(),
    };

    detector.handle_press(&event1);
    // Click far away should not count as double-click
    let click_type = detector.handle_press(&event2);
    assert_eq!(click_type, ClickType::Single);
}
