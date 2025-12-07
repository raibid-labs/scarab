use std::process::{Command, Stdio};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use shared_memory::ShmemConf;

/// Generate a unique shared memory name for the test run.
fn unique_shm_name(base: &str) -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    format!("/{base}_{}_{}", std::process::id(), nanos, 0u32)
}

#[test]
fn error_mode_writes_legible_message_on_pty_failure() {
    let shm_path = unique_shm_name("scarab_err_shm");
    let img_shm_path = unique_shm_name("scarab_err_img");

    let mut child = Command::new("cargo")
        .args(["run", "-p", "scarab-daemon"])
        .env("SCARAB_FORCE_PTY_FAIL", "1")
        .env("SCARAB_SHMEM_PATH", &shm_path)
        .env("SCARAB_IMAGE_SHMEM_PATH", &img_shm_path)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("failed to spawn daemon");

    // Give the daemon a moment to write the error grid
    std::thread::sleep(Duration::from_millis(500));

    // Open the shared memory written by the daemon
    let shmem = ShmemConf::new()
        .os_id(&shm_path)
        .open()
        .expect("failed to open shared memory");

    // Read a small slice of the grid
    let state_ptr = shmem.as_ptr() as *const scarab_protocol::SharedState;
    let text = unsafe {
        let state = &*state_ptr;
        let mut s = String::new();
        for (i, cell) in state.cells.iter().take(200).enumerate() {
            let ch = char::from_u32(cell.char_codepoint).unwrap_or(' ');
            s.push(ch);
            if (i + 1) % scarab_protocol::GRID_WIDTH == 0 {
                s.push('\n');
            }
        }
        s
    };

    // Ensure we see an ERROR banner referencing PTY
    assert!(
        text.contains("ERROR"),
        "error grid should contain ERROR text, got: {}",
        &text[..text.len().min(120)]
    );
    assert!(
        text.to_lowercase().contains("pty"),
        "error grid should reference PTY failure"
    );

    let _ = child.kill();
    let _ = child.wait();
}
