//! Integration systems for tracking external component counts
//!
//! This module provides systems that integrate with other Scarab subsystems
//! to collect telemetry data, such as navigation hints and focusable regions.

use crate::metrics::TelemetryData;
use bevy::prelude::*;

/// Marker trait for components that should be counted as navigation hints
///
/// Implement this on your NavHint component to enable automatic counting.
pub trait NavHintMarker: Component {}

/// Marker trait for components that should be counted as focusable regions
///
/// Implement this on your FocusableRegion component to enable automatic counting.
pub trait FocusableMarker: Component {}

/// Marker trait for components that should be counted as hint overlays
///
/// Implement this on your HintOverlay component to enable automatic counting.
pub trait HintOverlayMarker: Component {}

/// Generic system to count entities with a specific component
///
/// This is used internally but can be customized for specific counting needs.
pub fn count_entities<T: Component>(
    query: Query<&T>,
    mut telemetry: ResMut<TelemetryData>,
    field: impl FnOnce(&mut TelemetryData) -> &mut usize,
) {
    let count = query.iter().len();
    *field(&mut telemetry) = count;
}

/// System to update hint statistics from navigation components
///
/// This is a placeholder that can be specialized when integrating with scarab-client.
/// In the full implementation, this would query actual NavHint, FocusableRegion,
/// and HintOverlay components.
///
/// # Example Integration
///
/// ```rust,ignore
/// use scarab_telemetry_hud::integration::update_nav_hint_counts;
/// use scarab_nav::{NavHint, FocusableRegion};
/// use scarab_rendering::HintOverlay;
///
/// app.add_systems(Update, update_nav_hint_counts::<NavHint, FocusableRegion, HintOverlay>);
/// ```
pub fn update_nav_hint_counts<NH, FR, HO>(
    hint_query: Query<&NH>,
    focusable_query: Query<&FR>,
    overlay_query: Query<&HO>,
    mut telemetry: ResMut<TelemetryData>,
) where
    NH: Component,
    FR: Component,
    HO: Component,
{
    telemetry.hint_stats.hint_count = hint_query.iter().len();
    telemetry.hint_stats.focusable_count = focusable_query.iter().len();
    telemetry.hint_stats.overlay_count = overlay_query.iter().len();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Component)]
    struct TestComponent;

    #[test]
    fn test_count_entities() {
        let mut app = App::new();
        app.insert_resource(TelemetryData::default());

        // Spawn some test entities
        app.world_mut().spawn(TestComponent);
        app.world_mut().spawn(TestComponent);
        app.world_mut().spawn(TestComponent);

        // The count_entities function would be called by a system
        // This is just a structural test
    }
}
