# Detailed Issues - Scarab Audit December 2025

## Critical Issues

### 1. Window Resize Not Accounting for Status Bar
**File:** `scarab-client/src/ipc.rs:362-363`
**Severity:** CRITICAL

The `handle_window_resize()` function doesn't subtract STATUS_BAR_HEIGHT when calculating terminal rows:

```rust
let cols: u16 = (event.width / cell_width).floor() as u16;
let rows: u16 = (event.height / cell_height).floor() as u16;  // Missing status bar adjustment
```

**Impact:** When resizing the window, calculated row count includes space for status bar, causing text to render into the status bar area.

**Fix:**
```rust
let available_height = event.height - STATUS_BAR_HEIGHT;
let rows: u16 = (available_height / cell_height).floor() as u16;
```

---

### 2. Unsafe Shared Memory TOCTOU Race
**File:** `scarab-daemon/src/main.rs:277-303`
**Severity:** CRITICAL

Shared memory writes lack synchronization beyond atomic sequence number:

```rust
unsafe {
    let state = &mut *shm;
    state.cells[shm_idx] = self.grid.cells[local_idx];  // Direct write
    state.sequence_number = new_seq;
}
```

Client reads while daemon writes, creating TOCTOU race condition.

**Recommendation:** Use seqlock pattern or double-buffering.

---

### 3. Missing Semantic Zones Implementation
**File:** `scarab-daemon/src/ipc.rs:985-999`
**Severity:** HIGH

Four zone handlers are unimplemented:
- `ZonesRequest` (line 987)
- `CopyLastOutput` (line 991)
- `SelectZone` (line 995)
- `ExtractZoneText` (line 999)

---

### 4. Theme Merge Logic Bug
**File:** `scarab-config/src/config.rs:78,88`
**Severity:** HIGH

The `ScarabConfig::merge()` method has broken control flow:

```rust
// Line 78 - DEAD CODE
if other.telemetry != TelemetryConfig::default() {}

// Line 88 - INCORRECT PLACEMENT
self.telemetry = other.telemetry;  // Only executes if navigation == default!
```

Theme settings may be lost during config merge.

---

### 5. Missing Theme Resolver
**Files:** `scarab-config/src/config.rs:153-185`
**Severity:** HIGH

`ColorConfig` stores theme name but no mechanism exists to:
1. Resolve theme names to actual color palettes
2. Load predefined themes (dracula, nord, monokai, slime)
3. Apply theme overrides to the palette

The `theme` field is purely informational.

---

### 6. Status Bar Tabs Need Labels
**File:** `scarab-client/src/ui/dock.rs`
**Severity:** MEDIUM (User Request)

Current dock items show plugin info dynamically. User requests static tabs labeled:
- "meta"
- "phage"
- "tolaria"

---

## High Severity Issues

### 7. Lock Poisoning Panics
**Files:** Multiple
- `scarab-daemon/src/main.rs:379` - PTY writer lock unwrap
- `scarab-daemon/src/orchestrator.rs:196` - PTY reader lock unwrap
- `scarab-daemon/src/session/manager.rs:190` - Tab switching unwrap

All use `.unwrap()` which panics on poisoned locks.

---

### 8. Plugin Manager Deadlock Risk
**File:** `scarab-daemon/src/main.rs:357-393`

Plugin manager lock held during async operations:
```rust
let mut pm = pm_input.lock().await;
match pm.dispatch_input(&data).await { ... }
```

---

### 9. Unbounded Message Queue
**File:** `scarab-daemon/src/main.rs:286`

```rust
let (input_tx, mut input_rx) = mpsc::unbounded_channel::<Vec<u8>>();
```

Could cause memory exhaustion under rapid paste operations.

---

### 10. Image Cursor Movement Not Calculated
**File:** `scarab-daemon/src/vte.rs:815`

```rust
// TODO: Calculate actual cursor movement based on image size
self.cursor_y += 1;
```

Always moves cursor one line regardless of image dimensions.

---

### 11. Image Percentage Sizing Hardcoded
**File:** `scarab-daemon/src/images/placement.rs:87,102`

```rust
super::ImageSize::Percent(_) => 10,  // Hardcoded
super::ImageSize::Percent(_) => 5,   // Hardcoded
```

---

### 12. Missing Plugin Host Bindings for Themes
**File:** `scarab-plugin-api/src/host_bindings.rs`

No methods for:
- `apply_theme()`
- `set_palette_color()`
- Theme/color manipulation

---

### 13. Missing Config Validation on Load
**File:** `scarab-config/src/loader.rs:36-46`

`ConfigLoader::load()` doesn't call `ConfigValidator::validate()`.

---

### 14. Status Bar Color Rendering Incomplete
**File:** `scarab-client/src/ui/status_bar.rs:310-334`

Foreground/background color, bold/italic styling all marked "not yet supported".

---

## Medium Severity Issues

### 15. Compositor Always Blits
**File:** `scarab-daemon/src/main.rs:433-439`

Always blits to shared memory every frame rather than only when dirty.

---

### 16. Plugin Timeout Not Enforced
**File:** `scarab-daemon/src/plugin_manager/mod.rs:141`

`hook_timeout` field exists but not consistently used.

---

### 17. Fusabi VM Thread-Local Storage
**File:** `scarab-daemon/src/plugin_manager/fusabi_adapter.rs:19-21`

VM uses thread-local storage which may leak resources.

---

### 18. Hardcoded Plugin Failure Threshold
**File:** `scarab-daemon/src/plugin_manager/mod.rs:45`

`max_failures: 3` should be configurable.

---

### 19. EventRegistry Deprecated Without Migration
**File:** `scarab-plugin-api/src/lib.rs:38-41`

Marked deprecated but no migration guide.

---

### 20. StatusBarItem Missing Colors
**File:** `scarab-plugin-api/src/types.rs:38-63`

No color/style fields in StatusBarItem struct.

---

### 21. Missing Plugin Hooks
**File:** `scarab-plugin-api/src/plugin.rs:15-112`

Missing hooks:
- `on_theme_changed()`
- `on_config_reloaded()`
- `on_colors_changed()`

---

### 22. Incomplete Fusabi Config Extraction
**File:** `scarab-config/src/fusabi_loader.rs:376-381`

Plugin-specific configs discarded during extraction.

---

### 23. Color Validation Gaps
**File:** `scarab-config/src/validation.rs:114-141`

Invalid hex colors like `#zzz000` accepted.

---

## Low Severity Issues

### 24. Socket Race Condition
**File:** `scarab-daemon/src/ipc.rs:163-165`

Multiple daemon instances can interfere with socket files.

---

### 25. Font Fallback Silent
**File:** `scarab-client/src/rendering/text.rs:37-41`

Logs error but continues if no fonts loaded.

---

### 26. Grid Y-Position Comment Confusing
**File:** `scarab-client/src/integration.rs:120-126`

Comment states grid needs status bar accounting but code doesn't adjust.

---

### 27. Shift+Letter Handling Implicit
**File:** `scarab-client/src/ipc.rs:303-339`

Relies on Bevy's implicit behavior rather than explicit handling.

---

### 28. Missing Debug Assertions
**File:** `scarab-daemon/src/vte.rs:288`

Grid access bounds-checked but could use debug_assert! documentation.

---

### 29-48. Various Minor Issues
See testing-gaps.md for testing-related issues.
