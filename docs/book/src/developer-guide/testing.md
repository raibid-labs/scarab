# Testing

Scarab uses comprehensive testing to ensure reliability and correctness.

## Test Structure

Tests are organized by crate:

```
crates/
├── scarab-daemon/tests/     # Daemon integration tests
├── scarab-client/tests/     # Client integration tests
├── scarab-protocol/tests/   # IPC protocol tests
└── */src/**/*_test.rs       # Unit tests
```

## Running Tests

### All Tests

```bash
cargo test --workspace
```

### Specific Crate

```bash
cargo test -p scarab-daemon
cargo test -p scarab-client
cargo test -p scarab-protocol
```

### Specific Test

```bash
cargo test -p scarab-client focus_management
cargo test -p scarab-daemon pty_integration
```

### With Output

```bash
cargo test -- --nocapture
```

## Test Categories

### Unit Tests

Located in `src/` files as inline modules:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_feature() {
        // Test implementation
    }
}
```

### Integration Tests

Located in `tests/` directories:

```rust
// tests/integration_test.rs
#[test]
fn test_full_workflow() {
    // Multi-component test
}
```

## Navigation System Tests

The navigation system has comprehensive test coverage:

- **Focus Management**: Focus state transitions
- **Tab Management**: Create, delete, switch tabs
- **Pane Operations**: Split, close, resize
- **Spatial Navigation**: Directional movement

For detailed test plan, see:
- [Navigation Test Plan](../../navigation/TEST_PLAN.md)
- [Test Audit](../../audits/codex-2025-12-02-nav-ecs-001/)

## IPC Testing

Testing shared memory and protocol:

```bash
cargo test -p scarab-protocol
```

Key test areas:
- Shared memory layout (`#[repr(C)]`)
- Atomic operations
- Sequence number synchronization
- Zero-copy semantics

## Plugin Testing

Testing plugin loading and execution:

```bash
cargo test plugin_loader
cargo test fusabi_integration
```

## Continuous Integration

CI runs on every commit:

- Build all crates
- Run all tests
- Check formatting (`cargo fmt`)
- Run clippy lints
- Build documentation

## Test Coverage

Generate coverage reports:

```bash
# Install tarpaulin
cargo install cargo-tarpaulin

# Generate coverage
cargo tarpaulin --workspace --out Html
```

## Writing Good Tests

### Guidelines

1. **Test behavior, not implementation**
2. **Use descriptive test names**
3. **Keep tests isolated and independent**
4. **Use test fixtures for complex setup**
5. **Test edge cases and error paths**

### Example

```rust
#[test]
fn focus_should_move_to_next_pane_when_current_closed() {
    // Arrange
    let mut app = setup_test_app();
    let tab = create_tab(&mut app);
    let pane1 = create_pane(&mut app, tab);
    let pane2 = create_pane(&mut app, tab);
    focus_pane(&mut app, pane1);

    // Act
    close_pane(&mut app, pane1);

    // Assert
    assert_focused(&app, pane2);
}
```

## Benchmarking

Performance benchmarks using Criterion:

```bash
cargo bench -p scarab-client
cargo bench -p scarab-daemon
```

Key benchmarks:
- IPC throughput
- Rendering performance
- Navigation system latency
- Plugin execution time

## Debugging Tests

### Running with GDB

```bash
rust-gdb --args target/debug/deps/scarab_client-<hash>
```

### Logging in Tests

```rust
#[test]
fn test_with_logging() {
    let _ = env_logger::builder().is_test(true).try_init();
    // Test code with logging
}
```

## Resources

- [Rust Testing Guide](https://doc.rust-lang.org/book/ch11-00-testing.html)
- [Bevy Testing](https://bevyengine.org/learn/book/tests/)
- [Navigation Test Plan](../../navigation/TEST_PLAN.md)
