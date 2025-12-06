use anyhow::Result;
use portable_pty::PtySize;
use scarab_config::ConfigLoader;
use scarab_protocol::{SharedImageBuffer, SharedImagePlacement, SharedState, IMAGE_SHMEM_PATH, MAX_IMAGES, SHMEM_PATH};
use shared_memory::ShmemConf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tokio::sync::mpsc;

use scarab_daemon::ipc::{ClientRegistry, IpcServer, PtyHandle};
use scarab_daemon::orchestrator::PaneOrchestrator;
use scarab_daemon::plugin_manager::PluginManager;
use scarab_daemon::session::SessionManager;
use scarab_daemon::vte::TerminalState;

use scarab_plugin_api::context::PluginSharedState;
use scarab_plugin_api::PluginContext;

#[cfg(test)]
mod tests;

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    println!("Starting Scarab Daemon...");

    // 0. Load Configuration (Fusabi-based)
    let home_dir = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    let fusabi_config_path = std::path::PathBuf::from(&home_dir)
        .join(".config/scarab/config.fsx");
    let toml_config_path = std::path::PathBuf::from(&home_dir)
        .join(".config/scarab/config.toml");

    let config = if fusabi_config_path.exists() {
        println!("Loading Fusabi config from: {}", fusabi_config_path.display());
        scarab_config::FusabiConfigLoader::from_file(&fusabi_config_path)?
    } else if toml_config_path.exists() {
        println!("⚠️  Loading legacy TOML config from: {}", toml_config_path.display());
        println!("💡 Consider migrating to Fusabi config: {}", fusabi_config_path.display());
        ConfigLoader::from_file(&toml_config_path)?
    } else {
        println!("No config found (tried .fsx and .toml), using defaults");
        println!("Create {} to customize your terminal", fusabi_config_path.display());
        scarab_config::ScarabConfig::default()
    };

    // Apply environment variable overrides to telemetry config
    let telemetry = config.telemetry.from_env();

    if telemetry.is_enabled() {
        log::info!("Telemetry enabled: fps={}, seq={}, dirty={}, panes={}",
            telemetry.fps_log_interval_secs,
            telemetry.log_sequence_changes,
            telemetry.log_dirty_regions,
            telemetry.log_pane_events
        );
    }

    // 1. Initialize Session Manager
    let db_path = std::path::PathBuf::from(&home_dir).join(".local/share/scarab/sessions.db");

    let session_manager = std::sync::Arc::new(SessionManager::new(db_path)?);

    // Restore sessions from previous daemon runs
    session_manager.restore_sessions(
        &config.terminal.default_shell,
        config.terminal.columns,
        config.terminal.rows,
    )?;
    println!(
        "Session Manager: Active ({} sessions)",
        session_manager.session_count()
    );

    // Create default session if none exist
    if session_manager.session_count() == 0 {
        let cols = config.terminal.columns;
        let rows = config.terminal.rows;
        let session_id = session_manager.create_session("default".to_string(), cols, rows)?;
        println!(
            "Created default session: {} ({}x{})",
            session_id, cols, rows
        );
    }

    // PTY is now managed per-pane by SessionManager
    // The active pane's PTY master is accessed via session_manager.get_default_session()
    println!(
        "Terminal configuration: {}x{} (shell: {})",
        config.terminal.columns, config.terminal.rows, config.terminal.default_shell
    );

    // 2. Initialize Shared Memory
    // Try to create new shared memory, or open existing if it already exists
    let shmem = match ShmemConf::new()
        .size(std::mem::size_of::<SharedState>())
        .os_id(SHMEM_PATH)
        .create()
    {
        Ok(shmem) => {
            println!("Created shared memory at: {}", SHMEM_PATH);
            shmem
        }
        Err(_) => {
            // Shared memory already exists, try to open it
            println!("Shared memory already exists, attempting to open...");
            match ShmemConf::new().os_id(SHMEM_PATH).open() {
                Ok(shmem) => {
                    println!("Opened existing shared memory at: {}", SHMEM_PATH);
                    shmem
                }
                Err(e) => {
                    eprintln!("Failed to open existing shared memory: {}", e);
                    eprintln!("Try cleaning up with: ipcrm -M $(ipcs -m | grep scarab | awk '{{print $1}}')");
                    return Err(e.into());
                }
            }
        }
    };

    // Initialize shared state with zeroed memory
    let shared_ptr = shmem.as_ptr() as *mut SharedState;
    unsafe {
        std::ptr::write_bytes(shared_ptr, 0, 1);
    }

    let sequence_counter = Arc::new(AtomicU64::new(0));

    // Initialize SharedImageBuffer for iTerm2 image protocol
    let image_shmem = match ShmemConf::new()
        .size(std::mem::size_of::<SharedImageBuffer>())
        .os_id(IMAGE_SHMEM_PATH)
        .create()
    {
        Ok(shmem) => {
            println!("Created image shared memory at: {} ({} bytes)",
                IMAGE_SHMEM_PATH, std::mem::size_of::<SharedImageBuffer>());
            shmem
        }
        Err(_) => {
            println!("Image shared memory already exists, attempting to open...");
            match ShmemConf::new().os_id(IMAGE_SHMEM_PATH).open() {
                Ok(shmem) => {
                    println!("Opened existing image shared memory at: {}", IMAGE_SHMEM_PATH);
                    shmem
                }
                Err(e) => {
                    eprintln!("Failed to open existing image shared memory: {}", e);
                    eprintln!("Try cleaning up with: ipcrm -M $(ipcs -m | grep scarab | awk '{{print $1}}')");
                    return Err(e.into());
                }
            }
        }
    };

    // Initialize image buffer with zeroed memory
    let image_ptr = image_shmem.as_ptr() as *mut SharedImageBuffer;
    unsafe {
        std::ptr::write_bytes(image_ptr, 0, 1);
    }

    // VTE parser is now per-pane (inside TerminalState owned by each Pane)
    // The active pane's terminal state is blitted to shared memory
    println!("VTE Parser: Per-pane (multiplexing enabled)");
    println!("Scrollback buffer: 10,000 lines per pane");
    println!("Image support: iTerm2 protocol (max {} images)", MAX_IMAGES);

    // 3. Setup IPC Control Channel with channels for thread safety
    let (resize_tx, mut resize_rx) = mpsc::channel::<PtySize>(32);
    let (input_tx, mut input_rx) = mpsc::unbounded_channel::<Vec<u8>>();
    let pty_handle = PtyHandle::new(input_tx, resize_tx);

    let client_registry = ClientRegistry::new();

    // Initialize Plugin Manager
    let plugin_state = Arc::new(parking_lot::Mutex::new(PluginSharedState::new(
        config.terminal.columns,
        config.terminal.rows,
    )));
    let plugin_ctx = Arc::new(PluginContext::new(
        Default::default(),
        plugin_state.clone(),
        "daemon",
    ));
    let mut plugin_manager = PluginManager::new(plugin_ctx, client_registry.clone());

    if let Err(e) = plugin_manager
        .register_plugin(Box::new(scarab_nav::NavigationPlugin::new()))
        .await
    {
        eprintln!("Failed to register NavigationPlugin: {}", e);
    }

    // Register Palette Plugin
    if let Err(e) = plugin_manager
        .register_plugin(Box::new(scarab_palette::PalettePlugin::new()))
        .await
    {
        eprintln!("Failed to register PalettePlugin: {}", e);
    }

    // Register Session Plugin
    if let Err(e) = plugin_manager
        .register_plugin(Box::new(scarab_session::SessionPlugin::new()))
        .await
    {
        eprintln!("Failed to register SessionPlugin: {}", e);
    }

    // Discover and load plugins
    if let Err(e) = plugin_manager.discover_and_load().await {
        eprintln!("Failed to load plugins: {}", e);
    }

    let plugin_manager = Arc::new(tokio::sync::Mutex::new(plugin_manager));

    // Create Pane Orchestrator early so we can pass its command sender to IPC
    let orchestrator = PaneOrchestrator::new(session_manager.clone(), telemetry.log_pane_events);
    let orchestrator_tx = orchestrator.command_sender();

    let ipc_server = IpcServer::new(
        pty_handle.clone(),
        session_manager.clone(),
        client_registry.clone(),
        plugin_manager.clone(),
        orchestrator_tx,
    )
    .await?;

    // Spawn IPC server task
    tokio::spawn(async move {
        if let Err(e) = ipc_server.accept_loop().await {
            eprintln!("IPC server error: {}", e);
        }
    });

    // Spawn PTY writer task to handle input from IPC
    // Routes input to the active pane's PTY
    let sm_writer = session_manager.clone();
    let pm_input = plugin_manager.clone();
    tokio::spawn(async move {
        use std::io::Write;
        while let Some(data) = input_rx.recv().await {
            // Dispatch input to plugins
            let processed_data = {
                let mut pm = pm_input.lock().await;
                match pm.dispatch_input(&data).await {
                    Ok(d) => d,
                    Err(e) => {
                        eprintln!("Plugin input error: {}", e);
                        data
                    }
                }
            };

            if processed_data.is_empty() {
                continue; // Input consumed by plugin
            }

            // Route input to the active pane's PTY writer
            if let Some(session) = sm_writer.get_default_session() {
                if let Some(writer_arc) = session.get_active_pty_writer() {
                    let mut writer_lock = writer_arc.lock().unwrap();
                    if let Some(ref mut writer) = *writer_lock {
                        if let Err(e) = writer.write_all(&processed_data) {
                            eprintln!("PTY write error: {}", e);
                            continue;
                        }
                        if let Err(e) = writer.flush() {
                            eprintln!("PTY flush error: {}", e);
                            continue;
                        }
                    }
                }
            }
        }
    });

    println!("Daemon initialized. Listening for input...");

    // 4. Start the Pane Orchestrator (already created above, now run it)
    // This spawns parallel reader tasks for all panes
    tokio::spawn(async move {
        orchestrator.run().await;
    });

    println!("Pane Orchestrator: Active (parallel PTY reading)");

    // 5. Compositor Loop with Telemetry
    // Blits the active pane's grid to SharedState at ~60fps
    // PTY reading is handled by the orchestrator in parallel
    let mut last_sequence = 0u64;
    let compositor_interval = tokio::time::Duration::from_millis(16); // ~60fps

    // FPS tracking
    let mut fps_tracker = if telemetry.fps_log_interval_secs > 0 {
        Some(FpsTracker::new(telemetry.fps_log_interval_secs))
    } else {
        None
    };

    loop {
        tokio::select! {
            // Compositor tick - blit active pane to shared memory
            _ = tokio::time::sleep(compositor_interval) => {
                // Update FPS tracker
                if let Some(ref mut tracker) = fps_tracker {
                    tracker.tick();
                }

                // Get the active pane from session manager
                if let Some(session) = session_manager.get_default_session() {
                    if let Some(active_pane) = session.get_active_pane() {
                        let terminal_state_arc = active_pane.terminal_state();
                        let terminal_state = terminal_state_arc.read();

                        // Always blit to shared memory
                        // The sequence check was broken: it compared the sequence counter
                        // to itself, since the counter is only incremented in blit_to_shm().
                        // The orchestrator updates terminal_state but doesn't signal this.
                        // Blitting every frame is cheap (memcpy) and ensures the client
                        // always sees the latest content.
                        terminal_state.blit_to_shm(shared_ptr, &sequence_counter);

                        // Blit images to SharedImageBuffer
                        blit_images_to_shm(&terminal_state, image_ptr);

                        let new_seq = sequence_counter.load(Ordering::SeqCst);
                        if telemetry.log_sequence_changes && new_seq != last_sequence {
                            log::debug!("Sequence: {} -> {}", last_sequence, new_seq);
                        }
                        last_sequence = new_seq;
                    }
                }
            }

            // Handle resize events from IPC
            Some(pty_size) = resize_rx.recv() => {
                println!("Resizing active pane to {}x{}", pty_size.cols, pty_size.rows);

                // Resize the active pane (both PTY and terminal state)
                if let Some(session) = session_manager.get_default_session() {
                    if let Some(active_pane) = session.get_active_pane() {
                        if let Err(e) = active_pane.resize(pty_size.cols, pty_size.rows) {
                            eprintln!("Failed to resize pane: {}", e);
                        }

                        // Force blit after resize
                        let terminal_state_arc = active_pane.terminal_state();
                        let terminal_state = terminal_state_arc.read();
                        terminal_state.blit_to_shm(shared_ptr, &sequence_counter);

                        // Blit images after resize
                        blit_images_to_shm(&terminal_state, image_ptr);

                        last_sequence = sequence_counter.load(Ordering::SeqCst);
                    }
                }
            }
        }
    }

    // Cleanup shared memory
    #[allow(unreachable_code)]
    {
        drop(shmem);
        drop(image_shmem);
        println!("Daemon shutting down...");
        Ok(())
    }
}

/// Blit images from TerminalState to SharedImageBuffer
///
/// This copies image placements and blob data from the daemon's
/// per-pane image state to shared memory for client rendering.
fn blit_images_to_shm(state: &TerminalState, image_ptr: *mut SharedImageBuffer) {
    use scarab_protocol::IMAGE_BUFFER_SIZE;

    unsafe {
        let image_buffer = &mut *image_ptr;

        // Reset buffer
        image_buffer.count = 0;
        image_buffer.next_blob_offset = 0;

        for placement in state.image_placements() {
            if image_buffer.count as usize >= MAX_IMAGES {
                log::warn!("Image buffer full, skipping remaining images");
                break;
            }

            // Check blob size
            let blob_offset = image_buffer.next_blob_offset;
            let blob_size = placement.data.len() as u32;

            // Check if fits in buffer
            if (blob_offset + blob_size) as usize > IMAGE_BUFFER_SIZE {
                log::warn!("Image {} too large for buffer ({}+{} > {}), skipping",
                    placement.id, blob_offset, blob_size, IMAGE_BUFFER_SIZE);
                break; // Can't fit, stop adding images
            }

            // Copy blob data to circular buffer
            let start = blob_offset as usize;
            let end = (blob_offset + blob_size) as usize;
            image_buffer.blob_data[start..end].copy_from_slice(&placement.data);

            // Add placement metadata
            let idx = image_buffer.count as usize;
            image_buffer.placements[idx] = SharedImagePlacement {
                image_id: placement.id,
                x: placement.x,
                y: placement.y,
                width_cells: placement.width_cells,
                height_cells: placement.height_cells,
                pixel_width: placement.pixel_width,
                pixel_height: placement.pixel_height,
                blob_offset,
                blob_size,
                format: placement.format,
                flags: 1, // Valid bit set
                _padding: [0; 6],
            };

            image_buffer.count += 1;
            image_buffer.next_blob_offset = blob_offset + blob_size;
        }

        // Increment sequence number to signal client
        image_buffer.sequence_number += 1;

        if image_buffer.count > 0 {
            log::debug!("Blitted {} images to shared memory (sequence: {})",
                image_buffer.count, image_buffer.sequence_number);
        }
    }
}

/// FPS tracking for compositor performance monitoring
struct FpsTracker {
    /// Number of frames since last log
    frame_count: u64,
    /// Last log time
    last_log: std::time::Instant,
    /// Log interval in seconds
    log_interval_secs: u64,
}

impl FpsTracker {
    fn new(log_interval_secs: u64) -> Self {
        Self {
            frame_count: 0,
            last_log: std::time::Instant::now(),
            log_interval_secs,
        }
    }

    fn tick(&mut self) {
        self.frame_count += 1;

        let elapsed = self.last_log.elapsed();
        if elapsed.as_secs() >= self.log_interval_secs {
            let fps = self.frame_count as f64 / elapsed.as_secs_f64();
            log::info!(
                "Compositor: {:.1} fps (avg over {}s), {} frames",
                fps,
                elapsed.as_secs(),
                self.frame_count
            );

            self.frame_count = 0;
            self.last_log = std::time::Instant::now();
        }
    }
}
