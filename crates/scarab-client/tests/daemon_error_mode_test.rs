//! Client tests for daemon error mode
//!
//! These tests verify that the client can properly read and display error
//! messages from the daemon when PTY initialization fails.

use shared_memory::ShmemConf;
use std::process::{Command, Stdio};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// Generate a unique shared memory name for the test run.
fn unique_shm_name(base: &str) -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    format!("/{base}_{}_{}", std::process::id(), nanos)
}

/// Get path to daemon binary
fn get_daemon_path() -> std::path::PathBuf {
    // Look for the binary in the target directory
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".to_string());
    let target_dir = std::path::PathBuf::from(manifest_dir)
        .parent()
        .and_then(|p| p.parent())
        .map(|p| p.join("target/debug/scarab-daemon"))
        .filter(|p| p.exists());

    target_dir.unwrap_or_else(|| std::path::PathBuf::from("cargo"))
}

/// Spawn daemon with PTY failure for testing
fn spawn_daemon_with_pty_fail(
    shm_path: &str,
    img_shm_path: &str,
) -> (std::process::Child, tempfile::TempDir) {
    let temp_dir = tempfile::tempdir().expect("failed to create temp dir");
    let daemon_path = get_daemon_path();

    let child = if daemon_path.to_string_lossy() == "cargo" {
        Command::new("cargo")
            .args(["run", "-p", "scarab-daemon"])
            .env("SCARAB_FORCE_PTY_FAIL", "1")
            .env("SCARAB_SHMEM_PATH", shm_path)
            .env("SCARAB_IMAGE_SHMEM_PATH", img_shm_path)
            .env("HOME", temp_dir.path())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .expect("failed to spawn daemon")
    } else {
        Command::new(daemon_path)
            .env("SCARAB_FORCE_PTY_FAIL", "1")
            .env("SCARAB_SHMEM_PATH", shm_path)
            .env("SCARAB_IMAGE_SHMEM_PATH", img_shm_path)
            .env("HOME", temp_dir.path())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .expect("failed to spawn daemon")
    };

    (child, temp_dir)
}

/// Helper to extract text from SharedState cells
fn extract_grid_text(state_ptr: *const scarab_protocol::SharedState, max_cells: usize) -> String {
    unsafe {
        let state = &*state_ptr;
        let mut s = String::new();
        for (i, cell) in state.cells.iter().take(max_cells).enumerate() {
            let ch = char::from_u32(cell.char_codepoint).unwrap_or(' ');
            s.push(ch);
            if (i + 1) % scarab_protocol::GRID_WIDTH == 0 {
                s.push('\n');
            }
        }
        s
    }
}

/// Helper to wait for and open shared memory with retry logic
fn wait_and_open_shmem(shm_path: &str) -> shared_memory::Shmem {
    let mut attempts = 0;
    loop {
        attempts += 1;
        std::thread::sleep(Duration::from_millis(200));

        match ShmemConf::new().os_id(shm_path).open() {
            Ok(shmem) => return shmem,
            Err(_) if attempts < 25 => continue,
            Err(e) => panic!(
                "failed to open shared memory after {} attempts: {:?}",
                attempts, e
            ),
        }
    }
}

#[test]
fn client_can_read_daemon_error_grid() {
    let shm_path = unique_shm_name("scarab_client_err_shm");
    let img_shm_path = unique_shm_name("scarab_client_err_img");

    // Start daemon with forced PTY failure
    let (mut daemon, _temp_dir) = spawn_daemon_with_pty_fail(&shm_path, &img_shm_path);

    // Wait for daemon to create shared memory and write error grid
    let shmem = wait_and_open_shmem(&shm_path);

    let state_ptr = shmem.as_ptr() as *const scarab_protocol::SharedState;
    let text = extract_grid_text(state_ptr, 1000);
    let seq = unsafe {
        let state = &*state_ptr;
        state.sequence_number
    };

    // Verify client can read error text
    assert!(
        text.contains("ERROR"),
        "client should read ERROR text from shared memory"
    );
    assert!(
        text.to_lowercase().contains("pty") || text.to_lowercase().contains("fail"),
        "client should read PTY error details"
    );
    assert!(seq > 0, "client should see non-zero sequence number");

    let _ = daemon.kill();
    let _ = daemon.wait();
}

#[test]
fn client_can_detect_daemon_error_via_sequence() {
    let shm_path = unique_shm_name("scarab_client_seq_err_shm");
    let img_shm_path = unique_shm_name("scarab_client_seq_err_img");

    // Start daemon with forced PTY failure
    let (mut daemon, _temp_dir) = spawn_daemon_with_pty_fail(&shm_path, &img_shm_path);

    // Wait for daemon to create shared memory and write error grid
    let shmem = wait_and_open_shmem(&shm_path);

    let state_ptr = shmem.as_ptr() as *const scarab_protocol::SharedState;

    // Client checks sequence number to detect updates
    let seq = unsafe {
        let state = &*state_ptr;
        state.sequence_number
    };

    // Sequence > 0 indicates daemon wrote error grid
    assert!(
        seq > 0,
        "client should detect daemon wrote error via sequence number, got: {}",
        seq
    );

    let _ = daemon.kill();
    let _ = daemon.wait();
}

#[test]
fn client_can_read_full_error_message() {
    let shm_path = unique_shm_name("scarab_client_full_err_shm");
    let img_shm_path = unique_shm_name("scarab_client_full_err_img");

    // Start daemon with forced PTY failure
    let (mut daemon, _temp_dir) = spawn_daemon_with_pty_fail(&shm_path, &img_shm_path);

    // Wait for daemon to create shared memory and write error grid
    let shmem = wait_and_open_shmem(&shm_path);

    let state_ptr = shmem.as_ptr() as *const scarab_protocol::SharedState;
    let full_grid = extract_grid_text(
        state_ptr,
        scarab_protocol::GRID_WIDTH * scarab_protocol::GRID_HEIGHT,
    );

    // Verify error message is complete and readable
    assert!(
        full_grid.contains("ERROR"),
        "full grid should contain ERROR text"
    );

    // Error message should provide actionable information
    let lowercase = full_grid.to_lowercase();
    assert!(
        lowercase.contains("pty") || lowercase.contains("fail") || lowercase.contains("error"),
        "error message should contain descriptive text"
    );

    let _ = daemon.kill();
    let _ = daemon.wait();
}

#[test]
fn client_reads_valid_cursor_in_error_mode() {
    let shm_path = unique_shm_name("scarab_client_cursor_err_shm");
    let img_shm_path = unique_shm_name("scarab_client_cursor_err_img");

    // Start daemon with forced PTY failure
    let (mut daemon, _temp_dir) = spawn_daemon_with_pty_fail(&shm_path, &img_shm_path);

    // Wait for daemon to create shared memory and write error grid
    let shmem = wait_and_open_shmem(&shm_path);

    let state_ptr = shmem.as_ptr() as *const scarab_protocol::SharedState;
    let (cursor_x, cursor_y) = unsafe {
        let state = &*state_ptr;
        (state.cursor_x, state.cursor_y)
    };

    // Client should see valid cursor position
    assert!(
        cursor_x < scarab_protocol::GRID_WIDTH as u16,
        "client should read cursor_x within bounds, got: {}",
        cursor_x
    );
    assert!(
        cursor_y < scarab_protocol::GRID_HEIGHT as u16,
        "client should read cursor_y within bounds, got: {}",
        cursor_y
    );

    let _ = daemon.kill();
    let _ = daemon.wait();
}

#[test]
fn client_can_detect_error_mode_flag() {
    let shm_path = unique_shm_name("scarab_client_errmode_shm");
    let img_shm_path = unique_shm_name("scarab_client_errmode_img");

    // Start daemon with forced PTY failure
    let (mut daemon, _temp_dir) = spawn_daemon_with_pty_fail(&shm_path, &img_shm_path);

    // Wait for daemon to create shared memory and write error grid
    let shmem = wait_and_open_shmem(&shm_path);

    let state_ptr = shmem.as_ptr() as *const scarab_protocol::SharedState;

    // Client checks error_mode flag
    let error_mode = unsafe {
        let state = &*state_ptr;
        state.error_mode
    };

    // error_mode should be 1 when daemon encounters PTY failure
    assert_eq!(
        error_mode, 1,
        "client should detect error_mode=1 when daemon fails, got: {}",
        error_mode
    );

    let _ = daemon.kill();
    let _ = daemon.wait();
}
