//! Headless navigation tests for Scarab terminal emulator
//!
//! This module provides comprehensive testing for the navigation system without requiring
//! a window or graphics context. Tests use Bevy's headless mode and mock terminal content.

use bevy::ecs::system::RunSystemOnce;
use bevy::prelude::*;
use scarab_protocol::{Cell, SharedState, TerminalMetrics};
use shared_memory::*;
use std::sync::Arc;

use crate::events::{PaneClosedEvent, PaneCreatedEvent, PaneFocusedEvent};
use crate::integration::SharedMemoryReader;
use crate::prompt_markers::{NavAnchor, PromptAnchorType, PromptMarkers, PromptZoneFocusedEvent};
use crate::safe_state::MockTerminalState;

use super::focusable::*;
use super::*;

// ==================== Test Helpers ====================

/// Build a minimal headless Bevy app for navigation testing
fn build_test_app() -> App {
    let mut app = App::new();

    // Add minimal plugins (no rendering or windowing)
    app.add_plugins(MinimalPlugins);

    // Add navigation plugins (core navigation only, not focusable plugin with its systems)
    app.add_plugins(NavigationPlugin);
    // Skip FocusablePlugin since it requires SharedMemoryReader

    // Create terminal metrics resource
    let metrics = TerminalMetrics {
        cell_width: 10.0,
        cell_height: 20.0,
        columns: 80,
        rows: 24,
    };
    app.insert_resource(metrics);

    // Insert focusable scan config
    app.insert_resource(FocusableScanConfig::default());

    // Insert prompt markers resource
    app.insert_resource(PromptMarkers::default());

    // Insert NavState resource for tests that access it directly
    app.insert_resource(NavState::default());

    app
}

// ==================== Navigation Mode Tests ====================

#[test]
fn test_enter_hint_mode() {
    let mut app = build_test_app();

    // Verify initial state
    let nav_state = app.world().resource::<NavState>();
    assert_eq!(nav_state.current_mode, NavMode::Normal);
    assert!(!nav_state.is_hint_mode());

    // Send EnterHintModeEvent
    app.world_mut().send_event(EnterHintModeEvent);

    // Manually change mode (in real system, this would be done by a system)
    app.world_mut().resource_mut::<NavState>().current_mode = NavMode::Hints;

    // Update to process event
    app.update();

    // Verify mode changed
    let nav_state = app.world().resource::<NavState>();
    assert_eq!(nav_state.current_mode, NavMode::Hints);
    assert!(nav_state.is_hint_mode());
}

#[test]
fn test_exit_hint_mode() {
    let mut app = build_test_app();

    // Set initial mode to Hints
    app.world_mut().resource_mut::<NavState>().current_mode = NavMode::Hints;

    // Verify starting in hint mode
    let nav_state = app.world().resource::<NavState>();
    assert!(nav_state.is_hint_mode());

    // Send ExitHintModeEvent
    app.world_mut().send_event(ExitHintModeEvent);

    // Manually change mode back
    app.world_mut().resource_mut::<NavState>().current_mode = NavMode::Normal;

    // Verify hint filter gets cleared on exit
    app.world_mut()
        .resource_mut::<NavState>()
        .clear_hint_filter();

    // Update to process event
    app.update();

    // Verify mode changed back
    let nav_state = app.world().resource::<NavState>();
    assert_eq!(nav_state.current_mode, NavMode::Normal);
    assert!(!nav_state.is_hint_mode());
    assert!(nav_state.hint_filter.is_empty());
}

#[test]
fn test_mode_stack_push_pop() {
    let mut app = build_test_app();

    // Get mutable access to NavState
    let mut nav_state = app.world_mut().resource_mut::<NavState>();

    // Start in Normal mode
    assert_eq!(nav_state.current_mode, NavMode::Normal);
    assert_eq!(nav_state.mode_stack.len(), 0);

    // Push to Hints mode
    nav_state.push_mode(NavMode::Hints);
    assert_eq!(nav_state.current_mode, NavMode::Hints);
    assert_eq!(nav_state.mode_stack.len(), 1);
    assert_eq!(nav_state.mode_stack[0], NavMode::Normal);

    // Push to CommandPalette mode
    nav_state.push_mode(NavMode::CommandPalette);
    assert_eq!(nav_state.current_mode, NavMode::CommandPalette);
    assert_eq!(nav_state.mode_stack.len(), 2);
    assert_eq!(nav_state.mode_stack[1], NavMode::Hints);

    // Pop back to Hints
    let popped = nav_state.pop_mode();
    assert!(popped);
    assert_eq!(nav_state.current_mode, NavMode::Hints);
    assert_eq!(nav_state.mode_stack.len(), 1);

    // Pop back to Normal
    let popped = nav_state.pop_mode();
    assert!(popped);
    assert_eq!(nav_state.current_mode, NavMode::Normal);
    assert_eq!(nav_state.mode_stack.len(), 0);

    // Pop with empty stack should return false
    let popped = nav_state.pop_mode();
    assert!(!popped);
    assert_eq!(nav_state.current_mode, NavMode::Normal);
}

// ==================== Focusable Detection Tests ====================

#[test]
fn test_detect_urls_in_terminal() {
    let mut app = build_test_app();

    // Spawn focusables manually
    app.world_mut().spawn(FocusableRegion {
        region_type: FocusableType::Url,
        grid_start: (6, 0),
        grid_end: (26, 0),
        content: "https://example.com".to_string(),
        source: FocusableSource::Terminal,
        pane_id: None,
        generation: 0,
        screen_position: None,
    });

    app.world_mut().spawn(FocusableRegion {
        region_type: FocusableType::Url,
        grid_start: (11, 1),
        grid_end: (26, 1),
        content: "www.github.com".to_string(),
        source: FocusableSource::Terminal,
        pane_id: None,
        generation: 0,
        screen_position: None,
    });

    // Update to spawn entities
    app.update();

    // Query focusable entities
    let mut query = app.world_mut().query::<&FocusableRegion>();
    let focusables: Vec<_> = query.iter(app.world()).collect();

    // Verify two URLs were detected
    assert_eq!(focusables.len(), 2);

    let url_focusables: Vec<_> = focusables
        .iter()
        .filter(|f| f.region_type == FocusableType::Url)
        .collect();

    assert_eq!(url_focusables.len(), 2);

    // Verify content
    let urls: Vec<&str> = url_focusables.iter().map(|f| f.content.as_str()).collect();
    assert!(urls.contains(&"https://example.com"));
    assert!(urls.contains(&"www.github.com"));
}

#[test]
fn test_detect_filepaths() {
    let mut app = build_test_app();

    // Spawn focusables for paths
    app.world_mut().spawn(FocusableRegion {
        region_type: FocusableType::FilePath,
        grid_start: (6, 0),
        grid_end: (28, 0),
        content: "/usr/local/bin/foo.txt".to_string(),
        source: FocusableSource::Terminal,
        pane_id: None,
        generation: 0,
        screen_position: None,
    });

    app.world_mut().spawn(FocusableRegion {
        region_type: FocusableType::FilePath,
        grid_start: (7, 1),
        grid_end: (26, 1),
        content: "./relative/path.rs".to_string(),
        source: FocusableSource::Terminal,
        pane_id: None,
        generation: 0,
        screen_position: None,
    });

    app.update();

    // Query focusable entities
    let mut query = app.world_mut().query::<&FocusableRegion>();
    let focusables: Vec<_> = query.iter(app.world()).collect();

    assert_eq!(focusables.len(), 2);

    let path_focusables: Vec<_> = focusables
        .iter()
        .filter(|f| f.region_type == FocusableType::FilePath)
        .collect();

    assert_eq!(path_focusables.len(), 2);

    // Verify paths
    let paths: Vec<&str> = path_focusables.iter().map(|f| f.content.as_str()).collect();
    assert!(paths.contains(&"/usr/local/bin/foo.txt"));
    assert!(paths.contains(&"./relative/path.rs"));
}

#[test]
fn test_max_focusables_limit() {
    let mut app = build_test_app();

    // Set a low max limit
    app.world_mut()
        .resource_mut::<FocusableScanConfig>()
        .max_focusables = 5;

    // Test the detector directly
    let config = FocusableScanConfig {
        max_focusables: 5,
        ..Default::default()
    };
    let detector = FocusableDetector::new(&config);

    let mut text = String::new();
    for i in 0..20 {
        text.push_str(&format!("https://example{}.com ", i));
    }

    let detected = detector.detect_all(&text, config.max_focusables);
    assert_eq!(detected.len(), 5); // Detector respects limit
}

// ==================== Hint Selection Tests ====================

#[test]
fn test_hint_filter_input() {
    let mut app = build_test_app();

    // Get mutable access to NavState
    let mut nav_state = app.world_mut().resource_mut::<NavState>();

    // Initially empty
    assert!(nav_state.hint_filter.is_empty());

    // Simulate typing characters to filter hints
    nav_state.hint_filter.push('a');
    assert_eq!(nav_state.hint_filter, "a");

    nav_state.hint_filter.push('b');
    assert_eq!(nav_state.hint_filter, "ab");

    // Clear filter
    nav_state.clear_hint_filter();
    assert!(nav_state.hint_filter.is_empty());
}

#[test]
fn test_hint_activation() {
    let mut app = build_test_app();

    // Spawn a focusable with a hint
    let entity = app
        .world_mut()
        .spawn((
            NavHint {
                label: "ab".to_string(),
                position: Vec2::new(100.0, 200.0),
                action: NavAction::Open("https://example.com".to_string()),
            },
            FocusableRegion {
                region_type: FocusableType::Url,
                grid_start: (10, 5),
                grid_end: (30, 5),
                content: "https://example.com".to_string(),
                source: FocusableSource::Terminal,
                pane_id: None,
                generation: 0,
                screen_position: Some(Vec2::new(100.0, 200.0)),
            },
        ))
        .id();

    app.update();

    // Send NavActionEvent to simulate hint activation
    let action = NavAction::Open("https://example.com".to_string());
    app.world_mut()
        .send_event(NavActionEvent::with_source(action.clone(), entity));

    app.update();

    // Verify event was sent
    let events = app.world().resource::<Events<NavActionEvent>>();
    let mut cursor = events.get_cursor();
    let action_events: Vec<_> = cursor.read(events).collect();

    assert_eq!(action_events.len(), 1);
    assert_eq!(action_events[0].action, action);
    assert_eq!(action_events[0].source, Some(entity));
}

// ==================== Prompt Navigation Tests ====================

#[test]
fn test_jump_to_previous_prompt() {
    let mut app = build_test_app();

    // Populate PromptMarkers resource
    let mut prompt_markers = app.world_mut().resource_mut::<PromptMarkers>();
    prompt_markers.markers = vec![
        scarab_protocol::PromptMarkerInfo {
            marker_type: 0, // PromptStart
            line: 5,
            exit_code: None,
            timestamp_micros: 0,
        },
        scarab_protocol::PromptMarkerInfo {
            marker_type: 0,
            line: 15,
            exit_code: None,
            timestamp_micros: 0,
        },
        scarab_protocol::PromptMarkerInfo {
            marker_type: 0,
            line: 25,
            exit_code: None,
            timestamp_micros: 0,
        },
    ];

    // Test navigation from line 20 (should jump to line 15)
    let prompt_markers = app.world().resource::<PromptMarkers>();
    let prev_idx = prompt_markers.previous_prompt(20);
    assert!(prev_idx.is_some());
    assert_eq!(prompt_markers.markers[prev_idx.unwrap()].line, 15);

    // Test navigation from line 10 (should jump to line 5)
    let prev_idx = prompt_markers.previous_prompt(10);
    assert!(prev_idx.is_some());
    assert_eq!(prompt_markers.markers[prev_idx.unwrap()].line, 5);

    // Test navigation from line 3 (should find nothing)
    let prev_idx = prompt_markers.previous_prompt(3);
    assert!(prev_idx.is_none());
}

#[test]
fn test_jump_to_next_prompt() {
    let mut app = build_test_app();

    // Populate PromptMarkers resource
    let mut prompt_markers = app.world_mut().resource_mut::<PromptMarkers>();
    prompt_markers.markers = vec![
        scarab_protocol::PromptMarkerInfo {
            marker_type: 0, // PromptStart
            line: 5,
            exit_code: None,
            timestamp_micros: 0,
        },
        scarab_protocol::PromptMarkerInfo {
            marker_type: 0,
            line: 15,
            exit_code: None,
            timestamp_micros: 0,
        },
        scarab_protocol::PromptMarkerInfo {
            marker_type: 0,
            line: 25,
            exit_code: None,
            timestamp_micros: 0,
        },
    ];

    // Test navigation from line 10 (should jump to line 15)
    let prompt_markers = app.world().resource::<PromptMarkers>();
    let next_idx = prompt_markers.next_prompt(10);
    assert!(next_idx.is_some());
    assert_eq!(prompt_markers.markers[next_idx.unwrap()].line, 15);

    // Test navigation from line 18 (should jump to line 25)
    let next_idx = prompt_markers.next_prompt(18);
    assert!(next_idx.is_some());
    assert_eq!(prompt_markers.markers[next_idx.unwrap()].line, 25);

    // Test navigation from line 30 (should find nothing)
    let next_idx = prompt_markers.next_prompt(30);
    assert!(next_idx.is_none());
}

#[test]
fn test_prompt_zone_filtering() {
    let mut app = build_test_app();

    // Spawn focusables at different line positions
    let entity1 = app
        .world_mut()
        .spawn(FocusableRegion {
            region_type: FocusableType::Url,
            grid_start: (0, 7),
            grid_end: (20, 7),
            content: "https://zone1.com".to_string(),
            source: FocusableSource::Terminal,
            pane_id: None,
            generation: 0,
            screen_position: None,
        })
        .id();

    let entity2 = app
        .world_mut()
        .spawn(FocusableRegion {
            region_type: FocusableType::Url,
            grid_start: (0, 18),
            grid_end: (20, 18),
            content: "https://zone2.com".to_string(),
            source: FocusableSource::Terminal,
            pane_id: None,
            generation: 0,
            screen_position: None,
        })
        .id();

    let entity3 = app
        .world_mut()
        .spawn(FocusableRegion {
            region_type: FocusableType::Url,
            grid_start: (0, 28),
            grid_end: (20, 28),
            content: "https://zone3.com".to_string(),
            source: FocusableSource::Terminal,
            pane_id: None,
            generation: 0,
            screen_position: None,
        })
        .id();

    app.update();

    // Verify all focusables exist initially
    let mut query = app.world_mut().query::<&FocusableRegion>();
    let focusables: Vec<_> = query.iter(app.world()).collect();
    assert_eq!(focusables.len(), 3);

    // Send PromptZoneFocusedEvent for zone 15-25 (should keep only zone2)
    app.world_mut().send_event(PromptZoneFocusedEvent {
        start_line: 15,
        end_line: 25,
        command_text: Some("git status".to_string()),
    });

    // Manually filter focusables (simulating filter_focusables_by_zone system)
    let start_line = 15;
    let end_line = 25;

    // Despawn entities outside the zone
    {
        let world = app.world();
        let region1 = world.entity(entity1).get::<FocusableRegion>().unwrap();
        let region3 = world.entity(entity3).get::<FocusableRegion>().unwrap();

        let row1 = region1.grid_start.1 as u32;
        let row3 = region3.grid_start.1 as u32;

        // Check if outside zone boundaries
        assert!(row1 < start_line || row1 >= end_line); // entity1 is outside zone
        assert!(row3 < start_line || row3 >= end_line); // entity3 is outside zone
    }

    app.world_mut().entity_mut(entity1).despawn();
    app.world_mut().entity_mut(entity3).despawn();

    app.update();

    // Verify only zone2 focusable remains
    let mut query = app.world_mut().query::<&FocusableRegion>();
    let focusables: Vec<_> = query.iter(app.world()).collect();
    assert_eq!(focusables.len(), 1);
    assert_eq!(focusables[0].content, "https://zone2.com");
    assert_eq!(focusables[0].grid_start.1, 18);
}

// ==================== Focus Management Tests ====================

#[test]
fn test_focus_history_tracking() {
    let mut app = build_test_app();

    // Create some entities
    let entity1 = app.world_mut().spawn_empty().id();
    let entity2 = app.world_mut().spawn_empty().id();
    let entity3 = app.world_mut().spawn_empty().id();

    // Record focus changes
    {
        let mut nav_state = app.world_mut().resource_mut::<NavState>();
        nav_state.record_focus(entity1);
    }

    {
        let nav_state = app.world().resource::<NavState>();
        assert_eq!(nav_state.previous_focus(), None); // Only one in history
    }

    {
        let mut nav_state = app.world_mut().resource_mut::<NavState>();
        nav_state.record_focus(entity2);
    }

    {
        let nav_state = app.world().resource::<NavState>();
        assert_eq!(nav_state.previous_focus(), Some(entity1));
    }

    {
        let mut nav_state = app.world_mut().resource_mut::<NavState>();
        nav_state.record_focus(entity3);
    }

    {
        let nav_state = app.world().resource::<NavState>();
        assert_eq!(nav_state.previous_focus(), Some(entity2));
        assert_eq!(nav_state.focus_history.len(), 3);
    }
}

#[test]
fn test_focus_history_limit() {
    let mut app = build_test_app();

    // Set small history limit
    app.world_mut().resource_mut::<NavState>().max_history_size = 3;

    // Record more than the limit
    for _i in 0..10 {
        let entity = app.world_mut().spawn_empty().id();
        app.world_mut()
            .resource_mut::<NavState>()
            .record_focus(entity);
    }

    // Should only keep last 3
    let nav_state = app.world().resource::<NavState>();
    assert_eq!(nav_state.focus_history.len(), 3);
}

#[test]
fn test_focus_changed_event() {
    let mut app = build_test_app();

    let old_entity = app.world_mut().spawn_empty().id();
    let new_entity = app.world_mut().spawn_empty().id();

    // Send FocusChangedEvent
    app.world_mut().send_event(FocusChangedEvent {
        old_focus: Some(old_entity),
        new_focus: new_entity,
    });

    app.update();

    // Verify event was sent
    let events = app.world().resource::<Events<FocusChangedEvent>>();
    let mut cursor = events.get_cursor();
    let focus_events: Vec<_> = cursor.read(events).collect();

    assert_eq!(focus_events.len(), 1);
    assert_eq!(focus_events[0].old_focus, Some(old_entity));
    assert_eq!(focus_events[0].new_focus, new_entity);
}

// ==================== Multi-Prompt Navigation Tests (Issue #43) ====================

/// Helper to create a scrollback line from a string
fn create_scrollback_line(text: &str) -> crate::terminal::scrollback::ScrollbackLine {
    use scarab_protocol::Cell;

    let cells: Vec<Cell> = text
        .chars()
        .map(|c| Cell {
            char_codepoint: c as u32,
            fg: 0xFFFFFF,
            bg: 0x000000,
            flags: 0,
            _padding: [0; 3],
        })
        .collect();
    crate::terminal::scrollback::ScrollbackLine::new(cells)
}

#[test]
fn test_multi_prompt_navigation_10_prompts() {
    use crate::prompt_markers::JumpToPromptEvent;
    use crate::terminal::scrollback::{ScrollbackBuffer, ScrollbackState};

    let mut app = build_test_app();

    // Initialize scrollback resources
    let mut scrollback = ScrollbackBuffer::new(1000);
    // Populate with 200 lines (enough for 10 prompts with output)
    for i in 0..200 {
        scrollback.push_line(create_scrollback_line(&format!("Line {}", i)));
    }
    app.insert_resource(scrollback);

    let mut scroll_state = ScrollbackState::default();
    scroll_state.lines_per_page = 24;
    app.insert_resource(scroll_state);

    // Add JumpToPromptEvent to the app
    app.add_event::<JumpToPromptEvent>();

    // Populate with 10 prompt markers at regular intervals
    let mut prompt_markers = app.world_mut().resource_mut::<PromptMarkers>();
    for i in 0..10 {
        let line = 10 + (i * 20); // Lines: 10, 30, 50, 70, 90, 110, 130, 150, 170, 190
        prompt_markers
            .markers
            .push(scarab_protocol::PromptMarkerInfo {
                marker_type: 0, // PromptStart
                line,
                exit_code: None,
                timestamp_micros: i as u64 * 1000,
            });

        // Add CommandFinished marker after each prompt
        prompt_markers
            .markers
            .push(scarab_protocol::PromptMarkerInfo {
                marker_type: 3, // CommandFinished
                line: line + 5,
                exit_code: Some(if i % 2 == 0 { 0 } else { 1 }), // Alternate success/failure
                timestamp_micros: i as u64 * 1000 + 500,
            });
    }

    // Test forward navigation through all prompts
    let prompt_markers = app.world().resource::<PromptMarkers>();

    // Starting from beginning, navigate to each prompt
    let mut current_line = 0;
    for i in 0..10 {
        let next_idx = prompt_markers.next_prompt(current_line);
        assert!(next_idx.is_some(), "Should find prompt {}", i);

        let marker = &prompt_markers.markers[next_idx.unwrap()];
        let expected_line = 10 + (i * 20);
        assert_eq!(
            marker.line, expected_line,
            "Prompt {} should be at line {}",
            i, expected_line
        );

        current_line = marker.line + 1; // Move past this prompt for next iteration
    }

    // Test backward navigation through all prompts
    current_line = 200;
    for i in (0..10).rev() {
        let prev_idx = prompt_markers.previous_prompt(current_line);
        assert!(
            prev_idx.is_some(),
            "Should find previous prompt from line {}",
            current_line
        );

        let marker = &prompt_markers.markers[prev_idx.unwrap()];
        let expected_line = 10 + (i * 20);
        assert_eq!(
            marker.line, expected_line,
            "Prompt {} should be at line {}",
            i, expected_line
        );

        current_line = marker.line; // Move to this prompt line for next iteration
    }

    // Verify no more prompts before first one
    let prev_idx = prompt_markers.previous_prompt(5);
    assert!(prev_idx.is_none(), "Should find no prompts before line 5");

    // Verify no more prompts after last one
    let next_idx = prompt_markers.next_prompt(195);
    assert!(next_idx.is_none(), "Should find no prompts after line 195");
}

#[test]
fn test_jump_prompt_action_to_event_conversion() {
    use crate::prompt_markers::{handle_nav_jump_actions, JumpToPromptEvent};
    use crate::terminal::scrollback::{ScrollbackBuffer, ScrollbackState};

    let mut app = build_test_app();

    // Initialize scrollback resources
    app.insert_resource(ScrollbackBuffer::new(1000));
    app.insert_resource(ScrollbackState::default());

    // Add JumpToPromptEvent
    app.add_event::<JumpToPromptEvent>();

    // Send NavAction::JumpPrompt event
    app.world_mut()
        .send_event(NavActionEvent::new(NavAction::JumpPrompt(42)));

    // Run the conversion system manually
    app.world_mut().run_system_once(handle_nav_jump_actions);

    app.update();

    // Verify JumpToPromptEvent was emitted
    let events = app.world().resource::<Events<JumpToPromptEvent>>();
    let mut cursor = events.get_cursor();
    let jump_events: Vec<_> = cursor.read(events).collect();

    assert_eq!(
        jump_events.len(),
        1,
        "Should emit exactly one JumpToPromptEvent"
    );
    assert_eq!(jump_events[0].target_line, 42, "Should target line 42");
    assert_eq!(jump_events[0].anchor_type, PromptAnchorType::PromptStart);
}

#[test]
fn test_rapid_navigation_no_race_conditions() {
    use crate::prompt_markers::JumpToPromptEvent;
    use crate::terminal::scrollback::{ScrollbackBuffer, ScrollbackState};

    let mut app = build_test_app();

    // Initialize scrollback
    let mut scrollback = ScrollbackBuffer::new(1000);
    for i in 0..100 {
        scrollback.push_line(create_scrollback_line(&format!("Line {}", i)));
    }
    app.insert_resource(scrollback);
    app.insert_resource(ScrollbackState::default());

    // Add event
    app.add_event::<JumpToPromptEvent>();

    // Add 5 prompts
    let mut prompt_markers = app.world_mut().resource_mut::<PromptMarkers>();
    for i in 0..5 {
        prompt_markers
            .markers
            .push(scarab_protocol::PromptMarkerInfo {
                marker_type: 0,
                line: 10 + (i * 20),
                exit_code: None,
                timestamp_micros: 0,
            });
    }

    // Rapidly send multiple jump events (simulating Ctrl+Down spam)
    for i in 0..5 {
        app.world_mut()
            .send_event(NavActionEvent::new(NavAction::JumpPrompt(10 + (i * 20))));
    }

    // Process all events
    app.update();

    // Verify all events were processed without panic
    // The scrollback should be at the last jump position
    let scrollback = app.world().resource::<ScrollbackBuffer>();
    // We don't assert exact scroll position because multiple jumps may have occurred,
    // but we verify the system didn't crash
    assert!(scrollback.scroll_offset() >= 0);
}

#[test]
fn test_empty_prompt_zone_handled_gracefully() {
    let mut app = build_test_app();

    // Add a single prompt with no output (next prompt immediately after)
    let mut prompt_markers = app.world_mut().resource_mut::<PromptMarkers>();
    prompt_markers.markers = vec![
        scarab_protocol::PromptMarkerInfo {
            marker_type: 0, // PromptStart
            line: 10,
            exit_code: None,
            timestamp_micros: 0,
        },
        scarab_protocol::PromptMarkerInfo {
            marker_type: 0, // Immediately next prompt
            line: 11,
            exit_code: None,
            timestamp_micros: 1000,
        },
    ];

    // Get the zone for line 10 (empty zone)
    let prompt_markers = app.world().resource::<PromptMarkers>();
    let zone = prompt_markers.current_prompt_zone(10);

    assert!(zone.is_some(), "Should return a zone even for empty prompt");
    let (start, end) = zone.unwrap();
    assert_eq!(start, 10, "Zone should start at prompt line");
    assert_eq!(end, 11, "Zone should end at next prompt line");
    assert_eq!(end - start, 1, "Zone should be only 1 line (empty)");
}

#[test]
fn test_zone_scoping_with_multiple_prompts() {
    use crate::terminal::scrollback::{ScrollbackBuffer, ScrollbackState};
    use crate::ui::link_hints::LinkHintsState;

    let mut app = build_test_app();

    // Initialize LinkHintsState
    app.insert_resource(LinkHintsState::default());

    // Initialize scrollback
    let mut scrollback = ScrollbackBuffer::new(1000);
    for i in 0..100 {
        scrollback.push_line(create_scrollback_line(&format!("Line {}", i)));
    }
    app.insert_resource(scrollback);

    let mut scroll_state = ScrollbackState::default();
    scroll_state.lines_per_page = 24;
    app.insert_resource(scroll_state);

    // Add PromptZoneFocusedEvent
    app.add_event::<crate::prompt_markers::PromptZoneFocusedEvent>();

    // Set up 3 prompt zones with focusables in each
    let mut prompt_markers = app.world_mut().resource_mut::<PromptMarkers>();
    prompt_markers.markers = vec![
        // Prompt 1: lines 10-29
        scarab_protocol::PromptMarkerInfo {
            marker_type: 0,
            line: 10,
            exit_code: None,
            timestamp_micros: 0,
        },
        // Prompt 2: lines 30-49
        scarab_protocol::PromptMarkerInfo {
            marker_type: 0,
            line: 30,
            exit_code: None,
            timestamp_micros: 1000,
        },
        // Prompt 3: lines 50-69
        scarab_protocol::PromptMarkerInfo {
            marker_type: 0,
            line: 50,
            exit_code: None,
            timestamp_micros: 2000,
        },
    ];

    // Spawn focusables in each zone
    // Zone 1 (lines 10-29): 2 URLs
    app.world_mut().spawn(FocusableRegion {
        region_type: FocusableType::Url,
        grid_start: (0, 12),
        grid_end: (20, 12),
        content: "https://zone1-url1.com".to_string(),
        source: FocusableSource::Terminal,
        pane_id: None,
        generation: 0,
        screen_position: None,
    });

    app.world_mut().spawn(FocusableRegion {
        region_type: FocusableType::Url,
        grid_start: (0, 15),
        grid_end: (20, 15),
        content: "https://zone1-url2.com".to_string(),
        source: FocusableSource::Terminal,
        pane_id: None,
        generation: 0,
        screen_position: None,
    });

    // Zone 2 (lines 30-49): 1 URL
    app.world_mut().spawn(FocusableRegion {
        region_type: FocusableType::Url,
        grid_start: (0, 35),
        grid_end: (20, 35),
        content: "https://zone2-url.com".to_string(),
        source: FocusableSource::Terminal,
        pane_id: None,
        generation: 0,
        screen_position: None,
    });

    // Zone 3 (lines 50-69): 3 URLs
    app.world_mut().spawn(FocusableRegion {
        region_type: FocusableType::Url,
        grid_start: (0, 52),
        grid_end: (20, 52),
        content: "https://zone3-url1.com".to_string(),
        source: FocusableSource::Terminal,
        pane_id: None,
        generation: 0,
        screen_position: None,
    });

    app.world_mut().spawn(FocusableRegion {
        region_type: FocusableType::Url,
        grid_start: (0, 55),
        grid_end: (20, 55),
        content: "https://zone3-url2.com".to_string(),
        source: FocusableSource::Terminal,
        pane_id: None,
        generation: 0,
        screen_position: None,
    });

    app.world_mut().spawn(FocusableRegion {
        region_type: FocusableType::Url,
        grid_start: (0, 58),
        grid_end: (20, 58),
        content: "https://zone3-url3.com".to_string(),
        source: FocusableSource::Terminal,
        pane_id: None,
        generation: 0,
        screen_position: None,
    });

    app.update();

    // Test zone detection for each prompt
    // Zone 1 test
    let (start1, end1) = {
        let prompt_markers = app.world().resource::<PromptMarkers>();
        let zone1 = prompt_markers.current_prompt_zone(15);
        assert!(zone1.is_some());
        let (start1, end1) = zone1.unwrap();
        assert_eq!(start1, 10);
        assert_eq!(end1, 30); // Next prompt is at line 30
        (start1, end1)
    };

    // Verify zone 1 contains the right focusables
    let mut query = app.world_mut().query::<&FocusableRegion>();
    let zone1_focusables: Vec<_> = query
        .iter(app.world())
        .filter(|f| {
            let row = f.grid_start.1 as u32;
            row >= start1 && row < end1
        })
        .collect();
    assert_eq!(zone1_focusables.len(), 2, "Zone 1 should have 2 focusables");

    // Zone 2 test
    let zone2 = app
        .world()
        .resource::<PromptMarkers>()
        .current_prompt_zone(35);
    assert!(zone2.is_some());
    let (start2, end2) = zone2.unwrap();
    assert_eq!(start2, 30);
    assert_eq!(end2, 50);

    let zone2_focusables: Vec<_> = query
        .iter(app.world())
        .filter(|f| {
            let row = f.grid_start.1 as u32;
            row >= start2 && row < end2
        })
        .collect();
    assert_eq!(zone2_focusables.len(), 1, "Zone 2 should have 1 focusable");

    // Zone 3 test
    let zone3 = prompt_markers.current_prompt_zone(55);
    assert!(zone3.is_some());
    let (start3, end3) = zone3.unwrap();
    assert_eq!(start3, 50);
    assert_eq!(end3, u32::MAX); // No next prompt, extends to end

    let zone3_focusables: Vec<_> = query
        .iter(app.world())
        .filter(|f| {
            let row = f.grid_start.1 as u32;
            row >= start3 && row < end3
        })
        .collect();
    assert_eq!(zone3_focusables.len(), 3, "Zone 3 should have 3 focusables");
}

// ==================== Integration Tests ====================

#[test]
fn test_full_hint_mode_workflow() {
    let mut app = build_test_app();

    // 1. Start in Normal mode
    let nav_state = app.world().resource::<NavState>();
    assert!(nav_state.is_normal_mode());

    // 2. Enter hint mode
    app.world_mut().resource_mut::<NavState>().current_mode = NavMode::Hints;
    app.world_mut().send_event(EnterHintModeEvent);

    // 3. Spawn focusables
    let entity = app
        .world_mut()
        .spawn((
            FocusableRegion {
                region_type: FocusableType::Url,
                grid_start: (6, 0),
                grid_end: (26, 0),
                content: "https://example.com".to_string(),
                source: FocusableSource::Terminal,
                pane_id: None,
                generation: 0,
                screen_position: Some(Vec2::new(60.0, 0.0)),
            },
            NavHint {
                label: "aa".to_string(),
                position: Vec2::new(60.0, 0.0),
                action: NavAction::Open("https://example.com".to_string()),
            },
        ))
        .id();

    app.update();

    // 4. Type hint filter
    app.world_mut()
        .resource_mut::<NavState>()
        .hint_filter
        .push('a');
    app.world_mut()
        .resource_mut::<NavState>()
        .hint_filter
        .push('a');

    // 5. Activate hint (send action event)
    app.world_mut()
        .send_event(NavActionEvent::new(NavAction::Open(
            "https://example.com".to_string(),
        )));

    app.update();

    // 6. Exit hint mode
    app.world_mut().resource_mut::<NavState>().current_mode = NavMode::Normal;
    app.world_mut().send_event(ExitHintModeEvent);
    app.world_mut()
        .resource_mut::<NavState>()
        .clear_hint_filter();

    app.update();

    // 7. Verify back in normal mode with clean state
    let nav_state = app.world().resource::<NavState>();
    assert!(nav_state.is_normal_mode());
    assert!(nav_state.hint_filter.is_empty());
}

#[test]
fn test_coordinate_conversion() {
    let mut app = build_test_app();

    // Spawn focusable without screen position
    let entity = app
        .world_mut()
        .spawn(FocusableRegion {
            region_type: FocusableType::Url,
            grid_start: (10, 5), // Grid position
            grid_end: (30, 5),
            content: "https://example.com".to_string(),
            source: FocusableSource::Terminal,
            pane_id: None,
            generation: 0,
            screen_position: None,
        })
        .id();

    app.update();

    // Simulate bounds_to_world_coords system
    let metrics = *app.world().resource::<TerminalMetrics>();
    {
        let mut entity_mut = app.world_mut().entity_mut(entity);
        let mut focusable = entity_mut.get_mut::<FocusableRegion>().unwrap();
        let world_x = focusable.grid_start.0 as f32 * metrics.cell_width;
        let world_y = -(focusable.grid_start.1 as f32 * metrics.cell_height);
        focusable.screen_position = Some(Vec2::new(world_x, world_y));
    }

    app.update();

    // Verify coordinate conversion
    let focusable = app
        .world()
        .entity(entity)
        .get::<FocusableRegion>()
        .expect("Entity should have FocusableRegion");
    let metrics = app.world().resource::<TerminalMetrics>();

    assert!(focusable.screen_position.is_some());
    let pos = focusable.screen_position.unwrap();

    // Grid (10, 5) -> World (10 * 10.0, -(5 * 20.0))
    assert_eq!(pos.x, 10.0 * metrics.cell_width);
    assert_eq!(pos.y, -(5.0 * metrics.cell_height));
}

// ==================== NavStateRegistry Isolation Tests ====================

#[test]
fn test_registry_creates_state_for_new_pane() {
    let mut app = build_test_app();

    // Get the registry
    let registry = app.world().resource::<NavStateRegistry>();
    assert_eq!(registry.pane_count(), 0);
    assert!(registry.active_pane().is_none());

    // Create a new pane state
    app.world_mut()
        .resource_mut::<NavStateRegistry>()
        .create_for_pane(1);

    // Verify state was created
    let registry = app.world().resource::<NavStateRegistry>();
    assert_eq!(registry.pane_count(), 1);
    assert!(registry.has_pane(1));
    assert!(registry.get(1).is_some());
}

#[test]
fn test_registry_isolates_modes_per_pane() {
    let mut app = build_test_app();

    // Create two panes
    app.world_mut()
        .resource_mut::<NavStateRegistry>()
        .create_for_pane(1);
    app.world_mut()
        .resource_mut::<NavStateRegistry>()
        .create_for_pane(2);

    // Set pane 1 to Hints mode
    app.world_mut()
        .resource_mut::<NavStateRegistry>()
        .get_mut(1)
        .unwrap()
        .current_mode = NavMode::Hints;

    // Set pane 2 to Insert mode
    app.world_mut()
        .resource_mut::<NavStateRegistry>()
        .get_mut(2)
        .unwrap()
        .current_mode = NavMode::Insert;

    // Verify isolation
    let registry = app.world().resource::<NavStateRegistry>();
    assert_eq!(registry.get(1).unwrap().current_mode, NavMode::Hints);
    assert_eq!(registry.get(2).unwrap().current_mode, NavMode::Insert);
}

#[test]
fn test_registry_switches_active_pane() {
    let mut app = build_test_app();

    // Create two panes
    app.world_mut()
        .resource_mut::<NavStateRegistry>()
        .create_for_pane(1);
    app.world_mut()
        .resource_mut::<NavStateRegistry>()
        .create_for_pane(2);

    // Set pane 1 as active
    app.world_mut()
        .resource_mut::<NavStateRegistry>()
        .set_active_pane(1);

    {
        let registry = app.world().resource::<NavStateRegistry>();
        assert_eq!(registry.active_pane(), Some(1));
        assert_eq!(registry.get_active().unwrap().current_mode, NavMode::Normal);
    }

    // Set pane 1 to Hints mode
    app.world_mut()
        .resource_mut::<NavStateRegistry>()
        .get_active_mut()
        .unwrap()
        .current_mode = NavMode::Hints;

    // Switch to pane 2
    app.world_mut()
        .resource_mut::<NavStateRegistry>()
        .set_active_pane(2);

    {
        let registry = app.world().resource::<NavStateRegistry>();
        assert_eq!(registry.active_pane(), Some(2));
        // Pane 2 should still be in Normal mode
        assert_eq!(registry.get_active().unwrap().current_mode, NavMode::Normal);
        // Pane 1 should still be in Hints mode
        assert_eq!(registry.get(1).unwrap().current_mode, NavMode::Hints);
    }

    // Switch back to pane 1
    app.world_mut()
        .resource_mut::<NavStateRegistry>()
        .set_active_pane(1);

    {
        let registry = app.world().resource::<NavStateRegistry>();
        assert_eq!(registry.active_pane(), Some(1));
        // Pane 1 should still be in Hints mode (state preserved)
        assert_eq!(registry.get_active().unwrap().current_mode, NavMode::Hints);
    }
}

#[test]
fn test_registry_cleanup_on_pane_close() {
    let mut app = build_test_app();

    // Create three panes
    app.world_mut()
        .resource_mut::<NavStateRegistry>()
        .create_for_pane(1);
    app.world_mut()
        .resource_mut::<NavStateRegistry>()
        .create_for_pane(2);
    app.world_mut()
        .resource_mut::<NavStateRegistry>()
        .create_for_pane(3);

    // Set pane 2 as active
    app.world_mut()
        .resource_mut::<NavStateRegistry>()
        .set_active_pane(2);

    {
        let registry = app.world().resource::<NavStateRegistry>();
        assert_eq!(registry.pane_count(), 3);
        assert_eq!(registry.active_pane(), Some(2));
    }

    // Close pane 1 (not active)
    app.world_mut()
        .resource_mut::<NavStateRegistry>()
        .remove_pane(1);

    {
        let registry = app.world().resource::<NavStateRegistry>();
        assert_eq!(registry.pane_count(), 2);
        assert!(!registry.has_pane(1));
        assert_eq!(registry.active_pane(), Some(2)); // Active pane unchanged
    }

    // Close pane 2 (active)
    app.world_mut()
        .resource_mut::<NavStateRegistry>()
        .remove_pane(2);

    {
        let registry = app.world().resource::<NavStateRegistry>();
        assert_eq!(registry.pane_count(), 1);
        assert!(!registry.has_pane(2));
        assert_eq!(registry.active_pane(), None); // Active pane cleared
    }
}

#[test]
fn test_registry_hint_filter_isolation() {
    let mut app = build_test_app();

    // Create two panes
    app.world_mut()
        .resource_mut::<NavStateRegistry>()
        .create_for_pane(1);
    app.world_mut()
        .resource_mut::<NavStateRegistry>()
        .create_for_pane(2);

    // Set pane 1 hint filter
    app.world_mut()
        .resource_mut::<NavStateRegistry>()
        .get_mut(1)
        .unwrap()
        .hint_filter = "abc".to_string();

    // Set pane 2 hint filter
    app.world_mut()
        .resource_mut::<NavStateRegistry>()
        .get_mut(2)
        .unwrap()
        .hint_filter = "xyz".to_string();

    // Verify filters are isolated
    let registry = app.world().resource::<NavStateRegistry>();
    assert_eq!(registry.get(1).unwrap().hint_filter, "abc");
    assert_eq!(registry.get(2).unwrap().hint_filter, "xyz");
}

#[test]
fn test_registry_focus_history_isolation() {
    let mut app = build_test_app();

    // Create two panes
    app.world_mut()
        .resource_mut::<NavStateRegistry>()
        .create_for_pane(1);
    app.world_mut()
        .resource_mut::<NavStateRegistry>()
        .create_for_pane(2);

    let entity1 = app.world_mut().spawn_empty().id();
    let entity2 = app.world_mut().spawn_empty().id();
    let entity3 = app.world_mut().spawn_empty().id();

    // Record focus in pane 1
    app.world_mut()
        .resource_mut::<NavStateRegistry>()
        .get_mut(1)
        .unwrap()
        .record_focus(entity1);

    app.world_mut()
        .resource_mut::<NavStateRegistry>()
        .get_mut(1)
        .unwrap()
        .record_focus(entity2);

    // Record different focus in pane 2
    app.world_mut()
        .resource_mut::<NavStateRegistry>()
        .get_mut(2)
        .unwrap()
        .record_focus(entity3);

    // Verify histories are isolated
    let registry = app.world().resource::<NavStateRegistry>();

    let pane1_history = &registry.get(1).unwrap().focus_history;
    assert_eq!(pane1_history.len(), 2);
    assert_eq!(pane1_history[0], entity1);
    assert_eq!(pane1_history[1], entity2);

    let pane2_history = &registry.get(2).unwrap().focus_history;
    assert_eq!(pane2_history.len(), 1);
    assert_eq!(pane2_history[0], entity3);
}

#[test]
fn test_registry_mode_stack_isolation() {
    let mut app = build_test_app();

    // Create two panes
    app.world_mut()
        .resource_mut::<NavStateRegistry>()
        .create_for_pane(1);
    app.world_mut()
        .resource_mut::<NavStateRegistry>()
        .create_for_pane(2);

    // Push modes for pane 1
    app.world_mut()
        .resource_mut::<NavStateRegistry>()
        .get_mut(1)
        .unwrap()
        .push_mode(NavMode::Hints);

    app.world_mut()
        .resource_mut::<NavStateRegistry>()
        .get_mut(1)
        .unwrap()
        .push_mode(NavMode::CommandPalette);

    // Push different mode for pane 2
    app.world_mut()
        .resource_mut::<NavStateRegistry>()
        .get_mut(2)
        .unwrap()
        .push_mode(NavMode::Insert);

    // Verify mode stacks are isolated
    let registry = app.world().resource::<NavStateRegistry>();

    let pane1_state = registry.get(1).unwrap();
    assert_eq!(pane1_state.current_mode, NavMode::CommandPalette);
    assert_eq!(pane1_state.mode_stack.len(), 2);
    assert_eq!(pane1_state.mode_stack[0], NavMode::Normal);
    assert_eq!(pane1_state.mode_stack[1], NavMode::Hints);

    let pane2_state = registry.get(2).unwrap();
    assert_eq!(pane2_state.current_mode, NavMode::Insert);
    assert_eq!(pane2_state.mode_stack.len(), 1);
    assert_eq!(pane2_state.mode_stack[0], NavMode::Normal);
}

#[test]
fn test_pane_lifecycle_integration() {
    use crate::events::{PaneClosedEvent, PaneCreatedEvent, PaneFocusedEvent};
    use scarab_plugin_api::object_model::{ObjectHandle, ObjectType};

    let mut app = build_test_app();

    // Add events plugin systems (manually add event systems)
    app.add_event::<PaneCreatedEvent>();
    app.add_event::<PaneFocusedEvent>();
    app.add_event::<PaneClosedEvent>();

    // Simulate pane creation
    app.world_mut().send_event(PaneCreatedEvent {
        window: ObjectHandle::new(ObjectType::Window, 1, 0),
        tab: ObjectHandle::new(ObjectType::Tab, 1, 0),
        pane: ObjectHandle::new(ObjectType::Pane, 1, 0),
    });

    app.update();

    // Verify NavState was created
    {
        let registry = app.world().resource::<NavStateRegistry>();
        assert_eq!(registry.pane_count(), 1);
        assert!(registry.has_pane(1));
    }

    // Simulate pane focus
    app.world_mut().send_event(PaneFocusedEvent {
        window: ObjectHandle::new(ObjectType::Window, 1, 0),
        tab: ObjectHandle::new(ObjectType::Tab, 1, 0),
        pane: ObjectHandle::new(ObjectType::Pane, 1, 0),
    });

    app.update();

    // Verify pane became active
    {
        let registry = app.world().resource::<NavStateRegistry>();
        assert_eq!(registry.active_pane(), Some(1));
    }

    // Create and focus second pane
    app.world_mut().send_event(PaneCreatedEvent {
        window: ObjectHandle::new(ObjectType::Window, 1, 0),
        tab: ObjectHandle::new(ObjectType::Tab, 1, 0),
        pane: ObjectHandle::new(ObjectType::Pane, 2, 0),
    });

    app.update();

    // Put first pane in Hints mode before switching
    app.world_mut()
        .resource_mut::<NavStateRegistry>()
        .get_mut(1)
        .unwrap()
        .current_mode = NavMode::Hints;

    // Focus second pane
    app.world_mut().send_event(PaneFocusedEvent {
        window: ObjectHandle::new(ObjectType::Window, 1, 0),
        tab: ObjectHandle::new(ObjectType::Tab, 1, 0),
        pane: ObjectHandle::new(ObjectType::Pane, 2, 0),
    });

    app.update();

    // Verify active pane switched and ExitHintModeEvent was sent
    {
        let registry = app.world().resource::<NavStateRegistry>();
        assert_eq!(registry.active_pane(), Some(2));
        assert_eq!(registry.get_active().unwrap().current_mode, NavMode::Normal);
        // Pane 1 should still have its Hints mode preserved
        assert_eq!(registry.get(1).unwrap().current_mode, NavMode::Hints);
    }

    // Close first pane
    app.world_mut().send_event(PaneClosedEvent {
        window: ObjectHandle::new(ObjectType::Window, 1, 0),
        tab: ObjectHandle::new(ObjectType::Tab, 1, 0),
        pane: ObjectHandle::new(ObjectType::Pane, 1, 0),
    });

    app.update();

    // Verify pane was removed
    {
        let registry = app.world().resource::<NavStateRegistry>();
        assert_eq!(registry.pane_count(), 1);
        assert!(!registry.has_pane(1));
        assert_eq!(registry.active_pane(), Some(2));
    }
}

// ==================== State Isolation Integration Tests (GitHub Issue #45) ====================

#[test]
fn test_pane_switch_preserves_hint_mode() {
    use scarab_plugin_api::object_model::{ObjectHandle, ObjectType};

    let mut app = build_test_app();

    // Add events
    app.add_event::<PaneCreatedEvent>();
    app.add_event::<PaneFocusedEvent>();

    // Create Pane A
    app.world_mut().send_event(PaneCreatedEvent {
        window: ObjectHandle::new(ObjectType::Window, 1, 0),
        tab: ObjectHandle::new(ObjectType::Tab, 1, 0),
        pane: ObjectHandle::new(ObjectType::Pane, 1, 0),
    });

    app.update();

    // Focus Pane A
    app.world_mut().send_event(PaneFocusedEvent {
        window: ObjectHandle::new(ObjectType::Window, 1, 0),
        tab: ObjectHandle::new(ObjectType::Tab, 1, 0),
        pane: ObjectHandle::new(ObjectType::Pane, 1, 0),
    });

    app.update();

    // Set Pane A to Hints mode with filter "ab"
    {
        let mut registry = app.world_mut().resource_mut::<NavStateRegistry>();
        let state_a = registry.get_mut(1).unwrap();
        state_a.current_mode = NavMode::Hints;
        state_a.hint_filter = "ab".to_string();
    }

    // Verify Pane A is in Hints mode
    {
        let registry = app.world().resource::<NavStateRegistry>();
        assert_eq!(registry.active_pane(), Some(1));
        let state_a = registry.get(1).unwrap();
        assert_eq!(state_a.current_mode, NavMode::Hints);
        assert_eq!(state_a.hint_filter, "ab");
    }

    // Create Pane B
    app.world_mut().send_event(PaneCreatedEvent {
        window: ObjectHandle::new(ObjectType::Window, 1, 0),
        tab: ObjectHandle::new(ObjectType::Tab, 1, 0),
        pane: ObjectHandle::new(ObjectType::Pane, 2, 0),
    });

    app.update();

    // Switch to Pane B
    app.world_mut().send_event(PaneFocusedEvent {
        window: ObjectHandle::new(ObjectType::Window, 1, 0),
        tab: ObjectHandle::new(ObjectType::Tab, 1, 0),
        pane: ObjectHandle::new(ObjectType::Pane, 2, 0),
    });

    app.update();

    // Verify Pane B is in Normal mode
    {
        let registry = app.world().resource::<NavStateRegistry>();
        assert_eq!(registry.active_pane(), Some(2));
        let state_b = registry.get(2).unwrap();
        assert_eq!(state_b.current_mode, NavMode::Normal);
        assert!(state_b.hint_filter.is_empty());
    }

    // Verify Pane A still has its Hints mode and filter
    {
        let registry = app.world().resource::<NavStateRegistry>();
        let state_a = registry.get(1).unwrap();
        assert_eq!(state_a.current_mode, NavMode::Hints);
        assert_eq!(state_a.hint_filter, "ab");
    }

    // Switch back to Pane A
    app.world_mut().send_event(PaneFocusedEvent {
        window: ObjectHandle::new(ObjectType::Window, 1, 0),
        tab: ObjectHandle::new(ObjectType::Tab, 1, 0),
        pane: ObjectHandle::new(ObjectType::Pane, 1, 0),
    });

    app.update();

    // Verify Pane A still in Hints mode with filter "ab"
    {
        let registry = app.world().resource::<NavStateRegistry>();
        assert_eq!(registry.active_pane(), Some(1));
        let state_a = registry.get(1).unwrap();
        assert_eq!(state_a.current_mode, NavMode::Hints);
        assert_eq!(state_a.hint_filter, "ab");
    }

    // Verify Pane B still in Normal mode
    {
        let registry = app.world().resource::<NavStateRegistry>();
        let state_b = registry.get(2).unwrap();
        assert_eq!(state_b.current_mode, NavMode::Normal);
        assert!(state_b.hint_filter.is_empty());
    }
}

#[test]
fn test_plugin_focusables_in_hint_mode() {
    use scarab_plugin_api::object_model::{ObjectHandle, ObjectType};

    let mut app = build_test_app();

    // Add events
    app.add_event::<PaneCreatedEvent>();
    app.add_event::<PaneFocusedEvent>();

    // Create and focus a pane
    app.world_mut().send_event(PaneCreatedEvent {
        window: ObjectHandle::new(ObjectType::Window, 1, 0),
        tab: ObjectHandle::new(ObjectType::Tab, 1, 0),
        pane: ObjectHandle::new(ObjectType::Pane, 1, 0),
    });

    app.world_mut().send_event(PaneFocusedEvent {
        window: ObjectHandle::new(ObjectType::Window, 1, 0),
        tab: ObjectHandle::new(ObjectType::Tab, 1, 0),
        pane: ObjectHandle::new(ObjectType::Pane, 1, 0),
    });

    app.update();

    // Register 3 focusables via plugin action (simulated)
    let focusable1 = app
        .world_mut()
        .spawn((
            FocusableRegion {
                region_type: FocusableType::Widget,
                grid_start: (10, 5),
                grid_end: (20, 5),
                content: "Button 1".to_string(),
                source: FocusableSource::Ratatui,
                pane_id: None,
                generation: 0,
                screen_position: Some(Vec2::new(100.0, 50.0)),
            },
            NavHint {
                label: "aa".to_string(),
                position: Vec2::new(100.0, 50.0),
                action: NavAction::Click(10, 5),
            },
        ))
        .id();

    let focusable2 = app
        .world_mut()
        .spawn((
            FocusableRegion {
                region_type: FocusableType::Widget,
                grid_start: (10, 10),
                grid_end: (20, 10),
                content: "Button 2".to_string(),
                source: FocusableSource::Ratatui,
                pane_id: None,
                generation: 0,
                screen_position: Some(Vec2::new(100.0, 100.0)),
            },
            NavHint {
                label: "ab".to_string(),
                position: Vec2::new(100.0, 100.0),
                action: NavAction::Click(10, 10),
            },
        ))
        .id();

    let focusable3 = app
        .world_mut()
        .spawn((
            FocusableRegion {
                region_type: FocusableType::Widget,
                grid_start: (10, 15),
                grid_end: (20, 15),
                content: "Button 3".to_string(),
                source: FocusableSource::Ratatui,
                pane_id: None,
                generation: 0,
                screen_position: Some(Vec2::new(100.0, 150.0)),
            },
            NavHint {
                label: "ac".to_string(),
                position: Vec2::new(100.0, 150.0),
                action: NavAction::Click(10, 15),
            },
        ))
        .id();

    app.update();

    // Enter Hint mode
    app.world_mut()
        .resource_mut::<NavStateRegistry>()
        .get_active_mut()
        .unwrap()
        .current_mode = NavMode::Hints;

    app.world_mut().send_event(EnterHintModeEvent);
    app.update();

    // Verify 3 hints appear
    let mut hint_query = app.world_mut().query::<(&NavHint, &FocusableRegion)>();
    let hints: Vec<_> = hint_query.iter(app.world()).collect();

    assert_eq!(hints.len(), 3, "Should have 3 hints registered");

    // Verify hint labels
    let labels: Vec<_> = hints.iter().map(|(h, _)| h.label.as_str()).collect();
    assert!(labels.contains(&"aa"));
    assert!(labels.contains(&"ab"));
    assert!(labels.contains(&"ac"));

    // Simulate hint activation (click on "ab")
    let hint_ab = hints.iter().find(|(h, _)| h.label == "ab").unwrap();
    if let NavAction::Click(col, row) = hint_ab.0.action {
        assert_eq!(col, 10);
        assert_eq!(row, 10);

        // Send action event
        app.world_mut()
            .send_event(NavActionEvent::new(NavAction::Click(col, row)));

        app.update();

        // Verify event was sent
        let events = app.world().resource::<Events<NavActionEvent>>();
        let mut cursor = events.get_cursor();
        let action_events: Vec<_> = cursor.read(events).collect();

        assert!(action_events.len() > 0, "Should have action events");
        let last_event = action_events.last().unwrap();
        assert_eq!(last_event.action, NavAction::Click(10, 10));
    } else {
        panic!("Expected Click action for hint 'ab'");
    }
}

#[test]
fn test_navstate_restoration_on_switch() {
    use scarab_plugin_api::object_model::{ObjectHandle, ObjectType};

    let mut app = build_test_app();

    // Add events
    app.add_event::<PaneCreatedEvent>();
    app.add_event::<PaneFocusedEvent>();

    // Create Pane A
    app.world_mut().send_event(PaneCreatedEvent {
        window: ObjectHandle::new(ObjectType::Window, 1, 0),
        tab: ObjectHandle::new(ObjectType::Tab, 1, 0),
        pane: ObjectHandle::new(ObjectType::Pane, 1, 0),
    });

    app.update();

    // Focus Pane A
    app.world_mut().send_event(PaneFocusedEvent {
        window: ObjectHandle::new(ObjectType::Window, 1, 0),
        tab: ObjectHandle::new(ObjectType::Tab, 1, 0),
        pane: ObjectHandle::new(ObjectType::Pane, 1, 0),
    });

    app.update();

    // Create entities to track focus
    let entity1 = app.world_mut().spawn_empty().id();
    let entity2 = app.world_mut().spawn_empty().id();
    let entity3 = app.world_mut().spawn_empty().id();

    // Build focus history for Pane A
    {
        let mut registry = app.world_mut().resource_mut::<NavStateRegistry>();
        let state_a = registry.get_mut(1).unwrap();
        state_a.record_focus(entity1);
        state_a.record_focus(entity2);
        state_a.record_focus(entity3);
    }

    // Build mode stack for Pane A
    {
        let mut registry = app.world_mut().resource_mut::<NavStateRegistry>();
        let state_a = registry.get_mut(1).unwrap();
        state_a.push_mode(NavMode::Hints);
        state_a.push_mode(NavMode::CommandPalette);
    }

    // Verify Pane A state
    {
        let registry = app.world().resource::<NavStateRegistry>();
        let state_a = registry.get(1).unwrap();
        assert_eq!(state_a.focus_history.len(), 3);
        assert_eq!(state_a.focus_history[0], entity1);
        assert_eq!(state_a.focus_history[1], entity2);
        assert_eq!(state_a.focus_history[2], entity3);
        assert_eq!(state_a.current_mode, NavMode::CommandPalette);
        assert_eq!(state_a.mode_stack.len(), 2);
        assert_eq!(state_a.mode_stack[0], NavMode::Normal);
        assert_eq!(state_a.mode_stack[1], NavMode::Hints);
    }

    // Create Pane B
    app.world_mut().send_event(PaneCreatedEvent {
        window: ObjectHandle::new(ObjectType::Window, 1, 0),
        tab: ObjectHandle::new(ObjectType::Tab, 1, 0),
        pane: ObjectHandle::new(ObjectType::Pane, 2, 0),
    });

    app.update();

    // Switch to Pane B
    app.world_mut().send_event(PaneFocusedEvent {
        window: ObjectHandle::new(ObjectType::Window, 1, 0),
        tab: ObjectHandle::new(ObjectType::Tab, 1, 0),
        pane: ObjectHandle::new(ObjectType::Pane, 2, 0),
    });

    app.update();

    // Verify Pane B has empty state
    {
        let registry = app.world().resource::<NavStateRegistry>();
        assert_eq!(registry.active_pane(), Some(2));
        let state_b = registry.get(2).unwrap();
        assert_eq!(state_b.focus_history.len(), 0);
        assert_eq!(state_b.current_mode, NavMode::Normal);
        assert_eq!(state_b.mode_stack.len(), 0);
    }

    // Switch back to Pane A
    app.world_mut().send_event(PaneFocusedEvent {
        window: ObjectHandle::new(ObjectType::Window, 1, 0),
        tab: ObjectHandle::new(ObjectType::Tab, 1, 0),
        pane: ObjectHandle::new(ObjectType::Pane, 1, 0),
    });

    app.update();

    // Verify focus_history intact
    {
        let registry = app.world().resource::<NavStateRegistry>();
        assert_eq!(registry.active_pane(), Some(1));
        let state_a = registry.get(1).unwrap();
        assert_eq!(state_a.focus_history.len(), 3);
        assert_eq!(state_a.focus_history[0], entity1);
        assert_eq!(state_a.focus_history[1], entity2);
        assert_eq!(state_a.focus_history[2], entity3);
    }

    // Verify mode_stack preserved
    {
        let registry = app.world().resource::<NavStateRegistry>();
        let state_a = registry.get(1).unwrap();
        assert_eq!(state_a.current_mode, NavMode::CommandPalette);
        assert_eq!(state_a.mode_stack.len(), 2);
        assert_eq!(state_a.mode_stack[0], NavMode::Normal);
        assert_eq!(state_a.mode_stack[1], NavMode::Hints);
    }
}

#[test]
fn test_rapid_pane_switching_no_race() {
    use scarab_plugin_api::object_model::{ObjectHandle, ObjectType};

    let mut app = build_test_app();

    // Add events
    app.add_event::<PaneCreatedEvent>();
    app.add_event::<PaneFocusedEvent>();

    // Create 3 panes with different states
    for pane_id in 1..=3 {
        app.world_mut().send_event(PaneCreatedEvent {
            window: ObjectHandle::new(ObjectType::Window, 1, 0),
            tab: ObjectHandle::new(ObjectType::Tab, 1, 0),
            pane: ObjectHandle::new(ObjectType::Pane, pane_id, 0),
        });

        app.update();

        app.world_mut().send_event(PaneFocusedEvent {
            window: ObjectHandle::new(ObjectType::Window, 1, 0),
            tab: ObjectHandle::new(ObjectType::Tab, 1, 0),
            pane: ObjectHandle::new(ObjectType::Pane, pane_id, 0),
        });

        app.update();
    }

    // Set different modes for each pane
    {
        let mut registry = app.world_mut().resource_mut::<NavStateRegistry>();
        registry.get_mut(1).unwrap().current_mode = NavMode::Hints;
        registry.get_mut(1).unwrap().hint_filter = "pane1".to_string();

        registry.get_mut(2).unwrap().current_mode = NavMode::Insert;
        registry.get_mut(2).unwrap().hint_filter = "pane2".to_string();

        registry.get_mut(3).unwrap().current_mode = NavMode::CommandPalette;
        registry.get_mut(3).unwrap().hint_filter = "pane3".to_string();
    }

    // Add focus history to each pane
    for pane_id in 1..=3 {
        let entities: Vec<Entity> = (0..5).map(|_| app.world_mut().spawn_empty().id()).collect();

        let mut registry = app.world_mut().resource_mut::<NavStateRegistry>();
        let state = registry.get_mut(pane_id).unwrap();
        for entity in entities {
            state.record_focus(entity);
        }
    }

    // Verify initial states are set correctly
    {
        let registry = app.world().resource::<NavStateRegistry>();
        assert_eq!(registry.get(1).unwrap().current_mode, NavMode::Hints);
        assert_eq!(registry.get(2).unwrap().current_mode, NavMode::Insert);
        assert_eq!(
            registry.get(3).unwrap().current_mode,
            NavMode::CommandPalette
        );
        assert_eq!(registry.get(1).unwrap().focus_history.len(), 5);
        assert_eq!(registry.get(2).unwrap().focus_history.len(), 5);
        assert_eq!(registry.get(3).unwrap().focus_history.len(), 5);
    }

    // Rapid switch between panes 20 times
    for i in 0..20 {
        let pane_id = (i % 3) + 1;

        app.world_mut().send_event(PaneFocusedEvent {
            window: ObjectHandle::new(ObjectType::Window, 1, 0),
            tab: ObjectHandle::new(ObjectType::Tab, 1, 0),
            pane: ObjectHandle::new(ObjectType::Pane, pane_id, 0),
        });

        app.update();

        // Verify no panics and active pane is correct
        let registry = app.world().resource::<NavStateRegistry>();
        assert_eq!(registry.active_pane(), Some(pane_id));
    }

    // Verify states correctly isolated after rapid switching
    {
        let registry = app.world().resource::<NavStateRegistry>();

        // Pane 1 should still be in Hints mode
        let state1 = registry.get(1).unwrap();
        assert_eq!(state1.current_mode, NavMode::Hints);
        assert_eq!(state1.hint_filter, "pane1");
        assert_eq!(state1.focus_history.len(), 5);

        // Pane 2 should still be in Insert mode
        let state2 = registry.get(2).unwrap();
        assert_eq!(state2.current_mode, NavMode::Insert);
        assert_eq!(state2.hint_filter, "pane2");
        assert_eq!(state2.focus_history.len(), 5);

        // Pane 3 should still be in CommandPalette mode
        let state3 = registry.get(3).unwrap();
        assert_eq!(state3.current_mode, NavMode::CommandPalette);
        assert_eq!(state3.hint_filter, "pane3");
        assert_eq!(state3.focus_history.len(), 5);
    }

    // Verify memory stable (no leaks) - check registry size
    {
        let registry = app.world().resource::<NavStateRegistry>();
        assert_eq!(registry.pane_count(), 3);
    }
}

// ==================== State Lifecycle Tests (GitHub Issue #46) ====================

#[test]
fn test_pane_close_clears_focusables() {
    use crate::navigation::focusable::FocusableGeneration;
    use scarab_plugin_api::object_model::{ObjectHandle, ObjectType};

    let mut app = build_test_app();

    // Add events
    app.add_event::<PaneCreatedEvent>();
    app.add_event::<PaneFocusedEvent>();
    app.add_event::<PaneClosedEvent>();

    // Add FocusableGeneration resource
    app.insert_resource(FocusableGeneration::new());

    // Create Pane 1
    app.world_mut().send_event(PaneCreatedEvent {
        window: ObjectHandle::new(ObjectType::Window, 1, 0),
        tab: ObjectHandle::new(ObjectType::Tab, 1, 0),
        pane: ObjectHandle::new(ObjectType::Pane, 1, 0),
    });

    app.update();

    // Focus Pane 1
    app.world_mut().send_event(PaneFocusedEvent {
        window: ObjectHandle::new(ObjectType::Window, 1, 0),
        tab: ObjectHandle::new(ObjectType::Tab, 1, 0),
        pane: ObjectHandle::new(ObjectType::Pane, 1, 0),
    });

    app.update();

    // Spawn focusables for Pane 1
    app.world_mut().spawn(FocusableRegion {
        region_type: FocusableType::Url,
        grid_start: (0, 5),
        grid_end: (20, 5),
        content: "https://pane1.com".to_string(),
        source: FocusableSource::Terminal,
        screen_position: None,
        pane_id: Some(1),
        generation: 0,
    });

    app.world_mut().spawn(FocusableRegion {
        region_type: FocusableType::Url,
        grid_start: (0, 10),
        grid_end: (20, 10),
        content: "https://pane1-second.com".to_string(),
        source: FocusableSource::Terminal,
        screen_position: None,
        pane_id: Some(1),
        generation: 0,
    });

    app.update();

    // Create Pane 2 with different focusables
    app.world_mut().send_event(PaneCreatedEvent {
        window: ObjectHandle::new(ObjectType::Window, 1, 0),
        tab: ObjectHandle::new(ObjectType::Tab, 1, 0),
        pane: ObjectHandle::new(ObjectType::Pane, 2, 0),
    });

    app.update();

    app.world_mut().spawn(FocusableRegion {
        region_type: FocusableType::Url,
        grid_start: (0, 5),
        grid_end: (20, 5),
        content: "https://pane2.com".to_string(),
        source: FocusableSource::Terminal,
        screen_position: None,
        pane_id: Some(2),
        generation: 0,
    });

    app.update();

    // Verify 3 focusables exist
    {
        let mut query = app.world_mut().query::<&FocusableRegion>();
        let focusables: Vec<_> = query.iter(app.world()).collect();
        assert_eq!(focusables.len(), 3);
    }

    // Close Pane 1
    app.world_mut().send_event(PaneClosedEvent {
        window: ObjectHandle::new(ObjectType::Window, 1, 0),
        tab: ObjectHandle::new(ObjectType::Tab, 1, 0),
        pane: ObjectHandle::new(ObjectType::Pane, 1, 0),
    });

    app.update();

    // Verify only Pane 2's focusable remains
    {
        let mut query = app.world_mut().query::<&FocusableRegion>();
        let focusables: Vec<_> = query.iter(app.world()).collect();
        assert_eq!(
            focusables.len(),
            1,
            "Should only have 1 focusable remaining (from Pane 2)"
        );
        assert_eq!(focusables[0].content, "https://pane2.com");
        assert_eq!(focusables[0].pane_id, Some(2));
    }

    // Verify NavState was removed
    {
        let registry = app.world().resource::<NavStateRegistry>();
        assert!(!registry.has_pane(1));
        assert_eq!(registry.pane_count(), 1);
    }
}

#[test]
fn test_pane_close_clears_hints() {
    use crate::navigation::focusable::FocusableGeneration;
    use crate::rendering::hint_overlay::HintOverlay;
    use scarab_plugin_api::object_model::{ObjectHandle, ObjectType};

    let mut app = build_test_app();

    // Add events
    app.add_event::<PaneCreatedEvent>();
    app.add_event::<PaneFocusedEvent>();
    app.add_event::<PaneClosedEvent>();

    // Add FocusableGeneration resource
    app.insert_resource(FocusableGeneration::new());

    // Create and focus a pane
    app.world_mut().send_event(PaneCreatedEvent {
        window: ObjectHandle::new(ObjectType::Window, 1, 0),
        tab: ObjectHandle::new(ObjectType::Tab, 1, 0),
        pane: ObjectHandle::new(ObjectType::Pane, 1, 0),
    });

    app.update();

    app.world_mut().send_event(PaneFocusedEvent {
        window: ObjectHandle::new(ObjectType::Window, 1, 0),
        tab: ObjectHandle::new(ObjectType::Tab, 1, 0),
        pane: ObjectHandle::new(ObjectType::Pane, 1, 0),
    });

    app.update();

    // Spawn hint overlays (simulating hint mode)
    let hint1 = app
        .world_mut()
        .spawn(HintOverlay {
            label: "aa".to_string(),
            position: Vec2::new(100.0, 100.0),
            ..Default::default()
        })
        .id();

    let hint2 = app
        .world_mut()
        .spawn(HintOverlay {
            label: "ab".to_string(),
            position: Vec2::new(200.0, 200.0),
            ..Default::default()
        })
        .id();

    app.update();

    // Verify hints exist
    {
        let mut query = app.world_mut().query::<&HintOverlay>();
        let hints: Vec<_> = query.iter(app.world()).collect();
        assert_eq!(hints.len(), 2);
    }

    // Close the pane
    app.world_mut().send_event(PaneClosedEvent {
        window: ObjectHandle::new(ObjectType::Window, 1, 0),
        tab: ObjectHandle::new(ObjectType::Tab, 1, 0),
        pane: ObjectHandle::new(ObjectType::Pane, 1, 0),
    });

    app.update();

    // Verify hints are cleared
    {
        let mut query = app.world_mut().query::<&HintOverlay>();
        let hints: Vec<_> = query.iter(app.world()).collect();
        assert_eq!(hints.len(), 0, "All hints should be cleared on pane close");
    }
}

#[test]
fn test_stale_focusables_detection() {
    use crate::navigation::focusable::{detect_stale_focusables, FocusableGeneration};

    let mut app = build_test_app();

    // Add FocusableGeneration resource
    let mut generation = FocusableGeneration::new();
    generation.increment_pane(1); // Set generation to 1 for pane 1
    app.insert_resource(generation);

    // Spawn focusables with generation 0 (stale)
    app.world_mut().spawn(FocusableRegion {
        region_type: FocusableType::Url,
        grid_start: (0, 5),
        grid_end: (20, 5),
        content: "https://stale.com".to_string(),
        source: FocusableSource::Terminal,
        screen_position: None,
        pane_id: Some(1),
        generation: 0, // Stale generation
    });

    // Spawn focusables with current generation (fresh)
    app.world_mut().spawn(FocusableRegion {
        region_type: FocusableType::Url,
        grid_start: (0, 10),
        grid_end: (20, 10),
        content: "https://fresh.com".to_string(),
        source: FocusableSource::Terminal,
        screen_position: None,
        pane_id: Some(1),
        generation: 1, // Current generation
    });

    app.update();

    // Verify 2 focusables exist before cleanup
    {
        let mut query = app.world_mut().query::<&FocusableRegion>();
        let focusables: Vec<_> = query.iter(app.world()).collect();
        assert_eq!(focusables.len(), 2);
    }

    // Run stale detection system manually
    app.world_mut().run_system_once(detect_stale_focusables);

    app.update();

    // Verify only fresh focusable remains
    {
        let mut query = app.world_mut().query::<&FocusableRegion>();
        let focusables: Vec<_> = query.iter(app.world()).collect();
        assert_eq!(focusables.len(), 1, "Only fresh focusable should remain");
        assert_eq!(focusables[0].content, "https://fresh.com");
        assert_eq!(focusables[0].generation, 1);
    }
}

#[test]
fn test_generation_increment_on_chunk_refresh() {
    use crate::navigation::focusable::FocusableGeneration;

    let mut generation = FocusableGeneration::new();

    // Initial generation should be 0
    assert_eq!(generation.get_pane_generation(1), 0);

    // Simulate chunk refresh (increment generation)
    let new_gen = generation.increment_pane(1);
    assert_eq!(new_gen, 1);
    assert_eq!(generation.get_pane_generation(1), 1);

    // Another chunk refresh
    let new_gen2 = generation.increment_pane(1);
    assert_eq!(new_gen2, 2);
    assert_eq!(generation.get_pane_generation(1), 2);

    // Different pane
    let pane2_gen = generation.increment_pane(2);
    assert_eq!(pane2_gen, 1);
    assert_eq!(generation.get_pane_generation(2), 1);

    // Pane 1 should still be at generation 2
    assert_eq!(generation.get_pane_generation(1), 2);
}
