# Graphics Inspector Implementation

## Overview

The Graphics Inspector is a real-time debugging tool for monitoring inline terminal images rendered via Sixel, Kitty, and iTerm2 protocols.

**Location**: `/home/beengud/raibid-labs/scarab/crates/scarab-client/src/graphics_inspector.rs`

## Architecture

### Components

#### 1. GraphicsInspectorState (Resource)
```rust
pub struct GraphicsInspectorState {
    pub visible: bool,              // Inspector window visibility
    pub selected_index: usize,       // Currently selected image
    pub filter_text: String,         // Search filter
    pub sort_mode: ImageSortMode,    // Sort order
    pub stats: GraphicsStats,        // Real-time statistics
}
```

#### 2. GraphicsStats
```rust
pub struct GraphicsStats {
    pub total_loaded: usize,    // Total images in cache
    pub total_memory: usize,    // Memory usage in bytes
    pub visible_count: usize,   // Currently visible images
    pub peak_memory: usize,     // Peak memory usage
}
```

#### 3. ImageSortMode (Enum)
- `ById`: Chronological order by unique ID
- `ByPosition`: Grid position (y, x)
- `BySize`: Cell dimensions (largest first)
- `ByProtocol`: Format type (PNG, JPEG, GIF, RGBA)

### Plugin Integration

#### GraphicsInspectorPlugin
```rust
impl Plugin for GraphicsInspectorPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(GraphicsInspectorState::default())
            .add_systems(Update, (
                toggle_inspector_system,
                render_inspector_system
            ));
    }
}
```

## Systems

### 1. toggle_inspector_system
**Purpose**: Handle keyboard input to show/hide inspector

**Trigger**: `Ctrl+Shift+G`

**Implementation**:
```rust
fn toggle_inspector_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut state: ResMut<GraphicsInspectorState>,
) {
    // Check for Ctrl+Shift+G combination
    if ctrl_pressed && shift_pressed && g_just_pressed {
        state.visible = !state.visible;
    }
}
```

### 2. render_inspector_system
**Purpose**: Render the egui inspector window

**Dependencies**:
- `GraphicsInspectorState`: UI state
- `ImageCache`: Image placement data
- `SharedImageReader`: Live image buffer access
- `TerminalMetrics`: Coordinate conversion

**UI Layout**:
```
┌─────────────────────────────────────────────┐
│ Toolbar: Filter | Sort | Actions             │
├─────────────────────────────────────────────┤
│ Stats: Total: 5 | Visible: 5 | Memory: 2.5MB│
├──────────────┬──────────────────────────────┤
│ Image List   │ Image Details                 │
│              │                               │
│ #1 - PNG     │ ID: 1                        │
│ #2 - JPEG    │ Protocol: PNG                │
│ #3 - GIF     │ Position: (10, 5)            │
│              │ Dimensions: 20x10 cells       │
│              │ Memory: 12.3 KB               │
└──────────────┴──────────────────────────────┘
```

## Integration Points

### Shared Memory Integration
```rust
// Read from SharedImageReader
let active_count = reader.placements().count();

// Access image cache
let placements = cache.placements.clone();
```

### Coordinate Conversion
```rust
// Convert grid to screen coordinates
let (screen_x, screen_y) = metrics.grid_to_screen(placement.x, placement.y);
```

### Clipboard Integration
```rust
#[cfg(not(target_arch = "wasm32"))]
{
    use arboard::Clipboard;
    if let Ok(mut clipboard) = Clipboard::new() {
        clipboard.set_text(placement.id.to_string());
    }
}
```

## Data Flow

```
Daemon (scarab-daemon)
  ├─ Image Protocol Parsing (sixel.rs, kitty.rs, iterm2.rs)
  ├─ ImagePlacementState (placement.rs)
  └─ Write to SharedImageBuffer (shared memory)
      │
      ↓
Client (scarab-client)
  ├─ SharedImageReader: Read from shared memory
  ├─ ImageCache: Decode and cache textures
  ├─ ImagesPlugin: Render image sprites
  └─ GraphicsInspector: Monitor and debug
```

## Memory Management

### Statistics Calculation
```rust
fn update_stats(
    stats: &mut GraphicsStats,
    cache: &ImageCache,
    reader: Option<&SharedImageReader>
) {
    // Count active placements
    stats.total_loaded = cache.placements.len();

    // Sum compressed data sizes
    stats.total_memory = cache.placements
        .iter()
        .map(|p| p.shm_size)
        .sum();

    // Track peak usage
    if stats.total_memory > stats.peak_memory {
        stats.peak_memory = stats.total_memory;
    }
}
```

### Size Formatting
```rust
fn format_bytes(bytes: usize) -> String {
    // 1024 B   -> "1.00 KB"
    // 1048576 B -> "1.00 MB"
    // etc.
}
```

## Export Functionality

### Metadata Export
Exports JSON file with image details:
```json
{
  "id": 42,
  "format": "Png",
  "position": {
    "x": 10,
    "y": 5
  },
  "dimensions": {
    "width_cells": 20,
    "height_cells": 10
  },
  "shared_memory": {
    "offset": 4096,
    "size": 12345
  }
}
```

**Filename**: `image_{id}_metadata.json`

## Protocol Support

### Format Detection
```rust
fn format_protocol(format: ProtocolImageFormat) -> &'static str {
    match format {
        ProtocolImageFormat::Png => "PNG",
        ProtocolImageFormat::Jpeg => "JPEG",
        ProtocolImageFormat::Gif => "GIF",
        ProtocolImageFormat::Rgba => "RGBA (Sixel)",
    }
}
```

### Protocol Sources
- **PNG/JPEG/GIF**: iTerm2 inline images, Kitty graphics protocol
- **RGBA**: Sixel raster graphics (decoded to raw pixels)

## UI Features

### Filtering
```rust
// Filter by ID or protocol name
if !state.filter_text.is_empty() {
    let filter = state.filter_text.to_lowercase();
    placements.retain(|p| {
        p.id.to_string().contains(&filter)
        || format!("{:?}", p.format).to_lowercase().contains(&filter)
    });
}
```

### Sorting
```rust
match state.sort_mode {
    ImageSortMode::ById =>
        placements.sort_by_key(|p| p.id),
    ImageSortMode::ByPosition =>
        placements.sort_by_key(|p| (p.y, p.x)),
    ImageSortMode::BySize =>
        placements.sort_by_key(|p| Reverse(p.width_cells * p.height_cells)),
    ImageSortMode::ByProtocol =>
        placements.sort_by_key(|p| format!("{:?}", p.format)),
}
```

## Testing

### Unit Tests
```rust
#[cfg(test)]
mod tests {
    #[test]
    fn test_format_bytes() {
        assert_eq!(format_bytes(1024), "1.00 KB");
        assert_eq!(format_bytes(1048576), "1.00 MB");
    }

    #[test]
    fn test_format_protocol() {
        assert_eq!(format_protocol(ProtocolImageFormat::Png), "PNG");
    }

    #[test]
    fn test_inspector_state_default() {
        let state = GraphicsInspectorState::default();
        assert!(!state.visible);
    }
}
```

## Dependencies

### Crate Dependencies
```toml
bevy_egui = "0.31"      # UI framework
serde_json = "1.0"      # Metadata export
arboard = "3.3"         # Clipboard support
```

### Internal Dependencies
- `scarab_protocol`: Image types and shared memory layout
- `rendering::ImageCache`: Image placement data
- `rendering::SharedImageReader`: Shared memory access

## Performance Considerations

### Efficient Updates
- Only processes when inspector is visible
- Clones placement list for sorting (minimal overhead)
- No texture decoding in inspector (reads metadata only)

### Memory Impact
- Minimal overhead: ~1KB for state
- No image data duplication
- Statistics calculated on-demand

## Future Enhancements

### Planned Features
1. **Image Preview**: Thumbnail rendering in details panel
2. **Protocol Details**: Show Sixel palette, Kitty chunking info
3. **Export Image Data**: Save decoded image to file
4. **Delete Controls**: Remove specific images from cache
5. **Performance Metrics**: Load time, decode time tracking
6. **Network Stats**: Transfer size for remote sessions

### Technical Debt
- Add clipboard support for WASM target
- Implement image preview using bevy textures
- Add batch export for all images
- Support image deletion/clearing

## Integration Checklist

- [x] Create `graphics_inspector.rs` module
- [x] Add to `lib.rs` exports
- [x] Register plugin in `main.rs`
- [x] Add `bevy_egui` dependency
- [x] Document keyboard shortcuts
- [x] Write unit tests
- [x] Create user documentation
- [x] Add implementation notes

## Usage Example

```rust
// In main.rs
use scarab_client::GraphicsInspectorPlugin;

app.add_plugins(GraphicsInspectorPlugin);
println!("Graphics Inspector enabled - Press Ctrl+Shift+G to open");
```

## Troubleshooting

### Common Issues

**Inspector won't open**
- Verify keybinding not conflicting
- Check egui plugin is loaded
- Ensure state resource exists

**No images shown**
- Confirm daemon has images enabled
- Verify shared memory connection
- Check ImageCache is populated

**Statistics not updating**
- Ensure SharedImageReader resource exists
- Verify sequence numbers are incrementing
- Check for daemon errors

## Related Files

- `/home/beengud/raibid-labs/scarab/crates/scarab-client/src/graphics_inspector.rs` - Implementation
- `/home/beengud/raibid-labs/scarab/crates/scarab-client/src/rendering/images.rs` - Image rendering
- `/home/beengud/raibid-labs/scarab/crates/scarab-daemon/src/images/` - Protocol parsing
- `/home/beengud/raibid-labs/scarab/crates/scarab-protocol/src/lib.rs` - Shared memory types
- `/home/beengud/raibid-labs/scarab/docs/graphics-inspector.md` - User documentation
