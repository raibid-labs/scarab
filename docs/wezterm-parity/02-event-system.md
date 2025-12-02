# WS-2: Rich Event System

**Workstream ID:** WS-2
**Priority:** P0 (Critical Path)
**Estimated Complexity:** Medium
**Dependencies:** WS-1 (Object Model - partial)

## Overview

WezTerm's power comes from its extensive event system—users can hook into virtually anything. Scarab currently has 8 hook types; we need 20+ to match WezTerm's granularity.

## Current State Analysis

### Scarab's Existing Events

```rust
pub enum HookType {
    PreOutput,      // Before line displayed
    PostInput,      // After user input
    PreCommand,     // Before shell command
    PostCommand,    // After shell command
    OnResize,       // Terminal resized
    OnAttach,       // Client connected
    OnDetach,       // Client disconnected
    OnLoad,         // Plugin loaded (lifecycle)
}
```

**Limitations:**
- No focus events (window, pane)
- No title change events
- No bell/notification events
- No status bar update trigger
- No tab events (created, closed, switched)
- No custom event system

### WezTerm's Event System

```lua
-- WezTerm's wezterm.on supports these built-in events:
wezterm.on('gui-startup', function(cmd) end)
wezterm.on('update-status', function(window, pane) end)
wezterm.on('update-right-status', function(window, pane) end)
wezterm.on('format-tab-title', function(tab, tabs, panes, config, hover, max_width) end)
wezterm.on('format-window-title', function(tab, pane, tabs, panes, config) end)
wezterm.on('window-config-reloaded', function(window, pane) end)
wezterm.on('window-focus-changed', function(window, pane) end)
wezterm.on('window-resized', function(window, pane) end)
wezterm.on('bell', function(window, pane) end)
wezterm.on('open-uri', function(window, pane, uri) end)
wezterm.on('user-var-changed', function(window, pane, name, value) end)
wezterm.on('new-tab-button-click', function(window, pane, button) end)
wezterm.on('augment-command-palette', function(window, pane) end)

-- Plus custom events via wezterm.emit()
wezterm.emit('my-custom-event', arg1, arg2)
```

## Target Event Catalog

### Window Events

| Event Name | Parameters | Description |
|------------|------------|-------------|
| `WindowCreated` | `window` | New window opened |
| `WindowClosed` | `window_id` | Window closing |
| `WindowFocusChanged` | `window, pane, is_focused` | Focus gained/lost |
| `WindowResized` | `window, pane, cols, rows` | Dimensions changed |
| `WindowConfigReloaded` | `window, pane` | Config hot-reloaded |
| `GuiStartup` | - | GUI initialized |

### Tab Events

| Event Name | Parameters | Description |
|------------|------------|-------------|
| `TabCreated` | `tab, window` | New tab opened |
| `TabClosed` | `tab_id, window` | Tab closing |
| `TabSwitched` | `tab, window, old_tab_id` | Active tab changed |
| `NewTabButtonClick` | `window, pane, button` | + button clicked |

### Pane Events

| Event Name | Parameters | Description |
|------------|------------|-------------|
| `PaneCreated` | `pane, tab` | New pane spawned |
| `PaneClosed` | `pane_id, tab` | Pane closing |
| `PaneFocused` | `pane, window` | Pane gained focus |
| `PaneTitleChanged` | `pane, old_title, new_title` | Title updated |
| `PaneOutput` | `pane, text` | Output received (existing) |
| `PaneInput` | `pane, input` | Input sent (existing) |

### Terminal Events

| Event Name | Parameters | Description |
|------------|------------|-------------|
| `Bell` | `pane` | Terminal bell rang |
| `SelectionChanged` | `pane, selection_text` | Selection modified |
| `UserVarChanged` | `pane, name, value` | OSC user variable |
| `OpenUri` | `window, pane, uri` | Hyperlink clicked |
| `ScrollbackCleared` | `pane` | Buffer cleared |

### Status Events

| Event Name | Parameters | Description |
|------------|------------|-------------|
| `UpdateStatus` | `window, pane` | Periodic status update |
| `UpdateRightStatus` | `window, pane` | Right status needs refresh |
| `UpdateLeftStatus` | `window, pane` | Left status needs refresh |

### UI Events

| Event Name | Parameters | Description |
|------------|------------|-------------|
| `FormatTabTitle` | `tab, tabs, config` | Tab title formatting |
| `FormatWindowTitle` | `tab, pane, config` | Window title formatting |
| `AugmentCommandPalette` | `window, pane` | Add palette commands |

### Custom Events

```rust
// Allow users to define and emit custom events
Scarab.on("my-plugin-event", fn(args) { ... })
Scarab.emit("my-plugin-event", arg1, arg2)
```

## Architecture Design

### Event Registry

```rust
// In scarab-plugin-api
pub struct EventRegistry {
    handlers: HashMap<EventType, Vec<HandlerEntry>>,
    custom_handlers: HashMap<String, Vec<HandlerEntry>>,
    next_handler_id: AtomicU64,
}

pub struct HandlerEntry {
    pub id: u64,
    pub plugin_name: String,
    pub callback: Box<dyn EventCallback>,
    pub priority: i32,  // Higher = called first
}

pub trait EventCallback: Send + Sync {
    fn invoke(&self, args: &EventArgs) -> Result<EventResult, EventError>;
}

pub enum EventResult {
    Continue,           // Allow other handlers
    Stop,              // Prevent further handlers
    Modified(Vec<u8>), // Return modified data
}
```

### Event Types

```rust
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum EventType {
    // Window
    WindowCreated,
    WindowClosed,
    WindowFocusChanged,
    WindowResized,
    WindowConfigReloaded,
    GuiStartup,

    // Tab
    TabCreated,
    TabClosed,
    TabSwitched,
    NewTabButtonClick,

    // Pane
    PaneCreated,
    PaneClosed,
    PaneFocused,
    PaneTitleChanged,

    // Terminal
    Bell,
    SelectionChanged,
    UserVarChanged,
    OpenUri,

    // Status
    UpdateStatus,
    FormatTabTitle,
    FormatWindowTitle,

    // Legacy (map to existing hooks)
    Output,
    Input,
    PreCommand,
    PostCommand,
    Resize,
    Attach,
    Detach,

    // Custom
    Custom(String),
}
```

### Event Arguments

```rust
pub struct EventArgs {
    pub event_type: EventType,
    pub window: Option<ObjectHandle>,
    pub pane: Option<ObjectHandle>,
    pub tab: Option<ObjectHandle>,
    pub data: EventData,
    pub timestamp: Instant,
}

pub enum EventData {
    None,
    Text(String),
    Uri(String),
    FocusState { is_focused: bool },
    Dimensions { cols: u16, rows: u16 },
    TitleChange { old: String, new: String },
    UserVar { name: String, value: String },
    Selection { text: String, start: (u16, u16), end: (u16, u16) },
    MouseButton(MouseButton),
    ExitCode(i32),
}
```

## Event Flow

### Daemon-Side Events

```
┌─────────────────────────────────────────────────────────────────┐
│ Event Source (VTE Parser, PTY, IPC)                             │
└───────────────────────────┬─────────────────────────────────────┘
                            │ Raw Event
                            ▼
┌─────────────────────────────────────────────────────────────────┐
│ Event Dispatcher                                                 │
│                                                                  │
│ 1. Create EventArgs with object handles                         │
│ 2. Look up handlers in EventRegistry                            │
│ 3. Execute handlers in priority order                           │
│ 4. Check for Stop result                                         │
│ 5. Forward to client if needed (IPC)                            │
└───────────────────────────┬─────────────────────────────────────┘
                            │
           ┌────────────────┼────────────────┐
           ▼                ▼                ▼
    ┌───────────┐    ┌───────────┐    ┌───────────┐
    │ Plugin 1  │    │ Plugin 2  │    │ Plugin N  │
    │ Handler   │    │ Handler   │    │ Handler   │
    └───────────┘    └───────────┘    └───────────┘
```

### Client-Side Events

Some events originate in the client (window focus, UI interactions):

```rust
// In scarab-client
fn handle_window_focus_changed(
    windows: Query<&Window>,
    focus_events: EventReader<WindowFocused>,
    event_registry: Res<EventRegistry>,
    object_registry: Res<ObjectRegistry>,
) {
    for event in focus_events.read() {
        let window_handle = object_registry.get_window_handle(event.window);
        let pane_handle = object_registry.get_active_pane_handle(event.window);

        let args = EventArgs {
            event_type: EventType::WindowFocusChanged,
            window: Some(window_handle),
            pane: Some(pane_handle),
            data: EventData::FocusState { is_focused: event.focused },
            ..Default::default()
        };

        event_registry.dispatch(&args);
    }
}
```

## Integration with Fusabi

### Event Registration Syntax

```fsx
// In .fsx config file
module Config

open Scarab.Events

// Register handler for built-in event
On(EventType.WindowFocusChanged, fun window pane isFocused ->
    if isFocused then
        window.SetRightStatus([Text "Focused!"])
    else
        window.SetRightStatus([Text ""])
)

// Register handler for custom event
On("my-plugin-event", fun arg1 arg2 ->
    printfn "Custom event: %A %A" arg1 arg2
)

// Emit custom event
Emit("my-plugin-event", "hello", 42)
```

### Handler Registration in Rust

```rust
impl Plugin for MyPlugin {
    async fn on_load(&mut self, ctx: &mut PluginContext) -> Result<()> {
        // Register for events
        ctx.register_event(EventType::Bell, |args| {
            // Handle bell
            Ok(EventResult::Continue)
        });

        ctx.register_event(EventType::UpdateStatus, |args| {
            // Return status bar content
            let items = vec![
                RenderItem::text("Hello"),
                RenderItem::colored("World", Color::Blue),
            ];
            Ok(EventResult::StatusBarContent(items))
        });

        Ok(())
    }
}
```

## Status Bar Event Integration

The `UpdateStatus` event is special—it returns content to render:

```rust
pub fn dispatch_status_update(
    registry: &EventRegistry,
    window: ObjectHandle,
    pane: ObjectHandle,
) -> Vec<RenderItem> {
    let args = EventArgs {
        event_type: EventType::UpdateStatus,
        window: Some(window),
        pane: Some(pane),
        data: EventData::None,
        timestamp: Instant::now(),
    };

    let mut items = Vec::new();
    for handler in registry.get_handlers(&EventType::UpdateStatus) {
        match handler.invoke(&args) {
            Ok(EventResult::StatusBarContent(content)) => {
                items.extend(content);
            }
            Ok(EventResult::Stop) => break,
            _ => continue,
        }
    }
    items
}
```

## Implementation Plan

### Phase 1: Event Infrastructure (Week 1)

1. Define `EventType` enum with all target events
2. Create `EventRegistry` with handler storage
3. Implement `EventArgs` and `EventData`
4. Add dispatch logic with priority ordering

### Phase 2: Core Events (Week 1-2)

1. Wire up existing hooks to new event system
2. Add `WindowFocusChanged` (Bevy integration)
3. Add `Bell` event (VTE parser)
4. Add `PaneTitleChanged` (VTE OSC handling)

### Phase 3: Tab/Pane Events (Week 2)

1. Add `TabCreated`, `TabClosed`, `TabSwitched`
2. Add `PaneCreated`, `PaneClosed`, `PaneFocused`
3. Wire up IPC to propagate events across processes

### Phase 4: Status Events (Week 2-3)

1. Implement `UpdateStatus` with render item return
2. Add `FormatTabTitle`, `FormatWindowTitle`
3. Integrate with status bar rendering (WS-3)

### Phase 5: Custom Events (Week 3)

1. Add `Custom(String)` event type
2. Implement `Scarab.on()` and `Scarab.emit()` in Fusabi
3. Document event system for users

## Event Frequency Considerations

### High-Frequency Events

Some events fire very often and need throttling:

| Event | Frequency | Strategy |
|-------|-----------|----------|
| `UpdateStatus` | 1-10 Hz | Configurable interval |
| `SelectionChanged` | During drag | Debounce 50ms |
| `Output` | Per line | Batch if needed |

### Configuration

```toml
[events]
status_update_interval_ms = 100
selection_debounce_ms = 50
max_output_batch_size = 100
```

## Backward Compatibility

The existing `HookType` enum maps to new events:

```rust
impl From<HookType> for EventType {
    fn from(hook: HookType) -> Self {
        match hook {
            HookType::PreOutput => EventType::Output,
            HookType::PostInput => EventType::Input,
            HookType::PreCommand => EventType::PreCommand,
            HookType::PostCommand => EventType::PostCommand,
            HookType::OnResize => EventType::Resize,
            HookType::OnAttach => EventType::Attach,
            HookType::OnDetach => EventType::Detach,
        }
    }
}
```

Existing plugins continue to work via the legacy `Plugin` trait methods.

## Testing Strategy

### Unit Tests

```rust
#[test]
fn test_event_dispatch_order() {
    let mut registry = EventRegistry::new();

    let order = Arc::new(Mutex::new(Vec::new()));

    let order_clone = order.clone();
    registry.register(EventType::Bell, 10, move |_| {
        order_clone.lock().unwrap().push("high");
        Ok(EventResult::Continue)
    });

    let order_clone = order.clone();
    registry.register(EventType::Bell, 0, move |_| {
        order_clone.lock().unwrap().push("low");
        Ok(EventResult::Continue)
    });

    registry.dispatch(&EventArgs::new(EventType::Bell));

    assert_eq!(*order.lock().unwrap(), vec!["high", "low"]);
}
```

### Integration Tests

```fsx
// test_events.fsx
let mutable bellCount = 0

On(EventType.Bell, fun pane ->
    bellCount <- bellCount + 1
    Continue
)

// Trigger bell in test
TestHelper.SendBell()
assert (bellCount = 1)
```

## Success Criteria

- [ ] All 12 WezTerm window events have Scarab equivalents
- [ ] Custom events work via `On`/`Emit` in Fusabi
- [ ] `UpdateStatus` returns render items correctly
- [ ] Event priority ordering is respected
- [ ] High-frequency events are properly throttled
- [ ] Existing plugins work without modification
