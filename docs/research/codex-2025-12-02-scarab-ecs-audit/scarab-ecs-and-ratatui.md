# Scarab ECS & Ratatui Alignment Audit

Date: 2025-12-02  
Scope: Compare Scarab’s current/planned design to Bevy-driven terminal opportunities; assess ECS usage; explore Ratatui ↔ Bevy ECS mapping (incl. `bevy_ratatui`).

## Snapshot of Scarab Today (from repo inspection)
- **Split arch:** daemon (PTY/state) + client (Bevy renderer). Shared memory grid via `scarab-protocol`, IPC via sockets (`crates/scarab-client/src/ipc.rs`).
- **Rendering:** `IntegrationPlugin` builds a single terminal grid entity with `TerminalMesh` + `TextRenderer` resources (`crates/scarab-client/src/integration.rs`). Mesh regenerated when sequence changes; dirty-region tracking exists but not chunked.
- **Config/script:** Fusabi loader (`main.rs`), scripting plugin (`ScriptingPlugin`) but Fusabi→Bevy UI bindings are stubbed (`bevy_integration.rs` placeholder functions).
- **Plugins:** Custom plugin API (traits/hooks in `scarab-plugin-api`) with async hooks; not expressed as Bevy `Plugin`s—lives outside ECS; bridging via event registry inside `EventsPlugin` using `Arc<Mutex<EventRegistry>>`.
- **Tests:** Some ECS-friendly tests (`headless_poc`) but no golden grid snapshots; scrollback/tab/pane features largely TODO (per CURRENT_STATUS_AND_NEXT_STEPS.md).

## ECS Utilization Assessment
- **Strengths:**
  - Uses Bevy `Plugin` to group systems (IpcPlugin, EventsPlugin, ScrollbackPlugin, etc.).
  - Resources for shared state (`SharedMemoryReader`, `TextRenderer`) are injected and updated via systems.
  - Event bridging uses Bevy events for window focus/resize/daemon events.
- **Gaps / Under-leveraged ECS patterns:**
  - **Terminal data as monolith:** Grid lives outside ECS (shared memory + resource diff); no ECS components for cells/chunks/lines, so no per-entity change detection or spatial culling. Limits selective updates, hit-testing, and plugin-driven decoration.
  - **Chunked/Archetypal storage missing:** No chunk entities (e.g., tiles per 64x64 block) to localize dirty flags; mesh rebuild is whole-grid oriented despite a dirty-region helper.
  - **Systems not segmented by set/criteria:** Sync, render, metrics, and positioning all run in one chained set. PTY ingestion vs. rendering vs. UI overlays aren’t decoupled with run criteria/fixed timesteps, increasing contention.
  - **Plugin API bypasses ECS:** Plugins receive strings/bytes, not entities/components. No way to attach components (e.g., overlay, notification badge) from a plugin; also no scheduling inside Bevy world.
  - **Resource over mutex:** Event registry is `Arc<Mutex<...>>` instead of ECS `Resource` with system-driven dispatch; loses parallelism and type safety.
  - **Fusabi Bevy stubs:** `ui_spawn_*` natives are TODO; no resource access or entity spawning from scripts, so ECS exposure to scripting is absent.
  - **Rendering pipeline is mostly imperative:** Mesh regeneration occurs in a single system; no render-graph nodes or extraction/queue stages to exploit Bevy’s renderer properly.

## Scarab vs. Bevy-Hosted Terminal Opportunities (from research doc)
- **Missed differentiators:** current client stops at 2D text; no shaders/post-processing, spatial panes, or physics-driven UX. Dirty-region work exists but not tied to chunked ECS entities to enable partial updates/animations.
- **Hot-reload assets:** config hot-reload is planned, but fonts/shaders/themes not using asset server/hot reload.
- **Recording/testing:** No deterministic replay or headless golden-tests despite Bevy’s ScheduleRunner making this straightforward.

## Ratatui ↔ Bevy ECS: Models and Options
- **Ratatui model:** immediate-mode layout tree -> `Frame` buffer -> backend writes to terminal. Widgets are structs with `render(&mut Frame, area, buf)`.
- **Bevy ECS model:** data-driven components + systems; rendering via sprites/meshes; UI often uses bundles + events.
- **Existing bridge:** `bevy_ratatui` runs Ratatui on Bevy’s schedule (input forwarding, draw via `RatatuiContext::draw`). `bevy_ratatui_camera` renders a Bevy camera to a Ratatui widget (Bevy → Ratatui).
- **Mapping ideas for Scarab:**
  - **Bevy-driven Ratatui:** Use `bevy_ratatui` as base; Scarab client could host Ratatui widget trees as ECS resources, letting plugins submit widget trees that render into the terminal buffer (for HUDs/overlays).
  - **Ratatui-as-component:** Wrap widget descriptors in components (e.g., `RatatuiWidget<T>` where `T: Widget`) attached to entities representing panes/modals. Systems build a `Frame` buffer per entity and composite into Scarab’s atlas/mesh.
  - **Area/layout as components:** Store `Rect`, z-index, focus, and interaction handlers as components; systems compute layout then call Ratatui widgets to draw into shared buffers that are then converted to Bevy textures/quads.
  - **Event routing:** Map Bevy input (keyboard/mouse/gamepad) to Ratatui `Event`s; maintain focus stack as ECS resource; use Bevy events for TUI-level actions.

## Should Scarab create a bevy-ratatui library?
- **Pros:**
  - Leverages Ratatui’s mature widget ecosystem instead of rebuilding UI primitives.
  - Attracts Ratatui contributors; lowers maintenance of widget correctness (borders, layout, unicode).
  - Bridges daemon/client plugin model: daemon plugins could emit Ratatui widget trees serialized and rehydrated into Bevy resources.
  - Aligns with Bevy plugin culture: ship `ScarabRatatuiPlugin` that exposes systems/resources; third parties can layer Bevy plugins for new widgets or layouts.
  - Testing win: Ratatui already has buffer-based golden tests; same buffers could be validated headlessly inside Bevy CI.
- **Cons/Risks:**
  - Ratatui is immediate-mode; mapping to ECS may introduce copies if not chunked/diffed.
  - Layout computation costs could duplicate Scarab’s mesh path unless shared buffers are reused.
  - Need to reconcile render paths: Scarab currently renders glyphs via cosmic-text → mesh; Ratatui renders to a char buffer. Bridging requires a fast “buffer-to-mesh” path (ideally chunked).
- **Recommendation:** Build a **thin, reusable bridge crate** (internal or public) rather than forking Ratatui:
  - Base on `bevy_ratatui` for scheduling/input, but extend with:
    - `RatatuiSurface` component representing a target grid/overlay.
    - `RatatuiBufferBundle { buffer_handle, area, z }` storing persistent buffers per entity.
    - Systems: `ratatui_layout_system` (compute areas), `ratatui_render_system` (invoke widgets), `ratatui_flush_system` (diff buffers → Scarab mesh chunks).
    - Event mapping: Bevy input → Ratatui events; Ratatui `AppExit` → Bevy `AppExit`.
  - Keep widget compatibility: accept any `ratatui::widgets::Widget` impl; don’t wrap/rewrite widgets.
  - Provide serialization hooks for daemon-side plugins: schema for widget trees (or higher-level DSL) that can be reconstructed client-side.

## Plugin Architecture Alignment
- **Current:** Scarab plugins are F#/Rust async hooks; they do not participate in Bevy’s schedule and cannot spawn entities/components. Client plugins mostly respond to text streams and send notifications.
- **Alignment Plan:**
  - Define a **Bevy plugin layer**: `ScarabPluginHostPlugin` that exposes safe ECS APIs to plugins (spawn overlays, add status bar items, register commands). Provide a “capability” resource passed to plugins via FFI/IPC.
  - Bridge plugin hooks to Bevy events: incoming plugin actions enqueue ECS events; systems apply them (spawn entity, set component).
  - Make Fusabi stdlib bindings call ECS-safe functions (currently TODO stubs) to create UI entities or alter layout.
  - Re-express built-in features (link hints, palette, tutorial) as Bevy plugins composed of ECS systems/components; this dogfoods the model and shows plugin authors the expected pattern.
  - Align with Bevy sets: system sets for `InputIngest` (PTY/IPC), `TuiLayout`, `TuiRender`, `UiOverlay`, `Telemetry`; let plugins register systems into appropriate sets.

## Actionable Recommendations
- **Data model:** Introduce chunked ECS storage for terminal grid (e.g., `TerminalChunk { origin, size, buffer_handle, dirty }`). Keep shared memory as source-of-truth but project into chunk components for diff/render.
- **Ratatui bridge:** Prototype a `ratatui_surface` plugin using `bevy_ratatui`; render a Ratatui widget (e.g., command palette) into an overlay entity; measure buffer→mesh cost and diffing strategy.
- **Plugin API rewrite layer:** Add ECS-facing APIs (spawn overlay, status bar item, notifications) and expose them through Fusabi and Rust plugins. Remove `Arc<Mutex>` registries in favor of ECS resources/events.
- **Render pipeline:** Move mesh updates into extraction/queue stages or chunked atlas uploads; add optional shader/post-processing hooks for “cinematic terminal” path.
- **Testing:** Add headless CI job using `ScheduleRunnerPlugin` to tick systems, ingest synthetic PTY output, and snapshot buffers for golden tests; use Ratatui buffer assertions where applicable.
