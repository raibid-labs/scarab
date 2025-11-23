# Scarab 🪲

> **Next-Gen Rust Terminal Emulator** | Built for *raibid-labs*

Scarab is a high-performance, split-process terminal emulator built in Rust. It leverages the **Bevy** game engine for a rich, highly configurable UI and introduces a hybrid plugin system using **[Fusabi](https://github.com/fusabi-lang/fusabi)** - an official F# dialect scripting language for Rust.

## Current Status

**Phase**: 5 (Integration & Polish) - 🔄 In Progress
**Completion**: ~75% of MVP features complete
**Last Updated**: 2025-11-23

### Feature Status
- ✅ **VTE Parser**: ANSI escape sequence parsing, scrollback, colors
- ✅ **GPU Rendering**: cosmic-text with Bevy, 60 FPS @ 200x100 cells
- ✅ **IPC**: Zero-copy shared memory, lock-free synchronization
- ✅ **Plugin System**: Architecture complete (awaiting Fusabi runtime integration)
- 🟡 **Advanced UI**: 90% complete (Bevy 0.15 migration in progress)
- ✅ **Session Management**: SQLite persistence, multi-session support
- ✅ **Configuration**: TOML-based with hot-reload

### Recent Milestones (2025-11-23)

**All GitHub Issues Resolved!** 🎉
- ✅ Issue #1: Resolved SharedState struct conflicts
- ✅ Issue #2: Integrated UI features with SharedMemoryReader
- ✅ Issue #3: Cleaned up dead code (~200 lines removed)
- ✅ Issue #4: Implemented plugin loading system (600+ lines)
- ✅ Issue #5: Created comprehensive ROADMAP.md

**Documentation**
- 📘 [ROADMAP.md](./ROADMAP.md) - Strategic development roadmap (Phases 1-10)
- 📘 [IMPLEMENTATION_SUMMARY.md](./IMPLEMENTATION_SUMMARY.md) - Technical implementation details
- 📘 [MIGRATION_GUIDE.md](./MIGRATION_GUIDE.md) - Bevy 0.15 migration guide for contributors

## Recent Updates

**2025-11-23**: Major progress on integration and documentation
- Resolved all 5 GitHub issues (#1-5)
- Implemented plugin loading infrastructure (Issue #4)
- Integrated UI features with terminal state (Issue #2)
- Created comprehensive strategic roadmap
- Entered Phase 5: Integration & Polish

**2025-11-22**: Migrated to official Fusabi implementation
- Replaced custom `fusabi-vm` and `fusabi-interpreter` implementations with dependencies on the official Fusabi repository
- Now using `fusabi-vm` for bytecode execution and `fusabi-frontend` for F# script compilation
- Removed ~6,000 lines of duplicate implementation code

## Architecture at a Glance

*   **`scarab-daemon`**: Headless server. Manages PTYs and session state. Runs compiled `.fzb` plugins via official `fusabi-vm`.
*   **`scarab-client`**: Bevy-based GUI. Renders via Shared Memory. Runs interpreted `.fsx` scripts via official `fusabi-frontend` for hot-reloadable UI.
*   **`scarab-protocol`**: Defines the Zero-Copy shared memory layout.
*   **`scarab-platform`**: Platform-specific abstractions for Linux, macOS, and Windows.

## Quick Start

### Prerequisites
- Rust 1.75+ (with `cargo`)
- Linux (X11 or Wayland) - macOS and Windows support planned

### Running Scarab

1.  **Build the entire workspace**:
    ```bash
    cargo build --release
    ```

2.  **Run the Daemon** (headless server):
    ```bash
    cargo run --release -p scarab-daemon
    ```
    The daemon will:
    - Create shared memory at `/scarab_shm_v1`
    - Start IPC socket at `/tmp/scarab.sock`
    - Initialize session manager

3.  **Run the Client** (in a separate terminal):
    ```bash
    cargo run --release -p scarab-client
    ```
    The client will:
    - Connect to daemon via IPC
    - Map shared memory for zero-copy rendering
    - Open Bevy window with terminal UI

### Development Mode

For faster iteration during development:
```bash
# Terminal 1: Daemon
cargo run -p scarab-daemon

# Terminal 2: Client
cargo run -p scarab-client
```

### Testing

Run all tests:
```bash
cargo test --workspace
```

Run specific test suites:
```bash
# UI feature tests
cargo test -p scarab-client ui_tests

# Protocol tests
cargo test -p scarab-protocol
```

### Documentation

- **Architecture Overview**: See [CLAUDE.md](./CLAUDE.md)
- **Strategic Roadmap**: See [ROADMAP.md](./ROADMAP.md)
- **Implementation Details**: See [IMPLEMENTATION_SUMMARY.md](./IMPLEMENTATION_SUMMARY.md)
- **Bevy 0.15 Migration**: See [MIGRATION_GUIDE.md](./MIGRATION_GUIDE.md)
