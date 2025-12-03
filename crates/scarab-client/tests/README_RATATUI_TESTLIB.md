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

### ✅ Implemented Tests (11 tests - expanded in Issue #63)

#### Core PTY Tests (6 tests)
1. **`test_pty_output_passthrough`** - Verifies text sent through daemon PTY appears in terminal grid
2. **`test_grid_text_rendering`** - Tests multi-line text rendering at correct grid positions
3. **`test_nav_hotkey_sequences`** - Tests navigation hotkey ('f') produces expected behavior
4. **`test_cursor_position_tracking`** - Verifies cursor moves correctly as text is typed
5. **`test_wait_for_text_condition`** - Tests polling for text appearance with wait_for
6. **`test_multiple_commands_sequence`** - Verifies multiple commands execute correctly

#### Graphics Protocol Tests (3 tests - NEW in #63)
7. **`test_sixel_sequence_handling`** - Tests Sixel DCS sequence parsing without crashes
8. **`test_kitty_graphics_basic`** - Tests Kitty graphics protocol basic PNG transfer
9. **`test_kitty_graphics_chunked_transfer`** - Tests multi-chunk Kitty graphics transmission

#### Navigation System Tests (2 tests - NEW in #63)
10. **`test_nav_hint_mode`** - Tests entering/exiting navigation hint mode with 'f' and Escape
11. **`test_pane_navigation`** - Tests pane navigation keyboard shortcuts (Ctrl+H/L)

### 🚧 Blocked Tests (Awaiting ratatui-testlib Phase 4)

The following test scenarios are documented in the test file but cannot be implemented yet. See the inline comments in `ratatui_testlib_smoke.rs` for code examples of what we want to test.

#### 1. **Bevy ECS Component Querying** (GAP 1)
   - Query `FocusableRegion` components to verify focusables were detected
   - Verify `NavHint` entities spawn in hint mode with correct labels
   - Access `NavState` resource directly to check current mode
   - **Blocked by**: Need `BevyTuiTestHarness::query::<T>()` and `resource::<T>()`
   - **Upstream issue**: TBD (see ratatui-testlib roadmap Phase 4)

#### 2. **SharedMemory Direct Access** (GAP 2)
   - Read `scarab-protocol::SharedState` directly from shared memory segment
   - Verify grid cells update correctly after VTE parsing
   - Test sequence number synchronization between daemon and client
   - **Blocked by**: Need hybrid PTY + SharedMemory harness
   - **Upstream issue**: Requires custom harness for Scarab's split architecture

#### 3. **Navigation State Verification** (GAP 3)
   - Verify `NavMode` changes when 'f' is pressed (Normal → Hints)
   - Test prompt navigation with Ctrl+Up/Down jumps
   - Validate `PromptMarkers` resource contains detected prompt positions
   - Verify scroll position changes after prompt navigation
   - **Blocked by**: Need Bevy resource access and event inspection
   - **Current workaround**: Tests verify daemon responsiveness only

#### 4. **Coordinate Conversion Verification** (GAP 4)
   - Test grid → world coordinate conversion (terminal cells to screen pixels)
   - Verify `FocusableRegion.screen_position` is calculated correctly
   - Test with different `TerminalMetrics` (font sizes, cell dimensions)
   - Validate hint label positioning matches focusable bounds
   - **Blocked by**: Need in-process client testing with Bevy components

#### 5. **Graphics Protocol Deep Verification** (NEW - Issue #63)
   - Verify Sixel image was decoded and stored in image buffer
   - Check Kitty graphics image registry for correct image ID
   - Validate image metadata (width, height, format, position)
   - Verify image render commands added to rendering pipeline
   - Check GPU texture upload and atlas packing
   - **Blocked by**: Need SharedMemory + ImageBuffer access
   - **Current workaround**: Tests verify no crash and daemon remains responsive

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


## Upstream Dependency Tracking

### ratatui-testlib Requirements

Scarab requires the following features from ratatui-testlib to complete test coverage:

#### 1. BevyTuiTestHarness (Phase 4)

**Status**: Not yet implemented

**Required APIs**:
```rust
// Resource querying
harness.query_resource::<NavState>()
harness.query_resource::<TerminalMetrics>()
harness.query_resource::<PromptMarkers>()

// Component querying
harness.query_components::<&NavHint>()
harness.query_components::<&FocusableRegion>()

// Schedule execution
harness.run_schedule(Update)
```

**Blocked tests**: See `bevy_harness_stubs.rs` for 10 stub tests

#### 2. Hybrid PTY + In-Process Harness

**Status**: Not yet implemented

**Required APIs**:
```rust
// Spawn daemon in PTY, client in-process
let harness = HybridTestHarness::new()
    .with_daemon_pty()
    .with_client_inprocess()
    .build()?;

// Send to daemon PTY
harness.pty_write("echo hello\r")?;

// Query client ECS
let nav_state = harness.client().query_resource::<NavState>()?;
```

**Blocked tests**: `test_shared_state_grid_access` and integration scenarios

#### 3. SharedMemory Direct Access

**Status**: Not yet implemented

**Required APIs**:
```rust
// Read shared memory directly
let shared = harness.shared_state()?;
assert_eq!(shared.grid[0][0].character, 'h');
assert_eq!(shared.sequence_number.load(Ordering::SeqCst), expected);
```

**Blocked tests**: Grid synchronization and zero-copy validation

#### 4. Graphics Protocol Support

**Status**: Not yet implemented

**Required APIs**:
```rust
// Query image placements
let placements = harness.image_placements()?;
assert_eq!(placements[0].protocol, ImageProtocol::Kitty);
assert_eq!(placements[0].bounds, Rect::new(0, 0, 100, 50));
```

**Blocked tests**: Kitty/Sixel/iTerm2 protocol verification

#### 5. Performance Measurement Hooks

**Status**: Not yet implemented

**Required APIs**:
```rust
// Measure input-to-frame latency
let latency = harness.measure_latency(|| {
    harness.pty_write("x")?;
}, |state| state.frame_count > initial)?;
assert!(latency < Duration::from_millis(16));
```

**Blocked tests**: Performance regression detection

### Upstream Issue

**Scarab tracking**: https://github.com/raibid-labs/scarab/issues/64

**Template for upstream issue** (to be filed on ratatui-testlib):
```
Title: Bevy+Scarab hybrid harness with ECS/query + SharedState access

Summary: Scarab needs to validate nav/graphics end-to-end. Current TuiTestHarness
is PTY-only; BevyTuiTestHarness is a stub. Request:
- In-process harness that can run Bevy schedules (with or without bevy_ratatui)
- Optional child daemon in PTY for hybrid testing
- APIs to query resources/components (NavState, NavHint, PromptMarkers, TerminalMetrics)
- SharedState reader for mmap-backed grid assertions
- Graphics assertions for Sixel/Kitty/iTerm2 placements
- Timing hooks to measure input→frame latency
- Minimal example and CI-friendly headless mode
```

### Test Stubs Prepared

File: `bevy_harness_stubs.rs` contains 10 test stubs ready for implementation:

1. `test_query_nav_state_resource` - NavState resource access
2. `test_query_nav_hint_components` - NavHint entity queries
3. `test_query_focusable_region_components` - FocusableRegion queries
4. `test_shared_state_grid_access` - Direct grid assertions
5. `test_query_terminal_metrics_resource` - TerminalMetrics access
6. `test_query_prompt_markers_resource` - Shell integration markers
7. `test_grid_to_world_coordinate_conversion` - Coordinate math
8. `test_graphics_protocol_image_placement` - Graphics protocols
9. `test_measure_input_to_frame_latency` - Performance validation
10. `test_navigation_mode_lifecycle` - Mode transitions

All stubs are marked `#[ignore]` with tracking information and expected API documentation.


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

## Issue #63 Implementation Summary

**Status**: ✅ 11 smoke tests implemented and compiling (5 new tests added)
**Blocked**: Bevy ECS integration tests (awaiting ratatui-testlib Phase 4)

### What Was Added (Issue #63)

1. **Graphics Protocol Tests** (3 new tests):
   - `test_sixel_sequence_handling` - Validates Sixel DCS parsing
   - `test_kitty_graphics_basic` - Validates Kitty APC basic transfer
   - `test_kitty_graphics_chunked_transfer` - Validates Kitty multi-chunk transmission

2. **Navigation System Tests** (2 new tests):
   - `test_nav_hint_mode` - Validates hint mode activation with 'f' key
   - `test_pane_navigation` - Validates pane navigation hotkeys

### Test Enablement Strategy

**All tests remain `#[ignore]`** because they require:
- Compiled `scarab-daemon` binary
- PTY support (Linux/macOS)
- Integration test execution environment

Tests can be enabled by running:
```bash
cargo test --package scarab-client --test ratatui_testlib_smoke -- --ignored
```

### Future Work (Once ratatui-testlib Phase 4 Ships)

When Bevy ECS integration is available, we can add:
1. **Deep verification** of graphics protocol - check image buffers, not just responsiveness
2. **ECS state inspection** for navigation - query NavStateRegistry, NavHint entities
3. **SharedMemory access** - directly verify grid cells and sequence numbers
4. **Event tracking** - verify EnterHintModeEvent, PaneFocusedEvent, etc.

See GAP 1-5 documentation above for detailed test scenarios we want to enable.

### Related Issues

- **Issue #63**: Expand ratatui-testlib coverage with Kitty/Sixel/nav assertions
- **ratatui-testlib Phase 4**: Bevy ECS Integration (upstream dependency)
- **GAP 2**: Custom harness for Scarab's split daemon/client architecture
