# ratatui-testlib Integration Plan

## Overview
Plan for extending ratatui-testlib coverage as upstream harness (#25) evolves.

## Current Status
- PTY-level smoke tests running in CI
- BevyTuiTestHarness stubs implemented
- Upstream issue #25 filed for hybrid harness

## Phase 1: PTY-only (Current) ✅
- [x] PTY output passthrough
- [x] Grid text rendering verification
- [x] Cursor position tracking
- [x] Wait-for-text conditions
- [x] Command sequence execution
- [x] Basic navigation hotkey send

## Phase 2: ECS Integration (Pending Upstream #25)
- [ ] Query `NavState` resource for mode assertions
- [ ] Query `NavHint` components for hint labels
- [ ] Query `PromptMarkers` for OSC 133 zones
- [ ] Access Bevy `World` for arbitrary ECS queries
- [ ] Validate navigation focus transitions

## Phase 3: SharedState/Graphics
- [ ] Read `SharedState` grid directly (bypass PTY)
- [ ] Assert Sixel image placements and bounds
- [ ] Assert Kitty protocol state
- [ ] Assert iTerm2 inline image positions
- [ ] Validate image scaling and cropping

## Phase 4: Performance
- [ ] Record input→render latency per frame
- [ ] Measure frame time distribution
- [ ] Detect regressions vs baseline
- [ ] Integration with `cargo bench`

## Upstream Tracking
- Issue: ratatui/ratatui-testlib#25
- Status: Filed, awaiting implementation
- Scarab requirements:
  - Bevy ECS access hooks
  - SharedState read capability
  - Image protocol introspection

## Test Files
- `crates/scarab-client/tests/ratatui_testlib_smoke.rs` - PTY tests
- `crates/scarab-client/tests/bevy_harness_stubs.rs` - ECS stubs
