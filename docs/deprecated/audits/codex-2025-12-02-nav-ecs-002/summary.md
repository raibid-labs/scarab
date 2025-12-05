# Scarab Technical Audit – Navigation/ECS Integration (Post-Issue Closure)
Date: 2025-12-02  
Scope: Review latest state after all issues closed (`0d31544` release). Focus on navigation, ECS alignment, rendering, and Bevy-centric opportunities.

## Observations
- **ECS-native navigation landed** (`navigation/`, `input/nav_input.rs`): Nav modes (Normal/Hints/CommandPalette/Insert), NavAction events, mode stack, focus/history resources, focusable/hint components, and vimium/cosmos/spacemacs keymaps. Navigation plugin now lives in ECS, not the old plugin overlay model.
- **EventRegistry deprecated**: Navigation hooks now rely on Bevy events/resources; the mutex registry path appears removed/replaced.
- **Prompt markers present**: OSC 133 handling + `PromptMarkers` resource exists, but wiring to nav actions needs validation (JumpPrompt actions are defined).
- **Chunked rendering and ratatui bridge still present**: Terminal chunks, image pipeline, and ratatui bridge coexist with nav. No fresh test results here; assume previous mega-commit is baseline.
- **Issues closed**: Sixel/Kitty/SSH marketplace/security items are marked done, but no independent verification has been run in this session.

## Gaps / Risks
- **Validation gap**: None of the new pipelines (images, chunked meshes, ratatui overlay, nav) were executed/compiled in this session. Risk of silent regressions despite closed issues.
- **Nav ↔ data-plane cohesion**: Need proof that focusable detection pulls from terminal chunks, ratatui surfaces, prompt markers, and images without duplicating regex passes or missing bounds.
- **Input/keymap ergonomics**: Keymaps are hardcoded in `NavInputRouter`; no user-configurable overrides surfaced yet. Mode transitions across plugin UIs (palette/tutorial/copy mode) need conflict tests.
- **Render layering**: Hint overlays and nav focus must coexist with images/Kitty/Sixel and post-process shaders; z-order and blur exclusion not yet validated.
- **Session/multiplexing awareness**: With tabs/panes and SSH domains closed as issues, nav state must be per-session/pane; need tests ensuring isolation and correct focus restore after switches.

## Recommendations / Next Steps
1) **Stabilization & CI smoke**  
   - Run workspace build and targeted suites: nav tests, golden tests, chunk tests, image pipeline tests. Add a short “nav smoke” job: start client headless, synthesize grid with links/prompts, exercise Hint mode, JumpPrompt, pane/tab cycling.
2) **Focusable generation audit**  
   - Ensure a single focusable detection path feeds `NavHint/NavFocus`: (a) terminal chunk scan (URLs/paths/emails), (b) prompt markers → `NavAction::JumpPrompt`, (c) ratatui surfaces (palette/menu/status), (d) image placements (optional click actions). Remove legacy regex in plugins.
3) **Render stack verification**  
   - Validate z-order: terminal text < images/kitty/sixel < nav hints/focus outlines < post-process. Add a test that renders images + hints to confirm hints remain crisp with blur off/on.
4) **Keymap/config surfacing**  
   - Expose nav style selection and custom bindings via config/Fusabi. Provide defaults (vimium/cosmos/spacemacs) but allow overrides per mode. Document conflicts with copy/search modes.
5) **Session/pane/tab isolation**  
   - Store NavState per session/tab; on tab/pane switch, restore last focus/hint set for that context. Add headless tests: switch panes while in Hint mode → ensure mode exits or state rehydrates safely.
6) **Plugin bridge alignment**  
   - Ensure plugin-host and Fusabi bridge expose nav operations: request hint mode, register focusable nodes (e.g., marketplace/context menu items), open URLs/files. Verify security prompts (GPG trust) are nav-focusable.
7) **Prompt-aware navigation**  
   - Wire JumpPrompt actions directly to `PromptMarkers`; scope hint generation to current prompt block when in PromptNav mode. Add tests for multi-prompt buffers.
8) **Performance/telemetry**  
   - Instrument nav pipelines: time to generate hints, chunk diff cost, and input→action latency. Surface in TELEMETRY docs and add counters to log/metrics for “hints generated”, “focus switches”, “nav errors”.

## Proposed Work Items (short-list)
- Add CI job: `cargo test -p scarab-client --lib navigation::tests headless_harness golden_tests` (or equivalents) + a scripted nav smoke test.
- Implement configurable keymaps (config -> NavInputRouter) and document.
- Add per-session NavState and tests covering pane/tab switches.
- Write integration tests that: render links/prompts/images; enter Hint mode; activate JumpPrompt; verify PTY input or focus change; assert hints drawn above images.
- Document nav developer guide: how to register focusables from terminal scan, ratatui surfaces, plugins; how to avoid z-order issues with shaders/images.
