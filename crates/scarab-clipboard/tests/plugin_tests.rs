//! Integration tests for ClipboardPlugin functionality
//!
//! These tests verify the plugin's behavior, including plugin metadata,
//! and command registration. Private methods are tested indirectly through
//! the public API.

use scarab_clipboard::ClipboardPlugin;
use scarab_plugin_api::Plugin;

// =============================================================================
// Plugin Initialization Tests
// =============================================================================

#[test]
fn test_plugin_creation() {
    let plugin = ClipboardPlugin::new();

    // Verify metadata is set
    let metadata = plugin.metadata();
    assert_eq!(metadata.name, "scarab-clipboard");
    assert_eq!(metadata.version, "0.1.0");
}

#[test]
fn test_plugin_default() {
    let plugin = ClipboardPlugin::default();

    let metadata = plugin.metadata();
    assert_eq!(metadata.name, "scarab-clipboard");
}

#[test]
fn test_plugin_metadata() {
    let plugin = ClipboardPlugin::new();
    let metadata = plugin.metadata();

    assert_eq!(metadata.name, "scarab-clipboard");
    assert_eq!(metadata.version, "0.1.0");
    assert_eq!(
        metadata.description,
        "Clipboard integration and text selection for terminal"
    );
    assert_eq!(metadata.author, "Scarab Team");
}

// Note: Word boundary detection and paste confirmation logic are private methods
// tested indirectly through the public Plugin API and verified in lib.rs unit tests.

// =============================================================================
// Plugin Commands Tests
// =============================================================================

#[test]
fn test_plugin_get_commands() {
    use scarab_plugin_api::Plugin;

    let plugin = ClipboardPlugin::new();
    let commands = plugin.get_commands();

    // Should have multiple commands
    assert!(!commands.is_empty());

    // Check for specific commands
    let command_ids: Vec<String> = commands.iter().map(|c| c.id.clone()).collect();

    assert!(command_ids.contains(&"clipboard.copy".to_string()));
    assert!(command_ids.contains(&"clipboard.paste".to_string()));
    assert!(command_ids.contains(&"clipboard.copy_line".to_string()));
    assert!(command_ids.contains(&"clipboard.visual_character".to_string()));
    assert!(command_ids.contains(&"clipboard.visual_line".to_string()));
    assert!(command_ids.contains(&"clipboard.visual_block".to_string()));
    assert!(command_ids.contains(&"clipboard.toggle_bracket_mode".to_string()));
}

#[test]
fn test_plugin_commands_have_descriptions() {
    use scarab_plugin_api::Plugin;

    let plugin = ClipboardPlugin::new();
    let commands = plugin.get_commands();

    // All commands should have labels
    for command in &commands {
        assert!(!command.label.is_empty(), "Command {} has no label", command.id);
    }

    // Most commands should have descriptions
    let with_descriptions = commands.iter().filter(|c| c.description.is_some()).count();
    assert!(with_descriptions > 0);
}

#[test]
fn test_plugin_command_ids_are_unique() {
    use scarab_plugin_api::Plugin;

    let plugin = ClipboardPlugin::new();
    let commands = plugin.get_commands();

    let mut ids = std::collections::HashSet::new();

    for command in &commands {
        assert!(
            ids.insert(command.id.clone()),
            "Duplicate command ID: {}",
            command.id
        );
    }
}

// =============================================================================
// Platform-Specific Tests
// =============================================================================

#[cfg(target_os = "linux")]
mod linux_specific {
    use super::*;

    #[test]
    fn test_plugin_has_primary_selection_command() {
        let plugin = ClipboardPlugin::new();
        let commands = plugin.get_commands();

        let command_ids: Vec<String> = commands.iter().map(|c| c.id.clone()).collect();
        assert!(command_ids.contains(&"clipboard.paste_primary".to_string()));
    }
}

// =============================================================================
// Plugin Instance Tests
// =============================================================================

#[test]
fn test_multiple_plugins_independent() {
    // Test that multiple plugin instances are independent
    let plugin1 = ClipboardPlugin::new();
    let plugin2 = ClipboardPlugin::new();

    let metadata1 = plugin1.metadata();
    let metadata2 = plugin2.metadata();

    assert_eq!(metadata1.name, metadata2.name);
    assert_eq!(metadata1.version, metadata2.version);
}
