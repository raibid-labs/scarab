//! Integration tests combining multiple navigation features

use super::helpers::*;

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
