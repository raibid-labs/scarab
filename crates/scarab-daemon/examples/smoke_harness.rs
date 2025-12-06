//! Smoke test harness for Scarab daemon
//!
//! This binary verifies the core loop works end-to-end:
//! 1. Start scarab-daemon
//! 2. Send "ls\n" via IPC
//! 3. Read shared memory and verify output appears
//! 4. Exit 0 on success, 1 on failure
//!
//! Run with: cargo run --example smoke_harness
//! Or via just: just smoke

use anyhow::{Context, Result};
use scarab_protocol::{ControlMessage, SharedState, SHMEM_PATH, SOCKET_PATH, MAX_MESSAGE_SIZE};
use shared_memory::ShmemConf;
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};
use tokio::io::AsyncWriteExt;
use tokio::net::UnixStream;
use tokio::time::sleep;

const STARTUP_TIMEOUT: Duration = Duration::from_secs(5);
const OUTPUT_TIMEOUT: Duration = Duration::from_secs(10);
const POLL_INTERVAL: Duration = Duration::from_millis(100);

#[tokio::main]
async fn main() -> Result<()> {
    println!("🧪 Scarab Smoke Test Harness");
    println!("==============================\n");

    // 1. Start daemon process
    println!("Starting daemon...");
    let mut daemon = start_daemon()?;

    // Ensure daemon is killed on exit
    let _guard = DaemonGuard(&mut daemon);

    // 2. Wait for socket to appear
    println!("Waiting for socket: {}", SOCKET_PATH);
    wait_for_socket(STARTUP_TIMEOUT).await?;
    println!("✓ Socket found\n");

    // 3. Connect to daemon
    println!("Connecting to daemon...");
    let mut stream = UnixStream::connect(SOCKET_PATH)
        .await
        .context("Failed to connect to daemon socket")?;
    println!("✓ Connected\n");

    // 4. Open shared memory
    println!("Opening shared memory: {}", SHMEM_PATH);
    let shmem = ShmemConf::new()
        .os_id(SHMEM_PATH)
        .open()
        .context("Failed to open shared memory")?;
    let shared_ptr = shmem.as_ptr() as *const SharedState;
    println!("✓ Shared memory opened\n");

    // 5. Record initial sequence number
    let initial_seq = unsafe { (*shared_ptr).sequence_number };
    println!("Initial sequence number: {}", initial_seq);

    // 6. Send "ls\n" command
    println!("\nSending 'ls' command...");
    send_input(&mut stream, "ls\n").await?;
    println!("✓ Command sent\n");

    // 7. Poll shared memory until output appears or timeout
    println!("Polling shared memory for output...");
    let start = Instant::now();
    let mut last_seq = initial_seq;
    let mut found_output = false;

    while start.elapsed() < OUTPUT_TIMEOUT {
        sleep(POLL_INTERVAL).await;

        let current_seq = unsafe { (*shared_ptr).sequence_number };

        if current_seq != last_seq {
            println!("  Sequence changed: {} -> {}", last_seq, current_seq);
            last_seq = current_seq;

            // Check grid contents
            let grid_text = extract_grid_text(shared_ptr);

            // Look for common indicators of successful ls output:
            // - Non-empty content (beyond just prompts)
            // - Multiple lines with content
            if !grid_text.trim().is_empty() {
                println!("\n📋 Grid contents:");
                println!("{}", "-".repeat(60));
                println!("{}", grid_text);
                println!("{}", "-".repeat(60));

                // Check if we have what looks like ls output
                // This is heuristic: we look for multiple non-empty lines
                let non_empty_lines: Vec<_> = grid_text
                    .lines()
                    .filter(|line| !line.trim().is_empty())
                    .collect();

                if non_empty_lines.len() >= 2 {
                    println!("\n✅ Found output with {} non-empty lines", non_empty_lines.len());
                    found_output = true;
                    break;
                }
            }
        }
    }

    if !found_output {
        eprintln!("\n❌ FAILED: No output detected after {} seconds", OUTPUT_TIMEOUT.as_secs());
        eprintln!("Final sequence number: {}", last_seq);
        eprintln!("Expected: sequence changes and text in grid");
        std::process::exit(1);
    }

    println!("\n✅ SMOKE TEST PASSED");
    println!("   - Daemon started successfully");
    println!("   - Socket connection established");
    println!("   - Shared memory accessible");
    println!("   - Input sent via IPC");
    println!("   - Output appeared in shared memory");

    Ok(())
}

/// Start the daemon as a subprocess
fn start_daemon() -> Result<Child> {
    // Try to find the daemon binary
    // First check if we're in the workspace
    let daemon_path = if std::path::Path::new("target/debug/scarab-daemon").exists() {
        "target/debug/scarab-daemon"
    } else if std::path::Path::new("../../target/debug/scarab-daemon").exists() {
        "../../target/debug/scarab-daemon"
    } else {
        // Fallback: use cargo run
        return Command::new("cargo")
            .args(&["run", "-p", "scarab-daemon", "--quiet"])
            .stdout(Stdio::null())
            .stderr(Stdio::inherit())
            .spawn()
            .context("Failed to spawn daemon via cargo");
    };

    Command::new(daemon_path)
        .stdout(Stdio::null())
        .stderr(Stdio::inherit())
        .spawn()
        .context("Failed to spawn daemon")
}

/// Wait for socket file to appear
async fn wait_for_socket(timeout: Duration) -> Result<()> {
    let start = Instant::now();

    while start.elapsed() < timeout {
        if std::path::Path::new(SOCKET_PATH).exists() {
            // Socket exists, but wait a bit more to ensure daemon is listening
            sleep(Duration::from_millis(200)).await;
            return Ok(());
        }
        sleep(Duration::from_millis(100)).await;
    }

    anyhow::bail!(
        "Socket {} did not appear within {} seconds",
        SOCKET_PATH,
        timeout.as_secs()
    )
}

/// Send input command to daemon via IPC
async fn send_input(stream: &mut UnixStream, input: &str) -> Result<()> {
    let msg = ControlMessage::Input {
        data: input.as_bytes().to_vec(),
    };

    // Serialize with rkyv
    let bytes = rkyv::to_bytes::<_, MAX_MESSAGE_SIZE>(&msg)
        .context("Failed to serialize ControlMessage")?;
    let len = bytes.len() as u32;

    // Send length prefix + message
    stream.write_u32(len).await?;
    stream.write_all(&bytes).await?;
    stream.flush().await?;

    Ok(())
}

/// Extract text from shared memory grid
fn extract_grid_text(shared_ptr: *const SharedState) -> String {
    use scarab_protocol::{GRID_WIDTH, GRID_HEIGHT};

    let mut result = String::new();

    unsafe {
        let state = &*shared_ptr;

        for row in 0..GRID_HEIGHT {
            let mut line = String::new();
            let mut has_content = false;

            for col in 0..GRID_WIDTH {
                let idx = row * GRID_WIDTH + col;
                let cell = &state.cells[idx];

                // Convert codepoint to char
                if let Some(ch) = char::from_u32(cell.char_codepoint) {
                    if ch != ' ' || has_content {
                        line.push(ch);
                        if ch != ' ' {
                            has_content = true;
                        }
                    }
                } else {
                    line.push(' ');
                }
            }

            // Only include lines with content (trim trailing spaces)
            let trimmed = line.trim_end();
            if !trimmed.is_empty() || has_content {
                result.push_str(trimmed);
                result.push('\n');
            }
        }
    }

    result
}

/// RAII guard to kill daemon process on drop
struct DaemonGuard<'a>(&'a mut Child);

impl<'a> Drop for DaemonGuard<'a> {
    fn drop(&mut self) {
        println!("\nCleaning up daemon process...");
        if let Err(e) = self.0.kill() {
            eprintln!("Warning: Failed to kill daemon: {}", e);
        }
        if let Err(e) = self.0.wait() {
            eprintln!("Warning: Failed to wait for daemon: {}", e);
        }
    }
}
