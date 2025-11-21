use portable_pty::{CommandBuilder, NativePtySystem, PtySize, PtySystem};
use anyhow::Result;
use std::io::{Read, Write};
use std::sync::atomic::{AtomicU64, Ordering};
use tokio::sync::mpsc;
use shared_memory::ShmemConf;
use scarab_protocol::{SharedState, Cell, SHMEM_PATH, GRID_WIDTH, GRID_HEIGHT};
use std::sync::Arc;

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

    let mut reader = pair.master.try_clone_reader()?;
    let mut writer = pair.master.take_writer()?;

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

    println!("Daemon initialized. Listening for input...");

    // 3. Main Loop with VTE Parser Integration
    let mut buf = [0u8; 1024];
    loop {
        // In a real implementation, use tokio::select! to handle IPC events too
        match reader.read(&mut buf) {
            Ok(n) if n > 0 => {
                let data = &buf[..n];
                println!("PTY Output: {:?}", String::from_utf8_lossy(data));

                // Update shared memory atomically
                unsafe {
                    let state = &mut *shared_ptr;

                    // TODO: Integrate Alacritty VTE parser here
                    // For now, just mark as dirty and increment sequence
                    state.dirty_flag = 1;

                    // Atomically increment sequence number to signal client
                    let new_seq = sequence_counter.fetch_add(1, Ordering::SeqCst) + 1;
                    state.sequence_number = new_seq;
                }
            }
            Ok(_) => break, // EOF
            Err(e) => {
                eprintln!("PTY Error: {}", e);
                break;
            }
        }
    }

    // Cleanup shared memory
    drop(shmem);
    println!("Daemon shutting down...");

    Ok(())
}
