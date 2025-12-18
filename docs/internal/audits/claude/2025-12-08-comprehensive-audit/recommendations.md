# Prioritized Recommendations - Scarab Audit December 2025

## Immediate Priority (This Sprint)

### 1. Fix Window Resize Status Bar Bug
**File:** `scarab-client/src/ipc.rs:362-363`
**Effort:** 15 minutes

```rust
// Change this:
let rows: u16 = (event.height / cell_height).floor() as u16;

// To this:
use crate::ui::STATUS_BAR_HEIGHT;
let available_height = event.height - STATUS_BAR_HEIGHT;
let rows: u16 = (available_height / cell_height).floor() as u16;
```

### 2. Add Status Bar Tab Labels
**File:** `scarab-client/src/ui/status_bar.rs`
**Effort:** 30 minutes

Add static tabs with labels "meta", "phage", "tolaria" to status bar.

### 3. Fix Theme Merge Logic
**File:** `scarab-config/src/config.rs:78-88`
**Effort:** 20 minutes

Move telemetry assignment outside the navigation conditional:

```rust
// Fix line 78 - remove empty if block
// Fix line 88 - move outside else block
if other.telemetry != TelemetryConfig::default() {
    self.telemetry = other.telemetry;
}
```

---

## High Priority (Next Sprint)

### 4. Implement Theme Resolver
**Files:** New `scarab-config/src/theme_resolver.rs`
**Effort:** 2-3 hours

Create a `ThemeResolver` that:
1. Maps theme names ("slime", "dracula", "nord") to color palettes
2. Loads predefined themes from scarab-themes crate
3. Applies theme overrides to ColorConfig

### 5. Replace Lock Unwraps
**Files:** Multiple daemon files
**Effort:** 1-2 hours

Replace all `.unwrap()` on Mutex locks with proper error handling:

```rust
// Instead of:
let mut writer_lock = writer_arc.lock().unwrap();

// Use:
let mut writer_lock = match writer_arc.lock() {
    Ok(guard) => guard,
    Err(poisoned) => {
        log::warn!("Lock poisoned, recovering");
        poisoned.into_inner()
    }
};
```

### 6. Implement Semantic Zones
**File:** `scarab-daemon/src/ipc.rs:985-999`
**Effort:** 3-4 hours

Implement the four zone handlers:
- `ZonesRequest` - Send current zones to client
- `CopyLastOutput` - Copy last command output
- `SelectZone` - Handle zone selection
- `ExtractZoneText` - Extract zone text content

### 7. Add Bounded Input Channel
**File:** `scarab-daemon/src/main.rs:286`
**Effort:** 15 minutes

```rust
// Change:
let (input_tx, mut input_rx) = mpsc::unbounded_channel::<Vec<u8>>();

// To:
let (input_tx, mut input_rx) = mpsc::channel::<Vec<u8>>(1024);
```

---

## Medium Priority (Future Sprints)

### 8. Implement Shared Memory Seqlock
**Files:** `scarab-daemon/src/main.rs`, `scarab-protocol/src/lib.rs`
**Effort:** 4-6 hours

Replace current unsafe writes with seqlock pattern:
1. Client reads sequence before and after reading data
2. If sequence changed, retry read
3. Daemon increments sequence before AND after writes

### 9. Implement Status Bar Styling
**File:** `scarab-client/src/ui/status_bar.rs:310-334`
**Effort:** 2-3 hours

Add support for:
- Foreground/background colors
- Bold/italic styling
- Underline/strikethrough

### 10. Add Plugin Host Theme Bindings
**File:** `scarab-plugin-api/src/host_bindings.rs`
**Effort:** 2 hours

Add methods:
```rust
fn apply_theme(&mut self, theme_name: &str) -> Result<()>;
fn set_palette_color(&mut self, color_name: &str, value: &str) -> Result<()>;
fn get_current_theme(&self) -> String;
```

### 11. Add Config Validation on Load
**File:** `scarab-config/src/loader.rs:36-46`
**Effort:** 15 minutes

```rust
pub fn load(&self) -> Result<ScarabConfig> {
    let mut config = self.load_global()?;
    if let Some(local) = self.load_local()? {
        config.merge(local);
    }
    ConfigValidator::validate(&config)?;  // Add this
    Ok(config)
}
```

### 12. Fix Image Cursor Movement
**File:** `scarab-daemon/src/vte.rs:815`
**Effort:** 1 hour

Calculate actual cursor Y movement based on image cell dimensions.

---

## Testing Improvements

### 13. Add Integration Tests for IPC
**File:** New `scarab-daemon/src/ipc_tests.rs`
**Effort:** 3-4 hours

Cover:
- Message routing
- PTY handle management
- Client registration/unregistration

### 14. Create Dedicated Test Directories
**Crates:** scarab-clipboard, scarab-mouse, scarab-panes
**Effort:** 2-3 hours per crate

### 15. Replace Placeholder Smoke Tests
**Files:** `*/tests/smoke_tests.rs`
**Effort:** 2 hours

Replace `assert!(true)` with actual functional tests.

### 16. Fix Coverage Measurement
**File:** `.github/workflows/ci.yml`
**Effort:** 30 minutes

Include integration tests in coverage:
```yaml
cargo tarpaulin --workspace --timeout 300 --out Xml
# Remove: --exclude-files 'tests/*'
```

---

## Documentation Improvements

### 17. Document Per-Pane Zone Tracking
**File:** `scarab-daemon/src/vte.rs`
**Effort:** 15 minutes

Add comment explaining OSC 133 prompt markers are per-pane, not session-wide.

### 18. Add EventRegistry Migration Guide
**File:** `scarab-plugin-api/src/events.rs`
**Effort:** 30 minutes

Document replacement approach for deprecated EventRegistry.

---

## Release Plan

After implementing immediate priorities:
1. Bump version to `0.2.0-alpha.15`
2. Update CHANGELOG.md
3. Create release notes highlighting fixes

After high priority items:
1. Bump version to `0.2.0-beta.1`
2. Comprehensive testing pass
3. Documentation updates
