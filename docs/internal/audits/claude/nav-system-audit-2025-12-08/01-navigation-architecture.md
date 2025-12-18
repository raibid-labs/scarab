# Navigation Architecture Deep Dive

## Overview

Scarab's navigation system is split across two locations with fundamentally different designs:

1. **ECS-Native Module** (`navigation/`) - Well-integrated Bevy-first design
2. **Input Routing Module** (`input/nav_input.rs`) - Standalone, orphaned implementation

This document analyzes both systems and their (lack of) integration.

---

## Module Structure

### 1. Navigation Module (`crates/scarab-client/src/navigation/`)

```
navigation/
├── mod.rs          # Core types, state, pane lifecycle (600+ lines)
├── focusable.rs    # Focusable detection for URLs/paths/emails (580+ lines)
├── metrics.rs      # Performance metrics tracking (450+ lines)
└── tests.rs        # Integration tests (150+ lines)
```

**Total:** ~1,800 lines of well-structured ECS code

### 2. Input Module (`crates/scarab-client/src/input/`)

```
input/
├── mod.rs          # Re-exports (12 lines)
├── nav_input.rs    # Orphaned navigation input routing (921 lines)
└── key_tables.rs   # Key table wrapper for plugin API (246 lines)
```

**Total:** ~1,180 lines, with 921 lines effectively dead

---

## Type Conflicts Analysis

### NavMode Enum

**Location 1: `input/nav_input.rs:18-31`**
```rust
pub enum NavMode {
    Normal,       // Standard input
    Hints,        // Link hint selection
    Copy,         // Vim-like copy mode
    Search,       // Text search mode
    CommandPalette,
    PromptNav,    // Jump between prompts (UNIQUE)
}
```

**Location 2: `navigation/mod.rs:59-73`**
```rust
pub enum NavMode {
    Normal,
    Hints,
    Insert,       // DIFFERENT - text input mode
    CommandPalette,
}
```

**Conflicts:**
| Mode | input/nav_input.rs | navigation/mod.rs |
|------|-------------------|-------------------|
| Normal | ✓ | ✓ |
| Hints | ✓ | ✓ |
| Copy | ✓ | ✗ |
| Search | ✓ | ✗ |
| Insert | ✗ | ✓ |
| PromptNav | ✓ | ✗ |
| CommandPalette | ✓ | ✓ |

### NavAction Enum

**Location 1: `input/nav_input.rs:45-94` (Event-capable)**
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Event)]
pub enum NavAction {
    EnterHintMode,
    EnterCopyMode,
    EnterSearchMode,
    EnterCommandPalette,
    ExitCurrentMode,
    CancelAllModes,
    ActivateHint,
    HintChar(char),
    JumpToPrevPrompt,
    JumpToNextPrompt,
    // ... 10+ more variants
}
```

**Location 2: `navigation/mod.rs:145-169` (NOT Event-capable)**
```rust
#[derive(Debug, Clone, PartialEq)]
pub enum NavAction {
    Open(String),         // URL to open
    Click(u16, u16),      // Grid coordinates
    JumpPrompt(u32),      // Prompt index
    NextPane,
    PrevPane,
    NextTab,
    PrevTab,
    Cancel,
}
```

**Semantic Difference:**
- `input/nav_input.rs` - **Input commands** (what the user pressed)
- `navigation/mod.rs` - **UI operations** (what should happen)

These are at different abstraction levels and should be separate types, but naming collision causes confusion.

---

## System Registration Status

### NavigationPlugin (`navigation/mod.rs:516-546`)

**Registered Systems:**
```rust
impl Plugin for NavigationPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<NavStateRegistry>()
            .add_event::<EnterHintModeEvent>()
            .add_event::<ExitHintModeEvent>()
            .add_event::<NavActionEvent>()
            .add_event::<FocusChangedEvent>()
            .configure_sets(Update, (
                NavSystemSet::Input,
                NavSystemSet::Update,
                NavSystemSet::Render,
            ).chain())
            .add_systems(Update, (
                on_pane_created,      // ✓
                on_pane_focused,      // ✓
                on_pane_closed,       // ✓
            ).in_set(NavSystemSet::Update));
    }
}
```

**Status:** ✓ Properly registered

### FocusablePlugin (`navigation/focusable.rs:541-573`)

**Registered Systems:**
```rust
impl Plugin for FocusablePlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<FocusableScanConfig>()
            .init_resource::<FocusableGeneration>()
            .add_systems(Startup, initialize_focusable_detector)
            .add_systems(Update, (
                scan_terminal_focusables.in_set(NavSystemSet::Input),
                bounds_to_world_coords.in_set(NavSystemSet::Update),
                filter_focusables_by_zone.in_set(NavSystemSet::Update),
                cleanup_focusables.in_set(NavSystemSet::Update),
                detect_stale_focusables.in_set(NavSystemSet::Update),
            ));
    }
}
```

**Status:** ✓ Properly registered

### route_nav_input (`input/nav_input.rs:577-651`)

**NOT REGISTERED:**
```rust
// This function exists but is NEVER called
pub fn route_nav_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    router: Res<NavInputRouter>,
    mut mode_stack: ResMut<ModeStack>,
    mut action_writer: EventWriter<NavAction>,
) {
    // 74 lines of navigation routing logic
    // COMPLETELY ORPHANED
}
```

**In main.rs:**
```rust
// Resources are created...
.insert_resource(NavInputRouter::new(NavStyle::VimiumStyle))  // Line 218
.insert_resource(ModeStack::new())                             // Line 219
// ...but system is NEVER added
// MISSING: .add_systems(PreUpdate, route_nav_input)
```

---

## Data Flow Analysis

### Expected Flow (Not Working)

```
1. User presses Ctrl+F
   ↓
2. route_nav_input() matches binding      [NOT REGISTERED]
   ↓
3. NavAction::EnterHintMode emitted       [NEVER HAPPENS]
   ↓
4. Mode stack updated to Hints            [NEVER HAPPENS]
   ↓
5. EnterHintModeEvent emitted             [TRIGGERED ELSEWHERE?]
   ↓
6. scan_terminal_focusables() runs        [WORKS]
   ↓
7. HintOverlay entities spawned           [WORKS]
   ↓
8. User presses hint key (e.g., 'a')
   ↓
9. route_nav_input() matches HintChar     [NOT REGISTERED]
   ↓
10. NavActionEvent emitted                 [NEVER HAPPENS]
   ↓
11. Handler processes action               [NO HANDLER EXISTS]
```

### Actual Flow (Currently Working Partially)

The navigation system partially works because:

1. **EnterHintModeEvent** can be triggered by other code paths (e.g., LinkHintsPlugin)
2. **FocusablePlugin** properly scans and spawns focusables
3. **HintOverlayPlugin** renders the hints correctly

But:
- Mode switching via keyboard doesn't work
- Hint activation via keyboard doesn't work
- NavAction events are never processed

---

## State Management

### Per-Pane State (navigation/mod.rs)

```rust
#[derive(Resource, Default)]
pub struct NavStateRegistry {
    states: HashMap<u64, NavState>,  // Per-pane navigation state
    active_pane: Option<u64>,
}

#[derive(Default, Clone)]
pub struct NavState {
    pub mode_stack: Vec<NavMode>,    // Nested mode support
    pub focus_history: Vec<Entity>,  // Navigation history
    pub active_group: Option<String>,
}
```

**Features:**
- Per-pane isolation (each terminal pane has own nav state)
- Mode stacking (can push/pop modes)
- Focus history (50 entries max, circular buffer)
- Group filtering (filter focusables by category)

### Global State (input/nav_input.rs)

```rust
#[derive(Resource)]
pub struct ModeStack {
    stack: Vec<NavMode>,  // GLOBAL - not per-pane!
}

#[derive(Resource)]
pub struct NavInputRouter {
    current_style: NavStyle,
    keybindings: HashMap<NavStyle, Vec<KeyBinding>>,
}
```

**Conflicts:**
- `ModeStack` is global, `NavStateRegistry` is per-pane
- Cannot be reconciled without significant refactoring

---

## Component/Event Inventory

### Components (ECS)

| Component | Location | Purpose | Status |
|-----------|----------|---------|--------|
| `NavFocus` | navigation/mod.rs | Current focus marker | Used |
| `NavHint` | navigation/mod.rs | Hint label data | Used |
| `NavGroup` | navigation/mod.rs | Group membership | Used |
| `FocusableRegion` | focusable.rs | Detected targets | Used |
| `HintOverlay` | hint_overlay.rs | Visual rendering | Used |
| `HintFade` | hint_overlay.rs | Fade animation | Used |

### Events

| Event | Location | Emitters | Handlers |
|-------|----------|----------|----------|
| `EnterHintModeEvent` | navigation/mod.rs | LinkHintsPlugin | scan_terminal_focusables |
| `ExitHintModeEvent` | navigation/mod.rs | Multiple | cleanup_focusables |
| `NavActionEvent` | navigation/mod.rs | ??? | **NONE** |
| `FocusChangedEvent` | navigation/mod.rs | ??? | on_pane_focused |

### System Sets

```rust
pub enum NavSystemSet {
    Input,   // Keyboard/mouse processing
    Update,  // State updates
    Render,  // Visual output
}
```

These are configured to run in order: Input → Update → Render

---

## Missing Implementations

### 1. NavAction Handler

**Required System:**
```rust
fn handle_nav_actions(
    mut events: EventReader<NavActionEvent>,
    // ... resources for executing actions
) {
    for event in events.read() {
        match &event.action {
            NavAction::Open(url) => {
                // Open URL in browser
                #[cfg(target_os = "linux")]
                std::process::Command::new("xdg-open").arg(url).spawn();
            }
            NavAction::Click(col, row) => {
                // Send click to terminal
            }
            NavAction::JumpPrompt(index) => {
                // Scroll to prompt
            }
            NavAction::NextPane => {
                // Focus next pane
            }
            // ... etc
        }
    }
}
```

### 2. Mode Transition System

**Required System:**
```rust
fn handle_mode_transitions(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut registry: ResMut<NavStateRegistry>,
    mut enter_events: EventWriter<EnterHintModeEvent>,
    mut exit_events: EventWriter<ExitHintModeEvent>,
) {
    // Ctrl+F → Enter Hints mode
    if keyboard.just_pressed(KeyCode::KeyF)
        && keyboard.pressed(KeyCode::ControlLeft)
    {
        if let Some(state) = registry.get_active_mut() {
            state.mode_stack.push(NavMode::Hints);
            enter_events.send(EnterHintModeEvent { ... });
        }
    }

    // Escape → Exit current mode
    if keyboard.just_pressed(KeyCode::Escape) {
        if let Some(state) = registry.get_active_mut() {
            if state.mode_stack.pop().is_some() {
                exit_events.send(ExitHintModeEvent);
            }
        }
    }
}
```

---

## Recommendations

### Option A: Delete Orphaned Code (Recommended)

1. Remove `input/nav_input.rs` entirely
2. Remove `NavInputRouter` and `ModeStack` from main.rs
3. Add missing handlers to `navigation/mod.rs`
4. Use existing `NavStateRegistry` for mode management

**Pros:** Clean, maintains per-pane isolation
**Cons:** Lose some keybinding flexibility

### Option B: Integrate Existing Code

1. Register `route_nav_input` as system
2. Resolve type conflicts (rename one set)
3. Bridge global ModeStack to per-pane NavStateRegistry
4. Add NavAction handler

**Pros:** Preserves all written code
**Cons:** Complex, architectural mismatch

### Option C: Unified Rewrite

1. Design single navigation architecture
2. Support both per-pane and global modes
3. Single NavMode/NavAction type hierarchy
4. Proper event flow documentation

**Pros:** Clean architecture
**Cons:** Significant effort

---

## Test Coverage

### Existing Tests

- `navigation/tests.rs` - Mode stack, focus history
- Unit tests in `nav_input.rs` - Keybinding parsing

### Missing Tests

- Integration: Full navigation flow
- Integration: Mode transitions via keyboard
- Integration: NavAction execution
- E2E: User presses Ctrl+F, sees hints, presses hint key, action executes
