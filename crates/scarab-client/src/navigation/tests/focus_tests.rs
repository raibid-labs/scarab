//! Focus management tests

use super::helpers::*;

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
