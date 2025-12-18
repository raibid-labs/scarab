# Scarab Execution Roadmap (Bevy/Ratatui/ECS Alignment)
Date: 2025-12-02  
Context: Incorporates latest multiplexing commits (`PaneOrchestrator`, IPC tab/pane routing) and roadmap priorities from `docs/ROADMAP-AI.md`, plus Codex research on Bevy+Ratatui+ECS.

## Guiding Outcomes
1) **Rendering parity & media**: Images (iTerm2 → Bevy textures), future Sixel/Kitty; ligatures verified.  
2) **ECS-native UI**: Terminal grid chunked as ECS data; overlays/widgets are Bevy systems/plugins.  
3) **Ratatui compatibility**: Thin bevy-ratatui bridge for overlays/tests; avoid re-inventing widgets.  
4) **Plugin alignment**: Plugin API surfaced via Bevy events/resources; Fusabi bindings can spawn/modify ECS UI.  
5) **Headless CI**: Deterministic headless test path (ScheduleRunner) with snapshot/assert support (ties into ratatui-testlib issues #14–#16).

## Phase 0 – Triage & Foundations (1–2 days)
- **Stabilize harness**: Track ratatui-testlib issues #14–#16 for Sixel parsing, wait loop hangs, headless runner. If not landed, add local minimal headless harness for Scarab tests (ScheduleRunner + grid snapshot) under `crates/scarab-client/tests/headless_*`.
- **Perf/telemetry knobs**: Add low-verbosity logging for compositor + orchestrator to validate the new tab/pane flow (no code changes, just config/examples).

## Phase 1 – Media Pipeline (Images + Ligatures) (3–5 days)
- **Protocol extension** (`crates/scarab-protocol/src/lib.rs`): Add `SharedImageBuffer` & `ImagePlacement` (bounded blob buffer, count, offsets).  
- **Daemon**:
  - In `images/iterm2.rs` and `vte.rs`, capture decoded iTerm2 payloads → per-pane image store → write placements/blobs to shared buffer during `blit_to_shm()`.
  - Clear placements on RIS/full reset; reuse IDs to avoid leaks.
  - Wire compositor to include image buffer with active pane blit.
- **Client**:
  - New Bevy resource/system to read `SharedImageBuffer`, upload textures, spawn sprites/quads over terminal mesh (z > text).
  - Basic LRU for textures keyed by image_id; drop evicted blobs.
- **Ligatures verification**:
  - Ensure `cosmic-text` Harfbuzz feature enabled (`crates/scarab-client/Cargo.toml`).
  - Add a golden test (headless) rendering “== != -> =>” with Fira Code; assert glyphs differ from monospaced baseline.

## Phase 2 – ECS Grid & Dirty Diff (2–4 days)
- **Chunked grid projection**:
  - Introduce `TerminalChunk` component (e.g., 64x32 cells) entities representing regions of the grid; store dirty flags and mesh handles.
  - System: map `SharedMemoryReader` → mark dirty chunks based on seq/dirty region; generate meshes per chunk instead of whole-grid rebuild (`IntegrationPlugin` update).
  - Maintain `TerminalMetrics` for hit-testing; adjust `update_grid_position_system` to chunk origin-aware.
- **Testing**: Add headless test to ensure only touched chunks rebuild (simulate small write, assert only 1 chunk mesh changed).

## Phase 3 – Ratatui Bridge & Overlays (3–5 days)
- **Bridge crate (internal first)**: `crates/scarab-ratatui-bridge` (or module) wrapping bevy_ratatui concepts:
  - `RatatuiSurface` component (target area, z, buffer handle).
  - Systems: layout → render widget tree into buffers → diff to Scarab mesh or overlay texture.
  - Input mapping: Bevy input → Ratatui `Event`; focus stack as resource.
- **Prototype overlay**: Implement command palette as Ratatui widget via the bridge to prove compatibility; render over base grid with transparency.
- **Docs**: Short guide in `docs/` on how to register a Ratatui overlay as a Bevy plugin.

## Phase 4 – Plugin Alignment (Fusabi + Rust) (3–4 days)
- **Bevy plugin host**: Add `ScarabPluginHostPlugin` exposing ECS-safe commands via events/resources:
  - Actions: spawn overlay, status item, notification bubble, register keybinding.
  - Map Fusabi natives (`ui_spawn_*`) to dispatch these events instead of stubs.
- **Event routing**: Replace `Arc<Mutex<EventRegistry>>` with ECS resources/events; bridge daemon/plugin events into ECS system sets.
- **Dogfood**: Port link-hints/palette/tutorial to Bevy plugin form using the new host to set the pattern for third parties.

## Phase 5 – Shell Integration & UX polish (2–3 days)
- Implement OSC 133 markers in daemon; expose markers to client (render subtle gutter markers, enable jump-to-previous-prompt).
- Optional shader track: add toggleable post-process (slight blur/glow) to distinguish overlays; keep a low-power switch.

## Phase 6 – Headless CI & Snapshots (2–3 days, parallel-friendly)
- If ratatui-testlib #16 lands, consume it; else add local `headless_runner.rs`:
  - ScheduleRunnerPlugin app, feed synthetic PTY output (mock SharedState), render meshes, capture grid text → insta snapshots.
  - Add one golden test for iTerm2 image placement (once Phase 1 ships) and one for ligatures.

## Milestone Mapping to ROADMAP-AI priorities
- **Images**: Phase 1 delivers the missing pipeline (iTerm2 now end-to-end). Sixel/Kitty remain future.
- **Ligatures**: Phase 1 verifies/locks shaping.
- **Config/Plugins**: Phases 3–4 align Fusabi/plugins with Bevy ECS.
- **Testing**: Phases 2 & 6 create deterministic diffable paths; leverage ratatui-testlib if issues 14–16 resolve.

## Notes/Risks
- Image buffer size must be bounded (consider 16–32 MB cap). Enforce count limit and clear-on-reset to avoid leaks.
- Chunked grid must not regress perf; profile mesh generation; fallback to coarse chunk size if archetype churn is high.
- Bridge crate should stay thin—defer widget rewriting; accept any `ratatui::Widget` to keep compatibility.
