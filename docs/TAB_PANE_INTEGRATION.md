# Tab/Pane Plugin Integration Guide

This guide shows how to integrate the `scarab-tabs` and `scarab-panes` plugins into Scarab Terminal.

## Daemon Integration

### 1. Add Dependencies

Update `/home/beengud/raibid-labs/scarab/crates/scarab-daemon/Cargo.toml`:

```toml
[dependencies]
# ... existing dependencies ...
scarab-tabs = { path = "../scarab-tabs" }
scarab-panes = { path = "../scarab-panes" }
```

### 2. Register Plugins

Update `/home/beengud/raibid-labs/scarab/crates/scarab-daemon/src/main.rs`:

```rust
// Add imports
use scarab_tabs::TabsPlugin;
use scarab_panes::PanesPlugin;

// In main() function, after plugin_manager initialization:

// Register Tabs Plugin
if let Err(e) = plugin_manager
    .register_plugin(Box::new(TabsPlugin::new()))
    .await
{
    eprintln!("Failed to register TabsPlugin: {}", e);
}

// Register Panes Plugin
let panes_plugin = PanesPlugin::with_size(
    config.terminal.columns,
    config.terminal.rows,
);
if let Err(e) = plugin_manager
    .register_plugin(Box::new(panes_plugin))
    .await
{
    eprintln!("Failed to register PanesPlugin: {}", e);
}

// ... rest of initialization ...
```

### 3. Handle IPC Messages

Extend the IPC handler in `/home/beengud/raibid-labs/scarab/crates/scarab-daemon/src/ipc.rs`:

```rust
use scarab_protocol::{ControlMessage, DaemonMessage, TabInfo, PaneInfo};

impl IpcServer {
    async fn handle_control_message(&self, msg: ControlMessage) -> Result<()> {
        match msg {
            // ... existing handlers ...

            // Tab management
            ControlMessage::TabCreate { title } => {
                // Plugin will handle this via on_remote_command
                // Send confirmation to client
                self.send_to_client(DaemonMessage::TabCreated {
                    tab: TabInfo {
                        id: new_id,
                        title: title.unwrap_or_else(|| format!("Terminal {}", new_id)),
                        session_id: None,
                        is_active: true,
                        pane_count: 1,
                    }
                }).await?;
            }

            ControlMessage::TabClose { tab_id } => {
                // Handle tab closing
                self.send_to_client(DaemonMessage::TabClosed { tab_id }).await?;
            }

            // Pane management
            ControlMessage::PaneSplit { pane_id, direction } => {
                // Plugin will handle the split
                // Send layout update to client
            }

            ControlMessage::PaneClose { pane_id } => {
                // Handle pane closing
                self.send_to_client(DaemonMessage::PaneClosed { pane_id }).await?;
            }

            _ => {}
        }
        Ok(())
    }
}
```

## Client Integration

### 1. Add Dependencies

Update `/home/beengud/raibid-labs/scarab/crates/scarab-client/Cargo.toml`:

```toml
[dependencies]
# ... existing dependencies ...
scarab-tabs = { path = "../scarab-tabs" }
scarab-panes = { path = "../scarab-panes" }
```

### 2. Create Tab Bar UI Plugin

Create `/home/beengud/raibid-labs/scarab/crates/scarab-client/src/ui/tab_bar.rs`:

```rust
use bevy::prelude::*;
use scarab_config::ScarabConfig;

#[derive(Resource, Default)]
pub struct TabState {
    pub tabs: Vec<TabInfo>,
    pub active_tab_index: usize,
}

pub struct TabBarPlugin;

impl Plugin for TabBarPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TabState>()
            .add_systems(Update, (
                render_tab_bar,
                handle_tab_clicks,
            ));
    }
}

fn render_tab_bar(
    mut commands: Commands,
    config: Res<ScarabConfig>,
    tab_state: Res<TabState>,
    query: Query<Entity, With<TabBarContainer>>,
) {
    if !config.ui.show_tabs {
        return;
    }

    // Clear existing tab bar
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }

    // Create tab bar container
    let container = commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Px(30.0),
            flex_direction: FlexDirection::Row,
            ..default()
        },
        BackgroundColor(Color::srgb(0.2, 0.2, 0.2)),
        TabBarContainer,
    )).id();

    // Render individual tabs
    for (i, tab) in tab_state.tabs.iter().enumerate() {
        let is_active = i == tab_state.active_tab_index;
        let tab_entity = spawn_tab(&mut commands, tab, is_active);
        commands.entity(container).add_child(tab_entity);
    }
}

fn spawn_tab(commands: &mut Commands, tab: &TabInfo, is_active: bool) -> Entity {
    let bg_color = if is_active {
        Color::srgb(0.3, 0.5, 0.8)
    } else {
        Color::srgb(0.25, 0.25, 0.25)
    };

    commands.spawn((
        Node {
            width: Val::Px(150.0),
            height: Val::Percent(100.0),
            padding: UiRect::all(Val::Px(10.0)),
            margin: UiRect::horizontal(Val::Px(2.0)),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        BackgroundColor(bg_color),
        TabButton { id: tab.id },
    ))
    .with_children(|parent| {
        parent.spawn((
            Text::new(&tab.title),
            TextFont {
                font_size: 14.0,
                ..default()
            },
            TextColor(Color::WHITE),
        ));
    })
    .id()
}

#[derive(Component)]
struct TabBarContainer;

#[derive(Component)]
struct TabButton {
    id: u64,
}

fn handle_tab_clicks(
    interaction_query: Query<(&Interaction, &TabButton), Changed<Interaction>>,
    // Send TabSwitch message to daemon via IPC
) {
    for (interaction, tab_button) in interaction_query.iter() {
        if *interaction == Interaction::Pressed {
            // Send ControlMessage::TabSwitch { tab_id: tab_button.id }
        }
    }
}
```

### 3. Create Pane Border Renderer

Create `/home/beengud/raibid-labs/scarab/crates/scarab-client/src/ui/pane_borders.rs`:

```rust
use bevy::prelude::*;

#[derive(Resource, Default)]
pub struct PaneState {
    pub panes: Vec<PaneInfo>,
    pub active_pane_id: u64,
}

pub struct PaneBorderPlugin;

impl Plugin for PaneBorderPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PaneState>()
            .add_systems(Update, render_pane_borders);
    }
}

fn render_pane_borders(
    pane_state: Res<PaneState>,
    mut gizmos: Gizmos,
) {
    for pane in &pane_state.panes {
        let color = if pane.id == pane_state.active_pane_id {
            Color::srgb(0.3, 0.5, 0.8) // Blue for active
        } else {
            Color::srgb(0.4, 0.4, 0.4) // Gray for inactive
        };

        // Draw border around pane
        let x = pane.x as f32 * CHAR_WIDTH;
        let y = pane.y as f32 * CHAR_HEIGHT;
        let width = pane.width as f32 * CHAR_WIDTH;
        let height = pane.height as f32 * CHAR_HEIGHT;

        // Top border
        gizmos.line_2d(
            Vec2::new(x, y),
            Vec2::new(x + width, y),
            color,
        );
        // Right border
        gizmos.line_2d(
            Vec2::new(x + width, y),
            Vec2::new(x + width, y + height),
            color,
        );
        // Bottom border
        gizmos.line_2d(
            Vec2::new(x + width, y + height),
            Vec2::new(x, y + height),
            color,
        );
        // Left border
        gizmos.line_2d(
            Vec2::new(x, y + height),
            Vec2::new(x, y),
            color,
        );
    }
}

const CHAR_WIDTH: f32 = 10.0;  // TODO: Get from config
const CHAR_HEIGHT: f32 = 20.0; // TODO: Get from config
```

### 4. Update Main Client App

Update `/home/beengud/raibid-labs/scarab/crates/scarab-client/src/main.rs`:

```rust
mod ui;
use ui::{TabBarPlugin, PaneBorderPlugin};

fn main() {
    // ... existing setup ...

    let mut app = App::new();
    app
        // ... existing plugins ...
        .add_plugins(TabBarPlugin)      // Add tab bar
        .add_plugins(PaneBorderPlugin)  // Add pane borders
        // ... rest of setup ...
        .run();
}
```

### 5. Handle IPC Messages

Update the IPC message handler in `/home/beengud/raibid-labs/scarab/crates/scarab-client/src/ipc.rs`:

```rust
use scarab_protocol::DaemonMessage;

impl IpcClient {
    fn handle_daemon_message(&mut self, msg: DaemonMessage) {
        match msg {
            // ... existing handlers ...

            DaemonMessage::TabCreated { tab } => {
                // Update TabState resource
                self.tab_state.tabs.push(tab);
            }

            DaemonMessage::TabClosed { tab_id } => {
                // Remove tab from TabState
                self.tab_state.tabs.retain(|t| t.id != tab_id);
            }

            DaemonMessage::TabSwitched { tab_id } => {
                // Update active tab
                if let Some(index) = self.tab_state.tabs.iter().position(|t| t.id == tab_id) {
                    self.tab_state.active_tab_index = index;
                }
            }

            DaemonMessage::PaneCreated { pane } => {
                // Update PaneState resource
                self.pane_state.panes.push(pane);
            }

            DaemonMessage::PaneLayoutUpdate { panes } => {
                // Replace entire pane layout
                self.pane_state.panes = panes;
            }

            _ => {}
        }
    }
}
```

## Configuration

Add to `~/.config/scarab/config.toml`:

```toml
[ui]
show_tabs = true
tab_position = "top"  # "top" | "bottom" | "left" | "right"

[tabs]
max_tabs = 20
default_title_template = "Terminal {n}"
close_last_tab_quits = false

[panes]
default_split = "vertical"
border_style = "rounded"
border_color = "#4A90E2"
min_pane_size = 10

[plugins]
enabled = [
    "scarab-tabs",
    "scarab-panes",
    # ... other plugins ...
]
```

## Testing the Integration

### 1. Build the Project

```bash
cargo build --workspace
```

### 2. Start the Daemon

```bash
cargo run -p scarab-daemon
```

Expected output:
```
Starting Scarab Daemon...
Registered plugin: scarab-tabs v0.1.0
Registered plugin: scarab-panes v0.1.0
```

### 3. Start the Client

```bash
cargo run -p scarab-client
```

### 4. Test Keybindings

- **Create Tab**: Press `Ctrl+Shift+T`
- **Close Tab**: Press `Ctrl+Shift+W`
- **Switch Tab**: Press `Ctrl+Tab`
- **Split Pane**: Press `Ctrl+Shift+-` or `Ctrl+Shift+|`

### 5. Test Command Palette

- Press `Ctrl+P` to open Command Palette
- Search for "tab" or "pane" to see available commands
- Select a command to execute

## Troubleshooting

### Issue: Plugins not loading

**Solution**: Check daemon logs for registration errors. Ensure plugin API version compatibility.

### Issue: Keybindings not working

**Solution**: Verify input handling in `on_input` hook. Check for conflicts with other plugins.

### Issue: Tab bar not rendering

**Solution**: Ensure `show_tabs = true` in config. Check Bevy UI systems are running.

### Issue: Pane borders not visible

**Solution**: Verify gizmo rendering is enabled. Check pane coordinates are within window bounds.

## Next Steps

1. **PTY Integration**: Implement PTY session per pane
2. **State Persistence**: Integrate with scarab-session for saving/restoring
3. **Advanced Features**: Drag-and-drop, pane resizing, zoom mode
4. **Polish**: Animations, themes, accessibility

## Resources

- [Tab Plugin README](../crates/scarab-tabs/README.md)
- [Pane Plugin README](../crates/scarab-panes/README.md)
- [Design Document](TAB_PANE_DESIGN.md)
- [Scarab Plugin API](../crates/scarab-plugin-api/README.md)
