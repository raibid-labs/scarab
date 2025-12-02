# WezTerm Parity: Detailed Workstream Breakdown

**Document:** Parallelization Strategy & Dependency Analysis
**Date:** December 2, 2025

## Dependency Graph

```
                    ┌─────────────────────────────────────────┐
                    │                                         │
                    │           WS-5: Image Protocols         │
                    │           (Fully Parallel)              │
                    │                                         │
                    └─────────────────────────────────────────┘


    ┌───────────────┐
    │               │
    │  WS-1: Object │────────────┬────────────────────────────┐
    │    Model      │            │                            │
    │               │            │                            │
    └───────┬───────┘            │                            │
            │                    │                            │
            │ (partial)         │                            │
            ▼                    ▼                            ▼
    ┌───────────────┐    ┌───────────────┐           ┌───────────────┐
    │               │    │               │           │               │
    │  WS-2: Event  │    │  WS-3: Status │           │  WS-4: Key    │
    │    System     │◄───│    Bar API    │           │   Tables      │
    │               │    │               │           │               │
    └───────┬───────┘    └───────────────┘           └───────┬───────┘
            │                                                │
            │                                                │
            │                                                ▼
            │                                        ┌───────────────┐
            │                                        │               │
            └───────────────────────────────────────►│  WS-6: Copy   │
                                                     │    Mode       │
                                                     │               │
                                                     └───────────────┘
```

## Parallel Execution Matrix

| Week | Agent 1 | Agent 2 | Agent 3 |
|------|---------|---------|---------|
| 1 | **WS-1** Object Model (Core) | **WS-5** Image Protocols (iTerm2) | **WS-2** Event System (Start) |
| 2 | **WS-1** Object Model (Pane/Tab) | **WS-5** Image Protocols (Kitty) | **WS-2** Event System (Complete) |
| 3 | **WS-3** Status Bar API | **WS-5** Image Protocols (Sixel) | **WS-4** Key Tables |
| 4 | **WS-3** Status Bar (Polish) | **WS-6** Copy Mode | **WS-4** Key Tables (Complete) |
| 5 | Integration Testing | **WS-6** Copy Mode (Search) | Documentation |
| 6 | Bug Fixes | Bug Fixes | Release Preparation |

## Workstream Details

### WS-1: Fusabi Object Model Infrastructure

**Files to Create/Modify:**
```
crates/scarab-plugin-api/src/
├── object_model/
│   ├── mod.rs           # Module exports
│   ├── handle.rs        # ObjectHandle, ObjectType
│   ├── registry.rs      # ObjectRegistry trait + impls
│   ├── dispatcher.rs    # Method dispatch logic
│   ├── window.rs        # WindowProxy implementation
│   ├── pane.rs          # PaneProxy implementation
│   └── tab.rs           # TabProxy implementation
├── lib.rs               # Add object_model export

crates/scarab-daemon/src/
├── object_registry.rs   # DaemonObjectRegistry
├── plugin_manager/
│   └── mod.rs          # Update to provide objects

crates/scarab-client/src/
├── object_registry.rs   # ClientObjectRegistry
├── integration.rs       # Wire up object access

crates/scarab-protocol/src/
├── queries.rs           # DaemonQuery, QueryResponse
└── lib.rs              # Add query types
```

**Key Decisions:**
1. Use handle-based proxies (not raw references)
2. Generation counters for stale handle detection
3. Runtime dispatch initially (Option B from spec)
4. Async methods for IPC round-trips

**Deliverables:**
- [ ] `ObjectHandle` struct with type and ID
- [ ] `ObjectRegistry` trait with client/daemon impls
- [ ] `WindowProxy` with 10+ methods
- [ ] `PaneProxy` with 15+ methods
- [ ] `TabProxy` with 8+ methods
- [ ] IPC query/response messages
- [ ] Unit tests for dispatch logic

---

### WS-2: Rich Event System

**Files to Create/Modify:**
```
crates/scarab-plugin-api/src/
├── events/
│   ├── mod.rs           # Module exports
│   ├── types.rs         # EventType enum
│   ├── args.rs          # EventArgs, EventData
│   ├── registry.rs      # EventRegistry
│   └── handler.rs       # HandlerEntry, EventCallback

crates/scarab-daemon/src/
├── event_dispatcher.rs  # Daemon-side dispatch
├── vte.rs              # Add event firing
└── ipc.rs              # Propagate events to client

crates/scarab-client/src/
├── event_dispatcher.rs  # Client-side dispatch
└── systems/
    └── events.rs        # Bevy event integration
```

**Key Decisions:**
1. Separate registries for daemon/client events
2. Priority-based handler ordering
3. Throttling for high-frequency events
4. Backward compatibility with existing hooks

**Deliverables:**
- [ ] `EventType` enum with 20+ event types
- [ ] `EventRegistry` with priority dispatch
- [ ] Window events (focus, resize, etc.)
- [ ] Pane events (title, output, etc.)
- [ ] Custom event support (`On`/`Emit`)
- [ ] Integration tests

---

### WS-3: Status Bar Rendering API

**Files to Create/Modify:**
```
crates/scarab-plugin-api/src/
├── ui/
│   ├── mod.rs           # Module exports
│   ├── render_item.rs   # RenderItem enum
│   └── format.rs        # Format helper function

crates/scarab-client/src/
├── ui/
│   ├── status_bar.rs    # StatusBarState, setup, update
│   └── render_items.rs  # RenderItem -> Bevy Text conversion

crates/scarab-protocol/src/
└── lib.rs              # Add StatusBarUpdate message
```

**Key Decisions:**
1. Support both left and right status areas
2. Periodic + event-driven updates
3. Built-in components (clock, cwd, etc.)
4. Theme-aware color resolution

**Deliverables:**
- [ ] `RenderItem` enum with all variants
- [ ] `window.SetRightStatus()` method
- [ ] `window.SetLeftStatus()` method
- [ ] `UpdateStatus` event integration
- [ ] Built-in status components
- [ ] Example configurations

---

### WS-4: Key Tables & Modal Editing

**Files to Create/Modify:**
```
crates/scarab-client/src/
├── input/
│   ├── mod.rs           # Module exports
│   ├── key_table.rs     # KeyTable, KeyTableStack
│   ├── leader.rs        # LeaderKeyState
│   ├── actions.rs       # KeyAction enum
│   └── systems.rs       # Bevy input systems

crates/scarab-config/src/
├── keys.rs              # Key config parsing
└── defaults.rs          # Default key tables
```

**Key Decisions:**
1. Stack-based key table activation
2. One-shot and timeout modes
3. Leader key as virtual modifier
4. Default copy_mode table

**Deliverables:**
- [ ] `KeyTableStack` with resolution algorithm
- [ ] `LeaderKeyState` with timeout
- [ ] `ActivateKeyTable` action
- [ ] Config parsing (Fusabi + TOML)
- [ ] Default key tables
- [ ] Mode indicator integration

---

### WS-5: Image Protocol Support

**Files to Create/Modify:**
```
crates/scarab-daemon/src/
├── images/
│   ├── mod.rs           # Module exports
│   ├── iterm2.rs        # iTerm2 protocol parser
│   ├── kitty.rs         # Kitty protocol parser
│   ├── sixel.rs         # Sixel decoder
│   └── placement.rs     # ImagePlacementState
├── vte.rs              # Add OSC/APC/DCS handlers

crates/scarab-protocol/src/
├── images.rs            # ImagePlacement, ImageSharedMemory

crates/scarab-client/src/
├── rendering/
│   ├── images.rs        # ImageCache, render systems
│   └── mod.rs           # Add image module

tools/
└── imgcat/
    └── main.rs          # imgcat utility
```

**Key Decisions:**
1. Shared memory for large image data
2. iTerm2 first (simplest, widest support)
3. Cell-based placement (like WezTerm)
4. LRU cache for memory management

**Deliverables:**
- [ ] iTerm2 protocol parser
- [ ] Image shared memory region
- [ ] Bevy sprite rendering
- [ ] Image lifecycle management
- [ ] `imgcat` utility
- [ ] Kitty basic support (stretch)
- [ ] Sixel basic support (stretch)

---

### WS-6: Copy Mode & Advanced Selection

**Files to Create/Modify:**
```
crates/scarab-client/src/
├── copy_mode/
│   ├── mod.rs           # Module exports
│   ├── state.rs         # CopyModeState, Selection
│   ├── actions.rs       # CopyModeAction enum
│   ├── navigation.rs    # Movement logic
│   ├── selection.rs     # Text extraction
│   ├── search.rs        # Search functionality
│   └── systems.rs       # Bevy systems
├── ui/
│   └── copy_mode_ui.rs  # Cursor, highlights, indicator
```

**Key Decisions:**
1. Vim-like key bindings by default
2. Cell, line, and block selection modes
3. Integrated search with highlighting
4. Seamless clipboard integration

**Deliverables:**
- [ ] `CopyModeState` resource
- [ ] Movement actions (hjkl, word, etc.)
- [ ] Selection modes (v, V, Ctrl+v)
- [ ] Text extraction for all modes
- [ ] Search with n/N navigation
- [ ] Mode indicator
- [ ] Default key table

---

## Cross-Cutting Concerns

### IPC Additions

All workstreams add new IPC messages. Coordinate to avoid conflicts:

```rust
// Consolidated additions to ControlMessage
pub enum ControlMessage {
    // ... existing ...

    // WS-1: Object queries
    ObjectQuery { query: DaemonQuery },

    // WS-2: Event control
    EventSubscribe { event_type: EventType },

    // WS-5: Image data
    ImageAck { image_id: u64 },
}

// Consolidated additions to DaemonMessage
pub enum DaemonMessage {
    // ... existing ...

    // WS-1: Query responses
    QueryResponse { response: QueryResponse },

    // WS-2: Event notifications
    EventFired { event: EventType, args: EventArgs },

    // WS-3: Status updates
    StatusBarUpdate { side: StatusBarSide, items: Vec<RenderItem> },

    // WS-5: Image placements
    ImagePlaced { placement: ImagePlacement },
}
```

### Fusabi Bindings

Each workstream exposes new types to Fusabi. Coordinate naming:

```fsx
// WS-1: Objects
Window.Current()
pane.SendText("hello")

// WS-2: Events
On(EventType.Bell, handler)
Emit("custom-event", args)

// WS-3: Status Bar
window.SetRightStatus([Text("hello")])
Format([{Foreground = Color.Red}; {Text = "hi"}])

// WS-4: Key Tables
ActivateKeyTable { Name = "resize"; OneShot = false }

// WS-6: Copy Mode
CopyModeAction.MoveForwardWord
```

### Testing Strategy

**Per-Workstream:**
- Unit tests in `#[cfg(test)]` modules
- Integration tests in `tests/` directory

**Cross-Workstream:**
- End-to-end tests after WS-1 + WS-2 complete
- Status bar tests after WS-3 depends on WS-1/WS-2
- Full workflow tests after all complete

### Documentation

Each workstream produces:
1. API documentation (rustdoc)
2. User guide section (mdBook)
3. Example configurations
4. Migration guide (for breaking changes)

---

## Risk Assessment

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|------------|
| Object model performance | Medium | High | Profile early, cache aggressively |
| IPC message size limits | Low | Medium | Use shared memory for large data |
| Fusabi binding complexity | Medium | Medium | Start with runtime dispatch |
| Bevy 0.15 breaking changes | Low | High | Pin version, test thoroughly |
| Image memory exhaustion | Medium | Medium | LRU cache, configurable limits |
| Key table conflicts | Low | Low | Clear documentation, validation |

---

## Success Metrics

### Quantitative
- **Object Model:** <10ms method call latency
- **Events:** <1ms dispatch overhead
- **Status Bar:** 60fps rendering maintained
- **Images:** Support files up to 10MB
- **Copy Mode:** <100ms for 10k line search

### Qualitative
- Users can replicate WezTerm configs in Fusabi
- No Rust changes needed for custom status bars
- Copy mode feels "vim-like"
- Image rendering matches WezTerm quality

---

## Handoff Checklist

Before starting a workstream, ensure:
- [ ] Previous dependencies are at least 50% complete
- [ ] IPC message additions are merged
- [ ] Shared types (enums, structs) are defined
- [ ] Test fixtures are available

After completing a workstream:
- [ ] All tests pass
- [ ] Documentation is written
- [ ] Example configs work
- [ ] Performance meets targets
- [ ] Integration points are stable

---

## Coordination Points

**Daily Sync Topics:**
1. IPC message additions
2. Fusabi type naming
3. Shared resource conflicts
4. Test infrastructure needs

**Weekly Review Topics:**
1. Dependency progress
2. Risk status
3. Performance metrics
4. User feedback integration
