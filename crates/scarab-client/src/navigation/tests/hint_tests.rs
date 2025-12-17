//! Hint selection tests

use super::helpers::*;

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
