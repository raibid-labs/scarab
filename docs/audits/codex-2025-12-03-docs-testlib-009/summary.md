# Scarab Technical Audit – Docs/Testability/ratatui-testlib (Post v0.2.0-alpha.0)
Date: 2025-12-03  
Scope: After `b9516ba` (release v0.2.0-alpha.0) and mdBook CI build addition. Focus on documentation gaps, ratatui-testlib usage, and remaining testability gaps.

## Current State
- **Docs portal**: mdBook scaffold now exists (`docs/book/`, `book.toml`, SUMMARY, setup) and CI builds mdBook. Good step toward a unified portal.
- **Docs sprawl**: Legacy nav/plugin docs still present without deprecation notes; no top-level `docs/README` index linking to mdBook or marking canonical sources.
- **Testing**: ratatui-testlib smoke tests remain `#[ignore]` and PTY-only; no CI job runs them. Headless/golden/nav tests not advertised in top-level TESTING instructions.
- **ratatui-testlib**: Only used in `crates/scarab-client/tests/ratatui_testlib_smoke.rs` (PTY passthrough, cursor, wait_for, basic nav hotkey send). No ECS/shared-memory/graphics assertions. Upstream issue #25 filed for hybrid harness; no open issues remain but no integration yet.
- **Release**: v0.2.0-alpha.0 tagged; roadmap toward beta not yet defined in docs.

## Gaps
- **Doc index**: Missing a `docs/README.md` that points to mdBook, legacy docs, and deprecates old nav/plugin references.
- **Unified testing guide**: No single place listing commands for nav-smoke, golden, headless harness, ratatui-testlib smoke.
- **ratatui-testlib depth**: Still limited to PTY-level. ECS/nav/SharedState/graphics/perf coverage blocked until upstream harness from #25 is delivered.
- **CI coverage**: ratatui-testlib smoke not in CI; risk of drift.

## Recommendations
1) **Docs polish**
   - Add `docs/README.md` as index: link to mdBook, legacy docs, and mark deprecated nav/plugin docs. Include how to build mdBook (`mdbook build`, `mdbook test`, linkcheck).
   - In mdBook, add a short page explaining doc sources (mdBook vs rustdoc vs legacy) and how to contribute.
2) **Testing surface**
   - Add a top-level TESTING guide (or update existing) with explicit commands: `cargo test -p scarab-client --lib navigation::tests`, headless harness, golden tests, ratatui-testlib smoke (`--ignored`), and justfile shortcuts.
   - Enable a CI job to run ratatui-testlib smoke in a minimal mode (or gated on env) so regressions are caught when PTY is available.
3) **ratatui-testlib integration plan**
   - When upstream resolves #25 (hybrid harness), extend tests to: query NavState/NavHint/PromptMarkers, read SharedState grid, assert Sixel/Kitty/iTerm2 placements, and record input→render latency. Remove or replace TODO blocks in `ratatui_testlib_smoke.rs` with runnable tests.
4) **Release path**
   - Document path to `v0.2.0-beta`: criteria (CI green on smoke/golden/nav/ratatui-testlib, docs index live, plugin ABI frozen), then `v0.2.0` once release checklist passes.

## ratatui-testlib Upstream Status
- No open issues; new request filed earlier (#25) for Bevy+Scarab hybrid harness with ECS/SharedState/graphics/perf hooks. Awaiting implementation; no further upstream action needed until harness lands.
