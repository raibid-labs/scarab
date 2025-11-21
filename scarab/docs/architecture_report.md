# Architectural Blueprint: Scarab Terminal

    ## 1. Executive Summary
    Scarab is a distributed, GPU-accelerated terminal emulator designed for the **raibid-labs** ecosystem. It differentiates itself through a split-process architecture (Daemon/Client), a hybrid plugin runtime using **Fusabi-lang** (a custom F# dialect), and a game-engine-driven frontend (Bevy).

    ## 2. Core Architecture: The Split Model

    ### 2.1 The Daemon (`scarab-daemon`)
    *   **Role**: The "Source of Truth". Runs headless.
    *   **Responsibilities**:
        *   Owns the PTY (Pseudo-Terminal) processes via `portable-pty`.
        *   Maintains the grid state (Virtual Terminal Emulator).
        *   Hosts the **Fusabi VM** for high-performance, AOT-compiled plugins (`.fzb`).
    *   **Persistence**: If the client window closes or crashes, the daemon (and the shell session) survives.

    ### 2.2 The Client (`scarab-client`)
    *   **Role**: The "View". Runs the UI.
    *   **Tech Stack**: Rust + **Bevy Game Engine**.
    *   **Responsibilities**:
        *   Reads grid state from Shared Memory (Zero-Copy).
        *   Renders text using `cosmic-text` (cached in a texture atlas).
        *   Hosts the **Fusabi Interpreter** for hot-reloadable UI scripts (`.fsx`).
    *   **Input Handling**: Captures raw input and sends it to the Daemon via IPC.

    ### 2.3 The Bridge (`scarab-protocol`)
    *   **Bulk Data**: Shared Memory Ring Buffer. The Daemon writes cell updates; the Client reads them to update the GPU mesh.
    *   **Control Events**: Unix Domain Sockets (or Named Pipes on Windows) for events like `Resize`, `KeyDown`, `Paste`.

    ## 3. The Fusabi Hybrid Runtime
    To balance performance with developer experience (DX), Scarab uses two runtimes for Fusabi:

| Component | File Type | Runtime | Use Case |
| :--- | :--- | :--- | :--- |
| **Daemon** | `.fzb` (Binary) | **Fusabi VM** | High-perf hooks, triggers, output scanning (Regex), Mux logic. |
| **Client** | `.fsx` (Script) | **Interpreter** | UI layout, Animations, Vimium-style overlays, "Spacemacs" menus. |

    *   **Hot Reloading**: The Client watches `.fsx` files. On change, it re-parses the AST and updates the Bevy UI tree immediately without recompiling the Rust binary.

    ## 4. Roadmap
    1.  **Scaffolding**: Setup IPC and PTY bridging (Current Step).
    2.  **Rendering**: Implement `cosmic-text` integration in Bevy.
    3.  **Fusabi VM**: Implement the Bytecode serialization (`rkyv`) for the Daemon.
    4.  **Fusabi Interpreter**: Implement the AST walker for the Client.
