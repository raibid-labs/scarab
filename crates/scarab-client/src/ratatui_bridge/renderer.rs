// Ratatui Surface Rendering System
//
// This module converts Ratatui buffers to Bevy visual overlays, allowing
// Ratatui widgets to be displayed above the terminal grid content.

use super::surface::{RatatuiSurface, SurfaceBuffers};
use bevy::prelude::*;
use bevy::render::mesh::{Indices, Mesh2d, PrimitiveTopology};
use bevy::render::render_asset::RenderAssetUsages;
use bevy::sprite::{ColorMaterial, MeshMaterial2d};
use ratatui::style::Color as RatColor;
use scarab_protocol::TerminalMetrics;

/// Marker component for surface overlay entities
///
/// Each Ratatui surface spawns a corresponding overlay entity that contains
/// the visual representation. This component links the overlay back to its
/// source surface for lifecycle management.
#[derive(Component, Debug)]
pub struct SurfaceOverlay {
    /// The surface entity this overlay belongs to
    pub surface_entity: Entity,
}

/// Convert Ratatui color to Bevy color
///
/// Handles the full Ratatui color palette including:
/// - Named colors (16 standard + variants)
/// - RGB colors
/// - 256-color indexed palette
fn ratatui_to_bevy_color(color: RatColor) -> Color {
    match color {
        RatColor::Reset => Color::srgba(0.0, 0.0, 0.0, 0.0),
        RatColor::Black => Color::srgb(0.0, 0.0, 0.0),
        RatColor::Red => Color::srgb(0.8, 0.0, 0.0),
        RatColor::Green => Color::srgb(0.0, 0.8, 0.0),
        RatColor::Yellow => Color::srgb(0.8, 0.8, 0.0),
        RatColor::Blue => Color::srgb(0.0, 0.0, 0.8),
        RatColor::Magenta => Color::srgb(0.8, 0.0, 0.8),
        RatColor::Cyan => Color::srgb(0.0, 0.8, 0.8),
        RatColor::Gray => Color::srgb(0.5, 0.5, 0.5),
        RatColor::DarkGray => Color::srgb(0.3, 0.3, 0.3),
        RatColor::LightRed => Color::srgb(1.0, 0.3, 0.3),
        RatColor::LightGreen => Color::srgb(0.3, 1.0, 0.3),
        RatColor::LightYellow => Color::srgb(1.0, 1.0, 0.3),
        RatColor::LightBlue => Color::srgb(0.3, 0.3, 1.0),
        RatColor::LightMagenta => Color::srgb(1.0, 0.3, 1.0),
        RatColor::LightCyan => Color::srgb(0.3, 1.0, 1.0),
        RatColor::White => Color::srgb(1.0, 1.0, 1.0),
        RatColor::Rgb(r, g, b) => Color::srgb(r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0),
        RatColor::Indexed(idx) => indexed_color(idx),
    }
}

/// Convert 256-color index to RGB
///
/// The 256-color palette is structured as:
/// - 0-15: Standard colors (handled by named color mapping)
/// - 16-231: 6x6x6 RGB color cube
/// - 232-255: 24-step grayscale ramp
fn indexed_color(idx: u8) -> Color {
    if idx < 16 {
        // Standard colors - simplified mapping
        Color::srgb(0.5, 0.5, 0.5)
    } else if idx < 232 {
        // 216 color cube (6x6x6)
        let idx = idx - 16;
        let r = (idx / 36) % 6;
        let g = (idx / 6) % 6;
        let b = idx % 6;
        Color::srgb(r as f32 / 5.0, g as f32 / 5.0, b as f32 / 5.0)
    } else {
        // Grayscale ramp
        let gray = (idx - 232) as f32 / 23.0;
        Color::srgb(gray, gray, gray)
    }
}

/// Create a rectangle mesh for surface backgrounds
///
/// Generates a simple quad mesh with the specified dimensions.
fn create_rectangle_mesh(width: f32, height: f32) -> Mesh {
    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
    );

    // Vertices for a rectangle (2 triangles)
    // Origin at top-left, extends right and down (negative Y)
    let vertices = vec![
        [0.0, 0.0, 0.0],       // Top-left
        [width, 0.0, 0.0],     // Top-right
        [width, -height, 0.0], // Bottom-right
        [0.0, -height, 0.0],   // Bottom-left
    ];

    // UV coordinates (not needed for solid color, but required by some materials)
    let uvs = vec![[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]];

    // Normals (facing camera)
    let normals = vec![
        [0.0, 0.0, 1.0],
        [0.0, 0.0, 1.0],
        [0.0, 0.0, 1.0],
        [0.0, 0.0, 1.0],
    ];

    // Triangle indices (CCW winding)
    let indices = vec![0u32, 1, 2, 0, 2, 3];

    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertices);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_indices(Indices::U32(indices));

    mesh
}

/// System to render Ratatui surfaces as Bevy overlays
///
/// This system:
/// 1. Monitors changed surfaces (position, size, visibility)
/// 2. Creates or updates overlay entities with background meshes
/// 3. Positions overlays in screen space with proper z-ordering
///
/// # Future Work
/// - Text rendering via cosmic-text integration
/// - Cell-by-cell styling (colors, modifiers)
/// - Border rendering for widgets
pub fn render_surfaces(
    mut commands: Commands,
    buffers: Res<SurfaceBuffers>,
    metrics: Res<TerminalMetrics>,
    surfaces: Query<(Entity, &RatatuiSurface), Changed<RatatuiSurface>>,
    mut overlays: Query<(Entity, &SurfaceOverlay, &mut Transform, &mut Visibility)>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    for (surface_entity, surface) in surfaces.iter() {
        // Handle visibility changes
        if !surface.visible {
            // Hide existing overlay if any
            for (_, overlay, _, mut visibility) in overlays.iter_mut() {
                if overlay.surface_entity == surface_entity {
                    *visibility = Visibility::Hidden;
                }
            }
            continue;
        }

        // Get buffer for this surface (might not exist yet)
        let _buffer = buffers.get(&surface_entity);

        // Calculate screen position from grid coordinates
        let (screen_x, screen_y) = metrics.grid_to_screen(surface.x, surface.y);
        let z = surface.z_index;

        // Calculate surface dimensions in pixels
        let width = surface.width as f32 * metrics.cell_width;
        let height = surface.height as f32 * metrics.cell_height;

        // Check if overlay already exists
        let overlay_exists = overlays
            .iter()
            .any(|(_, o, _, _)| o.surface_entity == surface_entity);

        if !overlay_exists {
            // Spawn new overlay entity with background mesh
            let mesh = create_rectangle_mesh(width, height);
            let mesh_handle = meshes.add(mesh);

            // Semi-transparent dark background for overlays
            let material = materials.add(ColorMaterial::from(Color::srgba(0.1, 0.1, 0.1, 0.9)));

            // Position: grid coordinates converted to screen space
            // Y is flipped because Bevy uses Y-up but we use Y-down for terminal coordinates
            commands.spawn((
                SurfaceOverlay { surface_entity },
                Mesh2d(mesh_handle),
                MeshMaterial2d(material),
                Transform::from_xyz(
                    screen_x, -screen_y, // Flip Y for Bevy's coordinate system
                    z,
                ),
                Visibility::Visible,
            ));

            trace!(
                "Spawned overlay for surface {:?} at grid ({}, {}) -> screen ({:.2}, {:.2}) z={:.1}",
                surface_entity,
                surface.x,
                surface.y,
                screen_x,
                screen_y,
                z
            );
        } else {
            // Update existing overlay position/visibility
            for (_, overlay, mut transform, mut visibility) in overlays.iter_mut() {
                if overlay.surface_entity == surface_entity {
                    // Update transform if position or z-index changed
                    let new_x = screen_x;
                    let new_y = -screen_y;
                    let new_z = z;

                    if transform.translation.x != new_x
                        || transform.translation.y != new_y
                        || transform.translation.z != new_z
                    {
                        transform.translation.x = new_x;
                        transform.translation.y = new_y;
                        transform.translation.z = new_z;

                        trace!(
                            "Updated overlay position for surface {:?} to ({:.2}, {:.2}, {:.1})",
                            surface_entity,
                            new_x,
                            new_y,
                            new_z
                        );
                    }

                    // Ensure visibility is correct
                    if *visibility != Visibility::Visible {
                        *visibility = Visibility::Visible;
                    }
                }
            }
        }

        // TODO: Render buffer cells as text
        // This requires integration with the existing cosmic-text rendering pipeline
        // For now, we just render the background mesh to prove the overlay system works
    }
}

/// System to clean up overlays when surfaces are despawned
///
/// This ensures we don't leak overlay entities when surfaces are removed.
/// Runs automatically as part of the RatatuiBridgePlugin.
pub fn cleanup_overlays(
    mut commands: Commands,
    mut removed: RemovedComponents<RatatuiSurface>,
    overlays: Query<(Entity, &SurfaceOverlay)>,
) {
    for surface_entity in removed.read() {
        for (overlay_entity, overlay) in overlays.iter() {
            if overlay.surface_entity == surface_entity {
                commands.entity(overlay_entity).despawn_recursive();
                trace!("Despawned overlay for removed surface {:?}", surface_entity);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ratatui_color_conversion() {
        // Test basic colors
        let black = ratatui_to_bevy_color(RatColor::Black);
        assert_eq!(black, Color::srgb(0.0, 0.0, 0.0));

        let white = ratatui_to_bevy_color(RatColor::White);
        assert_eq!(white, Color::srgb(1.0, 1.0, 1.0));

        // Test RGB color
        let custom = ratatui_to_bevy_color(RatColor::Rgb(128, 64, 192));
        assert_eq!(
            custom,
            Color::srgb(128.0 / 255.0, 64.0 / 255.0, 192.0 / 255.0)
        );
    }

    #[test]
    fn test_indexed_color_ranges() {
        // Standard colors (0-15)
        let _std = indexed_color(7);

        // Color cube (16-231)
        let cube = indexed_color(16); // First color in cube
        assert!(cube.to_srgba().red >= 0.0 && cube.to_srgba().red <= 1.0);

        // Grayscale (232-255)
        let gray = indexed_color(244); // Mid-gray
        let rgba = gray.to_srgba();
        // Grayscale should have equal R, G, B components
        assert!((rgba.red - rgba.green).abs() < 0.01);
        assert!((rgba.green - rgba.blue).abs() < 0.01);
    }

    #[test]
    fn test_rectangle_mesh_creation() {
        let mesh = create_rectangle_mesh(100.0, 50.0);

        // Verify mesh has required attributes
        assert!(mesh.attribute(Mesh::ATTRIBUTE_POSITION).is_some());
        assert!(mesh.attribute(Mesh::ATTRIBUTE_UV_0).is_some());
        assert!(mesh.attribute(Mesh::ATTRIBUTE_NORMAL).is_some());

        // Verify indices exist
        assert!(mesh.indices().is_some());

        // Should have 6 indices (2 triangles)
        if let Some(Indices::U32(indices)) = mesh.indices() {
            assert_eq!(indices.len(), 6);
        } else {
            panic!("Expected U32 indices");
        }
    }

    #[test]
    fn test_surface_overlay_component() {
        let entity = Entity::from_raw(42);
        let overlay = SurfaceOverlay {
            surface_entity: entity,
        };

        assert_eq!(overlay.surface_entity, entity);
    }
}
