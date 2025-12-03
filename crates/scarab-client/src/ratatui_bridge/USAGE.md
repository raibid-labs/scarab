# Ratatui Bridge Usage Guide

## Command Palette Integration

The Command Palette is a reference implementation demonstrating the full Ratatui bridge capabilities.

### Quick Start

Add the plugins to your Bevy app:

```rust
use scarab_client::{RatatuiBridgePlugin, CommandPalettePlugin};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(RatatuiBridgePlugin)      // Core Ratatui bridge infrastructure
        .add_plugins(CommandPalettePlugin)     // Command palette widget
        .run();
}
```

### Keyboard Controls

- **Ctrl+Shift+P**: Toggle command palette visibility
- **Escape**: Close command palette
- **Up/Down Arrows**: Navigate commands
- **Type**: Filter commands by name or description
- **Enter**: Execute selected command
- **Backspace**: Delete filter characters

### Handling Selected Commands

Listen for `CommandSelected` events:

```rust
use scarab_client::CommandSelected;

fn handle_commands(
    mut events: EventReader<CommandSelected>,
) {
    for event in events.read() {
        match event.command_id.as_str() {
            "new_tab" => {
                // Handle new tab creation
                info!("Creating new tab");
            }
            "close_tab" => {
                // Handle tab closure
                info!("Closing current tab");
            }
            "settings" => {
                // Open settings
                info!("Opening settings");
            }
            _ => {
                warn!("Unknown command: {}", event.command_id);
            }
        }
    }
}

// Add to your app:
app.add_systems(Update, handle_commands);
```

### Customizing Commands

Modify the default commands by updating `CommandPaletteState`:

```rust
use scarab_client::{CommandPaletteState, PaletteCommand};

fn customize_commands(mut state: ResMut<CommandPaletteState>) {
    // Add a custom command
    state.commands.push(PaletteCommand {
        id: "custom_action".into(),
        label: "My Custom Action".into(),
        description: Some("Does something amazing".into()),
        shortcut: Some("Ctrl+Shift+A".into()),
    });

    // Refresh the filtered list
    state.update_filter();
}

// Run at startup:
app.add_systems(Startup, customize_commands);
```

## Creating Custom Ratatui Widgets

The command palette demonstrates the full pattern for custom widgets:

### 1. Define Your Widget Component

```rust
use bevy::prelude::*;
use scarab_client::ratatui_bridge::{RatatuiSurface, SurfaceBuffers};

#[derive(Component)]
pub struct MyCustomWidget;

/// Widget state resource
#[derive(Resource)]
pub struct MyWidgetState {
    pub visible: bool,
    pub content: String,
}
```

### 2. Spawn Surface Entity

```rust
fn spawn_widget(mut commands: Commands) {
    commands.spawn((
        MyCustomWidget,
        RatatuiSurface::new(10, 5, 40, 10)  // x, y, width, height (in cells)
            .with_z_index(150.0),            // Higher z = on top
    ));
}
```

### 3. Render Widget System

```rust
use ratatui::widgets::{Block, Borders, Paragraph};

fn render_my_widget(
    state: Res<MyWidgetState>,
    mut buffers: ResMut<SurfaceBuffers>,
    mut surfaces: Query<(Entity, &mut RatatuiSurface), With<MyCustomWidget>>,
) {
    let Ok((entity, mut surface)) = surfaces.get_single_mut() else {
        return;
    };

    surface.visible = state.visible;
    if !state.visible { return; }

    surface.mark_dirty();

    let buffer = buffers.get_or_create(entity, surface.width, surface.height);
    buffer.reset();

    // Render Ratatui widget
    let widget = Paragraph::new(state.content.as_str())
        .block(Block::default()
            .title("My Widget")
            .borders(Borders::ALL));

    widget.render(surface.rect(), buffer);
}
```

### 4. Handle Input

```rust
use scarab_client::ratatui_bridge::{SurfaceInputEvent, SurfaceFocus};
use ratatui::crossterm::event::{Event, KeyCode};

fn handle_widget_input(
    mut state: ResMut<MyWidgetState>,
    mut events: EventReader<SurfaceInputEvent>,
    widget_query: Query<Entity, With<MyCustomWidget>>,
) {
    let Ok(widget_entity) = widget_query.get_single() else {
        return;
    };

    for event in events.read() {
        if event.surface != widget_entity { continue; }

        if let Event::Key(key) = &event.event {
            match key.code {
                KeyCode::Char(c) => {
                    state.content.push(c);
                }
                KeyCode::Backspace => {
                    state.content.pop();
                }
                _ => {}
            }
        }
    }
}
```

### 5. Toggle Visibility

```rust
fn toggle_widget(
    keys: Res<ButtonInput<KeyCode>>,
    mut state: ResMut<MyWidgetState>,
    mut focus: ResMut<SurfaceFocus>,
    widget_query: Query<Entity, With<MyCustomWidget>>,
) {
    if keys.just_pressed(KeyCode::F1) {
        state.visible = !state.visible;

        if let Ok(entity) = widget_query.get_single() {
            if state.visible {
                focus.push(entity);  // Grab input focus
            } else {
                focus.remove(entity);  // Release focus
            }
        }
    }
}
```

### 6. Create Plugin

```rust
pub struct MyWidgetPlugin;

impl Plugin for MyWidgetPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<MyWidgetState>()
            .add_systems(Startup, spawn_widget)
            .add_systems(Update, (
                toggle_widget,
                handle_widget_input,
                render_my_widget,
            ).chain());
    }
}
```

## Architecture Notes

### Surface Lifecycle

1. **Spawn**: Create entity with `RatatuiSurface` component
2. **Render**: Widget systems write to `SurfaceBuffers`
3. **Convert**: Bridge automatically converts buffers to Bevy meshes
4. **Input**: Focused surface receives keyboard/mouse events
5. **Cleanup**: Buffers cleaned up when surface despawns

### Focus Management

- Focus stack maintains input routing priority
- Top of stack receives all input events
- Mouse clicks automatically update focus
- Manual focus control via `SurfaceFocus` resource

### Performance Tips

1. **Mark dirty only when needed**: Use `surface.mark_dirty()` only on state changes
2. **Hide when invisible**: Set `surface.visible = false` to skip rendering
3. **Reuse buffers**: Buffers are cached per entity
4. **Batch updates**: Group state changes before re-rendering

### Coordinate Systems

- **Grid coordinates**: Terminal cells (0,0 = top-left)
- **Screen coordinates**: Pixels (converted automatically)
- **Z-index**: Float for layering (higher = on top)

### Common Patterns

**Modal Dialog**:
- High z-index (200+)
- Centered position
- Grab focus on show
- Release focus on hide

**Status Bar**:
- Low z-index (50-99)
- Bottom or top edge
- Always visible
- No focus needed

**Overlay**:
- Medium z-index (100-199)
- Variable position
- Toggle visibility
- Conditional focus

## Example: Simple Notification Widget

```rust
use bevy::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph};
use scarab_client::ratatui_bridge::*;

#[derive(Component)]
struct NotificationWidget;

#[derive(Resource)]
struct NotificationState {
    message: String,
    visible: bool,
}

impl Default for NotificationState {
    fn default() -> Self {
        Self {
            message: String::new(),
            visible: false,
        }
    }
}

fn spawn_notification(mut commands: Commands) {
    commands.spawn((
        NotificationWidget,
        RatatuiSurface::new(150, 1, 45, 3)  // Top-right corner
            .with_z_index(250.0)
            .hidden(),
    ));
}

fn render_notification(
    state: Res<NotificationState>,
    mut buffers: ResMut<SurfaceBuffers>,
    mut surfaces: Query<(Entity, &mut RatatuiSurface), With<NotificationWidget>>,
) {
    let Ok((entity, mut surface)) = surfaces.get_single_mut() else { return };

    surface.visible = state.visible;
    if !state.visible { return; }

    if state.is_changed() {
        surface.mark_dirty();
    }

    let buffer = buffers.get_or_create(entity, surface.width, surface.height);
    buffer.reset();

    let widget = Paragraph::new(state.message.as_str())
        .block(Block::default()
            .borders(Borders::ALL)
            .title(" Notification "));

    widget.render(surface.rect(), buffer);
}

pub struct NotificationPlugin;

impl Plugin for NotificationPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<NotificationState>()
            .add_systems(Startup, spawn_notification)
            .add_systems(Update, render_notification);
    }
}
```

## Testing

The command palette includes comprehensive tests demonstrating:

- State management
- Filtering logic
- Navigation bounds checking
- Visibility toggling
- Selection handling

Run tests with:

```bash
cargo test -p scarab-client ratatui_bridge::command_palette
```
