# Scarab Technical Audit – Docs, Testability, ratatui-testlib, Plugins, Release
Date: 2025-12-02  
Scope: Post latest nav lifecycle/docs iteration. Focus on polish (docs/in-code docs), testability (incl. ratatui-testlib), plugin inventory, repo org, and release tagging.

## Findings
### Testability / ratatui-testlib
- Scarab still **does not use `ratatui-testlib`** in code or tests, despite being listed as a dependency in `crates/scarab-client` and `crates/scarab-daemon`.
- `ratatui-testlib` now has no open issues; README still lists mouse/resize/Kitty as post-MVP. Bevy/ratatui harness claims support, but Scarab hasn’t integrated it.
- Headless tests in Scarab rely on internal harness; no cross-validation with ratatui-testlib for PTY-level fidelity or Sixel/Kitty paths.

### Docs & In-Code Docs
- Navigation docs improved (`docs/navigation.md`), but overall docs are **fragmented**: multiple legacy plugin/nav docs, no central index, and no single “developer portal”.
- No pipeline tying `rustdoc` (code comments) to the `docs/` site; users must read code or markdown separately.
- Testing docs lack a clear “how to run nav/pane/image tests” entrypoint; no mention of ratatui-testlib usage.
- Repo docs structure is sprawling (numerous plugin-related files, legacy guides); hard to find canonical sources.

### Plugin Inventory (high level)
- Current plugins/crates: `scarab-nav`, `scarab-palette`, `scarab-session`, `scarab-platform`, `scarab-clipboard`, `scarab-themes`, plus examples and `scarab-atuin`.
- Planned (from roadmaps/issues): context menus, plugin marketplace UI, GPG verification, SSH domains, graphics protocols.
- Missing/high-value plugins: performance HUD (FPS/cache hits), telemetry dashboard, test harness plugin (record/replay), accessibility (screen reader export), diagnostics (log capture per pane), and a Bevy inspector overlay tailored for Scarab UI.

### Repo Organization
- Crates are cleanly separated; docs are not. No `docs/README` index; legacy nav/plugin docs coexist with new ECS-native flow.
- Tests live in multiple places (headless harness, golden tests), but no top-level test runner instructions in README/TESTING guides.

### Release Tagging
- Still on `v0.1.0-alpha.X` despite major architectural milestones (multiplexing, images, ECS nav, plugin host). Stability is improving; consider moving to `v0.2.0-alpha` after a stabilization sprint, then `v0.2.0-beta` once a CI gate of smoke tests is green.

## Recommendations
### A) Integrate ratatui-testlib (or prove we don’t need it)
1. Add dev-dependency path override to latest ratatui-testlib checkout and write a small suite under `crates/scarab-client/tests/ratatui_testlib_smoke.rs`:
   - Spawn headless client via ScheduleRunner, pipe PTY output through ratatui-testlib harness, assert on grid text, Sixel/Kitty (if supported), and nav hotkeys producing expected sequences.
   - Use BevyTuiTestHarness to query ECS (focusable counts, prompt markers) for nav correctness.
2. If a gap is found (e.g., kitty graphics or Bevy ECS snapshot missing), file an upstream issue with concrete repro. (No issue filed now since no blocker identified.)

### B) Docs/Portal Strategy (proposal)
1. **Tooling**: Use `mdBook` as the unifying shell with:
   - `mdbook-rustdoc` (or `mdbook-embeddoc`) to surface selected `rustdoc` excerpts (public API, plugin host, nav types).
   - `mdbook-linkcheck`, `mdbook-mermaid`, `mdbook-katex` as needed; CI to build/book + linkcheck.
   - Keep `cargo doc` for full API; link from mdBook.
2. **Structure**: Create `docs/README.md` as an index, add `docs/book` for mdBook sources:
   - User Guide (install, keymaps, navigation, plugins usage)
   - Developer Guide (ECS architecture, nav integration, plugin host, ratatui bridge)
   - Testing Guide (headless harness, ratatui-testlib, golden tests)
   - Reference (config schemas, IPC, protocol)
3. **Inspired by vector.dev**: They use generated reference docs (CUE) + site content. We can mirror by:
   - Generating config/IPC reference from schemas (`serde` structs → `schemars` → JSON → mdBook pages).
   - Auto-generating plugin API docs from `rustdoc-json` → `cargo doc` extracts.
4. Add a `docs/CONTRIBUTING-DOCS.md` describing how to update docs, build mdBook, and run linkcheck.

### C) Documentation Cleanup
1. Mark legacy nav/plugin docs as deprecated; point to the ECS-native navigation guide.
2. Add a single “Navigation Developer Guide” and “Navigation User Guide” under `docs/navigation/` and link from README.
3. Add `TESTING.md` (or update existing) with commands for: `cargo test -p scarab-client --lib navigation::tests headless_harness golden_tests`, and a `just nav-smoke` script.

### D) Plugins Plan
- **Short term**: Harden existing plugins (nav, palette, session) against new nav lifecycle and prompt markers; add telemetry HUD plugin; diagnostics/recorder plugin for replay.
- **Medium**: Context menu plugin, marketplace UI plugin (with GPG verification surfaces), SSH domains plugin, graphics viewer plugin (Kitty/Sixel overlay inspector).
- Consider splitting plugins into a separate repo only after a stable plugin ABI/API; until then, keep in mono-repo to avoid version skew.

### E) Release Guidance
- Target `v0.2.0-alpha.0` after a stabilization sprint focused on:
  - CI green for smoke (nav, images, chunked render), ratatui-testlib smoke, and golden tests.
  - Doc portal skeleton (mdBook scaffold + index + link to existing docs).
- Move to `v0.2.0-beta.1` once plugin ABI/API is frozen and nav/plugin docs are consolidated; then `v0.2.0` on a tagged green build with release checklist updated.

### F) Issue to File on ratatui-testlib (if adopted)
- None filed now; first integrate and only file if specific gaps appear (e.g., Kitty graphics validation or Bevy ECS snapshotting missing). If needed, propose: “Add Kitty graphics assertions and ECS snapshot diff helper” with repro from Scarab’s test harness.
