# WezTerm Parity Initiative

**Date:** December 2, 2025
**Author:** Claude (Opus 4.5)
**Status:** Research & Design Complete

## Executive Summary

This document outlines a comprehensive plan to achieve feature parity between Scarab's Fusabi scripting layer and WezTerm's Lua configuration API. The analysis identifies six parallelizable workstreams that can be executed concurrently by different agents or developers.

## Background

WezTerm represents the current gold standard for programmable terminal emulators. Its power comes from exposing a **full object model** to users, not just configuration values. Users can:

- Hook into virtually any terminal event
- Access live `Window`, `Pane`, and `Tab` objects with rich methods
- Programmatically build UI elements (status bars, tab titles)
- Create modal keyboard layers (copy mode, resize mode)
- Render images via multiple protocols (iTerm2, Kitty, Sixel)

Scarab's current Fusabi implementation is **hook-based** (`OnOutput`, `OnInput`) rather than **object-oriented**. This limits what users can accomplish without modifying Rust code.

## Gap Analysis Summary

| Feature Area | WezTerm | Scarab Current | Gap Severity |
|--------------|---------|----------------|--------------|
| Object Model | 80+ methods across Window/Pane/Tab | Stateless `PluginContext` | **Critical** |
| Event System | 12+ window events + custom | 8 hook types | **High** |
| Status Bar | `wezterm.format` + set_*_status | Hardcoded UI | **High** |
| Key Tables | Modal layers, leader keys | Basic keybindings | **Medium** |
| Image Protocols | iTerm2, Kitty, Sixel | None | **Medium** |
| Copy Mode | Full vim-like selection | Basic mouse selection | **Low** |

## Workstreams Overview

This initiative is divided into six parallelizable workstreams:

### WS-1: Fusabi Object Model Infrastructure
**Priority:** P0 (Critical Path)
**Estimated Complexity:** High
**Dependencies:** None
**Document:** [01-object-model.md](./01-object-model.md)

Design and implement the infrastructure to expose Rust state (Bevy entities, daemon sessions) to Fusabi as callable objects. This is the foundation all other features build upon.

### WS-2: Rich Event System
**Priority:** P0 (Critical Path)
**Estimated Complexity:** Medium
**Dependencies:** WS-1 (partial)
**Document:** [02-event-system.md](./02-event-system.md)

Expand the event system from 8 hook types to 20+ granular events matching WezTerm's capabilities. Enable custom event registration.

### WS-3: Status Bar Rendering API
**Priority:** P1 (High Value)
**Estimated Complexity:** Medium
**Dependencies:** WS-1, WS-2
**Document:** [03-status-bar-api.md](./03-status-bar-api.md)

Create a styled text API (`Format` function) and `OnStatusUpdate` hooks that allow plugins to programmatically define status bar content.

### WS-4: Key Tables & Modal Editing
**Priority:** P1 (High Value)
**Estimated Complexity:** Medium
**Dependencies:** WS-2
**Document:** [04-key-tables.md](./04-key-tables.md)

Implement named key tables, leader key support, and the key table activation stack for modal workflows.

### WS-5: Image Protocol Support
**Priority:** P2 (Feature Parity)
**Estimated Complexity:** High
**Dependencies:** None (parallel)
**Document:** [05-image-protocols.md](./05-image-protocols.md)

Implement iTerm2, Kitty, and Sixel image protocols in the VTE parser and Bevy renderer.

### WS-6: Copy Mode & Advanced Selection
**Priority:** P2 (Feature Parity)
**Estimated Complexity:** Medium
**Dependencies:** WS-4
**Document:** [06-copy-mode.md](./06-copy-mode.md)

Implement vim-like copy mode with configurable key bindings and semantic zone selection.

## Parallelization Strategy

```
Week 1-2:
├── WS-1: Object Model (starts immediately)
├── WS-5: Image Protocols (fully parallel, no deps)
└── WS-2: Event System (can start after WS-1 scaffolding)

Week 3-4:
├── WS-1: Object Model (continues)
├── WS-3: Status Bar API (after WS-1/WS-2 basics)
├── WS-4: Key Tables (after WS-2 events)
└── WS-5: Image Protocols (continues)

Week 5-6:
├── WS-3: Status Bar API (continues)
├── WS-4: Key Tables (continues)
├── WS-6: Copy Mode (after WS-4)
└── Integration & Testing
```

## Design Principles

1. **Minimize Breaking Changes**: Extend existing APIs rather than replacing them
2. **Incremental Delivery**: Each workstream should deliver usable features independently
3. **Type Safety**: Leverage Rust's type system for Fusabi<->Rust boundaries
4. **Performance First**: Object proxies should be lightweight (handle-based, not cloning)
5. **WezTerm Compatibility**: Where possible, mirror WezTerm's API naming for familiarity

## File Index

| File | Purpose |
|------|---------|
| [01-object-model.md](./01-object-model.md) | Fusabi object model architecture |
| [02-event-system.md](./02-event-system.md) | Expanded event system design |
| [03-status-bar-api.md](./03-status-bar-api.md) | Status bar rendering API |
| [04-key-tables.md](./04-key-tables.md) | Modal key tables system |
| [05-image-protocols.md](./05-image-protocols.md) | Image protocol implementation |
| [06-copy-mode.md](./06-copy-mode.md) | Copy mode and selection |
| [07-workstreams.md](./07-workstreams.md) | Detailed workstream breakdown |

## Success Criteria

1. Users can write `.fsx` scripts that access `Window`, `Pane`, `Tab` objects
2. Custom status bars can be defined entirely in Fusabi (no Rust changes)
3. Modal editing modes (resize, copy) work like WezTerm
4. At least one image protocol renders images inline
5. All WezTerm window events have Scarab equivalents

## References

- [WezTerm Lua API](https://wezterm.org/config/lua/)
- [WezTerm Window Events](https://wezterm.org/config/lua/window-events/)
- [WezTerm Key Tables](https://wezterm.org/config/key-tables.html)
- [WezTerm Image Protocols](https://wezterm.org/imgcat.html)
- [Scarab Gap Analysis](../analysis/wezterm-gap-analysis.md)
- [Claude Instructions](../developer/CLAUDE_WEZTERM_PARITY_INSTRUCTIONS.md)
