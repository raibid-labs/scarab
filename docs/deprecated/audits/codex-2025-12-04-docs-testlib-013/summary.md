# Scarab Technical Audit – Testlib/Fusabi Upgrades and Testing Gaps
Date: 2025-12-04  
Scope: Post `f36e92f` (docs index/deprecations) and `0cd2dfe` (Fusabi docs). Validate actual dependency state, testing coverage, and remaining gaps.

## Findings
- **ratatui-testlib**: Still pinned to `0.1.0` in Cargo.lock/Cargo.toml. Tests/stubs remain PTY-only and marked blocked. Upstream now offers Bevy/Hybrid/Headless harnesses with ECS queries, SharedState helpers, and graphics support. Open upstream issues: #31 (nav helpers), #32 (async/wait_for).
- **Fusabi**: Scarab still on `fusabi-frontend/vm 0.17.0` per Cargo.lock. Latest Fusabi release is v0.32.x. ECS-safe UI/nav bindings are documented, but not actually wired in code yet.
- **Testing guide/CI**: Docs index added, but testing guide does not cover ratatui-testlib harness usage or gated commands; no CI job detected for ratatui-testlib smoke. Navigation/golden/headless commands are not consolidated.
- **Harness stubs**: `crates/scarab-client/tests/bevy_harness_stubs.rs` and `ratatui_testlib_smoke.rs` still reference “blocked” harness, despite upstream availability.

## Impact
- We are missing high-value test coverage (ECS/nav/SharedState/graphics/perf) that is now possible with latest ratatui-testlib.
- Fusabi feature gap risks: scripts cannot use new ECS-safe UI/nav APIs until we upgrade, so plugin-side capabilities remain limited.
- Documentation/testing discoverability remains partial; ratatui-testlib path is undocumented locally.

## Recommendations
1) **Upgrade ratatui-testlib to latest** (with bevy/headless/sixel/kitty features) and port tests to Hybrid/Bevy harness:
   - Add ECS assertions (NavState/NavHint/PromptMarkers/TerminalMetrics).
   - Use SharedState helpers for grid checks.
   - Add graphics bounds checks (Kitty/Sixel/iTerm2) and latency assertions if available.
   - Unignore a gated subset (env: `SCARAB_TEST_RTL=1`) and add a CI job (gated/nightly).
2) **Upgrade Fusabi to v0.32.x**:
   - Wire ECS-safe UI/nav bindings (overlays/focusables/nav actions/keymap selection with capability limits).
   - Add integration tests with Fusabi scripts exercising these bindings.
   - Update in-repo docs to describe available Fusabi APIs (linking to external site as needed).
3) **Testing guide refresh**:
   - Add ratatui-testlib usage (commands, env gate) and consolidate nav/golden/headless commands; add justfile targets.
   - Note CI gating for ratatui-testlib smoke.
4) **Clean up stubs**:
   - Remove or implement “blocked” harness stubs now that upstream harnesses exist.

## Issues to (Re)open/track
- Scarab #87: ratatui-testlib upgrade + ECS/graphics/perf tests (still unimplemented; dependency pinned at 0.1.0).
- Scarab #88: Fusabi 0.32 upgrade + ECS/nav bindings (still on 0.17.0).
- Scarab #82: Testing guide/CI/just targets missing ratatui-testlib and consolidated commands.
