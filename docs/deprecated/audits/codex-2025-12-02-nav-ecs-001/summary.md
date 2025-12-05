# Scarab Technical Audit – Bevy/ECS + Navigation
Date: 2025-12-02  
Scope: Review post-`d381a08` state, Bevy/ECS alignment, and advanced navigation (vimium/cosmos-nvim/spacemacs patterns) within the ECS model.

## Key Findings (ordered)
- **Unverified mega-commit (`d381a08`)**: The “implement complete Bevy/ECS roadmap (Phases 0-6)” commit adds ~13k LOC, claims 300+ tests, image pipeline, ratatui bridge, plugin host, OSC 133, chunked grid, etc. None of this has been compiled or exercised here; the probability of hidden breakage is high. Risk: large surface for regressions, CI/runtime failures, and undocumented behaviors.
- **Navigation still plugin-side, not ECS-native**: `crates/scarab-nav` remains a Fusabi/Rust plugin that draws overlays via remote commands and parses PTY text/regex. It does not bind to Bevy ECS focus/selection components, nor to chunked grid or ratatui overlays. No unified focus tree or navigation state stored in ECS.
- **Input/event path duplication**: New `plugin_host`/`bevy_events` layers may coexist with legacy `EventsPlugin` mutex registry. Without consolidation, key events for navigation (focus change, cursor pos, prompt markers) can be lost or double-handled.
- **Chunked grid/render path unvalidated**: `terminal/chunks.rs` introduces chunk entities, but the dirty-marking strategy, extraction into meshes, and sync with `SharedMemoryReader` aren’t covered by reproducible tests here. Potential desync: per-chunk `last_sequence` vs. global seq could skip updates.
- **Ratatui bridge & command palette**: Large ratatui bridge module exists, but its integration path with navigation overlays (hint labels, focusable elements) isn’t wired. No evidence that hint overlays are rendered via the bridge or share focus with Bevy UI.
- **Prompt markers / OSC 133**: New `prompt_markers.rs` added; unclear if navigation uses these markers for jump-to-prompt or visual gutters. Integration with nav actions is missing.
- **Headless harness/tests**: Headless harness and 50+ golden tests are claimed; none were run. If real, they need gating in CI and documentation of how to run/select subsets.

## Navigation-Focused Recommendations
1. **Make navigation ECS-native**  
   - Define navigation components/resources: `NavFocus`, `NavHint`, `NavGroup`, `NavMode` (normal/hints/insert), `NavAction` enum (open, click, jump-prompt, pane/tab move).  
   - Represent focusable elements as entities with bounds (from terminal chunks or ratatui surfaces). Include source (terminal cell range, ratatui widget rect, prompt marker).
2. **Hook into terminal/rendition data**  
   - Emit ECS events when text regions are classified (URLs, file paths, prompt markers). Tag entities for hints instead of regex inside plugin.  
   - Use chunk metadata to map cell coords → world coords for cursor/hit testing; expose via resource so nav systems don’t parse PTY text.
3. **Unify input routing**  
   - Centralize keymap for nav modes (vimium/cosmos/spacemacs) in Bevy `Input` system set; dispatch events to nav systems. Avoid ad-hoc parsing in plugin.  
   - Provide a mode stack resource; expose commands: enter-hint-mode, cancel, jump-next-prompt, follow-hint, focus-pane/tab.
4. **Render hints via ECS/rend pipeline**  
   - Replace remote overlay commands with Bevy entities (`HintOverlay` with text/quad) rendered in 2D, optionally via ratatui bridge if text widgets are preferred.  
   - Animate hint spawn/fade leveraging Bevy time; keep them in a dedicated render layer above terminal mesh/images.
5. **Prompt-aware navigation**  
   - Consume `PromptMarkers` resource (OSC 133) to create `NavAnchor` entities; map keys (Ctrl+Up/Down) to focus jump across anchors; allow hint-filtering scoped to current prompt block.
6. **Plugin bridge**  
   - Expose nav actions via `ScarabPluginHostPlugin` so Fusabi/Rust plugins can request: enter hint mode, register custom actions, or inject focusable regions (e.g., status bar buttons) without direct PTY inspection.

## Broader Next Steps / Roadmap
- **Stabilize mega-commit**  
  - Compile & run targeted tests: `cargo test -p scarab-client -- tests::golden_tests`, image pipeline tests, headless harness tests. Gate with feature flags if needed.  
  - Validate chunked rendering vs. shared memory: instrument per-chunk mesh rebuild counts and sequence handling.
- **Navigation implementation plan**  
  1) Add navigation ECS types/resources and input system set; wire mode transitions.  
  2) Emit focusable entities from terminal scan pass (URL/file detection, prompt markers); optional ratatui widget export path.  
  3) Render hint overlays via Bevy (or ratatui surface) with z-layer; remove remote overlay dependence.  
  4) Integrate prompt markers into nav commands; add tests for jump-to-prompt.  
  5) Add headless nav tests: synthesize grid with links/prompts → assert hint labels and follow actions yield expected PTY sequences.
- **Event plumbing cleanup**  
  - Choose single event path (Bevy events via plugin_host) and deprecate mutex `EventRegistry`; ensure IPC events and daemon events flow into ECS for nav to consume.
- **Documentation**  
  - Document nav modes/keymaps (vimium/cosmos/spacemacs) in `docs/` and ensure bindings live in code (not just plugins).  
  - Add developer notes on how to register focusable regions and nav actions from plugins/rust code.

## Notes / Risks
- Large auto-generated changes may mask regressions; prioritize smoke tests on client startup, image render, and nav hotkeys before stacking new work.
- Navigation correctness depends on accurate cell→world mapping; chunk origin math and scaling need validation against window resize and HiDPI.

## Open Issues Alignment (Dec 3, 2025)
- **#34 Integration tests for tab/pane multiplexing**: Fold into stabilization—add headless tests ensuring nav focus and hint mode survive tab/pane switches and that chunk dirtying follows active pane.
- **#36 Deep shell integration**: Leverage `PromptMarkers` and nav anchors; ensure command/prompt zones become focusable regions for jump/selection; add semantic zones to nav entity tagging.
- **#35 Post-process shader effects**: If enabled, keep hint overlays and focus outlines in a distinct render layer to avoid blur; add a toggle in nav render system to opt-out of post-processing.
- **#31 Context menu system**: Represent context menus as Ratatui/Bevy overlay entities; navigation should treat menu items as focusable nodes, reusing the nav focus stack.
- **#28 Sixel / #29 Kitty graphics**: When adding these protocols, ensure the render path coexists with nav overlays (z-order) and that image regions expose bounds so nav can avoid covering them or target them when appropriate (e.g., clickable previews).
- **#30 SSH multiplexing**: Nav state should be session-aware (per remote domain); store nav mode/focus per session to avoid cross-session leaks.
- **#33 GPG for plugins / #32 plugin marketplace**: Tie into plugin host—nav actions should allow focusing marketplace UI elements rendered via Ratatui/Bevy; security work is orthogonal but ensures nav-triggered plugin installs surface trust prompts as nav-focusable overlays.
