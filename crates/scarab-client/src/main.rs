use bevy::prelude::*;
use bevy::render::camera::OrthographicProjection;
use scarab_client::integration::{IntegrationPlugin, SharedMemWrapper, SharedMemoryReader};
use scarab_client::{AdvancedUIPlugin, ScriptingPlugin, ScrollbackPlugin};
use scarab_config::{ConfigLoader, FusabiConfigLoader};
// Uncomment to enable hot-reloading config via bevy-fusabi:
// use scarab_config::ScarabConfigPlugin;
use scarab_protocol::{SharedState, SHMEM_PATH};
use shared_memory::ShmemConf;
use std::sync::Arc;

use scarab_client::ipc::IpcPlugin;

#[cfg(feature = "plugin-inspector")]
use scarab_client::PluginInspectorPlugin;

fn main() {
    // Load Configuration (Fusabi-based)
    let home_dir = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    let fusabi_config_path = std::path::PathBuf::from(&home_dir)
        .join(".config/scarab/config.fsx");
    let toml_config_path = std::path::PathBuf::from(&home_dir)
        .join(".config/scarab/config.toml");

    let config = if fusabi_config_path.exists() {
        println!("Loading Fusabi config from: {}", fusabi_config_path.display());
        FusabiConfigLoader::from_file(&fusabi_config_path)
            .expect("Failed to load Fusabi config")
    } else if toml_config_path.exists() {
        println!("⚠️  Loading legacy TOML config from: {}", toml_config_path.display());
        println!("💡 Consider migrating to Fusabi config: {}", fusabi_config_path.display());
        ConfigLoader::from_file(&toml_config_path)
            .expect("Failed to load TOML config")
    } else {
        println!("No config found (tried .fsx and .toml), using defaults");
        println!("Create {} to customize your terminal", fusabi_config_path.display());
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

    // Window icon loading note: Bevy 0.15 window icon support requires platform-specific handling
    // and may not be available in all backends. For now, we log if an icon path is configured.
    if let Some(icon_path) = &config.ui.window_icon {
        println!("Custom window icon specified: {}", icon_path);
        println!("Note: Window icon loading requires platform-specific implementation in Bevy 0.15");
    }

    let mut app = App::new();

    app.add_plugins(
        DefaultPlugins
            .set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Scarab Terminal".into(),
                    resolution: (window_width, window_height).into(),
                    window_theme: Some(bevy::window::WindowTheme::Dark),
                    ..default()
                }),
                ..default()
            })
            .set(bevy::log::LogPlugin {
                level: bevy::log::Level::INFO,
                filter: "wgpu=error,bevy_render::view::window=error,bevy_ecs=info".into(),
                ..default()
            }),
    )
        .add_plugins(IpcPlugin) // Add IPC support
        .add_plugins(ScrollbackPlugin) // Add scrollback buffer management
        .add_plugins(AdvancedUIPlugin) // Add advanced UI features (includes search, indicators)
        .add_plugins(ScriptingPlugin) // Add client-side scripting
        .add_plugins(IntegrationPlugin) // Add text rendering
        .insert_resource(reader)
        .insert_resource(config) // Make initial config available (will be updated by plugin)
        // NOTE: Uncomment the following line to enable hot-reloading config via bevy-fusabi
        // .add_plugins(ScarabConfigPlugin::new("config.fsx"))
        .add_systems(Startup, setup);

    // Conditionally add plugin inspector
    #[cfg(feature = "plugin-inspector")]
    {
        app.add_plugins(PluginInspectorPlugin);
        println!("Plugin Inspector enabled - Press Ctrl+Shift+P to open");
    }

    app.run();
}

fn setup(mut commands: Commands, windows: Query<&Window, With<bevy::window::PrimaryWindow>>) {
    // Use 2D camera for terminal rendering (terminal is 2D, not 3D!)
    println!("Camera setup: Using Camera2d for 2D terminal rendering");

    // Get window dimensions to set up proper viewport
    let (width, height) = if let Ok(window) = windows.get_single() {
        (window.width(), window.height())
    } else {
        (800.0, 600.0)
    };

    // Camera2d defaults to center origin. We want top-left origin.
    // Move camera so that (0,0) is at top-left of screen
    commands.spawn((
        Camera2d,
        Camera {
            clear_color: ClearColorConfig::Custom(Color::srgb(0.0, 0.0, 0.0)), // Black background
            ..default()
        },
        OrthographicProjection {
            // Standard 2D: viewport goes from (0,0) at top-left to (width, height) at bottom-right
            // But Camera2d has Y pointing up, so we need negative Y
            viewport_origin: Vec2::new(0.0, 0.0),
            ..OrthographicProjection::default_2d()
        },
        Transform::from_xyz(width / 2.0, -height / 2.0, 0.0),
    ));

    println!("Scarab Client Initialized with shared memory reader, IPC, scrollback, and scripting.");
}
