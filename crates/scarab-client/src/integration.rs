// Integration module - wires all components together
// This demonstrates the complete VTE → SharedState → Rendering pipeline

use crate::rendering::config::FontConfig;
use crate::rendering::text::{generate_terminal_mesh, TerminalMesh, TextRenderer};
use bevy::pbr::MeshMaterial3d;
use bevy::prelude::*;
use scarab_protocol::{SharedState, GRID_HEIGHT, GRID_WIDTH};
use shared_memory::Shmem;
use std::sync::Arc;

// Wrapper to make shared memory Send + Sync
pub struct SharedMemWrapper(pub Arc<Shmem>);

unsafe impl Send for SharedMemWrapper {}
unsafe impl Sync for SharedMemWrapper {}

/// Resource to hold shared memory state
#[derive(Resource)]
pub struct SharedMemoryReader {
    pub shmem: SharedMemWrapper,
    pub last_sequence: u64,
}

impl SharedMemoryReader {
    pub fn new(shmem: Arc<Shmem>) -> Self {
        Self {
            shmem: SharedMemWrapper(shmem),
            last_sequence: 0,
        }
    }
}

/// Integration plugin that wires all systems together
pub struct IntegrationPlugin;

impl Plugin for IntegrationPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_terminal_rendering)
            .add_systems(
                Update,
                (
                    sync_terminal_state_system,
                    update_terminal_rendering_system,
                    update_grid_position_system,
                )
                    .chain(),
            );
    }
}

/// Marker component for the terminal grid entity
#[derive(Component)]
pub struct TerminalGridEntity;

/// Align the terminal grid to the top-left of the window
fn update_grid_position_system(
    mut query: Query<&mut Transform, With<TerminalGridEntity>>,
    window_query: Query<&Window, With<bevy::window::PrimaryWindow>>,
    renderer: Option<Res<TextRenderer>>, // Use Option to avoid panic if not ready
) {
    let Ok(window) = window_query.get_single() else {
        return;
    };
    
    // Only update if renderer is ready (though positioning is mostly window dependent)
    let Some(_renderer) = renderer else {
        return;
    };

    for mut transform in query.iter_mut() {
        // Camera (Orthographic, WindowSize) has (0,0) at center.
        // Top-Left is (-width/2, +height/2).
        // Our mesh is generated with (0,0) as the top-left of the first character.
        // So we simply translate the entity to the top-left of the window.
        
        let x = -window.width() / 2.0;
        let y = window.height() / 2.0;
        
        // Only update if changed to avoid unnecessary dirty flags
        if transform.translation.x != x || transform.translation.y != y {
            transform.translation.x = x;
            transform.translation.y = y;
            // Z stays at 0.0
        }
    }
}

/// Setup the terminal rendering pipeline
fn setup_terminal_rendering(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut images: ResMut<Assets<Image>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Create text renderer
    let font_config = FontConfig::default();
    let mut renderer = TextRenderer::new(font_config, &mut images);
    renderer.update_metrics();

    let atlas_texture = renderer.atlas.texture.clone();

    // Create mesh for terminal grid
    // Use MAIN_WORLD | RENDER_WORLD so we can access it from both
    let mesh_handle = meshes.add(Mesh::new(
        bevy::render::mesh::PrimitiveTopology::TriangleList,
        bevy::render::render_asset::RenderAssetUsages::MAIN_WORLD
            | bevy::render::render_asset::RenderAssetUsages::RENDER_WORLD,
    ));

    // Create material that uses the glyph atlas
    let material = materials.add(StandardMaterial {
        base_color_texture: Some(atlas_texture),
        unlit: true,
        alpha_mode: AlphaMode::Blend,
        ..default()
    });

    // Spawn terminal grid entity (Bevy 0.15 API)
    commands.spawn((
        TerminalGridEntity,
        TerminalMesh::new(mesh_handle.clone()),
        Mesh3d(mesh_handle),
        MeshMaterial3d(material),
        Transform::from_xyz(0.0, 0.0, 0.0),
    ));

    // Insert renderer as resource
    commands.insert_resource(renderer);

    info!("Terminal rendering pipeline initialized");
}

/// Sync terminal state from shared memory
fn sync_terminal_state_system(mut state_reader: ResMut<SharedMemoryReader>) {
    // Read current sequence number from shared memory
    let shared_ptr = state_reader.shmem.0.as_ptr() as *const SharedState;

    unsafe {
        let state = &*shared_ptr;
        let current_seq = state.sequence_number;

        if current_seq != state_reader.last_sequence {
            // State has been updated by daemon
            info!(
                "Terminal state updated: seq {} -> {}, cursor ({}, {})",
                state_reader.last_sequence,
                current_seq,
                state.cursor_x,
                state.cursor_y
            );

            state_reader.last_sequence = current_seq;
        }
    }
}

/// Update terminal rendering from shared state
fn update_terminal_rendering_system(
    mut renderer: ResMut<TextRenderer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut images: ResMut<Assets<Image>>,
    mut query: Query<&mut TerminalMesh, With<TerminalGridEntity>>,
    state_reader: Res<SharedMemoryReader>,
) {
    let state = unsafe { &*(state_reader.shmem.0.as_ptr() as *const SharedState) };

    for mut terminal_mesh in query.iter_mut() {
        // Check if state changed
        let current_seq = state.sequence_number;
        if current_seq != terminal_mesh.last_sequence {
            info!("Mesh update triggered: seq {} -> {}", terminal_mesh.last_sequence, current_seq);
            terminal_mesh.dirty_region.mark_full_redraw();
            terminal_mesh.last_sequence = current_seq;
        }

        // Skip if nothing to update
        if terminal_mesh.dirty_region.is_empty() {
            continue;
        }

        info!("Generating mesh...");
        // Generate new mesh from terminal state
        let new_mesh = generate_terminal_mesh(
            state,
            &mut renderer,
            &terminal_mesh.dirty_region,
            &mut images,
        );

        info!("Mesh generated with {} vertices",
            new_mesh.attribute(Mesh::ATTRIBUTE_POSITION).map_or(0, |a| a.len()));

        // Update mesh asset
        if let Some(mesh) = meshes.get_mut(&terminal_mesh.mesh_handle) {
            *mesh = new_mesh;
            info!("Mesh updated successfully");
        } else {
            warn!("Failed to get mesh handle!");
        }

        // Clear dirty region
        terminal_mesh.dirty_region.clear();
    }
}

/// Helper to extract text from terminal grid for UI features
pub fn extract_grid_text(state: &SharedState) -> String {
    let mut text = String::with_capacity(GRID_WIDTH * GRID_HEIGHT);

    for row in 0..GRID_HEIGHT {
        for col in 0..GRID_WIDTH {
            let idx = row * GRID_WIDTH + col;
            let cell = &state.cells[idx];

            if cell.char_codepoint == 0 || cell.char_codepoint == 32 {
                text.push(' ');
            } else if let Some(ch) = char::from_u32(cell.char_codepoint) {
                text.push(ch);
            } else {
                text.push('?');
            }
        }
        if row < GRID_HEIGHT - 1 {
            text.push('\n');
        }
    }

    text
}

/// Helper to get cell at specific position
pub fn get_cell_at(state: &SharedState, x: usize, y: usize) -> Option<&scarab_protocol::Cell> {
    if x >= GRID_WIDTH || y >= GRID_HEIGHT {
        return None;
    }

    let idx = y * GRID_WIDTH + x;
    state.cells.get(idx)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_grid_text() {
        use scarab_protocol::Cell;

        let mut state = unsafe { std::mem::zeroed::<SharedState>() };

        // Set some test characters
        state.cells[0] = Cell {
            char_codepoint: 'H' as u32,
            fg: 0xFFFFFFFF,
            bg: 0xFF000000,
            flags: 0,
            _padding: [0; 3],
        };

        state.cells[1] = Cell {
            char_codepoint: 'i' as u32,
            fg: 0xFFFFFFFF,
            bg: 0xFF000000,
            flags: 0,
            _padding: [0; 3],
        };

        let text = extract_grid_text(&state);
        assert!(text.starts_with("Hi"));
    }

    #[test]
    fn test_get_cell_at() {
        let state = unsafe { std::mem::zeroed::<SharedState>() };

        assert!(get_cell_at(&state, 0, 0).is_some());
        assert!(get_cell_at(&state, GRID_WIDTH, 0).is_none());
        assert!(get_cell_at(&state, 0, GRID_HEIGHT).is_none());
    }
}
