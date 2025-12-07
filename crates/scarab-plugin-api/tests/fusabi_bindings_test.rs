//! Integration tests for Fusabi host bindings
//!
//! Tests capability checks, quota enforcement, and rate limiting for the
//! new ECS-safe UI/nav bindings added in Fusabi 0.21.0.

use parking_lot::Mutex;
use scarab_plugin_api::context::{PluginConfigData, PluginSharedState};
use scarab_plugin_api::error::PluginError;
use scarab_plugin_api::host_bindings::{HostBindingLimits, HostBindings, DEFAULT_RATE_LIMIT};
use scarab_plugin_api::navigation::PluginNavCapabilities;
use scarab_plugin_api::types::{JumpDirection, OverlayConfig, StatusBarItem};
use scarab_plugin_api::PluginContext;
use std::sync::Arc;

fn make_test_ctx() -> PluginContext {
    PluginContext::new(
        PluginConfigData::default(),
        Arc::new(Mutex::new(PluginSharedState::new(80, 24))),
        "test_fusabi_plugin",
    )
}

// ============================================================================
// Overlay Tests
// ============================================================================

#[test]
fn test_spawn_overlay_success() {
    let ctx = make_test_ctx();
    let bindings = HostBindings::with_defaults();

    let config = OverlayConfig::new(10, 5, "Hello, World!");
    let result = bindings.spawn_overlay(&ctx, config);

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 1);
    assert_eq!(bindings.resource_usage().overlays, 1);
}

#[test]
fn test_spawn_overlay_quota_exceeded() {
    let ctx = make_test_ctx();
    let limits = HostBindingLimits {
        max_overlays: 2,
        ..Default::default()
    };
    let bindings = HostBindings::new(limits, PluginNavCapabilities::default());

    let config = OverlayConfig::new(10, 5, "Overlay");
    assert!(bindings.spawn_overlay(&ctx, config.clone()).is_ok());
    bindings.reset_rate_limit();
    assert!(bindings.spawn_overlay(&ctx, config.clone()).is_ok());
    bindings.reset_rate_limit();

    let result = bindings.spawn_overlay(&ctx, config);
    assert!(
        matches!(result, Err(PluginError::QuotaExceeded { resource, limit, .. }) 
        if resource == "overlays" && limit == 2)
    );
}

#[test]
fn test_spawn_overlay_bounds_check() {
    let ctx = make_test_ctx();
    let limits = HostBindingLimits {
        bounds_check: true,
        max_coordinate: 100,
        ..Default::default()
    };
    let bindings = HostBindings::new(limits, PluginNavCapabilities::default());

    let config = OverlayConfig::new(150, 50, "Out of bounds");
    let result = bindings.spawn_overlay(&ctx, config);

    assert!(matches!(result, Err(PluginError::ValidationError(_))));
}

#[test]
fn test_remove_overlay() {
    let ctx = make_test_ctx();
    let bindings = HostBindings::with_defaults();

    let config = OverlayConfig::new(10, 5, "To remove");
    let overlay_id = bindings.spawn_overlay(&ctx, config).unwrap();
    assert_eq!(bindings.resource_usage().overlays, 1);

    bindings.reset_rate_limit();
    let result = bindings.remove_overlay(&ctx, overlay_id);
    assert!(result.is_ok());
    assert_eq!(bindings.resource_usage().overlays, 0);
}

// ============================================================================
// Status Bar Item Tests
// ============================================================================

#[test]
fn test_add_status_item_success() {
    let ctx = make_test_ctx();
    let bindings = HostBindings::with_defaults();

    let item = StatusBarItem::new("git", "main").with_priority(10);
    let result = bindings.add_status_item(&ctx, item);

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 1);
    assert_eq!(bindings.resource_usage().status_items, 1);
}

#[test]
fn test_add_status_item_quota_exceeded() {
    let ctx = make_test_ctx();
    let limits = HostBindingLimits {
        max_status_items: 2,
        ..Default::default()
    };
    let bindings = HostBindings::new(limits, PluginNavCapabilities::default());

    let item = StatusBarItem::new("label", "content");
    assert!(bindings.add_status_item(&ctx, item.clone()).is_ok());
    bindings.reset_rate_limit();
    assert!(bindings.add_status_item(&ctx, item.clone()).is_ok());
    bindings.reset_rate_limit();

    let result = bindings.add_status_item(&ctx, item);
    assert!(
        matches!(result, Err(PluginError::QuotaExceeded { resource, limit, .. }) 
        if resource == "status_items" && limit == 2)
    );
}

#[test]
fn test_remove_status_item() {
    let ctx = make_test_ctx();
    let bindings = HostBindings::with_defaults();

    let item = StatusBarItem::new("test", "value");
    let item_id = bindings.add_status_item(&ctx, item).unwrap();
    assert_eq!(bindings.resource_usage().status_items, 1);

    bindings.reset_rate_limit();
    let result = bindings.remove_status_item(&ctx, item_id);
    assert!(result.is_ok());
    assert_eq!(bindings.resource_usage().status_items, 0);
}

// ============================================================================
// Prompt Jump Tests
// ============================================================================

#[test]
fn test_prompt_jump_directions() {
    let ctx = make_test_ctx();
    let bindings = HostBindings::with_defaults();

    for direction in [
        JumpDirection::Up,
        JumpDirection::Down,
        JumpDirection::First,
        JumpDirection::Last,
    ] {
        let result = bindings.prompt_jump(&ctx, direction);
        assert!(result.is_ok());
        bindings.reset_rate_limit();
    }
}

// ============================================================================
// Rate Limiting Tests
// ============================================================================

#[test]
fn test_overlay_rate_limiting() {
    let ctx = make_test_ctx();
    let limits = HostBindingLimits {
        rate_limit: 3,
        max_overlays: 100,
        ..Default::default()
    };
    let bindings = HostBindings::new(limits, PluginNavCapabilities::default());

    let config = OverlayConfig::new(0, 0, "test");

    assert!(bindings.spawn_overlay(&ctx, config.clone()).is_ok());
    assert!(bindings.spawn_overlay(&ctx, config.clone()).is_ok());
    assert!(bindings.spawn_overlay(&ctx, config.clone()).is_ok());

    let result = bindings.spawn_overlay(&ctx, config);
    assert!(matches!(result, Err(PluginError::RateLimitExceeded { .. })));
}

#[test]
fn test_status_item_rate_limiting() {
    let ctx = make_test_ctx();
    let limits = HostBindingLimits {
        rate_limit: 2,
        max_status_items: 100,
        ..Default::default()
    };
    let bindings = HostBindings::new(limits, PluginNavCapabilities::default());

    let item = StatusBarItem::new("test", "value");

    assert!(bindings.add_status_item(&ctx, item.clone()).is_ok());
    assert!(bindings.add_status_item(&ctx, item.clone()).is_ok());

    let result = bindings.add_status_item(&ctx, item);
    assert!(matches!(result, Err(PluginError::RateLimitExceeded { .. })));
}

#[test]
fn test_prompt_jump_rate_limiting() {
    let ctx = make_test_ctx();
    let limits = HostBindingLimits {
        rate_limit: 2,
        ..Default::default()
    };
    let bindings = HostBindings::new(limits, PluginNavCapabilities::default());

    assert!(bindings.prompt_jump(&ctx, JumpDirection::Up).is_ok());
    assert!(bindings.prompt_jump(&ctx, JumpDirection::Down).is_ok());

    let result = bindings.prompt_jump(&ctx, JumpDirection::First);
    assert!(matches!(result, Err(PluginError::RateLimitExceeded { .. })));
}

// ============================================================================
// Type Construction Tests
// ============================================================================

#[test]
fn test_overlay_config_builder() {
    let config =
        OverlayConfig::new(10, 20, "Content").with_style(scarab_protocol::OverlayStyle::default());

    assert_eq!(config.x, 10);
    assert_eq!(config.y, 20);
    assert_eq!(config.content, "Content");
}

#[test]
fn test_status_bar_item_builder() {
    let item = StatusBarItem::new("branch", "main").with_priority(5);

    assert_eq!(item.label, "branch");
    assert_eq!(item.content, "main");
    assert_eq!(item.priority, 5);
}

#[test]
fn test_jump_direction_equality() {
    assert_eq!(JumpDirection::Up, JumpDirection::Up);
    assert_ne!(JumpDirection::Up, JumpDirection::Down);
    assert_ne!(JumpDirection::First, JumpDirection::Last);
}
