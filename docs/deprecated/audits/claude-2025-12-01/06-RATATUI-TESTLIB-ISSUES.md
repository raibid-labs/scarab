# Ratatui-testlib Feature Requests
**Date:** December 1, 2025
**Context:** Scarab Terminal Emulator Testing Requirements

---

## Overview

Ratatui-testlib is currently designed for **PTY-based terminal testing** (daemon/VTE testing). However, Scarab's client uses **Bevy ECS** for GUI rendering, creating a gap in testability.

**These feature requests would enable Scarab (and other Bevy+Ratatui apps) to fully test their frontends.**

---

## Issue #1: Bevy ECS Integration Support

### Title
**[Feature Request] Add Bevy ECS integration for testing Bevy+Ratatui applications**

### Description

**Problem:**
Many modern TUI applications use Bevy as their rendering engine (e.g., Scarab, dgx-pixels). Currently, ratatui-testlib can test PTY output but cannot test the Bevy ECS layer, leaving a significant testing gap for hybrid Bevy+Ratatui apps.

**Proposed Solution:**
Add a `BevyTuiTestHarness` that integrates both PTY testing and Bevy ECS querying.

**API Sketch:**
```rust
pub struct BevyTuiTestHarness {
    pty: PtySession,
    bevy_app: App, // Bevy ECS world
    screen: Screen,
}

impl BevyTuiTestHarness {
    pub fn new_with_bevy() -> Result<Self>;

    // Bevy-specific methods
    pub fn query<T: Component>(&self) -> Vec<&T>;
    pub fn assert_component_exists<T: Component>(&self);
    pub fn get_component<T: Component>(&self) -> Option<&T>;
    pub fn update_bevy(&mut self);

    // Hybrid assertions
    pub fn assert_component_matches_screen<T>(&self) -> Result<()>;
}
```

**Usage Example:**
```rust
#[test]
fn test_command_palette_renders() {
    let mut test = BevyTuiTestHarness::new_with_bevy()?;

    // Trigger UI event
    test.bevy_app.send_event(OpenCommandPalette);
    test.update_bevy()?;

    // Assert: Bevy component exists
    test.assert_component_exists::<CommandPaletteMarker>();

    // Assert: Screen shows it
    test.assert_screen_contains("Search commands...")?;
}
```

**Benefits:**
- Enables frontend testing for Bevy+Ratatui apps
- Bridges PTY testing and ECS testing
- Reusable for growing Bevy TUI ecosystem

**Alternatives Considered:**
- Build separate harness in Scarab (doesn't benefit ecosystem)
- Use only PTY testing (leaves ECS untested)

**Related Projects:**
- Scarab Terminal: https://github.com/raibid-labs/scarab
- dgx-pixels: https://github.com/raibid-labs/dgx-pixels
- bevy_ratatui: https://github.com/joshka/bevy_ratatui

**Priority:** HIGH (blocks frontend testing for Bevy TUI apps)

---

## Issue #2: Headless Mode Testing

### Title
**[Feature Request] Support headless testing without display server**

### Description

**Problem:**
Current implementation may require a display server (X11/Wayland) for certain operations. CI environments often run headless, making automated testing difficult.

**Proposed Solution:**
Ensure ratatui-testlib can run in fully headless environments:
- Mock any display-dependent operations
- Use Bevy's `MinimalPlugins` instead of `DefaultPlugins`
- Provide `--no-display` feature flag

**Test Case:**
```bash
# Should work in Docker without DISPLAY
docker run --rm rust:latest cargo test --features bevy,headless
```

**Priority:** HIGH (required for CI/CD)

---

## Issue #3: UI Component Position Assertions

### Title
**[Feature Request] Add assertions for UI component positioning and layout**

### Description

**Problem:**
When testing TUIs with complex layouts (tab bars, overlays, panels), developers need to verify that components render in the correct positions and don't overlap incorrectly.

**Proposed API:**
```rust
pub trait PositionAssertions {
    fn assert_within_bounds(&self, component: ComponentId, bounds: Rect) -> Result<()>;
    fn assert_at_position(&self, component: ComponentId, x: u16, y: u16) -> Result<()>;
    fn assert_no_overlap(&self, c1: ComponentId, c2: ComponentId) -> Result<()>;
    fn assert_aligned(&self, c1: ComponentId, c2: ComponentId, axis: Axis) -> Result<()>;
}
```

**Use Case:**
```rust
#[test]
fn test_tab_bar_at_bottom() {
    let test = BevyTuiTestHarness::new()?;

    let tab_bar = test.get_component_id::<TabBar>();
    let screen_height = test.screen_height();

    // Assert: Tab bar is at bottom (last 2 rows)
    test.assert_at_position(tab_bar, 0, screen_height - 2)?;
}

#[test]
fn test_overlay_within_preview_area() {
    let test = BevyTuiTestHarness::new()?;

    let overlay = test.get_component_id::<ImageOverlay>();
    let preview = Rect::new(10, 5, 50, 20);

    // Assert: Overlay stays within preview bounds
    test.assert_within_bounds(overlay, preview)?;
}
```

**Priority:** MEDIUM (quality of life for complex UIs)

---

## Issue #4: Snapshot Testing for Bevy Components

### Title
**[Feature Request] Snapshot testing for Bevy ECS component state**

### Description

**Problem:**
While ratatui-testlib supports terminal screen snapshots (via `insta`), Bevy components (positions, sizes, styles) aren't captured. This makes regression testing difficult for UI layout changes.

**Proposed Solution:**
Add component state serialization and snapshot testing:

```rust
#[derive(Serialize)]
struct ComponentSnapshot {
    component_type: String,
    position: (f32, f32),
    size: (f32, f32),
    visibility: bool,
    // ... other properties
}

impl BevyTuiTestHarness {
    pub fn snapshot_components<T: Component>(&self) -> Vec<ComponentSnapshot>;
}
```

**Usage:**
```rust
#[test]
fn test_ui_layout() {
    let test = setup_test_harness();

    let snapshot = test.snapshot_components::<Node>();
    insta::assert_json_snapshot!(snapshot);
}
```

**Benefits:**
- Catch unintended UI layout changes
- Visual review of component changes in PRs
- Regression prevention

**Priority:** MEDIUM (nice to have)

---

## Issue #5: Performance Benchmarking Support

### Title
**[Feature Request] Add performance profiling and benchmarking utilities**

### Description

**Problem:**
Scarab needs to ensure UI rendering performance doesn't regress (60+ FPS target). Need tools to benchmark rendering performance within test harness.

**Proposed API:**
```rust
impl BevyTuiTestHarness {
    pub fn benchmark_rendering(&mut self, iterations: usize) -> BenchmarkResults;
    pub fn profile_update_cycle(&mut self) -> ProfileResults;
    pub fn assert_fps(&self, min_fps: f64) -> Result<()>;
}

pub struct BenchmarkResults {
    pub avg_frame_time_ms: f64,
    pub p50_ms: f64,
    pub p95_ms: f64,
    pub p99_ms: f64,
}
```

**Usage:**
```rust
#[test]
fn test_rendering_performance() {
    let mut test = BevyTuiTestHarness::new()?;

    test.load_full_screen_grid();

    let results = test.benchmark_rendering(1000);
    assert!(results.avg_frame_time_ms < 16.7); // 60 FPS
}
```

**Priority:** LOW (can be implemented separately)

---

## Summary

### Critical for Scarab (Please Prioritize)

1. ✅ **Bevy ECS Integration** - Enables frontend testing entirely
2. ✅ **Headless Mode** - Required for CI/CD
3. ⚠️ **Position Assertions** - Quality of life, can work around

### Nice to Have

4. ⏸️ **Component Snapshots** - Can use manual JSON serialization
5. ⏸️ **Performance Benchmarking** - Can use Criterion separately

---

## Contribution Offer

The Scarab team is willing to:
- ✅ Contribute Bevy integration code upstream
- ✅ Write documentation and examples
- ✅ Maintain Bevy-specific features
- ✅ Help test cross-platform

**Contact:** https://github.com/raibid-labs/scarab/issues

---

**Document:** 06-RATATUI-TESTLIB-ISSUES.md
**Status:** Ready to file upstream
**Priority:** Issues #1 and #2 are CRITICAL for Scarab testing
