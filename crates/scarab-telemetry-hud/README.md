# Scarab Telemetry HUD

Performance metrics HUD overlay for the Scarab terminal emulator.

## Features

- **Real-time FPS counter**: Displays current frames per second
- **Frame time statistics**: Shows current, average, minimum, and maximum frame times
- **Configurable position**: Place the HUD in any corner of the screen
- **Toggle visibility**: Press F12 to show/hide the HUD
- **Low overhead**: Uses lock-free circular buffers and efficient rendering

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

## Controls

- **F12**: Toggle HUD visibility

## Metrics Displayed

The HUD displays the following performance metrics:

- **FPS**: Current frames per second
- **Frame Time**: Current frame rendering time in milliseconds
- **Avg**: Average frame time over the configured window
- **Min**: Minimum frame time in the window
- **Max**: Maximum frame time in the window
- **Frames**: Total number of frames rendered
- **Uptime**: Total elapsed time since application start

## Architecture

### Components

- **`TelemetryHudPlugin`**: Main plugin struct, configures and registers systems
- **`PerformanceMetrics`**: Resource that tracks frame timing data
- **`HudState`**: Resource controlling HUD visibility and position
- **`HudContainer`**: Component marking the HUD UI entity
- **`HudText`**: Component marking text elements within the HUD

### Systems

1. **`update_metrics`**: Collects frame timing data every frame
2. **`toggle_hud`**: Handles F12 key press to toggle visibility
3. **`render_hud`**: Creates/updates HUD UI when visible

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

## Integration with Scarab Client

To integrate with the Scarab client, add the plugin in the main app setup:

```rust
// In scarab-client/src/main.rs
use scarab_telemetry_hud::TelemetryHudPlugin;

app.add_plugins(TelemetryHudPlugin::default());
```

The HUD will automatically integrate with Scarab's rendering pipeline and respond to F12 key presses.

## Future Enhancements

Potential future improvements:

- Frame time graph visualization using `HudGraph` component
- Memory usage tracking
- Entity count metrics
- System execution time breakdown
- GPU metrics (if available)
- Customizable color themes
- Export metrics to file/logging system
- Integration with external profiling tools (Tracy, puffin)

## License

Licensed under MIT OR Apache-2.0, same as the Scarab project.
