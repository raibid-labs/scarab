//! Integration tests for semantic zones
//!
//! These tests verify the end-to-end functionality of zone tracking,
//! simulating real terminal interaction with OSC 133 markers.

use scarab_protocol::{ZoneTracker, ZoneType};

/// Simulates a complete command execution sequence
#[test]
fn test_complete_command_sequence() {
    let mut tracker = ZoneTracker::new(100);

    // Simulate: User sees prompt
    tracker.mark_prompt_start(100, 1_000_000); // Line 100, t=1s

    // Simulate: User types command
    tracker.mark_command_start(100, 1_100_000); // t=1.1s
    tracker.set_command_text("ls -la".to_string());

    // Simulate: Command starts outputting
    tracker.mark_command_executed(101, 1_200_000); // Line 101, t=1.2s

    // Simulate: Command outputs 20 lines (101-120)

    // Simulate: Command finishes successfully
    tracker.mark_command_finished(120, 0, 2_000_000); // Line 120, t=2s, exit code 0

    // Verify command block was created
    assert_eq!(tracker.command_blocks().len(), 1);

    let block = &tracker.command_blocks()[0];
    assert_eq!(block.start_row, 100);
    assert_eq!(block.end_row, 120);
    assert!(block.is_complete());
    assert!(block.is_success());
    assert_eq!(block.command_text(), Some("ls -la"));

    // Verify duration calculation (2s - 1s = 1s)
    let duration = block.duration_secs().unwrap();
    assert!(
        (duration - 1.0).abs() < 0.0001,
        "Duration was {} but expected ~1.0",
        duration
    ); // ~1s

    // Verify zones
    assert_eq!(tracker.zones().len(), 3); // Prompt, Input, Output

    let prompt_zone = tracker
        .zones()
        .iter()
        .find(|z| z.zone_type == ZoneType::Prompt)
        .unwrap();
    assert_eq!(prompt_zone.start_row, 100);
    assert_eq!(prompt_zone.end_row, 99); // Completed before input

    let input_zone = tracker
        .zones()
        .iter()
        .find(|z| z.zone_type == ZoneType::Input)
        .unwrap();
    assert_eq!(input_zone.start_row, 100);
    assert_eq!(input_zone.end_row, 100);
    assert_eq!(input_zone.command, Some("ls -la".to_string()));

    let output_zone = tracker
        .zones()
        .iter()
        .find(|z| z.zone_type == ZoneType::Output)
        .unwrap();
    assert_eq!(output_zone.start_row, 101);
    assert_eq!(output_zone.end_row, 120);
    assert_eq!(output_zone.exit_code, Some(0));
    assert!(output_zone.is_success());
}

/// Simulates a failed command
#[test]
fn test_failed_command() {
    let mut tracker = ZoneTracker::new(100);

    tracker.mark_prompt_start(50, 1000000);
    tracker.mark_command_start(50, 1100000);
    tracker.set_command_text("ls /nonexistent".to_string());
    tracker.mark_command_executed(51, 1200000);
    tracker.mark_command_finished(52, 2, 1300000); // Exit code 2

    let block = &tracker.command_blocks()[0];
    assert!(block.is_failure());
    assert_eq!(block.exit_code(), Some(2));
    assert_eq!(block.command_text(), Some("ls /nonexistent"));

    let output_zone = tracker
        .zones()
        .iter()
        .find(|z| z.zone_type == ZoneType::Output)
        .unwrap();
    assert!(output_zone.is_failure());
    assert_eq!(output_zone.exit_code, Some(2));
}

/// Simulates multiple commands in sequence
#[test]
fn test_multiple_commands() {
    let mut tracker = ZoneTracker::new(100);

    // Command 1: echo "test"
    tracker.mark_prompt_start(10, 1000000);
    tracker.mark_command_start(10, 1100000);
    tracker.set_command_text("echo test".to_string());
    tracker.mark_command_executed(11, 1200000);
    tracker.mark_command_finished(11, 0, 1300000);

    // Command 2: pwd
    tracker.mark_prompt_start(12, 2000000);
    tracker.mark_command_start(12, 2100000);
    tracker.set_command_text("pwd".to_string());
    tracker.mark_command_executed(13, 2200000);
    tracker.mark_command_finished(13, 0, 2300000);

    // Command 3: false
    tracker.mark_prompt_start(14, 3000000);
    tracker.mark_command_start(14, 3100000);
    tracker.set_command_text("false".to_string());
    tracker.mark_command_executed(15, 3200000);
    tracker.mark_command_finished(15, 1, 3300000);

    // Verify 3 command blocks
    assert_eq!(tracker.command_blocks().len(), 3);

    // Verify first command
    assert_eq!(
        tracker.command_blocks()[0].command_text(),
        Some("echo test")
    );
    assert!(tracker.command_blocks()[0].is_success());

    // Verify second command
    assert_eq!(tracker.command_blocks()[1].command_text(), Some("pwd"));
    assert!(tracker.command_blocks()[1].is_success());

    // Verify third command
    assert_eq!(tracker.command_blocks()[2].command_text(), Some("false"));
    assert!(tracker.command_blocks()[2].is_failure());
}

/// Simulates scrolling and verifies line number adjustment
#[test]
fn test_scroll_adjustment() {
    let mut tracker = ZoneTracker::new(100);

    // Create a command block at lines 10-20
    tracker.mark_prompt_start(10, 1000000);
    tracker.mark_command_start(10, 1100000);
    tracker.mark_command_executed(11, 1200000);
    tracker.mark_command_finished(20, 0, 1300000);

    // Verify initial positions
    let block = &tracker.command_blocks()[0];
    assert_eq!(block.start_row, 10);
    assert_eq!(block.end_row, 20);

    // Simulate scrolling up by 5 lines (terminal scrolls, lines move into scrollback)
    tracker.adjust_for_scroll(5);

    // Verify adjusted positions
    let block = &tracker.command_blocks()[0];
    assert_eq!(block.start_row, 15);
    assert_eq!(block.end_row, 25);

    // Verify zones also adjusted
    let output_zone = tracker
        .zones()
        .iter()
        .find(|z| z.zone_type == ZoneType::Output)
        .unwrap();
    assert_eq!(output_zone.start_row, 16); // Was 11, now 11+5=16
    assert_eq!(output_zone.end_row, 25); // Was 20, now 20+5=25
}

/// Simulates a long-running command
#[test]
fn test_long_running_command() {
    let mut tracker = ZoneTracker::new(100);

    tracker.mark_prompt_start(100, 1_000_000); // t=1s
    tracker.mark_command_start(100, 1_100_000); // t=1.1s
    tracker.set_command_text("sleep 10".to_string());
    tracker.mark_command_executed(101, 1_200_000); // t=1.2s
                                                   // ... command runs for 10 seconds ...
    tracker.mark_command_finished(101, 0, 11_200_000); // t=11.2s

    let block = &tracker.command_blocks()[0];

    // Verify duration is ~10 seconds
    // Duration = 11.2s - 1s = 10.2s
    let duration = block.duration_secs().unwrap();
    assert!(
        (duration - 10.2).abs() < 0.01,
        "Duration was {} but expected ~10.2",
        duration
    );
}

/// Tests the last_output_zone functionality for copy operations
#[test]
fn test_last_output_zone_retrieval() {
    let mut tracker = ZoneTracker::new(100);

    // No commands yet
    assert!(tracker.last_output_zone().is_none());

    // Add first command
    tracker.mark_prompt_start(10, 1000000);
    tracker.mark_command_start(10, 1100000);
    tracker.mark_command_executed(11, 1200000);
    tracker.mark_command_finished(15, 0, 1300000);

    // Should return first output zone
    let last = tracker.last_output_zone().unwrap();
    assert_eq!(last.start_row, 11);
    assert_eq!(last.end_row, 15);

    // Add second command
    tracker.mark_prompt_start(16, 2000000);
    tracker.mark_command_start(16, 2100000);
    tracker.mark_command_executed(17, 2200000);
    tracker.mark_command_finished(20, 0, 2300000);

    // Should return second (most recent) output zone
    let last = tracker.last_output_zone().unwrap();
    assert_eq!(last.start_row, 17);
    assert_eq!(last.end_row, 20);
}

/// Tests zone history limit enforcement
#[test]
fn test_history_limit() {
    let mut tracker = ZoneTracker::new(3); // Only keep 3 command blocks

    // Create 5 command blocks
    for i in 0..5 {
        let base = (i * 10) as u32;
        let time = (i * 1000000) as u64;

        tracker.mark_prompt_start(base, time);
        tracker.mark_command_start(base, time + 100000);
        tracker.mark_command_executed(base + 1, time + 200000);
        tracker.mark_command_finished(base + 2, 0, time + 300000);
    }

    // Should only have 3 blocks (most recent)
    assert_eq!(tracker.command_blocks().len(), 3);

    // Verify it's the last 3 (blocks at lines 20, 30, 40)
    assert_eq!(tracker.command_blocks()[0].start_row, 20);
    assert_eq!(tracker.command_blocks()[1].start_row, 30);
    assert_eq!(tracker.command_blocks()[2].start_row, 40);
}

/// Tests finding zone at a specific line
#[test]
fn test_zone_lookup_by_line() {
    let mut tracker = ZoneTracker::new(100);

    tracker.mark_prompt_start(100, 1000000);
    tracker.mark_command_start(100, 1100000);
    tracker.mark_command_executed(101, 1200000);
    tracker.mark_command_finished(110, 0, 1300000);

    // Line 105 should be in the output zone
    let zone = tracker.find_zone_at_line(105).unwrap();
    assert_eq!(zone.zone_type, ZoneType::Output);
    assert_eq!(zone.start_row, 101);
    assert_eq!(zone.end_row, 110);

    // Line 95 should not be in any zone
    assert!(tracker.find_zone_at_line(95).is_none());

    // Line 115 should not be in any zone
    assert!(tracker.find_zone_at_line(115).is_none());
}

/// Tests command block lookup by line
#[test]
fn test_block_lookup_by_line() {
    let mut tracker = ZoneTracker::new(100);

    // Command 1: lines 10-15
    tracker.mark_prompt_start(10, 1000000);
    tracker.mark_command_start(10, 1100000);
    tracker.mark_command_executed(11, 1200000);
    tracker.mark_command_finished(15, 0, 1300000);

    // Command 2: lines 20-25
    tracker.mark_prompt_start(20, 2000000);
    tracker.mark_command_start(20, 2100000);
    tracker.mark_command_executed(21, 2200000);
    tracker.mark_command_finished(25, 0, 2300000);

    // Line 12 should be in first command block
    let block = tracker.find_block_at_line(12).unwrap();
    assert_eq!(block.start_row, 10);
    assert_eq!(block.end_row, 15);

    // Line 23 should be in second command block
    let block = tracker.find_block_at_line(23).unwrap();
    assert_eq!(block.start_row, 20);
    assert_eq!(block.end_row, 25);

    // Line 17 should not be in any block
    assert!(tracker.find_block_at_line(17).is_none());
}
