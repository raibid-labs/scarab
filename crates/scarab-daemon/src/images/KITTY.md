# Kitty Graphics Protocol Implementation

This module implements the Kitty terminal graphics protocol for displaying images in the Scarab terminal emulator.

## Protocol Overview

The Kitty graphics protocol uses APC (Application Program Command) escape sequences to transmit and display images:

```
ESC _ G <key>=<value>,<key>=<value> ; <base64-payload> ESC \
```

## Key Parameters

### Action (a)
- `a=t` - Transmit image data (store for later display)
- `a=T` - Transmit and display immediately
- `a=p` - Display previously transmitted image (put)
- `a=d` - Delete image

### Format (f)
- `f=24` - RGB format (24-bit, 3 bytes per pixel)
- `f=32` - RGBA format (32-bit, 4 bytes per pixel)
- `f=100` - PNG format (compressed)

### Transmission (t)
- `t=d` - Direct transmission (base64 in command)
- `t=f` - File path
- `t=t` - Temporary file
- `t=s` - Shared memory

### Chunking (m)
- `m=0` - Final chunk (or single chunk)
- `m=1` - More chunks coming

### Identification
- `i=N` - Image ID (for referencing later)
- `p=N` - Placement ID (for multiple placements of same image)

### Dimensions
- `s=W` - Source width in pixels
- `v=H` - Source height in pixels
- `c=W` - Display width in terminal columns
- `r=H` - Display height in terminal rows

### Positioning
- `X=N` - X position on terminal grid (column)
- `Y=N` - Y position on terminal grid (row)
- `x=N` - X offset in pixels within source image
- `y=N` - Y offset in pixels within source image
- `z=N` - Z-index for stacking order

## Usage Examples

### Simple PNG Transmission

```rust
use scarab_daemon::images::{parse_kitty_graphics, KittyAction};

let sequence = b"a=T,f=100,i=1,c=10,r=5;iVBORw0KGgoAAAA...";
let cmd = parse_kitty_graphics(sequence).unwrap();

assert_eq!(cmd.action, KittyAction::TransmitAndDisplay);
assert_eq!(cmd.image_id, Some(1));
assert_eq!(cmd.display_columns, Some(10));
assert_eq!(cmd.display_rows, Some(5));
```

### Chunked Transfer

For large images, the protocol supports chunked transfer:

```rust
use scarab_daemon::images::{parse_kitty_graphics, ChunkedTransferState};

let mut state = ChunkedTransferState::new();

// First chunk (m=1 means more coming)
let chunk1 = b"a=t,f=100,i=1,m=1;aGVsbG8=";
let cmd = parse_kitty_graphics(chunk1).unwrap();
let result = state.add_chunk(cmd.image_id.unwrap(), cmd.payload, !cmd.more_chunks);
assert!(result.is_none()); // Not complete yet

// Final chunk (m=0)
let chunk2 = b"a=t,f=100,i=1,m=0;d29ybGQ=";
let cmd = parse_kitty_graphics(chunk2).unwrap();
let complete_data = state.add_chunk(cmd.image_id.unwrap(), cmd.payload, !cmd.more_chunks);
assert!(complete_data.is_some()); // Transfer complete!
```

### Display Existing Image

```rust
// Display previously transmitted image at specific position
let sequence = b"a=p,i=1,X=10,Y=5,c=20,r=10";
let cmd = parse_kitty_graphics(sequence).unwrap();

assert_eq!(cmd.action, KittyAction::Put);
assert_eq!(cmd.image_id, Some(1));
assert_eq!(cmd.grid_x, Some(10));
assert_eq!(cmd.grid_y, Some(5));
```

### RGB Format

```rust
// 2x2 RGB image (12 bytes total)
let rgb_data = base64::encode(&[
    255, 0, 0,    // Red pixel
    0, 255, 0,    // Green pixel
    0, 0, 255,    // Blue pixel
    255, 255, 0,  // Yellow pixel
]);
let sequence = format!("a=T,f=24,s=2,v=2;{}", rgb_data);
let cmd = parse_kitty_graphics(sequence.as_bytes()).unwrap();
```

## Integration with Scarab

The Kitty protocol integrates with Scarab's existing image infrastructure:

1. **VTE Parser**: Detects APC sequences with 'G' command
2. **Protocol Parser**: `parse_kitty_graphics()` parses the command
3. **Chunked State**: `ChunkedTransferState` accumulates chunks
4. **Format Conversion**: Raw RGB/RGBA converted to PNG via `convert_raw_to_png()`
5. **Placement Manager**: `ImagePlacementState` tracks displayed images
6. **Shared Memory**: Images written to SharedImageBuffer for client rendering

## Implementation Details

### Parsing

The parser handles:
- Multiple key=value pairs separated by commas or semicolons
- Optional base64 payload (last segment after semicolon)
- All standard Kitty protocol parameters
- Graceful handling of unknown parameters

### Chunked Transfers

The `ChunkedTransferState` manager:
- Accumulates chunks by image ID
- Returns `None` for incomplete transfers
- Returns `Some(complete_data)` when final chunk received
- Supports multiple concurrent chunked transfers
- Can be cleared to reset all pending transfers

### Format Conversion

Raw pixel formats (RGB/RGBA) are converted to PNG for storage:
- Validates pixel data size matches dimensions
- Uses the `png` crate for encoding
- Stores all images as PNG in SharedImageBuffer
- Client can decode PNG for rendering

### Error Handling

The parser is designed to be lenient:
- Unknown keys are logged and skipped
- Invalid values use defaults
- Malformed base64 returns `None`
- Missing required fields use protocol defaults

## Testing

Comprehensive tests cover:
- Simple transmit commands
- Transmit-and-display
- Chunked transfers (single and multiple)
- All format types (PNG, RGB, RGBA)
- Placement commands
- Delete commands
- Complex sequences with all parameters
- Invalid/malformed sequences

Run tests with:
```bash
cargo test -p scarab-daemon --lib images::kitty
```

Run the demo:
```bash
cargo run --example kitty_graphics_demo
```

## References

- [Kitty Graphics Protocol Specification](https://sw.kovidgoyal.net/kitty/graphics-protocol/)
- [Kitty Terminal](https://sw.kovidgoyal.net/kitty/)

## Future Enhancements

Potential improvements:
- [ ] File path transmission support (t=f)
- [ ] Shared memory transmission (t=s)
- [ ] Virtual placements (multiple placements of same image)
- [ ] Animation support (gap parameter)
- [ ] Compression detection and handling
- [ ] Image eviction policies for memory management
