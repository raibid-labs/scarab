//! Unified Hint Overlay System for Scarab Terminal Emulator
//!
//! This module provides a pure Bevy ECS-based hint rendering system that replaces
//! the previous RemoteCommand::DrawOverlay approach. It integrates with the
//! navigation module to display Vimium-style keyboard hints over the terminal.
//!
//! ## Features
//!
//! - **ECS-native rendering**: Uses Text2d entities and sprites for hint display
//! - **Automatic sync**: Spawns/despawns hint overlays based on NavHint entities
//! - **Filter-based styling**: Dims non-matching hints, highlights matching ones
//! - **Fade animations**: Smooth transitions when entering/exiting hint mode
//! - **Configurable appearance**: Resource-based styling configuration
//! - **Z-layer management**: Ensures hints appear above terminal but below modals
//!
//! ## Architecture
//!
//! The system works by observing NavHint components from the navigation module
//! and creating corresponding visual HintOverlay entities. This decouples the
//! logical navigation state from the rendering concerns.
//!
//! ## Usage Example
//!
//! ```rust,ignore
//! use bevy::prelude::*;
//! use scarab_client::rendering::hint_overlay::HintOverlayPlugin;
//!
//! App::new()
//!     .add_plugins(DefaultPlugins)
//!     .add_plugins(HintOverlayPlugin)
//!     .run();
//! ```

use super::layers::LAYER_HINTS;
use crate::navigation::{EnterHintModeEvent, ExitHintModeEvent, NavHint, NavSystemSet};
use bevy::prelude::*;

// ==================== Components ====================

/// Component marking an entity as a visual hint overlay
///
/// Each HintOverlay entity corresponds to a NavHint entity and handles
/// the visual rendering of the hint label with background.
#[derive(Component, Debug, Clone)]
pub struct HintOverlay {
    /// The hint label to display (e.g., "a", "ab", "xy")
    pub label: String,

    /// Screen position in pixel coordinates
    pub position: Vec2,

    /// Background color for the hint badge
    pub background_color: Color,

    /// Text color for the hint label
    pub text_color: Color,

    /// True if this hint matches the current filter input
    pub matched: bool,

    /// Z-layer for rendering order (uses LAYER_HINTS constant)
    pub z_layer: f32,

    /// Reference to the NavHint entity this overlay represents
    pub nav_hint_entity: Entity,
}

impl Default for HintOverlay {
    fn default() -> Self {
        Self {
            label: String::new(),
            position: Vec2::ZERO,
            background_color: Color::srgba(1.0, 0.75, 0.0, 0.9), // Amber/yellow
            text_color: Color::srgb(0.0, 0.0, 0.0),              // Black
            matched: false,
            z_layer: LAYER_HINTS,
            nav_hint_entity: Entity::PLACEHOLDER,
        }
    }
}

/// Marker component for the background sprite of a hint overlay
#[derive(Component)]
struct HintBackground;

/// Marker component for the text of a hint overlay
#[derive(Component)]
struct HintText;

/// Component for tracking fade animation state
///
/// Controls the opacity animation when hints fade in or out. Used internally
/// by the hint overlay system to provide smooth visual transitions.
#[derive(Component, Debug, Clone)]
pub struct HintFade {
    /// Current opacity (0.0 to 1.0)
    pub opacity: f32,

    /// Target opacity
    pub target_opacity: f32,

    /// Fade speed (units per second)
    pub fade_speed: f32,
}

impl Default for HintFade {
    fn default() -> Self {
        Self {
            opacity: 0.0,
            target_opacity: 1.0,
            fade_speed: 5.0, // Fade in/out over 0.2 seconds
        }
    }
}

// ==================== Resources ====================

/// Configuration resource for hint overlay styling
///
/// Use this resource to customize the appearance of hint overlays across
/// the entire application. Changes take effect on the next render pass.
#[derive(Resource, Debug, Clone)]
pub struct HintOverlayConfig {
    /// Default background color for unmatched hints
    pub background_color: Color,

    /// Default text color for hint labels
    pub text_color: Color,

    /// Background color when hint is matched by filter
    pub matched_color: Color,

    /// Background color for partially matched hints (filter prefix matches)
    pub partial_match_color: Color,

    /// Text color when hint is dimmed (doesn't match filter)
    pub dimmed_text_color: Color,

    /// Font size for hint labels in pixels
    pub font_size: f32,

    /// Padding around text in the hint badge (horizontal, vertical)
    pub padding: Vec2,

    /// Border radius for rounded corners (not implemented yet)
    pub border_radius: f32,

    /// Opacity for dimmed hints (0.0 to 1.0)
    pub dimmed_opacity: f32,

    /// Z-layer for hint overlays (uses LAYER_HINTS: above images, below modals)
    pub z_layer: f32,
}

impl Default for HintOverlayConfig {
    fn default() -> Self {
        Self {
            background_color: Color::srgba(1.0, 0.75, 0.0, 0.9), // Amber
            text_color: Color::srgb(0.0, 0.0, 0.0),              // Black
            matched_color: Color::srgba(0.0, 1.0, 0.0, 0.95),    // Green
            partial_match_color: Color::srgba(1.0, 0.85, 0.0, 0.9), // Lighter yellow
            dimmed_text_color: Color::srgba(0.3, 0.3, 0.3, 0.6), // Gray
            font_size: 14.0,
            padding: Vec2::new(6.0, 4.0),
            border_radius: 3.0,
            dimmed_opacity: 0.4,
            z_layer: LAYER_HINTS,
        }
    }
}

// ==================== Bundles ====================

/// Bundle for spawning a complete hint overlay entity
///
/// This bundle includes all necessary components to render a hint overlay
/// with background and text. The background is a colored sprite and the
/// text is a Text2d entity.
#[derive(Bundle)]
pub struct HintOverlayBundle {
    /// HintOverlay component with configuration
    pub hint: HintOverlay,

    /// Fade animation component
    pub fade: HintFade,

    /// Spatial transform for positioning
    pub transform: Transform,

    /// Global transform (computed by Bevy)
    pub global_transform: GlobalTransform,

    /// Visibility control
    pub visibility: Visibility,

    /// Inherited visibility (computed by Bevy)
    pub inherited_visibility: InheritedVisibility,

    /// View visibility (computed by Bevy)
    pub view_visibility: ViewVisibility,
}

impl HintOverlayBundle {
    /// Create a new hint overlay bundle from a HintOverlay component
    pub fn new(hint: HintOverlay) -> Self {
        let z = hint.z_layer;
        let pos = hint.position;

        Self {
            hint,
            fade: HintFade::default(),
            transform: Transform::from_translation(pos.extend(z)),
            global_transform: GlobalTransform::default(),
            visibility: Visibility::Visible,
            inherited_visibility: InheritedVisibility::default(),
            view_visibility: ViewVisibility::default(),
        }
    }
}

// ==================== Systems ====================

/// System: Render hint overlays based on NavHint entities
///
/// This system queries all NavHint entities and creates corresponding
/// HintOverlay visual entities. It maintains a 1:1 mapping between
/// NavHint (logical) and HintOverlay (visual).
///
/// Runs in: NavSystemSet::Render
fn render_hint_overlays(
    mut commands: Commands,
    nav_registry: Res<crate::NavStateRegistry>,
    config: Res<HintOverlayConfig>,
    nav_hints: Query<(Entity, &NavHint)>,
    existing_overlays: Query<(Entity, &HintOverlay)>,
    asset_server: Res<AssetServer>,
) {
    // Get the active pane's NavState
    let nav_state = match nav_registry.get_active() {
        Some(state) => state,
        None => return, // No active pane
    };

    // Only render hints when in hint mode
    if !nav_state.is_hint_mode() {
        return;
    }

    // Build a set of existing overlay entities for efficient lookup
    let existing_map: std::collections::HashMap<Entity, Entity> = existing_overlays
        .iter()
        .map(|(entity, overlay)| (overlay.nav_hint_entity, entity))
        .collect();

    // Create or update overlays for each NavHint
    for (nav_entity, nav_hint) in nav_hints.iter() {
        // Check if overlay already exists for this NavHint
        if existing_map.contains_key(&nav_entity) {
            continue; // Already rendered, skip
        }

        // Determine styling based on filter match
        let (bg_color, text_color, matched) = if nav_state.hint_filter.is_empty() {
            // No filter, show default colors
            (config.background_color, config.text_color, false)
        } else if nav_hint.label == nav_state.hint_filter {
            // Exact match
            (config.matched_color, config.text_color, true)
        } else if nav_hint.label.starts_with(&nav_state.hint_filter) {
            // Partial match
            (config.partial_match_color, config.text_color, false)
        } else {
            // No match, will be filtered out by update system
            continue;
        };

        // Create the overlay component
        let overlay = HintOverlay {
            label: nav_hint.label.clone(),
            position: nav_hint.position,
            background_color: bg_color,
            text_color,
            matched,
            z_layer: config.z_layer,
            nav_hint_entity: nav_entity,
        };

        // Spawn the hint overlay entity with background and text as children
        let overlay_entity = commands.spawn(HintOverlayBundle::new(overlay.clone())).id();

        // Spawn background sprite as child
        let bg_size = Vec2::new(
            config.font_size + config.padding.x * 2.0,
            config.font_size + config.padding.y * 2.0,
        );

        commands.entity(overlay_entity).with_children(|parent| {
            // Background sprite
            parent.spawn((
                HintBackground,
                Sprite {
                    color: overlay.background_color,
                    custom_size: Some(bg_size),
                    ..default()
                },
                Transform::from_translation(Vec3::new(bg_size.x / 2.0, -bg_size.y / 2.0, -0.1)),
            ));

            // Text label
            parent.spawn((
                HintText,
                Text2d::new(&overlay.label),
                TextFont {
                    font: asset_server.load("fonts/FiraCode-Regular.ttf"),
                    font_size: config.font_size,
                    ..default()
                },
                TextColor(overlay.text_color),
                Transform::from_translation(Vec3::new(bg_size.x / 2.0, -bg_size.y / 2.0, 0.0)),
                TextLayout::new_with_justify(JustifyText::Center),
            ));
        });
    }
}

/// System: Update hint overlay styling based on filter changes
///
/// This system responds to changes in NavState.hint_filter and updates
/// the colors and visibility of existing hint overlays.
///
/// Runs in: NavSystemSet::Update
fn update_hint_overlays(
    mut commands: Commands,
    nav_registry: Res<crate::NavStateRegistry>,
    config: Res<HintOverlayConfig>,
    mut overlays: Query<(Entity, &mut HintOverlay, &Children)>,
    nav_hints: Query<&NavHint>,
    mut backgrounds: Query<&mut Sprite, With<HintBackground>>,
    mut texts: Query<&mut TextColor, (With<HintText>, Without<HintBackground>)>,
) {
    // Get the active pane's NavState
    let nav_state = match nav_registry.get_active() {
        Some(state) => state,
        None => return, // No active pane
    };

    if !nav_state.is_hint_mode() {
        return;
    }

    for (overlay_entity, mut overlay, children) in overlays.iter_mut() {
        // Get the corresponding NavHint to ensure it still exists
        let nav_hint = match nav_hints.get(overlay.nav_hint_entity) {
            Ok(hint) => hint,
            Err(_) => {
                // NavHint no longer exists, despawn this overlay
                commands.entity(overlay_entity).despawn_recursive();
                continue;
            }
        };

        // Determine if this hint matches the current filter
        let should_show = if nav_state.hint_filter.is_empty() {
            true
        } else {
            nav_hint.label.starts_with(&nav_state.hint_filter)
        };

        if !should_show {
            // Hide non-matching hints
            commands.entity(overlay_entity).despawn_recursive();
            continue;
        }

        // Update styling based on match type
        let (bg_color, text_color, matched) = if nav_hint.label == nav_state.hint_filter {
            (config.matched_color, config.text_color, true)
        } else if nav_hint.label.starts_with(&nav_state.hint_filter) {
            (config.partial_match_color, config.text_color, false)
        } else {
            (config.background_color, config.text_color, false)
        };

        overlay.background_color = bg_color;
        overlay.text_color = text_color;
        overlay.matched = matched;

        // Update child components
        for &child in children.iter() {
            if let Ok(mut sprite) = backgrounds.get_mut(child) {
                sprite.color = bg_color;
            }
            if let Ok(mut text_color_comp) = texts.get_mut(child) {
                text_color_comp.0 = text_color;
            }
        }
    }
}

/// System: Animate hint fade in/out
///
/// Smoothly fades hint overlays in when entering hint mode and out when
/// exiting. Uses the HintFade component to track animation state.
///
/// Runs in: NavSystemSet::Render
fn animate_hint_fade(
    time: Res<Time>,
    mut overlays: Query<(&mut HintFade, &Children)>,
    mut backgrounds: Query<&mut Sprite, With<HintBackground>>,
    mut texts: Query<&mut TextColor, (With<HintText>, Without<HintBackground>)>,
) {
    let delta = time.delta_secs();

    for (mut fade, children) in overlays.iter_mut() {
        // Update opacity towards target
        if (fade.opacity - fade.target_opacity).abs() > 0.01 {
            let direction = (fade.target_opacity - fade.opacity).signum();
            fade.opacity += direction * fade.fade_speed * delta;
            fade.opacity = fade.opacity.clamp(0.0, 1.0);

            // Apply opacity to children
            for &child in children.iter() {
                if let Ok(mut sprite) = backgrounds.get_mut(child) {
                    sprite.color = sprite.color.with_alpha(sprite.color.alpha() * fade.opacity);
                }
                if let Ok(mut text_color) = texts.get_mut(child) {
                    text_color.0 = text_color.0.with_alpha(text_color.0.alpha() * fade.opacity);
                }
            }
        }
    }
}

/// System: Cleanup hint overlays when exiting hint mode
///
/// Listens for ExitHintModeEvent and despawns all HintOverlay entities.
/// This ensures a clean slate when re-entering hint mode.
///
/// Runs in: NavSystemSet::Update
fn cleanup_hint_overlays(
    mut commands: Commands,
    mut exit_events: EventReader<ExitHintModeEvent>,
    overlays: Query<Entity, With<HintOverlay>>,
) {
    for _event in exit_events.read() {
        // Despawn all hint overlay entities
        for entity in overlays.iter() {
            commands.entity(entity).despawn_recursive();
        }
    }
}

/// System: Initialize fade-in when entering hint mode
///
/// Listens for EnterHintModeEvent and sets up fade animations for all
/// hint overlays. This provides a smooth visual transition.
///
/// Runs in: NavSystemSet::Update
fn init_hint_fade_in(
    mut enter_events: EventReader<EnterHintModeEvent>,
    mut overlays: Query<&mut HintFade>,
) {
    for _event in enter_events.read() {
        // Set all overlays to fade in
        for mut fade in overlays.iter_mut() {
            fade.opacity = 0.0;
            fade.target_opacity = 1.0;
        }
    }
}

// ==================== Plugin ====================

/// Plugin that adds the hint overlay rendering system
///
/// This plugin integrates with the NavigationPlugin to provide visual
/// rendering of keyboard hints. It registers all necessary resources,
/// components, and systems.
///
/// # Dependencies
///
/// This plugin requires NavigationPlugin to be added first, as it depends
/// on navigation events and components.
///
/// # Example
///
/// ```rust,ignore
/// use bevy::prelude::*;
/// use scarab_client::navigation::NavigationPlugin;
/// use scarab_client::rendering::hint_overlay::HintOverlayPlugin;
///
/// App::new()
///     .add_plugins(DefaultPlugins)
///     .add_plugins(NavigationPlugin)
///     .add_plugins(HintOverlayPlugin)
///     .run();
/// ```
pub struct HintOverlayPlugin;

impl Plugin for HintOverlayPlugin {
    fn build(&self, app: &mut App) {
        app
            // Register configuration resource
            .init_resource::<HintOverlayConfig>()
            // Register systems in the appropriate sets
            .add_systems(
                Update,
                (
                    // Update phase: handle events and state changes
                    cleanup_hint_overlays.in_set(NavSystemSet::Update),
                    init_hint_fade_in.in_set(NavSystemSet::Update),
                    update_hint_overlays.in_set(NavSystemSet::Update),
                    // Render phase: create visual entities
                    render_hint_overlays.in_set(NavSystemSet::Render),
                    animate_hint_fade.in_set(NavSystemSet::Render),
                ),
            );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hint_overlay_default() {
        let overlay = HintOverlay::default();
        assert_eq!(overlay.label, "");
        assert_eq!(overlay.position, Vec2::ZERO);
        assert!(!overlay.matched);
        assert_eq!(overlay.z_layer, LAYER_HINTS);
    }

    #[test]
    fn test_hint_fade_default() {
        let fade = HintFade::default();
        assert_eq!(fade.opacity, 0.0);
        assert_eq!(fade.target_opacity, 1.0);
        assert_eq!(fade.fade_speed, 5.0);
    }

    #[test]
    fn test_config_default() {
        let config = HintOverlayConfig::default();
        assert_eq!(config.font_size, 14.0);
        assert_eq!(config.z_layer, LAYER_HINTS);
        assert_eq!(config.padding, Vec2::new(6.0, 4.0));
    }

    #[test]
    fn test_hint_overlay_bundle_creation() {
        let hint = HintOverlay {
            label: "ab".to_string(),
            position: Vec2::new(100.0, 200.0),
            z_layer: 150.0,
            ..default()
        };

        let bundle = HintOverlayBundle::new(hint.clone());
        assert_eq!(bundle.hint.label, "ab");
        assert_eq!(bundle.transform.translation.x, 100.0);
        assert_eq!(bundle.transform.translation.y, 200.0);
        assert_eq!(bundle.transform.translation.z, 150.0);
    }
}
