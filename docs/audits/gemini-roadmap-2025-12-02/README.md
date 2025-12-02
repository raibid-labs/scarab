# Audit Summary & Roadmap

I have verified Claude's changes and identified a critical architectural gap preventing the new features from working.

## Status
*   **Data Model**: ✅ **Complete**. `Session`, `Tab`, `Pane` structs are ready.
*   **Runtime Logic**: 🔴 **Missing**. The application still runs on the "Legacy Single PTY" logic. The new structures are instantiated but disconnected.

## The Roadmap (Phase 1.5)

The next set of tasks must focus on **connecting the wires**:

1.  **Decouple VTE**: Refactor `TerminalState` to own its grid data instead of writing directly to Shared Memory. This allows multiple `TerminalState` instances (one per Pane).
2.  **Pane Integration**: Move `TerminalState` into `Pane`.
3.  **Main Loop Refactor**:
    *   Remove legacy PTY code.
    *   Implement a loop that reads from the Active Pane's PTY, updates its `TerminalState`, and then copies that state to Shared Memory ("Blit").

## Instructions for Claude CLI

Please execute **Phase 1.5** as described in `docs/audits/gemini-roadmap-2025-12-02/02-vte-refactor-guide.md`.

**Primary Goal**: Refactor `crates/scarab-daemon/src/vte.rs` to support multiple instances by removing the direct dependency on `SharedState` pointers.
