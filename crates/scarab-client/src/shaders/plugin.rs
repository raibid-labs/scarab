//! Bevy plugin for Scarab post-processing effects
//!
//! This plugin integrates blur and glow shaders into the render pipeline,
//! automatically enabling/disabling effects based on overlay visibility and config.

use super::blur::{update_blur_settings, BlurSettings};
use super::glow::{update_glow_settings, GlowSettings};
use bevy::prelude::*;
use bevy::render::extract_component::{ExtractComponent, ExtractComponentPlugin};
use bevy::render::RenderApp;
use scarab_config::ScarabConfig;

/// Main plugin for post-processing effects
///
/// Adds blur and glow shaders to the render pipeline, with automatic
/// configuration management and overlay detection.
pub struct ScarabEffectsPlugin;

impl Plugin for ScarabEffectsPlugin {
    fn build(&self, app: &mut App) {
        // Add component extraction plugins
        app.add_plugins((
            ExtractComponentPlugin::<BlurSettings>::default(),
            ExtractComponentPlugin::<GlowSettings>::default(),
        ));

        // Add main app systems
        app.add_systems(
            Update,
            (
                update_blur_settings,
                update_glow_settings,
                apply_effects_to_overlays,
            ),
        );

        // Initialize effects on startup
        app.add_systems(Startup, setup_effects);

        // Add render pipeline setup if RenderApp exists
        let render_app = app.sub_app_mut(RenderApp);
        // TODO: Add render graph nodes when render pipeline is needed
        // render_app.add_systems(Render, render_effects);
    }
}

/// Marker component for entities that should have effects applied
#[derive(Component, Clone, Copy)]
pub struct EffectsTarget {
    /// Should blur be applied to this entity?
    pub apply_blur: bool,
    /// Should glow be applied to this entity?
    pub apply_glow: bool,
}

impl Default for EffectsTarget {
    fn default() -> Self {
        Self {
            apply_blur: false,
            apply_glow: false,
        }
    }
}

impl ExtractComponent for BlurSettings {
    type QueryData = &'static Self;
    type QueryFilter = ();
    type Out = Self;

    fn extract_component(item: bevy::ecs::query::QueryItem<Self::QueryData>) -> Option<Self::Out> {
        Some(*item)
    }
}

impl ExtractComponent for GlowSettings {
    type QueryData = &'static Self;
    type QueryFilter = ();
    type Out = Self;

    fn extract_component(item: bevy::ecs::query::QueryItem<Self::QueryData>) -> Option<Self::Out> {
        Some(*item)
    }
}

/// Initialize effects system
fn setup_effects(mut commands: Commands, config: Res<ScarabConfig>) {
    // Spawn a global effects controller entity
    let mut effects_entity = commands.spawn((
        Name::new("EffectsController"),
        BlurSettings {
            radius: config.effects.overlay_blur_radius,
            intensity: config.effects.overlay_blur_intensity,
            enabled: config.effects.should_render_blur(),
        },
        GlowSettings {
            radius: config.effects.overlay_glow_radius,
            intensity: config.effects.overlay_glow_intensity,
            enabled: config.effects.should_render_glow(),
            ..Default::default()
        },
    ));

    // Parse glow color from config
    let (r, g, b) = config.effects.glow_color_rgb();
    effects_entity.insert(GlowSettings {
        color: Vec3::new(r, g, b),
        ..GlowSettings::default()
    });

    info!("Scarab post-processing effects initialized");
    if config.effects.low_power_mode {
        info!("Low-power mode enabled: effects disabled");
    } else {
        if config.effects.should_render_blur() {
            info!(
                "Blur enabled: radius={}, intensity={}",
                config.effects.overlay_blur_radius, config.effects.overlay_blur_intensity
            );
        }
        if config.effects.should_render_glow() {
            info!(
                "Glow enabled: radius={}, intensity={}, color={}",
                config.effects.overlay_glow_radius,
                config.effects.overlay_glow_intensity,
                config.effects.overlay_glow_color
            );
        }
    }
}

/// System to apply effects to overlay entities
///
/// This system automatically detects overlays (command palette, search, etc.)
/// and applies blur/glow effects when they're visible.
fn apply_effects_to_overlays(
    mut commands: Commands,
    config: Res<ScarabConfig>,
    // Query for overlay entities (extend this to match your overlay components)
    overlay_query: Query<
        Entity,
        (
            Or<(
                With<crate::ui::command_palette::CommandPaletteState>,
                With<crate::ui::search_overlay::SearchOverlayState>,
                With<crate::ui::overlays::RemoteOverlay>,
            )>,
            Without<EffectsTarget>,
        ),
    >,
    // Query for existing effects targets
    effects_targets: Query<(Entity, &EffectsTarget)>,
) {
    // Check if effects are enabled
    let should_blur = config.effects.should_render_blur();
    let should_glow = config.effects.should_render_glow();

    // Add effects to new overlays
    for entity in overlay_query.iter() {
        commands.entity(entity).insert(EffectsTarget {
            apply_blur: should_blur,
            apply_glow: should_glow,
        });
    }

    // Update existing effects targets if config changed
    for (entity, target) in effects_targets.iter() {
        if target.apply_blur != should_blur || target.apply_glow != should_glow {
            commands.entity(entity).insert(EffectsTarget {
                apply_blur: should_blur,
                apply_glow: should_glow,
            });
        }
    }
}

/// Resource to track whether any overlays are currently visible
///
/// This is used to optimize rendering by only applying effects when needed.
#[derive(Resource, Default)]
pub struct OverlayVisibilityState {
    pub any_overlays_visible: bool,
    pub focused_overlay: Option<Entity>,
}

/// System to track overlay visibility
pub fn update_overlay_visibility(
    mut visibility_state: ResMut<OverlayVisibilityState>,
    overlay_query: Query<
        (Entity, &Visibility),
        Or<(
            With<crate::ui::command_palette::CommandPaletteState>,
            With<crate::ui::search_overlay::SearchOverlayState>,
            With<crate::ui::overlays::RemoteOverlay>,
        )>,
    >,
) {
    let mut any_visible = false;
    let mut focused = None;

    for (entity, visibility) in overlay_query.iter() {
        if *visibility != Visibility::Hidden {
            any_visible = true;
            // For now, just use the first visible overlay as focused
            // In a real implementation, track focus properly
            if focused.is_none() {
                focused = Some(entity);
            }
        }
    }

    visibility_state.any_overlays_visible = any_visible;
    visibility_state.focused_overlay = focused;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_effects_target_default() {
        let target = EffectsTarget::default();
        assert!(!target.apply_blur);
        assert!(!target.apply_glow);
    }

    #[test]
    fn test_plugin_builds() {
        let mut app = App::new();
        app.add_plugins(ScarabEffectsPlugin);
        // Plugin should add without panicking
    }
}
