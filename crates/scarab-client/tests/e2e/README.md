# Scarab E2E Test Framework

This directory contains the end-to-end integration test framework for Scarab terminal emulator. These tests validate the full daemon ↔ client workflow by spawning actual processes and testing real IPC communication.

## Overview

The E2E test framework tests the complete Scarab stack:
- **Daemon Process**: Headless server managing PTY processes
- **Client Process**: GUI client reading from shared memory
- **Shared Memory IPC**: Zero-copy bulk data transfer
- **Unix Socket Control**: Command and control messages
- **PTY Integration**: Real terminal interaction with bash

## Test Scenarios

### ✅ Implemented (8 scenarios)

1. **Basic Workflow** (`basic_workflow.rs`)
   - Echo commands
   - Multiple commands
   - Environment variables
   - Multiline input
   - Clear screen
   - Backspace handling

2. **Vim Editing** (`vim_editing.rs`) - _Requires vim, marked `#[ignore]`_
   - Opening vim
   - Insert mode
   - Text editing
   - Save/quit operations
   - Navigation commands
   - Search functionality

3. **Color Rendering** (`color_rendering.rs`)
   - ANSI color escape sequences
   - 16 basic colors
   - 256 color mode
   - Truecolor (24-bit RGB)
   - ls --color output
   - Bold/italic attributes

4. **Scrollback Buffer** (`scrollback.rs`)
   - Large output handling (1000+ lines)
   - Sequence number tracking
   - Continuous output
   - Rapid updates
   - Line wrapping
   - Partial line updates

5. **Session Persistence** (`session_persist.rs`)
   - Client disconnect/reconnect
   - State preservation
   - Daemon survival after client crash
   - Multiple disconnect cycles
   - Long-running processes

6. **Input Forwarding** (`input_forward.rs`)
   - Regular text input
   - Control sequences (Ctrl+C, Ctrl+D, Ctrl+L)
   - Arrow keys
   - Tab completion
   - Special characters
   - Unicode input (including emoji)
   - Rapid input handling

7. **Resize Handling** (`resize_handling.rs`)
   - Basic resize operations
   - Resize during output
   - Multiple resizes
   - Extreme sizes (very small/large)
   - Content preservation
   - Rapid resize changes

8. **Stress Testing** (`stress_test.rs`) - _All marked `#[ignore]`_
   - 1-hour continuous operation
   - Short stress test (5 minutes)
   - Memory stability
   - Rapid input stress
   - Concurrent commands
   - Resize stress
   - Disconnect stress

## Running Tests

### Run All E2E Tests (excluding ignored)
```bash
cargo test --test e2e
```

### Run With Output Visible
```bash
cargo test --test e2e -- --nocapture
```

### Run Specific Test
```bash
cargo test --test e2e test_basic_echo
cargo test --test e2e test_color_rendering
```

### Run Specific Test Module
```bash
cargo test --test e2e basic_workflow
cargo test --test e2e scrollback
```

### Run Ignored Tests (vim, stress)
```bash
# All ignored tests
cargo test --test e2e -- --ignored

# Specific ignored test
cargo test --test e2e test_vim_basic_editing -- --ignored
cargo test --test e2e test_stress_1_hour -- --ignored --nocapture
```

### Run in Parallel (faster)
```bash
cargo test --test e2e -- --test-threads=4
```

### Run Single-threaded (better for debugging)
```bash
cargo test --test e2e -- --test-threads=1
```

## Test Architecture

### E2ETestHarness

The core test harness (`harness.rs`) provides:

```rust
pub struct E2ETestHarness {
    daemon: Option<Child>,      // Daemon process handle
    client: Option<Child>,      // Client process handle
    shared_memory: Option<Shmem>, // Shared memory mapping
    socket_path: String,        // Unix socket path
    temp_dir: TempDir,          // Isolated temp directory
    daemon_bin: PathBuf,        // Path to daemon binary
    client_bin: PathBuf,        // Path to client binary
}
```

### Key Methods

**Process Management:**
- `new()` - Initialize harness, spawn daemon, wait for shared memory
- `start_client()` - Spawn client process
- `disconnect_client()` - Kill client (daemon survives)
- `reconnect_client()` - Spawn new client instance
- `cleanup()` - Terminate all processes

**IPC Communication:**
- `send_input(text)` - Send keyboard input via Unix socket
- `resize(cols, rows)` - Send resize command
- `get_shared_state()` - Read current SharedState from memory

**Output Verification:**
- `get_output(timeout)` - Get all visible text from terminal grid
- `get_line(line_num)` - Get specific line from grid
- `verify_output_contains(text, timeout)` - Poll until text appears

**Health Checks:**
- `daemon_is_alive()` - Check daemon process status
- `client_is_alive()` - Check client process status

### Automatic Cleanup

The harness implements `Drop` to ensure cleanup:
```rust
impl Drop for E2ETestHarness {
    fn drop(&mut self) {
        self.cleanup();
        // - Kills client and daemon
        // - Removes socket file
        // - Cleans up shared memory
    }
}
```

## Requirements

### System Requirements
- **OS**: Linux or macOS (Unix socket support)
- **Shell**: Bash available at default location
- **Permissions**: Write access to `/tmp`
- **Memory**: Sufficient for shared memory segments

### Build Requirements
- Daemon binary: `target/debug/scarab-daemon` or `target/release/scarab-daemon`
- Client binary: `target/debug/scarab-client` or `target/release/scarab-client`

The harness will automatically build binaries if not found.

### Optional (for specific tests)
- **vim**: Required for vim_editing tests
- **top/htop**: Used in some resize tests
- **seq, cat, ls**: Standard Unix utilities

## Test Design Principles

### 1. Isolation
Each test creates a fresh daemon and temporary directory. Tests don't share state.

### 2. Real Processes
Tests spawn actual daemon and client binaries, not mocked versions. This validates real-world behavior.

### 3. Timeouts
All operations use timeouts to prevent hanging tests:
- Daemon startup: 10 seconds
- Client connection: 5 seconds
- Output verification: Configurable per test

### 4. Polling
Output verification polls shared memory with 50ms intervals, allowing for rendering delays.

### 5. Cleanup
Resources are cleaned up automatically via `Drop` even if tests panic.

## Debugging Failed Tests

### Enable Verbose Output
```bash
cargo test --test e2e test_name -- --nocapture
```

### Check Process Status
Tests print process status and shared state:
```
=== Initializing E2E Test Harness ===
Temp directory: /tmp/.tmpXXXXXX
Daemon binary: target/debug/scarab-daemon
✓ Shared memory opened successfully
✓ Socket created at /tmp/scarab-daemon.sock
=== Harness initialized successfully ===
```

### Inspect Shared State
Tests log sequence numbers and grid contents:
```rust
let state = harness.get_shared_state()?;
println!("Sequence: {}", state.sequence_number);
println!("Cursor: ({}, {})", state.cursor_x, state.cursor_y);
```

### Manual Process Inspection
If tests hang, check processes manually:
```bash
# Find daemon process
ps aux | grep scarab-daemon

# Check shared memory
ls -la /dev/shm/scarab_shm_v1

# Check socket
ls -la /tmp/scarab-daemon.sock
```

### Run Single Test with Debug
```bash
RUST_LOG=debug cargo test --test e2e test_basic_echo -- --nocapture
```

## CI Integration

### Recommended CI Configuration

```yaml
# .github/workflows/e2e-tests.yml
name: E2E Tests

on: [push, pull_request]

jobs:
  e2e:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: Install Rust
        uses: actions-rs/toolchain@v1

      - name: Build binaries
        run: cargo build --workspace

      - name: Run E2E tests
        run: cargo test --test e2e -- --test-threads=1

      - name: Run stress tests (short)
        run: cargo test --test e2e test_stress_short -- --ignored --nocapture
```

### Skip Heavy Tests in CI
Stress tests are marked `#[ignore]` by default. Run them manually or in nightly CI:

```bash
# Run only fast tests
cargo test --test e2e

# Run stress tests in nightly CI
cargo test --test e2e test_stress_short -- --ignored
```

## Performance Notes

### Test Duration
- **Basic tests**: ~2-5 seconds each
- **Vim tests**: ~5-10 seconds each (requires vim)
- **Stress tests**: 5 minutes to 1 hour

### Parallelization
Tests can run in parallel since each spawns isolated processes:
```bash
cargo test --test e2e -- --test-threads=8
```

However, for debugging, single-threaded is clearer:
```bash
cargo test --test e2e -- --test-threads=1
```

### Resource Usage
Each test spawns 1 daemon process plus shared memory (~320KB). Running many tests in parallel may require system tuning:

```bash
# Increase shared memory limits if needed
sudo sysctl -w kernel.shmmax=1073741824
```

## Future Enhancements

### Potential Additions
- **Network transparency tests**: Remote daemon connections
- **Plugin system tests**: Fusabi plugin loading
- **Mouse input tests**: Mouse event forwarding
- **Clipboard tests**: Copy/paste operations
- **Config loading tests**: TOML configuration priority
- **Multiple session tests**: Session multiplexing
- **Performance benchmarks**: Throughput and latency

### Test Coverage Goals
- [x] Basic terminal operations
- [x] Color rendering
- [x] Scrollback handling
- [x] Session persistence
- [x] Input forwarding
- [x] Resize handling
- [x] Stress testing
- [ ] Plugin system integration
- [ ] Mouse support
- [ ] Alternate screen buffer
- [ ] Unicode edge cases
- [ ] Performance regression detection

## Contributing

When adding new E2E tests:

1. Create a new module file in `tests/e2e/`
2. Add module to `tests/e2e/mod.rs`
3. Use `E2ETestHarness` for process management
4. Add timeouts to all blocking operations
5. Clean up resources (harness does this automatically)
6. Mark long-running tests with `#[ignore]`
7. Document test purpose and requirements
8. Update this README with new scenarios

### Example New Test

```rust
// tests/e2e/my_feature.rs
use super::harness::E2ETestHarness;
use anyhow::Result;
use std::thread;
use std::time::Duration;

#[test]
fn test_my_feature() -> Result<()> {
    println!("\n=== Test: My Feature ===");

    let harness = E2ETestHarness::new()?;
    thread::sleep(Duration::from_secs(1));

    // Test implementation
    harness.send_input("test command\n")?;

    let found = harness.verify_output_contains(
        "expected output",
        Duration::from_secs(2)
    )?;

    assert!(found, "Feature should work");

    println!("=== Test Passed ===\n");
    Ok(())
}
```

## License

Same as Scarab project.
