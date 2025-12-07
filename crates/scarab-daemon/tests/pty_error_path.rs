//! PTY failure path tests for scarab-daemon
//!
//! These tests verify that the daemon properly handles PTY creation failures
//! and writes legible error messages to shared memory for the client to display.

use std::process::{Command, Stdio};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use shared_memory::ShmemConf;

/// Generate a unique shared memory name for the test run.
fn unique_shm_name(base: &str) -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    format!("/{base}_{}_{}", std::process::id(), nanos)
}

#[test]
fn pty_failure_writes_error_to_shared_memory() {
    let shm_path = unique_shm_name("scarab_pty_fail_test");
    let img_shm_path = unique_shm_name("scarab_pty_fail_img_test");

    // Create isolated temp directory for this test
    let temp_dir = tempfile::tempdir().expect("failed to create temp dir");

    // Spawn daemon with PTY failure forced
    let mut daemon = Command::new(env!("CARGO_BIN_EXE_scarab-daemon"))
        .env("SCARAB_FORCE_PTY_FAIL", "1")
        .env("SCARAB_SHMEM_PATH", &shm_path)
        .env("SCARAB_IMAGE_SHMEM_PATH", &img_shm_path)
        .env("HOME", temp_dir.path())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("failed to spawn daemon");

    // Wait for daemon to initialize and write error
    std::thread::sleep(Duration::from_secs(1));

    // Open shared memory
    let shmem = match ShmemConf::new().os_id(&shm_path).open() {
        Ok(s) => s,
        Err(e) => {
            let _ = daemon.kill();
            let _ = daemon.wait();
            panic!("Failed to open shared memory: {:?}", e);
        }
    };

    // Read grid from shared memory
    let state_ptr = shmem.as_ptr() as *const scarab_protocol::SharedState;
    let (sequence, error_mode, text) = unsafe {
        let state = &*state_ptr;
        let mut s = String::new();
        for (i, cell) in state.cells.iter().take(1000).enumerate() {
            let ch = char::from_u32(cell.char_codepoint).unwrap_or(' ');
            s.push(ch);
            if (i + 1) % scarab_protocol::GRID_WIDTH == 0 {
                s.push('\n');
            }
        }
        (state.sequence_number, state.error_mode, s)
    };

    // Clean up
    let _ = daemon.kill();
    let _ = daemon.wait();

    // Verify error was written
    assert!(
        sequence > 0,
        "Sequence number should be > 0 after error write, got: {}",
        sequence
    );
    assert_eq!(
        error_mode, 1,
        "error_mode flag should be 1 when PTY fails, got: {}",
        error_mode
    );
    assert!(
        text.contains("ERROR"),
        "Grid should contain ERROR text, got first 200 chars: {}",
        &text[..text.len().min(200)]
    );
    assert!(
        text.to_lowercase().contains("pty") || text.to_lowercase().contains("fail"),
        "Error message should reference PTY or failure"
    );
}
