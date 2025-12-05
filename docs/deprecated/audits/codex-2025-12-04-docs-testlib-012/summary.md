# Scarab Technical Audit – Testlib Upgrade, Fusabi Impact, Docs/Test Gaps
Date: 2025-12-04  
Scope: Post `4fd380f` (issues #87/#88 implemented? version bumped to v0.2.0-alpha.2). Assess ratatui-testlib new features, Fusabi v0.32.x impact, and remaining gaps in docs/testing.

## Status Snapshot
- **ratatui-testlib upstream**: Now includes BevyTuiTestHarness/HeadlessBevyRunner/HybridBevyHarness with ECS queries, headless mode, SharedState helpers, and graphics coverage (Kitty/Sixel). Async harness exists. Open issues: #31 (nav helpers), #32 (async/wait_for).
- **Scarab usage**: Still pinned to ratatui-testlib 0.1.0 in Cargo.lock/Cargo.toml; tests and stubs still marked “blocked”. No ECS/SharedState/graphics/perf assertions in place.
- **Fusabi**: Latest release v0.32.0 (Fusabi-lang). Scarab still on fusabi-frontend/vm 0.17.0 in Cargo.lock. Issue #88 exists to upgrade/bind new ECS/nav APIs.
- **Docs/testing guides**: In-repo docs index/testing guide still missing (docs handled externally, but minimal index + testing commands needed locally).

## Gaps & Impact
- **Testlib gap**: We’re not consuming the new Bevy/Hybrid harness; significant test coverage (ECS/nav/graphics) is untapped.
- **Version drift**: Fusabi 0.17.0 vs 0.32.0 upstream—risk of missing APIs/fixes for ECS-safe bindings.
- **Docs/testing discoverability**: No local docs index/testing guide yet; ratatui-testlib and nav/golden/headless commands not centralized.

## Recommended Plan
1) **ratatui-testlib upgrade/use**
   - Bump dev-deps to latest ratatui-testlib; enable `bevy`, `headless`, `sixel`, `kitty`, `snapshot` features as needed.
   - Replace PTY-only smoke with Hybrid/Bevy harness tests:
     - Query ECS (NavState/NavHint/PromptMarkers/TerminalMetrics).
     - Read SharedState grid via provided helpers.
     - Assert graphics (Kitty/Sixel/iTerm2) placements/bounds.
     - Measure input→render latency if exposed.
   - Unignore a minimal subset gated by env (e.g., `SCARAB_TEST_RTL=1`); wire CI job (nightly/gated).
2) **Fusabi upgrade/refactor**
   - Bump fusabi-frontend/vm to 0.32.x across crates.
   - Wire new ECS-safe UI/nav bindings (per #88): overlays/focusables/status items, nav actions, keymap/style selection, capability limits.
   - Add integration tests: Fusabi script spawns focusable/overlay and triggers nav action safely.
   - Update in-repo docs to describe Fusabi APIs (link to external docs if needed).
3) **Docs/Testing quick fixes (no mdBook)**
   - Add `docs/README.md` index pointing to external docs + rustdoc; mark legacy nav/plugin docs deprecated or move to `docs/deprecated/`.
   - Add/refresh Testing guide with commands for nav/core, headless/golden, ratatui-testlib (env-gated) and justfile targets.

## Issues to File/Track (if not already)
- Scarab: ensure #87 (testlib upgrade/ECS/graphics tests) and #88 (Fusabi 0.32 upgrade + ECS/nav bindings) are implemented; if gaps remain after upgrade, file follow-ups for any missing assertions or binding holes.
- Fusabi: #228 (nav/keymap config APIs) to ensure stdlib supports nav actions/keymap selection with safety limits.

## Release Note
- v0.2.0-alpha.2 is current. Beta readiness hinges on completing the upgrades above, CI gates, and local docs/testing guides. Plan beta gate once testlib/Fusabi upgrades land and tests are green.
