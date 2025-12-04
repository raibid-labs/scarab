# Scarab Technical Audit – Docs/Testability/ratatui-testlib/Plugins/Fusabi/Release
Date: 2025-12-04  
Scope: Post v0.2.0-alpha.0. Adjusted to exclude mdBook work (handled elsewhere). Focus on doc alignment, testing (ratatui-testlib), plugins/Fusabi, release path.

## Documentation (no mdBook asks)
- External docs effort lives in `~/raibid-labs/docs`; don’t duplicate that here.
- Add a minimal **docs index** in-repo (`docs/README.md`) pointing to: external docs site, rustdoc, and marking deprecated legacy nav/plugin docs.
- Add deprecation notes to legacy nav/plugin docs or move them under `docs/deprecated/`.
- Provide a concise **Testing guide** in-repo listing commands for nav/core tests, headless/golden, and ratatui-testlib smoke (with any env gating) plus justfile targets.

## ratatui-testlib Usage & Next Steps
- Current usage: `crates/scarab-client/tests/ratatui_testlib_smoke.rs` + README; PTY-only, many `#[ignore]`, no ECS/SharedState/graphics/perf assertions.
- Upstream: Async harness landed; open issues #25 (hybrid ECS harness), #31 (nav helpers), #32 (async/wait_for). No other open issues.
- Actions:
  1) Update smoke tests to use `AsyncTuiTestHarness` where appropriate; unignore a minimal subset, gated by env (`SCARAB_TEST_RTL=1`).
  2) Add graphics protocol checks (Sixel/Kitty/iTerm2) to ensure sequences don’t crash and are detected.
  3) Add a CI job (nightly/gated) to run ratatui-testlib smoke when PTY is available.
  4) When #25 lands, extend tests to query ECS (NavState/NavHint/PromptMarkers), read SharedState, validate graphics placements, and capture input→render latency; remove TODO blocks accordingly.

## Testability Gaps
- No ECS/resource access via ratatui-testlib yet; no SharedState/graphics/perf assertions.
- ratatui-testlib smoke not exercised in CI; risk of drift.

## Plugins & Fusabi
- **Current plugins**: nav, palette, session, platform, clipboard, themes, atuin, core plugin API.
- **Should build/finish**: context menu, marketplace UI + GPG trust prompts, SSH domains, graphics inspector, telemetry HUD, diagnostics recorder/replay, accessibility/export, Bevy UI inspector, performance HUD.
- **Fusabi needs (file issues if missing)**:
  - ECS-safe bindings to spawn overlays/focusables/status items.
  - Navigation actions (enter hint mode, register focusables, prompt jumps) exposed to scripts.
  - Keymap/style selection via Fusabi config.
  - Capability/quotas for plugin-driven UI/nav to avoid abuse.
- Keep plugins in monorepo until plugin ABI/API stabilizes (post-beta).

## Release Roadmap
- **v0.2.0-beta criteria**:
  - CI green on nav/core, headless/golden, ratatui-testlib smoke (subset), and lint/build.
  - Docs index present; legacy docs flagged; external docs site referenced.
  - Plugin ABI/API frozen for beta; keymaps/config documented.
  - Telemetry counters for nav/render/image present and documented.
- **Beyond beta (toward 0.3.x)**:
  - ratatui-testlib hybrid harness integrated; ECS/nav/graphics/perf tests live.
  - Marketplace/GPG, SSH domains, graphics inspector plugins shipped.
  - Performance budgets enforced (input→render, chunk rebuild).
  - Accessibility/export plugin delivered.

## Action Items
1) Add `docs/README.md` index pointing to external docs site + rustdoc; mark legacy nav/plugin docs deprecated or move to `docs/deprecated/`.
2) Add/refresh a Testing guide with commands for nav/core, headless/golden, ratatui-testlib (with env gating) and justfile targets.
3) Update ratatui-testlib smoke to use Async harness, add graphics protocol checks; unignore minimal set; add CI job (gated) to run them.
4) Track ratatui-testlib #25/#31/#32; expand tests to ECS/SharedState/graphics/perf when available.
5) File Fusabi issues if ECS-safe UI/nav bindings and keymap/config exposure are still missing.
6) Define v0.2.0-beta gate (above) and plan stabilization sprint.
