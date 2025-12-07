//! Smoke tests using IPC and shared memory for terminal testing.
//!
//! These tests validate that Scarab's terminal emulation works correctly via the actual
//! IPC + shared memory path. This provides real coverage by testing the production pipeline:
//! IPC socket → VTE parsing → Shared memory → TerminalStateReader.
//!
//! ## Test Strategy
//!
//! We:
//! 1. Spawn scarab-daemon (headless)
//! 2. Connect to /tmp/scarab-daemon.sock
//! 3. Send ControlMessage::Input via IPC
//! 4. Read /scarab_shm_v1 via TerminalStateReader
//! 5. Assert sequence number changes and text appears in the grid
//!
//! ## Environment Variable Gate
//!
//! Tests require the daemon binary and IPC support. Set `SCARAB_TEST_RTL=1` to run
//! the full test suite.
//!
//! ## Why This Approach
//!
//! Previous tests wrote to daemon stdin, which the daemon never reads. This gave zero
//! coverage of the IPC/shared-memory path and allowed prompt/input regressions to slip through.
//!
//! This refactor ensures we test the actual code path used in production.

use anyhow::{Context, Result};
use scarab_protocol::{ControlMessage, SharedState, MAX_MESSAGE_SIZE, SHMEM_PATH, SOCKET_PATH};
use shared_memory::{Shmem, ShmemConf};
use std::io::Write;
use std::os::unix::net::UnixStream;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

// =============================================================================
// TEST GATE HELPERS
// =============================================================================

/// Returns true if SCARAB_TEST_RTL=1 is set, enabling the full RTL test suite.
///
/// Tests that require the daemon binary and IPC support should check this before
/// running and return Ok(()) early if not set.
fn should_run_rtl_tests() -> bool {
    std::env::var("SCARAB_TEST_RTL")
        .map(|v| v == "1")
        .unwrap_or(false)
}

/// Maximum time to wait for daemon startup
const DAEMON_STARTUP_TIMEOUT: Duration = Duration::from_secs(5);

/// Helper to find the workspace root
fn find_workspace_root() -> Result<PathBuf> {
    let mut current = std::env::current_dir().context("Failed to get current directory")?;

    loop {
        let cargo_toml = current.join("Cargo.toml");
        if cargo_toml.exists() {
            let contents =
                std::fs::read_to_string(&cargo_toml).context("Failed to read Cargo.toml")?;
            if contents.contains("[workspace]") {
                return Ok(current);
            }
        }

        if !current.pop() {
            anyhow::bail!("Could not find workspace root");
        }
    }
}

/// Helper to build and find the scarab-daemon binary
fn get_daemon_binary() -> Result<PathBuf> {
    let workspace_root = find_workspace_root()?;

    // Check CARGO_TARGET_DIR first
    if let Ok(target_dir) = std::env::var("CARGO_TARGET_DIR") {
        let debug_bin = PathBuf::from(&target_dir).join("debug/scarab-daemon");
        let release_bin = PathBuf::from(&target_dir).join("release/scarab-daemon");
        if release_bin.exists() {
            return Ok(release_bin);
        }
        if debug_bin.exists() {
            return Ok(debug_bin);
        }
    }

    // Check standard target directory
    let debug_bin = workspace_root.join("target/debug/scarab-daemon");
    let release_bin = workspace_root.join("target/release/scarab-daemon");

    if release_bin.exists() {
        return Ok(release_bin);
    }

    if debug_bin.exists() {
        return Ok(debug_bin);
    }

    // Need to build
    println!("scarab-daemon not found, building...");
    let status = std::process::Command::new("cargo")
        .arg("build")
        .arg("-p")
        .arg("scarab-daemon")
        .current_dir(&workspace_root)
        .status()
        .context("Failed to execute cargo build")?;

    if !status.success() {
        anyhow::bail!("Failed to build scarab-daemon");
    }

    // Recheck after build
    if let Ok(target_dir) = std::env::var("CARGO_TARGET_DIR") {
        let debug_bin = PathBuf::from(target_dir).join("debug/scarab-daemon");
        if debug_bin.exists() {
            return Ok(debug_bin);
        }
    }

    if debug_bin.exists() {
        Ok(debug_bin)
    } else {
        anyhow::bail!("Failed to locate scarab-daemon after build")
    }
}

// =============================================================================
// IPC + SHARED MEMORY TEST INFRASTRUCTURE
// =============================================================================

/// Test harness for IPC and shared memory testing
struct DaemonTestHarness {
    daemon: Child,
    socket_stream: UnixStream,
    shared_memory: Shmem,
}

impl DaemonTestHarness {
    /// Create new harness, spawn daemon, connect via IPC, map shared memory
    fn new() -> Result<Self> {
        // Clean up any stale resources
        Self::cleanup_resources();

        let daemon_bin = get_daemon_binary()?;
        println!("Daemon binary: {}", daemon_bin.display());

        // Spawn daemon
        println!("Spawning daemon...");
        let daemon = Command::new(&daemon_bin)
            .env("RUST_LOG", "info")
            .env("SHELL", "/bin/sh")
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .spawn()
            .context("Failed to spawn daemon")?;

        // Wait for daemon to create socket
        let start = Instant::now();
        while !Path::new(SOCKET_PATH).exists() {
            if start.elapsed() > DAEMON_STARTUP_TIMEOUT {
                anyhow::bail!("Daemon failed to create socket within timeout");
            }
            thread::sleep(Duration::from_millis(50));
        }
        println!("Socket created at {}", SOCKET_PATH);

        // Connect to daemon socket
        let socket_stream =
            UnixStream::connect(SOCKET_PATH).context("Failed to connect to daemon socket")?;
        println!("Connected to daemon socket");

        // Wait for shared memory to be created
        let start = Instant::now();
        let shared_memory = loop {
            if start.elapsed() > DAEMON_STARTUP_TIMEOUT {
                anyhow::bail!("Daemon failed to create shared memory within timeout");
            }

            match ShmemConf::new()
                .size(std::mem::size_of::<SharedState>())
                .os_id(SHMEM_PATH)
                .open()
            {
                Ok(shmem) => {
                    println!("Shared memory opened successfully");
                    break shmem;
                }
                Err(_) => {
                    thread::sleep(Duration::from_millis(50));
                }
            }
        };

        // Give daemon a moment to fully initialize
        thread::sleep(Duration::from_millis(500));

        println!("Harness initialized successfully\n");

        Ok(Self {
            daemon,
            socket_stream,
            shared_memory,
        })
    }

    /// Send input via IPC (ControlMessage::Input)
    fn send_input(&mut self, text: &str) -> Result<()> {
        let msg = ControlMessage::Input {
            data: text.as_bytes().to_vec(),
        };

        // Serialize with rkyv
        let bytes =
            rkyv::to_bytes::<_, MAX_MESSAGE_SIZE>(&msg).context("Failed to serialize message")?;

        let len = bytes.len() as u32;

        // Write length prefix (Big Endian)
        self.socket_stream
            .write_all(&len.to_be_bytes())
            .context("Failed to write message length")?;

        // Write message data
        self.socket_stream
            .write_all(&bytes)
            .context("Failed to write message data")?;

        self.socket_stream
            .flush()
            .context("Failed to flush stream")?;

        Ok(())
    }

    /// Read current shared state
    fn get_state(&self) -> &SharedState {
        let ptr = self.shared_memory.as_ptr() as *const SharedState;
        unsafe { &*ptr }
    }

    /// Get current sequence number from shared memory
    fn get_sequence(&self) -> u64 {
        self.get_state().sequence_number
    }

    /// Get output as string from shared memory grid
    fn get_output(&self) -> String {
        let state = self.get_state();
        let mut output = String::new();

        for row in 0..scarab_protocol::GRID_HEIGHT {
            let mut line = String::new();
            for col in 0..scarab_protocol::GRID_WIDTH {
                let idx = row * scarab_protocol::GRID_WIDTH + col;
                if idx < state.cells.len() {
                    let cell = state.cells[idx];
                    if cell.char_codepoint != 0 {
                        if let Some(ch) = char::from_u32(cell.char_codepoint) {
                            line.push(ch);
                        }
                    }
                }
            }

            let trimmed = line.trim_end();
            if !trimmed.is_empty() {
                output.push_str(trimmed);
            }
            output.push('\n');
        }

        output
    }

    /// Wait for sequence number to change
    fn wait_for_sequence_change(&self, old_seq: u64, timeout: Duration) -> Result<bool> {
        let start = Instant::now();
        while start.elapsed() < timeout {
            let current_seq = self.get_sequence();
            if current_seq != old_seq {
                return Ok(true);
            }
            thread::sleep(Duration::from_millis(10));
        }
        Ok(false)
    }

    /// Wait for text to appear in output
    fn wait_for_text(&self, expected: &str, timeout: Duration) -> Result<bool> {
        let start = Instant::now();
        while start.elapsed() < timeout {
            let output = self.get_output();
            if output.contains(expected) {
                return Ok(true);
            }
            thread::sleep(Duration::from_millis(50));
        }
        Ok(false)
    }

    /// Clean up shared memory and socket files
    fn cleanup_resources() {
        if Path::new(SOCKET_PATH).exists() {
            let _ = std::fs::remove_file(SOCKET_PATH);
        }

        #[cfg(target_os = "linux")]
        {
            let shm_path = format!("/dev/shm{}", SHMEM_PATH);
            if Path::new(&shm_path).exists() {
                let _ = std::fs::remove_file(&shm_path);
            }
        }
    }
}

impl Drop for DaemonTestHarness {
    fn drop(&mut self) {
        let _ = self.daemon.kill();
        let _ = self.daemon.wait();
        Self::cleanup_resources();
    }
}

// =============================================================================
// CORE IPC + SHARED MEMORY TESTS
// =============================================================================

/// Test 1: IPC input + shared memory output (echo command)
///
/// Verifies that commands sent via IPC appear in shared memory grid.
/// This tests the IPC socket → VTE parsing → SharedMemory pipeline.
#[test]
fn test_ipc_input_and_shared_memory_output() -> Result<()> {
    if !should_run_rtl_tests() {
        println!("Skipping test (SCARAB_TEST_RTL != 1)");
        return Ok(());
    }

    println!("=== Test: IPC Input + Shared Memory Output ===");

    let mut harness = DaemonTestHarness::new()?;

    let seq_before = harness.get_sequence();
    println!("Sequence before: {}", seq_before);

    // Send command via IPC
    harness.send_input("echo 'Hello from IPC test'\r")?;
    println!("Sent: echo 'Hello from IPC test'");

    // Wait for sequence number to change
    let seq_changed = harness.wait_for_sequence_change(seq_before, Duration::from_secs(3))?;
    assert!(
        seq_changed,
        "Sequence number did not change after sending input"
    );

    let seq_after = harness.get_sequence();
    println!("Sequence after: {}", seq_after);
    assert!(
        seq_after > seq_before,
        "Sequence number should have increased"
    );

    // Wait for text to appear in shared memory
    let text_found = harness.wait_for_text("Hello from IPC test", Duration::from_secs(3))?;
    assert!(text_found, "Expected text not found in shared memory grid");

    let output = harness.get_output();
    println!("Output from shared memory:\n{}", output);

    println!("IPC input + shared memory output working");
    Ok(())
}

/// Test 2: Multi-line output via IPC
///
/// Sends a command that outputs multiple lines and verifies all appear in shared memory.
#[test]
fn test_multiline_output_via_ipc() -> Result<()> {
    if !should_run_rtl_tests() {
        println!("Skipping test (SCARAB_TEST_RTL != 1)");
        return Ok(());
    }

    println!("=== Test: Multi-line Output via IPC ===");

    let mut harness = DaemonTestHarness::new()?;

    // Send multi-line output
    harness.send_input("printf 'Line 1\\nLine 2\\nLine 3\\n'\r")?;
    println!("Sent: multi-line printf");

    // Wait for all text to appear
    let found_line1 = harness.wait_for_text("Line 1", Duration::from_secs(3))?;
    assert!(found_line1, "Line 1 not found in shared memory");

    let output = harness.get_output();
    println!("Output from shared memory:\n{}", output);

    // Verify all lines appear
    assert!(output.contains("Line 1"), "Line 1 not found");
    assert!(output.contains("Line 2"), "Line 2 not found");
    assert!(output.contains("Line 3"), "Line 3 not found");

    println!("Multi-line text rendering correct");
    Ok(())
}

/// Test 3: Prompt and command output detection via IPC
///
/// Tests that we can send a command (ls) via IPC and see output in shared memory.
/// This validates prompt + input + output handling through the full IPC pipeline.
#[test]
fn test_prompt_and_command_output_via_ipc() -> Result<()> {
    if !should_run_rtl_tests() {
        println!("Skipping test (SCARAB_TEST_RTL != 1)");
        return Ok(());
    }

    println!("=== Test: Prompt and Command Output via IPC ===");

    let mut harness = DaemonTestHarness::new()?;

    let seq_before = harness.get_sequence();

    // Send ls command
    harness.send_input("ls /tmp\r")?;
    println!("Sent: ls /tmp");

    // Wait for sequence to change (command executed)
    let seq_changed = harness.wait_for_sequence_change(seq_before, Duration::from_secs(3))?;
    assert!(seq_changed, "Sequence did not change after ls command");

    // Wait for output to appear (ls should output something or at least return to prompt)
    thread::sleep(Duration::from_millis(500));

    let output = harness.get_output();
    println!("Output from shared memory:\n{}", output);

    // The output should contain either files or be empty but have a prompt
    // We just verify that sequence changed and we got some output
    assert!(
        !output.trim().is_empty(),
        "Output should not be empty after ls command"
    );

    println!("Prompt + command output detection working");
    Ok(())
}

/// Test 4: Sequence number increments on updates
///
/// Validates that the sequence number in shared memory increments correctly.
#[test]
fn test_sequence_number_increments() -> Result<()> {
    if !should_run_rtl_tests() {
        println!("Skipping test (SCARAB_TEST_RTL != 1)");
        return Ok(());
    }

    println!("=== Test: Sequence Number Increments ===");

    let mut harness = DaemonTestHarness::new()?;

    let seq1 = harness.get_sequence();
    println!("Initial sequence: {}", seq1);

    // Send first command
    harness.send_input("echo 'test1'\r")?;
    harness.wait_for_sequence_change(seq1, Duration::from_secs(2))?;
    let seq2 = harness.get_sequence();
    println!("After first command: {}", seq2);
    assert!(seq2 > seq1, "Sequence should increase after first command");

    // Send second command
    harness.send_input("echo 'test2'\r")?;
    harness.wait_for_sequence_change(seq2, Duration::from_secs(2))?;
    let seq3 = harness.get_sequence();
    println!("After second command: {}", seq3);
    assert!(seq3 > seq2, "Sequence should increase after second command");

    println!("Sequence number increments correctly");
    Ok(())
}

/// Test 5: Multiple commands in sequence via IPC
///
/// Verifies that multiple commands can be executed and their output correctly captured.
#[test]
fn test_multiple_commands_via_ipc() -> Result<()> {
    if !should_run_rtl_tests() {
        println!("Skipping test (SCARAB_TEST_RTL != 1)");
        return Ok(());
    }

    println!("=== Test: Multiple Commands via IPC ===");

    let mut harness = DaemonTestHarness::new()?;

    // Command 1
    harness.send_input("echo 'Command 1'\r")?;
    let found1 = harness.wait_for_text("Command 1", Duration::from_secs(2))?;
    assert!(found1, "Command 1 output not found");
    println!("Command 1 executed");

    // Command 2
    harness.send_input("echo 'Command 2'\r")?;
    let found2 = harness.wait_for_text("Command 2", Duration::from_secs(2))?;
    assert!(found2, "Command 2 output not found");
    println!("Command 2 executed");

    // Command 3
    harness.send_input("echo 'Command 3'\r")?;
    let found3 = harness.wait_for_text("Command 3", Duration::from_secs(2))?;
    assert!(found3, "Command 3 output not found");
    println!("Command 3 executed");

    let output = harness.get_output();
    println!("Final output:\n{}", output);

    println!("Multiple commands sequence works via IPC");
    Ok(())
}
