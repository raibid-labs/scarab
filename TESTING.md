# Scarab Testing Guide

Comprehensive testing documentation for Scarab terminal emulator.

## Quick Reference

All test commands at a glance:

| Test Type | Command | Description |
|-----------|---------|-------------|
| **All Tests** | `cargo test --workspace` | Run all workspace tests |
| **Unit Tests** | `cargo test --workspace --lib` | Run all unit tests |
| **Integration Tests** | `cargo test --workspace --test '*'` | Run all integration tests |
| **Golden Tests** | `cargo test -p scarab-client --test golden_tests` | Snapshot regression tests |
| **Nav Smoke Tests** | `just nav-smoke` or `./scripts/nav-smoke-test.sh` | Navigation system validation |
| **Headless Tests** | `cargo test -p scarab-client --test headless_harness` | Bevy UI tests without GPU |
| **E2E Tests** | `cargo test -p scarab-client --test e2e` | Full daemon-client integration |
| **Workspace E2E** | `cargo test --test full_stack_test` | Workspace-level E2E tests |
| **Benchmarks** | `cargo bench --workspace` | Performance benchmarks |
| **CI Suite** | `just ci` | Format check + clippy + tests |
| **Quick Check** | `just quick` | Check + test (fast iteration) |

---

## Test Types

### 1. Unit Tests

Unit tests are embedded within source files using `#[cfg(test)]` modules or in separate test modules.

**Run all unit tests:**
```bash
cargo test --workspace --lib
```

**Run tests for a specific crate:**
```bash
cargo test -p scarab-client --lib
cargo test -p scarab-daemon --lib
cargo test -p scarab-protocol --lib
cargo test -p scarab-config --lib
cargo test -p scarab-platform --lib
cargo test -p scarab-plugin-api --lib
cargo test -p scarab-session --lib
cargo test -p scarab-themes --lib
```

**Run specific test modules:**
```bash
# Navigation tests
cargo test -p scarab-client --lib navigation

# VTE parser tests
cargo test -p scarab-daemon --lib vte

# Config parsing tests
cargo test -p scarab-config --lib parser
```

**What unit tests cover:**
- Component-level logic (functions, structs, methods)
- Edge cases and error handling
- Data structure operations
- Parser correctness
- State management

---

### 2. Integration Tests

Integration tests verify component interactions within individual crates. Located in `crates/<crate>/tests/`.

**Run all integration tests:**
```bash
cargo test --workspace --test '*'
```

**Client integration tests:**
```bash
cargo test -p scarab-client --test integration_e2e
cargo test -p scarab-client --test ui_tests
cargo test -p scarab-client --test command_palette_ui_tests
cargo test -p scarab-client --test link_hints_tests
cargo test -p scarab-client --test selection_tests
cargo test -p scarab-client --test overlay_tests
```

**Daemon integration tests:**
```bash
cargo test -p scarab-daemon --test ipc_integration
cargo test -p scarab-daemon --test session_integration
cargo test -p scarab-daemon --test plugin_integration
cargo test -p scarab-daemon --test vte_conformance
cargo test -p scarab-daemon --test tab_pane_multiplexing
```

**Platform integration tests:**
```bash
cargo test -p scarab-platform --test platform_tests
```

**Config integration tests:**
```bash
cargo test -p scarab-config --test integration_tests
```

**Theme integration tests:**
```bash
cargo test -p scarab-themes --test integration_test
```

**What integration tests cover:**
- Multi-module interactions
- Subsystem integration
- API contracts
- Feature workflows
- Platform-specific behavior

---

### 3. Golden Tests

Golden tests capture terminal grid snapshots for visual regression testing.

**Run golden tests:**
```bash
cargo test -p scarab-client --test golden_tests
```

**What golden tests verify:**
- Basic text rendering
- ANSI color output
- Unicode character support
- Emoji rendering
- Ligature display (Fira Code)
- Image placeholders (simulated)
- Grid boundary handling
- Terminal session simulation

**Notes:**
- All golden tests run in headless mode (no GPU/window required)
- Snapshots are compared against saved reference outputs
- Use `--nocapture` to see diff details on failure

---

### 4. Navigation Smoke Tests

Comprehensive navigation system validation suite.

**Run navigation smoke tests:**
```bash
just nav-smoke
```

Or directly:
```bash
./scripts/nav-smoke-test.sh
```

**What the smoke test includes:**
1. **Navigation Unit Tests** - Mode switching, focus management, per-pane state
2. **Golden Tests** - Snapshot validation for rendering
3. **Headless Harness Tests** - E2E navigation workflows

**Navigation features tested:**
- Mode switching (Normal, Hint, Insert)
- Focusable detection (URLs, paths, emails)
- Per-pane navigation state isolation
- Plugin bridge interactions
- OSC 133 prompt marker integration

**Exit codes:**
- `0` - All tests passed
- `1` - Navigation unit tests failed
- `2` - Golden tests failed
- `3` - Headless harness tests failed

See [Navigation Developer Guide](docs/navigation/developer-guide.md) for more details.

---

### 5. Headless Tests

Bevy UI system tests without display server requirements.

**Run all headless harness tests:**
```bash
cargo test -p scarab-client --test headless_harness
```

**Run specific headless tests:**
```bash
cargo test -p scarab-client --test headless_harness -- test_name
cargo test -p scarab-client --test headless_poc
cargo test -p scarab-client --test harness_examples
cargo test -p scarab-client --test harness_standalone
```

**What headless tests provide:**
- Mock terminal grid with programmable content
- Grid snapshot capture for verification
- Bevy ECS testing without GPU
- Terminal state simulation
- Deterministic system execution

**Use cases:**
- CI environments without display servers
- Fast UI component testing
- System interaction validation
- State management verification

See `crates/scarab-client/tests/headless_harness.rs` for API documentation.

---

### 6. E2E (End-to-End) Tests

Full daemon-client integration tests spawning actual processes.

**Run all E2E tests (excluding stress tests):**
```bash
cargo test -p scarab-client --test e2e
```

**Run specific E2E test suites:**
```bash
# Basic workflow tests
cargo test -p scarab-client --test e2e basic_workflow

# Vim editing tests (requires vim installed)
cargo test -p scarab-client --test e2e vim_editing -- --ignored

# Color rendering tests
cargo test -p scarab-client --test e2e color_rendering

# Scrollback tests
cargo test -p scarab-client --test e2e scrollback

# Session persistence tests
cargo test -p scarab-client --test e2e session_persist

# Input forwarding tests
cargo test -p scarab-client --test e2e input_forward

# Resize handling tests
cargo test -p scarab-client --test e2e resize_handling

# Stress tests (long-running)
cargo test -p scarab-client --test e2e stress_test -- --ignored
```

**What E2E tests cover:**
- **Basic Workflow**: Echo commands, environment variables, clear screen
- **Vim Editing**: Text editing, save/quit, navigation
- **Color Rendering**: ANSI colors, 256 color mode, true color
- **Scrollback**: Large output handling, continuous updates
- **Session Persistence**: Client disconnect/reconnect, state preservation
- **Input Forwarding**: Control sequences, arrow keys, Unicode
- **Resize Handling**: Dynamic terminal resizing, content preservation
- **Stress Testing**: Long-running stability tests

**Requirements:**
- Built binaries: `cargo build --workspace` (E2E tests spawn actual processes)
- Some tests require external tools (e.g., `vim`)

See [E2E Test Framework README](crates/scarab-client/tests/e2e/README.md) for detailed documentation.

---

### 7. Workspace E2E Tests

Full stack tests at the workspace root verify complete daemon-client workflows.

**Run workspace E2E tests:**
```bash
cargo test --test full_stack_test
cargo test --test program_interactions
```

**What workspace E2E tests cover:**
- Complete daemon-client communication
- IPC protocol validation
- PTY interaction
- Real program execution
- Multi-session scenarios

---

### 8. Benchmarks

Performance and regression testing.

**Run all benchmarks:**
```bash
cargo bench --workspace
```

Or via justfile:
```bash
just bench
```

**Run benchmarks for specific crate:**
```bash
cargo bench -p scarab-daemon
cargo bench -p scarab-client
```

**What benchmarks measure:**
- Rendering performance
- VTE parsing speed
- IPC throughput
- Plugin execution time
- Memory allocations

---

## Just Commands

The `justfile` provides convenient shortcuts for common test operations:

| Command | Equivalent | Description |
|---------|-----------|-------------|
| `just test` | `cargo test --workspace` | Run all workspace tests |
| `just test-verbose` | `cargo test --workspace -- --nocapture` | Run tests with output visible |
| `just nav-smoke` | `./scripts/nav-smoke-test.sh` | Navigation smoke test suite |
| `just bench` | `cargo bench --workspace` | Run all benchmarks |
| `just quick` | `cargo check --workspace && cargo test --workspace` | Quick iteration (check + test) |
| `just ci` | `cargo fmt --all -- --check && cargo clippy --workspace -- -D warnings && cargo test --workspace` | Full CI suite |

---

## Test Organization

Tests are organized by type and scope:

```
scarab/
├── tests/                          # Workspace-level tests
│   ├── integration/
│   │   └── full_stack_test.rs     # Complete daemon+client tests
│   └── e2e/
│       └── program_interactions.rs # Real program interactions
│
├── scripts/
│   └── nav-smoke-test.sh          # Navigation smoke test script
│
└── crates/
    ├── scarab-client/
    │   ├── src/
    │   │   └── navigation/
    │   │       └── tests.rs        # Navigation unit tests
    │   └── tests/
    │       ├── e2e/                # E2E test framework
    │       │   ├── harness.rs
    │       │   ├── basic_workflow.rs
    │       │   ├── vim_editing.rs
    │       │   └── ...
    │       ├── golden_tests.rs     # Snapshot tests
    │       ├── headless_harness.rs # Headless test infra
    │       ├── ui_tests.rs         # UI component tests
    │       └── ...
    │
    ├── scarab-daemon/
    │   ├── src/
    │   │   └── tests/
    │   │       ├── mod.rs
    │   │       └── vte_tests.rs
    │   └── tests/
    │       ├── ipc_integration.rs
    │       ├── session_integration.rs
    │       ├── plugin_integration.rs
    │       ├── vte_conformance.rs
    │       └── tab_pane_multiplexing.rs
    │
    └── scarab-*/
        └── tests/                  # Per-crate integration tests
            └── integration_tests.rs
```

---

## Writing Tests

### Unit Tests

Place unit tests inline with `#[cfg(test)]` or in dedicated test modules:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_feature() {
        let result = my_function();
        assert_eq!(result, expected_value);
    }
}
```

### Integration Tests

Place integration tests in `crates/<crate>/tests/`:

```rust
// crates/my-crate/tests/integration_test.rs
use my_crate::*;

#[test]
fn test_integration_scenario() {
    // Test cross-module behavior
}
```

### Headless Tests

Use the `HeadlessHarness` for testing Bevy UI components:

```rust
use crate::tests::harness::HeadlessHarness;

#[test]
fn test_ui_component() {
    let mut harness = HeadlessHarness::new();

    // Set terminal content
    harness.set_grid_text(0, 0, "Hello, World!");
    harness.tick_grid();

    // Update Bevy systems
    harness.update();

    // Verify results
    let snapshot = harness.capture_grid_snapshot();
    assert!(snapshot.contains("Hello, World!"));
}
```

### E2E Tests

Use the `E2ETestHarness` for full daemon-client tests:

```rust
use super::harness::E2ETestHarness;

#[test]
fn test_e2e_scenario() -> anyhow::Result<()> {
    let harness = E2ETestHarness::new()?;

    harness.send_input("echo test\n")?;

    let found = harness.verify_output_contains(
        "test",
        Duration::from_secs(2)
    )?;

    assert!(found);
    Ok(())
}
```

### Best Practices

1. **Use descriptive test names** - Test names should clearly describe what's being tested
2. **Test edge cases** - Don't just test the happy path
3. **Keep tests fast** - Use mocks and headless mode when possible
4. **Ensure test isolation** - Tests should not depend on each other
5. **Use timeouts** - Prevent hanging tests with appropriate timeouts
6. **Clean up resources** - Use RAII (Drop) for automatic cleanup
7. **Mark slow tests** - Use `#[ignore]` for long-running tests
8. **Document requirements** - Note any external dependencies (vim, etc.)

---

## CI Integration

Scarab tests are designed to run in CI environments:

### Headless Mode

All rendering tests use headless mode (no GPU/window required):
- Uses `MinimalPlugins` instead of `DefaultPlugins`
- Uses `ScheduleRunnerPlugin` for deterministic execution
- No display server dependencies

### Test Parallelization

Tests can run in parallel for faster CI:
```bash
cargo test --workspace -- --test-threads=8
```

For debugging, run single-threaded:
```bash
cargo test --workspace -- --test-threads=1
```

### CI Test Matrix

Recommended CI configuration:

```yaml
test:
  runs-on: ubuntu-latest
  steps:
    - uses: actions/checkout@v3
    - uses: actions-rs/toolchain@v1

    # Fast checks
    - run: cargo check --workspace
    - run: cargo clippy --workspace -- -D warnings
    - run: cargo fmt --all -- --check

    # Unit and integration tests
    - run: cargo test --workspace --lib
    - run: cargo test --workspace --test '*'

    # Navigation smoke tests
    - run: ./scripts/nav-smoke-test.sh

    # E2E tests (excluding stress tests)
    - run: cargo test -p scarab-client --test e2e
```

---

## Troubleshooting

### Common Issues

**Tests fail with "shared memory not found":**

E2E tests require the daemon to create shared memory. Ensure daemon starts successfully:
```bash
cargo test --test e2e -- --nocapture
```

**Tests hang indefinitely:**

Check for deadlocks or missing timeouts. Run with single thread for easier debugging:
```bash
cargo test --workspace -- --test-threads=1 --nocapture
```

**Navigation tests timeout:**

Navigation tests require the full ECS setup. Ensure Bevy headless mode is configured correctly. Check for:
- Missing resources
- System scheduling issues
- Uninitialized components

**E2E tests fail with "binary not found":**

E2E tests require built binaries. Build before testing:
```bash
cargo build --workspace
cargo test -p scarab-client --test e2e
```

**Golden tests fail with snapshot mismatch:**

Review the snapshot diff and update if the change is intentional:
```bash
cargo test -p scarab-client --test golden_tests -- --nocapture
```

### Debug Output

Enable debug logging:
```bash
RUST_LOG=debug cargo test -- --nocapture
```

Enable specific crate logging:
```bash
RUST_LOG=scarab_client=trace cargo test -- --nocapture
```

### Manual Test Inspection

For debugging failing E2E tests, inspect processes manually:

```bash
# Find running daemon
ps aux | grep scarab-daemon

# Check shared memory
ls -la /dev/shm/scarab_shm_*

# Check Unix socket
ls -la /tmp/scarab-daemon.sock

# Clean up stale resources
just clean-shm
just kill
```

---

## Performance Testing

### Benchmarks

Run performance benchmarks:
```bash
just bench
```

Run benchmarks for specific crate:
```bash
cargo bench -p scarab-daemon
cargo bench -p scarab-client
```

### Profiling

Build with profiling symbols:
```bash
just profile
```

Run with profiling enabled:
```bash
just run-profile
```

See [BENCHMARK_GUIDE.md](docs/BENCHMARK_GUIDE.md) for detailed profiling instructions (if available).

---

## Test Coverage

To generate test coverage reports:

```bash
# Install tarpaulin
cargo install cargo-tarpaulin

# Generate coverage report
cargo tarpaulin --workspace --out Html --output-dir coverage/

# View report
open coverage/index.html
```

---

## Further Reading

- [Navigation Developer Guide](docs/navigation/developer-guide.md) - Navigation system testing
- [E2E Test Framework README](crates/scarab-client/tests/e2e/README.md) - E2E test architecture
- [Headless Testing Quickstart](docs/testing/HEADLESS_TESTING_QUICKSTART.md) - Headless test infrastructure
- [Configuration Guide](docs/configuration.md) - Test configuration
- [Developer Architecture Guide](docs/developer/architecture.md) - System architecture
- [CLAUDE.md](./CLAUDE.md) - Project overview and build commands
- [ROADMAP.md](./ROADMAP.md) - Development roadmap

---

## Contributing

When adding new tests:

1. Follow existing test organization patterns
2. Use appropriate test type (unit, integration, E2E)
3. Add timeouts to prevent hanging tests
4. Document test requirements and purpose
5. Update this guide with new test suites
6. Ensure tests run in CI environments (headless)

---

**Last updated**: 2025-12-03
