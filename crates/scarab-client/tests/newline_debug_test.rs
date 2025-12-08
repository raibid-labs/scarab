//! Debug test for the newline issue after first command.
//!
//! This test types characters, presses Enter, then types more characters
//! to see if newlines are being inserted unexpectedly.

use anyhow::{Context, Result};
use scarab_protocol::{ControlMessage, SharedState, MAX_MESSAGE_SIZE, SHMEM_PATH, SHMEM_PATH_ENV, IMAGE_SHMEM_PATH_ENV, SOCKET_PATH};
use shared_memory::{Shmem, ShmemConf};
use std::io::Write;
use std::os::unix::net::UnixStream;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

const DAEMON_STARTUP_TIMEOUT: Duration = Duration::from_secs(5);

/// Test-specific shared memory paths to avoid conflicts with user config
const TEST_SHMEM_PATH: &str = "/scarab_test_newline_shm";
const TEST_IMAGE_SHMEM_PATH: &str = "/scarab_test_newline_img";

fn should_run_rtl_tests() -> bool {
    std::env::var("SCARAB_TEST_RTL")
        .map(|v| v == "1")
        .unwrap_or(false)
}

fn find_workspace_root() -> Result<PathBuf> {
    let mut current = std::env::current_dir().context("Failed to get current directory")?;
    loop {
        let cargo_toml = current.join("Cargo.toml");
        if cargo_toml.exists() {
            let contents = std::fs::read_to_string(&cargo_toml).context("Failed to read Cargo.toml")?;
            if contents.contains("[workspace]") {
                return Ok(current);
            }
        }
        if !current.pop() {
            anyhow::bail!("Could not find workspace root");
        }
    }
}

fn get_daemon_binary() -> Result<PathBuf> {
    // Check CARGO_TARGET_DIR first (used by cargo config)
    if let Ok(target_dir) = std::env::var("CARGO_TARGET_DIR") {
        let release_bin = PathBuf::from(&target_dir).join("release/scarab-daemon");
        let debug_bin = PathBuf::from(&target_dir).join("debug/scarab-daemon");
        if release_bin.exists() {
            return Ok(release_bin);
        }
        if debug_bin.exists() {
            return Ok(debug_bin);
        }
    }

    let workspace_root = find_workspace_root()?;
    let release_bin = workspace_root.join("target/release/scarab-daemon");
    let debug_bin = workspace_root.join("target/debug/scarab-daemon");

    if release_bin.exists() {
        return Ok(release_bin);
    }
    if debug_bin.exists() {
        return Ok(debug_bin);
    }
    anyhow::bail!("scarab-daemon binary not found (checked CARGO_TARGET_DIR and workspace target)")
}

struct TestHarness {
    daemon: Child,
    socket: UnixStream,
    shmem: Shmem,
}

impl TestHarness {
    fn new() -> Result<Self> {
        // Clean up - use test-specific paths
        let _ = std::fs::remove_file(SOCKET_PATH);
        #[cfg(target_os = "linux")]
        {
            let shm_path = format!("/dev/shm{}", TEST_SHMEM_PATH);
            let _ = std::fs::remove_file(&shm_path);
        }

        let daemon_bin = get_daemon_binary()?;
        println!("Using daemon: {}", daemon_bin.display());
        println!("Using test shared memory path: {}", TEST_SHMEM_PATH);

        // Start daemon with test-specific shared memory paths via env vars
        // Use env_clear() to prevent inheriting parent's SCARAB_SHMEM_PATH, then add back what we need
        let daemon = Command::new(&daemon_bin)
            .env_clear()
            .env("PATH", std::env::var("PATH").unwrap_or_default())
            .env("HOME", std::env::var("HOME").unwrap_or_default())
            .env("RUST_LOG", "debug")
            .env("SHELL", "/bin/bash")
            .env(SHMEM_PATH_ENV, TEST_SHMEM_PATH) // Override for test isolation
            .env(IMAGE_SHMEM_PATH_ENV, TEST_IMAGE_SHMEM_PATH)
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .spawn()
            .context("Failed to spawn daemon")?;

        // Wait for socket
        let start = Instant::now();
        while !Path::new(SOCKET_PATH).exists() {
            if start.elapsed() > DAEMON_STARTUP_TIMEOUT {
                anyhow::bail!("Daemon socket timeout");
            }
            thread::sleep(Duration::from_millis(50));
        }

        let socket = UnixStream::connect(SOCKET_PATH).context("Failed to connect")?;

        // Wait for shared memory using test-specific path
        let start = Instant::now();
        let shmem = loop {
            if start.elapsed() > DAEMON_STARTUP_TIMEOUT {
                anyhow::bail!("Shared memory timeout - daemon may not have started or used different path");
            }
            match ShmemConf::new()
                .size(std::mem::size_of::<SharedState>())
                .os_id(TEST_SHMEM_PATH)
                .open()
            {
                Ok(s) => break s,
                Err(e) => {
                    if start.elapsed() > Duration::from_secs(3) {
                        println!("Still waiting for shared memory at {} ... ({})", TEST_SHMEM_PATH, e);
                    }
                    thread::sleep(Duration::from_millis(50));
                }
            }
        };

        // Let shell initialize
        thread::sleep(Duration::from_millis(1000));

        Ok(Self { daemon, socket, shmem })
    }

    fn send(&mut self, text: &str) -> Result<()> {
        let msg = ControlMessage::Input {
            data: text.as_bytes().to_vec(),
        };
        let bytes = rkyv::to_bytes::<_, MAX_MESSAGE_SIZE>(&msg)?;
        let len = bytes.len() as u32;
        self.socket.write_all(&len.to_be_bytes())?;
        self.socket.write_all(&bytes)?;
        self.socket.flush()?;
        Ok(())
    }

    fn send_char(&mut self, c: char) -> Result<()> {
        self.send(&c.to_string())
    }

    fn send_enter(&mut self) -> Result<()> {
        self.send("\r")
    }

    fn get_state(&self) -> &SharedState {
        let ptr = self.shmem.as_ptr() as *const SharedState;
        unsafe { &*ptr }
    }

    fn get_grid_text(&self) -> String {
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
                    } else {
                        line.push(' ');
                    }
                }
            }
            output.push_str(&line.trim_end());
            output.push('\n');
        }
        output
    }

    fn wait(&self, ms: u64) {
        thread::sleep(Duration::from_millis(ms));
    }
}

impl Drop for TestHarness {
    fn drop(&mut self) {
        let _ = self.daemon.kill();
        let _ = self.daemon.wait();
        let _ = std::fs::remove_file(SOCKET_PATH);
    }
}

/// Test typing characters before and after pressing Enter
#[test]
fn test_newline_after_first_command() -> Result<()> {
    if !should_run_rtl_tests() {
        println!("Skipping (set SCARAB_TEST_RTL=1 to run)");
        return Ok(());
    }

    println!("\n=== Testing newline issue ===\n");

    let mut h = TestHarness::new()?;

    println!("Initial grid state:");
    println!("{}", h.get_grid_text());
    println!("---");

    // Type "ls"
    println!("Sending 'l'...");
    h.send_char('l')?;
    h.wait(100);

    println!("Sending 's'...");
    h.send_char('s')?;
    h.wait(100);

    println!("Grid after 'ls':");
    println!("{}", h.get_grid_text());
    println!("---");

    // Press Enter
    println!("Sending Enter...");
    h.send_enter()?;
    h.wait(500);

    println!("Grid after Enter:");
    println!("{}", h.get_grid_text());
    println!("---");

    // Now type "git" - this is where the issue might appear
    println!("Sending 'g'...");
    h.send_char('g')?;
    h.wait(100);

    println!("Grid after 'g':");
    println!("{}", h.get_grid_text());
    println!("---");

    println!("Sending 'i'...");
    h.send_char('i')?;
    h.wait(100);

    println!("Grid after 'i':");
    println!("{}", h.get_grid_text());
    println!("---");

    println!("Sending 't'...");
    h.send_char('t')?;
    h.wait(100);

    println!("Grid after 't':");
    println!("{}", h.get_grid_text());
    println!("---");

    println!("=== Test complete ===");
    Ok(())
}
