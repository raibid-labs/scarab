# Scarab Architecture

This document provides a comprehensive overview of Scarab's architecture, design decisions, and component interactions.

## Table of Contents

- [System Overview](#system-overview)
- [Core Components](#core-components)
- [Data Flow](#data-flow)
- [Design Decisions](#design-decisions)
- [Performance Considerations](#performance-considerations)

## System Overview

Scarab is built with a modular, multi-process architecture designed for performance and extensibility:

```
┌─────────────────────────────────────────────────────────────┐
│                     Scarab Client (Bevy)                     │
│  ┌────────────┐  ┌──────────┐  ┌─────────┐  ┌────────────┐ │
│  │  Rendering │  │    UI    │  │  Input  │  │ IPC Client │ │
│  │  (GPU/CPU) │  │ (egui)   │  │ Handler │  │  (rkyv)    │ │
│  └────────────┘  └──────────┘  └─────────┘  └────────────┘ │
└───────────────────────────┬─────────────────────────────────┘
                            │ Shared Memory + IPC
┌───────────────────────────┴─────────────────────────────────┐
│                      Scarab Daemon                           │
│  ┌────────────┐  ┌──────────┐  ┌─────────┐  ┌────────────┐ │
│  │  Session   │  │   PTY    │  │  IPC    │  │   Plugin   │ │
│  │  Manager   │  │  Master  │  │ Server  │  │   System   │ │
│  └────────────┘  └──────────┘  └─────────┘  └────────────┘ │
│  ┌────────────┐  ┌──────────┐                               │
│  │   VTE      │  │  Config  │                               │
│  │  Parser    │  │  Manager │                               │
│  └────────────┘  └──────────┘                               │
└─────────────────────────────────────────────────────────────┘
                            │
                    ┌───────┴────────┐
                    │  PTY Processes │
                    │ (bash/zsh/etc) │
                    └────────────────┘
```

## Core Components

### 1. Client (`scarab-client`)

The client handles all UI rendering and user interaction using the Bevy game engine.

**Responsibilities:**
- GPU-accelerated text rendering
- User input handling
- UI components (tabs, command palette, etc.)
- IPC communication with daemon

**Key Files:**
- `src/rendering/` - Text and UI rendering
- `src/input/` - Keyboard and mouse input
- `src/ui/` - UI components
- `src/ipc_client.rs` - IPC communication

**Technology:**
- **Bevy**: Game engine for rendering
- **cosmic-text**: Advanced text shaping and rendering
- **rkyv**: Zero-copy IPC serialization

### 2. Daemon (`scarab-daemon`)

The daemon manages terminal sessions and PTY processes.

**Responsibilities:**
- Session lifecycle management
- PTY process spawning and management
- VTE parsing and processing
- IPC server
- Plugin execution

**Key Files:**
- `src/session/` - Session management
- `src/pty/` - PTY handling
- `src/vte.rs` - Terminal escape sequence parsing
- `src/ipc_server.rs` - IPC server
- `src/plugin_manager.rs` - Plugin system

**Technology:**
- **portable-pty**: Cross-platform PTY handling
- **vte**: VT100/ANSI escape sequence parser
- **tokio**: Async runtime
- **rkyv**: Serialization

### 3. Protocol (`scarab-protocol`)

Defines the IPC protocol between client and daemon using zero-copy serialization.

**Messages:**
- `CreateSession`
- `AttachSession`
- `DetachSession`
- `SendInput`
- `ReceiveOutput`
- `ResizeTerminal`

**Technology:**
- **rkyv**: Zero-copy serialization
- **shared_memory**: Cross-process memory sharing

### 4. Configuration (`scarab-config`)

Manages TOML-based configuration with hot-reload support.

**Features:**
- Default configuration generation
- Hierarchical config loading (system → user → project)
- File watching for hot-reload (<100ms)
- Validation with helpful errors

**Configuration Hierarchy:**
1. System defaults
2. User config (`~/.config/scarab/config.toml`)
3. Project config (`.scarab.toml`)
4. Command-line arguments

### 5. Plugin API (`scarab-plugin-api`)

Provides the API for building third-party plugins.

**Hook Types:**
- `on_init` - Plugin initialization
- `on_input` - Process input before PTY
- `on_output` - Process output before rendering
- `on_resize` - Handle terminal resize

**Plugin Context:**
- Session information
- Terminal state
- Configuration access

### 6. FusABI VM (`fusabi-vm`, `fusabi-interpreter`)

A custom bytecode VM for secure, sandboxed plugin execution.

**Features:**
- Memory-safe execution
- Capability-based security
- Hot-reload support
- Performance profiling

## Data Flow

### Input Flow

```
User Input (Keyboard/Mouse)
    ↓
Client Input Handler
    ↓
Plugin Hooks (on_input)
    ↓
IPC Client → IPC Server
    ↓
Session Manager
    ↓
PTY Master
    ↓
Shell Process
```

### Output Flow

```
Shell Process
    ↓
PTY Master
    ↓
VTE Parser
    ↓
Session Buffer
    ↓
Plugin Hooks (on_output)
    ↓
IPC Server → IPC Client
    ↓
Shared Memory
    ↓
Renderer
    ↓
Display
```

### Configuration Flow

```
Config File Change
    ↓
File Watcher (notify)
    ↓
Config Loader
    ↓
Validation
    ↓
Merge with Defaults
    ↓
Apply to Components
    ↓
Trigger Re-render
```

## Design Decisions

### Why Multi-Process?

**Advantages:**
- **Stability**: Client crashes don't kill sessions
- **Security**: Daemon can run with lower privileges
- **Performance**: Isolate rendering from terminal processing
- **Flexibility**: Swap clients (TUI, GUI, Web)

**Trade-offs:**
- More complex IPC
- Additional memory overhead

### Why Bevy for Rendering?

**Advantages:**
- **GPU Acceleration**: Hardware-accelerated rendering
- **ECS Architecture**: Efficient data-oriented design
- **Plugin System**: Extends naturally
- **Performance**: 60+ FPS rendering

**Trade-offs:**
- Larger binary size
- Steeper learning curve

### Why rkyv for IPC?

**Advantages:**
- **Zero-copy**: Minimal serialization overhead
- **Type-safe**: Compile-time guarantees
- **Fast**: <1μs for typical messages
- **Validated**: Built-in validation

**Trade-offs:**
- Strict versioning required
- More complex than JSON

### Why FusABI?

**Advantages:**
- **Security**: Sandboxed execution
- **Hot-reload**: Update plugins without restart
- **Performance**: Competitive with native
- **Portable**: Cross-platform

**Trade-offs:**
- Custom tooling required
- Learning curve for plugin developers

## Performance Considerations

### Rendering Performance

**Target**: 60 FPS rendering with <16ms frame time

**Optimizations:**
- GPU-accelerated glyph rendering
- Dirty region tracking
- Glyph caching
- Atlas packing for textures

**Benchmarks:**
- Initial render: <50ms
- Frame update: <5ms
- Input latency: <10ms

### IPC Performance

**Target**: <1ms round-trip for typical messages

**Optimizations:**
- Zero-copy serialization (rkyv)
- Shared memory for large buffers
- Lock-free queues for events
- Batched message processing

**Benchmarks:**
- CreateSession: ~5ms
- SendInput: <0.1ms
- ReceiveOutput: <0.5ms

### VTE Parsing

**Target**: >100MB/s throughput

**Optimizations:**
- State machine parser (vte crate)
- SIMD for ASCII detection
- Buffer pooling
- Incremental parsing

**Benchmarks:**
- ASCII throughput: ~500MB/s
- UTF-8 throughput: ~200MB/s
- Escape sequences: ~100MB/s

### Memory Usage

**Target**: <100MB for typical usage

**Breakdown:**
- Client: ~50MB
- Daemon: ~30MB
- Per-session: ~5MB

**Optimizations:**
- Scrollback buffer limits
- Shared texture atlases
- Arc for shared strings
- Lazy loading for plugins

## Build System

### Workspace Structure

```
scarab/
├── crates/
│   ├── scarab-client/      # GUI client
│   ├── scarab-daemon/      # Session daemon
│   ├── scarab-protocol/    # IPC protocol
│   ├── scarab-config/      # Config system
│   ├── scarab-plugin-api/  # Plugin API
│   ├── fusabi-vm/          # VM runtime
│   └── fusabi-interpreter/ # VM bytecode
├── tests/                  # Integration tests
├── benches/                # Benchmarks
└── docs/                   # Documentation
```

### Build Profiles

- **dev**: Fast compile, debug symbols
- **release**: Optimized, no debug
- **release-with-debug**: Optimized + debug symbols
- **profiling**: For performance analysis

### Feature Flags

- `gpu-rendering`: Enable GPU acceleration (default)
- `plugin-system`: Enable plugin support (default)
- `profiling`: Tracy/puffin profiling support
- `tracy`: Tracy profiler integration
- `puffin`: Puffin profiler integration

## Testing Strategy

### Unit Tests

- Per-module tests in `src/*/tests.rs`
- Mock external dependencies
- Target: >80% coverage

### Integration Tests

- End-to-end workflows in `tests/integration/`
- Real PTY processes
- IPC communication

### E2E Tests

- Real program interactions in `tests/e2e/`
- vim, htop, git, etc.
- Visual regression tests

### Benchmarks

- Criterion benchmarks in `benches/`
- Performance regression tests
- CI tracking

## Contributing

See [CONTRIBUTING.md](../CONTRIBUTING.md) for:
- Code style guidelines
- PR process
- Testing requirements
- Documentation standards

## References

- [Bevy Engine](https://bevyengine.org/)
- [rkyv Serialization](https://rkyv.org/)
- [VTE Parser](https://docs.rs/vte/)
- [portable-pty](https://docs.rs/portable-pty/)
