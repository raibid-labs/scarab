# Scarab 🪲

> **Next-Gen Rust Terminal Emulator** | Built for *raibid-labs*

Scarab is a high-performance, split-process terminal emulator built in Rust. It leverages the **Bevy** game engine for a rich, highly configurable UI and introduces a hybrid plugin system using **Fusabi-lang** (an F# dialect).

## Architecture at a Glance

*   **`scarab-daemon`**: Headless server. Manages PTYs and session state. Runs compiled `.fzb` plugins.
*   **`scarab-client`**: Bevy-based GUI. Renders via Shared Memory. Runs interpreted `.fsx` scripts for hot-reloadable UI.
*   **`scarab-protocol`**: Defines the Zero-Copy shared memory layout.

## Getting Started

1.  **Run the Daemon**:
    ```bash
    cargo run -p scarab-daemon
    ```
2.  **Run the Client** (in a separate terminal):
    ```bash
    cargo run -p scarab-client
    ```
