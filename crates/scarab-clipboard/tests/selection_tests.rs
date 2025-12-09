//! Integration tests for text selection functionality
//!
//! These tests verify selection modes, region calculations, and state management.

use scarab_clipboard::{SelectionMode, SelectionRegion, SelectionState};

// =============================================================================
// SelectionMode Tests
// =============================================================================

#[test]
fn test_selection_mode_variants() {
    let character = SelectionMode::Character;
    let word = SelectionMode::Word;
    let line = SelectionMode::Line;
    let block = SelectionMode::Block;

    assert_ne!(character, word);
    assert_ne!(word, line);
    assert_ne!(line, block);
    assert_ne!(character, block);
}

#[test]
fn test_selection_mode_default() {
    let default_mode = SelectionMode::default();
    assert_eq!(default_mode, SelectionMode::Character);
}

#[test]
fn test_selection_mode_clone_copy() {
    let mode = SelectionMode::Line;
    let cloned = mode;
    assert_eq!(mode, cloned);
}

// =============================================================================
// SelectionRegion Tests
// =============================================================================

#[test]
fn test_selection_region_new() {
    let region = SelectionRegion::new(10, 20, 30, 40);

    assert_eq!(region.start_x, 10);
    assert_eq!(region.start_y, 20);
    assert_eq!(region.end_x, 30);
    assert_eq!(region.end_y, 40);
}

#[test]
fn test_selection_region_default() {
    let region = SelectionRegion::default();

    assert_eq!(region.start_x, 0);
    assert_eq!(region.start_y, 0);
    assert_eq!(region.end_x, 0);
    assert_eq!(region.end_y, 0);
}

#[test]
fn test_selection_region_contains_simple() {
    let region = SelectionRegion::new(5, 5, 10, 10);

    // Corner points
    assert!(region.contains(5, 5));
    assert!(region.contains(10, 10));
    assert!(region.contains(5, 10));
    assert!(region.contains(10, 5));

    // Center point
    assert!(region.contains(7, 7));

    // Edge points
    assert!(region.contains(5, 7));
    assert!(region.contains(10, 7));
    assert!(region.contains(7, 5));
    assert!(region.contains(7, 10));
}

#[test]
fn test_selection_region_contains_outside() {
    let region = SelectionRegion::new(5, 5, 10, 10);

    // Points outside the region
    assert!(!region.contains(4, 5));
    assert!(!region.contains(11, 10));
    assert!(!region.contains(5, 4));
    assert!(!region.contains(10, 11));
    assert!(!region.contains(0, 0));
    assert!(!region.contains(15, 15));
}

#[test]
fn test_selection_region_contains_reversed() {
    // Test with reversed coordinates (end before start)
    let region = SelectionRegion::new(10, 10, 5, 5);

    // Should still contain points in the logical region
    assert!(region.contains(5, 5));
    assert!(region.contains(10, 10));
    assert!(region.contains(7, 7));
}

#[test]
fn test_selection_region_is_empty_true() {
    let region = SelectionRegion::new(5, 5, 5, 5);
    assert!(region.is_empty());

    let region2 = SelectionRegion::default();
    assert!(region2.is_empty());
}

#[test]
fn test_selection_region_is_empty_false() {
    let region1 = SelectionRegion::new(5, 5, 10, 5);
    assert!(!region1.is_empty());

    let region2 = SelectionRegion::new(5, 5, 5, 10);
    assert!(!region2.is_empty());

    let region3 = SelectionRegion::new(5, 5, 10, 10);
    assert!(!region3.is_empty());
}

#[test]
fn test_selection_region_normalize_forward() {
    let mut region = SelectionRegion::new(5, 5, 10, 10);
    region.normalize();

    // Should remain unchanged (already normalized)
    assert_eq!(region.start_x, 5);
    assert_eq!(region.start_y, 5);
    assert_eq!(region.end_x, 10);
    assert_eq!(region.end_y, 10);
}

#[test]
fn test_selection_region_normalize_backward() {
    let mut region = SelectionRegion::new(10, 10, 5, 5);
    region.normalize();

    // Should swap coordinates
    assert_eq!(region.start_x, 5);
    assert_eq!(region.start_y, 5);
    assert_eq!(region.end_x, 10);
    assert_eq!(region.end_y, 10);
}

#[test]
fn test_selection_region_normalize_same_row() {
    let mut region = SelectionRegion::new(10, 5, 5, 5);
    region.normalize();

    // Should swap x coordinates when on same row
    assert_eq!(region.start_x, 5);
    assert_eq!(region.start_y, 5);
    assert_eq!(region.end_x, 10);
    assert_eq!(region.end_y, 5);
}

#[test]
fn test_selection_region_normalize_different_rows() {
    let mut region = SelectionRegion::new(10, 8, 5, 3);
    region.normalize();

    // Should swap when start_y > end_y
    assert_eq!(region.start_x, 5);
    assert_eq!(region.start_y, 3);
    assert_eq!(region.end_x, 10);
    assert_eq!(region.end_y, 8);
}

#[test]
fn test_selection_region_normalized_method() {
    let region = SelectionRegion::new(10, 10, 5, 5);
    let normalized = region.normalized();

    // Original should be unchanged
    assert_eq!(region.start_x, 10);
    assert_eq!(region.start_y, 10);
    assert_eq!(region.end_x, 5);
    assert_eq!(region.end_y, 5);

    // Normalized copy should be swapped
    assert_eq!(normalized.start_x, 5);
    assert_eq!(normalized.start_y, 5);
    assert_eq!(normalized.end_x, 10);
    assert_eq!(normalized.end_y, 10);
}

#[test]
fn test_selection_region_width() {
    let region1 = SelectionRegion::new(5, 5, 10, 10);
    assert_eq!(region1.width(), 6); // 10 - 5 + 1

    let region2 = SelectionRegion::new(0, 0, 79, 0);
    assert_eq!(region2.width(), 80); // Full 80-column width

    let region3 = SelectionRegion::new(10, 5, 5, 10);
    assert_eq!(region3.width(), 6); // Handles reversed coordinates
}

#[test]
fn test_selection_region_width_single_column() {
    let region = SelectionRegion::new(5, 5, 5, 10);
    assert_eq!(region.width(), 1);
}

#[test]
fn test_selection_region_height() {
    let region1 = SelectionRegion::new(5, 5, 10, 10);
    assert_eq!(region1.height(), 6); // 10 - 5 + 1

    let region2 = SelectionRegion::new(0, 0, 0, 23);
    assert_eq!(region2.height(), 24); // 24 rows

    let region3 = SelectionRegion::new(5, 10, 10, 5);
    assert_eq!(region3.height(), 6); // Handles reversed coordinates
}

#[test]
fn test_selection_region_height_single_row() {
    let region = SelectionRegion::new(5, 5, 10, 5);
    assert_eq!(region.height(), 1);
}

#[test]
fn test_selection_region_expand_to() {
    let mut region = SelectionRegion::new(5, 5, 5, 5);

    region.expand_to(10, 10);

    assert_eq!(region.start_x, 5);
    assert_eq!(region.start_y, 5);
    assert_eq!(region.end_x, 10);
    assert_eq!(region.end_y, 10);
}

#[test]
fn test_selection_region_expand_to_multiple_times() {
    let mut region = SelectionRegion::new(5, 5, 5, 5);

    region.expand_to(7, 7);
    assert_eq!(region.end_x, 7);
    assert_eq!(region.end_y, 7);

    region.expand_to(10, 10);
    assert_eq!(region.end_x, 10);
    assert_eq!(region.end_y, 10);

    region.expand_to(3, 3);
    assert_eq!(region.end_x, 3);
    assert_eq!(region.end_y, 3);
}

#[test]
fn test_selection_region_clone() {
    let region1 = SelectionRegion::new(5, 5, 10, 10);
    let region2 = region1.clone();

    assert_eq!(region1, region2);
    assert_eq!(region1.start_x, region2.start_x);
    assert_eq!(region1.start_y, region2.start_y);
    assert_eq!(region1.end_x, region2.end_x);
    assert_eq!(region1.end_y, region2.end_y);
}

// =============================================================================
// SelectionState Tests
// =============================================================================

#[test]
fn test_selection_state_new() {
    let state = SelectionState::new();

    assert!(!state.active);
    assert_eq!(state.mode, SelectionMode::default());
    assert!(state.region.is_empty());
}

#[test]
fn test_selection_state_default() {
    let state = SelectionState::default();

    assert!(!state.active);
    assert_eq!(state.mode, SelectionMode::default());
    assert!(state.region.is_empty());
}

#[test]
fn test_selection_state_start_character() {
    let mut state = SelectionState::new();

    state.start(10, 20, SelectionMode::Character);

    assert!(state.active);
    assert_eq!(state.mode, SelectionMode::Character);
    assert_eq!(state.region.start_x, 10);
    assert_eq!(state.region.start_y, 20);
    assert_eq!(state.region.end_x, 10);
    assert_eq!(state.region.end_y, 20);
}

#[test]
fn test_selection_state_start_different_modes() {
    let modes = [
        SelectionMode::Character,
        SelectionMode::Word,
        SelectionMode::Line,
        SelectionMode::Block,
    ];

    for mode in modes {
        let mut state = SelectionState::new();
        state.start(5, 5, mode);

        assert!(state.active);
        assert_eq!(state.mode, mode);
        assert_eq!(state.region.start_x, 5);
        assert_eq!(state.region.start_y, 5);
    }
}

#[test]
fn test_selection_state_update_active() {
    let mut state = SelectionState::new();

    state.start(5, 5, SelectionMode::Character);
    state.update(10, 10);

    assert!(state.active);
    assert_eq!(state.region.start_x, 5);
    assert_eq!(state.region.start_y, 5);
    assert_eq!(state.region.end_x, 10);
    assert_eq!(state.region.end_y, 10);
}

#[test]
fn test_selection_state_update_inactive() {
    let mut state = SelectionState::new();

    // Update without starting selection (should not change region)
    state.update(10, 10);

    assert!(!state.active);
    assert_eq!(state.region.start_x, 0);
    assert_eq!(state.region.start_y, 0);
    assert_eq!(state.region.end_x, 0);
    assert_eq!(state.region.end_y, 0);
}

#[test]
fn test_selection_state_update_multiple_times() {
    let mut state = SelectionState::new();

    state.start(5, 5, SelectionMode::Character);

    state.update(7, 7);
    assert_eq!(state.region.end_x, 7);
    assert_eq!(state.region.end_y, 7);

    state.update(10, 10);
    assert_eq!(state.region.end_x, 10);
    assert_eq!(state.region.end_y, 10);

    state.update(8, 12);
    assert_eq!(state.region.end_x, 8);
    assert_eq!(state.region.end_y, 12);
}

#[test]
fn test_selection_state_clear() {
    let mut state = SelectionState::new();

    state.start(5, 5, SelectionMode::Line);
    state.update(10, 10);

    assert!(state.active);
    assert!(!state.region.is_empty());

    state.clear();

    assert!(!state.active);
    assert!(state.region.is_empty());
    assert_eq!(state.region.start_x, 0);
    assert_eq!(state.region.start_y, 0);
    assert_eq!(state.region.end_x, 0);
    assert_eq!(state.region.end_y, 0);
}

#[test]
fn test_selection_state_has_selection_false_when_inactive() {
    let state = SelectionState::new();

    assert!(!state.has_selection());
}

#[test]
fn test_selection_state_has_selection_false_when_empty() {
    let mut state = SelectionState::new();

    state.start(5, 5, SelectionMode::Character);

    // Selection is active but still at start point (empty)
    assert!(!state.has_selection());
}

#[test]
fn test_selection_state_has_selection_true() {
    let mut state = SelectionState::new();

    state.start(5, 5, SelectionMode::Character);
    state.update(10, 10);

    assert!(state.has_selection());
}

#[test]
fn test_selection_state_has_selection_single_column() {
    let mut state = SelectionState::new();

    state.start(5, 5, SelectionMode::Character);
    state.update(10, 5); // Same row, different column

    assert!(state.has_selection());
}

#[test]
fn test_selection_state_has_selection_single_row() {
    let mut state = SelectionState::new();

    state.start(5, 5, SelectionMode::Character);
    state.update(5, 10); // Same column, different row

    assert!(state.has_selection());
}

#[test]
fn test_selection_state_normalized_region() {
    let mut state = SelectionState::new();

    state.start(10, 10, SelectionMode::Character);
    state.update(5, 5);

    let normalized = state.normalized_region();

    // Original region should be unchanged in state
    assert_eq!(state.region.start_x, 10);
    assert_eq!(state.region.start_y, 10);
    assert_eq!(state.region.end_x, 5);
    assert_eq!(state.region.end_y, 5);

    // Normalized copy should be swapped
    assert_eq!(normalized.start_x, 5);
    assert_eq!(normalized.start_y, 5);
    assert_eq!(normalized.end_x, 10);
    assert_eq!(normalized.end_y, 10);
}

#[test]
fn test_selection_state_restart() {
    let mut state = SelectionState::new();

    // First selection
    state.start(5, 5, SelectionMode::Character);
    state.update(10, 10);

    assert!(state.has_selection());
    assert_eq!(state.mode, SelectionMode::Character);

    // Start new selection (should replace old one)
    state.start(20, 20, SelectionMode::Line);

    assert!(state.active);
    assert_eq!(state.mode, SelectionMode::Line);
    assert_eq!(state.region.start_x, 20);
    assert_eq!(state.region.start_y, 20);
    assert_eq!(state.region.end_x, 20);
    assert_eq!(state.region.end_y, 20);
}

#[test]
fn test_selection_state_lifecycle() {
    let mut state = SelectionState::new();

    // 1. Start
    state.start(0, 0, SelectionMode::Block);
    assert!(state.active);
    assert!(!state.has_selection()); // Empty at start

    // 2. Expand
    state.update(5, 5);
    assert!(state.has_selection());

    // 3. Continue expanding
    state.update(10, 10);
    assert!(state.has_selection());

    // 4. Clear
    state.clear();
    assert!(!state.active);
    assert!(!state.has_selection());
}

// =============================================================================
// Edge Cases and Boundary Tests
// =============================================================================

#[test]
fn test_selection_region_max_coordinates() {
    let mut region = SelectionRegion::new(u16::MAX, u16::MAX, 0, 0);
    region.normalize(); // Should not panic

    assert!(region.contains(100, 100));
}

#[test]
fn test_selection_region_zero_coordinates() {
    let region = SelectionRegion::new(0, 0, 0, 0);

    assert!(region.is_empty());
    assert!(region.contains(0, 0));
    assert_eq!(region.width(), 1);
    assert_eq!(region.height(), 1);
}

#[test]
fn test_selection_state_update_to_same_position() {
    let mut state = SelectionState::new();

    state.start(5, 5, SelectionMode::Character);
    state.update(5, 5);

    assert!(state.active);
    assert!(!state.has_selection()); // Still at same position
}

#[test]
fn test_large_selection_region() {
    // Test with a large terminal (200 columns x 100 rows)
    let region = SelectionRegion::new(0, 0, 199, 99);

    assert_eq!(region.width(), 200);
    assert_eq!(region.height(), 100);
    assert!(region.contains(0, 0));
    assert!(region.contains(199, 99));
    assert!(region.contains(100, 50));
}
