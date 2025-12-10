// Integration module - wires all components together
// This demonstrates the complete VTE → SharedState → Rendering pipeline

use crate::events::WindowResizedEvent;
use crate::rendering::config::FontConfig;
use crate::rendering::text::{generate_terminal_mesh, TerminalMesh, TextRenderer};
use crate::safe_state::SafeSharedState;
use crate::ui::STATUS_BAR_HEIGHT;
use bevy::prelude::*;
use bevy::render::mesh::Mesh2d;
use bevy::sprite::MeshMaterial2d;
use scarab_protocol::{
    terminal_state::TerminalStateReader, TerminalMetrics, GRID_HEIGHT, GRID_WIDTH,
};
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
                    handle_terminal_resize_system,
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

/// Handle terminal resize events - update metrics and notify daemon
fn handle_terminal_resize_system(
    mut resize_events: EventReader<WindowResizedEvent>,
    mut metrics: ResMut<TerminalMetrics>,
    mut terminal_mesh_query: Query<&mut TerminalMesh, With<TerminalGridEntity>>,
    ipc: Option<Res<crate::ipc::IpcChannel>>,
) {
    for event in resize_events.read() {
        let cols = event.cols.min(scarab_protocol::GRID_WIDTH as u16).max(10);
        let rows = event.rows.min(scarab_protocol::GRID_HEIGHT as u16).max(5);

        // Only update if dimensions actually changed
        if cols != metrics.columns || rows != metrics.rows {
            info!(
                "Terminal resize: {}x{} -> {}x{} cells",
                metrics.columns, metrics.rows, cols, rows
            );

            // Update metrics resource
            metrics.columns = cols;
            metrics.rows = rows;

            // Send resize to daemon so PTY knows the new terminal size
            if let Some(ref ipc) = ipc {
                ipc.send(scarab_protocol::ControlMessage::Resize { cols, rows });
            }

            // Mark mesh for full redraw to regenerate with new grid dimensions
            for mut terminal_mesh in terminal_mesh_query.iter_mut() {
                terminal_mesh.dirty_region.mark_full_redraw();
            }
        }
    }
}

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
        // Centered camera means world origin is at screen center.
        // Shift grid so its top-left sits at the visible top-left.
        // The grid renders DOWNWARD from its origin, so we need to:
        // 1. Move left edge to -width/2 (left side of window)
        // 2. Move top edge to +height/2 (top of window)
        // The grid rows are already calculated to fit above the status bar,
        // so we don't need to offset Y for the status bar here.
        let x = -window.width() * 0.5;
        let y = window.height() * 0.5;

        // Only update if changed to avoid unnecessary dirty flags
        if transform.translation.x != x || transform.translation.y != y {
            transform.translation.x = x;
            transform.translation.y = y;
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

    info!(
        "Created ColorMaterial with atlas texture: {:?}",
        atlas_texture
    );

    // Calculate terminal dimensions from window size
    let (cols, rows) = if let Ok(window) = window_query.get_single() {
        let width = window.width();
        let height = window.height();
        // Account for status bar at the bottom - reduce available height
        let available_height = height - STATUS_BAR_HEIGHT;
        let cols = ((width / cell_width).floor() as u16)
            .min(scarab_protocol::GRID_WIDTH as u16)
            .max(80);
        let rows = ((available_height / cell_height).floor() as u16)
            .min(scarab_protocol::GRID_HEIGHT as u16)
            .max(24);

        info!(
            "Window: {}x{} pixels (available: {}x{} after status bar), Terminal: {}x{} cells, Cell: {:.2}x{:.2} pixels",
            width, height, width, available_height, cols, rows, cell_width, cell_height
        );
        info!(
            "Grid will span: {:.2}x{:.2} pixels",
            cols as f32 * cell_width,
            rows as f32 * cell_height
        );

        (cols, rows)
    } else {
        (80, 24) // Fallback
    };

    // Spawn terminal grid entity (Bevy 0.15 API)
    // Position grid at origin - it will naturally render from (0,0) to (width, -height)
    info!("Spawning terminal grid entity at (0, 0, 0)");
    info!(
        "Grid extends from (0, 0) to ({:.2}, {:.2})",
        cols as f32 * cell_width,
        -(rows as f32 * cell_height)
    );

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

    // Log initial sequence on first check
    static FIRST_CHECK: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(true);
    if FIRST_CHECK.swap(false, std::sync::atomic::Ordering::SeqCst) {
        // Also check what cells contain
        let cells = safe_state.cells();
        let non_empty = cells
            .iter()
            .filter(|c| c.char_codepoint != 0 && c.char_codepoint != 32)
            .count();
        info!(
            "Initial shared state: seq={}, non_empty_cells={}, dirty={}",
            current_seq,
            non_empty,
            safe_state.is_dirty()
        );
    }

    if current_seq != state_reader.last_sequence {
        // State has been updated by daemon
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
        let is_first_render =
            terminal_mesh.last_sequence == 0 && terminal_mesh.dirty_region.is_full_redraw();

        if current_seq != terminal_mesh.last_sequence {
            terminal_mesh.dirty_region.mark_full_redraw();
            terminal_mesh.last_sequence = current_seq;
        }

        // Skip if nothing to update UNLESS this is the first render
        if !is_first_render && terminal_mesh.dirty_region.is_empty() {
            continue;
        }

        // Generate new mesh from terminal state using safe wrapper
        let new_mesh = generate_terminal_mesh(
            &safe_state,
            &mut renderer,
            &terminal_mesh.dirty_region,
            &mut images,
        );

        // Update mesh asset using insert (proper way for Bevy 0.15+)
        meshes.insert(&terminal_mesh.mesh_handle, new_mesh);

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
pub fn get_cell_at(
    state: &impl TerminalStateReader,
    x: usize,
    y: usize,
) -> Option<&scarab_protocol::Cell> {
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
