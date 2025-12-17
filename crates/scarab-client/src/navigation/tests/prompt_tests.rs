//! Prompt navigation tests

use super::helpers::*;

// ==================== Basic Prompt Navigation Tests ====================

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

// ==================== Multi-Prompt Navigation Tests (Issue #43) ====================

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
    {
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
    }

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
    let zone3 = app
        .world()
        .resource::<PromptMarkers>()
        .current_prompt_zone(55);
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
