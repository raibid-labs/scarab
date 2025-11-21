# Scarab Text Rendering - Testing Guide

**Date**: 2025-11-21
**Issue**: #2 - Text Rendering Engine

---

## Prerequisites

1. **Rust toolchain**: Latest stable (1.75+)
2. **System fonts**: JetBrains Mono, Fira Code, or similar monospace font
3. **Platform**: macOS, Linux, or Windows with X11/Wayland
4. **GPU**: Any GPU with OpenGL 3.3+ or Vulkan support

---

## Building

### Development Build
```bash
cd /Users/beengud/raibid-labs/scarab/scarab
cargo build -p scarab-client
```

### Release Build (Recommended for Performance Testing)
```bash
cd /Users/beengud/raibid-labs/scarab/scarab
cargo build -p scarab-client --release
```

---

## Running

### Step 1: Start the Daemon (if not already running)
```bash
cd /Users/beengud/raibid-labs/scarab/scarab
cargo run -p scarab-daemon
```

Expected output:
```
✓ Shared memory initialized: /scarab_shm_v1
✓ IPC socket listening: /tmp/scarab-daemon.sock
✓ Scarab daemon started
```

### Step 2: Start the Client
```bash
# In another terminal
cd /Users/beengud/raibid-labs/scarab/scarab
cargo run -p scarab-client --release
```

Expected output:
```
✓ Connected to shared memory at: /scarab_shm_v1
┌─────────────────────────────────────────┐
│   Scarab Terminal - Text Renderer      │
├─────────────────────────────────────────┤
│ Font: JetBrains Mono                    │
│ Size: 14.0pt                             │
│ Grid: 200x100 cells                      │
│ Target: 60+ FPS                          │
└─────────────────────────────────────────┘
✓ Text Renderer initialized
  Cell dimensions: 8.40x16.80 px
  Atlas size: 4096x4096 px (~64MB VRAM)
✓ Terminal grid entity spawned
✓ Ready to render!
```

---

## Interactive Controls

### Font Size
- **Ctrl + =** or **Ctrl + Numpad+**: Increase font size
- **Ctrl + -** or **Ctrl + Numpad-**: Decrease font size
- Range: 6pt - 72pt

### Debugging
- **F3**: Show glyph atlas statistics
- **F5**: Force full redraw
- **F6**: Clear atlas and rebuild

### Example F3 Output:
```
┌─────────────────────────────────────┐
│ Glyph Atlas Statistics              │
├─────────────────────────────────────┤
│ Cached glyphs:    127               │
│ Used height:  512/4096 px          │
│ Occupancy:  12.5%                 │
│ Memory:  64.00 MB                 │
└─────────────────────────────────────┘
```

---

## Performance Testing

### 1. FPS Monitoring (Console)
The client outputs FPS diagnostics to the console every second:

```
2025-11-21T12:34:56.789Z INFO bevy diagnostic: fps: 60.234 (avg 60.123)
2025-11-21T12:34:57.789Z INFO bevy diagnostic: frame_time: 16.234ms (avg 16.456ms)
```

### 2. Frame Time Analysis
Watch for:
- **Target**: 60+ FPS (16.67ms per frame)
- **P99 Frame Time**: Should be <50ms
- **Spikes**: Occasional spikes on font resize are acceptable

### 3. Memory Profiling
Use system tools to monitor:
- **GPU Memory**: Should be <100MB
- **CPU Memory**: Should be <50MB
- **Atlas Occupancy**: Check with F3 key

---

## Testing Scenarios

### 1. Basic Rendering
**Goal**: Verify glyphs appear correctly

1. Start daemon and client
2. Check for visual artifacts
3. Verify cursor position
4. Check text alignment

**Pass Criteria**:
- No visual tearing
- Glyphs are sharp and clear
- Colors match specification
- No flickering

### 2. Font Resizing
**Goal**: Verify dynamic font size changes

1. Press Ctrl+= multiple times
2. Press Ctrl+- to decrease
3. Press F3 to check atlas usage

**Pass Criteria**:
- Font size changes smoothly
- No crashes or hangs
- Atlas rebuilds within 100ms
- Glyph quality maintained

### 3. Full Grid Fill
**Goal**: Verify performance with 20,000 cells

1. Ensure daemon writes to all cells
2. Monitor FPS with console diagnostics
3. Press F3 to check atlas occupancy

**Pass Criteria**:
- 60+ FPS maintained
- Frame time <50ms P99
- Atlas occupancy <80%
- No memory leaks

### 4. Text Attributes
**Goal**: Verify bold, italic, underline rendering

Test cells with:
- Regular text
- Bold text (flag 0x01)
- Italic text (flag 0x02)
- Underline (flag 0x04)
- Strikethrough (flag 0x08)
- Combinations (bold + italic + underline)

**Pass Criteria**:
- Attributes render correctly
- No visual artifacts
- Performance not impacted

### 5. Color Support
**Goal**: Verify 24-bit RGB colors

Test with:
- White on black (default)
- Colored foreground (red, green, blue)
- Colored background
- Dim attribute (50% brightness)
- Reverse video (fg/bg swap)

**Pass Criteria**:
- Colors match RGBA values
- Alpha blending works
- No color banding

### 6. Unicode Support
**Goal**: Verify Unicode and combining characters

Test with:
- ASCII characters (0x20-0x7E)
- Latin-1 supplement (0xA0-0xFF)
- Box drawing characters (0x2500-0x257F)
- Emoji (if supported by font)
- Combining diacritics

**Pass Criteria**:
- All glyphs render
- Fallback fonts work
- No missing character boxes

---

## Stress Testing

### 1. Rapid Updates
**Goal**: Verify dirty region tracking

1. Daemon rapidly updates random cells
2. Monitor FPS stability
3. Check for rendering artifacts

**Pass Criteria**:
- 60 FPS maintained
- No visual artifacts
- Dirty tracking works correctly

### 2. Atlas Overflow
**Goal**: Verify behavior when atlas fills

1. Generate 10,000+ unique glyphs
2. Monitor atlas occupancy with F3
3. Check for warnings in console

**Pass Criteria**:
- Warning logged when atlas full
- No crash or panic
- Graceful degradation

### 3. Long Duration
**Goal**: Verify no memory leaks

1. Run client for 1+ hour
2. Monitor memory usage
3. Periodically check FPS

**Pass Criteria**:
- Memory usage stable
- FPS doesn't degrade
- No crashes

---

## Automated Testing

### Unit Tests (TODO)
```bash
cargo test -p scarab-client rendering::
```

### Integration Tests (TODO)
```bash
cargo test -p scarab-client --test rendering_integration
```

---

## Troubleshooting

### Issue: Client fails to start
**Error**: "Failed to open shared memory"
**Solution**: Start the daemon first

### Issue: Black screen / no glyphs
**Possible Causes**:
1. Font not found - check console for warnings
2. Atlas initialization failed - check GPU memory
3. Mesh not generated - check sequence_number updates

**Debug Steps**:
1. Press F5 to force redraw
2. Press F3 to check atlas stats
3. Check console for errors
4. Verify daemon is writing to shared memory

### Issue: Low FPS (<60)
**Possible Causes**:
1. Debug build - use `--release`
2. Dirty region not working - check sequence_number
3. GPU bottleneck - reduce grid size
4. VSync disabled - check PresentMode

**Debug Steps**:
1. Build with `--release`
2. Monitor frame time diagnostics
3. Check GPU utilization
4. Profile with Tracy (if available)

### Issue: Blurry text
**Possible Causes**:
1. Subpixel rendering disabled
2. Font hinting disabled
3. Window scaling incorrect

**Solutions**:
1. Enable font hinting in config
2. Check window DPI settings
3. Adjust font size

---

## Performance Benchmarks

### Expected Performance (macOS M1)

| Metric | Target | Measured |
|--------|--------|----------|
| FPS | 60+ | TBD |
| Frame Time (avg) | <16.67ms | TBD |
| Frame Time (P99) | <50ms | TBD |
| GPU Memory | <100MB | ~64MB |
| Atlas Glyph Count | 10,000+ | TBD |
| Font Resize Time | <100ms | TBD |

*Run benchmarks with daemon writing test data*

---

## Continuous Integration

### CI Pipeline (TODO)
```yaml
name: Text Rendering Tests
on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: actions-rs/toolchain@v1
      - name: Build
        run: cargo build -p scarab-client --release
      - name: Unit Tests
        run: cargo test -p scarab-client
      - name: Headless Rendering Test
        run: cargo test -p scarab-client --test headless_render
```

---

## Visual Regression Testing (TODO)

Use screenshot comparison to detect rendering regressions:

```bash
# Capture reference screenshots
cargo test --test visual_regression -- --capture

# Compare against reference
cargo test --test visual_regression -- --compare
```

---

## Next Steps

1. **Integration Testing**: Test with real daemon output
2. **Performance Profiling**: Measure actual FPS and frame times
3. **Visual Regression**: Add screenshot comparison tests
4. **Stress Testing**: Generate synthetic workloads
5. **Platform Testing**: Test on Linux (X11/Wayland) and Windows
6. **Accessibility**: Test with screen readers
7. **Color Accuracy**: Verify RGBA conversion correctness

---

**Status**: Ready for Integration Testing
**Blocked By**: Issue #1 (VTE Parser) for real terminal data
**Next**: Run daemon with test data and measure performance
