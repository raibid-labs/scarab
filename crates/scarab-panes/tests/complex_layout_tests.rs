//! Integration tests for complex pane layouts
//!
//! These tests verify that complex multi-pane layouts work correctly,
//! including deep nesting, mixed splits, and stress testing.

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
async fn test_quad_split_layout() {
    // Create a 2x2 grid of panes
    let mut plugin = PanesPlugin::with_size(100, 50);
    let ctx = create_test_context();

    // Split horizontally first (top/bottom)
    plugin.on_remote_command("panes.split_horizontal", &ctx).await.unwrap();

    // Navigate to top pane
    plugin.on_remote_command("panes.navigate_up", &ctx).await.unwrap();

    // Split top pane vertically (left/right)
    plugin.on_remote_command("panes.split_vertical", &ctx).await.unwrap();

    // Navigate to bottom pane
    plugin.on_remote_command("panes.navigate_down", &ctx).await.unwrap();
    plugin.on_remote_command("panes.navigate_down", &ctx).await.unwrap();

    // Split bottom pane vertically (left/right)
    plugin.on_remote_command("panes.split_vertical", &ctx).await.unwrap();

    // Should now have 4 panes in a 2x2 grid
    // Commands should continue to work
    let result = plugin.on_remote_command("panes.navigate_up", &ctx).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_deep_horizontal_nesting() {
    // Create many horizontal splits (vertical stack)
    let mut plugin = PanesPlugin::with_size(80, 100);
    let ctx = create_test_context();

    // Create 5 horizontal splits
    for _ in 0..5 {
        plugin.on_remote_command("panes.split_horizontal", &ctx).await.unwrap();
    }

    // Should be able to navigate through all panes
    for _ in 0..5 {
        let result = plugin.on_remote_command("panes.navigate_up", &ctx).await;
        assert!(result.is_ok());
    }
}

#[tokio::test]
async fn test_deep_vertical_nesting() {
    // Create many vertical splits (horizontal row)
    let mut plugin = PanesPlugin::with_size(200, 40);
    let ctx = create_test_context();

    // Create 5 vertical splits
    for _ in 0..5 {
        plugin.on_remote_command("panes.split_vertical", &ctx).await.unwrap();
    }

    // Should be able to navigate through all panes
    for _ in 0..5 {
        let result = plugin.on_remote_command("panes.navigate_left", &ctx).await;
        assert!(result.is_ok());
    }
}

#[tokio::test]
async fn test_mixed_split_directions() {
    // Create a layout with alternating split directions
    let mut plugin = PanesPlugin::with_size(120, 60);
    let ctx = create_test_context();

    // Horizontal, then vertical, then horizontal, etc.
    plugin.on_remote_command("panes.split_horizontal", &ctx).await.unwrap();
    plugin.on_remote_command("panes.split_vertical", &ctx).await.unwrap();
    plugin.on_remote_command("panes.split_horizontal", &ctx).await.unwrap();
    plugin.on_remote_command("panes.split_vertical", &ctx).await.unwrap();

    // Should be able to navigate and resize
    let result = plugin.on_remote_command("panes.resize_up", &ctx).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_create_and_close_multiple_panes() {
    let mut plugin = PanesPlugin::with_size(100, 50);
    let ctx = create_test_context();

    // Create several panes
    for _ in 0..3 {
        plugin.on_remote_command("panes.split_horizontal", &ctx).await.unwrap();
    }

    // Close panes one by one
    for _ in 0..2 {
        plugin.on_remote_command("panes.close", &ctx).await.unwrap();
    }

    // Should still be able to create new panes
    let result = plugin.on_remote_command("panes.split_vertical", &ctx).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_resize_in_complex_layout() {
    let mut plugin = PanesPlugin::with_size(120, 60);
    let ctx = create_test_context();

    // Create quad split
    plugin.on_remote_command("panes.split_horizontal", &ctx).await.unwrap();
    plugin.on_remote_command("panes.split_vertical", &ctx).await.unwrap();
    plugin.on_remote_command("panes.navigate_down", &ctx).await.unwrap();
    plugin.on_remote_command("panes.split_vertical", &ctx).await.unwrap();

    // Try resizing in various directions
    plugin.on_remote_command("panes.resize_up", &ctx).await.unwrap();
    plugin.on_remote_command("panes.resize_left", &ctx).await.unwrap();
    plugin.on_remote_command("panes.resize_down", &ctx).await.unwrap();
    plugin.on_remote_command("panes.resize_right", &ctx).await.unwrap();

    // All resize operations should complete without error
}

#[tokio::test]
async fn test_terminal_resize_with_complex_layout() {
    let mut plugin = PanesPlugin::with_size(80, 24);
    let ctx = create_test_context();

    // Create a complex layout
    plugin.on_remote_command("panes.split_horizontal", &ctx).await.unwrap();
    plugin.on_remote_command("panes.split_vertical", &ctx).await.unwrap();

    // Resize terminal multiple times
    plugin.on_resize(100, 40, &ctx).await.unwrap();
    plugin.on_resize(120, 50, &ctx).await.unwrap();
    plugin.on_resize(60, 20, &ctx).await.unwrap();
    plugin.on_resize(200, 100, &ctx).await.unwrap();

    // Commands should still work after resizing
    let result = plugin.on_remote_command("panes.navigate_up", &ctx).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_navigation_in_complex_grid() {
    let mut plugin = PanesPlugin::with_size(150, 75);
    let ctx = create_test_context();

    // Create 3x3 grid of panes (simplified)
    // First row
    plugin.on_remote_command("panes.split_vertical", &ctx).await.unwrap();
    plugin.on_remote_command("panes.split_vertical", &ctx).await.unwrap();

    // Navigate and create second row
    plugin.on_remote_command("panes.navigate_left", &ctx).await.unwrap();
    plugin.on_remote_command("panes.navigate_left", &ctx).await.unwrap();
    plugin.on_remote_command("panes.split_horizontal", &ctx).await.unwrap();

    // Navigation should work in all directions
    plugin.on_remote_command("panes.navigate_right", &ctx).await.unwrap();
    plugin.on_remote_command("panes.navigate_up", &ctx).await.unwrap();
    plugin.on_remote_command("panes.navigate_left", &ctx).await.unwrap();
    plugin.on_remote_command("panes.navigate_down", &ctx).await.unwrap();

    // All navigation commands should succeed
}

#[tokio::test]
async fn test_stress_many_splits() {
    // Stress test with many panes
    let mut plugin = PanesPlugin::with_size(200, 100);
    let ctx = create_test_context();

    // Create 10 panes
    for i in 0..10 {
        let direction = if i % 2 == 0 {
            "panes.split_horizontal"
        } else {
            "panes.split_vertical"
        };
        plugin.on_remote_command(direction, &ctx).await.unwrap();
    }

    // Should be able to navigate
    for _ in 0..5 {
        plugin.on_remote_command("panes.navigate_up", &ctx).await.unwrap();
    }
}

#[tokio::test]
async fn test_close_panes_in_complex_layout() {
    let mut plugin = PanesPlugin::with_size(120, 60);
    let ctx = create_test_context();

    // Create 5 panes
    for _ in 0..4 {
        plugin.on_remote_command("panes.split_horizontal", &ctx).await.unwrap();
    }

    // Close panes in various orders
    plugin.on_remote_command("panes.navigate_up", &ctx).await.unwrap();
    plugin.on_remote_command("panes.navigate_up", &ctx).await.unwrap();
    plugin.on_remote_command("panes.close", &ctx).await.unwrap();

    plugin.on_remote_command("panes.navigate_down", &ctx).await.unwrap();
    plugin.on_remote_command("panes.close", &ctx).await.unwrap();

    // Should still have functional panes
    let result = plugin.on_remote_command("panes.split_vertical", &ctx).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_alternating_operations() {
    let mut plugin = PanesPlugin::with_size(100, 50);
    let ctx = create_test_context();

    // Alternate between split, navigate, resize, close
    plugin.on_remote_command("panes.split_horizontal", &ctx).await.unwrap();
    plugin.on_remote_command("panes.navigate_up", &ctx).await.unwrap();
    plugin.on_remote_command("panes.split_vertical", &ctx).await.unwrap();
    plugin.on_remote_command("panes.resize_down", &ctx).await.unwrap();
    plugin.on_remote_command("panes.navigate_right", &ctx).await.unwrap();
    plugin.on_remote_command("panes.split_horizontal", &ctx).await.unwrap();
    plugin.on_remote_command("panes.close", &ctx).await.unwrap();
    plugin.on_remote_command("panes.navigate_left", &ctx).await.unwrap();

    // All operations should succeed
}

#[tokio::test]
async fn test_resize_sequence() {
    let mut plugin = PanesPlugin::with_size(100, 50);
    let ctx = create_test_context();

    // Create split
    plugin.on_remote_command("panes.split_horizontal", &ctx).await.unwrap();

    // Resize multiple times in same direction
    for _ in 0..5 {
        plugin.on_remote_command("panes.resize_down", &ctx).await.unwrap();
    }

    // Resize back
    for _ in 0..5 {
        plugin.on_remote_command("panes.resize_up", &ctx).await.unwrap();
    }

    // Should handle repeated resizes
}

#[tokio::test]
async fn test_layout_after_multiple_terminal_resizes() {
    let mut plugin = PanesPlugin::with_size(80, 24);
    let ctx = create_test_context();

    // Create initial layout
    plugin.on_remote_command("panes.split_horizontal", &ctx).await.unwrap();
    plugin.on_remote_command("panes.split_vertical", &ctx).await.unwrap();

    // Resize terminal several times
    let sizes = vec![
        (100, 40),
        (60, 20),
        (150, 60),
        (80, 24),
        (200, 100),
    ];

    for (cols, rows) in sizes {
        plugin.on_resize(cols, rows, &ctx).await.unwrap();

        // Verify operations still work after each resize
        plugin.on_remote_command("panes.navigate_up", &ctx).await.unwrap();
        plugin.on_remote_command("panes.navigate_down", &ctx).await.unwrap();
    }
}

#[tokio::test]
async fn test_very_small_terminal_with_splits() {
    // Test behavior with very small terminal
    let mut plugin = PanesPlugin::with_size(10, 5);
    let ctx = create_test_context();

    // Try to create splits in tiny terminal
    plugin.on_remote_command("panes.split_horizontal", &ctx).await.unwrap();
    plugin.on_remote_command("panes.split_vertical", &ctx).await.unwrap();

    // Should not crash, even though panes will be tiny
}

#[tokio::test]
async fn test_very_large_terminal_with_splits() {
    // Test behavior with very large terminal
    let mut plugin = PanesPlugin::with_size(500, 250);
    let ctx = create_test_context();

    // Create multiple splits
    for _ in 0..8 {
        plugin.on_remote_command("panes.split_horizontal", &ctx).await.unwrap();
    }

    // Should handle large terminal gracefully
    let result = plugin.on_remote_command("panes.navigate_up", &ctx).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_comprehensive_workflow() {
    // Simulate a real-world workflow
    let mut plugin = PanesPlugin::with_size(120, 60);
    let ctx = create_test_context();

    // User creates a development layout
    // 1. Split for editor and terminal
    plugin.on_remote_command("panes.split_horizontal", &ctx).await.unwrap();

    // 2. Split terminal pane for tests
    plugin.on_remote_command("panes.split_vertical", &ctx).await.unwrap();

    // 3. Navigate to editor (top)
    plugin.on_remote_command("panes.navigate_up", &ctx).await.unwrap();

    // 4. Split editor for side-by-side files
    plugin.on_remote_command("panes.split_vertical", &ctx).await.unwrap();

    // 5. Resize to give more space to main editor
    plugin.on_remote_command("panes.navigate_left", &ctx).await.unwrap();
    plugin.on_remote_command("panes.resize_right", &ctx).await.unwrap();
    plugin.on_remote_command("panes.resize_right", &ctx).await.unwrap();

    // 6. Close side panel
    plugin.on_remote_command("panes.navigate_right", &ctx).await.unwrap();
    plugin.on_remote_command("panes.close", &ctx).await.unwrap();

    // 7. User changes terminal size
    plugin.on_resize(150, 75, &ctx).await.unwrap();

    // All operations should succeed
}
