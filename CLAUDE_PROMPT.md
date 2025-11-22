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
