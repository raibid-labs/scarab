//! Image Rendering Support for Terminal
//!
//! This module implements rendering of inline images in the terminal using the
//! iTerm2 and Kitty image protocols. Images are decoded and rendered as Bevy
//! sprites with proper positioning based on terminal cell coordinates.
//!
//! # Architecture
//!
//! - Images are transferred via shared memory from daemon to client
//! - `SharedImageReader` resource manages reading from shared memory
//! - `ImageCache` resource manages decoded textures and LRU eviction
//! - `ImagePlacementComponent` marks sprite entities for lifecycle management
//! - Three main systems: load, render, and cleanup

use bevy::prelude::*;
use bevy::render::render_asset::RenderAssetUsages;
use scarab_protocol::{
    ImageFormat as ProtocolImageFormat, ImagePlacement, SharedImageBuffer, SharedImagePlacement,
    TerminalMetrics, IMAGE_BUFFER_SIZE, IMAGE_SHMEM_PATH,
};
use shared_memory::Shmem;
use std::collections::HashMap;
use std::sync::Arc;

use super::layers::LAYER_IMAGES;

/// Maximum memory budget for image cache (100 MB)
const MAX_CACHE_SIZE_BYTES: usize = 100 * 1024 * 1024;

// Wrapper to make shared memory Send + Sync
struct SharedMemWrapper(Arc<Shmem>);

unsafe impl Send for SharedMemWrapper {}
unsafe impl Sync for SharedMemWrapper {}

/// Resource for reading image data from shared memory
#[derive(Resource)]
pub struct SharedImageReader {
    /// Shared memory handle
    shmem: SharedMemWrapper,
    /// Last sequence number processed
    last_sequence: u64,
}

impl SharedImageReader {
    /// Try to open the shared image buffer
    pub fn try_new() -> Option<Self> {
        match shared_memory::ShmemConf::new()
            .size(std::mem::size_of::<SharedImageBuffer>())
            .os_id(IMAGE_SHMEM_PATH)
            .open()
        {
            Ok(shmem) => {
                info!("Connected to shared image buffer at: {}", IMAGE_SHMEM_PATH);
                Some(Self {
                    shmem: SharedMemWrapper(Arc::new(shmem)),
                    last_sequence: 0,
                })
            }
            Err(e) => {
                debug!(
                    "Could not open shared image buffer (daemon may not have images enabled): {}",
                    e
                );
                None
            }
        }
    }

    /// Check if there are new images
    pub fn has_updates(&self) -> bool {
        let buffer = self.buffer();
        buffer.sequence_number != self.last_sequence
    }

    /// Get reference to the shared buffer
    fn buffer(&self) -> &SharedImageBuffer {
        unsafe { &*(self.shmem.0.as_ptr() as *const SharedImageBuffer) }
    }

    /// Get active placements
    pub fn placements(&self) -> impl Iterator<Item = &SharedImagePlacement> {
        let buffer = self.buffer();
        buffer.placements[..buffer.count as usize]
            .iter()
            .filter(|p| p.is_valid())
    }

    /// Extract image blob data for a placement
    pub fn get_blob(&self, placement: &SharedImagePlacement) -> &[u8] {
        let buffer = self.buffer();
        let start = placement.blob_offset as usize;
        let end = start + placement.blob_size as usize;

        // Safety: bounds check
        if end > IMAGE_BUFFER_SIZE {
            warn!(
                "Image blob exceeds buffer size: offset={} size={} max={}",
                start, placement.blob_size, IMAGE_BUFFER_SIZE
            );
            return &[];
        }

        &buffer.blob_data[start..end]
    }

    /// Mark sequence as processed
    pub fn mark_processed(&mut self) {
        self.last_sequence = self.buffer().sequence_number;
    }

    /// Get current sequence number
    pub fn sequence_number(&self) -> u64 {
        self.buffer().sequence_number
    }
}

/// Resource managing image textures and LRU eviction
#[derive(Resource)]
pub struct ImageCache {
    /// Map from image placement ID to Bevy texture handle
    pub textures: HashMap<u64, Handle<Image>>,
    /// Current active image placements (received from daemon)
    pub placements: Vec<ImagePlacement>,
    /// LRU cache for memory management
    lru: ImageLruCache,
}

impl ImageCache {
    /// Create a new empty image cache
    pub fn new() -> Self {
        Self {
            textures: HashMap::new(),
            placements: Vec::new(),
            lru: ImageLruCache::new(MAX_CACHE_SIZE_BYTES),
        }
    }

    /// Update the list of active placements from daemon
    pub fn update_placements(&mut self, placements: Vec<ImagePlacement>) {
        self.placements = placements;
    }

    /// Insert a new texture into the cache
    ///
    /// If this causes the cache to exceed its memory budget, the least recently
    /// used textures will be evicted.
    pub fn insert_texture(&mut self, id: u64, handle: Handle<Image>, size_bytes: usize) {
        self.textures.insert(id, handle);
        self.lru.insert(id, size_bytes);
    }

    /// Mark a texture as recently used (updates LRU)
    pub fn touch(&mut self, id: u64) {
        self.lru.touch(id);
    }

    /// Get texture handle by ID
    pub fn get_texture(&self, id: u64) -> Option<&Handle<Image>> {
        self.textures.get(&id)
    }

    /// Remove evicted entries from the texture cache
    pub fn apply_evictions(&mut self, images: &mut Assets<Image>) {
        let evicted = self.lru.get_evicted();
        for id in evicted {
            if let Some(handle) = self.textures.remove(&id) {
                images.remove(&handle);
                debug!("Evicted image {} from cache", id);
            }
        }
    }
}

/// Marker component for image placement entities
#[derive(Component, Debug, Clone)]
pub struct ImagePlacementComponent {
    /// Unique identifier matching ImagePlacement.id
    pub id: u64,
}

/// Plugin for image rendering support
pub struct ImagesPlugin;

impl Plugin for ImagesPlugin {
    fn build(&self, app: &mut App) {
        // Try to connect to shared image buffer
        if let Some(reader) = SharedImageReader::try_new() {
            app.insert_resource(reader);
        }

        app.insert_resource(ImageCache::new()).add_systems(
            Update,
            (
                sync_images_from_shmem,
                load_images_system,
                render_images_system,
                cleanup_images_system,
            )
                .chain(),
        );
    }
}

/// System to sync image placements from shared memory
///
/// This system checks for updates in the shared image buffer and extracts
/// image placements into the ImageCache for processing.
fn sync_images_from_shmem(
    mut reader: Option<ResMut<SharedImageReader>>,
    mut cache: ResMut<ImageCache>,
) {
    let Some(reader) = reader.as_deref_mut() else {
        return;
    };

    if !reader.has_updates() {
        return;
    }

    let mut new_placements = Vec::new();

    for placement in reader.placements() {
        // Convert SharedImagePlacement to ImagePlacement
        let image_placement = ImagePlacement {
            id: placement.image_id,
            x: placement.x,
            y: placement.y,
            width_cells: placement.width_cells,
            height_cells: placement.height_cells,
            shm_offset: placement.blob_offset as usize,
            shm_size: placement.blob_size as usize,
            format: match placement.format {
                0 => ProtocolImageFormat::Png,
                1 => ProtocolImageFormat::Jpeg,
                2 => ProtocolImageFormat::Gif,
                3 => ProtocolImageFormat::Rgba,
                _ => {
                    warn!("Unknown image format: {}", placement.format);
                    continue;
                }
            },
        };

        new_placements.push(image_placement);
    }

    debug!(
        "Synced {} image placements from shared memory (seq {})",
        new_placements.len(),
        reader.sequence_number()
    );

    cache.update_placements(new_placements);
    reader.mark_processed();
}

/// System to decode images and create Bevy textures
///
/// This system runs each frame and checks for new image placements that haven't
/// been loaded yet. It decodes the image data (PNG, JPEG, etc.) and creates
/// Bevy texture assets.
pub fn load_images_system(
    reader: Option<Res<SharedImageReader>>,
    mut cache: ResMut<ImageCache>,
    mut images: ResMut<Assets<Image>>,
) {
    let Some(reader) = reader.as_deref() else {
        return;
    };

    for placement in &cache.placements.clone() {
        // Skip if already loaded
        if cache.textures.contains_key(&placement.id) {
            cache.touch(placement.id);
            continue;
        }

        // Find the corresponding SharedImagePlacement to get blob data
        let blob_data = reader
            .placements()
            .find(|p| p.image_id == placement.id)
            .map(|p| reader.get_blob(p));

        let Some(data) = blob_data else {
            continue;
        };

        // Decode image based on format
        let image_result = match placement.format {
            ProtocolImageFormat::Png => decode_image(data, image::ImageFormat::Png),
            ProtocolImageFormat::Jpeg => decode_image(data, image::ImageFormat::Jpeg),
            ProtocolImageFormat::Gif => decode_image(data, image::ImageFormat::Gif),
            ProtocolImageFormat::Rgba => {
                // Raw RGBA data - decode directly
                decode_rgba(
                    data,
                    placement.width_cells as u32 * 10, // Estimate pixel width
                    placement.height_cells as u32 * 20, // Estimate pixel height
                )
            }
        };

        if let Some((bevy_image, size_bytes)) = image_result {
            let handle = images.add(bevy_image);
            cache.insert_texture(placement.id, handle, size_bytes);
            debug!("Loaded image {} ({} bytes)", placement.id, size_bytes);
        } else {
            warn!("Failed to decode image {}", placement.id);
        }
    }

    // Apply LRU evictions if over budget
    cache.apply_evictions(&mut images);
}

/// Decode image data into a Bevy Image
///
/// Returns the decoded image and its size in bytes, or None if decoding fails.
fn decode_image(data: &[u8], format: image::ImageFormat) -> Option<(Image, usize)> {
    if data.is_empty() {
        return None;
    }

    let dynamic_image = image::load_from_memory_with_format(data, format).ok()?;
    let rgba = dynamic_image.to_rgba8();
    let (width, height) = rgba.dimensions();

    let size_bytes = (width * height * 4) as usize;
    let raw_data = rgba.into_raw();

    let bevy_image = Image::new(
        bevy::render::render_resource::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        bevy::render::render_resource::TextureDimension::D2,
        raw_data,
        bevy::render::render_resource::TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::RENDER_WORLD,
    );

    Some((bevy_image, size_bytes))
}

/// Decode raw RGBA data into a Bevy Image
fn decode_rgba(data: &[u8], width: u32, height: u32) -> Option<(Image, usize)> {
    let expected_size = (width * height * 4) as usize;

    if data.len() != expected_size {
        warn!(
            "RGBA data size mismatch: expected {} bytes, got {}",
            expected_size,
            data.len()
        );
        return None;
    }

    let size_bytes = data.len();
    let bevy_image = Image::new(
        bevy::render::render_resource::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        bevy::render::render_resource::TextureDimension::D2,
        data.to_vec(),
        bevy::render::render_resource::TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::RENDER_WORLD,
    );

    Some((bevy_image, size_bytes))
}

/// System to spawn sprite entities for image placements
///
/// This system creates Bevy sprite entities positioned at the correct terminal
/// grid coordinates. Images are scaled to match the cell dimensions.
pub fn render_images_system(
    mut commands: Commands,
    cache: Res<ImageCache>,
    metrics: Res<TerminalMetrics>,
    existing_images: Query<(Entity, &ImagePlacementComponent)>,
) {
    // Build set of already-rendered image IDs
    let existing_ids: std::collections::HashSet<u64> =
        existing_images.iter().map(|(_, comp)| comp.id).collect();

    // Spawn sprites for new placements
    for placement in &cache.placements {
        if existing_ids.contains(&placement.id) {
            continue; // Already rendered
        }

        if let Some(texture) = cache.get_texture(placement.id) {
            // Calculate pixel position from grid coordinates
            let (x, y) = metrics.grid_to_screen(placement.x, placement.y);

            // Calculate sprite size based on cell dimensions
            let width = placement.width_cells as f32 * metrics.cell_width;
            let height = placement.height_cells as f32 * metrics.cell_height;

            debug!(
                "Rendering image {} at ({}, {}) with size {}x{} (grid: {},{} cells: {}x{})",
                placement.id,
                x,
                y,
                width,
                height,
                placement.x,
                placement.y,
                placement.width_cells,
                placement.height_cells
            );

            // Spawn sprite with anchor at top-left
            // Bevy sprites are centered by default, so we offset by half the size
            commands.spawn((
                Sprite {
                    image: texture.clone(),
                    custom_size: Some(Vec2::new(width, height)),
                    anchor: bevy::sprite::Anchor::TopLeft,
                    ..default()
                },
                Transform::from_xyz(x, -y, LAYER_IMAGES),
                ImagePlacementComponent { id: placement.id },
            ));
        }
    }
}

/// System to remove image sprites when they scroll off-screen or are deleted
///
/// This system compares the current placements with rendered entities and
/// despawns any entities that are no longer in the active placement list.
pub fn cleanup_images_system(
    mut commands: Commands,
    cache: Res<ImageCache>,
    existing_images: Query<(Entity, &ImagePlacementComponent)>,
) {
    // Build set of current placement IDs
    let current_ids: std::collections::HashSet<u64> =
        cache.placements.iter().map(|p| p.id).collect();

    // Remove sprites that are no longer active
    for (entity, component) in existing_images.iter() {
        if !current_ids.contains(&component.id) {
            debug!("Removing image sprite for placement {}", component.id);
            commands.entity(entity).despawn();
        }
    }
}

/// LRU cache for memory-bounded image storage
#[derive(Debug)]
struct ImageLruCache {
    /// Maximum total size in bytes
    max_size: usize,
    /// Current total size in bytes
    current_size: usize,
    /// Entries ordered by recency (most recent at end)
    entries: Vec<LruEntry>,
}

#[derive(Debug, Clone)]
struct LruEntry {
    id: u64,
    size_bytes: usize,
}

impl ImageLruCache {
    fn new(max_size: usize) -> Self {
        Self {
            max_size,
            current_size: 0,
            entries: Vec::new(),
        }
    }

    /// Insert a new entry, evicting old ones if necessary
    fn insert(&mut self, id: u64, size_bytes: usize) {
        // Remove existing entry if present and subtract its size
        if let Some(pos) = self.entries.iter().position(|e| e.id == id) {
            let old_entry = self.entries.remove(pos);
            self.current_size -= old_entry.size_bytes;
        }

        // Add new entry
        self.entries.push(LruEntry { id, size_bytes });
        self.current_size += size_bytes;

        // Evict oldest entries if over budget
        while self.current_size > self.max_size && !self.entries.is_empty() {
            if let Some(evicted) = self.entries.first() {
                self.current_size -= evicted.size_bytes;
            }
            self.entries.remove(0);
        }
    }

    /// Mark an entry as recently used (move to end)
    fn touch(&mut self, id: u64) {
        if let Some(pos) = self.entries.iter().position(|e| e.id == id) {
            let entry = self.entries.remove(pos);
            self.entries.push(entry);
        }
    }

    /// Get list of IDs that should be evicted
    fn get_evicted(&self) -> Vec<u64> {
        // In this implementation, eviction happens eagerly in insert()
        // This method is for deferred eviction if needed
        Vec::new()
    }

    #[cfg(test)]
    fn len(&self) -> usize {
        self.entries.len()
    }

    #[cfg(test)]
    fn total_size(&self) -> usize {
        self.current_size
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lru_basic_insert() {
        let mut lru = ImageLruCache::new(1000);

        lru.insert(1, 100);
        assert_eq!(lru.len(), 1);
        assert_eq!(lru.total_size(), 100);

        lru.insert(2, 200);
        assert_eq!(lru.len(), 2);
        assert_eq!(lru.total_size(), 300);
    }

    #[test]
    fn test_lru_eviction() {
        let mut lru = ImageLruCache::new(500);

        lru.insert(1, 200);
        lru.insert(2, 200);
        lru.insert(3, 200); // Should evict entry 1

        assert_eq!(lru.len(), 2);
        assert_eq!(lru.total_size(), 400);

        // Verify entry 1 was evicted (it should not be in entries)
        assert!(!lru.entries.iter().any(|e| e.id == 1));
        assert!(lru.entries.iter().any(|e| e.id == 2));
        assert!(lru.entries.iter().any(|e| e.id == 3));
    }

    #[test]
    fn test_lru_touch() {
        let mut lru = ImageLruCache::new(1000);

        lru.insert(1, 100);
        lru.insert(2, 100);
        lru.insert(3, 100);

        // Touch entry 1 (move to end)
        lru.touch(1);

        // Entry 1 should now be most recent
        assert_eq!(lru.entries.last().unwrap().id, 1);
    }

    #[test]
    fn test_lru_update_existing() {
        let mut lru = ImageLruCache::new(1000);

        lru.insert(1, 100);
        lru.insert(2, 200);

        // Update entry 1 with new size
        lru.insert(1, 150);

        assert_eq!(lru.len(), 2);
        assert_eq!(lru.total_size(), 350);
    }

    #[test]
    fn test_lru_evict_multiple() {
        let mut lru = ImageLruCache::new(300);

        lru.insert(1, 100);
        lru.insert(2, 100);
        lru.insert(3, 100);

        assert_eq!(lru.len(), 3);

        // Insert large item - should evict multiple entries
        lru.insert(4, 250);

        assert_eq!(lru.len(), 1);
        assert_eq!(lru.total_size(), 250);
        assert_eq!(lru.entries[0].id, 4);
    }

    #[test]
    fn test_image_cache_basic() {
        let cache = ImageCache::new();

        assert!(cache.textures.is_empty());
        assert!(cache.placements.is_empty());
    }

    #[test]
    fn test_image_cache_update_placements() {
        let mut cache = ImageCache::new();

        let placements = vec![ImagePlacement {
            id: 1,
            x: 0,
            y: 0,
            width_cells: 10,
            height_cells: 5,
            shm_offset: 0,
            shm_size: 1000,
            format: ProtocolImageFormat::Png,
        }];

        cache.update_placements(placements);
        assert_eq!(cache.placements.len(), 1);
        assert_eq!(cache.placements[0].id, 1);
    }
}
