# Fusabi ECS Bridge Usage Guide

This guide demonstrates how to use the Fusabi ECS bridge to connect Fusabi scripts with the Bevy ECS plugin host.

## Overview

The ECS bridge provides a thread-safe, event-driven communication channel between Fusabi scripts and Bevy ECS systems. Scripts can queue actions that are processed by ECS, and receive responses asynchronously.

## Architecture

```
┌─────────────────┐         ┌──────────────────┐         ┌─────────────────┐
│  Fusabi Script  │  ─────> │ FusabiNatives    │  ─────> │ PluginAction    │
│   (.fsx file)   │         │                  │         │   (Event)       │
└─────────────────┘         └──────────────────┘         └─────────────────┘
                                     │                             │
                                     │                             V
                                     │                    ┌─────────────────┐
                                     │                    │ ECS Systems     │
                                     │                    │ (process action)│
                                     │                    └─────────────────┘
                                     │                             │
                                     V                             V
                            ┌──────────────────┐         ┌─────────────────┐
                            │ PluginResponse   │  <─────┤ Response Events │
                            │   (collected)    │         │                 │
                            └──────────────────┘         └─────────────────┘
```

## Basic Setup

### 1. Add the Plugin to Your Bevy App

```rust
use bevy::prelude::*;
use scarab_client::FusabiEcsBridgePlugin;

fn main() {
    App::new()
        .add_plugins(FusabiEcsBridgePlugin)
        .run();
}
```

The plugin automatically:
- Initializes the `FusabiActionChannel` resource
- Registers `PluginAction` and `PluginResponse` events
- Sets up systems to flush actions and collect responses

### 2. Create Native Function Bindings

```rust
use scarab_client::{FusabiActionChannel, FusabiNatives};
use bevy::prelude::*;

fn setup_script_runtime(channel: Res<FusabiActionChannel>) {
    let plugin_id = "my_plugin".to_string();
    let natives = FusabiNatives::new(&channel, plugin_id);

    // Now you can call native functions from your Fusabi runtime
    natives.notify(
        "Hello".to_string(),
        "Script initialized!".to_string(),
        "info"
    );
}
```

## Available Native Functions

### UI Management

#### spawn_overlay
```rust
natives.ui_spawn_overlay(
    x: 10,           // terminal cell x position
    y: 5,            // terminal cell y position
    width: 40,       // width in cells
    height: 10,      // height in cells
    content: "Hello, World!".to_string(),
    z_index: 100.0   // rendering order
);
```

#### despawn_overlay
```rust
natives.ui_despawn_overlay(overlay_id: 42);
```

### Notifications

```rust
// Show info notification
natives.notify(
    "Title".to_string(),
    "Message body".to_string(),
    "info"  // "info" | "success" | "warning" | "error"
);
```

### Status Bar

```rust
// Add status item
natives.status_add(
    "right",              // "left" | "right"
    "🔥 Hot reload".to_string(),
    priority: 10          // higher = shown first
);

// Remove status item
natives.status_remove(item_id: 5);
```

### Keybindings

```rust
natives.register_keybinding(
    key: "p".to_string(),
    modifiers: vec!["ctrl".to_string(), "shift".to_string()],
    action_id: "my_custom_action".to_string()
);
```

### Terminal I/O

```rust
// Send input to terminal
natives.send_input(vec![0x1b, 0x5b, 0x41]); // ESC [ A (up arrow)

// Request terminal content
natives.get_terminal_rows(
    start_row: 0,
    end_row: 24
);
// Response will come via PluginResponse::TerminalContent
```

### Theme Updates

```rust
natives.update_theme(r#"{
    "foreground": "#f8f8f2",
    "background": "#282a36",
    "cursor": "#50fa7b"
}"#.to_string());
```

### Modals

```rust
use scarab_client::ModalItem;

natives.show_modal(
    "Select Option".to_string(),
    vec![
        ModalItem {
            label: "Option 1".to_string(),
            value: "opt1".to_string(),
            description: Some("First option".to_string()),
        },
        ModalItem {
            label: "Option 2".to_string(),
            value: "opt2".to_string(),
            description: None,
        },
    ]
);
```

## Handling Responses

Scripts can poll for responses from ECS systems:

```rust
fn poll_responses(channel: Res<FusabiActionChannel>) {
    let responses = channel.take_responses("my_plugin");

    for response in responses {
        match response {
            PluginResponse::OverlaySpawned { overlay_id, .. } => {
                println!("Overlay created with ID: {}", overlay_id);
            }
            PluginResponse::TerminalContent { rows, .. } => {
                println!("Received {} rows of terminal content", rows.len());
            }
            PluginResponse::KeybindingTriggered { action_id, .. } => {
                println!("Keybinding triggered: {}", action_id);
            }
            PluginResponse::Error { action, message, .. } => {
                eprintln!("Error in {}: {}", action, message);
            }
        }
    }
}
```

## Processing Actions in ECS

You can create custom systems to handle `PluginAction` events:

```rust
use bevy::prelude::*;
use scarab_client::{PluginAction, PluginResponse};

fn handle_plugin_actions(
    mut actions: EventReader<PluginAction>,
    mut responses: EventWriter<PluginResponse>,
) {
    for action in actions.read() {
        match action {
            PluginAction::SpawnOverlay { plugin_id, x, y, width, height, content, z_index } => {
                // Create the overlay entity
                let overlay_id = create_overlay(*x, *y, *width, *height, content, *z_index);

                // Send response
                responses.send(PluginResponse::OverlaySpawned {
                    plugin_id: plugin_id.clone(),
                    overlay_id,
                });
            }
            PluginAction::ShowNotification { plugin_id, title, message, level, duration_ms } => {
                // Show notification in UI
                show_notification(title, message, *level, *duration_ms);
            }
            // Handle other actions...
            _ => {}
        }
    }
}
```

## Thread Safety

The bridge uses `Arc<Mutex<Vec>>` internally for thread-safe communication:

- **Lock-free reads**: ECS systems use Bevy events (which are lock-free)
- **Minimal locking**: Only when queueing actions or collecting responses
- **No blocking**: Scripts never block the Bevy render thread

## Performance Considerations

1. **Batch operations**: Queue multiple actions before flushing
2. **Selective polling**: Only call `take_responses()` when needed
3. **System ordering**: The bridge systems run in Update schedule:
   ```
   flush_fusabi_actions (drains queue)
        ↓
   collect_fusabi_responses (stores results)
   ```

## Complete Example

```rust
use bevy::prelude::*;
use scarab_client::{
    FusabiActionChannel, FusabiEcsBridgePlugin, FusabiNatives,
    PluginAction, PluginResponse, NotificationLevel
};

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, FusabiEcsBridgePlugin))
        .add_systems(Startup, setup_script)
        .add_systems(Update, (handle_actions, poll_script_responses))
        .run();
}

fn setup_script(channel: Res<FusabiActionChannel>) {
    let natives = FusabiNatives::new(&channel, "example_plugin".to_string());

    // Show welcome notification
    natives.notify(
        "Welcome".to_string(),
        "Fusabi script initialized!".to_string(),
        "success"
    );

    // Spawn an overlay
    natives.ui_spawn_overlay(5, 2, 30, 8, "Plugin UI".to_string(), 50.0);

    // Register keybinding
    natives.register_keybinding(
        "r".to_string(),
        vec!["ctrl".to_string()],
        "reload_script".to_string()
    );
}

fn handle_actions(
    mut actions: EventReader<PluginAction>,
    mut responses: EventWriter<PluginResponse>,
) {
    for action in actions.read() {
        match action {
            PluginAction::SpawnOverlay { plugin_id, .. } => {
                // Implementation here
                responses.send(PluginResponse::OverlaySpawned {
                    plugin_id: plugin_id.clone(),
                    overlay_id: 123,
                });
            }
            _ => {}
        }
    }
}

fn poll_script_responses(channel: Res<FusabiActionChannel>) {
    let responses = channel.take_responses("example_plugin");

    for response in responses {
        info!("Script response: {:?}", response);
    }
}
```

## Testing

The bridge includes comprehensive tests:

```bash
cargo test -p scarab-client --lib ecs_bridge
```

All native functions are tested for:
- Correct action queueing
- Response filtering by plugin_id
- Thread-safe channel operations
- Event dispatching

## Next Steps

- Implement actual Fusabi VM integration with these natives
- Add hot-reload support for .fsx scripts
- Create example scripts demonstrating common patterns
- Build UI components to visualize plugin actions
