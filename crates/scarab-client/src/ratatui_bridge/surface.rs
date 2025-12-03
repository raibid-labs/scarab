// Surface component and buffer management for Ratatui bridge
// This module provides the infrastructure for rendering Ratatui widgets in Bevy

use bevy::prelude::*;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use std::collections::HashMap;

/// Component marking an entity as a Ratatui rendering surface
///
/// A surface represents a rectangular region in terminal grid coordinates
/// where Ratatui widgets can be rendered. Surfaces are positioned and sized
/// in terminal cells, making them coordinate-system compatible with the main
/// terminal grid.
///
/// # Example
/// ```ignore
/// commands.spawn(RatatuiSurface::new(10, 5, 80, 20));
/// ```
#[derive(Component, Debug, Clone)]
pub struct RatatuiSurface {
    /// X position in terminal grid coordinates (columns)
    pub x: u16,
    /// Y position in terminal grid coordinates (rows)
    pub y: u16,
    /// Width in terminal cells (columns)
    pub width: u16,
    /// Height in terminal cells (rows)
    pub height: u16,
    /// Z-index for overlay ordering (higher values render on top)
    /// Default: 100.0 for overlays above terminal content
    pub z_index: f32,
    /// Whether this surface needs re-rendering
    /// Set to true when the widget content changes
    pub dirty: bool,
    /// Visibility flag - hidden surfaces are not rendered
    pub visible: bool,
}

impl RatatuiSurface {
    /// Create a new Ratatui surface at the specified grid coordinates
    ///
    /// # Arguments
    /// * `x` - Column offset from left edge
    /// * `y` - Row offset from top edge
    /// * `width` - Width in terminal cells
    /// * `height` - Height in terminal cells
    ///
    /// # Returns
    /// A new surface with default z_index (100.0), marked dirty and visible
    pub fn new(x: u16, y: u16, width: u16, height: u16) -> Self {
        Self {
            x,
            y,
            width,
            height,
            z_index: 100.0, // Above terminal content (typically z=0-10)
            dirty: true,     // Initial render required
            visible: true,
        }
    }

    /// Create a surface with custom z-index for precise layer ordering
    pub fn with_z_index(mut self, z_index: f32) -> Self {
        self.z_index = z_index;
        self
    }

    /// Create a surface that starts hidden
    pub fn hidden(mut self) -> Self {
        self.visible = false;
        self
    }

    /// Get the Ratatui Rect for this surface
    ///
    /// This rect has origin at (0, 0) relative to the surface's own coordinate space.
    /// Use this when rendering widgets to the buffer.
    pub fn rect(&self) -> Rect {
        Rect::new(0, 0, self.width, self.height)
    }

    /// Get the absolute Rect for this surface in terminal grid coordinates
    ///
    /// Use this for positioning the rendered result within the terminal grid.
    pub fn absolute_rect(&self) -> Rect {
        Rect::new(self.x, self.y, self.width, self.height)
    }

    /// Mark this surface as needing re-render
    ///
    /// Call this after updating widget state to trigger a redraw.
    pub fn mark_dirty(&mut self) {
        self.dirty = true;
    }

    /// Clear the dirty flag after rendering
    pub fn mark_clean(&mut self) {
        self.dirty = false;
    }

    /// Show the surface
    pub fn show(&mut self) {
        if !self.visible {
            self.visible = true;
            self.dirty = true; // Re-render when becoming visible
        }
    }

    /// Hide the surface
    pub fn hide(&mut self) {
        self.visible = false;
    }

    /// Toggle visibility
    pub fn toggle(&mut self) {
        if self.visible {
            self.hide();
        } else {
            self.show();
        }
    }

    /// Update surface position (marks dirty if position changed)
    pub fn set_position(&mut self, x: u16, y: u16) {
        if self.x != x || self.y != y {
            self.x = x;
            self.y = y;
            self.dirty = true;
        }
    }

    /// Update surface size (marks dirty if size changed)
    ///
    /// Note: Resizing a surface will invalidate its buffer in SurfaceBuffers
    pub fn set_size(&mut self, width: u16, height: u16) {
        if self.width != width || self.height != height {
            self.width = width;
            self.height = height;
            self.dirty = true;
        }
    }

    /// Check if a point in grid coordinates is within this surface
    ///
    /// # Arguments
    /// * `col` - Column in terminal grid coordinates
    /// * `row` - Row in terminal grid coordinates
    ///
    /// # Returns
    /// True if the point is within the surface bounds
    pub fn contains_point(&self, col: u16, row: u16) -> bool {
        col >= self.x && col < self.x + self.width &&
        row >= self.y && row < self.y + self.height
    }
}

/// Resource holding rendered surface buffers
///
/// Maps entity IDs to their corresponding Ratatui buffers. Buffers are
/// created on-demand and reused across frames for efficiency.
///
/// # Buffer Lifecycle
/// - Buffers are created when first accessed via `get_or_create`
/// - Buffers are automatically removed when their surface entity is despawned
/// - Buffers are invalidated when surface dimensions change
#[derive(Resource, Default)]
pub struct SurfaceBuffers {
    /// Map of entity -> rendered buffer
    pub(super) buffers: HashMap<Entity, Buffer>,
}

impl SurfaceBuffers {
    /// Get or create a buffer for an entity
    ///
    /// If the entity has no buffer, or the buffer size doesn't match the requested
    /// dimensions, a new buffer is created with the correct size.
    ///
    /// # Arguments
    /// * `entity` - Entity ID of the surface
    /// * `width` - Required buffer width
    /// * `height` - Required buffer height
    ///
    /// # Returns
    /// Mutable reference to the buffer for this entity
    pub fn get_or_create(&mut self, entity: Entity, width: u16, height: u16) -> &mut Buffer {
        self.buffers
            .entry(entity)
            .and_modify(|buffer| {
                // Resize buffer if dimensions changed
                let current_area = buffer.area();
                if current_area.width != width || current_area.height != height {
                    *buffer = Buffer::empty(Rect::new(0, 0, width, height));
                }
            })
            .or_insert_with(|| Buffer::empty(Rect::new(0, 0, width, height)))
    }

    /// Get an existing buffer (read-only)
    ///
    /// Returns None if the buffer doesn't exist yet.
    pub fn get(&self, entity: &Entity) -> Option<&Buffer> {
        self.buffers.get(entity)
    }

    /// Get an existing buffer (mutable)
    ///
    /// Returns None if the buffer doesn't exist yet.
    pub fn get_mut(&mut self, entity: &Entity) -> Option<&mut Buffer> {
        self.buffers.get_mut(entity)
    }

    /// Remove buffer for an entity
    ///
    /// Called automatically when surface entities are despawned.
    pub fn remove(&mut self, entity: &Entity) {
        self.buffers.remove(entity);
    }

    /// Clear all buffers
    ///
    /// Use this for bulk cleanup operations.
    pub fn clear(&mut self) {
        self.buffers.clear();
    }

    /// Get the number of active buffers
    pub fn len(&self) -> usize {
        self.buffers.len()
    }

    /// Check if there are no active buffers
    pub fn is_empty(&self) -> bool {
        self.buffers.is_empty()
    }
}

/// Plugin for Ratatui bridge functionality
///
/// This plugin provides the core infrastructure for rendering Ratatui widgets
/// in Bevy. It manages surface buffers, rendering, and input handling.
///
/// # Systems
/// - `cleanup_removed_surfaces`: Removes buffers when surface entities despawn
/// - `render_surfaces`: Converts Ratatui buffers to Bevy overlays
/// - `cleanup_overlays`: Removes overlay entities when surfaces are despawned
/// - `handle_keyboard_input`: Routes keyboard events to focused surface
/// - `handle_mouse_input`: Routes mouse events and manages focus
/// - `cleanup_focus`: Removes despawned surfaces from focus stack
///
/// # Resources
/// - `SurfaceBuffers`: Central buffer storage
/// - `SurfaceFocus`: Focus management for input routing
///
/// # Events
/// - `SurfaceInputEvent`: Ratatui events sent to surfaces
pub struct RatatuiBridgePlugin;

impl Plugin for RatatuiBridgePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SurfaceBuffers>()
            .init_resource::<super::input::SurfaceFocus>()
            .add_event::<super::input::SurfaceInputEvent>()
            .add_systems(
                Update,
                (
                    cleanup_removed_surfaces,
                    super::renderer::render_surfaces,
                    super::renderer::cleanup_overlays,
                    super::input::handle_keyboard_input,
                    super::input::handle_mouse_input,
                    super::input::cleanup_focus,
                ),
            );

        info!("RatatuiBridgePlugin initialized with rendering and input pipeline");
    }
}

/// Clean up buffers when surface entities are removed
///
/// This system runs every frame and removes buffers for despawned surfaces,
/// preventing memory leaks.
fn cleanup_removed_surfaces(
    mut buffers: ResMut<SurfaceBuffers>,
    mut removed: RemovedComponents<RatatuiSurface>,
) {
    for entity in removed.read() {
        buffers.remove(&entity);
        trace!("Cleaned up buffer for removed surface entity: {:?}", entity);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_surface_creation() {
        let surface = RatatuiSurface::new(10, 20, 80, 24);
        assert_eq!(surface.x, 10);
        assert_eq!(surface.y, 20);
        assert_eq!(surface.width, 80);
        assert_eq!(surface.height, 24);
        assert_eq!(surface.z_index, 100.0);
        assert!(surface.dirty);
        assert!(surface.visible);
    }

    #[test]
    fn test_surface_rect() {
        let surface = RatatuiSurface::new(10, 20, 80, 24);
        let rect = surface.rect();
        assert_eq!(rect.x, 0);
        assert_eq!(rect.y, 0);
        assert_eq!(rect.width, 80);
        assert_eq!(rect.height, 24);
    }

    #[test]
    fn test_surface_absolute_rect() {
        let surface = RatatuiSurface::new(10, 20, 80, 24);
        let rect = surface.absolute_rect();
        assert_eq!(rect.x, 10);
        assert_eq!(rect.y, 20);
        assert_eq!(rect.width, 80);
        assert_eq!(rect.height, 24);
    }

    #[test]
    fn test_surface_dirty_flag() {
        let mut surface = RatatuiSurface::new(0, 0, 10, 10);
        assert!(surface.dirty);

        surface.mark_clean();
        assert!(!surface.dirty);

        surface.mark_dirty();
        assert!(surface.dirty);
    }

    #[test]
    fn test_surface_visibility() {
        let mut surface = RatatuiSurface::new(0, 0, 10, 10);
        assert!(surface.visible);

        surface.hide();
        assert!(!surface.visible);

        surface.show();
        assert!(surface.visible);
        assert!(surface.dirty); // Should be marked dirty when shown

        surface.mark_clean();
        surface.toggle();
        assert!(!surface.visible);

        surface.toggle();
        assert!(surface.visible);
    }

    #[test]
    fn test_surface_position_update() {
        let mut surface = RatatuiSurface::new(0, 0, 10, 10);
        surface.mark_clean();

        surface.set_position(5, 5);
        assert_eq!(surface.x, 5);
        assert_eq!(surface.y, 5);
        assert!(surface.dirty);

        surface.mark_clean();
        surface.set_position(5, 5); // Same position
        assert!(!surface.dirty); // Should not mark dirty
    }

    #[test]
    fn test_surface_size_update() {
        let mut surface = RatatuiSurface::new(0, 0, 10, 10);
        surface.mark_clean();

        surface.set_size(20, 20);
        assert_eq!(surface.width, 20);
        assert_eq!(surface.height, 20);
        assert!(surface.dirty);

        surface.mark_clean();
        surface.set_size(20, 20); // Same size
        assert!(!surface.dirty); // Should not mark dirty
    }

    #[test]
    fn test_surface_builder_pattern() {
        let surface = RatatuiSurface::new(0, 0, 10, 10)
            .with_z_index(200.0)
            .hidden();

        assert_eq!(surface.z_index, 200.0);
        assert!(!surface.visible);
    }

    #[test]
    fn test_surface_contains_point() {
        let surface = RatatuiSurface::new(10, 20, 80, 24);

        // Point inside surface
        assert!(surface.contains_point(10, 20)); // Top-left corner
        assert!(surface.contains_point(50, 30)); // Middle
        assert!(surface.contains_point(89, 43)); // Bottom-right corner (inclusive)

        // Point outside surface
        assert!(!surface.contains_point(9, 20));   // Left of surface
        assert!(!surface.contains_point(10, 19));  // Above surface
        assert!(!surface.contains_point(90, 30));  // Right of surface
        assert!(!surface.contains_point(50, 44));  // Below surface
        assert!(!surface.contains_point(0, 0));    // Far outside
    }

    #[test]
    fn test_buffer_creation() {
        let mut buffers = SurfaceBuffers::default();
        let entity = Entity::from_raw(42);

        let buffer = buffers.get_or_create(entity, 80, 24);
        assert_eq!(buffer.area().width, 80);
        assert_eq!(buffer.area().height, 24);
        assert_eq!(buffers.len(), 1);
    }

    #[test]
    fn test_buffer_resize() {
        let mut buffers = SurfaceBuffers::default();
        let entity = Entity::from_raw(42);

        // Create initial buffer
        buffers.get_or_create(entity, 80, 24);

        // Request different size - should resize
        let buffer = buffers.get_or_create(entity, 100, 30);
        assert_eq!(buffer.area().width, 100);
        assert_eq!(buffer.area().height, 30);
        assert_eq!(buffers.len(), 1); // Still only one buffer
    }

    #[test]
    fn test_buffer_removal() {
        let mut buffers = SurfaceBuffers::default();
        let entity = Entity::from_raw(42);

        buffers.get_or_create(entity, 80, 24);
        assert_eq!(buffers.len(), 1);

        buffers.remove(&entity);
        assert_eq!(buffers.len(), 0);
        assert!(buffers.is_empty());
    }

    #[test]
    fn test_buffer_get() {
        let mut buffers = SurfaceBuffers::default();
        let entity = Entity::from_raw(42);

        assert!(buffers.get(&entity).is_none());

        buffers.get_or_create(entity, 80, 24);
        assert!(buffers.get(&entity).is_some());
    }

    #[test]
    fn test_buffer_clear() {
        let mut buffers = SurfaceBuffers::default();

        buffers.get_or_create(Entity::from_raw(1), 80, 24);
        buffers.get_or_create(Entity::from_raw(2), 100, 30);
        buffers.get_or_create(Entity::from_raw(3), 120, 40);

        assert_eq!(buffers.len(), 3);

        buffers.clear();
        assert!(buffers.is_empty());
    }
}
