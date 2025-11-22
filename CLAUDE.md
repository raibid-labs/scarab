# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Scarab is a high-performance, split-process terminal emulator built in Rust. It features:
- **Split Architecture**: Daemon (headless server) + Client (Bevy-based GUI)
- **Zero-Copy IPC**: Shared memory for bulk data transfer between processes
- **Hybrid Plugin System**: Fusabi-lang (F# dialect) with dual runtimes
- **GPU-Accelerated Rendering**: Bevy game engine with cosmic-text

## Workspace Structure

This is a Cargo workspace with 5 crates:

```
scarab/
├── crates/
│   ├── scarab-daemon/      # Headless server, owns PTY processes
│   ├── scarab-client/      # Bevy GUI, renders via shared memory
│   ├── scarab-protocol/    # IPC definitions, shared memory layout (#[repr(C)])
│   ├── scarab-plugin-api/  # Shared plugin traits
│   └── scarab-config/      # Configuration management
```

**External Dependencies:**
- `fusabi-vm` - Official Fusabi VM runtime for .fzb bytecode (from https://github.com/fusabi-lang/fusabi)
- `fusabi-frontend` - Official Fusabi compiler/parser for .fsx scripts (from https://github.com/fusabi-lang/fusabi)

## Build Commands

**Build the entire workspace:**
```bash
cargo build
```

**Build with optimizations:**
```bash
cargo build --release
```

**Build specific crate:**
```bash
cargo build -p scarab-daemon
cargo build -p scarab-client
```

**Run daemon (headless server):**
```bash
cargo run -p scarab-daemon
```

**Run client (in separate terminal):**
```bash
cargo run -p scarab-client
```

**Check all crates:**
```bash
cargo check --workspace
```

**Run tests:**
```bash
cargo test --workspace
```

## Architecture Constraints

### scarab-protocol (IPC Layer)
- **CRITICAL**: All shared memory structs MUST be `#[repr(C)]`
- **CRITICAL**: Must be `#![no_std]` compatible for memory layout guarantees
- Uses `bytemuck::{Pod, Zeroable}` for safe zero-copy transmutation
- Shared memory path: `/scarab_shm_v1`
- Default grid: 200x100 cells

### scarab-daemon (Server)
- Owns PTY processes via `portable-pty`
- Maintains terminal grid state (source of truth)
- Writes to shared memory ring buffer
- Uses `alacritty_terminal` for VTE parsing
- Runs compiled `.fzb` plugins (Fusabi VM)
- **Persistence**: Survives client crashes/disconnects

### scarab-client (Bevy GUI)
- Reads from shared memory (zero-copy)
- Renders text using `cosmic-text` with texture atlas caching
- Uses `AtomicU64` sequence numbers for lock-free synchronization
- Runs interpreted `.fsx` scripts (hot-reloadable UI)
- Sends input to daemon via Unix Domain Sockets

### Synchronization Strategy
- **Lock-free**: Use `AtomicU64` sequence numbers in SharedState
- **Never block the Bevy render thread** - client reads are non-blocking
- Daemon increments sequence_number after each grid update
- Client polls sequence_number to detect changes

## Plugin System (Fusabi)

Scarab uses the official **Fusabi** scripting language (https://github.com/fusabi-lang/fusabi) - a high-performance F# dialect for Rust.

Two runtimes for different use cases:

| Runtime | File Type | Location | Crate | Purpose |
|---------|-----------|----------|-------|---------|
| Fusabi VM | `.fzb` | Daemon | `fusabi-vm` | Compiled bytecode for high-perf hooks, output scanning, mux logic |
| Fusabi Frontend | `.fsx` | Client | `fusabi-frontend` | F# source parser/compiler for hot-reloadable UI scripts |

**Hot Reloading**: `.fsx` files can be recompiled on-the-fly without Rust recompilation

## Key Dependencies

- **Fusabi** - Official F# scripting language for Rust
  - `fusabi-vm` - Bytecode VM runtime
  - `fusabi-frontend` - Parser, type checker, compiler
- `bevy` 0.15 - Game engine (minimal features: winit, ui, render, x11, wayland)
- `portable-pty` 0.8 - Cross-platform PTY handling
- `alacritty_terminal` 0.24 - VTE parser
- `cosmic-text` 0.11 - Text rendering
- `shared_memory` 0.12 - IPC shared memory
- `bytemuck` 1.14 - Safe zero-copy casting
- `tokio` 1.36 - Async runtime

## Development Roadmap

Current phase: **Scaffolding** (IPC and PTY bridging)

Next phases:
1. Implement Shared Memory IPC Bridge
2. Integrate `cosmic-text` rendering in Bevy
3. Integrate Fusabi VM for daemon plugins
4. Integrate Fusabi Frontend for client UI scripts

## Critical TODOs in Code

**scarab-daemon/src/main.rs:**
- Initialize Shared Memory writer
- Feed PTY output to Alacritty VTE parser
- Update SharedState grid from parsed terminal commands
- Increment sequence_number atomically

**scarab-client/src/main.rs:**
- Initialize Shared Memory reader
- Check SharedState sequence_number in sync_grid()
- Update texture/mesh when sequence changes
- Send input to daemon via socket

**scarab-protocol/src/lib.rs:**
- Fix incomplete `cells` field definition in SharedState (line 33)
- Fix incomplete ControlMessage enum attribute (line 37)

## Release Profile

Optimized for performance:
- LTO: thin
- Codegen units: 1
- Opt level: 3
