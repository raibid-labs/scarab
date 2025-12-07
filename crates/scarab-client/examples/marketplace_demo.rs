// Example demonstrating the plugin marketplace UI
//
// This shows how to use the marketplace overlay for browsing and installing plugins.
//
// Run with: cargo run -p scarab-client --example marketplace_demo
//
// Keybindings:
// - Ctrl+Shift+M: Toggle marketplace
// - /: Focus search
// - Tab: Switch categories
// - ↑↓: Navigate plugins
// - Enter: Install selected plugin
// - d: View plugin details
// - r: Refresh plugin list
// - q/Escape: Close marketplace

use bevy::prelude::*;
use scarab_client::{
    marketplace::{MarketplaceEvent, MarketplacePlugin},
    ratatui_bridge::RatatuiBridgePlugin,
};
use scarab_protocol::TerminalMetrics;

fn main() {
    App::new()
        .add_plugins((MinimalPlugins, RatatuiBridgePlugin, MarketplacePlugin))
        .insert_resource(TerminalMetrics::default())
        .add_systems(Startup, setup)
        .add_systems(Update, handle_toggle_key)
        .run();
}

fn setup(mut commands: Commands, mut events: EventWriter<MarketplaceEvent>) {
    info!("Marketplace demo started");
    info!("Press Ctrl+Shift+M to open the plugin marketplace");
    info!("Press Ctrl+C to exit");

    // Trigger initial marketplace open
    events.send(MarketplaceEvent::Open);
}

/// Handle global toggle keybinding
fn handle_toggle_key(keys: Res<ButtonInput<KeyCode>>, mut events: EventWriter<MarketplaceEvent>) {
    // Ctrl+Shift+M to toggle marketplace
    if keys.pressed(KeyCode::ControlLeft)
        && keys.pressed(KeyCode::ShiftLeft)
        && keys.just_pressed(KeyCode::KeyM)
    {
        events.send(MarketplaceEvent::Toggle);
    }
}
