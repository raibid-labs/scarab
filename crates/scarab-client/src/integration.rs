// Integration module - wires all components together
// This demonstrates the complete VTE → SharedState → Rendering pipeline

use crate::rendering::config::FontConfig;
use crate::rendering::text::{generate_terminal_mesh, TerminalMesh, TextRenderer};
use crate::safe_state::SafeSharedState;
use bevy::sprite::MeshMaterial2d;
use bevy::render::mesh::Mesh2d;
use bevy::prelude::*;
use scarab_protocol::{terminal_state::TerminalStateReader, GRID_HEIGHT, GRID_WIDTH};
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

    /// Get a safe wrapper around the shared state
    ///
    /// Returns a SafeSharedState that provides bounds-checked access
    /// to terminal state via the TerminalStateReader trait.
    pub fn get_safe_state(&self) -> SafeSharedState<'_> {
        SafeSharedState::from_shmem(&*self.shmem.0)
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
    let Ok(_window) = window_query.get_single() else {
        return;
    };

    // Only update if renderer is ready (though positioning is mostly window dependent)
    let Some(_renderer) = renderer else {
        return;
    };

    for mut transform in query.iter_mut() {
        // With ScalingMode::WindowSize and camera at (width/2, -height/2, 1000) looking at (width/2, -height/2, 0),
        // the camera viewport shows world coordinates from (0, 0) to (width, -height).
        // Our mesh is generated with local coordinates (0, 0) to (width, -height).
        // So the entity should stay at (0, 0, 0) - no translation needed!

        let x = 0.0;
        let y = 0.0;

        // Only update if changed to avoid unnecessary dirty flags
        if transform.translation.x != x || transform.translation.y != y {
            info!("Grid position update: ({:.2}, {:.2}, {:.2}) -> ({:.2}, {:.2}, {:.2})",
                  transform.translation.x, transform.translation.y, transform.translation.z,
                  x, y, transform.translation.z);
            transform.translation.x = x;
            transform.translation.y = y;
            // Z stays at 0.0
        } else {
            info!("Grid already at correct position: ({:.2}, {:.2}, {:.2})", x, y, transform.translation.z);
        }
    }
}

/// Setup the terminal rendering pipeline
fn setup_terminal_rendering(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut images: ResMut<Assets<Image>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    ipc: Option<Res<crate::ipc::IpcChannel>>,
    window_query: Query<&Window, With<bevy::window::PrimaryWindow>>,
) {
    // Create text renderer
    let font_config = FontConfig::default();

    // Extract values needed for metrics before moving font_config
    let _font_size = font_config.size;
    let _line_height = font_config.line_height;
    let cell_width = font_config.size * 0.6;
    let cell_height = font_config.size * 1.2;

    let mut renderer = TextRenderer::new(font_config, &mut images);
    renderer.update_metrics();

    let atlas_texture = renderer.atlas.texture.clone();

    // Create initial empty mesh (will be populated with terminal content)
    let initial_mesh = Mesh::new(
        bevy::render::mesh::PrimitiveTopology::TriangleList,
        bevy::render::render_asset::RenderAssetUsages::MAIN_WORLD
            | bevy::render::render_asset::RenderAssetUsages::RENDER_WORLD,
    );

    let mesh_handle = meshes.add(initial_mesh);

    // Create material for 2D rendering with ColorMaterial
    let material = materials.add(ColorMaterial {
        color: Color::WHITE,
        texture: Some(atlas_texture.clone()),
        ..default()
    });

    info!("Created ColorMaterial with atlas texture: {:?}", atlas_texture);

    // Calculate terminal dimensions from window size
    let (cols, rows) = if let Ok(window) = window_query.get_single() {
        let width = window.width();
        let height = window.height();
        let cols = ((width / cell_width).floor() as u16)
            .min(scarab_protocol::GRID_WIDTH as u16)
            .max(80);
        let rows = ((height / cell_height).floor() as u16)
            .min(scarab_protocol::GRID_HEIGHT as u16)
            .max(24);

        info!("Window: {}x{} pixels, Terminal: {}x{} cells, Cell: {:.2}x{:.2} pixels",
              width, height, cols, rows, cell_width, cell_height);
        info!("Grid will span: {:.2}x{:.2} pixels",
              cols as f32 * cell_width, rows as f32 * cell_height);

        (cols, rows)
    } else {
        (80, 24) // Fallback
    };

    // Spawn terminal grid entity (Bevy 0.15 API)
    // Position grid at origin - it will naturally render from (0,0) to (width, -height)
    info!("Spawning terminal grid entity at (0, 0, 0)");
    info!("Grid extends from (0, 0) to ({:.2}, {:.2})",
          cols as f32 * cell_width, -(rows as f32 * cell_height));

    // Spawn 2D mesh entity (Bevy 0.15 2D API)
    commands.spawn((
        TerminalGridEntity,
        TerminalMesh::new(mesh_handle.clone()),
        Mesh2d(mesh_handle),
        MeshMaterial2d(material),
        Transform::default(),
    ));

    info!("Spawned 2D terminal grid entity");

    // Insert renderer as resource
    commands.insert_resource(renderer);

    // Create and insert terminal metrics resource for mouse/input systems
    let metrics = scarab_protocol::TerminalMetrics {
        cell_width,
        cell_height,
        columns: cols,
        rows,
    };
    commands.insert_resource(metrics);
    info!("Terminal metrics initialized: {:?}", metrics);

    // Send initial window resize to daemon so PTY knows the terminal size
    if let Some(ipc) = ipc {
        info!("Sending initial resize to daemon: {}x{}", cols, rows);
        ipc.send(scarab_protocol::ControlMessage::Resize { cols, rows });
    }

    info!("Terminal rendering pipeline initialized");
}

/// Sync terminal state from shared memory
fn sync_terminal_state_system(mut state_reader: ResMut<SharedMemoryReader>) {
    // Use safe wrapper to access shared state
    let safe_state = state_reader.get_safe_state();
    let current_seq = safe_state.sequence();

    if current_seq != state_reader.last_sequence {
        // State has been updated by daemon
        let (cursor_x, cursor_y) = safe_state.cursor_pos();
        info!(
            "Terminal state updated: seq {} -> {}, cursor ({}, {})",
            state_reader.last_sequence,
            current_seq,
            cursor_x,
            cursor_y
        );

        state_reader.last_sequence = current_seq;
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
    // Use safe wrapper to access shared state
    let safe_state = state_reader.get_safe_state();

    for mut terminal_mesh in query.iter_mut() {
        // Check if state changed OR if this is the first render (last_sequence == 0 but we haven't rendered yet)
        let current_seq = safe_state.sequence();
        let is_first_render = terminal_mesh.last_sequence == 0 && terminal_mesh.dirty_region.is_full_redraw();

        if current_seq != terminal_mesh.last_sequence {
            info!("Mesh update triggered: seq {} -> {}", terminal_mesh.last_sequence, current_seq);
            terminal_mesh.dirty_region.mark_full_redraw();
            terminal_mesh.last_sequence = current_seq;
        }

        // Skip if nothing to update UNLESS this is the first render
        if !is_first_render && terminal_mesh.dirty_region.is_empty() {
            continue;
        }

        info!("Generating mesh... (first_render: {})", is_first_render);
        // Generate new mesh from terminal state using safe wrapper
        let new_mesh = generate_terminal_mesh(
            &safe_state,
            &mut renderer,
            &terminal_mesh.dirty_region,
            &mut images,
        );

        info!("Mesh generated with {} vertices",
            new_mesh.attribute(Mesh::ATTRIBUTE_POSITION).map_or(0, |a| a.len()));

        // Update mesh asset using insert (proper way for Bevy 0.15+)
        meshes.insert(&terminal_mesh.mesh_handle, new_mesh);
        info!("Mesh updated successfully via insert");

        // Clear dirty region
        terminal_mesh.dirty_region.clear();
    }
}

/// Helper to extract text from terminal grid for UI features
///
/// Now uses TerminalStateReader trait for safe access
pub fn extract_grid_text(state: &impl TerminalStateReader) -> String {
    let mut text = String::with_capacity(GRID_WIDTH * GRID_HEIGHT);
    let (width, height) = state.dimensions();

    for row in 0..height {
        for col in 0..width {
            if let Some(cell) = state.cell(row, col) {
                if cell.char_codepoint == 0 || cell.char_codepoint == 32 {
                    text.push(' ');
                } else if let Some(ch) = char::from_u32(cell.char_codepoint) {
                    text.push(ch);
                } else {
                    text.push('?');
                }
            } else {
                text.push(' ');
            }
        }
        if row < height - 1 {
            text.push('\n');
        }
    }

    text
}

/// Helper to get cell at specific position
///
/// Now uses TerminalStateReader trait for safe access
pub fn get_cell_at(state: &impl TerminalStateReader, x: usize, y: usize) -> Option<&scarab_protocol::Cell> {
    state.cell(y, x) // Note: cell() takes (row, col)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::safe_state::MockTerminalState;

    #[test]
    fn test_extract_grid_text() {
        use scarab_protocol::Cell;

        let mut mock = MockTerminalState::new(GRID_WIDTH, GRID_HEIGHT);

        // Set some test characters
        let mut cell_h = Cell::default();
        cell_h.char_codepoint = 'H' as u32;
        mock.set_cell(0, 0, cell_h);

        let mut cell_i = Cell::default();
        cell_i.char_codepoint = 'i' as u32;
        mock.set_cell(0, 1, cell_i);

        let text = extract_grid_text(&mock);
        assert!(text.starts_with("Hi"));
    }

    #[test]
    fn test_get_cell_at() {
        let mock = MockTerminalState::new(GRID_WIDTH, GRID_HEIGHT);

        assert!(get_cell_at(&mock, 0, 0).is_some());
        assert!(get_cell_at(&mock, GRID_WIDTH, 0).is_none());
        assert!(get_cell_at(&mock, 0, GRID_HEIGHT).is_none());
    }
}
