# Theme System Integration Guide

This document explains how to integrate the theme system with Scarab's daemon and client.

## Architecture Overview

```
┌─────────────────┐
│  scarab-daemon  │
│                 │
│  ThemePlugin    │ ◄─── Handles theme commands
│  ThemeManager   │      Manages theme state
└─────────────────┘
         │
         │ IPC (RemoteCommand)
         ▼
┌─────────────────┐
│  scarab-client  │
│                 │
│  ThemeUI        │ ◄─── Bevy UI components
│  Theme Preview  │      Live preview overlay
└─────────────────┘
```

## Daemon Integration

### 1. Load Plugin

In `scarab-daemon/src/main.rs`:

```rust
use scarab_themes::ThemePlugin;

// In plugin initialization
let theme_plugin = Box::new(ThemePlugin::new());
plugin_manager.register_plugin(theme_plugin)?;
```

### 2. Handle Theme Changes

The plugin automatically handles theme commands via `on_remote_command`. When a theme is applied, it should:

1. Update the config
2. Notify all connected clients
3. Persist the change

**TODO**: Extend `RemoteCommand` enum to support theme updates:

```rust
// In scarab-protocol/src/lib.rs
pub enum RemoteCommand {
    // ... existing variants ...

    ThemeChanged {
        theme_id: String,
        colors: ColorConfig,
    },
}
```

## Client Integration (Bevy UI)

### 1. Theme Selector UI

Create a Bevy system to display theme selection:

```rust
// In scarab-client/src/ui/theme_selector.rs

use bevy::prelude::*;
use scarab_themes::ThemeManager;

#[derive(Component)]
pub struct ThemeSelectorUI;

pub fn spawn_theme_selector(
    mut commands: Commands,
    manager: Res<ThemeManager>,
    asset_server: Res<AssetServer>,
) {
    commands
        .spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                position_type: PositionType::Absolute,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            background_color: Color::rgba(0.0, 0.0, 0.0, 0.8).into(),
            ..default()
        })
        .with_children(|parent| {
            // Modal container
            parent
                .spawn(NodeBundle {
                    style: Style {
                        width: Val::Px(600.0),
                        height: Val::Px(400.0),
                        flex_direction: FlexDirection::Column,
                        padding: UiRect::all(Val::Px(20.0)),
                        ..default()
                    },
                    background_color: Color::rgb(0.15, 0.15, 0.15).into(),
                    ..default()
                })
                .with_children(|modal| {
                    // Title
                    modal.spawn(TextBundle::from_section(
                        "Select Theme",
                        TextStyle {
                            font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                            font_size: 24.0,
                            color: Color::WHITE,
                        },
                    ));

                    // Theme list (scrollable)
                    for theme in manager.all_themes() {
                        spawn_theme_item(modal, theme, &asset_server);
                    }
                });
        })
        .insert(ThemeSelectorUI);
}

fn spawn_theme_item(
    parent: &mut ChildBuilder,
    theme: &Theme,
    asset_server: &AssetServer,
) {
    parent
        .spawn(ButtonBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Px(60.0),
                margin: UiRect::all(Val::Px(5.0)),
                padding: UiRect::all(Val::Px(10.0)),
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::Center,
                ..default()
            },
            background_color: Color::rgb(0.2, 0.2, 0.2).into(),
            ..default()
        })
        .with_children(|button| {
            // Theme name and description
            button.spawn(TextBundle::from_section(
                theme.name(),
                TextStyle {
                    font: asset_server.load("fonts/FiraSans-Regular.ttf"),
                    font_size: 18.0,
                    color: Color::WHITE,
                },
            ));

            // Color preview swatches
            spawn_color_preview(button, theme);
        });
}

fn spawn_color_preview(parent: &mut ChildBuilder, theme: &Theme) {
    parent
        .spawn(NodeBundle {
            style: Style {
                width: Val::Px(120.0),
                height: Val::Px(40.0),
                flex_direction: FlexDirection::Row,
                gap: Val::Px(2.0),
                ..default()
            },
            ..default()
        })
        .with_children(|container| {
            // Show 8 color swatches
            let colors = [
                &theme.colors.palette.red,
                &theme.colors.palette.green,
                &theme.colors.palette.yellow,
                &theme.colors.palette.blue,
                &theme.colors.palette.magenta,
                &theme.colors.palette.cyan,
                &theme.colors.foreground,
                &theme.colors.background,
            ];

            for color_hex in colors {
                let color = hex_to_color(color_hex);
                container.spawn(NodeBundle {
                    style: Style {
                        width: Val::Px(15.0),
                        height: Val::Px(40.0),
                        ..default()
                    },
                    background_color: color.into(),
                    ..default()
                });
            }
        });
}

// Helper to convert hex to Bevy Color
fn hex_to_color(hex: &str) -> Color {
    let hex = hex.trim_start_matches('#');
    let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(0) as f32 / 255.0;
    let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(0) as f32 / 255.0;
    let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(0) as f32 / 255.0;
    Color::rgb(r, g, b)
}
```

### 2. Live Preview System

Create a system to handle live preview:

```rust
// In scarab-client/src/ui/theme_preview.rs

use bevy::prelude::*;

#[derive(Resource)]
pub struct ThemePreview {
    pub active: bool,
    pub overlay_alpha: f32,
}

pub fn update_theme_preview(
    mut preview: ResMut<ThemePreview>,
    time: Res<Time>,
    keyboard: Res<Input<KeyCode>>,
) {
    // Fade in/out preview overlay
    if preview.active {
        preview.overlay_alpha = (preview.overlay_alpha + time.delta_seconds() * 2.0).min(1.0);
    } else {
        preview.overlay_alpha = (preview.overlay_alpha - time.delta_seconds() * 2.0).max(0.0);
    }

    // Cancel preview on Escape
    if keyboard.just_pressed(KeyCode::Escape) {
        preview.active = false;
    }
}

pub fn render_preview_overlay(
    mut commands: Commands,
    preview: Res<ThemePreview>,
    query: Query<Entity, With<ThemePreviewOverlay>>,
) {
    if preview.overlay_alpha > 0.0 {
        // Show preview banner
        if query.is_empty() {
            commands.spawn((
                NodeBundle {
                    style: Style {
                        width: Val::Percent(100.0),
                        height: Val::Px(40.0),
                        position_type: PositionType::Absolute,
                        top: Val::Px(0.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    background_color: Color::rgba(0.0, 0.5, 1.0, preview.overlay_alpha * 0.8).into(),
                    ..default()
                },
                ThemePreviewOverlay,
            ));
        }
    } else {
        // Remove preview banner
        for entity in query.iter() {
            commands.entity(entity).despawn_recursive();
        }
    }
}

#[derive(Component)]
struct ThemePreviewOverlay;
```

### 3. Add to Client Main

In `scarab-client/src/main.rs`:

```rust
mod ui {
    pub mod theme_selector;
    pub mod theme_preview;
}

use ui::{theme_selector, theme_preview};
use scarab_themes::ThemeManager;

fn main() {
    let mut app = App::new();

    // Initialize theme manager
    let mut theme_manager = ThemeManager::new();
    theme_manager.initialize().expect("Failed to init themes");

    app
        .add_plugins(DefaultPlugins)
        .insert_resource(theme_manager)
        .insert_resource(theme_preview::ThemePreview {
            active: false,
            overlay_alpha: 0.0,
        })
        .add_systems(Update, theme_preview::update_theme_preview)
        .add_systems(Update, theme_preview::render_preview_overlay)
        .run();
}
```

## TODO List

### High Priority

- [ ] Extend `RemoteCommand` enum to include `ThemeChanged` variant
- [ ] Implement IPC message passing for theme updates
- [ ] Create Bevy UI components for theme selector
- [ ] Add live preview system with smooth transitions
- [ ] Persist theme selection to config file

### Medium Priority

- [ ] Add keyboard shortcuts for quick theme switching (e.g., Ctrl+T)
- [ ] Implement theme preview with sample terminal output
- [ ] Add "favorite themes" feature
- [ ] Create theme carousel/slider UI
- [ ] Add theme search/filter by tags

### Low Priority

- [ ] Add theme auto-switch based on time of day
- [ ] Implement theme randomizer
- [ ] Add theme color extraction from wallpaper
- [ ] Create theme sharing/marketplace integration
- [ ] Add theme animation transitions

## Testing

Test the integration:

```bash
# Build the workspace
cargo build -p scarab-themes

# Run tests
cargo test -p scarab-themes

# Run integration tests
cargo test -p scarab-themes --test integration_test

# Check formatting
cargo fmt -p scarab-themes -- --check

# Run clippy
cargo clippy -p scarab-themes
```

## Example Theme Files

See `examples/custom-theme.toml` for a complete example of creating custom themes.

Example Base16 import:

```bash
# Download Base16 theme
curl -o ~/.config/scarab/themes/ocean.yaml \
  https://raw.githubusercontent.com/chriskempson/base16-schemes/master/ocean.yaml

# Import via command palette
# Open: Ctrl+Shift+P
# Type: "Theme: Import"
# Select: ocean.yaml
```

## Performance Considerations

1. **Theme Loading**: All themes are loaded at startup. For 13 themes, this adds ~10ms to startup time.

2. **Live Preview**: Preview updates trigger full terminal re-render. Use debouncing to limit update frequency.

3. **IPC**: Theme changes are sent via IPC. Consider batching if multiple theme operations occur rapidly.

4. **Memory**: Each theme uses ~2KB. Total memory overhead is minimal (<50KB for all themes).
