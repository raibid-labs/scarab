# Sixel Graphics Protocol Implementation

This document describes the Sixel graphics protocol implementation in Scarab terminal emulator.

## Overview

Sixel is a bitmap graphics format originally developed by DEC for their VT240 and later terminals. It enables inline graphics rendering in terminal applications.

**Implementation Status:** ✅ Complete

**Location:** `/home/beengud/raibid-labs/scarab/crates/scarab-daemon/src/images/sixel.rs`

## Architecture

The Sixel implementation follows the same pattern as iTerm2 image protocol:

1. **Parser** (`sixel.rs`) - Decodes DCS sequences into RGBA pixels
2. **VTE Integration** (`vte.rs`) - Hooks into DCS sequence callbacks
3. **Image Placement** (`placement.rs`) - Reuses existing image infrastructure
4. **Format Detection** (`format.rs`) - Supports RGBA format for raw pixel data

## Protocol Specification

### DCS Sequence Format

```
ESC P <P1> ; <P2> ; <P3> q <sixel_data> ST
```

- `ESC P` = DCS (Device Control String) introducer (0x1B 0x50)
- `P1` = Aspect ratio numerator (or mode: 0/2 = transparent background, 1 = color)
- `P2` = Aspect ratio denominator
- `P3` = Horizontal grid size (ignored)
- `q` = Sixel mode indicator
- `<sixel_data>` = Encoded sixel graphics data
- `ST` = String Terminator (ESC \ = 0x1B 0x5C)

### Sixel Data Encoding

Each character from `0x3F` ('?') to `0x7E` ('~') represents **6 vertical pixels**:

- Character value - 0x3F = 6-bit pattern
- Bit 0 (LSB) = top pixel
- Bit 5 (MSB) = bottom pixel

**Example:**
- `?` (0x3F) = 000000 (no pixels)
- `~` (0x7E) = 111111 (all 6 pixels)
- `C` (0x43) = 000100 (only 3rd pixel from top)

### Control Commands

#### Color Definition
```
#<Pc>;<Pu>;<Px>;<Py>;<Pz>
```
- `Pc` = Color number (0-255)
- `Pu` = Color format (1 = HLS, 2 = RGB)
- `Px, Py, Pz` = Color components (0-100 for RGB, 0-360/0-100/0-100 for HLS)

**Example:**
```
#5;2;100;0;0  // Define color 5 as red (RGB: 100, 0, 0)
```

#### Color Selection
```
#<Pc>
```
- Selects color `Pc` for subsequent drawing

#### Repeat Command
```
!<count><char>
```
- Repeats `<char>` `<count>` times

**Example:**
```
!10~  // Draw 10 sixels with all pixels set
```

#### Cursor Movement
- `$` = Carriage return (move cursor to left edge of current sixel row)
- `-` = Line feed (move down 6 pixels to next sixel row)

#### Raster Attributes (Optional)
```
"<Pan>;<Pad>;<Ph>;<Pv>
```
- `Pan` = Pixel aspect ratio numerator
- `Pad` = Pixel aspect ratio denominator
- `Ph` = Horizontal pixel count
- `Pv` = Vertical pixel count

## Implementation Details

### Parser State Machine (`SixelParser`)

```rust
struct SixelParser {
    x: u32,              // Current column
    y: u32,              // Current sixel row (each row = 6 pixels)
    max_x: u32,          // Maximum X reached (determines final width)
    max_y: u32,          // Maximum Y reached (determines final height)
    current_color: u8,   // Active color register
    palette: SixelPalette, // 256-color palette
    pixels: Vec<u8>,     // Canvas (color indices)
    canvas_width: u32,   // Allocated width
    canvas_height: u32,  // Allocated height
}
```

### Color Palette

Default VT340 16-color palette:
- Colors 0-15: ANSI-like colors (black, blue, red, magenta, green, cyan, yellow, white + bright variants)
- Colors 16-255: Grayscale gradient (for undefined colors)

Custom colors can be defined using the `#Pc;Pu;Px;Py;Pz` command.

### Memory Management

- **Dynamic Canvas Expansion**: Canvas grows as needed to accommodate sixel data
- **Maximum Dimensions**: 4096x4096 pixels (prevents DoS via excessive memory allocation)
- **Initial Allocation**: 256x256 pixels
- **Indexed Storage**: Pixels stored as color indices during parsing, converted to RGBA at the end

### VTE Integration

The VTE parser callbacks are implemented to capture DCS sequences:

1. **`hook()`** - Called when DCS sequence starts (e.g., `ESC P ... q`)
   - Sets `in_dcs = true`
   - Stores parameters in `dcs_buffer`

2. **`put()`** - Called for each data byte
   - Accumulates bytes in `dcs_buffer`

3. **`unhook()`** - Called when DCS ends (at `ST`)
   - Parses complete sequence
   - Converts to RGBA pixels
   - Creates `ImageData` and places in terminal grid

### Image Placement

Sixel images are converted to `ImageData` format:
```rust
ImageData {
    data: sixel_data.pixels,  // RGBA bytes
    width: ImageSize::Pixels(width),
    height: ImageSize::Pixels(height),
    preserve_aspect_ratio: true,
    inline: true,
    do_not_move_cursor: false,
    filename: None,
}
```

The image is then added using the existing `add_image()` infrastructure, which handles:
- Image placement tracking
- Automatic eviction when image limit is reached
- Scrolling behavior
- Cursor movement

## Testing

### Unit Tests

Located in `/home/beengud/raibid-labs/scarab/crates/scarab-daemon/src/images/sixel.rs`:

```bash
cargo test -p scarab-daemon --lib images::sixel::tests
```

Test coverage:
- ✅ HLS to RGB color conversion
- ✅ Simple sixel parsing
- ✅ Color definition and selection
- ✅ Carriage return and line feed
- ✅ Repeat command
- ✅ Transparent background mode
- ✅ Empty sequences
- ✅ Invalid sequences
- ✅ Aspect ratio parameters
- ✅ Complex multi-row images

### Manual Testing

Use the provided test script:
```bash
./test_sixel.sh
```

This generates various Sixel sequences to verify:
- Basic drawing
- Color palette customization
- Multi-row rendering
- Repeat commands
- Cursor positioning

### Example Sixel Sequence

Generate a simple red square:
```bash
printf '\033P1;1q#1;2;100;0;0#1~~~\033\\'
```

Breakdown:
- `\033P` = DCS start
- `1;1q` = 1:1 aspect ratio, sixel mode
- `#1;2;100;0;0` = Define color 1 as red (RGB 100,0,0)
- `#1` = Select color 1
- `~~~` = Draw 3 sixels (each 6 pixels tall)
- `\033\\` = ST (end sequence)

## Performance Characteristics

- **Zero-Copy Parsing**: Sixel data is parsed in a single pass
- **Memory Efficiency**: Pixels stored as 8-bit indices during parsing, expanded to RGBA only at the end
- **Canvas Growth**: Amortized O(1) for typical usage patterns
- **DoS Protection**: Maximum dimensions enforced (4096x4096)

## Comparison with iTerm2 Protocol

| Feature | Sixel | iTerm2 |
|---------|-------|--------|
| Format | Binary bitmap (DCS) | Base64-encoded images (OSC) |
| Color depth | 256 colors (palette) | 24-bit RGB (full color) |
| Transparency | Optional background transparency | Full alpha channel |
| Compression | None (raw bitmap) | PNG/JPEG compression |
| Cursor control | Automatic advance | Configurable |
| Use case | Legacy compatibility, simple graphics | Photos, complex images |

## Known Limitations

1. **No scrolling region support**: Sixel images don't respect scrolling regions
2. **Fixed pixel aspect ratio**: Assumes square pixels (1:1)
3. **No animation**: Each Sixel sequence is a static image
4. **Memory overhead**: Large images use significant memory in indexed form

## Future Enhancements

- [ ] Scrolling region clipping
- [ ] Non-square pixel aspect ratios
- [ ] Performance optimization for large images
- [ ] Sixel animation support (via multiple frames)
- [ ] Integration with GPU rendering pipeline

## References

- [DEC STD 070 - Video Systems Reference Manual](https://vt100.net/docs/vt3xx-gp/chapter14.html)
- [Sixel Graphics Wikipedia](https://en.wikipedia.org/wiki/Sixel)
- [libsixel](https://github.com/saitoha/libsixel) - Reference implementation
- [VT340 Programmer Reference Manual](https://vt100.net/docs/vt340pro/)

## Integration with Scarab

Sixel graphics work seamlessly with Scarab's existing infrastructure:

1. **Daemon Side**: Parses Sixel DCS sequences and converts to RGBA
2. **Shared Memory**: RGBA pixel data would be shared via `SharedImageBuffer`
3. **Client Side**: Renders using Bevy's GPU pipeline (same as iTerm2 images)
4. **Plugin Support**: Fusabi plugins can generate Sixel sequences programmatically

## Author

Implemented by Claude (Anthropic) for the Scarab terminal emulator project.

## License

Same as Scarab project (check root LICENSE file).
