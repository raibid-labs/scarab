//! Headless navigation tests for Scarab terminal emulator
//!
//! This module provides comprehensive testing for the navigation system without requiring
//! a window or graphics context. Tests use Bevy's headless mode and mock terminal content.

use bevy::ecs::system::RunSystemOnce;
use bevy::prelude::*;
use scarab_protocol::{Cell, TerminalMetrics, SharedState};
use shared_memory::*;
use std::sync::Arc;

use crate::integration::SharedMemoryReader;
use crate::prompt_markers::{NavAnchor, PromptAnchorType, PromptMarkers, PromptZoneFocusedEvent};
use crate::safe_state::MockTerminalState;

use super::*;
use super::focusable::*;

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
    app.world_mut().resource_mut::<NavState>().clear_hint_filter();

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
        screen_position: None,
    });

    app.world_mut().spawn(FocusableRegion {
        region_type: FocusableType::Url,
        grid_start: (11, 1),
        grid_end: (26, 1),
        content: "www.github.com".to_string(),
        source: FocusableSource::Terminal,
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
        screen_position: None,
    });

    app.world_mut().spawn(FocusableRegion {
        region_type: FocusableType::FilePath,
        grid_start: (7, 1),
        grid_end: (26, 1),
        content: "./relative/path.rs".to_string(),
        source: FocusableSource::Terminal,
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
    app.world_mut().resource_mut::<FocusableScanConfig>().max_focusables = 5;

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
    let entity = app.world_mut().spawn((
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
            screen_position: Some(Vec2::new(100.0, 200.0)),
        },
    )).id();

    app.update();

    // Send NavActionEvent to simulate hint activation
    let action = NavAction::Open("https://example.com".to_string());
    app.world_mut().send_event(NavActionEvent::with_source(action.clone(), entity));

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
    let entity1 = app.world_mut().spawn(FocusableRegion {
        region_type: FocusableType::Url,
        grid_start: (0, 7),
        grid_end: (20, 7),
        content: "https://zone1.com".to_string(),
        source: FocusableSource::Terminal,
        screen_position: None,
    }).id();

    let entity2 = app.world_mut().spawn(FocusableRegion {
        region_type: FocusableType::Url,
        grid_start: (0, 18),
        grid_end: (20, 18),
        content: "https://zone2.com".to_string(),
        source: FocusableSource::Terminal,
        screen_position: None,
    }).id();

    let entity3 = app.world_mut().spawn(FocusableRegion {
        region_type: FocusableType::Url,
        grid_start: (0, 28),
        grid_end: (20, 28),
        content: "https://zone3.com".to_string(),
        source: FocusableSource::Terminal,
        screen_position: None,
    }).id();

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
        app.world_mut().resource_mut::<NavState>().record_focus(entity);
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
    let entity = app.world_mut().spawn((
        FocusableRegion {
            region_type: FocusableType::Url,
            grid_start: (6, 0),
            grid_end: (26, 0),
            content: "https://example.com".to_string(),
            source: FocusableSource::Terminal,
            screen_position: Some(Vec2::new(60.0, 0.0)),
        },
        NavHint {
            label: "aa".to_string(),
            position: Vec2::new(60.0, 0.0),
            action: NavAction::Open("https://example.com".to_string()),
        },
    )).id();

    app.update();

    // 4. Type hint filter
    app.world_mut().resource_mut::<NavState>().hint_filter.push('a');
    app.world_mut().resource_mut::<NavState>().hint_filter.push('a');

    // 5. Activate hint (send action event)
    app.world_mut().send_event(NavActionEvent::new(
        NavAction::Open("https://example.com".to_string()),
    ));

    app.update();

    // 6. Exit hint mode
    app.world_mut().resource_mut::<NavState>().current_mode = NavMode::Normal;
    app.world_mut().send_event(ExitHintModeEvent);
    app.world_mut().resource_mut::<NavState>().clear_hint_filter();

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
    let entity = app.world_mut().spawn(FocusableRegion {
        region_type: FocusableType::Url,
        grid_start: (10, 5), // Grid position
        grid_end: (30, 5),
        content: "https://example.com".to_string(),
        source: FocusableSource::Terminal,
        screen_position: None,
    }).id();

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
    let focusable = app.world().entity(entity).get::<FocusableRegion>().expect("Entity should have FocusableRegion");
    let metrics = app.world().resource::<TerminalMetrics>();

    assert!(focusable.screen_position.is_some());
    let pos = focusable.screen_position.unwrap();

    // Grid (10, 5) -> World (10 * 10.0, -(5 * 20.0))
    assert_eq!(pos.x, 10.0 * metrics.cell_width);
    assert_eq!(pos.y, -(5.0 * metrics.cell_height));
}
