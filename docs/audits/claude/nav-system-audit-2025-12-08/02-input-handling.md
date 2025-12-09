# Input Handling System Analysis

## Overview

Scarab has multiple input handling systems operating at different levels:

1. **IPC Layer** - Sends keystrokes to daemon (`ipc.rs`)
2. **Navigation Layer** - Mode-aware routing (`nav_input.rs`) - ORPHANED
3. **Ratatui Layer** - Surface-focused input (`ratatui_bridge/input.rs`)
4. **Keybindings Layer** - Configurable shortcuts (`keybindings.rs`)
5. **Leader Key Layer** - Spacemacs-style menus (`leader_key.rs`)

---

## System Registration Matrix

| System | Plugin | Schedule | Set | Status |
|--------|--------|----------|-----|--------|
| `handle_keyboard_input` | IpcPlugin | Update | - | ✓ Active |
| `handle_character_input` | IpcPlugin | Update | - | ✓ Active |
| `handle_window_resize` | IpcPlugin | Update | - | ✓ Active |
| `receive_ipc_messages` | IpcPlugin | Update | - | ✓ Active |
| `handle_startup_command` | IpcPlugin | Update | - | ✓ Active |
| `route_nav_input` | **NONE** | PreUpdate | - | ✗ ORPHANED |
| `handle_keyboard_input` | RatatuiBridge | Update | - | ✓ Active |
| `handle_mouse_input` | RatatuiBridge | Update | - | ✓ Active |
| `cleanup_focus` | RatatuiBridge | Update | - | ✓ Active |
| `handle_keybindings_system` | KeybindingsPlugin | Update | - | ✓ Active |
| `handle_leader_key_system` | LeaderKeyPlugin | Update | Chained | ✓ Active |

---

## Input Flow Diagram

```
┌──────────────────────────────────────────────────────────────────────┐
│                        BEVY INPUT EVENTS                              │
│  ButtonInput<KeyCode>    KeyboardInput    MouseButtonInput           │
└──────────────────────────────────────────────────────────────────────┘
                                │
        ┌───────────────────────┼───────────────────────┐
        │                       │                       │
        ▼                       ▼                       ▼
┌───────────────┐   ┌───────────────────┐   ┌───────────────────────┐
│  IPC Layer    │   │ Navigation Layer  │   │   Ratatui Layer       │
│               │   │ (ORPHANED)        │   │                       │
│ handle_       │   │ route_nav_input   │   │ handle_keyboard_input │
│ keyboard_input│   │ NOT REGISTERED    │   │ handle_mouse_input    │
│               │   │                   │   │                       │
│ handle_       │   │ Would check:      │   │ Routes to focused     │
│ character_    │   │ - NavMode         │   │ RatatuiSurface        │
│ input         │   │ - KeyBindings     │   │                       │
└───────┬───────┘   └───────────────────┘   └───────────┬───────────┘
        │                                               │
        ▼                                               ▼
┌───────────────┐                           ┌───────────────────────┐
│ ControlMessage│                           │ SurfaceInputEvent     │
│ ::Input       │                           │                       │
└───────┬───────┘                           └───────────┬───────────┘
        │                                               │
        ▼                                               ▼
┌───────────────┐                           ┌───────────────────────┐
│   DAEMON      │                           │ Command Palette       │
│   (PTY)       │                           │ Search Overlay        │
│               │                           │ Plugin Menus          │
└───────────────┘                           └───────────────────────┘
```

---

## IPC Input Handlers (`ipc.rs`)

### handle_keyboard_input (Line 256-263)

**Purpose:** Send special keys to daemon

**Key Mappings:**
```rust
KeyCode::Enter     → b'\r'
KeyCode::Backspace → 0x7F
KeyCode::Tab       → b'\t'
KeyCode::Escape    → 0x1B
KeyCode::Space     → b' '
KeyCode::ArrowUp   → [0x1B, b'[', b'A']
KeyCode::ArrowDown → [0x1B, b'[', b'B']
// ... function keys, etc.
```

**Behavior:**
- Reads `ButtonInput<KeyCode>` via `get_just_pressed()`
- Converts to terminal bytes
- Sends as `ControlMessage::Input { data }` to daemon

### handle_character_input (Line 302-325)

**Purpose:** Send printable characters to daemon

**Behavior:**
- Reads `KeyboardInput` events
- Filters out key releases
- Skips keys handled by `handle_keyboard_input`
- Extracts character from `logical_key`
- Sends UTF-8 bytes to daemon

**Debug Output:** Contains `eprintln!("DEBUG char_input: ...")` (should be removed)

---

## Navigation Input Router (`nav_input.rs`)

### route_nav_input (Line 577-651) - NOT REGISTERED

**Intended Purpose:** Mode-aware keyboard routing

**Parameters:**
```rust
pub fn route_nav_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    router: Res<NavInputRouter>,
    mut mode_stack: ResMut<ModeStack>,
    mut action_writer: EventWriter<NavAction>,
)
```

**Behavior (if it were registered):**
1. Get current mode from `ModeStack`
2. Filter keybindings by active mode
3. Match keyboard state against bindings
4. Execute mode transitions:
   - `EnterHintMode` → Push Hints mode
   - `EnterCopyMode` → Push Copy mode
   - `ExitCurrentMode` → Pop current mode
   - `CancelAllModes` → Clear stack
5. In Hints mode: collect hint characters

### NavInputRouter (Line 315-335)

**Default Keybindings:**
| Key | Mode | Action |
|-----|------|--------|
| Ctrl+F | Normal | EnterHintMode |
| Escape | Any | CancelAllModes |
| Ctrl+V | Normal | EnterCopyMode |
| Ctrl+/ | Normal | EnterSearchMode |
| Ctrl+P | Normal | EnterCommandPalette |
| Escape | Copy | ExitCurrentMode |
| n | Search | NextSearchMatch |
| N | Search | PrevSearchMatch |

### ModeStack (Line 500-562)

```rust
pub struct ModeStack {
    stack: Vec<NavMode>,
}

impl ModeStack {
    pub fn push(&mut self, mode: NavMode);
    pub fn pop(&mut self) -> Option<NavMode>;
    pub fn clear(&mut self);
    pub fn current(&self) -> NavMode;  // Returns Normal if empty
    pub fn is_normal(&self) -> bool;
    pub fn depth(&self) -> usize;
}
```

---

## Ratatui Bridge Input (`ratatui_bridge/input.rs`)

### handle_keyboard_input (Line 198-248)

**Purpose:** Route keyboard to focused Ratatui surface

**Behavior:**
1. Find focused surface from `SurfaceFocus`
2. Skip if surface not visible
3. Convert Bevy `KeyCode` to Ratatui `KeyCode`
4. Apply Shift modifier to letters
5. Send `SurfaceInputEvent` with `RatEvent::Key`

### handle_mouse_input (Line 259-331)

**Purpose:** Route mouse clicks to surfaces

**Behavior:**
1. Convert screen coords to grid coords via `TerminalMetrics`
2. Find topmost surface under cursor
3. Update focus on mouse down
4. Convert to surface-local coordinates
5. Send `SurfaceInputEvent` with `RatEvent::Mouse`

### Key Conversion (Line 90-168)

Bevy → Ratatui crossterm format:
```rust
KeyCode::Backspace → RatKeyCode::Backspace
KeyCode::Enter     → RatKeyCode::Enter
KeyCode::ArrowLeft → RatKeyCode::Left
KeyCode::KeyA      → RatKeyCode::Char('a')
KeyCode::Digit1    → RatKeyCode::Char('1')
// etc.
```

---

## Keybindings System (`keybindings.rs`)

### handle_keybindings_system (Line 262-280)

**Purpose:** Global keybinding processing

**Behavior:**
1. Iterate registered keybindings
2. Check `just_pressed()` for primary key
3. Verify modifier requirements
4. Emit `KeyBindingTriggeredEvent`

**Default Bindings:**
- Ctrl+C/V/X - Copy/Paste/Cut
- Ctrl+Z/Y - Undo/Redo
- Ctrl+F/H - Search/Replace
- Ctrl+P - Command Palette
- Ctrl+K - Link Hints

---

## Leader Key System (`leader_key.rs`)

### System Chain

```rust
.add_systems(Update, (
    handle_leader_key_system,
    handle_menu_navigation_system,
    render_menu_system,
    timeout_check_system,
).chain())
```

Runs in sequence: activation → navigation → render → timeout

---

## Issues Identified

### 1. CRITICAL: route_nav_input Not Registered

**Evidence:**
```rust
// main.rs line 218-219 - resources created
.insert_resource(NavInputRouter::new(NavStyle::VimiumStyle))
.insert_resource(ModeStack::new())

// MISSING:
// .add_systems(PreUpdate, route_nav_input)
```

**Impact:**
- Mode switching via keyboard doesn't work
- Hint mode keyboard navigation broken
- Copy/Search modes inaccessible via keyboard

### 2. HIGH: Input Ordering Not Defined

All input handlers run in Update schedule with no ordering:
- IPC handlers (5 systems)
- Ratatui handlers (3 systems)
- Keybindings handler (1 system)
- Leader key handlers (4 systems, chained internally)

Potential for race conditions where multiple handlers process same key.

### 3. MEDIUM: No Input Consumption

No mechanism to mark input as "consumed":
- Ctrl+F could trigger:
  - IPC (send to daemon)
  - Search overlay (open search)
  - Link hints (enter hint mode)
  - Keybindings (general binding)

### 4. MEDIUM: Duplicate Key Mappings

Both files define Ctrl+F:
- `keybindings.rs` → Search
- `nav_input.rs` → EnterHintMode (if registered)

Ctrl+P conflicts:
- `keybindings.rs` → Command Palette
- `nav_input.rs` → EnterCommandPalette (if registered)

### 5. LOW: Debug Output Present

`handle_character_input` contains:
```rust
eprintln!("DEBUG char_input: sending {:?} ({:?})", s, bytes);
```

Should be removed or gated behind feature flag.

---

## Recommendations

### Immediate Fixes

1. **Remove orphaned nav_input.rs code** - It conflicts with existing keybindings.rs

2. **Add system ordering:**
```rust
.configure_sets(Update, (
    InputSet::Navigation,
    InputSet::Surface,
    InputSet::IPC,
).chain())
```

3. **Remove debug output** from production code

### Architectural Improvements

1. **Unified input routing:**
```rust
fn route_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mode: Res<CurrentInputMode>,
) -> InputDestination {
    match mode.current() {
        InputMode::Terminal => InputDestination::IPC,
        InputMode::Overlay(surface) => InputDestination::Surface(surface),
        InputMode::Navigation => InputDestination::NavSystem,
    }
}
```

2. **Input consumption mechanism:**
```rust
#[derive(Event)]
struct ConsumedInput {
    key: KeyCode,
    consumer: &'static str,
}
```

3. **Modal input state:**
```rust
#[derive(Resource)]
enum InputMode {
    Terminal,           // Pass to PTY
    Hints,              // Hint mode
    CommandPalette,     // Fuzzy search
    Search,             // Find in scrollback
    SurfaceFocused(Entity),  // Ratatui overlay
}
```
