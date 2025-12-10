//! Tests for pane creation and splitting operations

use scarab_panes::{PanesPlugin, SplitDirection};
use scarab_plugin_api::Plugin;

#[test]
fn test_initial_pane_creation() {
    let plugin = PanesPlugin::with_size(80, 24);
    // Plugin should start with one pane that takes full terminal size
    // We can't directly access internal state, so we test via plugin behavior
    // This is a basic smoke test
    assert_eq!(plugin.metadata().name, "scarab-panes");
}

#[test]
fn test_initial_pane_with_various_sizes() {
    // Test various terminal sizes
    let sizes = vec![
        (80, 24),   // Standard terminal
        (120, 40),  // Large terminal
        (40, 10),   // Small terminal
        (200, 100), // Very large terminal
        (1, 1),     // Minimal terminal
    ];

    for (cols, rows) in sizes {
        let plugin = PanesPlugin::with_size(cols, rows);
        assert_eq!(plugin.metadata().name, "scarab-panes");
    }
}

#[test]
fn test_plugin_metadata() {
    let plugin = PanesPlugin::new();
    let metadata = plugin.metadata();

    assert_eq!(metadata.name, "scarab-panes");
    assert_eq!(metadata.version, "0.1.0");
    assert!(!metadata.description.is_empty());
    assert!(!metadata.author.is_empty());
}

#[test]
fn test_plugin_commands() {
    let plugin = PanesPlugin::new();
    let commands = plugin.get_commands();

    // Should have commands for all operations
    assert!(!commands.is_empty());

    // Check for essential commands
    let command_ids: Vec<String> = commands.iter().map(|c| c.id.clone()).collect();

    assert!(command_ids.contains(&"panes.split_horizontal".to_string()));
    assert!(command_ids.contains(&"panes.split_vertical".to_string()));
    assert!(command_ids.contains(&"panes.close".to_string()));
    assert!(command_ids.contains(&"panes.navigate_up".to_string()));
    assert!(command_ids.contains(&"panes.navigate_down".to_string()));
    assert!(command_ids.contains(&"panes.navigate_left".to_string()));
    assert!(command_ids.contains(&"panes.navigate_right".to_string()));
    assert!(command_ids.contains(&"panes.resize_up".to_string()));
    assert!(command_ids.contains(&"panes.resize_down".to_string()));
    assert!(command_ids.contains(&"panes.resize_left".to_string()));
    assert!(command_ids.contains(&"panes.resize_right".to_string()));
    assert!(command_ids.contains(&"panes.zoom".to_string()));
}

#[test]
fn test_all_commands_have_descriptions() {
    let plugin = PanesPlugin::new();
    let commands = plugin.get_commands();

    for command in commands {
        assert!(
            !command.label.is_empty(),
            "Command {} has no label",
            command.id
        );
        assert!(
            command.description.is_some(),
            "Command {} has no description",
            command.id
        );
    }
}

#[test]
fn test_split_direction_serialization() {
    // Test that SplitDirection can be serialized/deserialized
    let horizontal = SplitDirection::Horizontal;
    let vertical = SplitDirection::Vertical;

    let h_json = serde_json::to_string(&horizontal).unwrap();
    let v_json = serde_json::to_string(&vertical).unwrap();

    let h_back: SplitDirection = serde_json::from_str(&h_json).unwrap();
    let v_back: SplitDirection = serde_json::from_str(&v_json).unwrap();

    assert_eq!(horizontal, h_back);
    assert_eq!(vertical, v_back);
}

#[test]
fn test_split_direction_equality() {
    assert_eq!(SplitDirection::Horizontal, SplitDirection::Horizontal);
    assert_eq!(SplitDirection::Vertical, SplitDirection::Vertical);
    assert_ne!(SplitDirection::Horizontal, SplitDirection::Vertical);
}

#[test]
fn test_plugin_default_constructor() {
    let plugin1 = PanesPlugin::new();
    let plugin2 = PanesPlugin::default();

    // Both should have same metadata
    assert_eq!(plugin1.metadata().name, plugin2.metadata().name);
    assert_eq!(plugin1.metadata().version, plugin2.metadata().version);
}

#[test]
fn test_plugin_with_custom_size() {
    // Test that we can create plugins with custom terminal sizes
    let plugin_small = PanesPlugin::with_size(40, 12);
    let plugin_large = PanesPlugin::with_size(200, 100);

    // Both should be valid plugins
    assert_eq!(plugin_small.metadata().name, "scarab-panes");
    assert_eq!(plugin_large.metadata().name, "scarab-panes");
}
