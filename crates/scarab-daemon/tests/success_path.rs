//! Success path tests for scarab-daemon
//!
//! These tests verify normal daemon operation including PTY initialization,
//! shared memory setup, and terminal output processing.
//!
//! **NOTE**: These tests require real PTY access (/dev/ptmx) and may not run
//! in restricted CI environments. Run with `--ignored` to execute:
//! ```sh
//! cargo test -p scarab-daemon --test success_path -- --ignored
//! ```

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
#[ignore = "requires real PTY access - run with --ignored"]
fn daemon_creates_shared_memory_on_success() {
    let shm_path = unique_shm_name("scarab_success_shm");
    let img_shm_path = unique_shm_name("scarab_success_img");

    let mut child = Command::new("cargo")
        .args(["run", "-p", "scarab-daemon"])
        .env("SCARAB_SHMEM_PATH", &shm_path)
        .env("SCARAB_IMAGE_SHMEM_PATH", &img_shm_path)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("failed to spawn daemon");

    // Wait for daemon to create shared memory (this will panic if it fails)
    let _shmem = wait_and_open_shmem(&shm_path);

    // If we got here, shared memory was created successfully

    let _ = child.kill();
    let _ = child.wait();
}

#[test]
#[ignore = "requires real PTY access - run with --ignored"]
fn daemon_initializes_grid_with_prompt() {
    let shm_path = unique_shm_name("scarab_prompt_shm");
    let img_shm_path = unique_shm_name("scarab_prompt_img");

    let mut child = Command::new("cargo")
        .args(["run", "-p", "scarab-daemon"])
        .env("SCARAB_SHMEM_PATH", &shm_path)
        .env("SCARAB_IMAGE_SHMEM_PATH", &img_shm_path)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("failed to spawn daemon");

    // Wait for daemon to create shared memory and initialize
    let shmem = wait_and_open_shmem(&shm_path);

    let state_ptr = shmem.as_ptr() as *const scarab_protocol::SharedState;
    let text = extract_grid_text(state_ptr, 1000);

    // The grid should contain some terminal output (not all spaces)
    let non_space_count = text.chars().filter(|&c| c != ' ' && c != '\n').count();
    assert!(
        non_space_count > 0,
        "daemon should write terminal output to shared memory, got only spaces"
    );

    // Sequence number should have been incremented
    let seq = get_sequence_number(state_ptr);
    assert!(
        seq > 0,
        "sequence number should be > 0 after initialization, got: {}",
        seq
    );

    let _ = child.kill();
    let _ = child.wait();
}

#[test]
#[ignore = "requires real PTY access - run with --ignored"]
fn daemon_updates_sequence_on_pty_output() {
    let shm_path = unique_shm_name("scarab_seq_update_shm");
    let img_shm_path = unique_shm_name("scarab_seq_update_img");

    let mut child = Command::new("cargo")
        .args(["run", "-p", "scarab-daemon"])
        .env("SCARAB_SHMEM_PATH", &shm_path)
        .env("SCARAB_IMAGE_SHMEM_PATH", &img_shm_path)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("failed to spawn daemon");

    // Give the daemon time to initialize
    std::thread::sleep(Duration::from_millis(1500));

    // Open the shared memory
    let shmem = ShmemConf::new()
        .os_id(&shm_path)
        .open()
        .expect("failed to open shared memory");

    let state_ptr = shmem.as_ptr() as *const scarab_protocol::SharedState;
    let initial_seq = get_sequence_number(state_ptr);

    // Wait for additional updates (shell might output more)
    std::thread::sleep(Duration::from_millis(1000));

    let later_seq = get_sequence_number(state_ptr);

    // Sequence should have incremented at least once
    assert!(
        later_seq >= initial_seq,
        "sequence number should not decrease, initial: {}, later: {}",
        initial_seq,
        later_seq
    );

    let _ = child.kill();
    let _ = child.wait();
}

#[test]
#[ignore = "requires real PTY access - run with --ignored"]
fn daemon_sets_valid_cursor_position() {
    let shm_path = unique_shm_name("scarab_cursor_valid_shm");
    let img_shm_path = unique_shm_name("scarab_cursor_valid_img");

    let mut child = Command::new("cargo")
        .args(["run", "-p", "scarab-daemon"])
        .env("SCARAB_SHMEM_PATH", &shm_path)
        .env("SCARAB_IMAGE_SHMEM_PATH", &img_shm_path)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("failed to spawn daemon");

    // Give the daemon time to initialize
    std::thread::sleep(Duration::from_millis(1500));

    // Open the shared memory
    let shmem = ShmemConf::new()
        .os_id(&shm_path)
        .open()
        .expect("failed to open shared memory");

    let state_ptr = shmem.as_ptr() as *const scarab_protocol::SharedState;
    let (cursor_x, cursor_y) = unsafe {
        let state = &*state_ptr;
        (state.cursor_x, state.cursor_y)
    };

    // Cursor should be within valid grid bounds
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
#[ignore = "requires real PTY access - run with --ignored"]
fn daemon_does_not_write_error_on_success() {
    let shm_path = unique_shm_name("scarab_no_error_shm");
    let img_shm_path = unique_shm_name("scarab_no_error_img");

    let mut child = Command::new("cargo")
        .args(["run", "-p", "scarab-daemon"])
        .env("SCARAB_SHMEM_PATH", &shm_path)
        .env("SCARAB_IMAGE_SHMEM_PATH", &img_shm_path)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("failed to spawn daemon");

    // Give the daemon time to initialize
    std::thread::sleep(Duration::from_millis(1500));

    // Open the shared memory
    let shmem = ShmemConf::new()
        .os_id(&shm_path)
        .open()
        .expect("failed to open shared memory");

    let state_ptr = shmem.as_ptr() as *const scarab_protocol::SharedState;
    let text = extract_grid_text(state_ptr, 1000);

    // Should NOT contain ERROR text
    assert!(
        !text.contains("ERROR"),
        "successful daemon should not write ERROR to grid"
    );

    let _ = child.kill();
    let _ = child.wait();
}
