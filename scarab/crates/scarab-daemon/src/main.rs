use portable_pty::{CommandBuilder, NativePtySystem, PtySize, PtySystem};
use anyhow::Result;
use std::io::Read;
use std::sync::atomic::AtomicU64;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;
use shared_memory::ShmemConf;
use scarab_protocol::{SharedState, SHMEM_PATH};

mod ipc;
mod vte;
use ipc::{IpcServer, PtyHandle};

#[cfg(test)]
mod tests;

#[tokio::main]
async fn main() -> Result<()> {
    println!("Starting Scarab Daemon...");

    // 1. Setup PTY System
    let pty_system = NativePtySystem::default();
    let pair = pty_system.openpty(PtySize {
        rows: 24,
        cols: 80,
        pixel_width: 0,
        pixel_height: 0,
    })?;

    let cmd = CommandBuilder::new("bash");
    let _child = pair.slave.spawn_command(cmd)?;

    // Important: Release slave handle in parent process
    drop(pair.slave);

    let reader = pair.master.try_clone_reader()?;
    let reader = Arc::new(Mutex::new(reader));
    let writer = pair.master.take_writer()?;

    // 2. Initialize Shared Memory
    let shmem = ShmemConf::new()
        .size(std::mem::size_of::<SharedState>())
        .os_id(SHMEM_PATH)
        .create()?;

    println!("Created shared memory at: {}", SHMEM_PATH);

    // Initialize shared state with zeroed memory
    let shared_ptr = shmem.as_ptr() as *mut SharedState;
    unsafe {
        std::ptr::write_bytes(shared_ptr, 0, 1);
    }

    let sequence_counter = Arc::new(AtomicU64::new(0));

    // Initialize VTE parser
    let mut terminal_state = vte::TerminalState::new(shared_ptr, sequence_counter.clone());
    println!("VTE Parser: Active");
    println!("Scrollback buffer: 10,000 lines");

    // 3. Setup IPC Control Channel with channels for thread safety
    let (resize_tx, mut resize_rx) = mpsc::channel::<PtySize>(32);
    let (input_tx, mut input_rx) = mpsc::unbounded_channel::<Vec<u8>>();
    let pty_handle = PtyHandle::new(input_tx, resize_tx);

    let ipc_server = IpcServer::new(pty_handle.clone()).await?;

    // Spawn IPC server task
    tokio::spawn(async move {
        if let Err(e) = ipc_server.accept_loop().await {
            eprintln!("IPC server error: {}", e);
        }
    });

    // Spawn PTY writer task to handle input from IPC
    let mut writer = writer;
    tokio::spawn(async move {
        use std::io::Write;
        while let Some(data) = input_rx.recv().await {
            if let Err(e) = writer.write_all(&data) {
                eprintln!("PTY write error: {}", e);
                break;
            }
            if let Err(e) = writer.flush() {
                eprintln!("PTY flush error: {}", e);
                break;
            }
        }
    });

    println!("Daemon initialized. Listening for input...");

    // 4. Main Loop with PTY reading and resize handling
    loop {
        tokio::select! {
            // Handle PTY output
            read_result = tokio::task::spawn_blocking({
                let reader_clone = Arc::clone(&reader);
                move || {
                    let mut buf = [0u8; 4096];
                    let mut reader_lock = reader_clone.lock().unwrap();
                    reader_lock.read(&mut buf).map(|n| (n, buf))
                }
            }) => {
                match read_result? {
                    Ok((n, buf)) if n > 0 => {
                        let data = &buf[..n];

                        // Debug output (can be disabled in production)
                        if cfg!(debug_assertions) {
                            print!("{}", String::from_utf8_lossy(data));
                        }

                        // Process output through VTE parser
                        // This will:
                        // 1. Parse ANSI escape sequences
                        // 2. Update grid cells with proper colors and attributes
                        // 3. Handle cursor positioning
                        // 4. Manage scrollback buffer
                        // 5. Atomically update shared state
                        terminal_state.process_output(data);
                    }
                    Ok(_) => break, // EOF
                    Err(e) => {
                        eprintln!("PTY Error: {}", e);
                        break;
                    }
                }
            }

            // Handle resize events from IPC
            Some(pty_size) = resize_rx.recv() => {
                println!("Resizing PTY to {}x{}", pty_size.cols, pty_size.rows);
                if let Err(e) = pair.master.resize(pty_size) {
                    eprintln!("Failed to resize PTY: {}", e);
                }
            }
        }
    }

    // Cleanup shared memory
    drop(shmem);
    println!("Daemon shutting down...");

    Ok(())
}
