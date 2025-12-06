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
//!
//! For environments where PTY creation is restricted (userns/sandbox), you can
//! start the daemon separately and use: --use-existing-daemon

use anyhow::{Context, Result};
use clap::Parser;
use scarab_protocol::{ControlMessage, SharedState, SHMEM_PATH, SHMEM_PATH_ENV, SOCKET_PATH, MAX_MESSAGE_SIZE};
use shared_memory::ShmemConf;
use std::io::{BufRead, BufReader};
use std::process::{Child, Command, Stdio};
use std::sync::mpsc;
use std::time::{Duration, Instant};
use tokio::io::AsyncWriteExt;
use tokio::net::UnixStream;
use tokio::time::sleep;

const STARTUP_TIMEOUT: Duration = Duration::from_secs(5);
const OUTPUT_TIMEOUT: Duration = Duration::from_secs(10);
const POLL_INTERVAL: Duration = Duration::from_millis(100);

#[derive(Parser, Debug)]
#[command(name = "smoke_harness")]
#[command(about = "Smoke test harness for Scarab daemon")]
struct Args {
    /// Use an existing daemon instead of spawning a new one.
    /// Useful for sandboxed environments where PTY creation is restricted.
    #[arg(long)]
    use_existing_daemon: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    println!("🧪 Scarab Smoke Test Harness");
    println!("==============================\n");

    // 1. Start daemon process (unless using existing)
    let daemon_handle = if args.use_existing_daemon {
        println!("Using existing daemon (--use-existing-daemon flag set)");
        println!("Make sure scarab-daemon is running separately.\n");
        None
    } else {
        println!("Starting daemon...");
        let (daemon, error_rx) = start_daemon()?;
        Some((daemon, error_rx))
    };

    // RAII guard for cleanup
    let _guard = daemon_handle.as_ref().map(|(d, _)| DaemonGuard(d.id()));

    // 2. Wait for socket to appear, checking for daemon exit
    println!("Waiting for socket: {}", SOCKET_PATH);
    if let Some((ref daemon, ref error_rx)) = daemon_handle {
        wait_for_socket_with_daemon_check(STARTUP_TIMEOUT, daemon.id(), error_rx).await?;
    } else {
        wait_for_socket(STARTUP_TIMEOUT).await?;
    }
    println!("✓ Socket found\n");

    // 3. Connect to daemon
    println!("Connecting to daemon...");
    let mut stream = UnixStream::connect(SOCKET_PATH)
        .await
        .context("Failed to connect to daemon socket")?;
    println!("✓ Connected\n");

    // 4. Open shared memory
    let shmem_path = std::env::var(SHMEM_PATH_ENV).unwrap_or_else(|_| SHMEM_PATH.to_string());
    println!("Opening shared memory: {}", shmem_path);
    let shmem = ShmemConf::new()
        .os_id(&shmem_path)
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

/// Start the daemon as a subprocess, capturing stderr for error detection
fn start_daemon() -> Result<(Child, mpsc::Receiver<String>)> {
    // Channel for daemon stderr messages
    let (error_tx, error_rx) = mpsc::channel();

    // Try to find the daemon binary
    let daemon_path = if std::path::Path::new("target/debug/scarab-daemon").exists() {
        "target/debug/scarab-daemon"
    } else if std::path::Path::new("../../target/debug/scarab-daemon").exists() {
        "../../target/debug/scarab-daemon"
    } else {
        // Fallback: use cargo run
        let mut child = Command::new("cargo")
            .args(["run", "-p", "scarab-daemon", "--quiet"])
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .spawn()
            .context("Failed to spawn daemon via cargo")?;

        // Spawn thread to capture stderr
        if let Some(stderr) = child.stderr.take() {
            spawn_stderr_reader(stderr, error_tx);
        }

        return Ok((child, error_rx));
    };

    let mut child = Command::new(daemon_path)
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .context("Failed to spawn daemon")?;

    // Spawn thread to capture stderr
    if let Some(stderr) = child.stderr.take() {
        spawn_stderr_reader(stderr, error_tx);
    }

    Ok((child, error_rx))
}

/// Spawn a thread to read daemon stderr and forward to channel
fn spawn_stderr_reader(stderr: std::process::ChildStderr, tx: mpsc::Sender<String>) {
    std::thread::spawn(move || {
        let reader = BufReader::new(stderr);
        for line in reader.lines() {
            if let Ok(line) = line {
                // Print to our stderr so user sees it
                eprintln!("[daemon] {}", line);
                // Also send to channel for error detection
                let _ = tx.send(line);
            }
        }
    });
}

/// Wait for socket file to appear, checking daemon status
async fn wait_for_socket_with_daemon_check(
    timeout: Duration,
    daemon_pid: u32,
    error_rx: &mpsc::Receiver<String>,
) -> Result<()> {
    let start = Instant::now();
    let mut collected_errors = Vec::new();

    while start.elapsed() < timeout {
        // Check if daemon has exited
        if !is_process_running(daemon_pid) {
            // Drain any remaining errors
            while let Ok(err) = error_rx.try_recv() {
                collected_errors.push(err);
            }

            // Determine the likely cause
            let error_summary = if collected_errors.iter().any(|e| e.contains("openpty") || e.contains("pty")) {
                "\n💡 PTY creation failed. This typically means:\n\
                   - /dev/ptmx is not accessible (check permissions)\n\
                   - devpts is not mounted (mount -t devpts devpts /dev/pts)\n\
                   - Running in a restricted sandbox/namespace\n\n\
                 For sandboxed environments, you can:\n\
                   1. Start the daemon outside the sandbox\n\
                   2. Use --use-existing-daemon to connect to a running daemon"
            } else if collected_errors.iter().any(|e| e.contains("Permission denied")) {
                "\n💡 Permission denied. Check file/device permissions."
            } else {
                ""
            };

            anyhow::bail!(
                "Daemon exited before socket was ready.\n\nCaptured stderr:\n{}\n{}",
                collected_errors.join("\n"),
                error_summary
            );
        }

        // Collect any errors that have come in
        while let Ok(err) = error_rx.try_recv() {
            collected_errors.push(err);
        }

        if std::path::Path::new(SOCKET_PATH).exists() {
            // Socket exists, but wait a bit more to ensure daemon is listening
            sleep(Duration::from_millis(200)).await;
            return Ok(());
        }
        sleep(Duration::from_millis(100)).await;
    }

    // Timeout - check if daemon died
    if !is_process_running(daemon_pid) {
        while let Ok(err) = error_rx.try_recv() {
            collected_errors.push(err);
        }
        anyhow::bail!(
            "Daemon exited during startup.\n\nCaptured stderr:\n{}",
            collected_errors.join("\n")
        );
    }

    anyhow::bail!(
        "Socket {} did not appear within {} seconds",
        SOCKET_PATH,
        timeout.as_secs()
    )
}

/// Wait for socket file to appear (simple version for --use-existing-daemon)
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
        "Socket {} did not appear within {} seconds.\n\
         Make sure the daemon is running: cargo run -p scarab-daemon",
        SOCKET_PATH,
        timeout.as_secs()
    )
}

/// Check if a process is still running
#[cfg(unix)]
fn is_process_running(pid: u32) -> bool {
    // kill with signal 0 checks if process exists without sending a signal
    unsafe { libc::kill(pid as i32, 0) == 0 }
}

#[cfg(not(unix))]
fn is_process_running(_pid: u32) -> bool {
    // On non-unix, assume process is running (fallback)
    true
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
struct DaemonGuard(u32); // Store PID

impl Drop for DaemonGuard {
    fn drop(&mut self) {
        println!("\nCleaning up daemon process (PID {})...", self.0);
        #[cfg(unix)]
        unsafe {
            libc::kill(self.0 as i32, libc::SIGTERM);
            // Give it a moment to shut down gracefully
            std::thread::sleep(Duration::from_millis(100));
            // Force kill if still running
            libc::kill(self.0 as i32, libc::SIGKILL);
        }
        #[cfg(not(unix))]
        {
            eprintln!("Warning: Cannot kill daemon on non-Unix platform");
        }
    }
}
