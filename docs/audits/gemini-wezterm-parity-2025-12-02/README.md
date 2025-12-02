# Audit Summary & Instructions for Claude

I have completed a deep technical audit of the `scarab` repository (v0.1.0-alpha.13).

## Audit Results
*   **Code Quality**: The critical issues (Unsafe Transmute, DB Performance) identified in the previous audit have been **FIXED**.
*   **Recent Features**: Scrollback UI, Fusabi Config Loader, and iTerm2 Image Parsing are **VERIFIED** as present in the code.
*   **WezTerm Parity**:
    *   **Configuration**: Good progress (Fusabi).
    *   **Scrollback**: Parity achieved.
    *   **Multiplexing**: 🔴 **Critical Gap**. No Tabs/Panes architecture exists yet.
    *   **Images**: ⚠️ **Partial**. Parser exists, but no rendering pipeline.

## Documentation Created
I have generated the following detailed reports in `docs/audits/gemini-wezterm-parity-2025-12-02/`:
1.  `01-parity-gap-analysis.md`: Detailed breakdown of missing WezTerm features.
2.  `02-claude-work-verification.md`: Validation of recent fixes and features.
3.  `03-implementation-plan.md`: Technical design for implementing Multiplexing and Image Rendering.

## Instructions for Claude CLI

Please proceed with **Phase 1: Multiplexing Architecture** as described in `docs/audits/gemini-wezterm-parity-2025-12-02/03-implementation-plan.md`.

### Immediate Tasks:
1.  **Refactor Session Model**:
    *   Create `Pane` struct in `scarab-daemon`.
    *   Move PTY ownership from `Session` to `Pane`.
    *   Implement basic "Active Pane" routing logic.
    *   *Constraint*: For the first iteration, you may implement a "Single Active Pane" model (Tabs) to avoid complex grid composition logic, but the data structure must support multiple panes.

2.  **Update IPC**:
    *   Ensure `GridState` updates reflect the *Active Pane*.

Refer to the Implementation Plan for detailed architectural guidance.
