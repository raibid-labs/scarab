# Scarab Technical Audit – Docs Gaps, ratatui-testlib Usage, Plugins, Release
Date: 2025-12-02  
Scope: Post-iteration (commit `a0578ff`); focus on documentation polish, testability (ratatui-testlib), plugin inventory, repo organization, and release guidance.

## Documentation Gaps
- **Fragmented navigation docs**: `docs/navigation.md` exists, but legacy nav/plugin docs remain; no single index or developer/user split. No doc that walks “how to integrate nav focusables via plugin host or ratatui bridge”.
- **Test docs scattered**: Headless/golden/nav and ratatui-testlib smoke tests aren’t linked from a central TESTING guide. No one-line commands in README/justfile.
- **Code ↔ docs bridge missing**: No system to surface `rustdoc` alongside `docs/`. No mdBook or site tying in API + guides. Config/IPC references not generated from schemas.
- **Repo doc sprawl**: Many plugin/reg/legacy files remain without deprecation notes; no `docs/README` index to canonicalize sources.

## ratatui-testlib Usage and Gaps
- Usage exists only in `crates/scarab-client/tests/ratatui_testlib_smoke.rs` (ignored tests) and `README_RATATUI_TESTLIB.md`. Tests are PTY-only: echo, grid text, cursor, wait_for, basic nav hotkey send. They explicitly note that BevyTuiTestHarness is a placeholder and can’t query ECS or shared memory.
- No integration with Bevy ECS via ratatui-testlib; all ECS/nav checks are marked “blocked” or TODO. No Sixel/Kitty assertions through testlib; relies on internal harness.
- Upstream ratatui-testlib shows no open issues; Bevy integration still not delivered, so Scarab remains limited to PTY-level validation.

### Missing ratatui-testlib Functionality for Scarab
1) **Bevy ECS harness**: Need a working `BevyTuiTestHarness` to query resources/components (NavState, NavHint, PromptMarkers, TerminalMetrics) and drive Bevy schedules without a window.
2) **Hybrid daemon+client harness**: Ability to run daemon in PTY and client in-process, sharing SharedState, to assert grid contents and nav overlays end-to-end.
3) **Shared memory access**: Direct read of `scarab-protocol::SharedState` from harness for grid/metrics assertions.
4) **Graphics protocols**: Assertions for Kitty graphics (now implemented in Scarab), Sixel, iTerm2; verify placement/bounds.
5) **Performance benchmarking hooks**: Measure latency from PTY input to rendered frame (or buffer) for regression detection.
6) **Nav-specific assertions**: Detect mode changes, hint counts, and prompt anchor jumps without relying on parsing PTY output.

## Proposed Issue for ratatui-testlib
File on `raibid-labs/ratatui-testlib`:
```
Title: Bevy+Scarab hybrid harness with ECS/query + SharedState access
Summary: Scarab needs to validate nav/graphics end-to-end. Current TuiTestHarness is PTY-only; BevyTuiTestHarness is a stub. Request:
- An in-process harness that can run Bevy schedules (with or without bevy_ratatui), and optionally launch a child daemon in a PTY.
- APIs to query resources/components (e.g., NavState/NavHint/PromptMarkers/TerminalMetrics).
- Optional SharedState reader for mmap-backed grid assertions.
- Graphics assertions for Sixel/Kitty/iTerm2 placements.
- Timing hooks to measure input→frame latency.
Add a minimal example and CI-friendly headless mode.
```

## Plugin Inventory (current/planned/should-build)
- **Current**: nav, palette, session, platform, clipboard, themes, atuin integration, core plugin API examples.
- **Planned** (from issues/roadmaps): context menus, plugin marketplace UI + GPG, SSH domains, kitty/sixel graphics support (now coded), post-process shaders, deep shell integration.
- **Should build**: telemetry HUD (FPS/cache/hint counts), diagnostics recorder/replay, accessibility (screen reader/export), graphics viewer/inspector, Bevy UI inspector overlay, marketplace browser with trust prompts.
- Repository split: keep plugins in monorepo until plugin ABI/API is frozen; splitting now risks version skew while APIs churn.

## Repo Organization
- Crate layout is fine; docs need an index and deprecation markers for legacy nav/plugin docs.
- Tests: add top-level TESTING guide and justfile targets for nav-smoke, golden, ratatui-testlib, headless harness.

## Release Guidance
- Move off perpetual `v0.1.0-alpha.X`: after a stabilization pass (green smoke/golden/nav/ratatui-testlib tests, doc index + mdBook scaffold), tag `v0.2.0-alpha.0`.
- Promote to `v0.2.0-beta.1` when plugin ABI/API is stable and docs consolidated; then `v0.2.0` once release checklist passes on CI.

## Action Items
1) File the ratatui-testlib issue above (hybrid harness + ECS/SharedState + graphics + perf hooks).
2) Expand ratatui-testlib usage: enable/ungate `ratatui_testlib_smoke.rs`, add ECS/nav assertions when upstream harness arrives; add Kitty/Sixel cases.
3) Docs: create `docs/README.md` index; scaffold mdBook (or similar) with rustdoc links; consolidate nav docs (user + developer) and testing guide with commands.
4) Add justfile scripts for nav-smoke, golden, ratatui-testlib tests.
5) Plan v0.2.0-alpha stabilization sprint with CI gates and doc portal skeleton.
