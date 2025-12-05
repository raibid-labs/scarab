# Scarab Technical Audit – Docs/Testability/ratatui-testlib (Post-Audit-007 Completion)
Date: 2025-12-02  
Scope: After `b9516ba` (“complete audit 007”) and prior issue closures. Focus on doc polish, testability, ratatui-testlib usage, and residual gaps.

## Current State
- **Docs**: Audit 007 called for an index and mdBook portal; no new doc index or mdBook scaffold is present. Navigation doc exists; legacy nav/plugin docs remain. Testing instructions are still scattered.
- **ratatui-testlib**: Scarab uses it only in `crates/scarab-client/tests/ratatui_testlib_smoke.rs` (ignored tests), PTY-only. Bevy/ECS/SharedState/graphics assertions remain TODO. Upstream issue #25 filed for a hybrid harness; no integration updates yet.
- **Plugins/Release**: Plugins remain in-monorepo; release tagging still at alpha line (no new tags beyond prior guidance).

## Gaps (unchanged or newly observed)
- **Doc consolidation** still missing: No `docs/README` index; no mdBook or combined API+guides; legacy nav/plugin docs not marked deprecated.
- **Test discoverability**: No single TESTING guide with commands/just targets for nav-smoke, golden, headless, ratatui-testlib.
- **ratatui-testlib depth**: Still PTY-level only; ECS/nav/graphics/perf assertions blocked pending upstream harness (#25).
- **CI confidence**: No evidence of CI jobs running the ratatui-testlib smoke suite; tests remain `#[ignore]`.

## Recommendations / Next Steps
1) **Docs**
   - Add `docs/README.md` index pointing to canonical nav/user/dev/testing/plugin references; mark legacy nav/plugin docs as deprecated.
   - Scaffold mdBook (or similar) with sections for User Guide, Developer Guide (ECS/nav/plugin host/ratatui bridge), Testing (headless/golden/ratatui-testlib), and Reference (config/IPC). Link to rustdoc.
   - Add a contributor doc for building docs (mdBook build/linkcheck) and how code changes should update docs.
2) **Testing**
   - Unignore/run ratatui-testlib smoke tests in CI (with gating to skip if daemon binary absent).
   - Add justfile/script entries: `nav-smoke`, `golden`, `headless`, `ratatui-testlib-smoke`.
   - When upstream resolves #25, add ECS/nav/SharedState/graphics assertions and migrate TODOs in `ratatui_testlib_smoke.rs` to runnable tests.
3) **ratatui-testlib integration**
   - Track upstream issue #25; once available, extend tests to:
     - Query NavState/NavHint/PromptMarkers.
     - Read SharedState grid directly for assertions.
     - Validate Sixel/Kitty/iTerm2 placements and bounds.
     - Capture input→render latency.
4) **Release**
   - Proceed with v0.2.0-alpha after a stabilization pass: CI green on smoke/golden/nav/ratatui-testlib (where applicable) and doc index/mdBook scaffold merged.
5) **Repo hygiene**
   - Add deprecation notes to legacy nav/plugin docs; prune or move outdated files to a `docs/deprecated/` folder for clarity.
