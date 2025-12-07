// Chunk-based rendering system for terminal grid
// Divides the grid into smaller regions for efficient partial updates

use bevy::prelude::*;
use bevy::render::mesh::{Indices, Mesh, Mesh2d, PrimitiveTopology};
use bevy::render::render_asset::RenderAssetUsages;
use bevy::sprite::{ColorMaterial, MeshMaterial2d};
use scarab_protocol::{
    terminal_state::TerminalStateReader, Cell, BUFFER_SIZE, GRID_HEIGHT, GRID_WIDTH,
};

use crate::integration::SharedMemoryReader;

/// Chunk dimensions (cells)
pub const CHUNK_WIDTH: u16 = 64;
pub const CHUNK_HEIGHT: u16 = 32;

/// Number of chunks needed to cover the grid
pub const CHUNKS_X: usize = (GRID_WIDTH as usize + CHUNK_WIDTH as usize - 1) / CHUNK_WIDTH as usize;
pub const CHUNKS_Y: usize =
    (GRID_HEIGHT as usize + CHUNK_HEIGHT as usize - 1) / CHUNK_HEIGHT as usize;

/// Marker component for terminal chunk entities
///
/// Each chunk represents a rectangular region of the terminal grid
/// and can be independently marked dirty for optimized rendering.
#[derive(Component, Debug)]
pub struct TerminalChunk {
    /// Chunk coordinates (not cell coordinates)
    pub chunk_x: u16,
    pub chunk_y: u16,
    /// Whether this chunk needs mesh rebuild
    pub dirty: bool,
    /// Last sequence number when this chunk was updated
    pub last_sequence: u64,
}

impl TerminalChunk {
    /// Create a new chunk at the given chunk coordinates
    pub fn new(chunk_x: u16, chunk_y: u16) -> Self {
        Self {
            chunk_x,
            chunk_y,
            dirty: true, // Start dirty for initial render
            last_sequence: 0,
        }
    }

    /// Get the cell coordinate range this chunk covers
    ///
    /// Returns (start_x, start_y, end_x, end_y) where end coordinates are exclusive
    pub fn cell_range(&self) -> (u16, u16, u16, u16) {
        let start_x = self.chunk_x * CHUNK_WIDTH;
        let start_y = self.chunk_y * CHUNK_HEIGHT;
        let end_x = ((self.chunk_x + 1) * CHUNK_WIDTH).min(GRID_WIDTH as u16);
        let end_y = ((self.chunk_y + 1) * CHUNK_HEIGHT).min(GRID_HEIGHT as u16);
        (start_x, start_y, end_x, end_y)
    }

    /// Check if a cell coordinate falls within this chunk
    pub fn contains_cell(&self, x: u16, y: u16) -> bool {
        let (sx, sy, ex, ey) = self.cell_range();
        x >= sx && x < ex && y >= sy && y < ey
    }

    /// Get the width of this chunk in cells
    pub fn width(&self) -> u16 {
        let (start_x, _, end_x, _) = self.cell_range();
        end_x - start_x
    }

    /// Get the height of this chunk in cells
    pub fn height(&self) -> u16 {
        let (_, start_y, _, end_y) = self.cell_range();
        end_y - start_y
    }
}

/// Component holding chunk's mesh and material handles
#[derive(Component, Debug)]
pub struct ChunkMesh {
    pub mesh_handle: Handle<Mesh>,
    pub material_handle: Handle<ColorMaterial>,
}

/// Resource tracking all chunk entities
///
/// Provides a 2D grid of chunk entities for efficient lookup
#[derive(Resource, Default, Debug)]
pub struct ChunkGrid {
    /// Entity IDs for each chunk, indexed by [chunk_y][chunk_x]
    pub chunks: [[Option<Entity>; CHUNKS_X]; CHUNKS_Y],
}

impl ChunkGrid {
    /// Get chunk coordinates for a cell position
    ///
    /// Returns (chunk_x, chunk_y)
    pub fn cell_to_chunk(x: u16, y: u16) -> (usize, usize) {
        ((x / CHUNK_WIDTH) as usize, (y / CHUNK_HEIGHT) as usize)
    }

    /// Mark a cell's chunk as dirty
    ///
    /// This is used when a specific cell changes and we need to re-render its chunk
    pub fn mark_dirty(&self, x: u16, y: u16, chunks: &mut Query<&mut TerminalChunk>) {
        let (cx, cy) = Self::cell_to_chunk(x, y);
        if cy < CHUNKS_Y && cx < CHUNKS_X {
            if let Some(entity) = self.chunks[cy][cx] {
                if let Ok(mut chunk) = chunks.get_mut(entity) {
                    chunk.dirty = true;
                }
            }
        }
    }

    /// Mark all chunks as dirty
    ///
    /// Used when the entire grid needs to be re-rendered
    pub fn mark_all_dirty(&self, chunks: &mut Query<&mut TerminalChunk>) {
        for row in &self.chunks {
            for entity_opt in row {
                if let Some(entity) = entity_opt {
                    if let Ok(mut chunk) = chunks.get_mut(*entity) {
                        chunk.dirty = true;
                    }
                }
            }
        }
    }

    /// Get entity at specific chunk coordinates
    pub fn get_chunk(&self, chunk_x: usize, chunk_y: usize) -> Option<Entity> {
        if chunk_y < CHUNKS_Y && chunk_x < CHUNKS_X {
            self.chunks[chunk_y][chunk_x]
        } else {
            None
        }
    }

    /// Get entity for the chunk containing the given cell
    pub fn get_chunk_for_cell(&self, x: u16, y: u16) -> Option<Entity> {
        let (cx, cy) = Self::cell_to_chunk(x, y);
        self.get_chunk(cx, cy)
    }
}

/// Resource storing previous frame's grid for dirty detection
///
/// This enables per-cell change detection by comparing current state
/// with the previous frame's state. Only chunks containing changed
/// cells will be marked dirty.
#[derive(Resource)]
pub struct PreviousGridState {
    /// Previous cell data (flattened grid)
    pub cells: Vec<Cell>,
    /// Last sequence number processed
    pub last_sequence: u64,
}

impl Default for PreviousGridState {
    fn default() -> Self {
        Self {
            cells: vec![Cell::default(); BUFFER_SIZE],
            last_sequence: 0,
        }
    }
}

impl PreviousGridState {
    /// Compare with current state and return dirty cell positions
    ///
    /// This performs a per-cell comparison to identify exactly which
    /// cells have changed since the last frame. Returns a Vec of (x, y)
    /// coordinates for changed cells.
    ///
    /// # Performance
    /// - Early exit if no changes (sequence number check)
    /// - Inline cell comparison for hot path optimization
    /// - Allocation only when changes are detected
    pub fn find_dirty_cells(&self, current: &[Cell]) -> Vec<(u16, u16)> {
        let mut dirty = Vec::new();

        // Compare each cell for changes
        for (idx, (prev, curr)) in self.cells.iter().zip(current.iter()).enumerate() {
            if !cells_equal(prev, curr) {
                let x = (idx % GRID_WIDTH) as u16;
                let y = (idx / GRID_WIDTH) as u16;
                dirty.push((x, y));
            }
        }

        dirty
    }

    /// Update stored state from current
    ///
    /// Call this after processing changes to update the baseline
    /// for the next frame's comparison.
    pub fn update(&mut self, current: &[Cell], sequence: u64) {
        self.cells.copy_from_slice(current);
        self.last_sequence = sequence;
    }
}

/// Fast cell equality check (compare relevant fields)
///
/// Compares only the fields that affect visual rendering.
/// Ignoring padding fields for performance.
#[inline]
fn cells_equal(a: &Cell, b: &Cell) -> bool {
    a.char_codepoint == b.char_codepoint && a.fg == b.fg && a.bg == b.bg && a.flags == b.flags
}

/// Generate mesh for a chunk's cell range
///
/// This creates a mesh containing only the background quads for cells in the chunk.
/// Text rendering is handled separately by the cosmic-text system for proper glyph handling.
fn generate_chunk_mesh(
    reader: &SharedMemoryReader,
    start_x: u16,
    start_y: u16,
    end_x: u16,
    end_y: u16,
    metrics: &scarab_protocol::TerminalMetrics,
) -> Mesh {
    let mut positions: Vec<[f32; 3]> = Vec::new();
    let mut colors: Vec<[f32; 4]> = Vec::new();
    let mut indices: Vec<u32> = Vec::new();

    let safe_state = reader.get_safe_state();
    let cells = safe_state.cells();

    for y in start_y..end_y {
        for x in start_x..end_x {
            let idx = y as usize * GRID_WIDTH + x as usize;
            if idx >= cells.len() {
                continue;
            }
            let cell = &cells[idx];

            // Skip cells with default black background (optimization)
            // Only render backgrounds that are non-default
            if cell.bg == 0x000000FF || cell.bg == 0 {
                continue;
            }

            // Local position within chunk (relative to chunk origin)
            let local_x = (x - start_x) as f32 * metrics.cell_width;
            let local_y = (y - start_y) as f32 * metrics.cell_height;

            // Add quad vertices for background
            let base_idx = positions.len() as u32;
            let bg_color = u32_to_color(cell.bg);

            // Four corners of the cell
            // Y-down coordinate system (matching terminal rendering)
            positions.push([local_x, -local_y, 0.0]);
            positions.push([local_x + metrics.cell_width, -local_y, 0.0]);
            positions.push([
                local_x + metrics.cell_width,
                -local_y - metrics.cell_height,
                0.0,
            ]);
            positions.push([local_x, -local_y - metrics.cell_height, 0.0]);

            colors.push(bg_color);
            colors.push(bg_color);
            colors.push(bg_color);
            colors.push(bg_color);

            // Two triangles (counter-clockwise winding)
            indices.extend_from_slice(&[
                base_idx,
                base_idx + 1,
                base_idx + 2,
                base_idx,
                base_idx + 2,
                base_idx + 3,
            ]);
        }
    }

    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::RENDER_WORLD,
    );

    // Only add attributes if we have vertices
    if !positions.is_empty() {
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
        mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);
        mesh.insert_indices(Indices::U32(indices));
    }

    mesh
}

/// Convert u32 RGBA to Bevy color array
fn u32_to_color(rgba: u32) -> [f32; 4] {
    let r = ((rgba >> 24) & 0xFF) as f32 / 255.0;
    let g = ((rgba >> 16) & 0xFF) as f32 / 255.0;
    let b = ((rgba >> 8) & 0xFF) as f32 / 255.0;
    let a = (rgba & 0xFF) as f32 / 255.0;
    [r, g, b, a]
}

/// System to generate meshes for dirty chunks
///
/// This system rebuilds meshes only for chunks that have been marked dirty,
/// significantly reducing rendering overhead for incremental terminal updates.
pub fn generate_chunk_meshes(
    mut commands: Commands,
    reader: Res<SharedMemoryReader>,
    metrics: Res<scarab_protocol::TerminalMetrics>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut chunks: Query<(Entity, &mut TerminalChunk, Option<&ChunkMesh>)>,
) {
    let mut dirty_count = 0;
    let mut updated_count = 0;

    for (entity, mut chunk, existing_mesh) in chunks.iter_mut() {
        if !chunk.dirty {
            continue;
        }

        dirty_count += 1;

        // Get cell range for this chunk
        let (start_x, start_y, end_x, end_y) = chunk.cell_range();

        // Generate mesh for this chunk's cells
        let mesh = generate_chunk_mesh(&reader, start_x, start_y, end_x, end_y, &metrics);

        // Calculate chunk world position
        let (world_x, world_y) = metrics.grid_to_screen(start_x, start_y);

        if let Some(chunk_mesh) = existing_mesh {
            // Update existing mesh
            meshes.insert(&chunk_mesh.mesh_handle, mesh);
            updated_count += 1;
        } else {
            // Create new mesh and material
            let mesh_handle = meshes.add(mesh);
            let material_handle = materials.add(ColorMaterial::from(Color::WHITE));

            commands.entity(entity).insert((
                ChunkMesh {
                    mesh_handle: mesh_handle.clone(),
                    material_handle: material_handle.clone(),
                },
                Mesh2d(mesh_handle),
                MeshMaterial2d(material_handle),
                Transform::from_xyz(world_x, -world_y, 0.0),
                Visibility::Visible,
            ));
            updated_count += 1;
        }

        chunk.dirty = false;
    }

    if dirty_count > 0 {
        debug!(
            "Chunk mesh generation: {}/{} chunks updated",
            updated_count, dirty_count
        );
    }
}

/// Plugin for chunk-based rendering
///
/// This plugin manages the lifecycle of chunk entities and tracks
/// which chunks need to be re-rendered based on SharedState changes.
pub struct ChunkPlugin;

impl Plugin for ChunkPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ChunkGrid>()
            .init_resource::<PreviousGridState>()
            .add_systems(Startup, spawn_chunks)
            .add_systems(Update, (mark_dirty_chunks, generate_chunk_meshes).chain());
    }
}

/// Spawn chunk entities on startup
///
/// Creates a grid of TerminalChunk entities that will later
/// have mesh components attached for rendering.
fn spawn_chunks(mut commands: Commands, mut grid: ResMut<ChunkGrid>) {
    info!(
        "Spawning chunk grid: {}x{} chunks ({} cells per chunk)",
        CHUNKS_X,
        CHUNKS_Y,
        CHUNK_WIDTH as usize * CHUNK_HEIGHT as usize
    );

    for cy in 0..CHUNKS_Y {
        for cx in 0..CHUNKS_X {
            let chunk = TerminalChunk::new(cx as u16, cy as u16);
            let (start_x, start_y, end_x, end_y) = chunk.cell_range();

            debug!(
                "Chunk ({}, {}) covers cells [{}, {}) to [{}, {})",
                cx, cy, start_x, start_y, end_x, end_y
            );

            let entity = commands.spawn(chunk).id();
            grid.chunks[cy][cx] = Some(entity);
        }
    }

    info!(
        "Successfully spawned {} chunk entities",
        CHUNKS_X * CHUNKS_Y
    );
}

/// System to mark chunks dirty based on SharedState changes
///
/// This system implements smart dirty region tracking by:
/// 1. Comparing current state with previous frame
/// 2. Identifying exactly which cells changed
/// 3. Marking only the chunks containing changed cells
///
/// This avoids unnecessary mesh rebuilding for chunks that haven't changed.
fn mark_dirty_chunks(
    reader: Res<SharedMemoryReader>,
    mut prev_state: ResMut<PreviousGridState>,
    grid: Res<ChunkGrid>,
    mut chunks: Query<&mut TerminalChunk>,
) {
    // Get current state using safe wrapper
    let safe_state = reader.get_safe_state();
    let current_seq = safe_state.sequence();

    // Early exit if no change
    if current_seq == prev_state.last_sequence {
        return;
    }

    // Get current cells from shared memory
    let current_cells = safe_state.cells();

    // Find dirty cells by comparing with previous state
    let dirty_cells = prev_state.find_dirty_cells(current_cells);

    // Track unique dirty chunks using a HashSet for deduplication
    let mut dirty_chunks = std::collections::HashSet::new();
    for (x, y) in &dirty_cells {
        let (cx, cy) = ChunkGrid::cell_to_chunk(*x, *y);
        dirty_chunks.insert((cx, cy));
    }

    // Update chunk dirty flags
    for cy in 0..CHUNKS_Y {
        for cx in 0..CHUNKS_X {
            if let Some(entity) = grid.chunks[cy][cx] {
                if let Ok(mut chunk) = chunks.get_mut(entity) {
                    if dirty_chunks.contains(&(cx, cy)) {
                        chunk.dirty = true;
                        chunk.last_sequence = current_seq;
                    }
                }
            }
        }
    }

    // Update previous state for next frame
    prev_state.update(current_cells, current_seq);

    // Debug telemetry (only log when changes detected)
    if !dirty_cells.is_empty() {
        debug!(
            "Dirty tracking: {} cells changed, {} chunks marked dirty (seq {})",
            dirty_cells.len(),
            dirty_chunks.len(),
            current_seq
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chunk_dimensions() {
        // With 200x100 grid and 64x32 chunks:
        // X: (200 + 64 - 1) / 64 = 263 / 64 = 4 chunks
        // Y: (100 + 32 - 1) / 32 = 131 / 32 = 4 chunks
        assert_eq!(CHUNKS_X, 4);
        assert_eq!(CHUNKS_Y, 4);
    }

    #[test]
    fn test_chunk_cell_range() {
        // First chunk (0, 0)
        let chunk = TerminalChunk::new(0, 0);
        assert_eq!(chunk.cell_range(), (0, 0, 64, 32));

        // Second chunk in X (1, 0)
        let chunk = TerminalChunk::new(1, 0);
        assert_eq!(chunk.cell_range(), (64, 0, 128, 32));

        // Last chunk (3, 3) - should be clipped to grid size
        let chunk = TerminalChunk::new(3, 3);
        let (start_x, start_y, end_x, end_y) = chunk.cell_range();
        assert_eq!(start_x, 192);
        assert_eq!(start_y, 96);
        assert_eq!(end_x, 200); // Clipped from 256 to GRID_WIDTH
        assert_eq!(end_y, 100); // Clipped from 128 to GRID_HEIGHT
    }

    #[test]
    fn test_chunk_contains_cell() {
        let chunk = TerminalChunk::new(0, 0);

        // Inside chunk
        assert!(chunk.contains_cell(0, 0));
        assert!(chunk.contains_cell(32, 16));
        assert!(chunk.contains_cell(63, 31));

        // Outside chunk
        assert!(!chunk.contains_cell(64, 0)); // Just outside in X
        assert!(!chunk.contains_cell(0, 32)); // Just outside in Y
        assert!(!chunk.contains_cell(100, 50));
    }

    #[test]
    fn test_cell_to_chunk() {
        // Top-left corner
        assert_eq!(ChunkGrid::cell_to_chunk(0, 0), (0, 0));

        // Within first chunk
        assert_eq!(ChunkGrid::cell_to_chunk(63, 31), (0, 0));

        // Second chunk in X
        assert_eq!(ChunkGrid::cell_to_chunk(64, 0), (1, 0));

        // Second chunk in Y
        assert_eq!(ChunkGrid::cell_to_chunk(0, 32), (0, 1));

        // Middle of grid
        assert_eq!(ChunkGrid::cell_to_chunk(100, 50), (1, 1));

        // Last cell in grid
        assert_eq!(
            ChunkGrid::cell_to_chunk(GRID_WIDTH as u16 - 1, GRID_HEIGHT as u16 - 1),
            (3, 3)
        );
    }

    #[test]
    fn test_chunk_width_height() {
        // Regular chunk
        let chunk = TerminalChunk::new(0, 0);
        assert_eq!(chunk.width(), 64);
        assert_eq!(chunk.height(), 32);

        // Edge chunk (should be smaller due to clipping)
        let chunk = TerminalChunk::new(3, 3);
        assert_eq!(chunk.width(), 8); // 200 - 192 = 8 cells
        assert_eq!(chunk.height(), 4); // 100 - 96 = 4 cells
    }

    #[test]
    fn test_chunk_plugin_integration() {
        // Verify ChunkPlugin spawns chunks correctly
        let mut app = App::new();

        // Manually set up resources and run startup system
        // (avoiding Update system which needs SharedMemoryReader)
        app.init_resource::<ChunkGrid>();
        app.add_systems(Startup, spawn_chunks);

        // Update once to trigger startup systems
        app.update();

        // Verify chunks were spawned
        let grid = app.world().resource::<ChunkGrid>();
        for cy in 0..CHUNKS_Y {
            for cx in 0..CHUNKS_X {
                assert!(
                    grid.chunks[cy][cx].is_some(),
                    "Chunk ({}, {}) should be spawned",
                    cx,
                    cy
                );
            }
        }

        // Verify all chunk entities exist and have correct coordinates
        let mut query = app.world_mut().query::<&TerminalChunk>();
        let count = query.iter(app.world()).fold(0, |acc, chunk| {
            assert!(chunk.chunk_x < CHUNKS_X as u16);
            assert!(chunk.chunk_y < CHUNKS_Y as u16);
            assert!(chunk.dirty); // Initial render should be dirty
            assert_eq!(chunk.last_sequence, 0);
            acc + 1
        });
        assert_eq!(count, CHUNKS_X * CHUNKS_Y);
    }

    #[test]
    fn test_cells_equal() {
        let cell1 = Cell::default();
        let mut cell2 = Cell::default();

        // Default cells should be equal
        assert!(cells_equal(&cell1, &cell2));

        // Change codepoint
        cell2.char_codepoint = 'A' as u32;
        assert!(!cells_equal(&cell1, &cell2));
        cell2.char_codepoint = cell1.char_codepoint;

        // Change foreground
        cell2.fg = 0xFF0000FF;
        assert!(!cells_equal(&cell1, &cell2));
        cell2.fg = cell1.fg;

        // Change background
        cell2.bg = 0x00FF00FF;
        assert!(!cells_equal(&cell1, &cell2));
        cell2.bg = cell1.bg;

        // Change flags
        cell2.flags = 0x01;
        assert!(!cells_equal(&cell1, &cell2));
        cell2.flags = cell1.flags;

        // Padding should not affect equality
        cell2._padding = [1, 2, 3];
        assert!(cells_equal(&cell1, &cell2));
    }

    #[test]
    fn test_previous_grid_state() {
        let mut prev = PreviousGridState::default();

        // Create modified cells
        let mut current = vec![Cell::default(); BUFFER_SIZE];
        current[0].char_codepoint = 'A' as u32;
        current[100].char_codepoint = 'B' as u32;
        current[1000].char_codepoint = 'C' as u32;

        // Find dirty cells
        let dirty = prev.find_dirty_cells(&current);
        assert_eq!(dirty.len(), 3);

        // Verify positions
        assert!(dirty.contains(&(0, 0))); // Index 0: x=0, y=0
        assert!(dirty.contains(&(100, 0))); // Index 100: x=100, y=0
        assert!(dirty.contains(&(1000 % GRID_WIDTH as u16, 1000 / GRID_WIDTH as u16))); // Index 1000

        // Update state
        prev.update(&current, 42);
        assert_eq!(prev.last_sequence, 42);

        // No changes should be detected now
        let dirty = prev.find_dirty_cells(&current);
        assert_eq!(dirty.len(), 0);
    }

    #[test]
    fn test_dirty_cell_to_chunk_mapping() {
        // Cell (0, 0) maps to chunk (0, 0)
        assert_eq!(ChunkGrid::cell_to_chunk(0, 0), (0, 0));

        // Cell (63, 31) is still in chunk (0, 0) - last cell of first chunk
        assert_eq!(ChunkGrid::cell_to_chunk(63, 31), (0, 0));

        // Cell (64, 0) is in chunk (1, 0)
        assert_eq!(ChunkGrid::cell_to_chunk(64, 0), (1, 0));

        // Cell (0, 32) is in chunk (0, 1)
        assert_eq!(ChunkGrid::cell_to_chunk(0, 32), (0, 1));

        // Cell (100, 50) is in chunk (1, 1)
        assert_eq!(ChunkGrid::cell_to_chunk(100, 50), (1, 1));
    }

    #[test]
    fn test_u32_to_color() {
        // White color
        let white = u32_to_color(0xFFFFFFFF);
        assert_eq!(white, [1.0, 1.0, 1.0, 1.0]);

        // Black color
        let black = u32_to_color(0x000000FF);
        assert_eq!(black, [0.0, 0.0, 0.0, 1.0]);

        // Red color
        let red = u32_to_color(0xFF0000FF);
        assert_eq!(red, [1.0, 0.0, 0.0, 1.0]);

        // Semi-transparent green
        let green = u32_to_color(0x00FF0080);
        assert_eq!(green[0], 0.0);
        assert_eq!(green[1], 1.0);
        assert_eq!(green[2], 0.0);
        assert!((green[3] - 0.502).abs() < 0.01); // 128/255 ≈ 0.502
    }
}
