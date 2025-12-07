//! Telemetry HUD Plugin for Scarab
//!
//! Displays real-time performance metrics as an overlay in the terminal emulator.
//! The HUD can be toggled with F12 and shows FPS, frame times, and other performance data.
//!
//! # Usage
//!
//! ```rust,ignore
//! use bevy::prelude::*;
//! use scarab_telemetry_hud::TelemetryHudPlugin;
//!
//! App::new()
//!     .add_plugins(DefaultPlugins)
//!     .add_plugins(TelemetryHudPlugin::default())
//!     .run();
//! ```
//!
//! # Features
//!
//! - Real-time FPS counter
//! - Frame time statistics (current, avg, min, max)
//! - Frame time graph (rolling window)
//! - Configurable position (top-right, top-left, bottom-right, bottom-left)
//! - Toggle visibility with F12
//! - Minimal performance overhead using lock-free atomics

pub mod integration;
mod metrics;
mod overlay;

pub use integration::update_nav_hint_counts;
pub use metrics::{
    CacheStats, ExtendedMetrics, HintStats, MemoryStats, PerformanceMetrics, PerformanceSnapshot,
    TelemetryData,
};
pub use overlay::{HudPosition, HudState};

use bevy::prelude::*;
use metrics::{update_cache_stats, update_hint_stats, update_memory_stats, update_metrics};
use overlay::{render_hud, toggle_hud};

/// Telemetry HUD Plugin
///
/// Tracks and displays performance metrics as an overlay.
/// Press F12 to toggle visibility.
///
/// The HUD displays:
/// - Current FPS
/// - Frame time (milliseconds)
/// - Average frame time
/// - Min/Max frame times
/// - Frame time graph
pub struct TelemetryHudPlugin {
    /// Initial visibility state
    pub visible: bool,
    /// Initial HUD position
    pub position: HudPosition,
    /// Frame time window size for averaging
    pub window_size: usize,
}

impl Default for TelemetryHudPlugin {
    fn default() -> Self {
        Self {
            visible: false,
            position: HudPosition::TopRight,
            window_size: 120, // 2 seconds at 60 FPS
        }
    }
}

impl TelemetryHudPlugin {
    /// Create a plugin with custom initial visibility
    pub fn with_visibility(mut self, visible: bool) -> Self {
        self.visible = visible;
        self
    }

    /// Create a plugin with custom position
    pub fn with_position(mut self, position: HudPosition) -> Self {
        self.position = position;
        self
    }

    /// Create a plugin with custom window size for averaging
    pub fn with_window_size(mut self, window_size: usize) -> Self {
        self.window_size = window_size;
        self
    }
}

impl Plugin for TelemetryHudPlugin {
    fn build(&self, app: &mut App) {
        // Initialize resources
        app.insert_resource(HudState {
            visible: self.visible,
            position: self.position,
        })
        .insert_resource(PerformanceMetrics::new(self.window_size))
        .insert_resource(TelemetryData::default());

        // Register systems
        // Always update metrics (minimal overhead)
        // Only render when visible (checked in system)
        app.add_systems(
            Update,
            (
                update_metrics,
                update_cache_stats,
                update_memory_stats,
                update_hint_stats,
                toggle_hud,
                render_hud,
            )
                .chain(),
        );

        info!(
            "TelemetryHudPlugin initialized (visible: {}, position: {:?})",
            self.visible, self.position
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plugin_defaults() {
        let plugin = TelemetryHudPlugin::default();
        assert!(!plugin.visible);
        assert!(matches!(plugin.position, HudPosition::TopRight));
        assert_eq!(plugin.window_size, 120);
    }

    #[test]
    fn test_plugin_builder() {
        let plugin = TelemetryHudPlugin::default()
            .with_visibility(true)
            .with_position(HudPosition::BottomLeft)
            .with_window_size(60);

        assert!(plugin.visible);
        assert!(matches!(plugin.position, HudPosition::BottomLeft));
        assert_eq!(plugin.window_size, 60);
    }
}
