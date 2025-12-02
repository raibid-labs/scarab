# Scarab Terminal - Deep Technical Audit (Pass 2)

**Date:** December 2, 2025
**Auditor:** Gemini (CLI Agent)
**Version:** v0.1.0-alpha.14 (Estimated)

## 1. Executive Summary

Following the initial audit, the Scarab team has made significant progress. Critical gaps in **Clipboard** (X11 Primary Selection), **Pane Management** (Resizing logic), and **Security** (GPG verification) have been addressed with robust implementations.

However, the **Mouse Support** subsystem remains in a semi-functional state. While click detection logic exists, the critical "wiring" to send paste events to the terminal shell is missing, and coordinate conversion relies on hardcoded values.

**Verdict:** The project has advanced from "Prototype" to "Alpha", but is not yet "Beta" quality due to the incomplete mouse interaction loop.

## 2. Status of Previous Critical Gaps

| Feature | Status | Notes |
| :--- | :--- | :--- |
| **Clipboard (X11)** | ✅ **FIXED** | `scarab-clipboard` now implements `ClipboardType::Primary` using `arboard` Linux extensions. |
| **Pane Resizing** | ✅ **FIXED** | `scarab-panes` now contains a sophisticated tree-based resizing algorithm (`resize_pane`, `recalculate_layout`). |
| **Security (GPG)** | ✅ **FIXED** | `scarab-config` now includes a full `PluginVerifier` using `sequoia-openpgp` for signature checking. |
| **Mouse Support** | ⚠️ **PARTIAL** | Click detection works, but Paste IPC is missing (see below). |

## 3. Deep Dive: The Mouse Support Gap

Despite updates, `crates/scarab-mouse` contains critical `TODO`s that prevent it from working correctly in a real-world scenario.

### A. Missing Paste IPC
In `crates/scarab-mouse/src/bevy_plugin.rs`, the `handle_middle_click` function detects the click and retrieves text from the clipboard, but **fails to send it to the daemon**.

```rust
// crates/scarab-mouse/src/bevy_plugin.rs
fn handle_middle_click(plugin_state: &mut MousePluginState, pos: Position) {
    // ...
    // TODO: Send text to terminal via IPC
    // This would require access to the IpcSender which isn't available in this function
    log::debug!("Primary selection paste text: {:?}", text);
}
```

**Impact:** Middle-click paste (a standard Linux terminal feature) logs the text to debug output but does nothing in the terminal.

### B. Hardcoded Coordinates
The coordinate conversion logic is currently hardcoded, which breaks mouse support on any window size other than 80x24.

```rust
// crates/scarab-mouse/src/bevy_plugin.rs
fn screen_to_grid(...) -> Position {
    // TODO: Use actual font metrics and terminal dimensions
    let cols = 80;
    let rows = 24;
    // ...
}
```

**Impact:** Clicking anywhere on a resized window will send incorrect row/column coordinates to the shell (e.g., vim, tmux), making mouse support unusable.

### C. Context Menus
The context menu logic is present but the UI is not spawned.
*   `// TODO: Spawn context menu UI entity`
*   `// TODO: Implement context menu interaction`

## 4. Updated Code Statistics
*   **TODO Count:** ~256 (Note: This count includes documentation TODOs, but significant code TODOs remain in `scarab-mouse`).
*   **New Implementations:** ~1,200 lines of code added/refactored in Clipboard, Panes, and Security.

## 5. Recommendations

### Immediate Actions (Sprint Priority)
1.  **Wire Mouse IPC:**
    *   Update `handle_middle_click` signature to accept `IpcSender`.
    *   Pass the `ipc` resource from `handle_mouse_input`.
    *   Send `ControlMessage::Input { data: text }` to the daemon.
2.  **Fix Coordinate Conversion:**
    *   Inject `TerminalMetrics` (cell width/height, rows/cols) into `screen_to_grid`.
    *   Remove hardcoded 80x24 values.
3.  **Verify Pane Resizing UI:**
    *   The logic is in the backend (`scarab-panes`), but ensure the frontend (`scarab-client`) sends the correct IPC commands to trigger it.

### Strategic Goals
*   **End-to-End Testing:** Create a test that simulates a middle-click and asserts that the PTY received the text. This would have caught the missing IPC wiring immediately.

## 6. Conclusion
The "Infrastructure" layer (Security, Layout Engine, Clipboard Manager) is now solid. The "Interaction" layer (Mouse click-to-action, Screen-to-Grid math) is the final bottleneck preventing a usable release.
