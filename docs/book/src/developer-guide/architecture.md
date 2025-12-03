# Architecture

Scarab uses a split-process architecture for robustness and performance.

## High-Level Overview

```
┌──────────────────┐         ┌──────────────────┐
│  scarab-daemon   │◄────────┤  scarab-client   │
│  (Headless)      │  IPC    │  (Bevy GUI)      │
│                  │────────►│                  │
│  - PTY processes │         │  - GPU rendering │
│  - State keeper  │         │  - User input    │
│  - Plugins (.fzb)│         │  - Scripts (.fsx)│
└──────────────────┘         └──────────────────┘
```

## Process Separation

### scarab-daemon (Server)
- Owns PTY processes via `portable-pty`
- Maintains terminal grid state (source of truth)
- Writes to shared memory ring buffer
- Uses `alacritty_terminal` for VTE parsing
- Runs compiled `.fzb` plugins (Fusabi VM)
- **Persists** across client crashes/disconnects

### scarab-client (GUI)
- Reads from shared memory (zero-copy)
- Renders text using `cosmic-text` with texture atlas caching
- Uses Bevy ECS for navigation and UI state
- Runs interpreted `.fsx` scripts (hot-reloadable UI)
- Sends input to daemon via Unix Domain Sockets

## IPC Layer (scarab-protocol)

The protocol crate defines shared memory layout:

- **CRITICAL**: All structs are `#[repr(C)]` for ABI stability
- Uses `bytemuck::{Pod, Zeroable}` for safe zero-copy transmutation
- Lock-free synchronization via `AtomicU64` sequence numbers
- Shared memory path: `/scarab_shm_v1`

For detailed IPC protocol documentation, see the [IPC Protocol Reference](../reference/ipc-protocol.md).

## Workspace Structure

This is a Cargo workspace with 5 crates:

```
scarab/
├── crates/
│   ├── scarab-daemon/      # Headless server
│   ├── scarab-client/      # Bevy GUI
│   ├── scarab-protocol/    # IPC definitions
│   ├── scarab-plugin-api/  # Plugin traits
│   └── scarab-config/      # Configuration
```

For complete project structure, see [CLAUDE.md](../../../CLAUDE.md).

## Key Design Decisions

1. **Split Process**: Daemon survives client crashes
2. **Zero-Copy IPC**: Shared memory for high-throughput data
3. **Lock-Free Sync**: Atomic sequence numbers prevent blocking
4. **ECS Architecture**: Native Bevy integration for navigation
5. **Dual Plugin Runtimes**: Compiled (performance) + Interpreted (flexibility)

## Next Steps

- [Navigation System](./navigation.md)
- [Plugin Development](./plugins.md)
- [Testing Guide](./testing.md)
