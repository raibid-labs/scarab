//! Async integration tests for the PanesPlugin
//!
//! These tests verify async plugin behavior including command handling and event processing.

use scarab_panes::PanesPlugin;
use scarab_plugin_api::{
    context::{PluginConfigData, PluginContext, PluginSharedState},
    Plugin,
};
use std::sync::Arc;
use parking_lot::Mutex;

// Helper function to create a test context
fn create_test_context() -> PluginContext {
    let config = PluginConfigData::default();
    let state = Arc::new(Mutex::new(PluginSharedState::new(80, 24)));
    PluginContext::new(config, state, "test-plugin")
}

#[tokio::test]
async fn test_on_resize_command() {
    let mut plugin = PanesPlugin::with_size(80, 24);
    let ctx = create_test_context();

    // Resize terminal
    let result = plugin.on_resize(100, 40, &ctx).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_split_horizontal_command() {
    let mut plugin = PanesPlugin::with_size(80, 40);
    let ctx = create_test_context();

    let result = plugin.on_remote_command("panes.split_horizontal", &ctx).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_split_vertical_command() {
    let mut plugin = PanesPlugin::with_size(80, 40);
    let ctx = create_test_context();

    let result = plugin.on_remote_command("panes.split_vertical", &ctx).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_close_pane_command() {
    let mut plugin = PanesPlugin::with_size(80, 40);
    let ctx = create_test_context();

    // First create a split
    plugin.on_remote_command("panes.split_horizontal", &ctx).await.unwrap();

    // Then close it
    let result = plugin.on_remote_command("panes.close", &ctx).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_cannot_close_last_pane_command() {
    let mut plugin = PanesPlugin::with_size(80, 40);
    let ctx = create_test_context();

    // Try to close the only pane
    let result = plugin.on_remote_command("panes.close", &ctx).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_navigate_up_command() {
    let mut plugin = PanesPlugin::with_size(80, 40);
    let ctx = create_test_context();

    // Create horizontal split first
    plugin.on_remote_command("panes.split_horizontal", &ctx).await.unwrap();

    // Try to navigate up
    let result = plugin.on_remote_command("panes.navigate_up", &ctx).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_navigate_down_command() {
    let mut plugin = PanesPlugin::with_size(80, 40);
    let ctx = create_test_context();

    plugin.on_remote_command("panes.split_horizontal", &ctx).await.unwrap();

    let result = plugin.on_remote_command("panes.navigate_down", &ctx).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_navigate_left_command() {
    let mut plugin = PanesPlugin::with_size(80, 40);
    let ctx = create_test_context();

    plugin.on_remote_command("panes.split_vertical", &ctx).await.unwrap();

    let result = plugin.on_remote_command("panes.navigate_left", &ctx).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_navigate_right_command() {
    let mut plugin = PanesPlugin::with_size(80, 40);
    let ctx = create_test_context();

    plugin.on_remote_command("panes.split_vertical", &ctx).await.unwrap();

    let result = plugin.on_remote_command("panes.navigate_right", &ctx).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_resize_up_command() {
    let mut plugin = PanesPlugin::with_size(80, 40);
    let ctx = create_test_context();

    plugin.on_remote_command("panes.split_horizontal", &ctx).await.unwrap();

    let result = plugin.on_remote_command("panes.resize_up", &ctx).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_resize_down_command() {
    let mut plugin = PanesPlugin::with_size(80, 40);
    let ctx = create_test_context();

    plugin.on_remote_command("panes.split_horizontal", &ctx).await.unwrap();

    let result = plugin.on_remote_command("panes.resize_down", &ctx).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_resize_left_command() {
    let mut plugin = PanesPlugin::with_size(80, 40);
    let ctx = create_test_context();

    plugin.on_remote_command("panes.split_vertical", &ctx).await.unwrap();

    let result = plugin.on_remote_command("panes.resize_left", &ctx).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_resize_right_command() {
    let mut plugin = PanesPlugin::with_size(80, 40);
    let ctx = create_test_context();

    plugin.on_remote_command("panes.split_vertical", &ctx).await.unwrap();

    let result = plugin.on_remote_command("panes.resize_right", &ctx).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_zoom_command() {
    let mut plugin = PanesPlugin::with_size(80, 40);
    let ctx = create_test_context();

    let result = plugin.on_remote_command("panes.zoom", &ctx).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_unknown_command() {
    let mut plugin = PanesPlugin::with_size(80, 40);
    let ctx = create_test_context();

    // Unknown command should not error, just be ignored
    let result = plugin.on_remote_command("panes.unknown_command", &ctx).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_multiple_sequential_commands() {
    let mut plugin = PanesPlugin::with_size(100, 50);
    let ctx = create_test_context();

    // Execute a sequence of commands
    plugin.on_remote_command("panes.split_horizontal", &ctx).await.unwrap();
    plugin.on_remote_command("panes.split_vertical", &ctx).await.unwrap();
    plugin.on_remote_command("panes.navigate_up", &ctx).await.unwrap();
    plugin.on_remote_command("panes.resize_down", &ctx).await.unwrap();

    // All commands should complete without error
}

#[tokio::test]
async fn test_resize_after_terminal_resize() {
    let mut plugin = PanesPlugin::with_size(80, 24);
    let ctx = create_test_context();

    // Create splits
    plugin.on_remote_command("panes.split_horizontal", &ctx).await.unwrap();

    // Resize terminal
    plugin.on_resize(120, 40, &ctx).await.unwrap();

    // Commands should still work after resize
    let result = plugin.on_remote_command("panes.split_vertical", &ctx).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_on_input_horizontal_split() {
    use scarab_plugin_api::Action;

    let mut plugin = PanesPlugin::with_size(80, 40);
    let ctx = create_test_context();

    // Ctrl+Shift+- (ASCII 0x1F) for horizontal split
    let input = &[0x1f];
    let result = plugin.on_input(input, &ctx).await;

    assert!(result.is_ok());
    match result.unwrap() {
        Action::Modify(_) => {
            // Success - split was created
        }
        Action::Continue => {
            panic!("Expected Modify action for split");
        }
        _ => panic!("Unexpected action"),
    }
}

#[tokio::test]
async fn test_on_input_vertical_split() {
    use scarab_plugin_api::Action;

    let mut plugin = PanesPlugin::with_size(80, 40);
    let ctx = create_test_context();

    // Ctrl+Shift+\ (ASCII 0x1C) for vertical split
    let input = &[0x1c];
    let result = plugin.on_input(input, &ctx).await;

    assert!(result.is_ok());
    match result.unwrap() {
        Action::Modify(_) => {
            // Success - split was created
        }
        Action::Continue => {
            panic!("Expected Modify action for split");
        }
        _ => panic!("Unexpected action"),
    }
}

#[tokio::test]
async fn test_on_input_unrecognized() {
    use scarab_plugin_api::Action;

    let mut plugin = PanesPlugin::with_size(80, 40);
    let ctx = create_test_context();

    // Some random input
    let input = b"hello";
    let result = plugin.on_input(input, &ctx).await;

    assert!(result.is_ok());
    match result.unwrap() {
        Action::Continue => {
            // Expected - unrecognized input should continue
        }
        _ => panic!("Expected Continue action for unrecognized input"),
    }
}
