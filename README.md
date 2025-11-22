# Scarab 🪲

> **Next-Gen Rust Terminal Emulator** | Built for *raibid-labs*

Scarab is a high-performance, split-process terminal emulator built in Rust. It leverages the **Bevy** game engine for a rich, highly configurable UI and introduces a hybrid plugin system using **[Fusabi](https://github.com/fusabi-lang/fusabi)** - an official F# dialect scripting language for Rust.

## Recent Updates

**2025-11-22**: Migrated to official Fusabi implementation
- Replaced custom `fusabi-vm` and `fusabi-interpreter` implementations with dependencies on the official Fusabi repository
- Now using `fusabi-vm` for bytecode execution and `fusabi-frontend` for F# script compilation
- Removed ~6,000 lines of duplicate implementation code

## Architecture at a Glance

*   **`scarab-daemon`**: Headless server. Manages PTYs and session state. Runs compiled `.fzb` plugins via official `fusabi-vm`.
*   **`scarab-client`**: Bevy-based GUI. Renders via Shared Memory. Runs interpreted `.fsx` scripts via official `fusabi-frontend` for hot-reloadable UI.
*   **`scarab-protocol`**: Defines the Zero-Copy shared memory layout.
*   **`scarab-platform`**: Platform-specific abstractions for Linux, macOS, and Windows.

## Getting Started

1.  **Run the Daemon**:
    ```bash
    cargo run -p scarab-daemon
    ```
2.  **Run the Client** (in a separate terminal):
    ```bash
    cargo run -p scarab-client
    ```
