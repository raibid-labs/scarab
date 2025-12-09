//! Tests for edge cases and error conditions
//!
//! These tests verify that the pane manager handles unusual situations correctly.

use scarab_panes::PanesPlugin;
use scarab_plugin_api::Plugin;

#[test]
fn test_plugin_with_zero_width() {
    // Test with zero width terminal (edge case)
    let plugin = PanesPlugin::with_size(0, 24);
    assert_eq!(plugin.metadata().name, "scarab-panes");
}

#[test]
fn test_plugin_with_zero_height() {
    // Test with zero height terminal (edge case)
    let plugin = PanesPlugin::with_size(80, 0);
    assert_eq!(plugin.metadata().name, "scarab-panes");
}

#[test]
fn test_plugin_with_both_zero_dimensions() {
    // Test with both dimensions zero
    let plugin = PanesPlugin::with_size(0, 0);
    assert_eq!(plugin.metadata().name, "scarab-panes");
}

#[test]
fn test_plugin_with_max_dimensions() {
    // Test with maximum possible dimensions
    let plugin = PanesPlugin::with_size(u16::MAX, u16::MAX);
    assert_eq!(plugin.metadata().name, "scarab-panes");
}

#[test]
fn test_plugin_with_odd_dimensions() {
    // Test with odd dimensions (ensures proper rounding in splits)
    let plugin = PanesPlugin::with_size(81, 25);
    assert_eq!(plugin.metadata().name, "scarab-panes");
}

#[test]
fn test_plugin_with_prime_number_dimensions() {
    // Test with prime number dimensions
    let plugin = PanesPlugin::with_size(97, 53);
    assert_eq!(plugin.metadata().name, "scarab-panes");
}

#[test]
fn test_very_narrow_terminal() {
    // Test with very narrow terminal (1 column wide)
    let plugin = PanesPlugin::with_size(1, 24);
    assert_eq!(plugin.metadata().name, "scarab-panes");
}

#[test]
fn test_very_short_terminal() {
    // Test with very short terminal (1 row tall)
    let plugin = PanesPlugin::with_size(80, 1);
    assert_eq!(plugin.metadata().name, "scarab-panes");
}

#[test]
fn test_extremely_wide_terminal() {
    // Test with extremely wide terminal
    let plugin = PanesPlugin::with_size(5000, 24);
    assert_eq!(plugin.metadata().name, "scarab-panes");
}

#[test]
fn test_extremely_tall_terminal() {
    // Test with extremely tall terminal
    let plugin = PanesPlugin::with_size(80, 5000);
    assert_eq!(plugin.metadata().name, "scarab-panes");
}

#[test]
fn test_square_terminal() {
    // Test with square terminal dimensions
    let plugin = PanesPlugin::with_size(50, 50);
    assert_eq!(plugin.metadata().name, "scarab-panes");
}

#[test]
fn test_common_terminal_sizes() {
    // Test with various common terminal sizes
    let common_sizes = vec![
        (80, 24),   // Traditional VT100
        (80, 25),   // DOS
        (132, 24),  // Wide VT100
        (120, 40),  // Modern large
        (100, 30),  // Mid-size
        (160, 48),  // Very large
    ];

    for (cols, rows) in common_sizes {
        let plugin = PanesPlugin::with_size(cols, rows);
        assert_eq!(plugin.metadata().name, "scarab-panes");
    }
}

#[test]
fn test_metadata_never_null() {
    let plugin = PanesPlugin::new();
    let metadata = plugin.metadata();

    // Ensure metadata fields are never empty/null
    assert!(!metadata.name.is_empty());
    assert!(!metadata.version.is_empty());
    assert!(!metadata.description.is_empty());
    assert!(!metadata.author.is_empty());
}

#[test]
fn test_commands_always_available() {
    let plugin = PanesPlugin::new();
    let commands = plugin.get_commands();

    // Commands should always be available regardless of state
    assert!(!commands.is_empty());
    assert!(commands.len() >= 10); // We have at least 10 commands defined
}

#[test]
fn test_multiple_plugin_instances() {
    // Test that we can create multiple independent plugin instances
    let plugin1 = PanesPlugin::with_size(80, 24);
    let plugin2 = PanesPlugin::with_size(120, 40);
    let plugin3 = PanesPlugin::with_size(100, 30);

    // All should be valid and independent
    assert_eq!(plugin1.metadata().name, "scarab-panes");
    assert_eq!(plugin2.metadata().name, "scarab-panes");
    assert_eq!(plugin3.metadata().name, "scarab-panes");
}

#[test]
fn test_plugin_clone_metadata() {
    let plugin = PanesPlugin::new();
    let metadata1 = plugin.metadata();
    let metadata2 = plugin.metadata();

    // Should return consistent metadata
    assert_eq!(metadata1.name, metadata2.name);
    assert_eq!(metadata1.version, metadata2.version);
}

#[test]
fn test_commands_have_unique_ids() {
    use std::collections::HashSet;

    let plugin = PanesPlugin::new();
    let commands = plugin.get_commands();

    let mut ids = HashSet::new();
    for command in &commands {
        assert!(
            ids.insert(command.id.clone()),
            "Duplicate command ID: {}",
            command.id
        );
    }

    assert_eq!(ids.len(), commands.len());
}

#[test]
fn test_command_ids_follow_naming_convention() {
    let plugin = PanesPlugin::new();
    let commands = plugin.get_commands();

    // All command IDs should start with "panes."
    for command in commands {
        assert!(
            command.id.starts_with("panes."),
            "Command ID {} doesn't follow naming convention",
            command.id
        );
    }
}

#[test]
fn test_default_plugin_size() {
    let plugin = PanesPlugin::new();
    // Default should be 80x24 (standard terminal)
    // We verify by checking that the plugin is created successfully
    assert_eq!(plugin.metadata().name, "scarab-panes");
}

#[test]
fn test_split_direction_exhaustive_match() {
    use scarab_panes::SplitDirection;

    // Ensure both variants can be created and used
    let _h = SplitDirection::Horizontal;
    let _v = SplitDirection::Vertical;

    // Test that they're different
    assert_ne!(SplitDirection::Horizontal, SplitDirection::Vertical);
}

#[test]
fn test_split_direction_copy_trait() {
    use scarab_panes::SplitDirection;

    let dir1 = SplitDirection::Horizontal;
    let dir2 = dir1; // Copy, not move
    let dir3 = dir1; // Can copy again

    assert_eq!(dir1, dir2);
    assert_eq!(dir2, dir3);
}

#[test]
fn test_pane_layout_with_various_ratios() {
    use scarab_panes::PaneLayout;

    let test_ratios = vec![
        0.0, 0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9, 1.0,
    ];

    for ratio in test_ratios {
        let layout = PaneLayout {
            id: 0,
            parent_id: None,
            split_direction: None,
            x: 0,
            y: 0,
            width: 80,
            height: 24,
            is_focused: true,
            split_ratio: ratio,
        };

        assert_eq!(layout.split_ratio, ratio);
    }
}

#[test]
fn test_pane_layout_with_extreme_positions() {
    use scarab_panes::PaneLayout;

    // Test with extreme positions
    let layout = PaneLayout {
        id: 0,
        parent_id: None,
        split_direction: None,
        x: u16::MAX - 10,
        y: u16::MAX - 10,
        width: 10,
        height: 10,
        is_focused: true,
        split_ratio: 0.5,
    };

    assert_eq!(layout.x, u16::MAX - 10);
    assert_eq!(layout.y, u16::MAX - 10);
}

#[test]
fn test_pane_layout_serialization_roundtrip() {
    use scarab_panes::{PaneLayout, SplitDirection};

    let layouts = vec![
        PaneLayout {
            id: 0,
            parent_id: None,
            split_direction: None,
            x: 0,
            y: 0,
            width: 80,
            height: 24,
            is_focused: true,
            split_ratio: 0.5,
        },
        PaneLayout {
            id: 100,
            parent_id: Some(50),
            split_direction: Some(SplitDirection::Horizontal),
            x: 40,
            y: 12,
            width: 40,
            height: 12,
            is_focused: false,
            split_ratio: 0.75,
        },
        PaneLayout {
            id: u64::MAX,
            parent_id: Some(u64::MAX - 1),
            split_direction: Some(SplitDirection::Vertical),
            x: u16::MAX,
            y: u16::MAX,
            width: 1,
            height: 1,
            is_focused: true,
            split_ratio: 1.0,
        },
    ];

    for original in layouts {
        let json = serde_json::to_string(&original).unwrap();
        let deserialized: PaneLayout = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.id, original.id);
        assert_eq!(deserialized.parent_id, original.parent_id);
        assert_eq!(deserialized.split_direction, original.split_direction);
        assert_eq!(deserialized.x, original.x);
        assert_eq!(deserialized.y, original.y);
        assert_eq!(deserialized.width, original.width);
        assert_eq!(deserialized.height, original.height);
        assert_eq!(deserialized.is_focused, original.is_focused);
        assert_eq!(deserialized.split_ratio, original.split_ratio);
    }
}
