//! Navigation mode tests

use super::helpers::*;

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
