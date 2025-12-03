# Fonts

Configure font rendering and typography in Scarab.

## Quick Links

For complete customization documentation, see:
- [Customization Guide](../../../CUSTOMIZATION.md) - Complete customization guide

## Font Configuration

Edit `~/.config/scarab/config.toml`:

```toml
[appearance]
font_family = "Fira Code"
font_size = 14.0
line_height = 1.2
```

## Recommended Fonts

Terminal-optimized monospace fonts:

### Programming Fonts
- **Fira Code** - Ligature support, excellent readability
- **JetBrains Mono** - Clear characters, ligatures
- **Cascadia Code** - Microsoft's monospace font
- **Source Code Pro** - Adobe's monospace font
- **Monaco** - macOS classic
- **Consolas** - Windows classic

### Classic Fonts
- **Menlo** - macOS default
- **Courier New** - Universal classic
- **Liberation Mono** - Open source classic
- **DejaVu Sans Mono** - Unicode coverage

## Font Features

### Font Size

Set font size in points:

```toml
[appearance]
font_size = 14.0  # Points
```

### Line Height

Adjust line spacing:

```toml
[appearance]
line_height = 1.2  # Multiplier (1.0 = single spacing)
```

### Font Weight

Configure font weight:

```toml
[appearance]
font_weight = "normal"  # normal, bold, light, etc.
```

## Advanced Configuration

### Font Fallback

Specify fallback fonts for missing glyphs:

```toml
[appearance]
font_family = "Fira Code"
font_fallback = ["Noto Sans Mono", "DejaVu Sans Mono"]
```

### Per-Style Fonts

Use different fonts for different styles:

```toml
[appearance.fonts]
regular = "Fira Code"
bold = "Fira Code Bold"
italic = "Fira Code Italic"
bold_italic = "Fira Code Bold Italic"
```

### Ligature Support

Enable programming ligatures:

```toml
[appearance]
font_ligatures = true
```

Supported ligatures (with Fira Code):
- `->` → →
- `=>` → ⇒
- `!=` → ≠
- `==` → ==
- `>=` → ≥
- `<=` → ≤
- And many more...

## Font Rendering

### Antialiasing

Configure antialiasing:

```toml
[appearance.font_rendering]
antialias = "subpixel"  # none, grayscale, subpixel
hinting = "full"        # none, slight, medium, full
```

### Subpixel Rendering

Fine-tune subpixel rendering:

```toml
[appearance.font_rendering]
subpixel_order = "rgb"  # rgb, bgr, vrgb, vbgr
lcd_filter = "default"  # default, light, legacy, none
```

## Unicode Support

Scarab supports full Unicode rendering:

- Basic Multilingual Plane (BMP)
- Emoji (🎉 🚀 ⚡)
- Math symbols (∑ ∫ √)
- Box drawing (┌─┐)
- Powerline symbols ()

### Font for Symbols

Specify a font for special symbols:

```toml
[appearance]
symbol_font = "Noto Color Emoji"
```

## Performance

### Font Caching

Scarab uses cosmic-text for efficient font rendering:
- Texture atlas caching
- GPU-accelerated rendering
- Minimal CPU usage

### Large Font Sizes

For presentations or accessibility:

```toml
[appearance]
font_size = 24.0  # Large font
line_height = 1.3
```

## Troubleshooting

### Font Not Found

If a font isn't found, Scarab will:
1. Try fallback fonts
2. Fall back to system default
3. Log a warning

Check available fonts:
```bash
fc-list : family | sort | uniq
```

### Rendering Issues

For rendering issues:

1. Verify font is installed:
   ```bash
   fc-match "Font Name"
   ```

2. Clear font cache:
   ```bash
   fc-cache -f -v
   ```

3. Check Scarab logs for font warnings

## See Also

- [Customization](./customization.md) - General customization
- [Themes](./themes.md) - Color themes
- [Configuration Schema](../reference/config-schema.md) - Complete reference
