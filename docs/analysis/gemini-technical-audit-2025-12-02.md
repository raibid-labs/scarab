# Scarab Terminal - Deep Technical Audit

**Date:** December 2, 2025
**Auditor:** Gemini (CLI Agent)
**Version:** v0.1.0-alpha.13

## 1. Executive Summary

Scarab is an ambitious, high-performance terminal emulator built on a modern Rust stack (Bevy, Fusabi, cosmic-text). The architecture—splitting the headless daemon from the GPU-accelerated client—is robust and well-suited for crash resilience and session persistence.

However, the project is currently in a "functional prototype" or "early alpha" state. While the core rendering and plugin architecture are in place, critical interactive features (mouse support, robust clipboard integration, pane resizing) are largely unimplemented or marked with `TODO`. Documentation is voluminous but contains significant placeholder content.

**Verdict:** promising foundation, but not yet ready for "daily driver" usage due to interaction gaps.

## 2. Architecture Analysis

### Strengths
*   **Split-Process Model:** The separation of `scarab-daemon` (state/PTY) and `scarab-client` (UI/Rendering) is excellent. It ensures that a UI crash doesn't kill the shell session.
*   **Zero-Copy IPC:** Utilization of shared memory for grid updates is a performance critical design choice that appears well-implemented.
*   **Modular Crates:** The `crates/` directory shows a clean separation of concerns (`scarab-protocol`, `scarab-plugin-api`, etc.).
*   **Plugin First:** The architecture treats core features (like `scarab-mouse`, `scarab-nav`) as plugins, which ensures the plugin API is powerful enough for 3rd party developers.

### Weaknesses / Risks
*   **Complexity:** The "Fusabi" dual-runtime approach (VM in daemon, Interpreter in client) significantly increases the complexity of the system. Maintaining a custom language dialect and two runtimes is a heavy burden.
*   **IPC Bottlenecks:** While grid data is shared memory, the control events rely on sockets. The `TODO`s in `scarab-mouse` suggest that the event piping for complex interactions (like mouse drags) is not fully wired up yet.

## 3. Code Quality & Technical Debt

A scan of the codebase reveals **174+ "TODO" comments**, indicating significant unfinished work.

### Critical Gaps
1.  **Mouse Support (`crates/scarab-mouse`)**:
    *   This crate is largely a skeleton.
    *   Multiple instances of `// TODO: Send to daemon via IPC` indicate that mouse events are captured by the client but not forwarded to the PTY or plugins effectively.
    *   Selection logic is missing or incomplete.

2.  **Clipboard (`crates/scarab-clipboard`)**:
    *   `// TODO: Implement proper X11 primary selection support`. This is a standard feature for Linux terminals and its absence is a major usability issue.

3.  **Pane Management (`crates/scarab-panes`)**:
    *   Resizing logic is missing: `// TODO: Implement pane resizing logic`.
    *   This means split panes are likely fixed-size or non-functional for layout adjustments.

4.  **Client UI (`crates/scarab-client`)**:
    *   Scrollback coordination: `// TODO: Convert cursor_pos to scrollback line`.
    *   Search Overlay: Regex and case-sensitivity are hardcoded to `false`.

### Security
*   `crates/scarab-config/src/registry/security.rs`: `// TODO: Implement GPG signature verification`. Currently, plugin security appears to be minimal or non-existent.

## 4. Documentation Status

The project has an impressive amount of documentation in `docs/`, but quality varies.

*   **Excellent:** Architecture reports, `CURRENT_STATUS_AND_NEXT_STEPS.md` (very honest and up-to-date).
*   **Needs Work:**
    *   `README.md`: Mentions features (like mouse support) that the code audit suggests are incomplete.
    *   Placeholder Artifacts: `PULL_REQUEST_TEMPLATE.md` lists many GIFs (`todo: record`) that likely don't exist yet.
    *   Tutorials: `docs/tutorials/` exists but some files have `TODO` sections.

## 5. Testing & Tooling

### Tooling
*   **`justfile`**: Excellent. Provides a unified interface for all dev tasks (build, run, test, plugin-dev).
*   **CI**: GitHub Actions are present (`.github/workflows`), covering CI, performance, and plugins.

### Testing
*   **Unit/Integration**: Good coverage in `crates/scarab-daemon/tests` (IPC, Plugin integration).
*   **UI Tests**: `crates/scarab-client/tests` contains actual UI logic tests (harness based), which is a high-quality practice for a GUI app.

## 6. Recommendations

### Priority 1: Interaction Basics (The "Daily Driver" Threshold)
Before adding more features, the team must resolve the `TODO`s in:
1.  **Mouse Events:** Wire up the IPC to send mouse clicks/drags to the daemon/PTY.
2.  **Clipboard:** Implement X11 Primary Selection (middle-click paste).
3.  **Selection:** Fix coordinate conversion for scrollback selection.

### Priority 2: Security
*   Implement the GPG signature verification for plugins before encouraging a plugin ecosystem.

### Priority 3: Cleanup
*   Review the 174 TODOs. Many are likely stale or easy fixes (e.g., hardcoded config values).
*   Update documentation to accurately reflect "Implemented" vs "Planned" to manage user expectations.

### Priority 4: Fusabi Simplification
*   Evaluate if the dual-runtime complexity is worth it. If the VM is strictly for performance, ensure benchmarks justify the maintenance cost vs just using the interpreter or a standard embedded language (like Lua or Rhai), or ensure Fusabi's unique value prop (F# syntax) is critical.
