use bevy::prelude::*;
use scarab_client::integration::{IntegrationPlugin, SharedMemWrapper, SharedMemoryReader};
use scarab_client::{AdvancedUIPlugin, ScriptingPlugin};
use scarab_config::ConfigLoader;
use scarab_protocol::{SharedState, SHMEM_PATH};
use shared_memory::ShmemConf;
use std::sync::Arc;

mod ipc;
use ipc::IpcPlugin;

fn main() {
    // Load Configuration
    let home_dir = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    let config_path = std::path::PathBuf::from(&home_dir).join(".config/scarab/config.toml");

    let config = if config_path.exists() {
        println!("Loading config from: {}", config_path.display());
        ConfigLoader::from_file(&config_path).expect("Failed to load config")
    } else {
        println!(
            "No config found at {}, using defaults",
            config_path.display()
        );
        scarab_config::ScarabConfig::default()
    };

    // Initialize shared memory before Bevy app starts
    let shmem = match ShmemConf::new()
        .size(std::mem::size_of::<SharedState>())
        .os_id(SHMEM_PATH)
        .open()
    {
        Ok(m) => {
            println!("Connected to shared memory at: {}", SHMEM_PATH);
            Arc::new(m)
        }
        Err(e) => {
            eprintln!(
                "Failed to open shared memory: {}. Is the daemon running?",
                e
            );
            std::process::exit(1);
        }
    };

    // Initialize the resource from the library crate
    let reader = SharedMemoryReader {
        shmem: SharedMemWrapper(shmem),
        last_sequence: 0,
    };

    // Calculate window size from terminal dimensions
    // Use font size to estimate pixel dimensions (rough approximation)
    let char_width = config.font.size * 0.6; // Monospace approximation
    let char_height = config.font.size * 1.2;
    let window_width = config.terminal.columns as f32 * char_width;
    let window_height = config.terminal.rows as f32 * char_height;

    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Scarab Terminal".into(),
                resolution: (window_width, window_height).into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(IpcPlugin) // Add IPC support
        .add_plugins(AdvancedUIPlugin) // Add advanced UI features
        .add_plugins(ScriptingPlugin) // Add client-side scripting
        .add_plugins(IntegrationPlugin) // Add text rendering
        .insert_resource(reader)
        .insert_resource(config) // Make config available to all systems
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands) {
    // Bevy 0.15 uses Camera2d component, Camera2dBundle is deprecated
    commands.spawn(Camera2d::default());
    // IntegrationPlugin handles spawning the TerminalGridEntity
    println!("Scarab Client Initialized with shared memory reader, IPC, and scripting.");
}
