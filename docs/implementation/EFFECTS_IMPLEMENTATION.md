# Post-Processing Shader Effects Implementation

## Overview

Complete implementation of GitHub Issue #35: Post-process shader effects for the Scarab terminal emulator.

## Architecture

### Components

1. **WGSL Shaders** (`/home/beengud/raibid-labs/scarab/assets/shaders/`)
   - `blur.wgsl` - Two-pass Gaussian blur with separable kernel
   - `glow.wgsl` - Edge-based glow effect with radial falloff

2. **Rust Modules** (`/home/beengud/raibid-labs/scarab/crates/scarab-client/src/shaders/`)
   - `mod.rs` - Module exports
   - `blur.rs` - Blur shader integration and settings
   - `glow.rs` - Glow shader integration and settings
   - `plugin.rs` - Bevy plugin for effects management

3. **Configuration** (`/home/beengud/raibid-labs/scarab/crates/scarab-config/src/config.rs`)
   - `EffectsConfig` struct with comprehensive settings
   - Helper methods for rendering decisions
   - TOML serialization support

## Features

### Blur Shader
- **Two-pass Gaussian blur** for optimal performance
- **Configurable radius** (2.0 to 8.0+ pixels)
- **Intensity control** (0.0 to 1.0)
- **Pre-computed kernel weights** for 5-tap and 9-tap modes
- **Automatic kernel selection** based on radius

### Glow Shader
- **Edge-based detection** using alpha channel analysis
- **Radial distance falloff** for smooth glow
- **Configurable color** via hex color codes
- **Intensity control** (0.0 to 1.0)
- **Spiral search pattern** for efficient edge finding

### Configuration System
All effects are configurable via TOML:

```toml
[effects]
overlay_blur_enabled = true
overlay_blur_radius = 4.0
overlay_blur_intensity = 0.8
overlay_glow_enabled = true
overlay_glow_radius = 6.0
overlay_glow_color = "#8be9fd"
overlay_glow_intensity = 0.7
low_power_mode = false
```

### Low-Power Mode
When enabled, **all post-processing effects are disabled** to conserve GPU resources and battery life.

## Integration

### Main App Integration
The effects plugin is integrated into `main.rs`:

```rust
use scarab_client::ScarabEffectsPlugin;

app.add_plugins(ScarabEffectsPlugin) // Add post-processing effects
```

### Automatic Overlay Detection
The plugin automatically detects overlays and applies effects:
- Command Palette
- Search Overlay
- Remote Overlays (plugin UI)

## Testing

### Configuration Tests
7 comprehensive tests covering:
- Default configuration
- Effect enable/disable logic
- Blur rendering conditions
- Glow rendering conditions
- Color parsing (hex with/without #)
- Serialization/deserialization
- Integration with ScarabConfig

All tests pass:
```
running 7 tests
test config::effects_tests::test_effects_config_default ... ok
test config::effects_tests::test_should_render_blur ... ok
test config::effects_tests::test_should_render_glow ... ok
test config::effects_tests::test_effects_enabled_check ... ok
test config::effects_tests::test_glow_color_parsing ... ok
test config::effects_tests::test_effects_config_serialization ... ok
test config::effects_tests::test_effects_config_in_scarab_config ... ok
```

## Performance Considerations

### Blur Performance
- **Two-pass approach**: O(2*N) instead of O(N²)
- **Separable kernel**: Horizontal then vertical passes
- **Dual kernel sizes**: 5-tap for subtle, 9-tap for heavy blur
- **Intensity blending**: Can reduce blur strength without changing kernel

### Glow Performance
- **Spiral search pattern**: Reduces redundant sampling
- **Early exit**: Skips fully opaque pixels
- **Distance-based culling**: Only searches within glow radius
- **Edge threshold**: Configurable sensitivity

## API Reference

### EffectsConfig Methods

```rust
pub fn has_effects_enabled(&self) -> bool
```
Check if any effects are enabled (respecting low_power_mode).

```rust
pub fn should_render_blur(&self) -> bool
```
Determine if blur should be rendered this frame.

```rust
pub fn should_render_glow(&self) -> bool
```
Determine if glow should be rendered this frame.

```rust
pub fn glow_color_rgb(&self) -> (f32, f32, f32)
```
Parse hex color string to normalized RGB values (0.0-1.0).

### Bevy Components

```rust
#[derive(Component)]
pub struct BlurSettings {
    pub radius: f32,
    pub intensity: f32,
    pub enabled: bool,
}
```

```rust
#[derive(Component)]
pub struct GlowSettings {
    pub radius: f32,
    pub color: Vec3,
    pub intensity: f32,
    pub edge_threshold: f32,
    pub falloff_power: f32,
    pub enabled: bool,
}
```

## File Structure

```
scarab/
├── assets/
│   └── shaders/
│       ├── blur.wgsl          # Gaussian blur shader (WGSL)
│       └── glow.wgsl          # Glow border shader (WGSL)
├── crates/
│   ├── scarab-client/
│   │   └── src/
│   │       ├── shaders/
│   │       │   ├── mod.rs     # Module exports
│   │       │   ├── blur.rs    # Blur integration
│   │       │   ├── glow.rs    # Glow integration
│   │       │   └── plugin.rs  # Bevy plugin
│   │       ├── lib.rs         # Exports ScarabEffectsPlugin
│   │       └── main.rs        # Adds plugin to app
│   └── scarab-config/
│       └── src/
│           └── config.rs      # EffectsConfig definition + tests
└── EFFECTS_IMPLEMENTATION.md  # This file
```

## Configuration Examples

### Minimal Effects (Performance Mode)
```toml
[effects]
overlay_blur_enabled = true
overlay_blur_radius = 2.0
overlay_blur_intensity = 0.5
overlay_glow_enabled = false
```

### Maximum Visual Quality
```toml
[effects]
overlay_blur_enabled = true
overlay_blur_radius = 8.0
overlay_blur_intensity = 1.0
overlay_glow_enabled = true
overlay_glow_radius = 8.0
overlay_glow_color = "#8be9fd"
overlay_glow_intensity = 1.0
```

### Battery-Saving Mode
```toml
[effects]
low_power_mode = true
```

## Future Enhancements

Potential improvements for future iterations:

1. **Additional Shader Effects**
   - Chromatic aberration
   - Vignette
   - Bloom
   - Color grading

2. **Performance Optimizations**
   - Compute shader implementation for blur
   - Mipmap-based blur for large radii
   - Conditional rendering based on FPS

3. **Advanced Glow Features**
   - Multiple glow colors per overlay
   - Animated glow pulses
   - Directional glow (top/bottom/sides)

4. **Quality Presets**
   - Low/Medium/High/Ultra presets
   - Auto-adjustment based on GPU capabilities

## Notes

- Shaders use WGSL (WebGPU Shading Language), Bevy 0.15's native shader language
- All GPU operations are non-blocking to maintain 60+ FPS
- Effects are applied in post-processing, not affecting terminal text rendering
- Configuration hot-reloading is supported via bevy-fusabi integration
