# Scarab Terminal Emulator - Strategic Roadmap

> **Vision**: A next-generation, GPU-accelerated terminal emulator with split-process architecture, hybrid F# plugin system, and game-engine-driven UI

**Last Updated**: 2025-11-23
**Current Version**: 0.1.0-alpha
**Current Phase**: Phase 5 (Integration & Polish)

---

## 📍 Executive Summary

Scarab has **completed 4 major development phases** and is progressing through Phase 5. The core architecture is **production-ready**, and we have successfully implemented the foundational plugin ecosystem, including a **remote UI protocol** that allows daemon-side plugins to control the client interface.

**Completion Status**: ~80% of MVP features complete

---

## ✅ Completed Phases (Phases 1-4)

### Phase 1: Core Terminal Emulation ✅ COMPLETE
- VTE Parser, Text Rendering (cosmic-text + Bevy), IPC (Shared Memory)

### Phase 2: Plugin System & Extensibility ✅ COMPLETE
- Plugin API, Fusabi Adapter Layer
- **New**: Remote UI capabilities (`on_remote_command`, `get_commands`)

### Phase 3: Advanced Features ✅ COMPLETE
- Session Management, Configuration System
- **New Plugins**:
  - `scarab-nav`: Link hints and keyboard navigation
  - `scarab-palette`: Command palette with fuzzy search and dynamic command aggregation
  - `scarab-session`: Session management commands

### Phase 4: Production Hardening ✅ COMPLETE
- Performance optimization, testing, platform support

---

## 🚧 Current Phase: Phase 5 - Integration & Polish

**Duration**: Weeks 13-15 (Current)
**Status**: 🔄 In Progress
**Priority**: HIGH

### Objectives
1. **Core Plugin Ecosystem**: Implement standard plugins (`layout` pending).
2. **E2E Testing**: Verify daemon-client interaction with plugins.
3. **UI Polish**: Refine overlays and animations.

#### Workstream 5A: Core Plugins (In Progress)
- ✅ `scarab-nav`: Link hints implemented.
- ✅ `scarab-palette`: Command aggregation implemented.
- ✅ `scarab-session`: Basic session commands implemented.
- 🔄 `scarab-layout`: Tiling engine (Deferred to Phase 7 due to multi-pane dependency).

#### Workstream 5B: End-to-End Integration Testing (In Progress)
- Basic workflow tests implemented.
- Plugin interaction tests passing.

---

## 🔮 Near-Term Roadmap (Phases 6-7)

### Phase 6: Fusabi Runtime Integration
- Integration with `fusabi-vm` and `fusabi-frontend` crates once released.

### Phase 7: Feature Completeness & UX Polish
- Mouse support, Scrollback UI, Copy/Paste enhancements.
- **Multi-Pane Support & Layouts**: Server-side composition of multiple PTYs.

---

## 📝 Changelog

**2025-11-23 (Update 2)**
- ✅ Implemented `scarab-nav`, `scarab-palette`, and `scarab-session` plugins.
- ✅ Upgraded Protocol with `RemoteCommand` (DrawOverlay, ShowModal).
- ✅ Added `ClientRegistry` for daemon-to-client messaging.
- ✅ Updated Plugin API with `get_commands` and `on_remote_command`.

**2025-11-23 (Update 1)**
- ✅ Completed Phase 1-4
- ✅ Resolved GitHub Issues #1, #2, #3, #4