//! E2E Test Harness for Scarab Terminal Emulator
//!
//! This module provides the core test harness for end-to-end integration testing.
//! It manages daemon and client processes, shared memory, and IPC communication.

use anyhow::{Context, Result, bail};
use scarab_protocol::{SharedState, Cell, ControlMessage, SHMEM_PATH, SOCKET_PATH, MAX_MESSAGE_SIZE};
use shared_memory::{Shmem, ShmemConf};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::thread;
use std::time::{Duration, Instant};
use tempfile::TempDir;

/// Maximum time to wait for daemon startup
const DAEMON_STARTUP_TIMEOUT: Duration = Duration::from_secs(10);

/// Maximum time to wait for client connection
const CLIENT_CONNECT_TIMEOUT: Duration = Duration::from_secs(5);

/// Poll interval for checking shared memory updates
const POLL_INTERVAL: Duration = Duration::from_millis(50);

/// E2E Test Harness
///
/// Manages the lifecycle of daemon and client processes for end-to-end testing.
/// Provides methods to interact with the terminal through IPC and verify output
/// from shared memory.
pub struct E2ETestHarness {
    daemon: Option<Child>,
    client: Option<Child>,
    shared_memory: Option<Shmem>,
    socket_path: String,
    temp_dir: TempDir,
    daemon_bin: PathBuf,
    client_bin: PathBuf,
}

impl E2ETestHarness {
    /// Create a new test harness
    ///
    /// This will:
    /// 1. Build daemon and client binaries if needed
    /// 2. Spawn the daemon process
    /// 3. Wait for shared memory to be created
    /// 4. Map shared memory for reading
    pub fn new() -> Result<Self> {
        println!("=== Initializing E2E Test Harness ===");

        // Create temporary directory for test artifacts
        let temp_dir = tempfile::tempdir()
            .context("Failed to create temporary directory")?;

        println!("Temp directory: {}", temp_dir.path().display());

        // Get paths to binaries
        let daemon_bin = Self::find_or_build_daemon()?;
        let client_bin = Self::find_or_build_client()?;

        println!("Daemon binary: {}", daemon_bin.display());
        println!("Client binary: {}", client_bin.display());

        // Clean up any existing shared memory or socket from previous tests
        Self::cleanup_resources();

        // Spawn daemon
        println!("Spawning daemon...");
        let daemon = Command::new(&daemon_bin)
            .env("RUST_LOG", "info")
            .env("HOME", temp_dir.path()) // Isolate daemon data
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .context("Failed to spawn daemon process")?;

        // Wait for daemon to initialize
        thread::sleep(Duration::from_millis(500));

        // Wait for shared memory to be created
        let start = Instant::now();
        let shared_memory = loop {
            if start.elapsed() > DAEMON_STARTUP_TIMEOUT {
                bail!("Daemon failed to create shared memory within timeout");
            }

            match ShmemConf::new()
                .size(std::mem::size_of::<SharedState>())
                .os_id(SHMEM_PATH)
                .open()
            {
                Ok(shmem) => {
                    println!("✓ Shared memory opened successfully");
                    break Some(shmem);
                }
                Err(_) => {
                    thread::sleep(Duration::from_millis(100));
                }
            }
        };

        // Wait for socket to be created
        let start = Instant::now();
        while !Path::new(SOCKET_PATH).exists() {
            if start.elapsed() > DAEMON_STARTUP_TIMEOUT {
                bail!("Daemon failed to create socket within timeout");
            }
            thread::sleep(Duration::from_millis(100));
        }

        println!("✓ Socket created at {}", SOCKET_PATH);
        println!("=== Harness initialized successfully ===\n");

        Ok(Self {
            daemon: Some(daemon),
            client: None,
            shared_memory,
            socket_path: SOCKET_PATH.to_string(),
            temp_dir,
            daemon_bin,
            client_bin,
        })
    }

    /// Start the client process
    pub fn start_client(&mut self) -> Result<()> {
        println!("Starting client...");

        let client = Command::new(&self.client_bin)
            .env("RUST_LOG", "info")
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .context("Failed to spawn client process")?;

        self.client = Some(client);

        // Give client time to connect
        thread::sleep(Duration::from_millis(500));

        println!("✓ Client started\n");

        Ok(())
    }

    /// Send input to the daemon via IPC
    pub fn send_input(&self, text: &str) -> Result<()> {
        use std::os::unix::net::UnixStream;

        let mut stream = UnixStream::connect(&self.socket_path)
            .context("Failed to connect to daemon socket")?;

        // Create input message
        let msg = ControlMessage::Input {
            data: text.as_bytes().to_vec(),
        };

        // Serialize with rkyv
        let bytes = rkyv::to_bytes::<_, MAX_MESSAGE_SIZE>(&msg)
            .context("Failed to serialize message")?;

        let len = bytes.len() as u32;

        // Write length prefix
        stream.write_all(&len.to_le_bytes())
            .context("Failed to write message length")?;

        // Write message data
        stream.write_all(&bytes)
            .context("Failed to write message data")?;

        stream.flush()
            .context("Failed to flush stream")?;

        Ok(())
    }

    /// Get the current shared state
    pub fn get_shared_state(&self) -> Result<SharedState> {
        let shmem = self.shared_memory.as_ref()
            .context("Shared memory not initialized")?;

        let ptr = shmem.as_ptr() as *const SharedState;

        // Safe because we control the daemon and know the layout
        let state = unsafe { std::ptr::read_volatile(ptr) };

        Ok(state)
    }

    /// Get output from the terminal grid
    ///
    /// Returns a string containing all visible text on the screen.
    /// Newlines separate rows.
    pub fn get_output(&self, timeout: Duration) -> Result<String> {
        // Wait a moment for output to be rendered
        thread::sleep(timeout);

        let state = self.get_shared_state()?;

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

            // Trim trailing spaces but keep the line
            let trimmed = line.trim_end();
            if !trimmed.is_empty() {
                output.push_str(trimmed);
            }
            output.push('\n');
        }

        Ok(output)
    }

    /// Verify that output contains the expected text
    ///
    /// Polls shared memory until the text appears or timeout is reached.
    pub fn verify_output_contains(&self, expected: &str, timeout: Duration) -> Result<bool> {
        let start = Instant::now();

        while start.elapsed() < timeout {
            let output = self.get_output(Duration::from_millis(10))?;

            if output.contains(expected) {
                println!("✓ Found expected text: '{}'", expected);
                return Ok(true);
            }

            thread::sleep(POLL_INTERVAL);
        }

        println!("✗ Expected text not found: '{}'", expected);
        println!("Current output:\n{}", self.get_output(Duration::from_millis(10))?);

        Ok(false)
    }

    /// Get a specific line from the terminal grid
    pub fn get_line(&self, line_num: usize) -> Result<String> {
        let state = self.get_shared_state()?;

        if line_num >= scarab_protocol::GRID_HEIGHT {
            bail!("Line number {} out of bounds", line_num);
        }

        let mut line = String::new();
        for col in 0..scarab_protocol::GRID_WIDTH {
            let idx = line_num * scarab_protocol::GRID_WIDTH + col;
            if idx < state.cells.len() {
                let cell = state.cells[idx];
                if cell.char_codepoint != 0 {
                    if let Some(ch) = char::from_u32(cell.char_codepoint) {
                        line.push(ch);
                    }
                }
            }
        }

        Ok(line.trim_end().to_string())
    }

    /// Disconnect the client (kill it)
    pub fn disconnect_client(&mut self) -> Result<()> {
        if let Some(mut client) = self.client.take() {
            println!("Disconnecting client...");
            client.kill().context("Failed to kill client process")?;
            client.wait().context("Failed to wait for client")?;
            println!("✓ Client disconnected\n");
        }
        Ok(())
    }

    /// Reconnect the client (spawn a new instance)
    pub fn reconnect_client(&mut self) -> Result<()> {
        self.start_client()
    }

    /// Send a resize command to the daemon
    pub fn resize(&self, cols: u16, rows: u16) -> Result<()> {
        use std::os::unix::net::UnixStream;

        println!("Sending resize: {}x{}", cols, rows);

        let mut stream = UnixStream::connect(&self.socket_path)
            .context("Failed to connect to daemon socket")?;

        let msg = ControlMessage::Resize { cols, rows };

        let bytes = rkyv::to_bytes::<_, MAX_MESSAGE_SIZE>(&msg)
            .context("Failed to serialize resize message")?;

        let len = bytes.len() as u32;

        stream.write_all(&len.to_le_bytes())?;
        stream.write_all(&bytes)?;
        stream.flush()?;

        println!("✓ Resize command sent\n");

        Ok(())
    }

    /// Check if the daemon process is still alive
    pub fn daemon_is_alive(&mut self) -> bool {
        if let Some(ref mut daemon) = self.daemon {
            match daemon.try_wait() {
                Ok(Some(_)) => false, // Process has exited
                Ok(None) => true,     // Process is still running
                Err(_) => false,      // Error checking status
            }
        } else {
            false
        }
    }

    /// Check if the client process is still alive
    pub fn client_is_alive(&mut self) -> bool {
        if let Some(ref mut client) = self.client {
            match client.try_wait() {
                Ok(Some(_)) => false,
                Ok(None) => true,
                Err(_) => false,
            }
        } else {
            false
        }
    }

    /// Cleanup all resources
    pub fn cleanup(&mut self) {
        println!("\n=== Cleaning up test harness ===");

        // Kill client first
        if let Some(mut client) = self.client.take() {
            let _ = client.kill();
            let _ = client.wait();
            println!("✓ Client terminated");
        }

        // Then kill daemon
        if let Some(mut daemon) = self.daemon.take() {
            let _ = daemon.kill();
            let _ = daemon.wait();
            println!("✓ Daemon terminated");
        }

        // Clean up shared memory
        self.shared_memory = None;

        // Clean up resources
        Self::cleanup_resources();

        println!("=== Cleanup complete ===\n");
    }

    /// Clean up shared memory and socket files
    fn cleanup_resources() {
        if Path::new(SOCKET_PATH).exists() {
            let _ = std::fs::remove_file(SOCKET_PATH);
        }

        // Note: shared memory cleanup is OS-specific
        // On Linux, shared memory under /dev/shm persists until explicitly removed
        #[cfg(target_os = "linux")]
        {
            let shm_path = format!("/dev/shm{}", SHMEM_PATH);
            if Path::new(&shm_path).exists() {
                let _ = std::fs::remove_file(&shm_path);
            }
        }
    }

    /// Find or build the daemon binary
    fn find_or_build_daemon() -> Result<PathBuf> {
        // Try to find built binary first
        let workspace_root = Self::find_workspace_root()?;

        let debug_bin = workspace_root.join("target/debug/scarab-daemon");
        let release_bin = workspace_root.join("target/release/scarab-daemon");

        if release_bin.exists() {
            return Ok(release_bin);
        }

        if debug_bin.exists() {
            return Ok(debug_bin);
        }

        // Build if not found
        println!("Daemon binary not found, building...");
        Self::build_daemon(&workspace_root)?;

        if debug_bin.exists() {
            Ok(debug_bin)
        } else {
            bail!("Failed to build daemon binary")
        }
    }

    /// Find or build the client binary
    fn find_or_build_client() -> Result<PathBuf> {
        let workspace_root = Self::find_workspace_root()?;

        let debug_bin = workspace_root.join("target/debug/scarab-client");
        let release_bin = workspace_root.join("target/release/scarab-client");

        if release_bin.exists() {
            return Ok(release_bin);
        }

        if debug_bin.exists() {
            return Ok(debug_bin);
        }

        // Build if not found
        println!("Client binary not found, building...");
        Self::build_client(&workspace_root)?;

        if debug_bin.exists() {
            Ok(debug_bin)
        } else {
            bail!("Failed to build client binary")
        }
    }

    /// Build the daemon binary
    fn build_daemon(workspace_root: &Path) -> Result<()> {
        let status = Command::new("cargo")
            .arg("build")
            .arg("-p")
            .arg("scarab-daemon")
            .current_dir(workspace_root)
            .status()
            .context("Failed to execute cargo build for daemon")?;

        if !status.success() {
            bail!("Failed to build daemon");
        }

        Ok(())
    }

    /// Build the client binary
    fn build_client(workspace_root: &Path) -> Result<()> {
        let status = Command::new("cargo")
            .arg("build")
            .arg("-p")
            .arg("scarab-client")
            .current_dir(workspace_root)
            .status()
            .context("Failed to execute cargo build for client")?;

        if !status.success() {
            bail!("Failed to build client");
        }

        Ok(())
    }

    /// Find the workspace root directory
    fn find_workspace_root() -> Result<PathBuf> {
        let mut current = std::env::current_dir()
            .context("Failed to get current directory")?;

        loop {
            let cargo_toml = current.join("Cargo.toml");
            if cargo_toml.exists() {
                // Check if this is a workspace root
                let contents = std::fs::read_to_string(&cargo_toml)
                    .context("Failed to read Cargo.toml")?;
                if contents.contains("[workspace]") {
                    return Ok(current);
                }
            }

            if !current.pop() {
                bail!("Could not find workspace root");
            }
        }
    }
}

impl Drop for E2ETestHarness {
    fn drop(&mut self) {
        self.cleanup();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_harness_initialization() -> Result<()> {
        let _harness = E2ETestHarness::new()?;

        // Verify daemon is running - if we got here, initialization succeeded
        // The harness Drop will clean up

        Ok(())
    }

    #[test]
    fn test_send_input() -> Result<()> {
        let harness = E2ETestHarness::new()?;

        // Send simple input
        harness.send_input("echo test\n")?;

        // Wait for output
        thread::sleep(Duration::from_millis(500));

        Ok(())
    }
}
