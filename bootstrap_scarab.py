import os
import textwrap

def write_file(path, content):
    """Writes content to a file, creating directories if needed."""
    full_path = os.path.join("scarab", path)
    os.makedirs(os.path.dirname(full_path), exist_ok=True)
    with open(full_path, "w", encoding="utf-8") as f:
        f.write(content.strip() + "\n")
    print(f"Created: {full_path}")

def main():
    print("Bootstrapping Scarab Terminal Environment...")

    # ==========================================
    # 1. DOCUMENTATION
    # ==========================================
    
    research_doc = textwrap.dedent(r"""
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
    """)

    readme = textwrap.dedent(r"""
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
    """)

    claude_prompt = textwrap.dedent(r"""
    **System Role:** You are an expert Rust Systems Architect specializing in high-performance terminal emulators, game engines (Bevy), and compiler design.

    **Context:** We are building **Scarab**, a client/server terminal emulator.
    *   **Repo Path:** `raibid-labs/scarab`
    *   **Stack:** Rust, Bevy, Tokio, Portable-PTY, Alacritty-Terminal (parser), Cosmic-Text.
    *   **Architecture:** Daemon (Headless) + Client (Bevy UI) connected via Shared Memory.

    **Current Task:**
    We have generated the scaffolding. We need to implement the **Shared Memory IPC Bridge**.

    1.  **Review** `crates/scarab-protocol/src/lib.rs`. It defines the `SharedState` struct. Ensure it is `#[repr(C)]` and layout-compatible for `mmap`.
    2.  **Implement** the `Writer` logic in `scarab-daemon`. It should take output from the PTY, parse it, and update the Shared Memory ring buffer.
    3.  **Implement** the `Reader` logic in `scarab-client`. It should read the Shared Memory and update a Bevy resource/component.

    **Constraint:** Ensure the Shared Memory implementation uses a lock-free or low-latency synchronization primitive (like `AtomicU64` sequence numbers) to avoid blocking the Bevy render thread.
    """)

    # ==========================================
    # 2. CARGO WORKSPACE
    # ==========================================

    root_cargo = textwrap.dedent(r"""
    [workspace]
    resolver = "2"
    members = [
        "crates/scarab-client",
        "crates/scarab-daemon",
        "crates/scarab-protocol",
        "crates/scarab-plugin-api",
        "crates/fusabi-vm",
        "crates/fusabi-interpreter",
    ]

    [workspace.dependencies]
    bevy = { version = "0.15", default-features = false, features = ["bevy_winit", "bevy_core_pipeline", "bevy_pbr", "bevy_ui", "bevy_render", "x11", "wayland"] }
    tokio = { version = "1.36", features = ["full"] }
    portable-pty = "0.8"
    alacritty_terminal = "0.24"
    rkyv = { version = "0.7", features = ["validation"] }
    bytemuck = { version = "1.14", features = ["derive"] }
    anyhow = "1.0"
    serde = { version = "1.0", features = ["derive"] }
    cosmic-text = "0.11"
    shared_memory = "0.12"
    crossbeam = "0.8"
    
    [profile.release]
    lto = "thin"
    codegen-units = 1
    opt-level = 3
    """)

    # ==========================================
    # 3. CRATE: PROTOCOL (IPC)
    # ==========================================

    protocol_lib = textwrap.dedent(r"""
    #![no_std]
    // This crate defines the data layout shared between Daemon and Client.
    // It must be #[repr(C)] to ensure memory layout consistency across processes.

    use bytemuck::{Pod, Zeroable};

    pub const SHMEM_PATH: &str = "/scarab_shm_v1";
    pub const GRID_WIDTH: usize = 200;
    pub const GRID_HEIGHT: usize = 100;
    pub const BUFFER_SIZE: usize = GRID_WIDTH * GRID_HEIGHT;

    #[repr(C)]
    #[derive(Copy, Clone, Pod, Zeroable)]
    pub struct Cell {
        pub char_codepoint: u32, 
        pub fg: u32,   // RGBA
        pub bg: u32,   // RGBA
        pub flags: u8, // Bold, Italic, etc.
        pub _padding: [u8; 3], // Align to 16 bytes
    }

    // A double-buffered grid state living in shared memory
    #[repr(C)]
    #[derive(Copy, Clone, Pod, Zeroable)]
    pub struct SharedState {
        pub sequence_number: u64, // Atomic sequence for synchronization
        pub dirty_flag: u8,       
        pub cursor_x: u16,
        pub cursor_y: u16,
        pub _padding: [u8; 3],
        // Fixed size buffer for the "visible" screen. 
        // In production, use offset pointers to a larger ring buffer.
        pub cells:,
    }

    // Control messages (Sent via Socket/Pipe, not ShMem)
    #
    pub enum ControlMessage {
        Resize { cols: u16, rows: u16 },
        Input { data: alloc::vec::Vec<u8> },
        LoadPlugin { path: alloc::string::String },
    }
    
    extern crate alloc;
    """)

    protocol_cargo = textwrap.dedent(r"""
    [package]
    name = "scarab-protocol"
    version = "0.1.0"
    edition = "2021"

    [dependencies]
    bytemuck = { workspace = true }
    serde = { workspace = true }
    rkyv = { workspace = true }
    """)

    # ==========================================
    # 4. CRATE: DAEMON (Server)
    # ==========================================

    daemon_main = textwrap.dedent(r"""
    use portable_pty::{CommandBuilder, NativePtySystem, PtySize, PtySystem};
    use anyhow::Result;
    use std::io::{Read, Write};
    use tokio::sync::mpsc;

    #[tokio::main]
    async fn main() -> Result<()> {
        println!("Starting Scarab Daemon...");

        // 1. Setup PTY System
        let pty_system = NativePtySystem::default();
        let pair = pty_system.openpty(PtySize {
            rows: 24,
            cols: 80,
            pixel_width: 0,
            pixel_height: 0,
        })?;

        let cmd = CommandBuilder::new("bash");
        let _child = pair.slave.spawn_command(cmd)?;
        
        // Important: Release slave handle in parent process
        drop(pair.slave);

        let mut reader = pair.master.try_clone_reader()?;
        let mut writer = pair.master.take_writer()?;

        // 2. TODO: Initialize Shared Memory Here
        println!("Daemon initialized. Listening for input...");

        // 3. Main Loop
        let mut buf = [0u8; 1024];
        loop {
            // In a real implementation, use tokio::select! to handle IPC events too
            match reader.read(&mut buf) {
                Ok(n) if n > 0 => {
                    let data = &buf[..n];
                    println!("PTY Output: {:?}", String::from_utf8_lossy(data));
                    
                    // TODO: 
                    // 1. Feed data to Alacritty VTE parser
                    // 2. Update SharedState grid
                    // 3. Increment sequence_number
                }
                Ok(_) => break, // EOF
                Err(e) => {
                    eprintln!("PTY Error: {}", e);
                    break;
                }
            }
        }

        Ok(())
    }
    """)

    daemon_cargo = textwrap.dedent(r"""
    [package]
    name = "scarab-daemon"
    version = "0.1.0"
    edition = "2021"

    [dependencies]
    scarab-protocol = { path = "../scarab-protocol" }
    fusabi-vm = { path = "../fusabi-vm" }
    tokio = { workspace = true }
    portable-pty = { workspace = true }
    alacritty_terminal = { workspace = true }
    anyhow = { workspace = true }
    shared_memory = { workspace = true }
    """)

    # ==========================================
    # 5. CRATE: CLIENT (Bevy)
    # ==========================================

    client_main = textwrap.dedent(r"""
    use bevy::prelude::*;
    use scarab_protocol::SharedState;

    // Marker for the grid entity
    #[derive(Component)]
    struct TerminalGrid;

    fn main() {
        App::new()
           .add_plugins(DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Scarab Terminal".into(),
                    resolution: (1024.0, 768.0).into(),
                   ..default()
                }),
               ..default()
            }))
           .add_systems(Startup, setup)
           .add_systems(Update, (sync_grid, handle_input))
           .run();
    }

    fn setup(mut commands: Commands) {
        commands.spawn(Camera2dBundle::default());
        println!("Scarab Client Initialized.");
        
        // TODO: Initialize Shared Memory reader here
    }

    fn sync_grid() {
        // TODO: Check SharedState sequence number
        // If changed, update the texture/mesh
    }

    fn handle_input(keys: Res<Input<KeyCode>>) {
        // TODO: Send input to Daemon via socket
        if keys.just_pressed(KeyCode::Space) {
            println!("Space pressed - checking for Leader Key...");
        }
    }
    """)

    client_cargo = textwrap.dedent(r"""
    [package]
    name = "scarab-client"
    version = "0.1.0"
    edition = "2021"

    [dependencies]
    scarab-protocol = { path = "../scarab-protocol" }
    fusabi-interpreter = { path = "../fusabi-interpreter" }
    bevy = { workspace = true }
    cosmic-text = { workspace = true }
    anyhow = { workspace = true }
    shared_memory = { workspace = true }
    """)

    # ==========================================
    # 6. FUSABI COMPONENTS
    # ==========================================

    vm_lib = textwrap.dedent(r"""
    // fusabi-vm: The AOT runtime for the Daemon
    pub struct VirtualMachine;

    impl VirtualMachine {
        pub fn new() -> Self { Self }
        pub fn exec_binary(&self, _bytes: &[u8]) {
            // Execute.fzb bytecode
        }
    }
    """)

    interpreter_lib = textwrap.dedent(r"""
    // fusabi-interpreter: The Script runtime for the Client
    pub struct Interpreter;

    impl Interpreter {
        pub fn eval(&self, _script: &str) {
            // Parse and run.fsx script
        }
    }
    """)

    # Writing Files
    write_file("docs/architecture_report.md", research_doc)
    write_file("README.md", readme)
    write_file("CLAUDE_PROMPT.md", claude_prompt)
    write_file("Cargo.toml", root_cargo)
    
    write_file("crates/scarab-protocol/src/lib.rs", protocol_lib)
    write_file("crates/scarab-protocol/Cargo.toml", protocol_cargo)
    
    write_file("crates/scarab-daemon/src/main.rs", daemon_main)
    write_file("crates/scarab-daemon/Cargo.toml", daemon_cargo)
    
    write_file("crates/scarab-client/src/main.rs", client_main)
    write_file("crates/scarab-client/Cargo.toml", client_cargo)
    
    write_file("crates/fusabi-vm/src/lib.rs", vm_lib)
    write_file("crates/fusabi-vm/Cargo.toml", '[package]\nname = "fusabi-vm"\nversion = "0.1.0"\nedition = "2021"\n[dependencies]')
    
    write_file("crates/fusabi-interpreter/src/lib.rs", interpreter_lib)
    write_file("crates/fusabi-interpreter/Cargo.toml", '[package]\nname = "fusabi-interpreter"\nversion = "0.1.0"\nedition = "2021"\n[dependencies]')
    
    write_file("crates/scarab-plugin-api/src/lib.rs", "// Shared API traits")
    write_file("crates/scarab-plugin-api/Cargo.toml", '[package]\nname = "scarab-plugin-api"\nversion = "0.1.0"\nedition = "2021"\n[dependencies]')

if __name__ == "__main__":
    main()