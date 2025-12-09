//! Integration tests for context menu functionality

use scarab_mouse::context_menu::{ContextMenu, MenuItem};
use scarab_mouse::types::Position;

#[test]
fn test_menu_item_creation() {
    let item = MenuItem::new("test_id", "Test Label");
    assert_eq!(item.id, "test_id");
    assert_eq!(item.label, "Test Label");
    assert!(item.enabled);
    assert!(!item.separator);
    assert!(item.shortcut.is_none());
}

#[test]
fn test_menu_item_with_shortcut() {
    let item = MenuItem::new("copy", "Copy").with_shortcut("Ctrl+C");
    assert_eq!(item.shortcut, Some("Ctrl+C".to_string()));
}

#[test]
fn test_menu_item_with_enabled() {
    let item = MenuItem::new("action", "Action").with_enabled(false);
    assert!(!item.enabled);

    let item = MenuItem::new("action", "Action").with_enabled(true);
    assert!(item.enabled);
}

#[test]
fn test_menu_item_disabled() {
    let item = MenuItem::new("action", "Action").disabled();
    assert!(!item.enabled);
}

#[test]
fn test_menu_item_separator() {
    let item = MenuItem::separator();
    assert!(item.separator);
    assert!(!item.enabled);
    assert!(item.id.is_empty());
    assert!(item.label.is_empty());
}

#[test]
fn test_context_menu_creation() {
    let menu = ContextMenu::new(Position::new(10, 10));
    assert_eq!(menu.position, Position::new(10, 10));
    assert_eq!(menu.items.len(), 0);
    assert_eq!(menu.selected_index, 0);
}

#[test]
fn test_context_menu_add_item() {
    let mut menu = ContextMenu::new(Position::new(0, 0));

    menu.add_item(MenuItem::new("item1", "Item 1"));
    menu.add_item(MenuItem::new("item2", "Item 2"));

    assert_eq!(menu.items.len(), 2);
    assert_eq!(menu.items[0].id, "item1");
    assert_eq!(menu.items[1].id, "item2");
}

#[test]
fn test_standard_menu_with_selection() {
    let menu = ContextMenu::standard(Position::new(10, 10), true);

    // Should have multiple items
    assert!(menu.items.len() > 0);

    // Copy should be enabled with selection
    let copy_item = menu.get_item("copy").unwrap();
    assert!(copy_item.enabled);

    // Clear selection should be enabled with selection
    let clear_item = menu.get_item("clear_selection").unwrap();
    assert!(clear_item.enabled);
}

#[test]
fn test_standard_menu_without_selection() {
    let menu = ContextMenu::standard(Position::new(10, 10), false);

    // Copy should be disabled without selection
    let copy_item = menu.get_item("copy").unwrap();
    assert!(!copy_item.enabled);

    // Clear selection should be disabled without selection
    let clear_item = menu.get_item("clear_selection").unwrap();
    assert!(!clear_item.enabled);

    // Paste should always be enabled
    let paste_item = menu.get_item("paste").unwrap();
    assert!(paste_item.enabled);
}

#[test]
fn test_standard_menu_shortcuts() {
    let menu = ContextMenu::standard(Position::new(10, 10), true);

    let copy_item = menu.get_item("copy").unwrap();
    assert!(copy_item.shortcut.is_some());
    assert!(copy_item.shortcut.as_ref().unwrap().contains("Ctrl"));

    let paste_item = menu.get_item("paste").unwrap();
    assert!(paste_item.shortcut.is_some());
}

#[test]
fn test_standard_menu_has_separators() {
    let menu = ContextMenu::standard(Position::new(10, 10), true);

    let has_separator = menu.items.iter().any(|item| item.separator);
    assert!(has_separator);
}

#[test]
fn test_url_menu() {
    let url = "https://example.com";
    let menu = ContextMenu::url_menu(Position::new(5, 5), url.to_string());

    // Should have URL-specific items
    let open_item = menu.get_item("open_url");
    assert!(open_item.is_some());
    assert!(open_item.unwrap().label.contains(url));

    let copy_url_item = menu.get_item("copy_url");
    assert!(copy_url_item.is_some());

    // Should also have standard items
    assert!(menu.get_item("copy").is_some());
    assert!(menu.get_item("paste").is_some());
}

#[test]
fn test_file_menu() {
    let path = "/path/to/file.txt";
    let menu = ContextMenu::file_menu(Position::new(5, 5), path.to_string());

    // Should have file-specific items
    let open_item = menu.get_item("open_file");
    assert!(open_item.is_some());
    assert!(open_item.unwrap().label.contains(path));

    let copy_path_item = menu.get_item("copy_path");
    assert!(copy_path_item.is_some());

    // Should also have standard items
    assert!(menu.get_item("copy").is_some());
    assert!(menu.get_item("paste").is_some());
}

#[test]
fn test_menu_select_next() {
    let mut menu = ContextMenu::new(Position::new(0, 0));
    menu.add_item(MenuItem::new("item1", "Item 1"));
    menu.add_item(MenuItem::new("item2", "Item 2"));
    menu.add_item(MenuItem::new("item3", "Item 3"));

    assert_eq!(menu.selected_index, 0);

    menu.select_next();
    assert_eq!(menu.selected_index, 1);

    menu.select_next();
    assert_eq!(menu.selected_index, 2);

    // Should wrap around
    menu.select_next();
    assert_eq!(menu.selected_index, 0);
}

#[test]
fn test_menu_select_prev() {
    let mut menu = ContextMenu::new(Position::new(0, 0));
    menu.add_item(MenuItem::new("item1", "Item 1"));
    menu.add_item(MenuItem::new("item2", "Item 2"));
    menu.add_item(MenuItem::new("item3", "Item 3"));

    assert_eq!(menu.selected_index, 0);

    // Should wrap around backwards
    menu.select_prev();
    assert_eq!(menu.selected_index, 2);

    menu.select_prev();
    assert_eq!(menu.selected_index, 1);

    menu.select_prev();
    assert_eq!(menu.selected_index, 0);
}

#[test]
fn test_menu_navigation_skips_separators() {
    let mut menu = ContextMenu::new(Position::new(0, 0));
    menu.add_item(MenuItem::new("item1", "Item 1"));
    menu.add_item(MenuItem::separator());
    menu.add_item(MenuItem::new("item2", "Item 2"));

    assert_eq!(menu.selected_index, 0);

    menu.select_next();
    // Should skip the separator at index 1
    assert_eq!(menu.selected_index, 2);

    menu.select_prev();
    // Should skip the separator going back
    assert_eq!(menu.selected_index, 0);
}

#[test]
fn test_selected_item() {
    let mut menu = ContextMenu::new(Position::new(0, 0));
    menu.add_item(MenuItem::new("item1", "Item 1"));
    menu.add_item(MenuItem::new("item2", "Item 2"));

    let item = menu.selected_item();
    assert!(item.is_some());
    assert_eq!(item.unwrap().id, "item1");

    menu.select_next();
    let item = menu.selected_item();
    assert_eq!(item.unwrap().id, "item2");
}

#[test]
fn test_selected_item_empty_menu() {
    let menu = ContextMenu::new(Position::new(0, 0));
    let item = menu.selected_item();
    assert!(item.is_none());
}

#[test]
fn test_get_item_by_id() {
    let mut menu = ContextMenu::new(Position::new(0, 0));
    menu.add_item(MenuItem::new("copy", "Copy"));
    menu.add_item(MenuItem::new("paste", "Paste"));

    let copy_item = menu.get_item("copy");
    assert!(copy_item.is_some());
    assert_eq!(copy_item.unwrap().label, "Copy");

    let paste_item = menu.get_item("paste");
    assert!(paste_item.is_some());

    let missing_item = menu.get_item("nonexistent");
    assert!(missing_item.is_none());
}

#[test]
fn test_menu_position() {
    let pos = Position::new(42, 84);
    let menu = ContextMenu::new(pos);
    assert_eq!(menu.position, pos);
}

#[test]
fn test_menu_item_builder_pattern() {
    let item = MenuItem::new("save", "Save")
        .with_shortcut("Ctrl+S")
        .with_enabled(true);

    assert_eq!(item.id, "save");
    assert_eq!(item.label, "Save");
    assert_eq!(item.shortcut, Some("Ctrl+S".to_string()));
    assert!(item.enabled);
}

#[test]
fn test_menu_item_disabled_builder() {
    let item = MenuItem::new("action", "Action").disabled();
    assert!(!item.enabled);
}

#[test]
fn test_empty_menu_navigation() {
    let mut menu = ContextMenu::new(Position::new(0, 0));

    // Should not panic on empty menu
    menu.select_next();
    menu.select_prev();

    assert_eq!(menu.selected_index, 0);
}

#[test]
fn test_single_item_menu_navigation() {
    let mut menu = ContextMenu::new(Position::new(0, 0));
    menu.add_item(MenuItem::new("item1", "Item 1"));

    menu.select_next();
    assert_eq!(menu.selected_index, 0); // Should wrap to same item

    menu.select_prev();
    assert_eq!(menu.selected_index, 0);
}

#[test]
#[ignore = "BUG: Infinite loop when all items are separators - see select_next/select_prev impl"]
fn test_all_separators_menu() {
    let mut menu = ContextMenu::new(Position::new(0, 0));
    menu.add_item(MenuItem::separator());
    menu.add_item(MenuItem::separator());

    // Navigation should not hang even with all separators
    // BUG: This currently hangs due to infinite loop in select_next/select_prev
    menu.select_next();
    menu.select_prev();
}

#[test]
fn test_standard_menu_items_exist() {
    let menu = ContextMenu::standard(Position::new(0, 0), true);

    // Verify expected items exist
    assert!(menu.get_item("copy").is_some());
    assert!(menu.get_item("paste").is_some());
    assert!(menu.get_item("select_all").is_some());
    assert!(menu.get_item("clear_selection").is_some());
    assert!(menu.get_item("search").is_some());
    assert!(menu.get_item("new_tab").is_some());
}

#[test]
fn test_url_menu_position() {
    let pos = Position::new(15, 25);
    let menu = ContextMenu::url_menu(pos, "http://test.com".to_string());
    assert_eq!(menu.position, pos);
}

#[test]
fn test_file_menu_position() {
    let pos = Position::new(20, 30);
    let menu = ContextMenu::file_menu(pos, "/test/path".to_string());
    assert_eq!(menu.position, pos);
}

#[test]
fn test_menu_clone() {
    let menu = ContextMenu::standard(Position::new(10, 10), true);
    let cloned = menu.clone();

    assert_eq!(menu.position, cloned.position);
    assert_eq!(menu.items.len(), cloned.items.len());
    assert_eq!(menu.selected_index, cloned.selected_index);
}

#[test]
fn test_menu_item_clone() {
    let item = MenuItem::new("test", "Test").with_shortcut("Ctrl+T");
    let cloned = item.clone();

    assert_eq!(item.id, cloned.id);
    assert_eq!(item.label, cloned.label);
    assert_eq!(item.shortcut, cloned.shortcut);
    assert_eq!(item.enabled, cloned.enabled);
}

#[test]
fn test_complex_navigation_scenario() {
    let mut menu = ContextMenu::new(Position::new(0, 0));
    menu.add_item(MenuItem::new("item1", "Item 1"));
    menu.add_item(MenuItem::separator());
    menu.add_item(MenuItem::new("item2", "Item 2"));
    menu.add_item(MenuItem::new("item3", "Item 3"));
    menu.add_item(MenuItem::separator());
    menu.add_item(MenuItem::new("item4", "Item 4"));

    // Navigate through menu
    assert_eq!(menu.selected_item().unwrap().id, "item1");

    menu.select_next();
    assert_eq!(menu.selected_item().unwrap().id, "item2");

    menu.select_next();
    assert_eq!(menu.selected_item().unwrap().id, "item3");

    menu.select_next();
    assert_eq!(menu.selected_item().unwrap().id, "item4");

    menu.select_next();
    assert_eq!(menu.selected_item().unwrap().id, "item1"); // Wrapped around

    // Navigate backwards
    menu.select_prev();
    assert_eq!(menu.selected_item().unwrap().id, "item4");
}
