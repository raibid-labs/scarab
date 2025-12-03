# Sixel Graphics Quick Reference

## For Developers

### Parsing Sixel Sequences

```rust
use scarab_daemon::images::{parse_sixel_dcs, SixelData};

// Parse a Sixel DCS sequence
let dcs_data = b"1;1q#1;2;100;0;0#1~~~";
match parse_sixel_dcs(dcs_data) {
    Some(sixel) => {
        println!("Image: {}x{} pixels", sixel.width, sixel.height);
        // sixel.pixels is Vec<u8> of RGBA data
        // Length: width * height * 4 bytes
    }
    None => println!("Invalid Sixel sequence"),
}
```

### Generating Sixel Sequences

```rust
// Basic structure
"\x1bP{params}q{sixel_data}\x1b\\"

// Example: Red square
"\x1bP1;1q#1;2;100;0;0#1~~~\x1b\\"
```

### Sixel Command Cheat Sheet

| Command | Format | Description | Example |
|---------|--------|-------------|---------|
| **DCS Start** | `ESC P` | Begin Sixel sequence | `\x1bP` |
| **Parameters** | `P1;P2;P3q` | Aspect ratio, grid size | `1;1q` |
| **Define Color** | `#Pc;Pu;Px;Py;Pz` | Set color register | `#5;2;100;0;0` (red) |
| **Select Color** | `#Pc` | Choose active color | `#5` |
| **Draw Sixel** | `?` to `~` | 6 vertical pixels | `~` (all 6 bits) |
| **Repeat** | `!count char` | Repeat character | `!10~` |
| **Carriage Return** | `$` | Move to line start | `$` |
| **Line Feed** | `-` | Down 6 pixels | `-` |
| **Raster Attrs** | `"Pan;Pad;Ph;Pv` | Pixel dimensions | `"1;1;100;50` |
| **End** | `ST` | Terminate sequence | `\x1b\\` |

### Color Formats

#### RGB (Pu=2)
```
#<color_num>;2;<red>;<green>;<blue>
```
- Values: 0-100 (converted to 0-255)
- Example: `#1;2;100;0;0` = bright red

#### HLS (Pu=1)
```
#<color_num>;1;<hue>;<lightness>;<saturation>
```
- Hue: 0-360 degrees
- Lightness: 0-100
- Saturation: 0-100
- Example: `#1;1;0;50;100` = pure red

### Sixel Character Encoding

Each character represents 6 vertical pixels (bits 0-5):

```
Character = 0x3F + bit_pattern

Bit 0 (LSB) = top pixel
Bit 1       = 2nd pixel
Bit 2       = 3rd pixel
Bit 3       = 4th pixel
Bit 4       = 5th pixel
Bit 5 (MSB) = bottom pixel
```

**Examples:**
- `?` (0x3F) = 0b000000 = no pixels
- `@` (0x40) = 0b000001 = top pixel only
- `O` (0x4F) = 0b010000 = 5th pixel only
- `~` (0x7E) = 0b111111 = all 6 pixels

### Common Patterns

#### Solid Line
```rust
// Horizontal line (10 sixels wide, all pixels set)
"!10~"
```

#### Vertical Stripe
```rust
// Blue and red alternating columns
"#1;2;0;0;100#2;2;100;0;0#1~#2~#1~#2~"
```

#### Two-Row Pattern
```rust
// Red on top, blue below
"#1;2;100;0;0#2;2;0;0;100#1~~~-#2~~~"
```

#### Transparent Background
```rust
// Use P1=0 or P1=2 for transparent background
"\x1bP0;1q#1~\x1b\\"
```

## For Terminal Users

### Testing Sixel Support

```bash
# Run the test script
./test_sixel.sh

# Or generate a simple test
printf '\033P1;1q#1;2;100;0;0#1~~~\033\\'
```

### Tools That Generate Sixel

- **ImageMagick:** `convert image.png sixel:-`
- **libsixel:** `img2sixel image.png`
- **lsix:** `lsix` (ls with thumbnails)
- **ffmpeg:** `ffmpeg -i video.mp4 -pix_fmt rgb24 -f sixel -`

### Example: Display Image

```bash
# Convert PNG to Sixel
convert image.png -resize 800x600 sixel:- | cat

# Or with img2sixel
img2sixel -w 800 image.png
```

## Debugging

### Enable Logging

```bash
RUST_LOG=scarab_daemon::vte=debug cargo run
```

Look for:
- "Sixel DCS sequence started"
- "Sixel DCS sequence complete"
- "Parsed Sixel image: WxH pixels"

### Common Issues

**1. No image appears**
- Check DCS sequence has correct format: `ESC P ... q ... ST`
- Verify color is defined before use
- Ensure sixel characters are in range 0x3F-0x7E

**2. Wrong colors**
- RGB values are 0-100, not 0-255
- HLS hue is 0-360, lightness/saturation 0-100
- Color must be defined before selection

**3. Image truncated**
- Check maximum dimensions (4096x4096)
- Verify sequence terminator (ST) is present

**4. Performance issues**
- Large images (>1000x1000) may be slow
- Consider reducing resolution before conversion

## Protocol Flow

```
Application
    ↓ (generates)
Sixel Sequence (ESC P ... q ... ST)
    ↓ (sent to)
Terminal PTY
    ↓ (parsed by)
VTE Parser
    ↓ (callbacks)
hook() → put()* → unhook()
    ↓ (calls)
parse_sixel_dcs()
    ↓ (returns)
SixelData (RGBA pixels)
    ↓ (converted to)
ImageData
    ↓ (placed in)
Terminal Grid
    ↓ (shared via)
SharedImageBuffer
    ↓ (rendered by)
Bevy Client (GPU)
```

## Performance Tips

### For Generators:
1. Use repeat commands (`!count char`) for solid areas
2. Limit color palette to commonly-used colors
3. Avoid excessive dimensions (800x600 is usually enough)
4. Consider compression before Sixel conversion

### For Terminal:
1. Images are parsed on-demand (lazy)
2. RGBA conversion happens once at parse completion
3. Memory usage: width * height * 4 bytes
4. Maximum 64 images per pane (auto-eviction)

## Additional Resources

- Full implementation docs: `SIXEL_IMPLEMENTATION.md`
- Implementation summary: `SIXEL_SUMMARY.md`
- Test script: `test_sixel.sh`
- Source code: `crates/scarab-daemon/src/images/sixel.rs`

## Support

For issues or questions:
1. Check logs with `RUST_LOG=debug`
2. Verify sequence format with test script
3. Review unit tests for examples
4. Consult DEC VT340 documentation

---

**Last Updated:** 2025-12-03
**Scarab Version:** 0.1.0-alpha.15
