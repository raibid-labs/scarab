//! Navigation integration tests for Scarab terminal emulator
//!
//! Tests the navigation system's mode handling, focusable detection,
//! and action event processing.

use bevy::prelude::*;
use scarab_client::events::{PaneClosedEvent, PaneCreatedEvent, PaneFocusedEvent};
use scarab_client::navigation::{
    EnterHintModeEvent, ExitHintModeEvent, FocusChangedEvent, FocusableGeneration, NavAction,
    NavActionEvent, NavMode, NavStateRegistry, NavigationPlugin,
};

/// Helper to create a minimal test app with navigation
fn create_test_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);

    // Add required events that NavigationPlugin depends on
    app.add_event::<PaneCreatedEvent>();
    app.add_event::<PaneFocusedEvent>();
    app.add_event::<PaneClosedEvent>();

    // Add required resources
    app.insert_resource(FocusableGeneration::new());

    app.add_plugins(NavigationPlugin);
    app
}

#[test]
fn test_nav_state_registry_creation() {
    let mut app = create_test_app();
    app.update();

    // Registry should exist
    let registry = app.world().resource::<NavStateRegistry>();
    assert!(
        registry.active_pane().is_none(),
        "No active pane initially"
    );
}

#[test]
fn test_mode_stack_push_pop() {
    let mut app = create_test_app();
    app.update();

    // Create a pane state manually for testing
    {
        let mut registry = app.world_mut().resource_mut::<NavStateRegistry>();
        registry.create_for_pane(1);
        registry.set_active_pane(1);
    }

    app.update();

    // Verify initial state is Normal
    {
        let registry = app.world().resource::<NavStateRegistry>();
        let state = registry.get_active().expect("Should have active state");
        assert_eq!(state.current_mode, NavMode::Normal);
    }

    // Push Hints mode
    {
        let mut registry = app.world_mut().resource_mut::<NavStateRegistry>();
        if let Some(state) = registry.get_active_mut() {
            state.push_mode(NavMode::Hints);
        }
    }

    // Verify Hints mode
    {
        let registry = app.world().resource::<NavStateRegistry>();
        let state = registry.get_active().expect("Should have active state");
        assert_eq!(state.current_mode, NavMode::Hints);
    }

    // Pop back to Normal
    {
        let mut registry = app.world_mut().resource_mut::<NavStateRegistry>();
        if let Some(state) = registry.get_active_mut() {
            state.pop_mode();
        }
    }

    // Verify back to Normal
    {
        let registry = app.world().resource::<NavStateRegistry>();
        let state = registry.get_active().expect("Should have active state");
        assert_eq!(state.current_mode, NavMode::Normal);
    }
}

#[test]
fn test_pane_isolation() {
    let mut app = create_test_app();
    app.update();

    // Create two panes
    {
        let mut registry = app.world_mut().resource_mut::<NavStateRegistry>();
        registry.create_for_pane(1);
        registry.create_for_pane(2);
        registry.set_active_pane(1);
    }

    // Set pane 1 to Hints mode
    {
        let mut registry = app.world_mut().resource_mut::<NavStateRegistry>();
        if let Some(state) = registry.get_active_mut() {
            state.push_mode(NavMode::Hints);
        }
    }

    // Switch to pane 2
    {
        let mut registry = app.world_mut().resource_mut::<NavStateRegistry>();
        registry.set_active_pane(2);
    }

    // Pane 2 should be in Normal mode
    {
        let registry = app.world().resource::<NavStateRegistry>();
        let state = registry.get_active().expect("Should have active state");
        assert_eq!(
            state.current_mode,
            NavMode::Normal,
            "Pane 2 should be Normal"
        );
    }

    // Switch back to pane 1
    {
        let mut registry = app.world_mut().resource_mut::<NavStateRegistry>();
        registry.set_active_pane(1);
    }

    // Pane 1 should still be in Hints mode
    {
        let registry = app.world().resource::<NavStateRegistry>();
        let state = registry.get_active().expect("Should have active state");
        assert_eq!(
            state.current_mode,
            NavMode::Hints,
            "Pane 1 should still be Hints"
        );
    }
}

#[test]
fn test_nav_action_event_defined() {
    let mut app = create_test_app();
    app.update();

    // Send a NavActionEvent
    app.world_mut().send_event(NavActionEvent {
        action: NavAction::Cancel,
        source: None,
        timestamp: std::time::Instant::now(),
    });

    app.update();

    // Event should be processed without panic
    // (The handler logs but doesn't crash)
}

#[test]
fn test_enter_exit_hint_mode_events() {
    let mut app = create_test_app();
    app.update();

    // Send EnterHintModeEvent
    app.world_mut().send_event(EnterHintModeEvent);
    app.update();

    // Event should be registered and processed
    let events = app.world().resource::<Events<EnterHintModeEvent>>();
    let mut cursor = events.get_cursor();
    let enter_events: Vec<_> = cursor.read(events).collect();
    assert_eq!(
        enter_events.len(),
        1,
        "Should have one EnterHintModeEvent"
    );

    // Send ExitHintModeEvent
    app.world_mut().send_event(ExitHintModeEvent);
    app.update();

    // Event should be registered and processed
    let events = app.world().resource::<Events<ExitHintModeEvent>>();
    let mut cursor = events.get_cursor();
    let exit_events: Vec<_> = cursor.read(events).collect();
    assert_eq!(exit_events.len(), 1, "Should have one ExitHintModeEvent");
}

#[test]
fn test_focus_changed_event() {
    let mut app = create_test_app();
    app.update();

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

#[test]
fn test_navigation_action_variants() {
    let mut app = create_test_app();
    app.update();

    // Test all NavAction variants
    let actions = vec![
        NavAction::Open("https://example.com".to_string()),
        NavAction::Click(10, 20),
        NavAction::JumpPrompt(42),
        NavAction::NextPane,
        NavAction::PrevPane,
        NavAction::NextTab,
        NavAction::PrevTab,
        NavAction::Cancel,
    ];

    for action in actions {
        app.world_mut()
            .send_event(NavActionEvent::new(action.clone()));
    }

    app.update();

    // All events should be processed without panic
    let events = app.world().resource::<Events<NavActionEvent>>();
    let mut cursor = events.get_cursor();
    let action_events: Vec<_> = cursor.read(events).collect();

    assert_eq!(action_events.len(), 8, "Should have 8 action events");
}

#[test]
fn test_registry_pane_management() {
    let mut app = create_test_app();
    app.update();

    let mut registry = app.world_mut().resource_mut::<NavStateRegistry>();

    // Initially empty
    assert_eq!(registry.pane_count(), 0);
    assert!(!registry.has_pane(1));

    // Create pane
    registry.create_for_pane(1);
    assert_eq!(registry.pane_count(), 1);
    assert!(registry.has_pane(1));
    assert!(registry.get(1).is_some());

    // Create another pane
    registry.create_for_pane(2);
    assert_eq!(registry.pane_count(), 2);
    assert!(registry.has_pane(2));

    // Remove pane
    registry.remove_pane(1);
    assert_eq!(registry.pane_count(), 1);
    assert!(!registry.has_pane(1));
    assert!(registry.has_pane(2));
}

#[test]
fn test_registry_active_pane_tracking() {
    let mut app = create_test_app();
    app.update();

    let mut registry = app.world_mut().resource_mut::<NavStateRegistry>();

    // No active pane initially
    assert!(registry.active_pane().is_none());
    assert!(registry.get_active().is_none());

    // Set active pane (auto-creates if needed)
    registry.set_active_pane(1);
    assert_eq!(registry.active_pane(), Some(1));
    assert!(registry.get_active().is_some());

    // Switch active pane
    registry.set_active_pane(2);
    assert_eq!(registry.active_pane(), Some(2));
    assert!(registry.get_active().is_some());

    // Remove active pane
    registry.remove_pane(2);
    assert!(registry.active_pane().is_none());
}

#[test]
fn test_mode_persistence_across_pane_switches() {
    let mut app = create_test_app();
    app.update();

    {
        let mut registry = app.world_mut().resource_mut::<NavStateRegistry>();
        registry.create_for_pane(1);
        registry.create_for_pane(2);
        registry.create_for_pane(3);
    }

    // Set different modes for each pane
    {
        let mut registry = app.world_mut().resource_mut::<NavStateRegistry>();
        registry.get_mut(1).unwrap().current_mode = NavMode::Normal;
        registry.get_mut(2).unwrap().current_mode = NavMode::Hints;
        registry.get_mut(3).unwrap().current_mode = NavMode::Insert;
    }

    // Switch between panes and verify mode persistence
    {
        let mut registry = app.world_mut().resource_mut::<NavStateRegistry>();
        registry.set_active_pane(1);
        assert_eq!(registry.get_active().unwrap().current_mode, NavMode::Normal);

        registry.set_active_pane(2);
        assert_eq!(registry.get_active().unwrap().current_mode, NavMode::Hints);

        registry.set_active_pane(3);
        assert_eq!(registry.get_active().unwrap().current_mode, NavMode::Insert);

        registry.set_active_pane(1);
        assert_eq!(registry.get_active().unwrap().current_mode, NavMode::Normal);
    }
}

#[test]
fn test_mode_stack_operations() {
    let mut app = create_test_app();
    app.update();

    {
        let mut registry = app.world_mut().resource_mut::<NavStateRegistry>();
        registry.create_for_pane(1);
        registry.set_active_pane(1);
    }

    // Test mode stack operations
    {
        let mut registry = app.world_mut().resource_mut::<NavStateRegistry>();
        let state = registry.get_active_mut().unwrap();

        // Initial state
        assert_eq!(state.current_mode, NavMode::Normal);
        assert_eq!(state.mode_stack.len(), 0);

        // Push modes
        state.push_mode(NavMode::Hints);
        assert_eq!(state.current_mode, NavMode::Hints);
        assert_eq!(state.mode_stack.len(), 1);

        state.push_mode(NavMode::CommandPalette);
        assert_eq!(state.current_mode, NavMode::CommandPalette);
        assert_eq!(state.mode_stack.len(), 2);

        // Pop modes
        assert!(state.pop_mode());
        assert_eq!(state.current_mode, NavMode::Hints);
        assert_eq!(state.mode_stack.len(), 1);

        assert!(state.pop_mode());
        assert_eq!(state.current_mode, NavMode::Normal);
        assert_eq!(state.mode_stack.len(), 0);

        // Pop from empty stack
        assert!(!state.pop_mode());
        assert_eq!(state.current_mode, NavMode::Normal);
    }
}

#[test]
fn test_focus_history_tracking() {
    let mut app = create_test_app();
    app.update();

    {
        let mut registry = app.world_mut().resource_mut::<NavStateRegistry>();
        registry.create_for_pane(1);
        registry.set_active_pane(1);
    }

    let entity1 = app.world_mut().spawn_empty().id();
    let entity2 = app.world_mut().spawn_empty().id();
    let entity3 = app.world_mut().spawn_empty().id();

    // Record focus changes
    {
        let mut registry = app.world_mut().resource_mut::<NavStateRegistry>();
        let state = registry.get_active_mut().unwrap();

        state.record_focus(entity1);
        assert_eq!(state.previous_focus(), None); // Only one in history

        state.record_focus(entity2);
        assert_eq!(state.previous_focus(), Some(entity1));

        state.record_focus(entity3);
        assert_eq!(state.previous_focus(), Some(entity2));
    }
}

#[test]
fn test_hint_filter_operations() {
    let mut app = create_test_app();
    app.update();

    {
        let mut registry = app.world_mut().resource_mut::<NavStateRegistry>();
        registry.create_for_pane(1);
        registry.set_active_pane(1);
    }

    {
        let mut registry = app.world_mut().resource_mut::<NavStateRegistry>();
        let state = registry.get_active_mut().unwrap();

        // Initially empty
        assert!(state.hint_filter.is_empty());

        // Add to filter
        state.hint_filter.push('a');
        state.hint_filter.push('b');
        assert_eq!(state.hint_filter, "ab");

        // Clear filter
        state.clear_hint_filter();
        assert!(state.hint_filter.is_empty());
    }
}

#[test]
fn test_system_set_ordering() {
    let mut app = create_test_app();

    // Verify that NavSystemSet is configured correctly
    // The sets should be: Input -> Update -> Render (chained)
    app.update();

    // If the plugin registered the sets correctly, this should not panic
    // We can't directly test ordering without running systems, but we can
    // verify the app doesn't crash when updating with the configured sets
}

#[test]
fn test_nav_action_event_with_source() {
    let mut app = create_test_app();
    app.update();

    let entity = app.world_mut().spawn_empty().id();

    // Test event creation with source
    let action = NavAction::Open("https://test.com".to_string());
    let event = NavActionEvent::with_source(action.clone(), entity);

    assert_eq!(event.action, action);
    assert_eq!(event.source, Some(entity));

    // Send the event
    app.world_mut().send_event(event);
    app.update();

    // Verify event was processed
    let events = app.world().resource::<Events<NavActionEvent>>();
    let mut cursor = events.get_cursor();
    let action_events: Vec<_> = cursor.read(events).collect();

    assert_eq!(action_events.len(), 1);
    assert_eq!(action_events[0].source, Some(entity));
}

#[test]
fn test_multiple_panes_independent_states() {
    let mut app = create_test_app();
    app.update();

    // Create 3 panes
    {
        let mut registry = app.world_mut().resource_mut::<NavStateRegistry>();
        for i in 1..=3 {
            registry.create_for_pane(i);
        }
    }

    // Set different states for each pane
    {
        let mut registry = app.world_mut().resource_mut::<NavStateRegistry>();

        let state1 = registry.get_mut(1).unwrap();
        state1.current_mode = NavMode::Hints;
        state1.hint_filter = "filter1".to_string();

        let state2 = registry.get_mut(2).unwrap();
        state2.current_mode = NavMode::Insert;
        state2.hint_filter = "filter2".to_string();

        let state3 = registry.get_mut(3).unwrap();
        state3.current_mode = NavMode::CommandPalette;
        state3.hint_filter = "filter3".to_string();
    }

    // Verify independence
    {
        let registry = app.world().resource::<NavStateRegistry>();

        let state1 = registry.get(1).unwrap();
        assert_eq!(state1.current_mode, NavMode::Hints);
        assert_eq!(state1.hint_filter, "filter1");

        let state2 = registry.get(2).unwrap();
        assert_eq!(state2.current_mode, NavMode::Insert);
        assert_eq!(state2.hint_filter, "filter2");

        let state3 = registry.get(3).unwrap();
        assert_eq!(state3.current_mode, NavMode::CommandPalette);
        assert_eq!(state3.hint_filter, "filter3");
    }
}

#[test]
fn test_nav_mode_helper_methods() {
    let mut app = create_test_app();
    app.update();

    {
        let mut registry = app.world_mut().resource_mut::<NavStateRegistry>();
        registry.create_for_pane(1);
        registry.set_active_pane(1);
    }

    {
        let mut registry = app.world_mut().resource_mut::<NavStateRegistry>();
        let state = registry.get_active_mut().unwrap();

        // Test Normal mode
        state.current_mode = NavMode::Normal;
        assert!(state.is_normal_mode());
        assert!(!state.is_hint_mode());
        assert!(!state.is_insert_mode());
        assert!(!state.is_command_palette_mode());

        // Test Hints mode
        state.current_mode = NavMode::Hints;
        assert!(!state.is_normal_mode());
        assert!(state.is_hint_mode());
        assert!(!state.is_insert_mode());
        assert!(!state.is_command_palette_mode());

        // Test Insert mode
        state.current_mode = NavMode::Insert;
        assert!(!state.is_normal_mode());
        assert!(!state.is_hint_mode());
        assert!(state.is_insert_mode());
        assert!(!state.is_command_palette_mode());

        // Test CommandPalette mode
        state.current_mode = NavMode::CommandPalette;
        assert!(!state.is_normal_mode());
        assert!(!state.is_hint_mode());
        assert!(!state.is_insert_mode());
        assert!(state.is_command_palette_mode());
    }
}

// ==================== URL Detection and Opening Tests ====================

#[test]
fn test_url_open_action() {
    let mut app = create_test_app();
    app.update();

    // Send URL open action
    let url = "https://github.com/raibid-labs/scarab".to_string();
    app.world_mut()
        .send_event(NavActionEvent::new(NavAction::Open(url.clone())));

    app.update();

    // Verify event was processed
    let events = app.world().resource::<Events<NavActionEvent>>();
    let mut cursor = events.get_cursor();
    let action_events: Vec<_> = cursor.read(events).collect();

    assert_eq!(action_events.len(), 1);
    match &action_events[0].action {
        NavAction::Open(opened_url) => assert_eq!(opened_url, &url),
        _ => panic!("Expected Open action"),
    }
}

#[test]
fn test_file_path_open_action() {
    let mut app = create_test_app();
    app.update();

    // Send file path open action
    let path = "/home/user/code/src/main.rs".to_string();
    app.world_mut()
        .send_event(NavActionEvent::new(NavAction::Open(path.clone())));

    app.update();

    // Verify event was processed
    let events = app.world().resource::<Events<NavActionEvent>>();
    let mut cursor = events.get_cursor();
    let action_events: Vec<_> = cursor.read(events).collect();

    assert_eq!(action_events.len(), 1);
    match &action_events[0].action {
        NavAction::Open(opened_path) => assert_eq!(opened_path, &path),
        _ => panic!("Expected Open action"),
    }
}

// ==================== Hint Mode Entry/Exit Tests ====================

#[test]
fn test_hint_mode_entry() {
    let mut app = create_test_app();
    app.update();

    {
        let mut registry = app.world_mut().resource_mut::<NavStateRegistry>();
        registry.create_for_pane(1);
        registry.set_active_pane(1);
    }

    // Send EnterHintModeEvent
    app.world_mut().send_event(EnterHintModeEvent);
    app.update();

    // Verify event was sent
    let events = app.world().resource::<Events<EnterHintModeEvent>>();
    let mut cursor = events.get_cursor();
    let enter_events: Vec<_> = cursor.read(events).collect();
    assert_eq!(enter_events.len(), 1);
}

#[test]
fn test_hint_mode_exit() {
    let mut app = create_test_app();
    app.update();

    {
        let mut registry = app.world_mut().resource_mut::<NavStateRegistry>();
        registry.create_for_pane(1);
        registry.set_active_pane(1);
    }

    // Send ExitHintModeEvent
    app.world_mut().send_event(ExitHintModeEvent);
    app.update();

    // Verify event was sent
    let events = app.world().resource::<Events<ExitHintModeEvent>>();
    let mut cursor = events.get_cursor();
    let exit_events: Vec<_> = cursor.read(events).collect();
    assert_eq!(exit_events.len(), 1);
}

// ==================== Pane Navigation Tests ====================

#[test]
fn test_next_pane_action() {
    let mut app = create_test_app();
    app.update();

    app.world_mut()
        .send_event(NavActionEvent::new(NavAction::NextPane));

    app.update();

    let events = app.world().resource::<Events<NavActionEvent>>();
    let mut cursor = events.get_cursor();
    let action_events: Vec<_> = cursor.read(events).collect();

    assert_eq!(action_events.len(), 1);
    assert_eq!(action_events[0].action, NavAction::NextPane);
}

#[test]
fn test_prev_pane_action() {
    let mut app = create_test_app();
    app.update();

    app.world_mut()
        .send_event(NavActionEvent::new(NavAction::PrevPane));

    app.update();

    let events = app.world().resource::<Events<NavActionEvent>>();
    let mut cursor = events.get_cursor();
    let action_events: Vec<_> = cursor.read(events).collect();

    assert_eq!(action_events.len(), 1);
    assert_eq!(action_events[0].action, NavAction::PrevPane);
}

// ==================== Tab Navigation Tests ====================

#[test]
fn test_next_tab_action() {
    let mut app = create_test_app();
    app.update();

    app.world_mut()
        .send_event(NavActionEvent::new(NavAction::NextTab));

    app.update();

    let events = app.world().resource::<Events<NavActionEvent>>();
    let mut cursor = events.get_cursor();
    let action_events: Vec<_> = cursor.read(events).collect();

    assert_eq!(action_events.len(), 1);
    assert_eq!(action_events[0].action, NavAction::NextTab);
}

#[test]
fn test_prev_tab_action() {
    let mut app = create_test_app();
    app.update();

    app.world_mut()
        .send_event(NavActionEvent::new(NavAction::PrevTab));

    app.update();

    let events = app.world().resource::<Events<NavActionEvent>>();
    let mut cursor = events.get_cursor();
    let action_events: Vec<_> = cursor.read(events).collect();

    assert_eq!(action_events.len(), 1);
    assert_eq!(action_events[0].action, NavAction::PrevTab);
}

// ==================== Prompt Jumping Tests ====================

#[test]
fn test_jump_prompt_action() {
    let mut app = create_test_app();
    app.update();

    let target_line = 42;
    app.world_mut()
        .send_event(NavActionEvent::new(NavAction::JumpPrompt(target_line)));

    app.update();

    let events = app.world().resource::<Events<NavActionEvent>>();
    let mut cursor = events.get_cursor();
    let action_events: Vec<_> = cursor.read(events).collect();

    assert_eq!(action_events.len(), 1);
    match action_events[0].action {
        NavAction::JumpPrompt(line) => assert_eq!(line, target_line),
        _ => panic!("Expected JumpPrompt action"),
    }
}

// ==================== Focus Tracking Tests ====================

#[test]
fn test_focus_tracking_across_pane_switches() {
    let mut app = create_test_app();
    app.update();

    // Create three panes
    {
        let mut registry = app.world_mut().resource_mut::<NavStateRegistry>();
        for i in 1..=3 {
            registry.create_for_pane(i);
        }
    }

    // Create entities to track focus
    let entity1 = app.world_mut().spawn_empty().id();
    let entity2 = app.world_mut().spawn_empty().id();
    let entity3 = app.world_mut().spawn_empty().id();

    // Set different focus for each pane
    {
        let mut registry = app.world_mut().resource_mut::<NavStateRegistry>();
        registry.get_mut(1).unwrap().record_focus(entity1);
        registry.get_mut(2).unwrap().record_focus(entity2);
        registry.get_mut(3).unwrap().record_focus(entity3);
    }

    // Switch between panes and verify focus is tracked correctly
    {
        let mut registry = app.world_mut().resource_mut::<NavStateRegistry>();

        registry.set_active_pane(1);
        assert_eq!(
            registry.get_active().unwrap().focus_history.last(),
            Some(&entity1)
        );

        registry.set_active_pane(2);
        assert_eq!(
            registry.get_active().unwrap().focus_history.last(),
            Some(&entity2)
        );

        registry.set_active_pane(3);
        assert_eq!(
            registry.get_active().unwrap().focus_history.last(),
            Some(&entity3)
        );
    }
}

// ==================== Pane Lifecycle Tests ====================

#[test]
fn test_pane_created_event_processing() {
    use scarab_plugin_api::object_model::{ObjectHandle, ObjectType};

    let mut app = create_test_app();
    app.update();

    // Send PaneCreatedEvent
    app.world_mut().send_event(PaneCreatedEvent {
        window: ObjectHandle::new(ObjectType::Window, 1, 0),
        tab: ObjectHandle::new(ObjectType::Tab, 1, 0),
        pane: ObjectHandle::new(ObjectType::Pane, 1, 0),
    });

    app.update();

    // Verify NavState was created
    let registry = app.world().resource::<NavStateRegistry>();
    assert!(registry.has_pane(1), "Pane 1 should have NavState");
    assert_eq!(registry.pane_count(), 1);
}

#[test]
fn test_pane_focused_event_processing() {
    use scarab_plugin_api::object_model::{ObjectHandle, ObjectType};

    let mut app = create_test_app();
    app.update();

    // Create pane first
    app.world_mut().send_event(PaneCreatedEvent {
        window: ObjectHandle::new(ObjectType::Window, 1, 0),
        tab: ObjectHandle::new(ObjectType::Tab, 1, 0),
        pane: ObjectHandle::new(ObjectType::Pane, 1, 0),
    });

    app.update();

    // Send PaneFocusedEvent
    app.world_mut().send_event(PaneFocusedEvent {
        window: ObjectHandle::new(ObjectType::Window, 1, 0),
        tab: ObjectHandle::new(ObjectType::Tab, 1, 0),
        pane: ObjectHandle::new(ObjectType::Pane, 1, 0),
    });

    app.update();

    // Verify pane became active
    let registry = app.world().resource::<NavStateRegistry>();
    assert_eq!(registry.active_pane(), Some(1));
}

#[test]
fn test_pane_closed_event_processing() {
    use scarab_plugin_api::object_model::{ObjectHandle, ObjectType};

    let mut app = create_test_app();
    app.update();

    // Create and focus pane
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

    // Close the pane
    app.world_mut().send_event(PaneClosedEvent {
        window: ObjectHandle::new(ObjectType::Window, 1, 0),
        tab: ObjectHandle::new(ObjectType::Tab, 1, 0),
        pane: ObjectHandle::new(ObjectType::Pane, 1, 0),
    });

    app.update();

    // Verify pane was removed
    let registry = app.world().resource::<NavStateRegistry>();
    assert!(!registry.has_pane(1), "Pane 1 should be removed");
    assert_eq!(registry.pane_count(), 0);
    assert_eq!(registry.active_pane(), None);
}

// ==================== Click Action Tests ====================

#[test]
fn test_click_action() {
    let mut app = create_test_app();
    app.update();

    let col = 10;
    let row = 5;
    app.world_mut()
        .send_event(NavActionEvent::new(NavAction::Click(col, row)));

    app.update();

    let events = app.world().resource::<Events<NavActionEvent>>();
    let mut cursor = events.get_cursor();
    let action_events: Vec<_> = cursor.read(events).collect();

    assert_eq!(action_events.len(), 1);
    match action_events[0].action {
        NavAction::Click(c, r) => {
            assert_eq!(c, col);
            assert_eq!(r, row);
        }
        _ => panic!("Expected Click action"),
    }
}

// ==================== Cancel Action Tests ====================

#[test]
fn test_cancel_action() {
    let mut app = create_test_app();
    app.update();

    app.world_mut()
        .send_event(NavActionEvent::new(NavAction::Cancel));

    app.update();

    let events = app.world().resource::<Events<NavActionEvent>>();
    let mut cursor = events.get_cursor();
    let action_events: Vec<_> = cursor.read(events).collect();

    assert_eq!(action_events.len(), 1);
    assert_eq!(action_events[0].action, NavAction::Cancel);
}
