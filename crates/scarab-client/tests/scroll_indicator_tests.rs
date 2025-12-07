// Comprehensive scroll indicator UI tests using HeadlessTestHarness
// Tests the scroll indicator system integration with scrollback buffer

use bevy::prelude::*;

// Import test harness
mod harness;
use harness::mocks::MockSharedMemoryReader;
use harness::HeadlessTestHarness;

// Import scrollback components
use scarab_client::terminal::scrollback::{
    ScrollbackBuffer, ScrollbackLine, ScrollbackState, SearchState,
};
use scarab_protocol::Cell;

/// Helper to create a scrollback line from text
fn create_line(text: &str) -> ScrollbackLine {
    let cells: Vec<Cell> = text
        .chars()
        .map(|c| Cell {
            char_codepoint: c as u32,
            fg: 0xFFFFFFFF,
            bg: 0x000000FF,
            flags: 0,
            _padding: [0; 3],
        })
        .collect();
    ScrollbackLine::new(cells)
}

/// Helper to populate scrollback buffer with test lines
fn populate_scrollback(buffer: &mut ScrollbackBuffer, line_count: usize) {
    for i in 0..line_count {
        let line_text = format!("Line {} of scrollback history", i + 1);
        buffer.push_line(create_line(&line_text));
    }
}

/// Helper to create a percentage string for scroll position
fn scroll_percentage(buffer: &ScrollbackBuffer) -> f32 {
    if buffer.line_count() == 0 {
        return 100.0;
    }

    let offset = buffer.scroll_offset();
    let total = buffer.line_count();

    if offset == 0 {
        100.0 // At bottom
    } else if offset >= total {
        0.0 // At top
    } else {
        ((total - offset) as f32 / total as f32) * 100.0
    }
}

// =============================================================================
// Test 1: Scroll Position Tracking
// =============================================================================

#[test]
fn test_scroll_position_tracking() {
    let mut harness = HeadlessTestHarness::new();

    // Initialize scrollback buffer
    let mut buffer = ScrollbackBuffer::new(100);

    // Add 50 lines
    populate_scrollback(&mut buffer, 50);

    // Initially at bottom
    assert_eq!(buffer.scroll_offset(), 0);
    assert!(buffer.is_at_bottom());

    // Scroll up 10 lines
    buffer.scroll_up(10);
    assert_eq!(buffer.scroll_offset(), 10);
    assert!(!buffer.is_at_bottom());

    // Scroll up 20 more lines
    buffer.scroll_up(20);
    assert_eq!(buffer.scroll_offset(), 30);

    // Scroll down 15 lines
    buffer.scroll_down(15);
    assert_eq!(buffer.scroll_offset(), 15);

    // Scroll back to bottom
    buffer.scroll_to_bottom();
    assert_eq!(buffer.scroll_offset(), 0);
    assert!(buffer.is_at_bottom());
}

// =============================================================================
// Test 2: Scroll Indicator Visibility (Not at Bottom)
// =============================================================================

#[test]
fn test_scroll_indicator_visible_when_scrolled() {
    let mut harness = HeadlessTestHarness::new();

    // Initialize resources
    let mut buffer = ScrollbackBuffer::new(100);
    let mut state = ScrollbackState::new(25);

    // Add lines
    populate_scrollback(&mut buffer, 50);

    // Initially at bottom - indicator should not be shown
    state.is_scrolled = !buffer.is_at_bottom();
    assert!(!state.is_scrolled, "Indicator should be hidden at bottom");

    // Scroll up - indicator should be shown
    buffer.scroll_up(10);
    state.is_scrolled = !buffer.is_at_bottom();
    assert!(
        state.is_scrolled,
        "Indicator should be visible when scrolled"
    );

    // Verify scroll offset is actually set
    assert_eq!(buffer.scroll_offset(), 10);
}

// =============================================================================
// Test 3: Scroll Indicator Hide (When at Bottom)
// =============================================================================

#[test]
fn test_scroll_indicator_hidden_at_bottom() {
    let mut harness = HeadlessTestHarness::new();

    let mut buffer = ScrollbackBuffer::new(100);
    let mut state = ScrollbackState::new(25);

    populate_scrollback(&mut buffer, 50);

    // Scroll up
    buffer.scroll_up(20);
    state.is_scrolled = !buffer.is_at_bottom();
    assert!(state.is_scrolled);

    // Scroll back to bottom
    buffer.scroll_to_bottom();
    state.is_scrolled = !buffer.is_at_bottom();
    assert!(
        !state.is_scrolled,
        "Indicator should hide when returning to bottom"
    );
    assert!(buffer.is_at_bottom());
}

// =============================================================================
// Test 4: Scroll Percentage Calculation
// =============================================================================

#[test]
fn test_scroll_percentage_calculation() {
    let mut harness = HeadlessTestHarness::new();

    let mut buffer = ScrollbackBuffer::new(100);
    populate_scrollback(&mut buffer, 100);

    // At bottom - 100%
    let pct = scroll_percentage(&buffer);
    assert_eq!(pct, 100.0, "Should be 100% at bottom");

    // Scroll to top - 0%
    buffer.scroll_to_top();
    let pct = scroll_percentage(&buffer);
    assert_eq!(pct, 0.0, "Should be 0% at top");

    // Scroll to middle - ~50%
    buffer.scroll_to_bottom();
    buffer.scroll_up(50);
    let pct = scroll_percentage(&buffer);
    assert!(
        (pct - 50.0).abs() < 1.0,
        "Should be ~50% at middle, got {}",
        pct
    );

    // Scroll to 75% (25 lines from top means offset 25, which is 75% of the way from top to bottom)
    buffer.scroll_to_top();
    buffer.scroll_down(75);
    let pct = scroll_percentage(&buffer);
    assert!(
        (pct - 75.0).abs() < 1.0,
        "Should be ~75% at 25 lines from top, got {}",
        pct
    );
}

// =============================================================================
// Test 5: Scroll Up/Down Behavior
// =============================================================================

#[test]
fn test_scroll_up_down_behavior() {
    let mut harness = HeadlessTestHarness::new();

    let mut buffer = ScrollbackBuffer::new(100);
    populate_scrollback(&mut buffer, 50);

    // Test scroll up increments
    buffer.scroll_up(1);
    assert_eq!(buffer.scroll_offset(), 1);

    buffer.scroll_up(5);
    assert_eq!(buffer.scroll_offset(), 6);

    buffer.scroll_up(10);
    assert_eq!(buffer.scroll_offset(), 16);

    // Test scroll down decrements
    buffer.scroll_down(6);
    assert_eq!(buffer.scroll_offset(), 10);

    buffer.scroll_down(5);
    assert_eq!(buffer.scroll_offset(), 5);

    // Test scroll down past bottom (should clamp to 0)
    buffer.scroll_down(100);
    assert_eq!(buffer.scroll_offset(), 0);
    assert!(buffer.is_at_bottom());
}

// =============================================================================
// Test 6: Page Up/Page Down Behavior
// =============================================================================

#[test]
fn test_page_up_down_behavior() {
    let mut harness = HeadlessTestHarness::new();

    let mut buffer = ScrollbackBuffer::new(200);
    let state = ScrollbackState::new(25); // 25 lines per page

    populate_scrollback(&mut buffer, 100);

    // Page up once
    buffer.scroll_up(state.lines_per_page);
    assert_eq!(buffer.scroll_offset(), 25);

    // Page up again
    buffer.scroll_up(state.lines_per_page);
    assert_eq!(buffer.scroll_offset(), 50);

    // Page down once
    buffer.scroll_down(state.lines_per_page);
    assert_eq!(buffer.scroll_offset(), 25);

    // Page down to bottom
    buffer.scroll_down(state.lines_per_page);
    assert_eq!(buffer.scroll_offset(), 0);
    assert!(buffer.is_at_bottom());
}

// =============================================================================
// Test 7: Scroll to Top/Bottom
// =============================================================================

#[test]
fn test_scroll_to_top_bottom() {
    let mut harness = HeadlessTestHarness::new();

    let mut buffer = ScrollbackBuffer::new(100);
    populate_scrollback(&mut buffer, 75);

    // Scroll somewhere in the middle
    buffer.scroll_up(30);
    assert_eq!(buffer.scroll_offset(), 30);

    // Jump to top
    buffer.scroll_to_top();
    assert_eq!(buffer.scroll_offset(), 75);
    assert!(!buffer.is_at_bottom());

    // Verify we're at the oldest line
    assert_eq!(buffer.scroll_offset(), buffer.line_count());

    // Jump to bottom
    buffer.scroll_to_bottom();
    assert_eq!(buffer.scroll_offset(), 0);
    assert!(buffer.is_at_bottom());
}

// =============================================================================
// Test 8: Scroll with Large Scrollback Buffer
// =============================================================================

#[test]
fn test_scroll_with_large_buffer() {
    let mut harness = HeadlessTestHarness::new();

    let mut buffer = ScrollbackBuffer::new(10_000);

    // Add 5000 lines
    populate_scrollback(&mut buffer, 5000);
    assert_eq!(buffer.line_count(), 5000);

    // Scroll to top
    buffer.scroll_to_top();
    assert_eq!(buffer.scroll_offset(), 5000);

    // Scroll down 100 lines
    buffer.scroll_down(100);
    assert_eq!(buffer.scroll_offset(), 4900);

    // Scroll up 500 lines (should clamp at max)
    buffer.scroll_up(500);
    assert_eq!(buffer.scroll_offset(), 5000);

    // Jump to bottom
    buffer.scroll_to_bottom();
    assert_eq!(buffer.scroll_offset(), 0);

    // Scroll percentage at various positions
    buffer.scroll_up(2500); // Middle
    let pct = scroll_percentage(&buffer);
    assert!(
        (pct - 50.0).abs() < 1.0,
        "Should be ~50% at middle of large buffer"
    );
}

// =============================================================================
// Test 9: Scroll Indicator Position Information
// =============================================================================

#[test]
fn test_scroll_indicator_position_info() {
    let mut harness = HeadlessTestHarness::new();

    let mut buffer = ScrollbackBuffer::new(100);
    populate_scrollback(&mut buffer, 80);

    // At bottom
    let info = format!("Bottom (0/80)");
    assert!(buffer.is_at_bottom());

    // Scrolled up 20 lines
    buffer.scroll_up(20);
    let info = format!("{}/{}", buffer.scroll_offset(), buffer.line_count());
    assert_eq!(info, "20/80");

    // At top
    buffer.scroll_to_top();
    let info = format!("Top ({}/{})", buffer.scroll_offset(), buffer.line_count());
    assert_eq!(info, "Top (80/80)");
}

// =============================================================================
// Test 10: Scroll Position Preservation During Buffer Growth
// =============================================================================

#[test]
fn test_scroll_position_preservation_on_growth() {
    let mut harness = HeadlessTestHarness::new();

    let mut buffer = ScrollbackBuffer::new(100);
    populate_scrollback(&mut buffer, 50);

    // Scroll up 20 lines
    buffer.scroll_up(20);
    assert_eq!(buffer.scroll_offset(), 20);

    // Add more lines (simulating new terminal output)
    for i in 50..60 {
        let line_text = format!("New line {}", i + 1);
        buffer.push_line(create_line(&line_text));
    }

    // Scroll offset should be preserved (still viewing same historical content)
    assert_eq!(buffer.scroll_offset(), 20);
    assert_eq!(buffer.line_count(), 60);
}

// =============================================================================
// Test 11: Scroll Indicator with Search Results
// =============================================================================

#[test]
fn test_scroll_indicator_with_search() {
    let mut harness = HeadlessTestHarness::new();

    let mut buffer = ScrollbackBuffer::new(100);

    // Add lines with searchable content
    buffer.push_line(create_line("The quick brown fox"));
    buffer.push_line(create_line("jumps over the lazy dog"));
    buffer.push_line(create_line("The fox is quick"));
    buffer.push_line(create_line("Another line here"));
    buffer.push_line(create_line("The fox runs fast"));

    // Search for "fox"
    buffer.search("fox".to_string(), false, false);

    let search_state = buffer.search_state().unwrap();
    assert_eq!(search_state.total_results, 3);

    // Scroll should jump to first result
    // The search should position us at the first match
    assert!(!buffer.is_at_bottom());

    // Navigate to next result
    buffer.next_search_result();
    assert_eq!(buffer.search_state().unwrap().current_index, 1);

    // Verify scroll indicator should be visible during search
    assert!(!buffer.is_at_bottom());
}

// =============================================================================
// Test 12: Scroll Indicator Bounds Checking
// =============================================================================

#[test]
fn test_scroll_indicator_bounds_checking() {
    let mut harness = HeadlessTestHarness::new();

    let mut buffer = ScrollbackBuffer::new(100);
    populate_scrollback(&mut buffer, 30);

    // Try to scroll up beyond buffer size
    buffer.scroll_up(1000);
    assert_eq!(
        buffer.scroll_offset(),
        30,
        "Should clamp to max buffer size"
    );

    // Try to scroll down beyond bottom
    buffer.scroll_down(2000);
    assert_eq!(buffer.scroll_offset(), 0, "Should clamp to 0 (bottom)");
    assert!(buffer.is_at_bottom());

    // Test with empty buffer
    let mut empty_buffer = ScrollbackBuffer::new(100);
    empty_buffer.scroll_up(10);
    assert_eq!(
        empty_buffer.scroll_offset(),
        0,
        "Empty buffer should stay at 0"
    );
    assert!(empty_buffer.is_at_bottom());
}

// =============================================================================
// Test 13: Scroll Offset After Buffer Eviction
// =============================================================================

#[test]
fn test_scroll_offset_after_eviction() {
    let mut harness = HeadlessTestHarness::new();

    // Small buffer that will evict old lines
    let mut buffer = ScrollbackBuffer::new(10);

    // Fill buffer to capacity
    populate_scrollback(&mut buffer, 10);
    assert_eq!(buffer.line_count(), 10);

    // Scroll up 5 lines
    buffer.scroll_up(5);
    assert_eq!(buffer.scroll_offset(), 5);

    // Add 5 more lines (should evict 5 oldest)
    for i in 10..15 {
        buffer.push_line(create_line(&format!("Line {}", i + 1)));
    }

    // Buffer should still have 10 lines
    assert_eq!(buffer.line_count(), 10);

    // Scroll offset should be adjusted (reduced by evicted count while maintaining view)
    assert_eq!(
        buffer.scroll_offset(),
        0,
        "Scroll offset should be reduced after eviction"
    );
}

// =============================================================================
// Test 14: Visible Lines Calculation
// =============================================================================

#[test]
fn test_visible_lines_calculation() {
    let mut harness = HeadlessTestHarness::new();

    let mut buffer = ScrollbackBuffer::new(100);
    populate_scrollback(&mut buffer, 50);

    // At bottom - no scrollback lines should be visible
    let visible = buffer.get_visible_lines(25);
    assert_eq!(visible.len(), 0, "No scrollback lines visible at bottom");

    // Scroll up 25 lines - should show 25 lines
    buffer.scroll_up(25);
    let visible = buffer.get_visible_lines(25);
    assert_eq!(visible.len(), 25, "Should show 25 scrollback lines");

    // Verify the lines are from correct range
    // When scrolled up 25, we see lines from index (50-25) to (50-25+25-1)
    assert!(visible[0].to_string().contains("Line 26"));
    assert!(visible[24].to_string().contains("Line 50"));
}

// =============================================================================
// Test 15: Scroll State Resource Integration
// =============================================================================

#[test]
fn test_scroll_state_resource() {
    let mut harness = HeadlessTestHarness::new();

    // Create state with specific lines per page
    let state = ScrollbackState::new(30);
    assert_eq!(state.lines_per_page, 30);
    assert!(!state.is_scrolled);
    assert!(!state.search_visible);
    assert_eq!(state.search_input, "");

    // Test state mutation
    let mut state = state;
    state.is_scrolled = true;
    state.search_visible = true;
    state.search_input = "test query".to_string();

    assert!(state.is_scrolled);
    assert!(state.search_visible);
    assert_eq!(state.search_input, "test query");
}

// =============================================================================
// Test 16: Scroll Indicator Text Format
// =============================================================================

#[test]
fn test_scroll_indicator_text_format() {
    let mut harness = HeadlessTestHarness::new();

    let mut buffer = ScrollbackBuffer::new(100);
    populate_scrollback(&mut buffer, 100);

    // Test different scroll positions and their display format

    // At top
    buffer.scroll_to_top();
    let indicator_text = format!("↑ {}%", scroll_percentage(&buffer) as u32);
    assert_eq!(indicator_text, "↑ 0%");

    // At 25%
    buffer.scroll_to_bottom();
    buffer.scroll_up(75);
    let indicator_text = format!("↑ {}%", scroll_percentage(&buffer) as u32);
    assert_eq!(indicator_text, "↑ 25%");

    // At 50%
    buffer.scroll_to_bottom();
    buffer.scroll_up(50);
    let indicator_text = format!("↑ {}%", scroll_percentage(&buffer) as u32);
    assert_eq!(indicator_text, "↑ 50%");

    // At 75%
    buffer.scroll_to_bottom();
    buffer.scroll_up(25);
    let indicator_text = format!("↑ {}%", scroll_percentage(&buffer) as u32);
    assert_eq!(indicator_text, "↑ 75%");

    // At bottom (indicator should be hidden, but if shown would be 100%)
    buffer.scroll_to_bottom();
    let indicator_text = format!("↓ {}%", scroll_percentage(&buffer) as u32);
    assert_eq!(indicator_text, "↓ 100%");
}

// =============================================================================
// Test 17: Scroll Performance with Rapid Updates
// =============================================================================

#[test]
fn test_scroll_performance_rapid_updates() {
    let mut harness = HeadlessTestHarness::new();

    let mut buffer = ScrollbackBuffer::new(1000);
    populate_scrollback(&mut buffer, 500);

    // Rapidly scroll up and down
    // Each iteration: up 10, down 5 = net +5
    // After 100 iterations: 500 net upward, which will clamp to 500 (buffer max)
    for _ in 0..100 {
        buffer.scroll_up(10);
        buffer.scroll_down(5);
    }

    // Should be clamped at buffer size (500 lines)
    // After iteration 100, offset should be at 500 (clamped)
    assert!(
        buffer.scroll_offset() >= 495 && buffer.scroll_offset() <= 500,
        "Offset should be near max (495-500), got {}",
        buffer.scroll_offset()
    );

    // Verify buffer integrity
    assert_eq!(buffer.line_count(), 500);

    // Test that we can't scroll beyond max
    buffer.scroll_up(1000);
    assert_eq!(buffer.scroll_offset(), 500, "Should be clamped at 500");
}

// =============================================================================
// Test 18: Scroll Indicator with Empty Buffer
// =============================================================================

#[test]
fn test_scroll_indicator_empty_buffer() {
    let mut harness = HeadlessTestHarness::new();

    let buffer = ScrollbackBuffer::new(100);

    // Empty buffer should be at bottom
    assert!(buffer.is_at_bottom());
    assert_eq!(buffer.scroll_offset(), 0);
    assert_eq!(buffer.line_count(), 0);

    // Percentage should be 100% (at bottom)
    let pct = scroll_percentage(&buffer);
    assert_eq!(pct, 100.0);
}

// =============================================================================
// Test 19: Clear Buffer Resets Scroll State
// =============================================================================

#[test]
fn test_clear_buffer_resets_scroll() {
    let mut harness = HeadlessTestHarness::new();

    let mut buffer = ScrollbackBuffer::new(100);
    populate_scrollback(&mut buffer, 50);

    // Scroll up
    buffer.scroll_up(20);
    assert_eq!(buffer.scroll_offset(), 20);
    assert!(!buffer.is_at_bottom());

    // Clear buffer
    buffer.clear();

    // Should reset to bottom
    assert_eq!(buffer.line_count(), 0);
    assert_eq!(buffer.scroll_offset(), 0);
    assert!(buffer.is_at_bottom());
}

// =============================================================================
// Test 20: Scroll Indicator Updates on New Content
// =============================================================================

#[test]
fn test_scroll_indicator_updates_on_new_content() {
    let mut harness = HeadlessTestHarness::new();

    let mut buffer = ScrollbackBuffer::new(100);
    let mut state = ScrollbackState::new(25);

    populate_scrollback(&mut buffer, 30);

    // At bottom initially
    assert!(buffer.is_at_bottom());
    state.is_scrolled = false;

    // Scroll up
    buffer.scroll_up(10);
    state.is_scrolled = !buffer.is_at_bottom();
    assert!(state.is_scrolled);

    // Add new content (simulating terminal output)
    buffer.push_line(create_line("New content arrived"));
    buffer.push_line(create_line("More new content"));

    // Still scrolled (not auto-scrolling to bottom)
    assert_eq!(buffer.scroll_offset(), 10);
    assert!(!buffer.is_at_bottom());

    // User scrolls back to bottom
    buffer.scroll_to_bottom();
    state.is_scrolled = !buffer.is_at_bottom();
    assert!(!state.is_scrolled);
}
