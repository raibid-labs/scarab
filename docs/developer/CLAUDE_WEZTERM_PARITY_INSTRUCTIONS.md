# Instructions for Claude: WezTerm Parity & Fusabi Object Model

**Objective:** Close the architectural and feature gaps between Scarab's Fusabi scripting layer and WezTerm's Lua configuration API.

**Context:**
Gemini has performed a deep gap analysis (`docs/analysis/wezterm-gap-analysis.md`) and identified that while Scarab has a powerful plugin system, it lacks the "Object Model" that makes WezTerm so programmable. WezTerm exposes live objects (`window`, `pane`, `tab`) to Lua, allowing users to script the UI logic (e.g., status bars, custom event handling) rather than just "hooking" into a predefined pipeline.

**Your Mission:**
You are responsible for **Phase 2: Fusabi Object Model & Parity Features**. You need to design and begin implementing the infrastructure to expose Bevy/Daemon state to the Fusabi runtime in an object-oriented manner.

## Task 1: Research & Specification
1.  **Analyze WezTerm's API:**
    *   Study `wezterm.window`, `wezterm.pane`, and `wezterm.tab` objects.
    *   Understand how `wezterm.on` works for custom events.
    *   Look at `wezterm.format` for styled text in status bars.
2.  **Design the Fusabi Equivalent:**
    *   Draft a spec in `docs/architecture/FUSABI_OBJECT_MODEL.md`.
    *   How do we map a Rust `Entity` (Bevy) or `SessionId` (Daemon) to a Fusabi `struct`/`class`?
    *   Define the API surface:
        *   `Window.Current()` -> returns current window handle.
        *   `window.ActivePane()` -> returns pane handle.
        *   `pane.SendText(text)`
        *   `pane.GetTitle()`

## Task 2: Issue Creation
Create detailed markdown files in `docs/issues/` (or actual GitHub issues if you have access, otherwise simulate them) for the following:

*   **Issue: Fusabi Object Model Infrastructure**
    *   Mechanism to proxy method calls from Fusabi VM -> Host -> Daemon/Client IPC.
*   **Issue: Status Bar Rendering API**
    *   Allow `OnStatusUpdate` hook in Fusabi to return a "Render List" (styled text, widgets) that Bevy renders.
    *   Deprecate/Refactor hardcoded status bars.
*   **Issue: Rich Event System**
    *   Expand `PluginApi` to support granular events: `FocusChanged`, `TitleChanged`, `BellRing`, `SelectionChanged`.
*   **Issue: Advanced Rendering**
    *   Ligature support via `cosmic-text`.
    *   Image protocol support (Sixel/iTerm2/Kitty).

## Task 3: Implementation Kickoff
Start with **Issue: Status Bar Rendering API**.
1.  This is the most visible gap. Users want customizable status bars.
2.  Define a `RenderItem` struct in `scarab-plugin-api`.
3.  Create a hook `OnStatusUpdate` that returns `Vec<RenderItem>`.
4.  Implement the rendering logic in `scarab-client` to consume this list.

**References:**
*   `docs/analysis/wezterm-gap-analysis.md` (Gemini's Analysis)
*   `crates/scarab-plugin-api/` (Current API)
*   `crates/scarab-client/src/ui/` (Where rendering happens)

**Note:** Gemini is handling Phase 1 (Performance Benchmarking) in parallel. Coordinate if you touch the rendering pipeline heavily.
