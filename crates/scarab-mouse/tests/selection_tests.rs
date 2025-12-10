//! Integration tests for text selection functionality

use scarab_mouse::selection::{find_word_at, Selection, SelectionKind};
use scarab_mouse::types::Position;

// Helper function to create a simple character grid for testing
fn create_test_grid() -> Vec<Vec<char>> {
    vec![
        vec!['H', 'e', 'l', 'l', 'o', ' ', 'W', 'o', 'r', 'l', 'd'],
        vec![
            'T', 'h', 'i', 's', ' ', 'i', 's', ' ', 'a', ' ', 't', 'e', 's', 't',
        ],
        vec!['f', 'o', 'o', '-', 'b', 'a', 'r', '_', 'b', 'a', 'z'],
        vec![
            'h', 't', 't', 'p', 's', ':', '/', '/', 'e', 'x', 'a', 'm', 'p', 'l', 'e', '.', 'c',
            'o', 'm',
        ],
    ]
}

fn get_char_from_grid(grid: &[Vec<char>], pos: Position) -> Option<char> {
    grid.get(pos.y as usize)
        .and_then(|row| row.get(pos.x as usize))
        .copied()
}

#[test]
fn test_selection_creation() {
    let sel = Selection::new(
        Position::new(0, 0),
        Position::new(5, 0),
        SelectionKind::Normal,
    );
    assert_eq!(sel.start, Position::new(0, 0));
    assert_eq!(sel.end, Position::new(5, 0));
    assert_eq!(sel.kind, SelectionKind::Normal);
}

#[test]
fn test_character_selection() {
    let sel = Selection::character(Position::new(2, 1), Position::new(7, 1));
    assert_eq!(sel.kind, SelectionKind::Normal);
}

#[test]
fn test_word_selection() {
    let sel = Selection::word(Position::new(0, 0), Position::new(4, 0));
    assert_eq!(sel.kind, SelectionKind::Word);
}

#[test]
fn test_line_selection() {
    let sel = Selection::line(0, 2, 80);
    assert_eq!(sel.start, Position::new(0, 0));
    assert_eq!(sel.end, Position::new(79, 2));
    assert_eq!(sel.kind, SelectionKind::Line);
}

#[test]
fn test_selection_normalization() {
    // Forward selection
    let sel = Selection::character(Position::new(2, 1), Position::new(5, 3));
    let (start, end) = sel.normalized();
    assert_eq!(start, Position::new(2, 1));
    assert_eq!(end, Position::new(5, 3));

    // Backward selection (should be swapped)
    let sel = Selection::character(Position::new(5, 3), Position::new(2, 1));
    let (start, end) = sel.normalized();
    assert_eq!(start, Position::new(2, 1));
    assert_eq!(end, Position::new(5, 3));
}

#[test]
fn test_selection_normalization_same_line() {
    // Forward on same line
    let sel = Selection::character(Position::new(2, 1), Position::new(5, 1));
    let (start, end) = sel.normalized();
    assert_eq!(start, Position::new(2, 1));
    assert_eq!(end, Position::new(5, 1));

    // Backward on same line
    let sel = Selection::character(Position::new(5, 1), Position::new(2, 1));
    let (start, end) = sel.normalized();
    assert_eq!(start, Position::new(2, 1));
    assert_eq!(end, Position::new(5, 1));
}

#[test]
fn test_selection_contains_linear() {
    let sel = Selection::character(Position::new(2, 1), Position::new(5, 3));

    // Before selection
    assert!(!sel.contains(Position::new(1, 1)));
    assert!(!sel.contains(Position::new(0, 0)));

    // Start of selection
    assert!(sel.contains(Position::new(2, 1)));

    // Middle of selection
    assert!(sel.contains(Position::new(4, 2)));

    // End of selection
    assert!(sel.contains(Position::new(5, 3)));

    // After selection
    assert!(!sel.contains(Position::new(6, 3)));
    assert!(!sel.contains(Position::new(0, 4)));
}

#[test]
fn test_selection_contains_block() {
    let sel = Selection::new(
        Position::new(2, 1),
        Position::new(5, 3),
        SelectionKind::Block,
    );

    // Inside rectangle
    assert!(sel.contains(Position::new(2, 1)));
    assert!(sel.contains(Position::new(3, 2)));
    assert!(sel.contains(Position::new(5, 3)));

    // Outside rectangle - left/right
    assert!(!sel.contains(Position::new(1, 2)));
    assert!(!sel.contains(Position::new(6, 2)));

    // Outside rectangle - top/bottom
    assert!(!sel.contains(Position::new(3, 0)));
    assert!(!sel.contains(Position::new(3, 4)));
}

#[test]
fn test_selection_update_end() {
    let mut sel = Selection::character(Position::new(2, 1), Position::new(5, 1));
    assert_eq!(sel.end, Position::new(5, 1));

    sel.update_end(Position::new(10, 2));
    assert_eq!(sel.end, Position::new(10, 2));
}

#[test]
fn test_expand_to_word() {
    let grid = create_test_grid();

    // Start with selection in middle of "World"
    let mut sel = Selection::character(Position::new(7, 0), Position::new(8, 0));

    sel.expand_to_word(|pos| get_char_from_grid(&grid, pos));

    // Should expand to full word "World"
    assert_eq!(sel.start.x, 6); // 'W'
    assert_eq!(sel.end.x, 10); // 'd'
    assert_eq!(sel.kind, SelectionKind::Word);
}

#[test]
fn test_expand_to_word_with_hyphens() {
    let grid = create_test_grid();

    // Select middle of "foo-bar_baz"
    let mut sel = Selection::character(Position::new(4, 2), Position::new(4, 2));

    sel.expand_to_word(|pos| get_char_from_grid(&grid, pos));

    // Should expand to full compound word
    assert_eq!(sel.start.x, 0); // 'f'
    assert_eq!(sel.end.x, 10); // 'z'
    assert_eq!(sel.kind, SelectionKind::Word);
}

#[test]
fn test_get_text_single_line() {
    let grid = create_test_grid();
    let sel = Selection::character(Position::new(0, 0), Position::new(4, 0));

    let text = sel.get_text(|pos| get_char_from_grid(&grid, pos), 80);
    assert_eq!(text, "Hello");
}

#[test]
fn test_get_text_multi_line() {
    let grid = create_test_grid();
    let sel = Selection::character(Position::new(6, 0), Position::new(3, 1));

    let text = sel.get_text(|pos| get_char_from_grid(&grid, pos), 14);
    assert!(text.contains("World"));
    assert!(text.contains("\n"));
    assert!(text.contains("This"));
}

#[test]
fn test_get_text_block_selection() {
    let grid = create_test_grid();

    // Select a rectangular block
    let sel = Selection::new(
        Position::new(0, 0),
        Position::new(4, 1),
        SelectionKind::Block,
    );

    let text = sel.get_text(|pos| get_char_from_grid(&grid, pos), 14);

    // Should have two lines
    assert!(text.contains("\n"));
    let lines: Vec<&str> = text.lines().collect();
    assert_eq!(lines.len(), 2);
}

#[test]
fn test_get_text_full_line() {
    let grid = create_test_grid();
    let sel = Selection::line(0, 0, 11);

    let text = sel.get_text(|pos| get_char_from_grid(&grid, pos), 11);
    assert_eq!(text, "Hello World");
}

#[test]
fn test_find_word_at_valid() {
    let grid = create_test_grid();

    // Find word "Hello"
    let result = find_word_at(Position::new(1, 0), |pos| get_char_from_grid(&grid, pos));
    assert!(result.is_some());
    let (start, end) = result.unwrap();
    assert_eq!(start.x, 0);
    assert_eq!(end.x, 4);
}

#[test]
fn test_find_word_at_compound() {
    let grid = create_test_grid();

    // Find compound word "foo-bar_baz"
    let result = find_word_at(Position::new(5, 2), |pos| get_char_from_grid(&grid, pos));
    assert!(result.is_some());
    let (start, end) = result.unwrap();
    assert_eq!(start.x, 0);
    assert_eq!(end.x, 10);
}

#[test]
fn test_find_word_at_space() {
    let grid = create_test_grid();

    // Try to find word at space position
    let result = find_word_at(Position::new(5, 0), |pos| get_char_from_grid(&grid, pos));
    assert!(result.is_none());
}

#[test]
fn test_find_word_at_start_of_line() {
    let grid = create_test_grid();

    let result = find_word_at(Position::new(0, 1), |pos| get_char_from_grid(&grid, pos));
    assert!(result.is_some());
    let (start, end) = result.unwrap();
    assert_eq!(start.x, 0);
    assert_eq!(end.x, 3); // "This"
}

#[test]
fn test_find_word_at_end_of_line() {
    let grid = create_test_grid();

    let result = find_word_at(Position::new(10, 0), |pos| get_char_from_grid(&grid, pos));
    assert!(result.is_some());
    let (start, end) = result.unwrap();
    assert_eq!(start.x, 6);
    assert_eq!(end.x, 10); // "World"
}

#[test]
fn test_selection_kinds() {
    let pos1 = Position::new(0, 0);
    let pos2 = Position::new(5, 0);

    let normal = Selection::new(pos1, pos2, SelectionKind::Normal);
    assert_eq!(normal.kind, SelectionKind::Normal);

    let word = Selection::new(pos1, pos2, SelectionKind::Word);
    assert_eq!(word.kind, SelectionKind::Word);

    let line = Selection::new(pos1, pos2, SelectionKind::Line);
    assert_eq!(line.kind, SelectionKind::Line);

    let block = Selection::new(pos1, pos2, SelectionKind::Block);
    assert_eq!(block.kind, SelectionKind::Block);
}

#[test]
fn test_selection_equality() {
    let sel1 = Selection::character(Position::new(0, 0), Position::new(5, 0));
    let sel2 = Selection::character(Position::new(0, 0), Position::new(5, 0));
    let sel3 = Selection::character(Position::new(0, 0), Position::new(6, 0));

    assert_eq!(sel1, sel2);
    assert_ne!(sel1, sel3);
}

#[test]
fn test_empty_selection() {
    let grid = create_test_grid();

    // Zero-width selection
    let sel = Selection::character(Position::new(5, 0), Position::new(5, 0));
    let text = sel.get_text(|pos| get_char_from_grid(&grid, pos), 80);

    // Should contain single character
    assert_eq!(text.len(), 1);
}

#[test]
fn test_selection_across_empty_cells() {
    let sparse_grid = vec![vec!['A', ' ', ' ', 'B'], vec![' ', 'C', ' ', ' ']];

    let sel = Selection::character(Position::new(0, 0), Position::new(3, 1));
    let text = sel.get_text(|pos| get_char_from_grid(&sparse_grid, pos), 4);

    // Should include spaces
    assert!(text.contains(' '));
    assert!(text.contains('A'));
    assert!(text.contains('B'));
    assert!(text.contains('C'));
}

#[test]
fn test_block_selection_text_extraction() {
    let grid = vec![
        vec!['1', '2', '3', '4', '5'],
        vec!['6', '7', '8', '9', '0'],
        vec!['A', 'B', 'C', 'D', 'E'],
    ];

    // Select middle 3x3 block
    let sel = Selection::new(
        Position::new(1, 0),
        Position::new(3, 2),
        SelectionKind::Block,
    );

    let text = sel.get_text(|pos| get_char_from_grid(&grid, pos), 5);
    let lines: Vec<&str> = text.lines().collect();

    assert_eq!(lines.len(), 3);
    assert_eq!(lines[0], "234");
    assert_eq!(lines[1], "789");
    assert_eq!(lines[2], "BCD");
}

#[test]
fn test_normalized_preserves_single_point() {
    let sel = Selection::character(Position::new(5, 5), Position::new(5, 5));
    let (start, end) = sel.normalized();
    assert_eq!(start, Position::new(5, 5));
    assert_eq!(end, Position::new(5, 5));
}
