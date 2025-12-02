# WS-4: Key Tables & Modal Editing

**Workstream ID:** WS-4
**Priority:** P1 (High Value)
**Estimated Complexity:** Medium
**Dependencies:** WS-2 (Event System)

## Overview

WezTerm's key tables enable modal keyboard configurations—different key bindings active in different contexts. Users can create resize modes, copy modes, and custom modal layers. This is essential for power users who want vim-like workflows.

## Current State Analysis

### Scarab's Current Keybindings

```toml
# ~/.config/scarab/config.toml
[keybindings]
"Ctrl+Shift+T" = "NewTab"
"Ctrl+Shift+W" = "CloseTab"
"Ctrl+Shift+N" = "NewWindow"
```

**Limitations:**
- Flat structure (no modes/tables)
- No leader key support
- No timeout-based modes
- No copy mode

### WezTerm's Key Tables

```lua
config.leader = { key = 'a', mods = 'CTRL', timeout_milliseconds = 1000 }

config.keys = {
  { key = 'r', mods = 'LEADER', action = act.ActivateKeyTable {
    name = 'resize_pane',
    one_shot = false,
  }},
  { key = 'a', mods = 'LEADER', action = act.ActivateKeyTable {
    name = 'activate_pane',
    one_shot = true,
  }},
}

config.key_tables = {
  resize_pane = {
    { key = 'h', action = act.AdjustPaneSize { 'Left', 1 } },
    { key = 'l', action = act.AdjustPaneSize { 'Right', 1 } },
    { key = 'Escape', action = 'PopKeyTable' },
  },
  activate_pane = {
    { key = 'h', action = act.ActivatePaneDirection 'Left' },
    { key = 'l', action = act.ActivatePaneDirection 'Right' },
  },
  copy_mode = { ... },
}
```

## Target Architecture

### Key Table Stack

Each window maintains a stack of active key tables:

```rust
#[derive(Resource)]
pub struct KeyTableStack {
    stack: Vec<KeyTableActivation>,
    default_table: KeyTable,
}

pub struct KeyTableActivation {
    pub name: String,
    pub table: KeyTable,
    pub one_shot: bool,
    pub timeout: Option<Instant>,
    pub replace_current: bool,
}

pub struct KeyTable {
    pub name: String,
    pub bindings: HashMap<KeyCombo, KeyAction>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct KeyCombo {
    pub key: KeyCode,
    pub mods: Modifiers,
}

#[derive(Clone, Debug)]
pub struct Modifiers {
    pub ctrl: bool,
    pub alt: bool,
    pub shift: bool,
    pub super_key: bool,
    pub leader: bool,  // Virtual modifier
}
```

### Key Resolution Algorithm

```rust
impl KeyTableStack {
    pub fn resolve(&self, combo: &KeyCombo) -> Option<&KeyAction> {
        // Search from top of stack downward
        for activation in self.stack.iter().rev() {
            if let Some(action) = activation.table.bindings.get(combo) {
                return Some(action);
            }
        }

        // Fall through to default table
        self.default_table.bindings.get(combo)
    }

    pub fn handle_key(&mut self, combo: KeyCombo, time: Instant) -> Option<KeyAction> {
        // Check for expired timeouts
        self.expire_timeouts(time);

        if let Some(action) = self.resolve(&combo) {
            let action = action.clone();

            // Handle one_shot: pop after any keypress
            if self.stack.last().map(|a| a.one_shot).unwrap_or(false) {
                self.stack.pop();
            }

            return Some(action);
        }

        // Unmatched key in one_shot mode also pops
        if self.stack.last().map(|a| a.one_shot).unwrap_or(false) {
            self.stack.pop();
        }

        None
    }

    fn expire_timeouts(&mut self, now: Instant) {
        self.stack.retain(|activation| {
            activation.timeout.map(|t| t > now).unwrap_or(true)
        });
    }
}
```

### Leader Key

The leader key is a special modifier that becomes active temporarily:

```rust
#[derive(Resource)]
pub struct LeaderKeyState {
    pub is_active: bool,
    pub activated_at: Option<Instant>,
    pub timeout_ms: u64,
    pub key: KeyCombo,
}

impl LeaderKeyState {
    pub fn activate(&mut self) {
        self.is_active = true;
        self.activated_at = Some(Instant::now());
    }

    pub fn check_timeout(&mut self) -> bool {
        if let Some(activated) = self.activated_at {
            if activated.elapsed().as_millis() as u64 > self.timeout_ms {
                self.deactivate();
                return true;  // Did timeout
            }
        }
        false
    }

    pub fn deactivate(&mut self) {
        self.is_active = false;
        self.activated_at = None;
    }
}
```

### Key Actions

```rust
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum KeyAction {
    // Table management
    ActivateKeyTable {
        name: String,
        one_shot: bool,
        timeout_milliseconds: Option<u64>,
        replace_current: bool,
    },
    PopKeyTable,
    ClearKeyTableStack,

    // Pane actions
    ActivatePaneDirection(Direction),
    AdjustPaneSize { direction: Direction, amount: i32 },
    SplitPane { direction: SplitDirection },
    ClosePane,
    ZoomPane,

    // Tab actions
    ActivateTab(i32),  // Can be negative for relative
    ActivateTabRelative(i32),
    SpawnTab,
    CloseTab,
    MoveTab(i32),

    // Window actions
    SpawnWindow,
    CloseWindow,
    ToggleFullscreen,
    Maximize,

    // Terminal actions
    SendString(String),
    SendKey { key: KeyCode, mods: Modifiers },
    ScrollByPage(i32),
    ScrollByLine(i32),
    ScrollToTop,
    ScrollToBottom,
    ClearScrollback,

    // Clipboard
    Copy,
    Paste,
    CopyTo(ClipboardKind),
    PasteFrom(ClipboardKind),

    // Mode
    ActivateCopyMode,
    ActivateSearchMode,

    // Custom
    EmitEvent { event: String, args: Vec<String> },
    RunCommand(String),
    Noop,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum Direction {
    Left,
    Right,
    Up,
    Down,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum SplitDirection {
    Horizontal,
    Vertical,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum ClipboardKind {
    Primary,
    Clipboard,
}
```

## Configuration Syntax

### Fusabi Config

```fsx
// ~/.config/scarab/config.fsx
module Keys

open Scarab.Keys

// Leader key configuration
config.Leader <- {
    Key = KeyCode.A
    Mods = Modifiers.Ctrl
    TimeoutMs = 1000u
}

// Default key table
config.Keys <- [
    // Direct actions
    { Key = KeyCode.T; Mods = Modifiers.CtrlShift; Action = SpawnTab }
    { Key = KeyCode.W; Mods = Modifiers.CtrlShift; Action = CloseTab }

    // Leader key combos
    { Key = KeyCode.R; Mods = Modifiers.Leader; Action = ActivateKeyTable {
        Name = "resize_pane"
        OneShot = false
    }}
    { Key = KeyCode.A; Mods = Modifiers.Leader; Action = ActivateKeyTable {
        Name = "activate_pane"
        OneShot = true
    }}
    { Key = KeyCode.C; Mods = Modifiers.Leader; Action = ActivateCopyMode }
]

// Named key tables
config.KeyTables <- Map.ofList [
    "resize_pane", [
        { Key = KeyCode.H; Action = AdjustPaneSize(Left, 1) }
        { Key = KeyCode.J; Action = AdjustPaneSize(Down, 1) }
        { Key = KeyCode.K; Action = AdjustPaneSize(Up, 1) }
        { Key = KeyCode.L; Action = AdjustPaneSize(Right, 1) }
        { Key = KeyCode.Left; Action = AdjustPaneSize(Left, 1) }
        { Key = KeyCode.Down; Action = AdjustPaneSize(Down, 1) }
        { Key = KeyCode.Up; Action = AdjustPaneSize(Up, 1) }
        { Key = KeyCode.Right; Action = AdjustPaneSize(Right, 1) }
        { Key = KeyCode.Escape; Action = PopKeyTable }
    ]

    "activate_pane", [
        { Key = KeyCode.H; Action = ActivatePaneDirection(Left) }
        { Key = KeyCode.J; Action = ActivatePaneDirection(Down) }
        { Key = KeyCode.K; Action = ActivatePaneDirection(Up) }
        { Key = KeyCode.L; Action = ActivatePaneDirection(Right) }
    ]

    "copy_mode", [
        // Vim-like navigation
        { Key = KeyCode.H; Action = CopyMode(MoveLeft) }
        { Key = KeyCode.J; Action = CopyMode(MoveDown) }
        { Key = KeyCode.K; Action = CopyMode(MoveUp) }
        { Key = KeyCode.L; Action = CopyMode(MoveRight) }
        { Key = KeyCode.W; Action = CopyMode(MoveWordForward) }
        { Key = KeyCode.B; Action = CopyMode(MoveWordBackward) }
        { Key = KeyCode.V; Action = CopyMode(ToggleSelection) }
        { Key = KeyCode.Y; Action = CopyMode(CopyAndExit) }
        { Key = KeyCode.Escape; Action = CopyMode(Exit) }
        { Key = KeyCode.Q; Action = CopyMode(Exit) }
    ]
]
```

### TOML Alternative

```toml
[leader]
key = "a"
mods = "CTRL"
timeout_milliseconds = 1000

[[keys]]
key = "t"
mods = "CTRL|SHIFT"
action = "SpawnTab"

[[keys]]
key = "r"
mods = "LEADER"
action = { ActivateKeyTable = { name = "resize_pane", one_shot = false } }

[key_tables.resize_pane]
bindings = [
  { key = "h", action = { AdjustPaneSize = { direction = "Left", amount = 1 } } },
  { key = "Escape", action = "PopKeyTable" },
]
```

## Bevy Integration

### Input System

```rust
pub fn key_input_system(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut key_stack: ResMut<KeyTableStack>,
    mut leader_state: ResMut<LeaderKeyState>,
    mut action_events: EventWriter<KeyActionEvent>,
    time: Res<Time>,
) {
    // Check leader timeout
    if leader_state.check_timeout() {
        // Leader expired, show indicator
    }

    // Build current modifiers
    let mods = Modifiers {
        ctrl: keyboard_input.pressed(KeyCode::ControlLeft)
            || keyboard_input.pressed(KeyCode::ControlRight),
        alt: keyboard_input.pressed(KeyCode::AltLeft)
            || keyboard_input.pressed(KeyCode::AltRight),
        shift: keyboard_input.pressed(KeyCode::ShiftLeft)
            || keyboard_input.pressed(KeyCode::ShiftRight),
        super_key: keyboard_input.pressed(KeyCode::SuperLeft)
            || keyboard_input.pressed(KeyCode::SuperRight),
        leader: leader_state.is_active,
    };

    // Check for leader key press
    for key in keyboard_input.get_just_pressed() {
        let combo = KeyCombo { key: *key, mods: mods.clone() };

        // Is this the leader key?
        if combo == leader_state.key && !leader_state.is_active {
            leader_state.activate();
            continue;
        }

        // Look up action
        if let Some(action) = key_stack.handle_key(combo, time.elapsed()) {
            action_events.send(KeyActionEvent(action));

            // Deactivate leader after any key (matched or not)
            if leader_state.is_active {
                leader_state.deactivate();
            }
        }
    }
}

#[derive(Event)]
pub struct KeyActionEvent(pub KeyAction);
```

### Action Execution

```rust
pub fn execute_key_action_system(
    mut action_events: EventReader<KeyActionEvent>,
    mut key_stack: ResMut<KeyTableStack>,
    key_tables: Res<KeyTableRegistry>,
    mut ipc: ResMut<IpcChannel>,
    // ... other resources
) {
    for KeyActionEvent(action) in action_events.read() {
        match action {
            KeyAction::ActivateKeyTable { name, one_shot, timeout_milliseconds, replace_current } => {
                if let Some(table) = key_tables.get(name) {
                    let activation = KeyTableActivation {
                        name: name.clone(),
                        table: table.clone(),
                        one_shot: *one_shot,
                        timeout: timeout_milliseconds.map(|ms| Instant::now() + Duration::from_millis(ms)),
                        replace_current: *replace_current,
                    };

                    if *replace_current && !key_stack.stack.is_empty() {
                        key_stack.stack.pop();
                    }

                    key_stack.stack.push(activation);
                }
            }

            KeyAction::PopKeyTable => {
                key_stack.stack.pop();
            }

            KeyAction::ClearKeyTableStack => {
                key_stack.stack.clear();
            }

            KeyAction::SendString(s) => {
                ipc.send(ControlMessage::Input { data: s.as_bytes().to_vec() });
            }

            KeyAction::SpawnTab => {
                ipc.send(ControlMessage::TabCreate { title: None });
            }

            KeyAction::ActivateCopyMode => {
                if let Some(table) = key_tables.get("copy_mode") {
                    key_stack.stack.push(KeyTableActivation {
                        name: "copy_mode".into(),
                        table: table.clone(),
                        one_shot: false,
                        timeout: None,
                        replace_current: false,
                    });
                    // Enter copy mode state
                    copy_mode_state.enter();
                }
            }

            // ... handle other actions
        }
    }
}
```

## Mode Indicator

Show users which mode is active:

```rust
#[derive(Resource, Default)]
pub struct ModeIndicator {
    pub current_mode: String,
    pub dirty: bool,
}

pub fn update_mode_indicator_system(
    key_stack: Res<KeyTableStack>,
    leader_state: Res<LeaderKeyState>,
    mut indicator: ResMut<ModeIndicator>,
) {
    let new_mode = if leader_state.is_active {
        "LEADER".to_string()
    } else if let Some(top) = key_stack.stack.last() {
        top.name.to_uppercase()
    } else {
        "NORMAL".to_string()
    };

    if new_mode != indicator.current_mode {
        indicator.current_mode = new_mode;
        indicator.dirty = true;
    }
}
```

Display in status bar or as overlay:

```fsx
On(EventType.UpdateLeftStatus, fun window pane ->
    let mode = Scarab.GetCurrentMode()
    if mode <> "NORMAL" then
        window.SetLeftStatus([
            Background(Color.Hex "#ff9e64")
            Foreground(Color.Hex "#1a1b26")
            Bold
            Text($" {mode} ")
            ResetAttributes
        ])
    else
        window.SetLeftStatus([])
)
```

## Default Key Tables

Provide sensible defaults:

```rust
impl Default for KeyTableRegistry {
    fn default() -> Self {
        let mut registry = Self::new();

        // Default copy_mode table (vim-like)
        registry.register("copy_mode", KeyTable {
            name: "copy_mode".into(),
            bindings: hashmap! {
                KeyCombo::new(KeyCode::KeyH, Modifiers::NONE) => KeyAction::CopyMode(CopyModeAction::MoveLeft),
                KeyCombo::new(KeyCode::KeyJ, Modifiers::NONE) => KeyAction::CopyMode(CopyModeAction::MoveDown),
                KeyCombo::new(KeyCode::KeyK, Modifiers::NONE) => KeyAction::CopyMode(CopyModeAction::MoveUp),
                KeyCombo::new(KeyCode::KeyL, Modifiers::NONE) => KeyAction::CopyMode(CopyModeAction::MoveRight),
                KeyCombo::new(KeyCode::KeyW, Modifiers::NONE) => KeyAction::CopyMode(CopyModeAction::MoveWordForward),
                KeyCombo::new(KeyCode::KeyB, Modifiers::NONE) => KeyAction::CopyMode(CopyModeAction::MoveWordBackward),
                KeyCombo::new(KeyCode::Digit0, Modifiers::NONE) => KeyAction::CopyMode(CopyModeAction::MoveToLineStart),
                KeyCombo::new(KeyCode::Digit4, Modifiers::SHIFT) => KeyAction::CopyMode(CopyModeAction::MoveToLineEnd),  // $
                KeyCombo::new(KeyCode::KeyG, Modifiers::NONE) => KeyAction::CopyMode(CopyModeAction::MoveToBottom),
                KeyCombo::new(KeyCode::KeyG, Modifiers::SHIFT) => KeyAction::CopyMode(CopyModeAction::MoveToTop),
                KeyCombo::new(KeyCode::KeyV, Modifiers::NONE) => KeyAction::CopyMode(CopyModeAction::ToggleSelection),
                KeyCombo::new(KeyCode::KeyV, Modifiers::SHIFT) => KeyAction::CopyMode(CopyModeAction::ToggleLineSelection),
                KeyCombo::new(KeyCode::KeyY, Modifiers::NONE) => KeyAction::CopyMode(CopyModeAction::CopyAndExit),
                KeyCombo::new(KeyCode::Escape, Modifiers::NONE) => KeyAction::CopyMode(CopyModeAction::Exit),
                KeyCombo::new(KeyCode::KeyQ, Modifiers::NONE) => KeyAction::CopyMode(CopyModeAction::Exit),
            },
        });

        // Default search_mode table
        registry.register("search_mode", KeyTable {
            name: "search_mode".into(),
            bindings: hashmap! {
                KeyCombo::new(KeyCode::Enter, Modifiers::NONE) => KeyAction::Search(SearchAction::Confirm),
                KeyCombo::new(KeyCode::Escape, Modifiers::NONE) => KeyAction::Search(SearchAction::Cancel),
                KeyCombo::new(KeyCode::KeyN, Modifiers::CTRL) => KeyAction::Search(SearchAction::NextMatch),
                KeyCombo::new(KeyCode::KeyP, Modifiers::CTRL) => KeyAction::Search(SearchAction::PrevMatch),
            },
        });

        registry
    }
}
```

## Implementation Plan

### Phase 1: Core Infrastructure (Week 1)

1. Define `KeyTableStack`, `KeyTable`, `KeyCombo`
2. Implement key resolution algorithm
3. Create `KeyTableRegistry` resource
4. Wire up basic input handling

### Phase 2: Leader Key (Week 1)

1. Implement `LeaderKeyState`
2. Add leader modifier to `KeyCombo`
3. Handle leader timeout
4. Visual indicator for leader active

### Phase 3: Key Actions (Week 2)

1. Define `KeyAction` enum
2. Implement `ActivateKeyTable`, `PopKeyTable`
3. Implement pane/tab/window actions
4. Wire up IPC for cross-process actions

### Phase 4: Configuration (Week 2)

1. Parse key tables from config.fsx
2. Parse key tables from config.toml
3. Implement hot-reload of key tables
4. Default key tables

### Phase 5: Mode Indicator (Week 2-3)

1. Create `ModeIndicator` resource
2. Update system for mode changes
3. Status bar integration
4. Optional overlay display

## Testing Strategy

### Unit Tests

```rust
#[test]
fn test_key_table_stack_resolution() {
    let mut stack = KeyTableStack::default();

    // Add a table
    stack.stack.push(KeyTableActivation {
        name: "test".into(),
        table: KeyTable {
            bindings: hashmap! {
                KeyCombo::new(KeyCode::KeyH, Modifiers::NONE) => KeyAction::Noop,
            },
        },
        one_shot: false,
        timeout: None,
        replace_current: false,
    });

    // H should resolve to the table's action
    let combo = KeyCombo::new(KeyCode::KeyH, Modifiers::NONE);
    assert!(stack.resolve(&combo).is_some());

    // J should fall through to default
    let combo = KeyCombo::new(KeyCode::KeyJ, Modifiers::NONE);
    assert!(stack.resolve(&combo).is_none());
}

#[test]
fn test_one_shot_pop() {
    let mut stack = KeyTableStack::default();

    stack.stack.push(KeyTableActivation {
        name: "test".into(),
        table: KeyTable { bindings: HashMap::new() },
        one_shot: true,
        timeout: None,
        replace_current: false,
    });

    assert_eq!(stack.stack.len(), 1);

    // Any keypress should pop
    stack.handle_key(KeyCombo::new(KeyCode::KeyX, Modifiers::NONE), Instant::now());

    assert_eq!(stack.stack.len(), 0);
}
```

### Integration Tests

```fsx
// test_key_tables.fsx
// Verify leader key -> resize mode flow
TestHelper.PressKey(KeyCode.A, Modifiers.Ctrl)  // Leader
assert (Scarab.LeaderIsActive())

TestHelper.PressKey(KeyCode.R, Modifiers.None)  // Enter resize
assert (Scarab.GetCurrentMode() = "RESIZE_PANE")

TestHelper.PressKey(KeyCode.H, Modifiers.None)  // Resize left
// Verify pane resized

TestHelper.PressKey(KeyCode.Escape, Modifiers.None)  // Exit
assert (Scarab.GetCurrentMode() = "NORMAL")
```

## Success Criteria

- [ ] Leader key activates with configured timeout
- [ ] `ActivateKeyTable` pushes new table to stack
- [ ] One-shot tables pop after single keypress
- [ ] Timeout-based tables expire correctly
- [ ] Mode indicator shows current mode
- [ ] Default copy_mode key table works
- [ ] Custom key tables load from config
