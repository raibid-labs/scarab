# WS-1: Fusabi Object Model Infrastructure

**Workstream ID:** WS-1
**Priority:** P0 (Critical Path)
**Estimated Complexity:** High
**Dependencies:** None

## Overview

This workstream establishes the foundation for exposing Scarab's internal state to Fusabi scripts as callable objects. WezTerm's power derives from providing live `Window`, `Pane`, and `Tab` objects that users can query and manipulate—we must do the same.

## Current State Analysis

### Scarab's Existing Approach

```rust
// Current: Stateless context passed to hooks
pub struct PluginContext {
    pub config: PluginConfigData,
    pub state: Arc<Mutex<PluginSharedState>>,
    // ... utility methods, no object handles
}

// Plugins receive a snapshot, not a live reference
async fn on_output(&mut self, line: &str, ctx: &PluginContext) -> Result<Action>
```

**Limitations:**
- No way to access "the current window" or "all tabs"
- Cannot call methods like `pane.send_text("hello")`
- No object identity (can't store references between calls)

### WezTerm's Approach

```lua
-- WezTerm: Callbacks receive live objects
wezterm.on('update-status', function(window, pane)
  local title = pane:get_title()
  local cwd = pane:get_current_working_dir()
  window:set_right_status(wezterm.format{
    {Text = cwd.file_path},
  })
end)
```

**Power:**
- Objects have methods that execute immediately
- Objects navigate to related objects (`pane:tab()`, `tab:window()`)
- Users write UI logic in scripting, not Rust

## Architecture Design

### Core Concept: Handle-Based Proxies

Instead of passing raw Rust structs to Fusabi (impossible due to memory safety), we use **handles**—lightweight IDs that proxy method calls through a dispatcher.

```
┌─────────────────────────────────────────────────────────────┐
│ Fusabi Script                                               │
│                                                             │
│   let pane = Window.ActivePane()                            │
│   pane.SendText("hello")                                    │
│                                                             │
└────────────────────┬────────────────────────────────────────┘
                     │ Method call with handle
                     ▼
┌─────────────────────────────────────────────────────────────┐
│ Object Dispatcher (Rust)                                    │
│                                                             │
│   match (handle.type, method_name) {                        │
│     (ObjectType::Pane, "SendText") => {                     │
│       let pane_id = handle.id;                              │
│       ipc_sender.send(ControlMessage::Input {               │
│         pane_id, data: args[0].as_bytes()                   │
│       })                                                    │
│     }                                                       │
│   }                                                         │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

### Object Handle Structure

```rust
// In scarab-plugin-api
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ObjectHandle {
    pub object_type: ObjectType,
    pub id: u64,
    pub generation: u32,  // Detect stale handles
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ObjectType {
    Window,
    Tab,
    Pane,
    MuxWindow,    // For daemon-side mux operations
    MuxTab,
    MuxPane,
}
```

### Object Registries

Each process (client/daemon) maintains a registry mapping handles to live objects:

```rust
// In scarab-client (Bevy)
pub struct ObjectRegistry {
    windows: HashMap<u64, Entity>,    // Bevy Entity
    tabs: HashMap<u64, Entity>,
    panes: HashMap<u64, Entity>,
    next_id: AtomicU64,
    generation: AtomicU32,
}

// In scarab-daemon
pub struct DaemonObjectRegistry {
    sessions: HashMap<u64, SessionId>,
    panes: HashMap<u64, PtyHandle>,
    // ...
}
```

### Method Dispatch

```rust
pub trait FusabiObject: Send + Sync {
    fn type_name(&self) -> &'static str;
    fn call_method(
        &self,
        method: &str,
        args: &[FusabiValue],
        ctx: &DispatchContext,
    ) -> Result<FusabiValue, ObjectError>;
    fn get_property(&self, name: &str) -> Result<FusabiValue, ObjectError>;
}

// Example implementation for Pane
impl FusabiObject for PaneProxy {
    fn call_method(&self, method: &str, args: &[FusabiValue], ctx: &DispatchContext) -> Result<FusabiValue> {
        match method {
            "SendText" => {
                let text = args.get(0).ok_or(ObjectError::MissingArg)?.as_string()?;
                ctx.ipc.send(ControlMessage::Input {
                    pane_id: self.id,
                    data: text.as_bytes().to_vec(),
                });
                Ok(FusabiValue::Unit)
            }
            "GetTitle" => {
                Ok(FusabiValue::String(self.cached_title.clone()))
            }
            "GetCurrentWorkingDir" => {
                // This may require IPC round-trip for fresh data
                let cwd = ctx.query_daemon(DaemonQuery::PaneCwd(self.id))?;
                Ok(FusabiValue::String(cwd))
            }
            _ => Err(ObjectError::UnknownMethod(method.to_string()))
        }
    }
}
```

## API Surface Design

### Window Object

Maps to a Bevy window entity in the client.

| Method | WezTerm Equivalent | Description |
|--------|-------------------|-------------|
| `Window.Current()` | - | Get the current window handle |
| `window.ActivePane()` | `window:active_pane()` | Get focused pane |
| `window.ActiveTab()` | `window:active_tab()` | Get active tab |
| `window.Tabs()` | - | List all tabs |
| `window.SetRightStatus(items)` | `window:set_right_status()` | Set status bar |
| `window.SetLeftStatus(items)` | `window:set_left_status()` | Set status bar |
| `window.ToastNotification(msg)` | `window:toast_notification()` | Show notification |
| `window.GetDimensions()` | `window:get_dimensions()` | Window size |
| `window.Maximize()` | `window:maximize()` | Maximize window |
| `window.ToggleFullscreen()` | `window:toggle_fullscreen()` | Fullscreen toggle |
| `window.PerformAction(action)` | `window:perform_action()` | Execute key action |

### Pane Object

Maps to a PTY session in the daemon.

| Method | WezTerm Equivalent | Description |
|--------|-------------------|-------------|
| `pane.SendText(text)` | `pane:send_text()` | Send input to PTY |
| `pane.SendPaste(text)` | `pane:send_paste()` | Send as paste (bracketed) |
| `pane.GetTitle()` | `pane:get_title()` | Get pane title |
| `pane.GetCurrentWorkingDir()` | `pane:get_current_working_dir()` | Get CWD |
| `pane.GetCursorPosition()` | `pane:get_cursor_position()` | Cursor (x, y) |
| `pane.GetDimensions()` | `pane:get_dimensions()` | Pane size in cells |
| `pane.GetForegroundProcessName()` | `pane:get_foreground_process_name()` | Active process |
| `pane.GetLinesAsText(start, end)` | `pane:get_lines_as_text()` | Extract text |
| `pane.IsAltScreenActive()` | `pane:is_alt_screen_active()` | Alt buffer check |
| `pane.HasUnseenOutput()` | `pane:has_unseen_output()` | New output flag |
| `pane.Activate()` | `pane:activate()` | Focus this pane |
| `pane.Split(direction)` | `pane:split()` | Create split |
| `pane.Tab()` | `pane:tab()` | Get parent tab |
| `pane.Window()` | `pane:window()` | Get parent window |
| `pane.PaneId()` | `pane:pane_id()` | Unique identifier |
| `pane.InjectOutput(text)` | `pane:inject_output()` | Write to terminal |

### Tab Object

| Method | WezTerm Equivalent | Description |
|--------|-------------------|-------------|
| `tab.TabId()` | `tab:tab_id()` | Unique identifier |
| `tab.Panes()` | `tab:panes()` | List panes in tab |
| `tab.ActivePane()` | `tab:active_pane()` | Focused pane |
| `tab.SetTitle(title)` | `tab:set_title()` | Set tab title |
| `tab.GetTitle()` | `tab:get_title()` | Get tab title |
| `tab.Activate()` | `tab:activate()` | Switch to tab |
| `tab.Window()` | `tab:window()` | Parent window |
| `tab.IsActive()` | - | Is this tab focused |

### PaneInformation (Snapshot Struct)

For performance-critical callbacks (status bar updates), provide a pre-computed snapshot:

```rust
pub struct PaneInfo {
    pub pane_id: u64,
    pub is_active: bool,
    pub is_zoomed: bool,
    pub width: u16,
    pub height: u16,
    pub title: String,
    pub foreground_process_name: String,
    pub current_working_dir: Option<String>,
    pub has_unseen_output: bool,
}
```

## Implementation Plan

### Phase 1: Core Infrastructure (Week 1)

1. **Define `ObjectHandle` and `ObjectType`** in `scarab-plugin-api`
2. **Create `ObjectRegistry`** trait and implementations
3. **Implement `FusabiObject` trait** with method dispatch
4. **Add `ObjectDispatcher`** to route calls to correct handlers

### Phase 2: Window Object (Week 1-2)

1. Register Bevy window entities in registry on creation
2. Implement `WindowProxy` with basic methods:
   - `Current()`, `ActivePane()`, `ActiveTab()`
   - `SetRightStatus()`, `SetLeftStatus()`
3. Wire up IPC for cross-process calls

### Phase 3: Pane Object (Week 2)

1. Register PTY handles in daemon registry
2. Implement `PaneProxy` with:
   - `SendText()`, `SendPaste()`
   - `GetTitle()`, `GetCursorPosition()`
   - `GetLinesAsText()`
3. Handle IPC round-trips for live data

### Phase 4: Tab Object (Week 2)

1. Track tab entities/IDs in both processes
2. Implement navigation: `Tab.Panes()`, `Pane.Tab()`
3. Tab management: `SetTitle()`, `Activate()`

### Phase 5: Integration (Week 3)

1. Update `PluginContext` to provide object access
2. Create convenience constructors (e.g., `Window.Current()`)
3. Update hook signatures to pass objects instead of raw context
4. Write comprehensive tests

## Fusabi Binding Strategy

### Option A: Generate Bindings at Compile Time

Use a macro to generate Fusabi type definitions from Rust:

```rust
#[fusabi_object]
impl WindowProxy {
    #[fusabi_method]
    pub fn active_pane(&self) -> ObjectHandle { ... }
}
```

**Pros:** Type-safe, IDE support
**Cons:** Requires Fusabi compiler integration

### Option B: Runtime Reflection

Use dynamic dispatch with string method names:

```rust
// In Fusabi script
let pane = call_method(window_handle, "ActivePane", [])
```

**Pros:** Simpler initial implementation
**Cons:** No compile-time checking, verbose

### Recommendation: Start with Option B, Migrate to A

Begin with runtime dispatch for rapid prototyping. Once stable, add code generation for better DX.

## IPC Considerations

### Sync vs Async

Some methods return immediately (cached data), others require daemon round-trips.

```rust
// Fast (cached in client)
pane.GetTitle()     // Returns cached title
pane.IsActive()     // Local state

// Slow (IPC required)
pane.GetCurrentWorkingDir()    // Daemon queries /proc
pane.GetLinesAsText(0, 100)    // Daemon reads scrollback
```

### Caching Strategy

1. **Snapshot on Callback Entry**: Pre-populate `PaneInfo` before calling hooks
2. **Lazy Loading**: Only fetch expensive data when accessed
3. **TTL Cache**: Cache CWD for 1 second, invalidate on focus change

### New IPC Messages

```rust
// Client -> Daemon queries
pub enum DaemonQuery {
    PaneCwd { pane_id: u64 },
    PaneScrollback { pane_id: u64, start: u32, end: u32 },
    PaneProcessInfo { pane_id: u64 },
    TabList { window_id: u64 },
}

// Daemon -> Client responses
pub enum QueryResponse {
    Cwd(String),
    Scrollback(Vec<String>),
    ProcessInfo { name: String, pid: u32 },
    TabList(Vec<TabInfo>),
}
```

## Testing Strategy

### Unit Tests

```rust
#[test]
fn test_pane_proxy_send_text() {
    let (tx, rx) = mpsc::channel();
    let proxy = PaneProxy::new(42, tx);

    proxy.call_method("SendText", &[FusabiValue::String("hello".into())], &ctx);

    let msg = rx.recv().unwrap();
    assert_eq!(msg, ControlMessage::Input { pane_id: 42, data: b"hello".to_vec() });
}
```

### Integration Tests

```fsx
// test_object_model.fsx
let window = Window.Current()
let pane = window.ActivePane()
assert (pane.PaneId() > 0u64)

pane.SendText("echo test\n")
// ... verify output
```

## Open Questions

1. **Handle Lifetime**: How do we invalidate handles when panes/tabs close?
   - **Proposal**: Generation counter, return error on stale handle access

2. **Thread Safety**: Fusabi scripts run on daemon thread, but some data lives in client
   - **Proposal**: All cross-process calls go through IPC, no shared mutable state

3. **Error Handling**: What happens when `GetCwd()` fails?
   - **Proposal**: Return `Option<String>`, document which methods can fail

## Success Criteria

- [ ] `Window.Current()` returns valid handle in Fusabi script
- [ ] `pane.SendText("test")` successfully sends to PTY
- [ ] `pane.GetTitle()` returns correct title
- [ ] Object navigation works: `window.ActivePane().Tab().Window()` returns same window
- [ ] Stale handles are detected and produce clear errors
