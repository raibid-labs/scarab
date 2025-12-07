use bevy::prelude::*;
use bevy::render::camera::OrthographicProjection;
use scarab_client::input::{ModeStack, NavInputRouter, NavStyle};
use scarab_client::integration::{IntegrationPlugin, SharedMemWrapper, SharedMemoryReader};
use scarab_client::navigation::{FocusablePlugin, NavigationPlugin};
use scarab_client::rendering::HintOverlayPlugin;
use scarab_client::{
    AccessibilityPlugin, AdvancedUIPlugin, CopyModePlugin, EventsPlugin, GraphicsInspectorPlugin,
    ImagesPlugin, ScarabEffectsPlugin, ScarabTelemetryPlugin, ScriptingPlugin, ScrollbackPlugin,
    TutorialPlugin,
};
use scarab_config::{ConfigLoader, FusabiConfigLoader};
// Uncomment to enable hot-reloading config via bevy-fusabi:
// use scarab_config::ScarabConfigPlugin;
use scarab_protocol::terminal_state::TerminalStateReader;
use scarab_protocol::{SharedState, SHMEM_PATH, SHMEM_PATH_ENV};
use shared_memory::ShmemConf;
use std::sync::Arc;

use clap::Parser;
use scarab_client::ipc::{IpcPlugin, StartupCommand};

#[cfg(feature = "plugin-inspector")]
use scarab_client::PluginInspectorPlugin;

#[cfg(debug_assertions)]
use scarab_client::BevyInspectorPlugin;

#[derive(Parser, Debug)]
#[command(author, version, about = "Scarab Terminal Client")]
struct Args {
    /// Command to execute on startup (sends input to the running shell)
    #[arg(long)]
    command: Option<String>,

    /// Run in headless mode (no window, dump terminal grid and exit)
    #[arg(long)]
    headless: bool,
}

fn main() {
    let args = Args::parse();

    // Load Configuration (Fusabi-based)
    let home_dir = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    let fusabi_config_path = std::path::PathBuf::from(&home_dir).join(".config/scarab/config.fsx");
    let toml_config_path = std::path::PathBuf::from(&home_dir).join(".config/scarab/config.toml");

    let config = if fusabi_config_path.exists() {
        println!(
            "Loading Fusabi config from: {}",
            fusabi_config_path.display()
        );
        FusabiConfigLoader::from_file(&fusabi_config_path).expect("Failed to load Fusabi config")
    } else if toml_config_path.exists() {
        println!(
            "⚠️  Loading legacy TOML config from: {}",
            toml_config_path.display()
        );
        println!(
            "💡 Consider migrating to Fusabi config: {}",
            fusabi_config_path.display()
        );
        ConfigLoader::from_file(&toml_config_path).expect("Failed to load TOML config")
    } else {
        println!("No config found (tried .fsx and .toml), using defaults");
        println!(
            "Create {} to customize your terminal",
            fusabi_config_path.display()
        );
        scarab_config::ScarabConfig::default()
    };

    // Initialize shared memory before Bevy app starts
    // Support environment variable override for sandboxed environments
    let shmem_path = std::env::var(SHMEM_PATH_ENV).unwrap_or_else(|_| SHMEM_PATH.to_string());

    let shmem = match ShmemConf::new()
        .size(std::mem::size_of::<SharedState>())
        .os_id(&shmem_path)
        .open()
    {
        Ok(m) => {
            println!("Connected to shared memory at: {}", shmem_path);
            Arc::new(m)
        }
        Err(e) => {
            eprintln!("Failed to open shared memory at {}: {}", shmem_path, e);
            eprintln!("Is the daemon running?");
            eprintln!("");
            eprintln!("If using a custom shared memory path, ensure both daemon and client");
            eprintln!("use the same {} setting.", SHMEM_PATH_ENV);
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

    // Branch: Headless mode vs Normal windowed mode
    if args.headless {
        run_headless(reader, args.command);
    } else {
        run_windowed(reader, config, window_width, window_height, args.command);
    }
}

/// Run in headless mode (no window, dump terminal grid and exit)
fn run_headless(reader: SharedMemoryReader, command: Option<String>) {
    println!("Running in headless mode");

    let mut app = App::new();

    // Use MinimalPlugins for headless operation
    app.add_plugins(MinimalPlugins);

    // Add IPC plugin for command injection
    app.add_plugins(IpcPlugin);

    // Insert shared memory reader
    app.insert_resource(reader);

    // Insert startup command if provided
    if let Some(cmd) = command {
        println!("Startup command: {}", cmd);
        app.insert_resource(StartupCommand(cmd));
    }

    // Add headless mode marker resource
    app.insert_resource(HeadlessMode {
        startup_time: std::time::Instant::now(),
        max_wait_secs: 5.0,
        command_sent: false,
        initial_sequence: 0,
    });

    // Add headless system to dump grid and exit
    app.add_systems(Update, headless_dump_and_exit);

    println!("Headless mode initialized, waiting for terminal output...");
    app.run();
}

/// Run in normal windowed mode
fn run_windowed(
    reader: SharedMemoryReader,
    config: scarab_config::ScarabConfig,
    window_width: f32,
    window_height: f32,
    command: Option<String>,
) {
    // Window icon loading note: Bevy 0.15 window icon support requires platform-specific handling
    // and may not be available in all backends. For now, we log if an icon path is configured.
    if let Some(icon_path) = &config.ui.window_icon {
        println!("Custom window icon specified: {}", icon_path);
        println!(
            "Note: Window icon loading requires platform-specific implementation in Bevy 0.15"
        );
    }

    let mut app = App::new();

    if let Some(cmd) = command {
        app.insert_resource(StartupCommand(cmd));
    }

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
    .add_plugins(EventsPlugin::default()) // Add event handling (client and daemon forwarding)
    .add_plugins(NavigationPlugin) // Add core navigation system (modes, events, state)
    .add_plugins(FocusablePlugin) // Add focusable detection and scanning
    .add_plugins(HintOverlayPlugin) // Add hint overlay rendering
    .add_plugins(ScrollbackPlugin) // Add scrollback buffer management
    .add_plugins(CopyModePlugin) // Add vim-like copy mode navigation
    .add_plugins(ImagesPlugin) // Add inline image rendering support
    .add_plugins(AdvancedUIPlugin) // Add advanced UI features (includes search, indicators)
    .add_plugins(ScriptingPlugin) // Add client-side scripting
    .add_plugins(IntegrationPlugin) // Add text rendering
    .add_plugins(TutorialPlugin) // Add interactive tutorial system
    .add_plugins(ScarabEffectsPlugin) // Add post-processing effects (blur, glow)
    .add_plugins(ScarabTelemetryPlugin) // Add telemetry HUD overlay (Ctrl+Shift+T to toggle)
    .add_plugins(AccessibilityPlugin) // Add accessibility features (screen reader, export, high contrast)
    .insert_resource(reader)
    .insert_resource(config) // Make initial config available (will be updated by plugin)
    .insert_resource(NavInputRouter::new(NavStyle::VimiumStyle)) // Initialize navigation input router with Vimium-style keybindings
    .insert_resource(ModeStack::new()) // Initialize mode stack (starts in Normal mode)
    // NOTE: Uncomment the following line to enable hot-reloading config via bevy-fusabi
    // .add_plugins(ScarabConfigPlugin::new("config.fsx"))
    .add_systems(Startup, setup);

    // Conditionally add plugin inspector
    #[cfg(feature = "plugin-inspector")]
    {
        app.add_plugins(PluginInspectorPlugin);
        println!("Plugin Inspector enabled - Press Ctrl+Shift+P to open");
    }

    // Add graphics inspector
    app.add_plugins(GraphicsInspectorPlugin);
    println!("Graphics Inspector enabled - Press Ctrl+Shift+G to open");

    // Conditionally add Bevy ECS inspector (debug builds only)
    #[cfg(debug_assertions)]
    {
        app.add_plugins(BevyInspectorPlugin);
        println!("Bevy Inspector enabled - Press Ctrl+Shift+I to open");
    }

    app.run();
}

/// Headless mode state tracker
#[derive(Resource)]
struct HeadlessMode {
    startup_time: std::time::Instant,
    max_wait_secs: f32,
    command_sent: bool,
    initial_sequence: u64,
}

/// System that waits for terminal updates, dumps grid, and exits
fn headless_dump_and_exit(
    mut headless: ResMut<HeadlessMode>,
    reader: Res<SharedMemoryReader>,
    mut app_exit: EventWriter<bevy::app::AppExit>,
) {
    // Get safe state wrapper
    let safe_state = reader.get_safe_state();
    let current_seq = safe_state.sequence();

    // Initialize initial sequence on first run
    if !headless.command_sent {
        headless.initial_sequence = current_seq;
        headless.command_sent = true;
        println!("Initial sequence number: {}", current_seq);
        return;
    }

    // Check if terminal has been updated (sequence changed)
    let has_output = current_seq > headless.initial_sequence;

    // Calculate elapsed time
    let elapsed = headless.startup_time.elapsed().as_secs_f32();

    // Exit conditions:
    // 1. Terminal has output and we've waited a bit for it to stabilize (0.5s)
    // 2. Timeout reached (5 seconds by default)
    if (has_output && elapsed > 0.5) || elapsed > headless.max_wait_secs {
        if has_output {
            println!(
                "Terminal output detected (seq: {} -> {}), dumping grid...",
                headless.initial_sequence, current_seq
            );
        } else {
            println!(
                "Timeout reached after {:.1}s, dumping grid anyway...",
                elapsed
            );
        }

        // Dump terminal grid to stdout
        dump_terminal_grid(&safe_state);

        // Exit the app
        println!("Headless mode complete, exiting.");
        app_exit.send(bevy::app::AppExit::Success);
    }
}

/// Dump the terminal grid to stdout
fn dump_terminal_grid(state: &impl TerminalStateReader) {
    let (cols, rows) = state.dimensions();
    let (cursor_x, cursor_y) = state.cursor_pos();

    println!("=== TERMINAL GRID DUMP ===");
    println!("Dimensions: {}x{}", cols, rows);
    println!("Cursor: ({}, {})", cursor_x, cursor_y);
    println!("Sequence: {}", state.sequence());
    println!("--- GRID CONTENT ---");

    for row in 0..rows {
        for col in 0..cols {
            if let Some(cell) = state.cell(row, col) {
                let ch = if cell.char_codepoint == 0 || cell.char_codepoint == 32 {
                    ' '
                } else {
                    char::from_u32(cell.char_codepoint).unwrap_or('?')
                };
                print!("{}", ch);
            } else {
                print!(" ");
            }
        }
        println!();
    }
    println!("=== END GRID DUMP ===");
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

    println!(
        "Scarab Client Initialized with shared memory reader, IPC, scrollback, and scripting."
    );
}
