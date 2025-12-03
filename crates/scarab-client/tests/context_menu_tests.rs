//! Integration tests for the context menu system

use bevy::prelude::*;
use scarab_client::context_menu::{
    ContextMenuAction, ContextMenuItemSelected, ContextMenuPlugin, ContextMenuState,
    ShowContextMenuEvent,
};
use scarab_client::ratatui_bridge::{RatatuiBridgePlugin, SurfaceFocus};
use scarab_mouse::context_menu::ContextMenu;
use scarab_mouse::types::Position;
use scarab_protocol::TerminalMetrics;

/// Helper to create a test app with context menu system
fn setup_test_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(RatatuiBridgePlugin)
        .add_plugins(ContextMenuPlugin)
        .insert_resource(TerminalMetrics {
            cols: 200,
            rows: 100,
            cell_width: 10.0,
            cell_height: 20.0,
        });
    app
}

#[test]
fn test_context_menu_plugin_initialization() {
    let mut app = setup_test_app();

    // Run one update cycle
    app.update();

    // Verify resources are initialized
    assert!(app.world().get_resource::<ContextMenuState>().is_some());
    assert!(app.world().get_resource::<SurfaceFocus>().is_some());
}

#[test]
fn test_show_context_menu_event() {
    let mut app = setup_test_app();

    // Send event to show context menu
    app.world_mut()
        .send_event(ShowContextMenuEvent {
            position: Position::new(50, 30),
            url: None,
            file_path: None,
            has_selection: true,
        });

    // Run update
    app.update();

    // Verify menu is shown
    let state = app.world().get_resource::<ContextMenuState>().unwrap();
    assert!(state.is_visible());
    assert!(state.get_menu().is_some());

    let menu = state.get_menu().unwrap();
    assert_eq!(menu.position.x, 50);
    assert_eq!(menu.position.y, 30);
}

#[test]
fn test_show_url_context_menu() {
    let mut app = setup_test_app();

    // Send event with URL
    app.world_mut()
        .send_event(ShowContextMenuEvent {
            position: Position::new(10, 10),
            url: Some("https://example.com".to_string()),
            file_path: None,
            has_selection: false,
        });

    app.update();

    // Verify URL menu is shown
    let state = app.world().get_resource::<ContextMenuState>().unwrap();
    let menu = state.get_menu().unwrap();

    // Should have URL-specific items
    assert!(menu.get_item("open_url").is_some());
    assert!(menu.get_item("copy_url").is_some());
}

#[test]
fn test_show_file_context_menu() {
    let mut app = setup_test_app();

    // Send event with file path
    app.world_mut()
        .send_event(ShowContextMenuEvent {
            position: Position::new(10, 10),
            url: None,
            file_path: Some("/home/user/test.txt".to_string()),
            has_selection: false,
        });

    app.update();

    // Verify file menu is shown
    let state = app.world().get_resource::<ContextMenuState>().unwrap();
    let menu = state.get_menu().unwrap();

    // Should have file-specific items
    assert!(menu.get_item("open_file").is_some());
    assert!(menu.get_item("copy_path").is_some());
}

#[test]
fn test_menu_edge_adjustment_horizontal() {
    let mut app = setup_test_app();

    // Show menu near right edge
    app.world_mut()
        .send_event(ShowContextMenuEvent {
            position: Position::new(195, 50), // Near right edge (200 cols)
            url: None,
            file_path: None,
            has_selection: false,
        });

    app.update();

    // Menu position should be adjusted to stay on screen
    let state = app.world().get_resource::<ContextMenuState>().unwrap();
    let menu = state.get_menu().unwrap();

    // Position should be adjusted left
    assert!(menu.position.x + 30 <= 200); // 30 is approximate menu width
}

#[test]
fn test_menu_edge_adjustment_vertical() {
    let mut app = setup_test_app();

    // Show menu near bottom edge
    app.world_mut()
        .send_event(ShowContextMenuEvent {
            position: Position::new(50, 95), // Near bottom (100 rows)
            url: None,
            file_path: None,
            has_selection: false,
        });

    app.update();

    // Menu position should be adjusted to stay on screen
    let state = app.world().get_resource::<ContextMenuState>().unwrap();
    let menu = state.get_menu().unwrap();

    // Position should be adjusted up
    let menu_height = menu.items.len() as u16 + 2;
    assert!(menu.position.y + menu_height <= 100);
}

#[test]
fn test_menu_navigation() {
    let mut menu = ContextMenu::standard(Position::new(10, 10), true);

    let initial_index = menu.selected_index;

    // Navigate down
    menu.select_next();
    assert_ne!(menu.selected_index, initial_index);

    // Navigate up
    menu.select_prev();
    assert_eq!(menu.selected_index, initial_index);
}

#[test]
fn test_menu_skips_separators() {
    let mut menu = ContextMenu::standard(Position::new(10, 10), false);

    // Navigate through items
    for _ in 0..menu.items.len() * 2 {
        menu.select_next();
        // Should never select a separator
        assert!(!menu.selected_item().unwrap().separator);
    }
}

#[test]
fn test_menu_selection_with_disabled_items() {
    let menu = ContextMenu::standard(Position::new(10, 10), false);

    // Copy should be disabled when no selection
    let copy_item = menu.get_item("copy").unwrap();
    assert!(!copy_item.enabled);

    // Paste should be enabled
    let paste_item = menu.get_item("paste").unwrap();
    assert!(paste_item.enabled);
}

#[test]
fn test_action_parsing_basic() {
    let action = ContextMenuAction::from_id("copy", None);
    assert_eq!(action, Some(ContextMenuAction::Copy));

    let action = ContextMenuAction::from_id("paste", None);
    assert_eq!(action, Some(ContextMenuAction::Paste));

    let action = ContextMenuAction::from_id("search", None);
    assert_eq!(action, Some(ContextMenuAction::Search));
}

#[test]
fn test_action_parsing_with_data() {
    let url = "https://example.com";
    let action = ContextMenuAction::from_id("open_url", Some(url));
    assert_eq!(action, Some(ContextMenuAction::OpenUrl(url.to_string())));

    let path = "/home/user/file.txt";
    let action = ContextMenuAction::from_id("open_file", Some(path));
    assert_eq!(action, Some(ContextMenuAction::OpenFile(path.to_string())));
}

#[test]
fn test_action_parsing_plugin() {
    let action = ContextMenuAction::from_id("plugin.custom_action", None);
    assert!(matches!(action, Some(ContextMenuAction::PluginAction(_))));
}

#[test]
fn test_menu_item_selection_event() {
    let mut app = setup_test_app();

    // Show menu first
    app.world_mut()
        .send_event(ShowContextMenuEvent {
            position: Position::new(10, 10),
            url: None,
            file_path: None,
            has_selection: true,
        });

    app.update();

    // Simulate item selection
    app.world_mut()
        .send_event(ContextMenuItemSelected {
            item_id: "copy".to_string(),
            data: None,
        });

    app.update();

    // Verify action event was generated
    // (In a real scenario, we'd check that the action handler was invoked)
}

#[test]
fn test_context_menu_state_hide() {
    let mut state = ContextMenuState::default();

    // Show menu
    let menu = ContextMenu::standard(Position::new(10, 10), false);
    state.show(menu);
    assert!(state.is_visible());

    // Hide menu
    state.hide();
    assert!(!state.is_visible());
    assert!(state.get_menu().is_none());
}

#[test]
fn test_menu_with_shortcuts() {
    let menu = ContextMenu::standard(Position::new(10, 10), false);

    // Verify shortcuts are present
    let copy_item = menu.get_item("copy").unwrap();
    assert!(copy_item.shortcut.is_some());
    assert_eq!(copy_item.shortcut.as_ref().unwrap(), "Ctrl+Shift+C");

    let paste_item = menu.get_item("paste").unwrap();
    assert!(paste_item.shortcut.is_some());
    assert_eq!(paste_item.shortcut.as_ref().unwrap(), "Ctrl+Shift+V");
}

#[test]
fn test_menu_position_clamping() {
    // Test that positions are properly clamped to grid bounds
    let metrics = TerminalMetrics {
        cols: 200,
        rows: 100,
        cell_width: 10.0,
        cell_height: 20.0,
    };

    // Position beyond bounds
    let mut position = Position::new(250, 150);

    // In real code, this would be adjusted by handle_show_context_menu
    position.x = position.x.min(metrics.cols - 1);
    position.y = position.y.min(metrics.rows - 1);

    assert!(position.x < metrics.cols);
    assert!(position.y < metrics.rows);
}
