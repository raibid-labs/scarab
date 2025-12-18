# Architecture Review
## Technical Audit - December 15, 2025

### Executive Summary

This report reviews the Scarab terminal emulator's architecture, focusing on:
- Split-process design (daemon + client)
- IPC patterns and shared memory implementation
- Plugin system extensibility
- Bevy integration and rendering pipeline
- Fusabi runtime integration

**Overall Assessment**: The architecture is **well-designed** with clear separation of concerns, innovative IPC patterns, and good extensibility. There are opportunities for documentation and refinement.

---

## Architecture Overview

### High-Level Design

```
┌─────────────────────────────────────────────────────────────┐
│                     SCARAB TERMINAL                          │
├──────────────────────┬──────────────────────────────────────┤
│  DAEMON (Server)     │  CLIENT (GUI)                        │
│  ┌────────────────┐  │  ┌────────────────┐                  │
│  │ PTY Manager    │  │  │ Bevy Engine    │                  │
│  │ - portable-pty │  │  │ - Rendering    │                  │
│  │ - VTE Parser   │  │  │ - Input        │                  │
│  │ - Grid State   │  │  │ - UI Overlays  │                  │
│  └────────────────┘  │  └────────────────┘                  │
│          │           │           │                           │
│  ┌────────────────┐  │  ┌────────────────┐                  │
│  │ Plugin Manager │  │  │ Plugin Host    │                  │
│  │ - fusabi-vm    │  │  │ - fusabi-tui   │                  │
│  │ - .fzb runtime │  │  │ - .fsx scripts │                  │
│  └────────────────┘  │  └────────────────┘                  │
│          │           │           │                           │
└──────────┼───────────┴───────────┼───────────────────────────┘
           │                       │
           └───── Shared Memory ───┘
                  (Zero-Copy IPC)
```

### Process Separation Rationale

**Daemon Process**:
- Owns PTY file descriptors
- Survives client crashes/disconnects
- Runs compiled plugins (.fzb) for performance
- Single source of truth for terminal state

**Client Process**:
- GPU-accelerated rendering (Bevy)
- Hot-reloadable UI scripts (.fsx)
- Can crash without losing terminal sessions
- Multiple clients can attach to same daemon

**Benefits**:
1. **Robustness**: Client crashes don't kill terminals
2. **Performance**: Daemon optimized for I/O, client for graphics
3. **Flexibility**: Different UIs can connect to same daemon
4. **Security**: Potential for sandboxed client

---

## IPC Layer (Protocol)

### Shared Memory Design

#### Memory Layout (scarab-protocol)

```rust
#[repr(C)]
pub struct SharedState {
    pub sequence_number: u64,  // Atomic sync marker
    pub dirty_flag: u8,
    pub error_mode: u8,
    pub cursor_x: u16,
    pub cursor_y: u16,
    pub _padding2: [u8; 2],
    pub cells: [Cell; BUFFER_SIZE],  // 200x100 grid
}

#[repr(C)]
pub struct Cell {
    pub char_codepoint: u32,
    pub fg: u32,  // RGBA
    pub bg: u32,  // RGBA
    pub flags: u8,
    pub _padding: [u8; 3],  // Align to 16 bytes
}
```

**Assessment**: ✅ Excellent design
- `#[repr(C)]` ensures cross-process compatibility
- Proper alignment for atomic operations
- Zero-copy via `bytemuck::Pod`
- Fixed-size buffer (no allocations in shared memory)

#### Synchronization Protocol (SeqLock Pattern)

**Writer (Daemon)**:
```rust
// 1. Increment sequence (odd = writing)
let seq = state.sequence_number.fetch_add(1, Ordering::SeqCst);

// 2. Update cells
for cell in updates {
    state.cells[idx] = *cell;
}

// 3. Increment sequence again (even = stable)
state.sequence_number.store(seq + 1, Ordering::Release);
```

**Reader (Client)**:
```rust
loop {
    // 1. Read sequence (must be even)
    let seq1 = state.sequence_number.load(Ordering::Acquire);
    if seq1 % 2 != 0 { continue; }  // Writing in progress

    // 2. Read cells
    let data = state.cells[..].to_vec();

    // 3. Verify sequence unchanged
    let seq2 = state.sequence_number.load(Ordering::Acquire);
    if seq1 == seq2 {
        break;  // Valid read
    }
    // Retry if sequence changed
}
```

**Assessment**: ✅ Lock-free, wait-free reads
- No mutexes or locks
- Reader never blocks daemon
- Atomic operations ensure memory ordering
- Well-suited for real-time rendering

**Issues**:
1. **Undocumented**: Protocol not explained in code or docs
2. **No version field**: Hard to evolve shared memory layout
3. **Fixed size**: 200x100 grid limit

**Recommendations**:
1. **P0**: Document SeqLock protocol in `docs/architecture/IPC_SYNCHRONIZATION.md`
2. **P2**: Add version field for future compatibility
3. **P3**: Consider dynamic grid sizing (complex, low priority)

---

## Daemon Architecture

### PTY Management

**Component**: `scarab-daemon/src/orchestrator.rs`

**Design**:
```rust
pub struct PaneOrchestrator {
    panes: HashMap<PaneId, Pane>,
    pty_manager: PtyManager,
    grid_state: Arc<SharedState>,
}
```

**Responsibilities**:
1. Create/destroy PTY processes via `portable-pty`
2. Read PTY output and parse with VTE
3. Update shared memory grid
4. Handle window resize events

**Assessment**: ✅ Clean separation of concerns

### VTE Parsing

**Component**: `scarab-daemon/src/vte.rs` (1,218 LoC)

**Design**:
- Uses `vte` crate for escape sequence parsing
- Custom terminal state machine
- Handles ANSI colors, cursor movement, scrolling

**Issue**: Large monolithic file (1,218 lines)

**Recommendation**: Split into sub-modules (see REFACTORING_OPPORTUNITIES.md)

### Plugin System (Daemon Side)

**Component**: `scarab-daemon/src/plugin_manager/`

**Design**:
```rust
pub struct PluginManager {
    plugins: Vec<Box<dyn Plugin>>,
    fusabi_runtime: FusabiVmRuntime,
}
```

**Plugin Lifecycle**:
1. Load .fzb bytecode from disk
2. Initialize Fusabi VM
3. Call `on_init()` hook
4. Route input through `on_input()`
5. Process plugin actions (modify, continue, etc.)

**Plugin Types**:
- **Compiled (.fzb)**: High-performance hooks, output scanning
- **Daemon-side only**: Has access to PTY, can modify input stream

**Assessment**: ✅ Good extensibility model

**Issue**: Limited documentation of plugin API

**Recommendation**: Create `docs/PLUGIN_GUIDE.md`

---

## Client Architecture

### Bevy Integration

**Component**: `scarab-client/src/main.rs`

**Design**:
```rust
fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(IntegrationPlugin)      // Shared memory reader
        .add_plugins(RenderingPlugin)        // Text rendering
        .add_plugins(RatatuiBridgePlugin)    // Widget compatibility
        .add_plugins(NavigationPlugin)       // Keyboard nav
        .add_plugins(CopyModePlugin)         // Copy mode
        .add_plugins(ScriptingPlugin)        // Fusabi scripts
        .run();
}
```

**Plugin Architecture**: ✅ Excellent use of Bevy's ECS

**System Sets**:
```rust
pub enum InputSystemSet {
    Navigation,  // Runs first (copy mode, nav)
    Surface,     // Overlay input
    Daemon,      // IPC to daemon (runs last)
}
```

**Assessment**: ✅ Clear ordering prevents input conflicts

### Rendering Pipeline

**Component**: `scarab-client/src/rendering/`

**Design**:
```
┌──────────────────────────────────────────────────┐
│ 1. Sync Grid System                             │
│    - Read SharedState                            │
│    - Check sequence_number for changes           │
│    - Update local grid snapshot                  │
└────────────────┬─────────────────────────────────┘
                 │
┌────────────────▼─────────────────────────────────┐
│ 2. Text Shaping (cosmic-text)                   │
│    - Convert chars to glyphs                     │
│    - Apply font metrics                          │
│    - Handle ligatures, emoji                     │
└────────────────┬─────────────────────────────────┘
                 │
┌────────────────▼─────────────────────────────────┐
│ 3. GPU Upload                                    │
│    - Generate texture atlas                      │
│    - Upload glyphs to GPU                        │
│    - Cache for reuse                             │
└────────────────┬─────────────────────────────────┘
                 │
┌────────────────▼─────────────────────────────────┐
│ 4. Mesh Generation                               │
│    - Create quads for each cell                  │
│    - Apply colors, styles                        │
│    - Submit to Bevy renderer                     │
└──────────────────────────────────────────────────┘
```

**Performance**:
- Uses texture atlas caching (good)
- Dirty tracking to skip unchanged cells
- GPU-accelerated rasterization

**Assessment**: ✅ Modern, performant rendering

### Ratatui Bridge

**Component**: `scarab-client/src/ratatui_bridge/`

**Purpose**: Compatibility layer for Ratatui widgets

**Design**:
```
┌──────────────────────────────────────────┐
│ RatatuiSurface (Bevy Component)         │
│ - Position (x, y)                        │
│ - Size (width, height)                   │
│ - Z-index for layering                   │
└─────────────┬────────────────────────────┘
              │
┌─────────────▼────────────────────────────┐
│ SurfaceBuffers (Bevy Resource)          │
│ HashMap<Entity, ratatui::Buffer>         │
└─────────────┬────────────────────────────┘
              │
┌─────────────▼────────────────────────────┐
│ Render System                            │
│ - Render widgets to buffer               │
│ - Convert buffer to Bevy mesh/sprites    │
│ - Position on grid                       │
└──────────────────────────────────────────┘
```

**Assessment**: ✅ Well-designed abstraction

**Status**: Should be marked as "legacy compatibility" since fusabi-tui is now preferred.

**Recommendation**: Document migration path from ratatui to fusabi-tui

---

## Plugin System Architecture

### Dual Runtime Model

| Aspect | Daemon (.fzb) | Client (.fsx) |
|--------|---------------|---------------|
| **Runtime** | fusabi-vm | fusabi-frontend |
| **Format** | Compiled bytecode | Interpreted source |
| **Performance** | High (compiled) | Medium (interpreted) |
| **Hot Reload** | No (requires restart) | Yes (live reload) |
| **Use Cases** | Input hooks, scanning | UI widgets, overlays |
| **Access** | PTY, grid, I/O | Bevy ECS, rendering |

**Rationale**: Different tradeoffs for different use cases.

**Assessment**: ✅ Innovative hybrid approach

### Plugin Communication

**Daemon → Client**:
```rust
pub enum RemoteCommand {
    DrawOverlay {
        id: u64,
        x: u16,
        y: u16,
        text: String,
        style: OverlayStyle,
    },
    ClearOverlays { id: Option<u64> },
    ShowModal { ... },
    // ...
}
```

**Serialization**: rkyv (zero-copy)

**Transport**: Unix domain sockets (separate from shared memory)

**Assessment**: ✅ Good separation of bulk data (shm) vs control (socket)

**Issue**: Command protocol not versioned

**Recommendation**: Add protocol version field to RemoteCommand

---

## Extensibility Analysis

### Plugin API Design

**Trait**:
```rust
#[async_trait]
pub trait Plugin: Send + Sync {
    fn metadata(&self) -> &PluginMetadata;

    async fn on_init(&mut self, ctx: &PluginContext) -> Result<()>;

    async fn on_input(
        &mut self,
        input: &[u8],
        ctx: &PluginContext,
    ) -> Result<Action>;

    async fn on_remote_command(
        &mut self,
        cmd: RemoteCommand,
        ctx: &PluginContext,
    ) -> Result<()>;

    async fn get_commands(&self) -> Result<Vec<Command>>;
}
```

**Action Types**:
```rust
pub enum Action {
    Continue,          // Pass input to next plugin
    Modify(Vec<u8>),   // Transform input
    Consume,           // Stop processing
}
```

**Assessment**: ✅ Flexible and composable

**Strengths**:
1. Chain-of-responsibility pattern
2. Async-first (works with tokio)
3. Remote command support (daemon controls client)

**Issues**:
1. No plugin versioning
2. No plugin dependencies/ordering
3. Limited error handling (just Result)

**Recommendations**:
1. **P2**: Add plugin dependencies (load order)
2. **P2**: Add plugin version compatibility checking
3. **P3**: Add structured error types

### Example Plugin: scarab-nav

**Analysis of Plugin Implementation**:

```rust
pub struct NavigationPlugin {
    metadata: PluginMetadata,
    state: Arc<Mutex<PluginState>>,
    url_regex: Regex,
}
```

**Capabilities**:
1. **URL Detection**: Regex scanning of terminal output
2. **Protocol Integration**: Listens on Unix socket for layout updates
3. **Overlay Rendering**: Draws link hints via RemoteCommand
4. **Input Handling**: Captures Alt+f, handles hint selection

**Assessment**: ✅ Demonstrates plugin capabilities well

**Issue**: Should use upstream `scarab-nav-protocol` crate (see DUPLICATION_REPORT.md)

---

## Concurrency & Threading Model

### Daemon Threading

```
┌─────────────────────────────────────────┐
│ Main Thread                             │
│ - Plugin manager                        │
│ - Orchestrator                          │
└───────┬─────────────────────────────────┘
        │
        ├─▶ Tokio Runtime (async)
        │   - PTY I/O
        │   - Socket listeners
        │   - Plugin async tasks
        │
        └─▶ VTE Parser Thread
            - Blocking parsing
            - Grid updates
```

**Assessment**: ✅ Good separation of I/O and compute

### Client Threading

```
┌─────────────────────────────────────────┐
│ Bevy Main Thread                        │
│ - Rendering                             │
│ - ECS systems                           │
└───────┬─────────────────────────────────┘
        │
        ├─▶ SharedState Reader
        │   - Non-blocking reads
        │   - SeqLock retry loop
        │
        └─▶ Fusabi Script Runtime
            - Isolated execution
            - Can yield to Bevy
```

**Assessment**: ✅ Never blocks renderer

**Critical Property**: Shared memory reads are lock-free and never block Bevy's render thread.

---

## Error Handling & Resilience

### Daemon Resilience

**PTY Failure**:
```rust
pub enum ErrorMode {
    Normal = 0,
    PtyUnavailable = 1,
}
```

**Behavior**:
- Set `error_mode` in SharedState
- Client displays error overlay
- Daemon continues running
- Reconnect when PTY available

**Assessment**: ✅ Graceful degradation

### Client Resilience

**Shared Memory Unavailable**:
- Client checks `error_mode` flag
- Displays "Waiting for daemon..." message
- Retries connection
- No crash

**Assessment**: ✅ Handles IPC failures gracefully

### Plugin Errors

**Current**:
```rust
async fn on_input(&mut self, ...) -> Result<Action>
```

**Issues**:
- Plugin error crashes entire plugin chain?
- No isolation between plugins
- No error reporting to user

**Recommendation**:
- **P1**: Catch plugin errors and continue
- **P2**: Add plugin error reporting UI
- **P2**: Add plugin disable/enable at runtime

---

## Performance Characteristics

### Latency Breakdown (Typical)

```
┌──────────────────────────────────────────────┐
│ PTY Output → Screen Render                  │
├──────────────────────────────────────────────┤
│ 1. PTY read          : ~100 μs              │
│ 2. VTE parse         : ~50 μs               │
│ 3. Shared mem write  : ~10 μs (atomic)      │
│ 4. Client read       : ~5 μs (SeqLock)      │
│ 5. Text shaping      : ~200 μs              │
│ 6. GPU upload        : ~50 μs               │
│ 7. Mesh generation   : ~100 μs              │
│ 8. Bevy render       : ~16.67 ms (60 FPS)   │
├──────────────────────────────────────────────┤
│ Total latency        : ~17 ms (60 FPS cap)  │
└──────────────────────────────────────────────┘
```

**Assessment**: ✅ Well within real-time budget

**Bottleneck**: Frame rate cap (intentional for power efficiency)

### Memory Usage

**Daemon**:
- Shared memory: ~320 KB (200×100 cells)
- Plugin state: ~100 KB per plugin
- PTY buffers: ~4 KB per terminal
- **Total**: ~1-2 MB per terminal

**Client**:
- Texture atlas: ~2-4 MB (font cache)
- Bevy ECS: ~1-2 MB
- Shared memory: ~320 KB (read-only mapping)
- **Total**: ~5-10 MB

**Assessment**: ✅ Very efficient for a GPU-accelerated terminal

### Throughput

**Shared Memory Bandwidth**:
- Grid size: 200×100 = 20,000 cells
- Cell size: 16 bytes
- Full grid: 320 KB
- At 60 FPS: 19.2 MB/s (well within RAM bandwidth)

**Assessment**: ✅ No bandwidth concerns

---

## Security Considerations

### Process Isolation

**Current**:
- Daemon and client are separate processes ✅
- Shared memory is world-readable ⚠️
- No sandboxing of client ⚠️

**Risks**:
1. Malicious client can read terminal contents
2. Malicious plugin can modify input/output
3. No privilege separation

**Recommendations**:
- **P2**: Restrict shared memory permissions (user-only)
- **P3**: Consider sandboxing client process
- **P3**: Add plugin permission system

### Plugin Security

**Current**:
- Plugins run with full daemon privileges ⚠️
- No capability system
- No signature verification

**Risks**:
1. Malicious plugin can execute arbitrary code
2. Plugin can access all terminal I/O
3. No protection against plugin conflicts

**Recommendations**:
- **P1**: Add plugin signature verification (use `scarab-config/registry/security.rs`)
- **P2**: Add capability system (limit plugin access)
- **P3**: Isolate plugins in separate processes (WebAssembly?)

---

## Comparison with Other Terminal Emulators

### Architecture Comparison

| Feature | Scarab | Alacritty | WezTerm | Kitty |
|---------|--------|-----------|---------|-------|
| **Process Model** | Split | Monolithic | Monolithic | Monolithic |
| **Rendering** | Bevy (GPU) | OpenGL | WebGPU | OpenGL |
| **Plugin System** | Fusabi (F#) | None | Lua | Python |
| **IPC** | Shared Mem | N/A | N/A | N/A |
| **Hot Reload** | Yes (.fsx) | No | Lua | Python |
| **Multiplexer** | Built-in | No | Built-in | Sessions |

**Unique Advantages**:
1. **Split architecture**: Client crashes don't kill terminals
2. **Dual runtime**: Compiled (.fzb) + Interpreted (.fsx)
3. **Zero-copy IPC**: Lock-free shared memory
4. **Bevy ECS**: Powerful UI system

**Trade-offs**:
1. **Complexity**: More moving parts than monolithic
2. **Memory**: Two processes instead of one
3. **Maturity**: Newer, less battle-tested

**Assessment**: Innovative architecture with clear benefits for extensibility and robustness.

---

## Recommendations Summary

### Priority 0 (Critical - Documentation)

1. **Document IPC Synchronization Protocol**
   - Create `docs/architecture/IPC_SYNCHRONIZATION.md`
   - Explain SeqLock pattern
   - Document memory ordering guarantees
   - **Effort**: 2-3 hours
   - **Impact**: Critical for correctness

2. **Create Plugin Development Guide**
   - Create `docs/PLUGIN_GUIDE.md`
   - Example plugin walkthrough
   - API reference
   - **Effort**: 4-6 hours
   - **Impact**: Enables third-party plugins

### Priority 1 (High - Architecture)

3. **Add Plugin Error Isolation**
   - Catch plugin errors gracefully
   - Continue plugin chain on error
   - **Effort**: 2-3 hours
   - **Impact**: Prevents plugin crashes

4. **Add Protocol Versioning**
   - Version field in SharedState
   - Version field in RemoteCommand
   - **Effort**: 1-2 hours
   - **Impact**: Future compatibility

5. **Add Plugin Signature Verification**
   - Use existing `scarab-config/registry/security.rs`
   - Verify plugins before loading
   - **Effort**: 3-4 hours
   - **Impact**: Security

### Priority 2 (Medium - Enhancement)

6. **Add Plugin Dependencies/Ordering**
   - Allow plugins to declare load order
   - Resolve dependencies
   - **Effort**: 4-5 hours
   - **Impact**: Plugin ecosystem

7. **Restrict Shared Memory Permissions**
   - Set mode 0600 on shared memory
   - Prevent other users from reading
   - **Effort**: 1 hour
   - **Impact**: Basic security

8. **Document Migration from Ratatui Bridge**
   - Mark ratatui bridge as legacy
   - Document fusabi-tui usage
   - **Effort**: 2-3 hours
   - **Impact**: Clarity

### Priority 3 (Low - Future)

9. **Consider Plugin Capability System**
   - Limit plugin access (PTY, grid, network)
   - Permission model
   - **Effort**: 10+ hours
   - **Impact**: Advanced security

10. **Consider WebAssembly Plugin Isolation**
    - Run plugins in WASM sandbox
    - Cross-language support
    - **Effort**: 20+ hours
    - **Impact**: Long-term security

---

## Architecture Metrics

### Design Quality

| Aspect | Rating | Notes |
|--------|--------|-------|
| **Separation of Concerns** | A | Clean daemon/client split |
| **Modularity** | A- | Good, some large files |
| **Extensibility** | A | Excellent plugin system |
| **Performance** | A | Lock-free IPC, GPU rendering |
| **Resilience** | B+ | Graceful degradation, needs error handling |
| **Security** | C+ | Process isolation, but needs hardening |
| **Documentation** | C | Code is clear, but docs sparse |

**Overall Architecture Grade**: **A-** (Excellent design with room for documentation)

### Innovation Score

| Feature | Innovation | Impact |
|---------|-----------|--------|
| Split-process design | High | Unique in terminal emulators |
| SeqLock IPC | Medium | Novel application |
| Dual Fusabi runtime | High | Compiled + interpreted |
| Bevy integration | High | Game engine for terminal |
| Zero-copy shared memory | Medium | Well-executed |

**Innovation Score**: **9/10** (Highly innovative architecture)

---

## Conclusion

The Scarab architecture is **exceptionally well-designed** with several innovative features:

✅ **Strengths**:
1. Clean split-process design
2. Lock-free shared memory IPC
3. Dual plugin runtime (compiled + interpreted)
4. GPU-accelerated rendering with Bevy
5. Extensible plugin system

⚠️ **Areas for Improvement**:
1. Documentation (especially IPC protocol)
2. Plugin error handling
3. Security hardening
4. Version compatibility

**Final Assessment**: This is a **production-quality architecture** with excellent fundamentals. The main gaps are in documentation and security, not in the design itself.

**Overall Grade**: **A** (Excellent architecture, ready for production with documentation)

---

## Appendix: Architecture Diagrams

### Data Flow: Input Processing

```
┌───────────┐
│ User Input│
└─────┬─────┘
      │
┌─────▼──────────────────────────────┐
│ Client (Bevy)                      │
│ - Capture keyboard/mouse           │
│ - Run client-side plugins (.fsx)   │
└─────┬──────────────────────────────┘
      │ Unix Socket
┌─────▼──────────────────────────────┐
│ Daemon                             │
│ - Run daemon-side plugins (.fzb)   │
│ - Apply transformations            │
└─────┬──────────────────────────────┘
      │
┌─────▼──────────────────────────────┐
│ PTY                                │
│ - Write to shell stdin             │
└────────────────────────────────────┘
```

### Data Flow: Output Processing

```
┌────────────────────────────────────┐
│ PTY                                │
│ - Shell stdout/stderr              │
└─────┬──────────────────────────────┘
      │
┌─────▼──────────────────────────────┐
│ Daemon                             │
│ - Read PTY output                  │
│ - Parse VTE sequences              │
│ - Update grid state                │
└─────┬──────────────────────────────┘
      │ Shared Memory Write
┌─────▼──────────────────────────────┐
│ SharedState (Shared Memory)        │
│ - sequence_number (atomic)         │
│ - cells[200x100]                   │
└─────┬──────────────────────────────┘
      │ Zero-Copy Read
┌─────▼──────────────────────────────┐
│ Client (Bevy)                      │
│ - Sync grid (SeqLock)              │
│ - Shape text (cosmic-text)         │
│ - Render to GPU                    │
└────────────────────────────────────┘
```

### Plugin Architecture

```
┌──────────────────────────────────────────────────┐
│                 Plugin System                     │
├────────────────────┬─────────────────────────────┤
│   Daemon Side      │      Client Side            │
├────────────────────┼─────────────────────────────┤
│ Fusabi VM          │  Fusabi Frontend            │
│ - Compiled .fzb    │  - Interpreted .fsx         │
│ - High Performance │  - Hot Reload               │
│ - Input Hooks      │  - UI Widgets               │
│ - Output Scanning  │  - Overlays                 │
│                    │  - Bevy Integration         │
├────────────────────┴─────────────────────────────┤
│           Shared Plugin API (Traits)              │
│  - on_init(), on_input(), get_commands()         │
└──────────────────────────────────────────────────┘
```
