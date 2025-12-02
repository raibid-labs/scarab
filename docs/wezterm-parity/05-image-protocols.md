# WS-5: Image Protocol Support

**Workstream ID:** WS-5
**Priority:** P2 (Feature Parity)
**Estimated Complexity:** High
**Dependencies:** None (can run in parallel)

## Overview

Modern terminals support inline image display through various protocols. WezTerm supports iTerm2, Kitty, and Sixel protocols. This workstream implements image protocol support in Scarab's VTE parser and Bevy renderer.

## Protocol Comparison

| Protocol | Format | Quality | Complexity | Adoption |
|----------|--------|---------|------------|----------|
| **iTerm2** | Base64 PNG/JPEG | Excellent | Low | High |
| **Kitty** | Chunked, shared mem | Excellent | High | Medium |
| **Sixel** | Palette-based bitmap | Lower | Medium | Legacy |

### Recommendation

**Priority Order:**
1. **iTerm2** - Simplest, widely supported, good quality
2. **Kitty** - Modern, efficient, good for large images
3. **Sixel** - Legacy support, lower priority

## iTerm2 Image Protocol

### Escape Sequence Format

```
ESC ] 1337 ; File = [arguments] : [base64 data] BEL
```

**Arguments:**
- `name=<base64 filename>` - Optional filename
- `size=<bytes>` - File size in bytes
- `width=<N>` - Width in cells, pixels, or percent
- `height=<N>` - Height in cells, pixels, or percent
- `preserveAspectRatio=<0|1>` - Maintain aspect ratio
- `inline=<0|1>` - Display inline (1) or as download (0)
- `doNotMoveCursor=<0|1>` - WezTerm extension

### Parser Implementation

```rust
// In scarab-daemon/src/vte.rs

#[derive(Debug)]
pub struct ImageData {
    pub data: Vec<u8>,
    pub width: ImageSize,
    pub height: ImageSize,
    pub preserve_aspect_ratio: bool,
    pub inline: bool,
    pub do_not_move_cursor: bool,
    pub filename: Option<String>,
}

#[derive(Debug, Clone, Copy)]
pub enum ImageSize {
    Auto,
    Cells(u16),
    Pixels(u32),
    Percent(f32),
}

impl TerminalState {
    fn handle_osc(&mut self, params: &[&[u8]]) {
        // OSC 1337 ; File = ...
        if params.len() >= 2 && params[0] == b"1337" {
            if let Some(file_data) = self.parse_iterm2_image(&params[1..]) {
                self.handle_inline_image(file_data);
            }
        }
    }

    fn parse_iterm2_image(&self, params: &[&[u8]]) -> Option<ImageData> {
        // Parse "File=name=...:size=...:width=...:<base64>"
        let combined = params.join(&b';');
        let s = std::str::from_utf8(&combined).ok()?;

        if !s.starts_with("File=") {
            return None;
        }

        let s = &s[5..];  // Skip "File="

        // Split on ':' to separate arguments from data
        let parts: Vec<&str> = s.splitn(2, ':').collect();
        let (args_str, data_b64) = match parts.as_slice() {
            [args, data] => (*args, *data),
            _ => return None,
        };

        let mut image = ImageData::default();

        // Parse arguments
        for arg in args_str.split(';') {
            let kv: Vec<&str> = arg.splitn(2, '=').collect();
            if let [key, value] = kv.as_slice() {
                match *key {
                    "name" => {
                        image.filename = base64::decode(value).ok()
                            .and_then(|b| String::from_utf8(b).ok());
                    }
                    "width" => image.width = parse_image_size(value),
                    "height" => image.height = parse_image_size(value),
                    "preserveAspectRatio" => image.preserve_aspect_ratio = *value != "0",
                    "inline" => image.inline = *value == "1",
                    "doNotMoveCursor" => image.do_not_move_cursor = *value == "1",
                    _ => {}
                }
            }
        }

        // Decode base64 image data
        image.data = base64::decode(data_b64).ok()?;

        Some(image)
    }
}

fn parse_image_size(s: &str) -> ImageSize {
    if s == "auto" {
        ImageSize::Auto
    } else if s.ends_with('%') {
        s[..s.len()-1].parse().map(ImageSize::Percent).unwrap_or(ImageSize::Auto)
    } else if s.ends_with("px") {
        s[..s.len()-2].parse().map(ImageSize::Pixels).unwrap_or(ImageSize::Auto)
    } else {
        s.parse().map(ImageSize::Cells).unwrap_or(ImageSize::Auto)
    }
}
```

### Shared Memory Image Transfer

Images are large—send via shared memory, not IPC messages:

```rust
// In scarab-protocol
pub struct ImagePlacement {
    pub id: u64,
    pub x: u16,           // Grid column
    pub y: u16,           // Grid row
    pub width_cells: u16,
    pub height_cells: u16,
    pub shm_offset: usize,
    pub shm_size: usize,
    pub format: ImageFormat,
}

pub enum ImageFormat {
    Png,
    Jpeg,
    Gif,
    Rgba,  // Raw RGBA pixels
}

// Separate shared memory region for images
pub struct ImageSharedMemory {
    pub images: Vec<ImagePlacement>,
    pub data: Vec<u8>,  // Pooled image data
}
```

### Bevy Renderer

```rust
// In scarab-client/src/rendering/images.rs

#[derive(Resource)]
pub struct ImageCache {
    pub textures: HashMap<u64, Handle<Image>>,
    pub placements: Vec<ImagePlacement>,
}

pub fn load_images_system(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    mut cache: ResMut<ImageCache>,
    image_shm: Res<ImageSharedMemory>,
) {
    for placement in &image_shm.placements {
        if cache.textures.contains_key(&placement.id) {
            continue;  // Already loaded
        }

        // Extract image data from shared memory
        let data = &image_shm.data[placement.shm_offset..placement.shm_offset + placement.shm_size];

        // Decode image
        let dynamic_image = match placement.format {
            ImageFormat::Png => image::load_from_memory_with_format(data, image::ImageFormat::Png),
            ImageFormat::Jpeg => image::load_from_memory_with_format(data, image::ImageFormat::Jpeg),
            _ => continue,
        };

        if let Ok(img) = dynamic_image {
            let rgba = img.to_rgba8();
            let (width, height) = rgba.dimensions();

            let bevy_image = Image::new(
                Extent3d { width, height, depth_or_array_layers: 1 },
                TextureDimension::D2,
                rgba.into_raw(),
                TextureFormat::Rgba8UnormSrgb,
                RenderAssetUsages::RENDER_WORLD,
            );

            let handle = images.add(bevy_image);
            cache.textures.insert(placement.id, handle);
        }
    }
}

pub fn render_images_system(
    mut commands: Commands,
    cache: Res<ImageCache>,
    image_shm: Res<ImageSharedMemory>,
    metrics: Res<TerminalMetrics>,
    mut image_query: Query<(Entity, &ImagePlacementComponent)>,
) {
    let existing: HashSet<u64> = image_query.iter().map(|(_, c)| c.id).collect();

    for placement in &image_shm.placements {
        if existing.contains(&placement.id) {
            continue;  // Already rendered
        }

        if let Some(texture) = cache.textures.get(&placement.id) {
            // Calculate pixel position from grid position
            let x = placement.x as f32 * metrics.cell_width;
            let y = placement.y as f32 * metrics.cell_height;
            let width = placement.width_cells as f32 * metrics.cell_width;
            let height = placement.height_cells as f32 * metrics.cell_height;

            commands.spawn((
                SpriteBundle {
                    texture: texture.clone(),
                    sprite: Sprite {
                        custom_size: Some(Vec2::new(width, height)),
                        ..default()
                    },
                    transform: Transform::from_xyz(x, -y, 1.0),  // Z=1 to render above text
                    ..default()
                },
                ImagePlacementComponent { id: placement.id },
            ));
        }
    }
}

#[derive(Component)]
pub struct ImagePlacementComponent {
    pub id: u64,
}
```

## Kitty Graphics Protocol

### Overview

Kitty's protocol is more complex, supporting:
- Chunked transmission (for large images)
- Shared memory transmission
- Image IDs and placement IDs
- Animation support
- Unicode placeholders

### Escape Sequence Format

```
ESC _ G <control data> ; <payload> ESC \
```

**Control Data Keys:**
- `a=<action>` - t(transmit), T(transmit+display), p(put), d(delete)
- `f=<format>` - 24(RGB), 32(RGBA), 100(PNG)
- `t=<transmission>` - d(direct), f(file), t(temp file), s(shared memory)
- `s=<width>`, `v=<height>` - Image dimensions
- `i=<id>` - Image ID
- `p=<placement_id>` - Placement ID
- `m=<more>` - More chunks coming

### Parser Implementation

```rust
// Kitty graphics use APC (Application Program Command)
impl TerminalState {
    fn handle_apc(&mut self, data: &[u8]) {
        if data.starts_with(b"G") {
            self.parse_kitty_graphics(&data[1..]);
        }
    }

    fn parse_kitty_graphics(&mut self, data: &[u8]) {
        // Split control;payload
        let parts: Vec<&[u8]> = data.splitn(2, |&b| b == b';').collect();
        let control = parts.get(0).unwrap_or(&[].as_slice());
        let payload = parts.get(1).unwrap_or(&[].as_slice());

        let mut params = KittyGraphicsParams::default();

        // Parse control keys
        for kv in control.split(|&b| b == b',') {
            if kv.len() >= 3 && kv[1] == b'=' {
                let key = kv[0];
                let value = &kv[2..];
                match key {
                    b'a' => params.action = value.first().copied(),
                    b'f' => params.format = parse_u32(value),
                    b't' => params.transmission = value.first().copied(),
                    b's' => params.width = parse_u32(value),
                    b'v' => params.height = parse_u32(value),
                    b'i' => params.image_id = parse_u32(value),
                    b'p' => params.placement_id = parse_u32(value),
                    b'm' => params.more = value.first() == Some(&b'1'),
                    _ => {}
                }
            }
        }

        self.process_kitty_graphics(params, payload);
    }
}

#[derive(Default)]
struct KittyGraphicsParams {
    action: Option<u8>,
    format: Option<u32>,
    transmission: Option<u8>,
    width: Option<u32>,
    height: Option<u32>,
    image_id: Option<u32>,
    placement_id: Option<u32>,
    more: bool,
}
```

## Sixel Support

### Overview

Sixel is a legacy DEC format that encodes images as character sequences. Each "sixel" represents a 1x6 pixel column.

### Parser Implementation

```rust
// Sixel starts with DCS (Device Control String)
// DCS P1 ; P2 ; P3 q <sixel data> ST

impl TerminalState {
    fn handle_dcs(&mut self, data: &[u8]) {
        if data.contains(&b'q') {
            self.parse_sixel(data);
        }
    }

    fn parse_sixel(&mut self, data: &[u8]) {
        // Find 'q' marker
        let q_pos = data.iter().position(|&b| b == b'q').unwrap_or(0);
        let sixel_data = &data[q_pos + 1..];

        let mut decoder = SixelDecoder::new();
        decoder.decode(sixel_data);

        if let Some(image) = decoder.finish() {
            self.handle_inline_image(image);
        }
    }
}

pub struct SixelDecoder {
    width: u32,
    height: u32,
    palette: [u32; 256],
    pixels: Vec<u8>,
    x: u32,
    y: u32,
    current_color: u8,
}

impl SixelDecoder {
    pub fn decode(&mut self, data: &[u8]) {
        let mut i = 0;
        while i < data.len() {
            match data[i] {
                b'#' => {
                    // Color definition or selection
                    i = self.parse_color(&data[i..]);
                }
                b'$' => {
                    // Carriage return
                    self.x = 0;
                    i += 1;
                }
                b'-' => {
                    // New line (move down 6 pixels)
                    self.x = 0;
                    self.y += 6;
                    i += 1;
                }
                b'!' => {
                    // Repeat
                    i = self.parse_repeat(&data[i..]);
                }
                b'?' ..= b'~' => {
                    // Sixel data (6 vertical pixels)
                    self.draw_sixel(data[i] - b'?');
                    self.x += 1;
                    i += 1;
                }
                _ => i += 1,
            }
        }
    }

    fn draw_sixel(&mut self, bits: u8) {
        for bit in 0..6 {
            if bits & (1 << bit) != 0 {
                let px = self.x;
                let py = self.y + bit as u32;
                // Set pixel at (px, py) to current_color
            }
        }
    }
}
```

## Image Lifecycle Management

### Placement Model

Images are placed at grid coordinates and can be:
- Overwritten (text writes over image area)
- Scrolled (image moves with scrollback)
- Deleted (explicit delete command or scroll out)

```rust
#[derive(Debug)]
pub struct ImagePlacementState {
    pub placements: Vec<ActivePlacement>,
    pub image_data: HashMap<u64, ImageData>,
    pub next_id: AtomicU64,
}

#[derive(Debug)]
pub struct ActivePlacement {
    pub id: u64,
    pub image_id: u64,
    pub x: u16,
    pub y: u16,
    pub width: u16,
    pub height: u16,
    pub z_index: i32,
}

impl ImagePlacementState {
    /// Handle scrolling - adjust y coordinates
    pub fn scroll(&mut self, lines: i32) {
        for placement in &mut self.placements {
            placement.y = (placement.y as i32 + lines).max(0) as u16;
        }

        // Remove placements that scrolled off
        self.placements.retain(|p| p.y < self.visible_rows);
    }

    /// Handle text overwrite - punch holes in images
    pub fn overwrite_cells(&mut self, x: u16, y: u16, width: u16) {
        // Note: WezTerm maps images to cells, so overwrites create "holes"
        // This is a known limitation vs Kitty which keeps images separate
    }

    /// Garbage collect unused image data
    pub fn gc(&mut self) {
        let used_ids: HashSet<u64> = self.placements.iter()
            .map(|p| p.image_id)
            .collect();

        self.image_data.retain(|id, _| used_ids.contains(id));
    }
}
```

## imgcat Utility

Provide a command-line tool for testing:

```rust
// In scarab-tools/src/bin/imgcat.rs
fn main() {
    let args: Vec<String> = std::env::args().collect();
    let filename = args.get(1).expect("Usage: imgcat <file>");

    let data = std::fs::read(filename).expect("Failed to read file");
    let b64 = base64::encode(&data);

    // iTerm2 protocol
    print!("\x1b]1337;File=inline=1:{}\x07", b64);
}
```

## Implementation Plan

### Phase 1: iTerm2 Protocol (Week 1-2)

1. Parse OSC 1337 sequences in VTE
2. Extract image data and dimensions
3. Store images in shared memory
4. Basic Bevy sprite rendering

### Phase 2: Image Caching (Week 2)

1. Implement `ImageCache` resource
2. Texture atlas for small images
3. LRU eviction for memory management
4. Handle image lifecycle (scroll, delete)

### Phase 3: Kitty Protocol (Week 3)

1. Parse APC sequences
2. Handle chunked transmission
3. Support image IDs and placements
4. Unicode placeholder support (optional)

### Phase 4: Sixel (Week 4)

1. Implement sixel decoder
2. Palette management
3. Convert to RGBA for rendering
4. Basic sixel support

### Phase 5: Polish (Week 4-5)

1. imgcat utility
2. Performance optimization
3. Memory limits and garbage collection
4. Documentation

## Testing Strategy

### Unit Tests

```rust
#[test]
fn test_iterm2_parse() {
    let seq = b"File=name=dGVzdC5wbmc=;width=10;height=auto;inline=1:iVBORw0KGgo...";
    let image = parse_iterm2_image(seq).unwrap();

    assert_eq!(image.filename, Some("test.png".to_string()));
    assert_eq!(image.width, ImageSize::Cells(10));
    assert_eq!(image.height, ImageSize::Auto);
    assert!(image.inline);
}

#[test]
fn test_sixel_decoder() {
    let sixel = b"#0;2;0;0;0#1;2;100;100;100#1~~#0!10~";
    let mut decoder = SixelDecoder::new();
    decoder.decode(sixel);
    let image = decoder.finish().unwrap();

    assert!(image.width > 0);
    assert!(image.height > 0);
}
```

### Integration Tests

```bash
# Test with imgcat
cargo run --bin imgcat -- test.png

# Test with various tools
curl -sL https://iterm2.com/utilities/imgcat | bash -s -- test.png
viu test.png
```

## Success Criteria

- [ ] `imgcat` displays PNG images inline
- [ ] Images scroll with terminal content
- [ ] Images are deleted when overwritten
- [ ] Memory usage stays bounded (LRU eviction)
- [ ] At least iTerm2 protocol fully works
- [ ] Kitty basic support (stretch goal)
- [ ] Sixel basic support (stretch goal)
