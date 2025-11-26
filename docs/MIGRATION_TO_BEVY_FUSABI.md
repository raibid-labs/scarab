# Migration Guide: adopting `bevy-fusabi`

**Status: ✅ COMPLETED**

This document outlines the steps required to refactor Scarab to use the shared `bevy-fusabi` crate for configuration and scripting.

## Migration Summary

The migration to `bevy-fusabi` has been successfully completed with the following changes:

1. **Dependencies Updated**: Added `bevy-fusabi` to workspace and `scarab-config`
2. **Plugin Created**: `ScarabConfigPlugin` provides Bevy integration with hot-reloading
3. **Extraction Refactored**: Config extraction now works with `fusabi_vm::Value` directly
4. **Client Updated**: scarab-client now supports the new plugin (currently opt-in)

## Usage

## 1. Add Dependency

Update `Cargo.toml` in the workspace and relevant crates (`scarab-config`, `scarab-daemon`) to depend on `bevy-fusabi`.

```toml
[dependencies]
# Adjust path as necessary based on where you cloned fusabi-lang
bevy-fusabi = { path = "../../fusabi-lang/bevy-fusabi" }
```

## 2. Refactor `scarab-config`

The goal is to replace the manual file reading and compilation pipeline in `FusabiConfigLoader` with Bevy's asset system.

### A. Remove Manual Loading Logic
Delete or deprecate `FusabiConfigLoader::from_file` and `from_source`. We no longer need to manually instantiate `Lexer`, `Parser`, or `Compiler`.

### B. Implement `FusabiConfigPlugin`
Create a Bevy plugin that handles the asynchronous loading of configuration.

```rust
// crates/scarab-config/src/plugin.rs

use bevy::prelude::*;
use bevy_fusabi::prelude::*;
use fusabi_vm::Vm;

#[derive(Resource)]
pub struct ConfigHandle(pub Handle<FusabiScript>);

pub struct ScarabConfigPlugin;

impl Plugin for ScarabConfigPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(bevy_fusabi::FusabiPlugin)
           .add_systems(Startup, load_config)
           .add_systems(Update, apply_config_updates);
    }
}

fn load_config(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Bevy automatically handles .fsx (source) or .fzb (bytecode)
    let handle = asset_server.load("config.fsx");
    commands.insert_resource(ConfigHandle(handle));
}

fn apply_config_updates(
    mut events: EventReader<AssetEvent<FusabiScript>>,
    scripts: Res<Assets<FusabiScript>>,
    mut config_store: ResMut<ScarabConfig>, // Your existing config resource
) {
    for event in events.read() {
        match event {
            AssetEvent::LoadedWithDependencies { id } | AssetEvent::Modified { id } => {
                if let Some(script) = scripts.get(*id) {
                    info!("Reloading configuration: {}", script.name);
                    apply_script(script, &mut config_store);
                }
            }
            _ => {}
        }
    }
}

fn apply_script(script: &FusabiScript, config: &mut ScarabConfig) {
    // 1. Deserialize bytecode
    let chunk = match script.to_chunk() {
        Ok(c) => c,
        Err(e) => { error!("Config error: {}", e); return; }
    };

    // 2. Execute VM
    let mut vm = Vm::new();
    match vm.execute(chunk) {
        Ok(result) => {
            // 3. Reuse existing extraction logic from fusabi_loader.rs
            // You will need to refactor `extract_terminal_config`, etc. 
            // to accept `&Value` instead of `&FusabiModule`.
            update_config_from_value(config, &result);
        }
        Err(e) => error!("Runtime error in config: {:?}", e),
    }
}
```

## 3. Startup vs. Runtime

Scarab currently loads some config synchronously for window creation. 
*   **Strategy:** Keep a simplified synchronous loader *only* for the initial window bootstrap (if absolutely necessary), but move all runtime-updateable settings (fonts, colors, keybindings) to the asset system.
*   **Better Strategy:** Use `bevy-fusabi`'s logic manually for bootstrap if needed, or rely on default window settings until the config asset loads (popping in visuals).

## 4. Cleanup

**NOTE**: `fusabi-frontend` and `fusabi-vm` dependencies are still kept in `scarab-config` for backward compatibility with the synchronous `FusabiConfigLoader`. These can be made optional in a future refactor when all clients fully migrate to the plugin system.

---

## Using the New Plugin System

### Basic Usage

Add the plugin to your Bevy app:

```rust
use scarab_config::ScarabConfigPlugin;

app.add_plugins(ScarabConfigPlugin::default());
```

By default, it will look for `config.fsx` in the assets directory. You can specify a custom path:

```rust
app.add_plugins(ScarabConfigPlugin::new("my_config.fsx"));
```

### Asset Directory Setup

Create an `assets` directory in your project root and place your `config.fsx` file there:

```
scarab/
├── assets/
│   └── config.fsx    # Your Fusabi configuration
├── crates/
│   └── scarab-client/
└── Cargo.toml
```

### Hot Reloading

The plugin automatically watches for changes to your config file. When you modify `config.fsx`, the changes will be applied immediately without restarting the application.

### Accessing Config

The config is available as a Bevy resource:

```rust
fn my_system(config: Res<ScarabConfig>) {
    println!("Terminal columns: {}", config.terminal.columns);
}
```

### Backward Compatibility

The old synchronous loading approach is still available for cases where you need config before the Bevy app starts (e.g., window sizing):

```rust
use scarab_config::FusabiConfigLoader;

let config = FusabiConfigLoader::load_with_fallback()?;
// Use config for pre-Bevy initialization
app.insert_resource(config);
```

### Migration Path for scarab-client

Currently, `scarab-client` uses a hybrid approach:
1. Synchronous loading for initial window setup
2. Config inserted as a resource for runtime use
3. Plugin integration is available but opt-in (see commented code in `main.rs`)

To enable hot-reloading in scarab-client:
1. Uncomment the `ScarabConfigPlugin` import and usage in `main.rs`
2. Create an `assets/config.fsx` file
3. The config will update live when you modify the file
