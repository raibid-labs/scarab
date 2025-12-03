//! HUD Overlay Rendering
//!
//! This module handles the visual rendering of the telemetry HUD overlay.
//! It uses Bevy's UI system to display performance metrics in a configurable position.

use crate::metrics::PerformanceMetrics;
use bevy::prelude::*;

/// HUD position on screen
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HudPosition {
    TopRight,
    TopLeft,
    BottomRight,
    BottomLeft,
}

/// HUD state resource
///
/// Controls the visibility and position of the performance HUD.
#[derive(Resource)]
pub struct HudState {
    /// Whether the HUD is currently visible
    pub visible: bool,

    /// Position of the HUD on screen
    pub position: HudPosition,
}

/// Marker component for the HUD container
#[derive(Component)]
pub struct HudContainer;

/// Marker component for the HUD text elements
#[derive(Component)]
pub struct HudText;

/// Marker component for the frame time graph
#[derive(Component)]
pub struct HudGraph;

/// System: Toggle HUD visibility with F12
pub(crate) fn toggle_hud(keys: Res<ButtonInput<KeyCode>>, mut state: ResMut<HudState>) {
    if keys.just_pressed(KeyCode::F12) {
        state.visible = !state.visible;
        if state.visible {
            info!("Telemetry HUD enabled");
        } else {
            info!("Telemetry HUD disabled");
        }
    }
}

/// System: Render the HUD overlay
///
/// Creates or updates the HUD UI based on current metrics.
/// Only renders when the HUD is visible.
pub(crate) fn render_hud(
    mut commands: Commands,
    state: Res<HudState>,
    metrics: Res<PerformanceMetrics>,
    container_query: Query<Entity, With<HudContainer>>,
    text_query: Query<Entity, With<HudText>>,
) {
    // If HUD should not be visible, despawn any existing UI
    if !state.visible {
        for entity in container_query.iter() {
            commands.entity(entity).despawn_recursive();
        }
        return;
    }

    // If container doesn't exist, spawn it
    if container_query.is_empty() {
        spawn_hud(&mut commands, &state);
        return;
    }

    // Update existing HUD text with current metrics
    for entity in text_query.iter() {
        update_hud_text(&mut commands, entity, &metrics);
    }
}

/// Spawn the HUD UI hierarchy
fn spawn_hud(commands: &mut Commands, state: &HudState) {
    let (justify_content, align_items) = match state.position {
        HudPosition::TopRight => (JustifyContent::FlexEnd, AlignItems::FlexStart),
        HudPosition::TopLeft => (JustifyContent::FlexStart, AlignItems::FlexStart),
        HudPosition::BottomRight => (JustifyContent::FlexEnd, AlignItems::FlexEnd),
        HudPosition::BottomLeft => (JustifyContent::FlexStart, AlignItems::FlexEnd),
    };

    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content,
                align_items,
                padding: UiRect::all(Val::Px(16.0)),
                ..default()
            },
            HudContainer,
            // Make container non-interactive
            GlobalZIndex(999),
        ))
        .with_children(|parent| {
            // HUD panel with semi-transparent background
            parent
                .spawn((
                    Node {
                        flex_direction: FlexDirection::Column,
                        padding: UiRect::all(Val::Px(12.0)),
                        row_gap: Val::Px(4.0),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.85)),
                    BorderRadius::all(Val::Px(8.0)),
                ))
                .with_children(|panel| {
                    // Title
                    panel.spawn((
                        Text::new("TELEMETRY"),
                        TextFont {
                            font_size: 14.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.7, 0.9, 1.0)),
                        HudText,
                    ));

                    // Performance metrics text - will be updated each frame
                    panel.spawn((
                        Text::new("Initializing..."),
                        TextFont {
                            font_size: 12.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.9, 0.9, 0.9)),
                        HudText,
                    ));
                });
        });
}

/// Update HUD text with current metrics
fn update_hud_text(commands: &mut Commands, entity: Entity, metrics: &PerformanceMetrics) {
    let snapshot = metrics.snapshot();

    // Format metrics text
    let text = format!(
        "FPS: {:.0}\n\
         Frame Time: {:.2}ms\n\
         Avg: {:.2}ms  Min: {:.2}ms  Max: {:.2}ms\n\
         Frames: {}  Uptime: {:.1}s",
        snapshot.current_fps,
        snapshot.current_frame_time_ms,
        snapshot.avg_frame_time_ms,
        snapshot.min_frame_time_ms,
        snapshot.max_frame_time_ms,
        snapshot.total_frames,
        snapshot.total_elapsed_secs,
    );

    // Update the text component
    commands.entity(entity).insert(Text::new(text));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hud_state_creation() {
        let state = HudState {
            visible: true,
            position: HudPosition::TopRight,
        };

        assert!(state.visible);
        assert_eq!(state.position, HudPosition::TopRight);
    }

    #[test]
    fn test_hud_position_variants() {
        let positions = vec![
            HudPosition::TopRight,
            HudPosition::TopLeft,
            HudPosition::BottomRight,
            HudPosition::BottomLeft,
        ];

        for pos in positions {
            let state = HudState {
                visible: true,
                position: pos,
            };
            assert_eq!(state.position, pos);
        }
    }
}
