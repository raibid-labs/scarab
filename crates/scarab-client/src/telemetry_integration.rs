//! Telemetry HUD integration for Scarab client
//!
//! This module integrates the scarab-telemetry-hud plugin with the Scarab client,
//! connecting it to navigation systems and configuration.

use bevy::prelude::*;
use scarab_config::ScarabConfig;
use scarab_telemetry_hud::{TelemetryHudPlugin, HudPosition, TelemetryData};
use crate::navigation::{NavHint, focusable::FocusableRegion};
use crate::rendering::hint_overlay::HintOverlay;

/// Scarab-specific telemetry plugin that integrates with configuration
pub struct ScarabTelemetryPlugin;

impl Plugin for ScarabTelemetryPlugin {
    fn build(&self, app: &mut App) {
        // Get config to determine initial settings
        let config = app.world().get_resource::<ScarabConfig>();

        let (visible, position) = if let Some(cfg) = config {
            let pos = match cfg.telemetry.hud_position.as_str() {
                "top-left" => HudPosition::TopLeft,
                "bottom-right" => HudPosition::BottomRight,
                "bottom-left" => HudPosition::BottomLeft,
                _ => HudPosition::TopRight,
            };
            (cfg.telemetry.hud_enabled, pos)
        } else {
            (false, HudPosition::TopRight)
        };

        // Add the core telemetry plugin with config-based settings
        app.add_plugins(
            TelemetryHudPlugin::default()
                .with_visibility(visible)
                .with_position(position)
        );

        // Add system to count navigation components
        app.add_systems(Update, update_scarab_hint_counts);

        info!("Scarab telemetry integration initialized");
    }
}

/// System to update hint counts from Scarab navigation components
fn update_scarab_hint_counts(
    hint_query: Query<&NavHint>,
    focusable_query: Query<&FocusableRegion>,
    overlay_query: Query<&HintOverlay>,
    mut telemetry: ResMut<TelemetryData>,
) {
    telemetry.hint_stats.hint_count = hint_query.iter().len();
    telemetry.hint_stats.focusable_count = focusable_query.iter().len();
    telemetry.hint_stats.overlay_count = overlay_query.iter().len();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plugin_initialization() {
        let mut app = App::new();
        app.add_plugins(ScarabTelemetryPlugin);

        // Verify resources are initialized
        assert!(app.world().get_resource::<TelemetryData>().is_some());
    }
}
