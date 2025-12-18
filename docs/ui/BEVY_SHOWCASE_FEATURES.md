# Bevy Showcase Features

This document describes the advanced UI features that showcase the power of Scarab's Bevy-based rendering engine.

## Overview

Scarab leverages Bevy 0.15's game engine capabilities to provide terminal features that would be impossible or very difficult in traditional terminal emulators:

1. **Tab Animations** - Smooth transitions between tabs
2. **Enhanced Glow Effects** - Dynamic glowing borders with mode colors and pulse effects
3. **Dashboard Panes** - Data visualization widgets instead of terminal output

## Tab Animations

Location: `crates/scarab-client/src/ui/tab_animations.rs`

### Features

- **Slide transitions** when switching tabs
- **Fade in/out** for new/closing tabs
- **Hover effects** with subtle scaling
- Multiple easing functions (cubic, quad, sine)

### Components

```rust
// Tab switching animation
TabTransition {
    from_index: 0,
    to_index: 1,
    duration: 0.25,
    easing: TabEasingFunction::EaseOutCubic,
}

// Fade animation for new tabs
TabFade::fade_in(0.2)

// Hover state
TabHover {
    is_hovered: true,
    progress: 0.8,
    speed: 5.0,
}
```

### Configuration

```rust
TabAnimationConfig {
    switch_duration: 0.25,  // seconds
    fade_duration: 0.2,     // seconds
    enabled: true,
    hover_enabled: true,
}
```

## Enhanced Glow Effects

Location: `crates/scarab-client/src/shaders/glow.rs`

### Features

- **Static glow** - Constant intensity borders
- **Pulse glow** - Animated pulsing for notifications
- **Mode glow** - Color changes based on terminal mode
- **Breathing glow** - Slow fade in/out

### Glow Types

```rust
pub enum GlowType {
    Static,                    // Constant intensity
    Pulse { speed: u32 },     // Pulsing (for notifications)
    Breathing { speed: u32 }, // Slow breathing
    Mode,                      // Color based on mode
}
```

### Mode Colors

Different colors for different terminal modes:

- **Normal**: Slime green (#a8df5a)
- **Copy**: Cyan (#8be9fd)
- **Search**: Gold (#ffd700)
- **Window**: Purple (#bd93f9)
- **Font**: Red (#ff5555)
- **Pick**: Green (#4caf50)
- **Hint**: Orange (#ff9800)

### Usage

```rust
// Static glow for active pane
let glow = create_static_glow(
    Vec3::new(0.545, 0.914, 0.992), // Cyan
    0.7,  // intensity
    6.0   // radius
);

// Pulsing glow for notification
let (glow, pulse) = create_pulse_glow(
    Vec3::new(1.0, 0.596, 0.0), // Orange
    2.0,  // 2 pulses per second
    8.0   // radius
);

// Mode indicator glow (color changes with mode)
let glow = create_mode_glow(10.0);
```

### Shader Details

The glow shader (`assets/shaders/glow.wgsl`) uses:

1. **Edge detection** via alpha channel sampling
2. **Spiral search** pattern to find nearest edges
3. **Distance-based falloff** with configurable power curve
4. **Color compositing** with alpha blending

## Dashboard Panes

Location: `crates/scarab-client/src/ui/dashboard.rs`

### Features

A special pane type that renders data visualizations instead of terminal output.

### Widget Types

```rust
pub enum DashboardWidget {
    // Time series data
    LineChart {
        label: String,
        data: Vec<f32>,
        max_points: usize,
    },

    // Bar chart with labels
    BarChart {
        label: String,
        data: Vec<(String, f32)>,
    },

    // Single value gauge
    Gauge {
        label: String,
        value: f32,
        max: f32,
        show_percentage: bool,
    },

    // Text display
    Text {
        content: String,
        style: TextDisplayStyle,
    },

    // Data table
    Table {
        headers: Vec<String>,
        rows: Vec<Vec<String>>,
    },
}
```

### Layout Modes

```rust
pub enum DashboardLayout {
    Vertical,              // Stack widgets vertically
    Horizontal,            // Arrange horizontally
    Grid { columns: usize }, // Grid layout
    Custom,                // Custom positioning
}
```

### Example: System Monitor Dashboard

```rust
let dashboard = create_system_monitor_dashboard();

// Returns a dashboard with:
// - CPU usage line chart (60 data points)
// - Memory gauge (percentage)
// - Process table (PID, Name, CPU%)
```

### Creating Custom Dashboards

```rust
let dashboard = DashboardState::new("my_dashboard")
    .with_refresh_rate(Duration::from_secs(2))
    .with_layout(DashboardLayout::Vertical)
    .with_widgets(vec![
        DashboardWidget::LineChart {
            id: "metric".to_string(),
            label: "Requests/sec".to_string(),
            data: vec![],
            color: Color::srgb(0.3, 0.8, 0.3),
            max_points: 100,
            min_value: Some(0.0),
            max_value: Some(1000.0),
        },
        DashboardWidget::Gauge {
            id: "error_rate".to_string(),
            label: "Error Rate".to_string(),
            value: 2.5,
            max: 10.0,
            color: Color::srgb(0.8, 0.3, 0.3),
            show_percentage: true,
        },
    ]);
```

### Updating Dashboard Data

Dashboards automatically refresh at their configured rate. You can also trigger manual updates:

```rust
// Send update event
events.send(DashboardUpdateEvent {
    dashboard_id: "my_dashboard".to_string(),
});

// Update widget data in response
widget.push_data_point(new_value);  // For line charts
widget.update_value(new_value);     // For gauges
```

## Integration

All three features are automatically enabled when using `AdvancedUIPlugin`:

```rust
app.add_plugins(AdvancedUIPlugin);
```

Individual plugins can also be used:

```rust
app.add_plugins((
    TabAnimationsPlugin,
    DashboardPlugin,
));
```

## Configuration

### Via Config File

```toml
[effects]
# Glow effects
overlay_glow_enabled = true
overlay_glow_radius = 6.0
overlay_glow_color = "#a8df5a"
overlay_glow_intensity = 0.7

[ui]
# Animations
animations_enabled = true
tab_animation_duration = 0.25
hover_effects_enabled = true

# Dashboards
dashboard_enabled = true
```

### Via Code

```rust
// Tab animations
let config = TabAnimationConfig {
    switch_duration: 0.3,
    fade_duration: 0.2,
    enabled: true,
    hover_enabled: true,
};

// Glow colors
let colors = ModeGlowColors {
    normal: Vec3::new(0.659, 0.875, 0.353),
    copy: Vec3::new(0.545, 0.914, 0.992),
    // ... other modes
};
```

## Performance

All features are designed for high performance:

- **Tab animations** use simple transforms (GPU accelerated)
- **Glow shader** runs on GPU with configurable quality
- **Dashboards** only re-render when data changes
- **Automatic cleanup** of finished animations

### Low Power Mode

Disable effects to save GPU resources:

```toml
[effects]
low_power_mode = true
```

This disables all post-processing effects regardless of individual settings.

## Future Enhancements

Planned improvements for future sprints:

1. **Particle effects** for command execution
2. **Smooth scroll** animations
3. **More widget types** (sparklines, heat maps)
4. **3D depth effects** (experimental)
5. **Custom easing curves** using Bevy's Curve API
6. **Spring physics** for bouncy animations

## Testing

Run tests with:

```bash
cargo test -p scarab-client tab_animations
cargo test -p scarab-client dashboard
cargo test -p scarab-client glow
```

## Examples

See example dashboards in `examples/`:

- `system_monitor.rs` - System resource dashboard
- `git_status.rs` - Git repository dashboard
- `build_progress.rs` - Build system visualization

## References

- [Bevy 0.15 Animation](https://bevyengine.org/learn/book/animation/)
- [Bevy UI System](https://bevyengine.org/learn/book/ui/)
- [Bevy Shaders](https://bevyengine.org/learn/book/shaders/)
- [UI Implementation Plan](../UI_IMPLEMENTATION_PLAN.md)
