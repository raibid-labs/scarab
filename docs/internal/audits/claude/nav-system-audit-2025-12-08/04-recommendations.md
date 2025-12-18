# Detailed Recommendations

## Priority Matrix

| ID | Issue | Impact | Effort | Priority |
|----|-------|--------|--------|----------|
| R1 | Delete orphaned nav_input.rs types | High | Low | P0 |
| R2 | Implement NavAction handler | High | Low | P0 |
| R3 | Delete ui_stub.rs | Medium | Trivial | P0 |
| R4 | Add input system ordering | Medium | Medium | P1 |
| R5 | Remove debug eprintln statements | Low | Trivial | P1 |
| R6 | Add navigation integration tests | Medium | Medium | P1 |
| R7 | Implement FusabiTuiPlugin | Low | High | P2 |
| R8 | Enable StatusBar rich styling | Low | Medium | P2 |
| R9 | Implement missing commands | Low | Low | P2 |
| R10 | Unify input handling architecture | High | High | P3 |

---

## P0 - Critical (Do Immediately)

### R1: Delete Orphaned Navigation Types

**Problem:** `input/nav_input.rs` contains duplicate `NavMode` and `NavAction` types that conflict with `navigation/mod.rs`.

**Files to modify:**
- `crates/scarab-client/src/input/nav_input.rs`
- `crates/scarab-client/src/input/mod.rs`
- `crates/scarab-client/src/main.rs`

**Changes:**

1. **Delete from `input/nav_input.rs`:**
   - Lines 17-31: `NavMode` enum (use `navigation::NavMode` instead)
   - Lines 45-94: `NavAction` enum (rename or remove)
   - Lines 313-497: `NavInputRouter` struct and impl
   - Lines 500-562: `ModeStack` struct and impl
   - Lines 577-651: `route_nav_input` function

2. **Or: Rename to avoid collision:**
   ```rust
   // If keeping for future use:
   pub enum InputAction {  // Was NavAction
       EnterHintMode,
       EnterCopyMode,
       // ...
   }
   ```

3. **Update `main.rs` lines 218-219:**
   ```rust
   // DELETE these lines:
   .insert_resource(NavInputRouter::new(NavStyle::VimiumStyle))
   .insert_resource(ModeStack::new())
   ```

4. **Update `input/mod.rs` line 10:**
   ```rust
   // Remove orphaned exports:
   pub use nav_input::{
       // DELETE: route_nav_input, KeyBinding, ModeStack, Modifier, NavAction, NavInputRouter, NavMode, NavStyle,
       KeyBinding, Modifier,  // Keep only what's used
   };
   ```

**Estimated time:** 30 minutes

---

### R2: Implement NavAction Handler

**Problem:** `NavActionEvent` is defined but no system processes it.

**File to modify:** `crates/scarab-client/src/navigation/mod.rs`

**Add new system:**

```rust
/// System to handle navigation actions
fn handle_nav_actions(
    mut events: EventReader<NavActionEvent>,
    ipc: Option<Res<crate::ipc::IpcChannel>>,
) {
    for event in events.read() {
        match &event.action {
            NavAction::Open(url) => {
                // Open URL in default browser
                #[cfg(target_os = "linux")]
                {
                    let _ = std::process::Command::new("xdg-open")
                        .arg(url)
                        .spawn();
                }
                #[cfg(target_os = "macos")]
                {
                    let _ = std::process::Command::new("open")
                        .arg(url)
                        .spawn();
                }
            }
            NavAction::Click(col, row) => {
                // Send mouse click to terminal
                if let Some(ipc) = &ipc {
                    // Implementation depends on protocol
                    println!("Click at ({}, {}) - not yet implemented", col, row);
                }
            }
            NavAction::JumpPrompt(index) => {
                // Scroll to prompt - integrate with scrollback
                println!("Jump to prompt {} - not yet implemented", index);
            }
            NavAction::NextPane => {
                // Focus next pane in session
                println!("Next pane - not yet implemented");
            }
            NavAction::PrevPane => {
                // Focus previous pane
                println!("Prev pane - not yet implemented");
            }
            NavAction::NextTab => {
                // Switch to next tab
                println!("Next tab - not yet implemented");
            }
            NavAction::PrevTab => {
                // Switch to previous tab
                println!("Prev tab - not yet implemented");
            }
            NavAction::Cancel => {
                // Cancel current action (usually handled elsewhere)
            }
        }
    }
}
```

**Register in NavigationPlugin:**

```rust
impl Plugin for NavigationPlugin {
    fn build(&self, app: &mut App) {
        app
            // ... existing code ...
            .add_systems(
                Update,
                (
                    on_pane_created,
                    on_pane_focused,
                    on_pane_closed,
                    handle_nav_actions,  // ADD THIS
                ).in_set(NavSystemSet::Update),
            );
    }
}
```

**Estimated time:** 1 hour

---

### R3: Delete ui_stub.rs

**Problem:** Deprecated stub file causes confusion.

**Files to modify:**
- `crates/scarab-client/src/ui_stub.rs` - DELETE
- `crates/scarab-client/src/lib.rs`

**Changes:**

1. **Delete file:**
   ```bash
   rm crates/scarab-client/src/ui_stub.rs
   ```

2. **Update `lib.rs`:**
   ```rust
   // DELETE this line:
   mod ui_stub;
   ```

**Estimated time:** 5 minutes

---

## P1 - High Priority (This Week)

### R4: Add Input System Ordering

**Problem:** Multiple input handlers run in Update with no ordering.

**File to modify:** `crates/scarab-client/src/lib.rs` or create new plugin

**Add system sets:**

```rust
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum InputSystemSet {
    /// Navigation and mode handling (first)
    Navigation,
    /// Overlay/surface input (second)
    Surface,
    /// IPC to daemon (last)
    Daemon,
}
```

**Configure in main.rs:**

```rust
app.configure_sets(
    Update,
    (
        InputSystemSet::Navigation,
        InputSystemSet::Surface,
        InputSystemSet::Daemon,
    ).chain()
);
```

**Update plugin registrations:**

```rust
// In IpcPlugin
.add_systems(
    Update,
    (
        handle_keyboard_input,
        handle_character_input,
    ).in_set(InputSystemSet::Daemon),
)

// In RatatuiBridgePlugin
.add_systems(
    Update,
    handle_keyboard_input.in_set(InputSystemSet::Surface),
)
```

**Estimated time:** 2 hours

---

### R5: Remove Debug Statements

**Problem:** Debug output in production code.

**Files to modify:**
- `crates/scarab-client/src/ipc.rs` line 321
- `crates/scarab-client/src/input/nav_input.rs` lines 582-585

**Changes:**

```rust
// DELETE or gate behind feature:
#[cfg(feature = "debug-input")]
eprintln!("DEBUG char_input: sending {:?} ({:?})", s, bytes);
```

**Estimated time:** 10 minutes

---

### R6: Add Navigation Integration Tests

**Problem:** No tests verify end-to-end navigation flow.

**File to create:** `crates/scarab-client/tests/navigation_integration.rs`

**Test cases:**

```rust
#[test]
fn test_enter_hint_mode() {
    // Setup app with NavigationPlugin
    // Send EnterHintModeEvent
    // Verify focusables spawned
    // Verify hint overlays rendered
}

#[test]
fn test_hint_selection() {
    // Enter hint mode
    // Simulate hint key press
    // Verify NavActionEvent emitted
}

#[test]
fn test_mode_stack_persistence() {
    // Enter hint mode
    // Enter search mode (nested)
    // Press Escape
    // Verify back in hint mode
    // Press Escape
    // Verify back in normal mode
}

#[test]
fn test_pane_isolation() {
    // Create two panes
    // Enter hint mode in pane 1
    // Focus pane 2
    // Verify pane 2 is in normal mode
    // Focus pane 1
    // Verify pane 1 is still in hint mode
}
```

**Estimated time:** 4 hours

---

## P2 - Medium Priority (This Sprint)

### R7: Implement FusabiTuiPlugin

**Problem:** Plugin is a no-op stub.

**File to modify:** `crates/scarab-client/src/ui/fusabi_widgets.rs`

**Options:**
1. Implement actual Fusabi TUI integration
2. Remove from AdvancedUIPlugin bundle
3. Gate behind feature flag

**If removing:**

```rust
// In ui/mod.rs, remove from bundle:
app.add_plugins((
    LinkHintsPlugin,
    // ... other plugins ...
    // FusabiTuiPlugin,  // REMOVE
    StatusBarPlugin,
));
```

**Estimated time:** 1 day (implement) or 10 minutes (remove)

---

### R8: Enable StatusBar Rich Styling

**Problem:** Text styling (colors, bold, italic) disabled.

**File to modify:** `crates/scarab-client/src/ui/status_bar.rs`

**Current code (lines 308-342):**
```rust
// TODO: Apply styling to Text node
// Currently disabled pending Bevy 0.15 text API
```

**Enable with Bevy 0.15 API:**
```rust
fn create_styled_text(item: &RenderItem) -> Text {
    let mut sections = Vec::new();

    let style = TextFont {
        font_size: 14.0,
        ..default()
    };

    let color = match item.fg_color {
        Some(StatusColor::Rgb(r, g, b)) => Color::srgb_u8(r, g, b),
        Some(StatusColor::Ansi(ansi)) => ansi_to_bevy_color(ansi),
        None => Color::WHITE,
    };

    sections.push((
        item.text.clone(),
        TextStyle {
            font: default(),
            font_size: 14.0,
            color,
        },
    ));

    Text::from_sections(sections)
}
```

**Estimated time:** 2 hours

---

### R9: Implement Missing Commands

**Problem:** `reload_config` and `help` commands are stubs.

**File to modify:** `crates/scarab-client/src/ui/command_palette.rs`

**Implement reload_config:**
```rust
"reload_config" => {
    // Emit event to trigger config reload
    // Actual reload handled by config plugin
    events.send(ConfigReloadRequestedEvent);
}
```

**Implement help:**
```rust
"help" => {
    // Open help overlay or external documentation
    #[cfg(target_os = "linux")]
    {
        let _ = std::process::Command::new("xdg-open")
            .arg("https://scarab-terminal.dev/docs")
            .spawn();
    }
}
```

**Estimated time:** 1 hour

---

## P3 - Lower Priority (Backlog)

### R10: Unify Input Handling Architecture

**Problem:** Fragmented input handling across multiple systems.

**Proposed Architecture:**

```rust
/// Central input router that determines where input goes
#[derive(Resource)]
pub struct InputRouter {
    /// Current input destination
    mode: InputMode,
    /// Mode history for pop behavior
    stack: Vec<InputMode>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum InputMode {
    /// Normal terminal input - forward to daemon
    Terminal,
    /// Hint mode - consume hint characters
    Hints,
    /// Command palette - consume all input
    CommandPalette,
    /// Search mode - forward to search overlay
    Search,
    /// Copy mode - vim-like selection
    Copy,
    /// Focused surface (ratatui) - forward to surface
    Surface(Entity),
}

/// System that runs first and routes input
fn route_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut router: ResMut<InputRouter>,
    mut ipc_events: EventWriter<IpcInputEvent>,
    mut nav_events: EventWriter<NavInputEvent>,
    mut surface_events: EventWriter<SurfaceInputEvent>,
) {
    // Consume input based on mode
    // Only forward to appropriate handler
}
```

**Benefits:**
- Single source of truth for input routing
- Clear precedence rules
- Input consumption tracking
- Testable in isolation

**Estimated time:** 1-2 weeks

---

## Implementation Order

```
Week 1:
├── R1: Delete orphaned types (30 min)
├── R2: Implement NavAction handler (1 hr)
├── R3: Delete ui_stub.rs (5 min)
├── R5: Remove debug statements (10 min)
└── Total: ~2 hours

Week 2:
├── R4: Add input system ordering (2 hrs)
├── R6: Add integration tests (4 hrs)
└── Total: ~6 hours

Week 3:
├── R8: Enable StatusBar styling (2 hrs)
├── R9: Implement commands (1 hr)
├── R7: FusabiTuiPlugin decision (10 min - 1 day)
└── Total: 3-11 hours

Backlog:
└── R10: Unified input architecture (1-2 weeks)
```

---

## Success Criteria

After implementing P0 and P1:

- [ ] No type conflicts between navigation modules
- [ ] NavActionEvent triggers actual operations
- [ ] No stub files in codebase
- [ ] Input handlers run in defined order
- [ ] No debug output in production
- [ ] Navigation integration tests pass

After implementing P2:

- [ ] Status bar shows colored text
- [ ] reload_config command works
- [ ] help command opens documentation
- [ ] FusabiTuiPlugin implemented or removed

After implementing P3:

- [ ] Single InputRouter controls all input
- [ ] Input consumption prevents double-handling
- [ ] Mode transitions are atomic and auditable
