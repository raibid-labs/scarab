# Scarab Technical Audit – Navigation/Docs/Repo Org (Post-Lifecycle Integration)
Date: 2025-12-02  
Scope: Review after `7ad4817` (pane-event nav lifecycle) and `a22d537` (nav docs updates). Focus on navigation state, plugin API, documentation gaps, and repo organization.

## What Landed
- **Nav lifecycle on pane events** (`7ad4817`): Navigation state now reacts to pane creation/destruction/switch events (saving/restoring, clearing on destroy).
- **Docs update** (`a22d537`): Navigation doc expanded to include plugin bridge, per-pane behavior, best practices.
- **Navigation architecture**: ECS-native nav modes/actions, per-pane isolation, prompt markers, plugin bridge exposure remain in place.

## Current Risks / Gaps
- **Unvalidated runtime**: No build/tests run here. Need to confirm lifecycle handlers actually fire in-app and no stale hints remain.
- **Documentation fragmentation**: Navigation guidance exists in `docs/navigation.md`, but developer-facing “how to integrate” is scattered (plugin-host docs elsewhere, ratatui bridge in module docs). No consolidated quickstart/checklist.
- **Repo sprawl**: `docs/` contains many legacy/duplicate navigation/plugin docs (plugin-* files, issues/21-resolve-nav-redundancy.md). Users may be confused about current canonical flow (ECS nav vs legacy plugin overlays).
- **Config surface**: Keymap/style overrides, per-pane nav toggles, and plugin capability limits are not clearly documented for users/admins.
- **Testing visibility**: No simple entry point to run nav/pane lifecycle tests; golden/headless suites aren’t clearly referenced from README/TESTING guides.

## Recommendations / Next Steps
1) **Validation**
   - Run targeted tests: nav unit/integration, pane-switch lifecycle, headless harness/golden tests. Add a minimal “nav smoke” command in `justfile` or `scripts/` to exercise hint mode, pane switch, and prompt jump headlessly.
2) **Docs consolidation**
   - Create a single “Navigation Developer Guide” under `docs/navigation/` with:
     - Quickstart (enable plugin, set keymap, enter hint mode)
     - Per-pane behavior and lifecycle
     - Plugin bridge usage (register focusables, trigger actions)
     - Ratatui overlay integration
     - Config options (keymap/style overrides, capability limits)
   - Add a short “Navigation User Guide” for end users (keybindings, modes).
   - Cross-link from README and `docs/navigation.md` to the canonical guides; mark legacy nav docs as deprecated where applicable.
3) **Repo organization**
   - Add a `docs/README.md` or index pointing to canonical docs (navigation, plugin host, ratatui bridge, testing). Prune/flag obsolete nav/plugin documents (e.g., legacy overlay flow).
4) **Config & safety**
   - Document keymap overrides (config/TOML/Fusabi) and plugin capability limits for nav APIs; ensure defaults are sane and noted.
5) **Telemetry**
   - Add nav lifecycle counters (pane switch rehydrate time, cleared hints, plugin-registered focusables) to TELEMETRY docs and ensure the instrumentation exists.

## Repo Structure Notes
- Top-level crates are well-named; navigation lives in `crates/scarab-client/src/navigation` with input in `input/nav_input.rs`.
- Docs are extensive but sprawling; navigation, plugin host, and ratatui bridge guidance are spread across multiple files. A curated index and deprecation notes will reduce confusion.
