# Scarab Technical Audit Report
**Date:** December 1, 2025
**Auditor:** Gemini Agent

## 1. Executive Summary
The Scarab project is a high-performance terminal emulator built on the Bevy game engine. This audit assesses the current state of the rendering pipeline, code architecture, and integration with the 'mimic' VTE testing library.

**Key Findings:**
- **Rendering Correctness:** The project correctly utilizes a 2D rendering pipeline (`Camera2d`, `Mesh2d`, `OrthographicProjection`). The concern regarding "using a 3D renderer for a 2D job" appears to be **resolved** in the current codebase.
- **Architecture:** The architecture is robust, featuring a clean separation between the headless `scarab-daemon` and the graphical `scarab-client` via shared memory IPC.
- **Testing (Mimic):** Integration with the `mimic` VTE conformance library is **present but inactive**. The test harness exists but lacks the necessary hooks in `mimic` to perform actual state verification.

## 2. Rendering Pipeline Assessment
### Claim Verification
> "Verify that we are still following this as an example: https://github.com/cxreiff/bevy_ratatui. You both missed that we were trying to use a 3d renderer for a 2d rendering job."

**Analysis:**
- **Status:** **Compliant (mostly).**
- **Details:** 
    - `crates/scarab-client/src/main.rs` explicitly initializes a `Camera2d` with an `OrthographicProjection`.
    - `crates/scarab-client/src/integration.rs` uses `Mesh2d` and `MeshMaterial2d` (standard Bevy 0.15+ 2D components).
    - The rendering strategy involves generating a single dynamic mesh (`generate_terminal_mesh`) backed by a font atlas, which is a highly efficient 2D technique.
    - There are **no signs** of 3D components (`PbrBundle`, `Camera3d`, `StandardMaterial`) being used for the terminal grid.
    - While it does not literally depend on `bevy_ratatui`, it follows the *spirit* of efficient 2D rendering (rendering a grid state to a unified visual representation) rather than spawning thousands of individual entity sprites.

## 3. Code Quality & Structure
- **Modular Design:** The use of Bevy plugins (`IntegrationPlugin`, `IpcPlugin`, `ScriptingPlugin`) allows for excellent separation of concerns.
- **IPC Model:** The shared memory ring-buffer approach for `daemon` <-> `client` communication is appropriate for low-latency terminal updates.
- **Configuration:** The dual-loader approach (Fusabi scripting + TOML fallback) provides great flexibility but adds complexity.

## 4. Mimic Integration & Testing
The `mimic` library (located at `~/raibid-labs/mimic`) is intended to serve as the "golden standard" for VTE parsing to ensure Scarab handles ANSI escape codes correctly.

**Current Status:**
- `crates/scarab-daemon/tests/vte_conformance.rs` imports `mimic` but performs manual assertions on Scarab's state.
- No direct comparison between `mimic`'s internal state and Scarab's `SharedState` is currently occurring.

**Blocking Issues:**
To fully utilize `mimic` for integration testing and benchmarking, the following features are missing from the `mimic` library:

1.  **Headless Input API:** Scarab needs to feed raw byte streams (simulating PTY output) directly into `mimic`'s parser without needing a full shell session or PTY allocation.
2.  **State Inspection API:** `mimic` must expose its internal grid state (character cells, foreground/background colors, attributes) in a structured, public format that can be compared diff-wise against Scarab's `SharedState`.

## 5. Action Items
1.  **Implement Missing Mimic Features:** Update the `mimic` library to expose the necessary APIs for headless parsing and state inspection.
2.  **Activate Conformance Tests:** Update `vte_conformance.rs` to feed the same input to both Scarab and Mimic, then assert `scarab_state == mimic_state`.
3.  **Benchmark:** Create a benchmark comparing Scarab's parsing throughput against Mimic's reference implementation to ensure performance isn't sacrificed for correctness.
