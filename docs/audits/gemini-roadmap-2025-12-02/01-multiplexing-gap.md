# Audit Report: Multiplexing Architecture Gap
**Date:** December 2, 2025
**Auditor:** Gemini Agent

## 1. Verification of Claude's Work
I have reviewed the changes in `crates/scarab-daemon/src/session/`.
*   **✅ Data Model**: `Pane`, `Tab`, `Session` structs are implemented correctly. `Session` now manages a hierarchy of Tabs and Panes.
*   **✅ PTY Management**: `Pane` now owns the `pty_master`, ensuring each pane has an independent shell.
*   **✅ Layout**: Basic `Rect` viewport logic is present.

## 2. Critical Architecture Gap: "The Wires are Cut"
While the data model exists, the runtime is still using the legacy single-PTY logic.
*   **Issue A**: `main.rs` spawns a *separate, standalone* PTY that bypasses the `SessionManager`. The `Session`'s PTYs are spawned but never read from.
*   **Issue B**: `vte.rs` (`TerminalState`) is hardcoded to write directly to the **Shared Memory** (`SharedState`).
    *   This makes it impossible to have multiple background panes, as they would all overwrite the single shared screen buffer.
*   **Result**: If you run `scarab-daemon` now, it acts exactly like the old version (single shell), ignoring the new Session/Pane logic.

## 3. Roadmap: Phase 1.5 - Wiring the Multiplexer

To achieve actual multiplexing (even just switching between tabs), we must refactor the VTE pipeline.

### Step 1: Decouple VTE from Shared Memory
Refactor `crates/scarab-daemon/src/vte.rs`:
*   Change `TerminalState` to write to a generic `Grid` buffer (e.g., `Vec<Cell>`) instead of `*mut SharedState`.
*   Move `TerminalState` ownership into `Pane`.
    *   Each `Pane` must have its own `TerminalState` (cursor, attributes, scrollback) and `Grid` (cells).

### Step 2: The Compositor (Main Loop Refactor)
Refactor `crates/scarab-daemon/src/main.rs`:
1.  **Remove Legacy PTY**: Delete the PTY spawning logic in `main`.
2.  **Input Routing**:
    *   When IPC receives input, route it to `session_manager.get_active_pty_master()`.
3.  **Output Processing Loop**:
    *   Iterate over **all** active PTYs (or just the active one for MVP).
    *   Read PTY output -> Feed to that Pane's `TerminalState`.
4.  **Render Pass**:
    *   After processing updates, **lock the Active Pane**.
    *   Copy the Active Pane's `Grid` (cells) to the `SharedState` (IPC memory).
    *   Update the sequence number to trigger Client redraw.

### Step 3: Testing
*   Verify that `Ctrl+Shift+T` (or whatever keybind triggers `create_tab`) creates a new Tab/Pane/PTY.
*   Verify that input only goes to the active tab.
*   Verify that switching tabs updates the screen to the new tab's content.

## 4. Next Steps
The immediate priority is **Refactoring `vte.rs`** to be instance-based rather than global-SHM-based.
