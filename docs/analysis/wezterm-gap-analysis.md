# Scarab vs WezTerm: Deep Technical Audit & Gap Analysis

**Date:** December 2, 2025
**Auditor:** Gemini (CLI Agent)
**Status:** Pass 3 (Re-Audit & Comparative Analysis)

## 1. Re-Audit of Critical Systems

Following the latest code updates, I have re-examined the `scarab-mouse` crate.

*   ✅ **Mouse IPC (Paste):** The `handle_middle_click` function now correctly accepts an `IpcSender` and sends a `ControlMessage::Input` with the clipboard content. The "logging but not pasting" bug is **FIXED**.
*   ✅ **Coordinate Mapping:** The `handle_mouse_input` system now attempts to retrieve `TerminalMetrics` and uses `metrics.screen_to_grid`. It falls back to hardcoded values only if metrics are missing. This is a robust **FIX** for the resizing issue.

**Conclusion:** The "Daily Driver" blockers identified in Pass 2 are resolved. Scarab is now functionally usable for interactive terminal work.

## 2. Deep Technical Audit: WezTerm Architecture

To provide a roadmap for Scarab's growth, I have analyzed **WezTerm**, the current gold standard for programmable terminals.

### 2.1 WezTerm's Lua Layer
WezTerm does not just use Lua for "settings" (key/value pairs); it exposes a **full object model** of the terminal state to the Lua runtime.

*   **Event-Driven:** `wezterm.on('event-name', callback)` allows users to hook into virtually anything (window creation, focus change, periodic status updates).
*   **Object Model:**
    *   `window`: Represents an OS window. Methods: `set_right_status`, `maximize`, `toast_notification`.
    *   `pane`: Represents a terminal pane. Methods: `send_text`, `send_paste`, `get_current_working_dir`, `split`.
    *   `tab`: Represents a tab. Methods: `set_title`, `activate`.
*   **Rich Text:** The `wezterm.format` function allows users to construct complex, multi-colored status lines programmatically.

### 2.2 The "Fusabi Gap" (Scarab's Deficits)

Scarab's Fusabi implementation is currently "Hook-Based" (`OnOutput`, `OnInput`) rather than "Object-Oriented".

| Feature | WezTerm (Lua) | Scarab (Fusabi) | Gap Severity |
| :--- | :--- | :--- | :--- |
| **State Access** | `window:active_pane()` returns a live object | `ctx` context passed to hooks (stateless) | 🔴 **High** |
| **Status Bar** | `update-status` event + `window:set_right_status(formatted_text)` | Custom overlays via plugins (harder) | 🟠 **Medium** |
| **Key Tables** | `key_tables` for modal layers (resize, copy mode) | Basic `keybindings` map | 🟠 **Medium** |
| **Events** | Hundreds (user-vars, bell, title-change) | <10 (Input, Output, Resize, Load) | 🔴 **High** |
| **Async/IO** | `wezterm.run_child_process` (async) | Limited/None (security sandbox?) | 🟡 **Low** |

**Insight:** WezTerm allows the *user* to write the UI logic (status bars, tab bars) in Lua. Scarab currently hardcodes the UI in Bevy and lets plugins "influence" it. To reach WezTerm parity, Scarab needs to **expose the Bevy UI tree to Fusabi**.

## 3. Performance Benchmarking Strategy

To prove Scarab's "High Performance" claims, we need rigorous metrics.

### 3.1 Leveraging `ratatui-testlib`
Since you have `~/raibid-labs/ratatui-testlib`, we can use it for **Correctness & Rendering Fidelity**.
*   **Goal:** Ensure Scarab renders TUI apps (like `htop`, `vim`) exactly as expected.
*   **Method:** Create a Rust test harness in `tests/` that imports `ratatui-testlib` (via local path dependency) and runs a "headless" Scarab client against golden reference screenshots or grid state dumps.

### 3.2 Throughput & Latency (The "Fast" Factor)
I recommend adopting standard tools to compare against Alacritty, Kitty, and WezTerm:

1.  **`vtebench`**: Measures throughput (cat large files, alt-screen switching).
    *   *Metric:* MB/s of text processed.
2.  **`typometer`**: Measures "photon-to-photon" latency.
    *   *Metric:* Input latency (ms).
3.  **`term-perf`**: A script to measure frame times during heavy load.

## 4. Next Steps (Roadmap)

### Phase 1: Benchmark Foundation (Immediate)
1.  **Integrate Testlib:** Add `ratatui-testlib` as a dev-dependency (path: `../ratatui-testlib`).
2.  **Benchmark Suite:** Create `benches/rendering_benchmark.rs` using `criterion` to measure grid update speeds.

### Phase 2: Fusabi Object Model (The WezTerm Killer)
1.  **Expose Objects:** Create `Window`, `Pane`, and `Tab` types in Fusabi that proxy calls to the Daemon via IPC.
2.  **Status Bar API:** Allow Fusabi plugins to return a "Render List" for the status bar, which Bevy then renders.
3.  **Expanded Events:** Fire Fusabi hooks on `FocusChanged`, `TitleChanged`, `Bell`, etc.

### Phase 3: WezTerm Parity Features
1.  **Ligatures:** `cosmic-text` supports this, ensure it's enabled.
2.  **Image Protocol:** WezTerm has a powerful image protocol (`imgcat`). Scarab needs to implement this in the Bevy renderer (rendering images on quads in the grid).
3.  **Sixel Support:** Essential for legacy image compatibility.

## 5. Proposed Action Plan

I recommend we start by **setting up the benchmark suite** to establish a baseline before adding more features.

**Shall I scaffold a `benches/` directory with a `vtebench`-style throughput test?**
