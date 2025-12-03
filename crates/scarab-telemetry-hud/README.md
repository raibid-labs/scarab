# Scarab Telemetry HUD

Real-time performance metrics overlay for the Scarab terminal emulator.

## Features

- **Performance Metrics**: FPS, frame time (current, average, min, max)
- **Cache Statistics**: Glyph cache, texture atlas usage
- **Memory Tracking**: Process memory, heap, GPU memory (Linux only)
- **Navigation Stats**: Active hints, focusable regions, overlay counts
- **Configurable Position**: Top-right, top-left, bottom-right, bottom-left
- **Toggle Hotkeys**: F12 or Ctrl+Shift+T
- **Low Overhead**: Lock-free atomics, efficient ECS queries

## Usage

Add the plugin to your Bevy app:

```rust
use bevy::prelude::*;
use scarab_telemetry_hud::TelemetryHudPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(TelemetryHudPlugin::default())
        .run();
}
```

### Configuration

The plugin can be configured with a builder pattern:

```rust
// Start with HUD visible
App::new()
    .add_plugins(TelemetryHudPlugin::default()
        .with_visibility(true))
    .run();

// Position in bottom-left corner
App::new()
    .add_plugins(TelemetryHudPlugin::default()
        .with_position(HudPosition::BottomLeft))
    .run();

// Custom averaging window (60 frames = 1 second at 60 FPS)
App::new()
    .add_plugins(TelemetryHudPlugin::default()
        .with_window_size(60))
    .run();

// Combine configurations
App::new()
    .add_plugins(TelemetryHudPlugin::default()
        .with_visibility(true)
        .with_position(HudPosition::TopLeft)
        .with_window_size(120))
    .run();
```

### HUD Positions

- `HudPosition::TopRight` (default)
- `HudPosition::TopLeft`
- `HudPosition::BottomRight`
- `HudPosition::BottomLeft`

## Scarab Integration

Use `ScarabTelemetryPlugin` for automatic configuration:

```rust
use scarab_client::ScarabTelemetryPlugin;

app.add_plugins(ScarabTelemetryPlugin);  // Reads from ScarabConfig
```

### Configuration (TOML)

Add to `~/.config/scarab/config.toml`:

```toml
[telemetry]
hud_enabled = true
hud_position = "top-right"
hud_hotkey = "Ctrl+Shift+T"
hud_show_memory = true
hud_show_cache = true
hud_show_hints = true
```

### Configuration (Fusabi)

Add to `~/.config/scarab/config.fsx`:

```fsharp
{
  telemetry = {
    hud_enabled = true
    hud_position = "top-right"
    hud_hotkey = "Ctrl+Shift+T"
    hud_show_memory = true
    hud_show_cache = true
    hud_show_hints = true
  }
}
```

## Controls

- **F12**: Toggle HUD visibility (legacy)
- **Ctrl+Shift+T**: Toggle HUD visibility (configurable)

## Metrics Displayed

### Performance
- **FPS**: Current frames per second
- **Frame Time**: Current frame time in milliseconds
- **Avg/Min/Max**: Statistical frame time data
- **Frames**: Total frames rendered
- **Uptime**: Total elapsed time

### Cache
- **Glyphs**: Number of cached glyphs
- **Hit Rate**: Glyph cache hit percentage
- **Atlases**: Number of texture atlases
- **Tex Mem**: Total texture memory usage

### Memory
- **Process**: Total process memory (RSS on Linux)
- **Heap**: Heap allocation size
- **GPU**: GPU memory usage (estimated)

### Navigation
- **Hints**: Active NavHint entities
- **Focusable**: FocusableRegion count
- **Overlays**: Visible HintOverlay entities

## Architecture

### Resources

- **`TelemetryHudPlugin`**: Main plugin, registers systems
- **`PerformanceMetrics`**: Frame timing data
- **`TelemetryData`**: Cache, memory, navigation stats
- **`HudState`**: Visibility and position control

### Systems

1. **`update_metrics`**: Frame timing collection
2. **`update_cache_stats`**: Cache metrics (stub for integration)
3. **`update_memory_stats`**: Memory sampling (Linux: procfs)
4. **`update_hint_stats`**: Navigation entity counting (via integration)
5. **`toggle_hud`**: F12 / Ctrl+Shift+T handler
6. **`render_hud`**: UI creation and updates

### Performance Considerations

- Metrics collection uses a bounded `VecDeque` (circular buffer) to prevent unbounded memory growth
- Statistics are computed only when a new sample is added
- HUD rendering only occurs when visible
- No locking primitives - all operations are lock-free
- Minimal allocations during steady-state operation

## Testing

Run the test suite:

```bash
cargo test -p scarab-telemetry-hud
```

All metrics collection and statistics computation are thoroughly tested.

## Extension Points

### Custom Cache Integration

```rust
use scarab_telemetry_hud::TelemetryData;

fn update_custom_cache(
    mut telemetry: ResMut<TelemetryData>,
    my_cache: Res<MyGlyphCache>,
) {
    telemetry.cache_stats.glyph_count = my_cache.len();
    telemetry.cache_stats.glyph_hit_rate = my_cache.hit_rate();
}
```

### Component Counting

```rust
use scarab_telemetry_hud::integration::update_nav_hint_counts;

app.add_systems(
    Update,
    update_nav_hint_counts::<NavHint, FocusableRegion, HintOverlay>
);
```

## Platform Support

- **Linux**: Full support (memory via `/proc/self/status`)
- **macOS/Windows**: Performance and navigation (memory stub)

## Future Enhancements

- Frame time graph visualization
- GPU memory tracking (platform-specific APIs)
- Customizable color themes
- Export metrics to file
- Integration with Tracy/puffin profilers

## License

Licensed under MIT OR Apache-2.0, same as the Scarab project.
