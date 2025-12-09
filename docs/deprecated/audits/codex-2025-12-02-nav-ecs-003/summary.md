# Scarab Technical Audit – Navigation/Plugins (Post-Iteration)
Date: 2025-12-02  
Scope: Review latest changes (`804232e` per-pane NavState, `26bb379` plugin nav API). Focus on navigation isolation, plugin surfacing, and remaining validation gaps.

## What Landed
- **Per-pane NavState isolation** (#39): Navigation state is now stored per pane/tab (commit `804232e`). Focus history/mode stack should no longer leak across panes.
- **Plugin bridge navigation API** (#42): Navigation operations are exposed to the plugin host/Fusabi bridge (enter hint mode, register focusables, trigger nav actions).

## Current Risks / Gaps
- **No runtime verification**: These changes haven’t been compiled/run here. Need tests to confirm state isolation and plugin-driven nav actions.
- **State rehydration**: On tab/pane switch, restoring focus/hints may race with chunk updates/prompt markers. Risk of stale or missing focusables.
- **Plugin API safety**: Nav API exposure could allow plugins to spam focusables or force actions; need capability checks/rate limiting.
- **Telemetry/observability**: No metrics confirming per-pane state swaps or plugin-driven nav actions; hard to diagnose regressions.
- **Z-order/render**: Still unverified that hint overlays remain above images/kitty/sixel/post-process shaders after nav actions from plugins.
- **Config UX**: Keymap/behavior overrides still not exposed to users; plugin-triggered modes could conflict with local keymaps.

## Recommendations / Next Steps
1) **Validation & Tests**
   - Add headless integration tests:
     - Switch panes while in Hint mode; assert NavState is isolated and restored correctly.
     - Plugin-sourced focusables: inject via bridge, enter Hint mode, activate action; assert PTY input or focus change.
     - Render layering: images+kitty/sixel + hints; ensure hints draw on top.
   - Run workspace build + targeted suites (nav tests, golden tests, headless harness).
2) **State lifecycle**
   - Ensure pane/tab switch emits events consumed by nav to save/restore focusables/hints after chunk refresh.
   - Clear hints/focus when pane is destroyed; avoid dangling entities.
3) **Plugin guardrails**
   - Add capability flags/quota for plugin nav APIs (max focusables, rate limit actions).
   - Validate inputs: bounds within viewport, action types allowed.
4) **Keymap/config**
   - Surface nav style/bindings in config/Fusabi; document how plugin-triggered nav coexists with user keymaps.
5) **Telemetry**
   - Counters for: pane switch rehydrate time, focusable counts, plugin nav actions accepted/rejected, hint generation latency.
   - Log warnings on dropped/stale focusables during pane swaps.
6) **Docs**
   - Update nav developer guide to include plugin bridge usage, per-pane behavior, and best practices for registering focusables.
