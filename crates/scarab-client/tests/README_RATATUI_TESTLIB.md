# Ratatui-testlib Smoke Tests

This directory contains smoke tests using ratatui-testlib v0.1.0 for PTY-level fidelity testing of Scarab's terminal emulation.

## Test File

- **`ratatui_testlib_smoke.rs`** - 6 smoke tests for PTY-level terminal emulation

## Running the Tests

The tests are currently marked with `#[ignore]` because they require:
1. A compiled `scarab-daemon` binary
2. PTY support (Linux/macOS)
3. Longer execution time suitable for integration testing

To run them:

```bash
# Run all ratatui-testlib tests
cargo test --package scarab-client --test ratatui_testlib_smoke -- --ignored

# Run a specific test
cargo test --package scarab-client --test ratatui_testlib_smoke test_pty_output_passthrough -- --ignored

# Show output while running
cargo test --package scarab-client --test ratatui_testlib_smoke -- --ignored --nocapture
```

## Test Coverage

### ✅ Implemented Tests (6 tests)

1. **`test_pty_output_passthrough`** - Verifies text sent through daemon PTY appears in terminal grid
2. **`test_grid_text_rendering`** - Tests multi-line text rendering at correct grid positions
3. **`test_nav_hotkey_sequences`** - Tests navigation hotkey ('f') produces expected behavior
4. **`test_cursor_position_tracking`** - Verifies cursor moves correctly as text is typed
5. **`test_wait_for_text_condition`** - Tests polling for text appearance with wait_for
6. **`test_multiple_commands_sequence`** - Verifies multiple commands execute correctly

### 🚧 Blocked Tests (Awaiting ratatui-testlib Phase 4)

The following test scenarios are documented in the test file but cannot be implemented yet:

1. **Bevy ECS Component Querying**
   - Query `FocusableRegion` components
   - Verify `NavHint` entities spawn in hint mode
   - Access `NavState` resource directly

2. **SharedMemory Direct Access**
   - Read `scarab-protocol::SharedState` directly
   - Verify grid cells update correctly
   - Test sequence number synchronization

3. **Navigation State Verification**
   - Verify `NavMode` changes
   - Test prompt navigation (Ctrl+Up/Down)
   - Validate `PromptMarkers` resource

4. **Coordinate Conversion**
   - Test grid → world coordinate conversion
   - Verify `screen_position` calculations
   - Test with different `TerminalMetrics`

## Current Limitations

### ratatui-testlib v0.1.0 Status

- **Bevy ECS Integration**: Placeholder only (Phase 4 not yet implemented)
- **Testing Mode**: Process-based only (spawn daemon in PTY)
- **State Access**: Limited to PTY output, no direct SharedMemory access

### Upstream Feature Requests

See test file for detailed documentation of gaps:
- Bevy component querying
- Hybrid process testing (daemon in PTY + client in-process)
- SharedMemory protocol integration
- Navigation event verification

### Roadmap

According to ratatui-testlib's roadmap:
- **Phase 4** (Bevy ECS Integration) - Estimated 2-3 weeks
- **Phase 6** (Polish + Docs) - Final MVP release

Once Phase 4 is complete, we can implement the blocked test scenarios.

## Test Strategy

### Current Approach (PTY-Based)

```
┌─────────────────┐
│ Test Harness    │
│ (TuiTestHarness)│
└────────┬────────┘
         │
         ├─ Spawn scarab-daemon in PTY
         ├─ Send keyboard input
         ├─ Read terminal output
         └─ Assert on screen contents
```

### Future Approach (Bevy-Integrated)

```
┌──────────────────────┐
│ BevyTuiTestHarness   │
│ (Bevy ECS + PTY)     │
└─────────┬────────────┘
          │
          ├─ Daemon in PTY (as subprocess)
          ├─ Client in-process (Bevy ECS access)
          ├─ Query components directly
          ├─ Access SharedMemory
          └─ Verify navigation state
```

## Integration with Existing Tests

Scarab has extensive test coverage:

- **Headless Tests** (`headless_poc.rs`) - Bevy MinimalPlugins without rendering
- **Navigation Tests** (`src/navigation/tests.rs`) - Unit tests for NavState/FocusableRegion
- **E2E Tests** (`tests/e2e/`) - Full daemon + client integration tests
- **Ratatui-testlib Smoke Tests** (this file) - PTY-level fidelity tests

### When to Use Each

| Test Type | Use For | Speed | Fidelity |
|-----------|---------|-------|----------|
| Unit | Component logic | Very Fast | Low |
| Headless | Bevy ECS state | Fast | Medium |
| E2E | Full stack | Slow | High |
| ratatui-testlib | PTY behavior | Medium | Very High |

## Resources

- **ratatui-testlib**: https://github.com/raibid-labs/ratatui-testlib
- **Roadmap**: https://github.com/raibid-labs/ratatui-testlib/blob/main/docs/ROADMAP.md
- **Phase 4 Tracking**: https://github.com/raibid-labs/ratatui-testlib/issues/TBD

## Contributing

To add more smoke tests:

1. Follow the pattern in `ratatui_testlib_smoke.rs`
2. Use `#[ignore]` for tests requiring daemon binary
3. Document any gaps blocking full implementation
4. Consider contributing upstream feature requests to ratatui-testlib

---

**Status**: ✅ 6 smoke tests implemented and compiling
**Blocked**: Bevy ECS integration tests (awaiting ratatui-testlib Phase 4)
