// Example demonstrating the Ratatui bridge with rendering
// This shows how to create surfaces and render Ratatui widgets as Bevy overlays
//
// Task C1: Surface infrastructure (buffers, components)
// Task C2: Rendering system (buffer → Bevy meshes/overlays) - COMPLETE
//
// Run with: cargo run -p scarab-client --example ratatui_bridge_demo

use bevy::prelude::*;
use scarab_client::ratatui_bridge::{Buffer, RatatuiBridgePlugin, RatatuiSurface, SurfaceBuffers};
use ratatui::widgets::{Block, Borders, Widget};
use scarab_protocol::TerminalMetrics;

fn main() {
    App::new()
        .add_plugins((MinimalPlugins, RatatuiBridgePlugin))
        .insert_resource(TerminalMetrics::default()) // Required for rendering
        .add_systems(Startup, setup_surfaces)
        .add_systems(Update, render_widgets)
        .run();
}

/// Spawn some example surfaces
fn setup_surfaces(mut commands: Commands) {
    info!("Setting up Ratatui surfaces...");

    // Create a command palette surface (centered, 60x10 cells)
    commands.spawn((
        RatatuiSurface::new(10, 5, 60, 10).with_z_index(200.0),
        CommandPaletteSurface,
    ));

    // Create a notification surface (top-right, 30x5 cells)
    commands.spawn((
        RatatuiSurface::new(150, 0, 30, 5).with_z_index(150.0),
        NotificationSurface,
    ));

    // Create a hidden status bar (bottom, full width, initially hidden)
    commands.spawn((
        RatatuiSurface::new(0, 95, 200, 5)
            .with_z_index(50.0)
            .hidden(),
        StatusBarSurface,
    ));

    info!("Surfaces created successfully - overlays will be rendered automatically");
}

/// Marker components for different surface types
#[derive(Component)]
struct CommandPaletteSurface;

#[derive(Component)]
struct NotificationSurface;

#[derive(Component)]
struct StatusBarSurface;

/// Render Ratatui widgets to surface buffers
///
/// The RatatuiBridgePlugin automatically converts these buffers to Bevy overlays
/// with proper positioning and z-ordering.
fn render_widgets(
    mut buffers: ResMut<SurfaceBuffers>,
    mut command_palette_query: Query<
        (Entity, &mut RatatuiSurface),
        (With<CommandPaletteSurface>, Changed<RatatuiSurface>),
    >,
    mut notification_query: Query<
        (Entity, &mut RatatuiSurface),
        (With<NotificationSurface>, Changed<RatatuiSurface>),
    >,
) {
    // Render command palette
    for (entity, mut surface) in command_palette_query.iter_mut() {
        if surface.visible && surface.dirty {
            let buffer = buffers.get_or_create(entity, surface.width, surface.height);

            // Render a Ratatui block widget
            let widget = Block::default()
                .title("Command Palette")
                .borders(Borders::ALL)
                .border_style(ratatui::style::Style::default().fg(ratatui::style::Color::Cyan));

            widget.render(surface.rect(), buffer);

            surface.mark_clean();
            info!(
                "Rendered command palette at ({}, {}) size {}x{} - overlay will appear at z=200",
                surface.x, surface.y, surface.width, surface.height
            );
        }
    }

    // Render notification
    for (entity, mut surface) in notification_query.iter_mut() {
        if surface.visible && surface.dirty {
            let buffer = buffers.get_or_create(entity, surface.width, surface.height);

            let widget = Block::default()
                .title("Notification")
                .borders(Borders::ALL)
                .border_style(ratatui::style::Style::default().fg(ratatui::style::Color::Yellow));

            widget.render(surface.rect(), buffer);

            surface.mark_clean();
            info!(
                "Rendered notification at ({}, {}) size {}x{} - overlay will appear at z=150",
                surface.x, surface.y, surface.width, surface.height
            );
        }
    }
}
