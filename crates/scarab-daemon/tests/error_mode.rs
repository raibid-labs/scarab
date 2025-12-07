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

/// Spawn daemon with isolated environment
fn spawn_daemon_with_pty_fail(
    shm_path: &str,
    img_shm_path: &str,
) -> (std::process::Child, tempfile::TempDir) {
    let temp_dir = tempfile::tempdir().expect("failed to create temp dir");
    let child = Command::new(env!("CARGO_BIN_EXE_scarab-daemon"))
        .env("SCARAB_FORCE_PTY_FAIL", "1")
        .env("SCARAB_SHMEM_PATH", shm_path)
        .env("SCARAB_IMAGE_SHMEM_PATH", img_shm_path)
        .env("HOME", temp_dir.path())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("failed to spawn daemon");
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

/// Helper to get sequence number from SharedState
fn get_sequence_number(state_ptr: *const scarab_protocol::SharedState) -> u64 {
    unsafe {
        let state = &*state_ptr;
        state.sequence_number
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
fn error_mode_writes_legible_message_on_pty_failure() {
    let shm_path = unique_shm_name("scarab_err_shm");
    let img_shm_path = unique_shm_name("scarab_err_img");

    let (mut child, _temp_dir) = spawn_daemon_with_pty_fail(&shm_path, &img_shm_path);

    // Wait for shared memory creation
    std::thread::sleep(Duration::from_secs(1));

    // Open shared memory
    let shmem = match ShmemConf::new().os_id(&shm_path).open() {
        Ok(s) => s,
        Err(e) => {
            let _ = child.kill();
            let _ = child.wait();
            panic!("failed to open shared memory: {:?}", e);
        }
    };

    // Read a small slice of the grid
    let state_ptr = shmem.as_ptr() as *const scarab_protocol::SharedState;
    let text = extract_grid_text(state_ptr, 400);

    // Ensure we see an ERROR banner referencing PTY
    assert!(
        text.contains("ERROR"),
        "error grid should contain ERROR text, got: {}",
        &text[..text.len().min(120)]
    );
    assert!(
        text.to_lowercase().contains("pty") || text.to_lowercase().contains("fail"),
        "error grid should reference PTY failure"
    );

    let _ = child.kill();
    let _ = child.wait();
}

#[test]
fn error_mode_increments_sequence_number() {
    let shm_path = unique_shm_name("scarab_err_seq_shm");
    let img_shm_path = unique_shm_name("scarab_err_seq_img");

    let (mut child, _temp_dir) = spawn_daemon_with_pty_fail(&shm_path, &img_shm_path);

    // Wait for daemon to create shared memory and write error grid
    let shmem = wait_and_open_shmem(&shm_path);

    let state_ptr = shmem.as_ptr() as *const scarab_protocol::SharedState;
    let seq = get_sequence_number(state_ptr);

    // Sequence number should be > 0 after error grid is written
    assert!(
        seq > 0,
        "error grid should increment sequence number, got: {}",
        seq
    );

    let _ = child.kill();
    let _ = child.wait();
}

#[test]
fn error_mode_sets_cursor_position() {
    let shm_path = unique_shm_name("scarab_err_cursor_shm");
    let img_shm_path = unique_shm_name("scarab_err_cursor_img");

    let (mut child, _temp_dir) = spawn_daemon_with_pty_fail(&shm_path, &img_shm_path);

    // Wait for daemon to create shared memory and write error grid
    let shmem = wait_and_open_shmem(&shm_path);

    let state_ptr = shmem.as_ptr() as *const scarab_protocol::SharedState;
    let (cursor_x, cursor_y) = unsafe {
        let state = &*state_ptr;
        (state.cursor_x, state.cursor_y)
    };

    // Cursor should be positioned within grid bounds
    assert!(
        cursor_x < scarab_protocol::GRID_WIDTH as u16,
        "cursor_x should be within grid width, got: {}",
        cursor_x
    );
    assert!(
        cursor_y < scarab_protocol::GRID_HEIGHT as u16,
        "cursor_y should be within grid height, got: {}",
        cursor_y
    );

    let _ = child.kill();
    let _ = child.wait();
}

#[test]
fn error_mode_sets_dirty_flag() {
    let shm_path = unique_shm_name("scarab_err_dirty_shm");
    let img_shm_path = unique_shm_name("scarab_err_dirty_img");

    let (mut child, _temp_dir) = spawn_daemon_with_pty_fail(&shm_path, &img_shm_path);

    // Wait for daemon to create shared memory and write error grid
    let shmem = wait_and_open_shmem(&shm_path);

    let state_ptr = shmem.as_ptr() as *const scarab_protocol::SharedState;
    let dirty_flag = unsafe {
        let state = &*state_ptr;
        state.dirty_flag
    };

    // Dirty flag should be set after error grid is written
    assert_eq!(
        dirty_flag, 1,
        "error grid should set dirty flag, got: {}",
        dirty_flag
    );

    let _ = child.kill();
    let _ = child.wait();
}

#[test]
fn error_mode_sets_error_mode_flag() {
    let shm_path = unique_shm_name("scarab_err_flag_shm");
    let img_shm_path = unique_shm_name("scarab_err_flag_img");

    let (mut child, _temp_dir) = spawn_daemon_with_pty_fail(&shm_path, &img_shm_path);

    // Wait for daemon to create shared memory and write error grid
    let shmem = wait_and_open_shmem(&shm_path);

    let state_ptr = shmem.as_ptr() as *const scarab_protocol::SharedState;
    let error_mode = unsafe {
        let state = &*state_ptr;
        state.error_mode
    };

    // error_mode flag should be set to 1 when PTY fails
    assert_eq!(
        error_mode, 1,
        "error_mode flag should be set to 1 on PTY failure, got: {}",
        error_mode
    );

    let _ = child.kill();
    let _ = child.wait();
}
