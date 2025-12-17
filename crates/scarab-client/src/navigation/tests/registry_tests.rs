//! NavStateRegistry isolation tests

use super::helpers::*;

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
