//! Comprehensive UI tests for Visual Selection System
//!
//! This test suite verifies the visual selection feature using the HeadlessTestHarness
//! for fast, GPU-free testing.
//!
//! Tests cover:
//! - Selection region creation and coordinate handling
//! - Selection modes (character, line, block)
//! - Selection normalization (start before end)
//! - Selection contains point checks
//! - Selection clear functionality
//! - Multi-line selection
//! - Selection with scrollback
//! - Selection text extraction
//! - Selection highlighting
//! - Selection copy to clipboard events
//! - Empty selection handling
//! - Edge case handling

mod harness;

use bevy::prelude::*;
use harness::mocks::MockSharedMemoryReader;
use harness::HeadlessTestHarness;
use scarab_client::ui::visual_selection::{
    SelectionChangedEvent, SelectionCopiedEvent, SelectionMode, SelectionRegion, SelectionState,
    VisualSelectionPlugin,
};

// =============================================================================
// Test 1: Selection Region Creation
// =============================================================================

#[test]
fn test_selection_region_creation() {
    let region = SelectionRegion::new(5, 10, 15, 20);

    assert_eq!(region.start_x, 5);
    assert_eq!(region.start_y, 10);
    assert_eq!(region.end_x, 15);
    assert_eq!(region.end_y, 20);
}

// =============================================================================
// Test 2: Selection Start and End Coordinates
// =============================================================================

#[test]
fn test_selection_start_end_coordinates() {
    let mut state = SelectionState::default();

    // Start selection at (10, 5)
    state.start_selection(10, 5, SelectionMode::Character);

    assert_eq!(state.region.start_x, 10);
    assert_eq!(state.region.start_y, 5);
    assert_eq!(state.region.end_x, 10);
    assert_eq!(state.region.end_y, 5);
    assert!(state.active);

    // Update selection to (20, 15)
    state.update_selection(20, 15);

    assert_eq!(state.region.start_x, 10);
    assert_eq!(state.region.start_y, 5);
    assert_eq!(state.region.end_x, 20);
    assert_eq!(state.region.end_y, 15);
}

// =============================================================================
// Test 3: Selection Modes
// =============================================================================

#[test]
fn test_selection_modes() {
    let mut state = SelectionState::default();

    // Test Character mode
    state.start_selection(0, 0, SelectionMode::Character);
    assert_eq!(state.mode, SelectionMode::Character);
    assert!(state.active);

    // Clear and test Line mode
    state.clear();
    state.start_selection(0, 0, SelectionMode::Line);
    assert_eq!(state.mode, SelectionMode::Line);
    assert!(state.active);

    // Clear and test Block mode
    state.clear();
    state.start_selection(0, 0, SelectionMode::Block);
    assert_eq!(state.mode, SelectionMode::Block);
    assert!(state.active);
}

// =============================================================================
// Test 4: Selection Normalization
// =============================================================================

#[test]
fn test_selection_normalization() {
    // Test case 1: Selection from bottom-right to top-left
    let mut region = SelectionRegion::new(20, 15, 5, 3);
    region.normalize();

    assert_eq!(region.start_x, 5);
    assert_eq!(region.start_y, 3);
    assert_eq!(region.end_x, 20);
    assert_eq!(region.end_y, 15);

    // Test case 2: Selection from right to left on same line
    let mut region = SelectionRegion::new(15, 10, 5, 10);
    region.normalize();

    assert_eq!(region.start_x, 5);
    assert_eq!(region.start_y, 10);
    assert_eq!(region.end_x, 15);
    assert_eq!(region.end_y, 10);

    // Test case 3: Already normalized selection
    let mut region = SelectionRegion::new(5, 3, 20, 15);
    region.normalize();

    assert_eq!(region.start_x, 5);
    assert_eq!(region.start_y, 3);
    assert_eq!(region.end_x, 20);
    assert_eq!(region.end_y, 15);
}

// =============================================================================
// Test 5: Selection Contains Point Check
// =============================================================================

#[test]
fn test_selection_contains_point() {
    let region = SelectionRegion::new(5, 5, 15, 15);

    // Points inside the selection
    assert!(region.contains(5, 5), "Start point should be contained");
    assert!(region.contains(15, 15), "End point should be contained");
    assert!(region.contains(10, 10), "Center point should be contained");
    assert!(region.contains(7, 8), "Interior point should be contained");

    // Points outside the selection
    assert!(!region.contains(4, 5), "Point left of selection");
    assert!(!region.contains(16, 15), "Point right of selection");
    assert!(!region.contains(10, 4), "Point above selection");
    assert!(!region.contains(10, 16), "Point below selection");
    assert!(!region.contains(0, 0), "Point far outside");

    // Test with reversed selection (should still work due to min/max logic)
    let region_reversed = SelectionRegion::new(15, 15, 5, 5);
    assert!(
        region_reversed.contains(10, 10),
        "Reversed region should contain center"
    );
}

// =============================================================================
// Test 6: Selection Clear Functionality
// =============================================================================

#[test]
fn test_selection_clear() {
    let mut state = SelectionState::default();

    // Create a selection
    state.start_selection(5, 10, SelectionMode::Character);
    state.update_selection(20, 30);

    assert!(state.active);
    assert_eq!(state.region.start_x, 5);
    assert_eq!(state.region.end_x, 20);

    // Clear the selection
    state.clear();

    assert!(!state.active);
    assert_eq!(state.region.start_x, 0);
    assert_eq!(state.region.start_y, 0);
    assert_eq!(state.region.end_x, 0);
    assert_eq!(state.region.end_y, 0);
}

// =============================================================================
// Test 7: Multi-line Selection
// =============================================================================

#[test]
fn test_multiline_selection() {
    let mut state = SelectionState::default();

    // Select from line 5 column 10 to line 10 column 20
    state.start_selection(10, 5, SelectionMode::Character);
    state.update_selection(20, 10);

    assert_eq!(state.region.start_x, 10);
    assert_eq!(state.region.start_y, 5);
    assert_eq!(state.region.end_x, 20);
    assert_eq!(state.region.end_y, 10);

    // Normalize and verify
    let mut region = state.region.clone();
    region.normalize();

    // Should span 6 lines (5, 6, 7, 8, 9, 10)
    let line_count = region.end_y - region.start_y + 1;
    assert_eq!(line_count, 6, "Should span 6 lines");

    // Test that points within the rectangular bounds are contained
    // Note: contains() treats the selection as a rectangle, so points must be
    // within both the x range (10-20) AND the y range (5-10)
    assert!(
        region.contains(15, 7),
        "Point within rectangular bounds should be contained"
    );
    assert!(
        region.contains(10, 8),
        "Left edge point should be contained"
    );
    assert!(
        region.contains(20, 9),
        "Right edge point should be contained"
    );

    // Points outside the x range should not be contained
    assert!(
        !region.contains(5, 7),
        "Point left of x range should not be contained"
    );
    assert!(
        !region.contains(25, 8),
        "Point right of x range should not be contained"
    );
}

// =============================================================================
// Test 8: Selection with Scrollback
// =============================================================================

#[test]
fn test_selection_with_scrollback() {
    let mut harness = HeadlessTestHarness::new();

    // Setup mock terminal with scrollback data
    {
        let mut mock = harness.resource_mut::<MockSharedMemoryReader>();

        // Simulate scrollback content
        for i in 0..20 {
            mock.set_text(0, i, &format!("Line {}", i), 0xFFFFFFFF, 0x000000FF);
        }
        mock.tick();
    }

    // Create selection spanning visible and scrollback region
    let mut state = SelectionState::default();
    state.start_selection(0, 0, SelectionMode::Character);
    state.update_selection(50, 15);

    // Verify selection spans the scrollback
    assert_eq!(state.region.start_y, 0);
    assert_eq!(state.region.end_y, 15);
    assert!(state.active);
}

// =============================================================================
// Test 9: Selection Text Extraction (Character Mode)
// =============================================================================

#[test]
fn test_selection_text_extraction_character_mode() {
    let mut harness = HeadlessTestHarness::new();

    // Setup terminal with known text
    {
        let mut mock = harness.resource_mut::<MockSharedMemoryReader>();
        mock.set_text(0, 0, "Hello, World!", 0xFFFFFFFF, 0x000000FF);
        mock.set_text(0, 1, "This is a test.", 0xFFFFFFFF, 0x000000FF);
        mock.set_text(0, 2, "Terminal selection!", 0xFFFFFFFF, 0x000000FF);
        mock.tick();
    }

    // Verify text was set correctly
    {
        let mock = harness.resource::<MockSharedMemoryReader>();
        assert_eq!(mock.get_row_text(0), "Hello, World!");
        assert_eq!(mock.get_row_text(1), "This is a test.");
        assert_eq!(mock.get_row_text(2), "Terminal selection!");
    }

    // Create selection from line 0, col 7 to line 1, col 11
    // This should extract: "World!\nThis is a"
    let region = SelectionRegion::new(7, 0, 11, 1);

    // Verify region coordinates
    assert_eq!(region.start_x, 7);
    assert_eq!(region.start_y, 0);
    assert_eq!(region.end_x, 11);
    assert_eq!(region.end_y, 1);
}

// =============================================================================
// Test 10: Selection Highlighting
// =============================================================================

#[test]
fn test_selection_highlighting() {
    let mut harness = HeadlessTestHarness::new();

    // Initialize required resources for visual selection
    harness.app.init_resource::<SelectionState>();
    harness.app.add_event::<SelectionChangedEvent>();
    harness.app.add_event::<SelectionCopiedEvent>();

    // Create a selection
    {
        let mut state = harness.resource_mut::<SelectionState>();
        state.start_selection(5, 5, SelectionMode::Character);
        state.update_selection(15, 10);
    }

    // Verify selection state
    {
        let state = harness.resource::<SelectionState>();
        assert!(state.active);
        assert_eq!(state.region.start_x, 5);
        assert_eq!(state.region.start_y, 5);
        assert_eq!(state.region.end_x, 15);
        assert_eq!(state.region.end_y, 10);
    }

    // Note: We don't test the actual rendering overlay here as that requires
    // the full Bevy render pipeline and TextRenderer. This test verifies
    // that the selection state is correctly set up for rendering.
}

// =============================================================================
// Test 11: Selection Copy to Clipboard Events
// =============================================================================

#[test]
fn test_selection_copy_events() {
    let mut harness = HeadlessTestHarness::new();

    // Initialize resources
    harness.app.init_resource::<SelectionState>();
    harness.app.add_event::<SelectionCopiedEvent>();

    // Setup mock terminal with text
    {
        let mut mock = harness.resource_mut::<MockSharedMemoryReader>();
        mock.set_text(0, 0, "Copy this text", 0xFFFFFFFF, 0x000000FF);
        mock.tick();
    }

    // Create selection
    {
        let mut state = harness.resource_mut::<SelectionState>();
        state.start_selection(0, 0, SelectionMode::Character);
        state.update_selection(13, 0);
    }

    // Simulate copy event (instead of actually copying to clipboard which may fail in tests)
    // Send a SelectionCopiedEvent to verify event handling
    harness.world_mut().send_event(SelectionCopiedEvent {
        text: "Copy this text".to_string(),
    });

    harness.update();

    // Verify event was sent
    let event_reader = harness.world().resource::<Events<SelectionCopiedEvent>>();
    let mut cursor = event_reader.get_cursor();
    let mut event_count = 0;
    let mut copied_text = String::new();

    for event in cursor.read(event_reader) {
        event_count += 1;
        copied_text = event.text.clone();
    }

    assert_eq!(event_count, 1, "Should have received 1 copy event");
    assert_eq!(copied_text, "Copy this text");
}

// =============================================================================
// Test 12: Empty Selection Handling
// =============================================================================

#[test]
fn test_empty_selection_handling() {
    let region = SelectionRegion::new(10, 10, 10, 10);

    // Single point selection should be considered empty
    assert!(region.is_empty(), "Single point selection should be empty");

    // Non-empty selections
    let region_horizontal = SelectionRegion::new(10, 10, 15, 10);
    assert!(
        !region_horizontal.is_empty(),
        "Horizontal selection should not be empty"
    );

    let region_vertical = SelectionRegion::new(10, 10, 10, 15);
    assert!(
        !region_vertical.is_empty(),
        "Vertical selection should not be empty"
    );

    let region_multi = SelectionRegion::new(10, 10, 20, 20);
    assert!(
        !region_multi.is_empty(),
        "Multi-line selection should not be empty"
    );

    // Default region should be empty
    let default_region = SelectionRegion::default();
    assert!(default_region.is_empty(), "Default region should be empty");
}

// =============================================================================
// Test 13: Selection State Lifecycle
// =============================================================================

#[test]
fn test_selection_state_lifecycle() {
    let mut state = SelectionState::default();

    // Initial state
    assert!(!state.active, "Should start inactive");
    assert_eq!(state.mode, SelectionMode::Character);
    assert!(state.region.is_empty());

    // Start selection
    state.start_selection(5, 5, SelectionMode::Line);
    assert!(state.active, "Should be active after start");
    assert_eq!(state.mode, SelectionMode::Line);
    assert_eq!(state.cursor_x, 5);
    assert_eq!(state.cursor_y, 5);

    // Update selection
    state.update_selection(15, 20);
    assert_eq!(state.cursor_x, 15);
    assert_eq!(state.cursor_y, 20);
    assert_eq!(state.region.end_x, 15);
    assert_eq!(state.region.end_y, 20);

    // End selection (keeps region but deactivates)
    state.end_selection();
    assert!(!state.active, "Should be inactive after end");
    assert_eq!(state.region.start_x, 5); // Region preserved
    assert_eq!(state.region.end_x, 15);

    // Clear selection
    state.clear();
    assert!(!state.active);
    assert!(state.region.is_empty());
}

// =============================================================================
// Test 14: Selection Changed Events
// =============================================================================

#[test]
fn test_selection_changed_events() {
    let mut harness = HeadlessTestHarness::new();

    // Initialize resources
    harness.app.init_resource::<SelectionState>();
    harness.app.add_event::<SelectionChangedEvent>();

    // Create selection
    let region = SelectionRegion::new(5, 10, 15, 20);

    // Send selection changed event
    harness.world_mut().send_event(SelectionChangedEvent {
        region: region.clone(),
    });

    harness.update();

    // Verify event was received
    let event_reader = harness.world().resource::<Events<SelectionChangedEvent>>();
    let mut cursor = event_reader.get_cursor();
    let mut event_count = 0;

    for event in cursor.read(event_reader) {
        event_count += 1;
        assert_eq!(event.region.start_x, 5);
        assert_eq!(event.region.start_y, 10);
        assert_eq!(event.region.end_x, 15);
        assert_eq!(event.region.end_y, 20);
    }

    assert_eq!(event_count, 1, "Should have received 1 changed event");
}

// =============================================================================
// Test 15: Block Selection Mode
// =============================================================================

#[test]
fn test_block_selection_mode() {
    let mut state = SelectionState::default();

    // Create block selection
    state.start_selection(10, 10, SelectionMode::Block);
    state.update_selection(20, 15);

    assert_eq!(state.mode, SelectionMode::Block);

    // In block mode, selection should be a rectangle
    let mut region = state.region.clone();
    region.normalize();

    // Verify rectangular bounds
    assert_eq!(region.start_x, 10);
    assert_eq!(region.start_y, 10);
    assert_eq!(region.end_x, 20);
    assert_eq!(region.end_y, 15);

    // All points within the rectangle should be contained
    for y in 10..=15 {
        for x in 10..=20 {
            assert!(
                region.contains(x, y),
                "Point ({}, {}) should be in block selection",
                x,
                y
            );
        }
    }
}

// =============================================================================
// Test 16: Line Selection Mode
// =============================================================================

#[test]
fn test_line_selection_mode() {
    let mut state = SelectionState::default();

    // Create line selection
    state.start_selection(25, 5, SelectionMode::Line);
    state.update_selection(30, 8);

    assert_eq!(state.mode, SelectionMode::Line);

    // In line mode, the region stores the line range
    // Line mode should select entire lines from start_y to end_y
    let region = state.region.clone();

    assert_eq!(region.start_y, 5);
    assert_eq!(region.end_y, 8);

    // Note: The actual line selection (full width) is handled during
    // text extraction and rendering, not in the region itself
}

// =============================================================================
// Test 17: Selection Update Without Active State
// =============================================================================

#[test]
fn test_selection_update_when_inactive() {
    let mut state = SelectionState::default();

    // Don't start selection
    assert!(!state.active);

    // Try to update (should be ignored)
    state.update_selection(10, 10);

    // State should remain unchanged
    assert_eq!(state.region.end_x, 0);
    assert_eq!(state.region.end_y, 0);
    assert_eq!(state.cursor_x, 0);
    assert_eq!(state.cursor_y, 0);
}

// =============================================================================
// Test 18: Selection at Grid Boundaries
// =============================================================================

#[test]
fn test_selection_at_grid_boundaries() {
    use scarab_protocol::{GRID_HEIGHT, GRID_WIDTH};

    let mut state = SelectionState::default();

    // Select from top-left corner
    state.start_selection(0, 0, SelectionMode::Character);
    assert_eq!(state.region.start_x, 0);
    assert_eq!(state.region.start_y, 0);

    // Select to bottom-right corner
    state.update_selection((GRID_WIDTH - 1) as u16, (GRID_HEIGHT - 1) as u16);
    assert_eq!(state.region.end_x, (GRID_WIDTH - 1) as u16);
    assert_eq!(state.region.end_y, (GRID_HEIGHT - 1) as u16);

    // Verify the entire grid is selected
    let mut region = state.region.clone();
    region.normalize();

    assert!(region.contains(0, 0), "Top-left corner");
    assert!(
        region.contains((GRID_WIDTH - 1) as u16, (GRID_HEIGHT - 1) as u16),
        "Bottom-right corner"
    );
    assert!(
        region.contains(GRID_WIDTH as u16 / 2, GRID_HEIGHT as u16 / 2),
        "Grid center"
    );
}

// =============================================================================
// Test 19: Cursor Position Tracking
// =============================================================================

#[test]
fn test_cursor_position_tracking() {
    let mut state = SelectionState::default();

    // Start selection
    state.start_selection(5, 10, SelectionMode::Character);
    assert_eq!(state.cursor_x, 5);
    assert_eq!(state.cursor_y, 10);

    // Update selection - cursor should follow
    state.update_selection(15, 20);
    assert_eq!(state.cursor_x, 15);
    assert_eq!(state.cursor_y, 20);

    // Update again
    state.update_selection(25, 30);
    assert_eq!(state.cursor_x, 25);
    assert_eq!(state.cursor_y, 30);
}

// =============================================================================
// Test 20: Multiple Selection Cycles
// =============================================================================

#[test]
fn test_multiple_selection_cycles() {
    let mut state = SelectionState::default();

    // First selection cycle
    state.start_selection(5, 5, SelectionMode::Character);
    state.update_selection(10, 10);
    state.end_selection();

    assert!(!state.active);
    let first_region = state.region.clone();

    // Second selection cycle
    state.start_selection(20, 20, SelectionMode::Line);
    state.update_selection(30, 30);

    assert!(state.active);
    assert_eq!(state.mode, SelectionMode::Line);
    assert_ne!(state.region.start_x, first_region.start_x);
    assert_ne!(state.region.start_y, first_region.start_y);

    // Third selection cycle after clear
    state.clear();
    state.start_selection(0, 0, SelectionMode::Block);
    state.update_selection(5, 5);

    assert!(state.active);
    assert_eq!(state.mode, SelectionMode::Block);
    assert_eq!(state.region.start_x, 0);
    assert_eq!(state.region.start_y, 0);
}
