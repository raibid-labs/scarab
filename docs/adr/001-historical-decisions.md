# Architectural Decision Records - Historical Summary

**Document Purpose**: This document consolidates key architectural decisions made during Scarab's early development phases (2025-11 through 2025-12). It serves as a historical record extracted from implementation plans, execution summaries, and technical reports that have been archived.

**Last Updated**: 2025-12-15

---

## Table of Contents

1. [Core Architecture Decisions](#1-core-architecture-decisions)
2. [Plugin System Architecture](#2-plugin-system-architecture)
3. [Migration to Bevy 0.15](#3-migration-to-bevy-015)
4. [Navigation and UI System](#4-navigation-and-ui-system)
5. [Performance Optimizations](#5-performance-optimizations)
6. [Testing Strategy](#6-testing-strategy)
7. [Distribution and Packaging](#7-distribution-and-packaging)

---

## 1. Core Architecture Decisions

### ADR-001: Split-Process Architecture

**Date**: Early 2025-11
**Status**: Accepted and Implemented

**Context**: Traditional terminal emulators use a monolithic architecture where the GUI and terminal state live in the same process.

**Decision**: Scarab uses a split-process architecture with:
- **Daemon** (scarab-daemon): Headless server that owns PTY processes and terminal state
- **Client** (scarab-client): GPU-accelerated GUI built with Bevy
- **IPC**: Zero-copy shared memory with lock-free synchronization

**Rationale**:
- **Resilience**: Client crashes don't kill terminal sessions
- **Flexibility**: Multiple clients can attach to the same daemon
- **Performance**: Zero-copy memory sharing eliminates serialization overhead
- **Remote Access**: Foundation for remote terminal access (future)

**Consequences**:
- Increased complexity in state synchronization
- Requires careful management of shared memory layout
- All shared structs must be `#[repr(C)]` for ABI stability
- Lock-free synchronization using `AtomicU64` sequence numbers

**Implementation Files**:
- `crates/scarab-protocol/` - IPC definitions
- `crates/scarab-daemon/src/ipc/` - Daemon IPC implementation
- `crates/scarab-client/src/ipc/` - Client IPC implementation

---

### ADR-002: Shared Memory Protocol

**Date**: 2025-11
**Status**: Accepted and Implemented

**Context**: Need efficient IPC between daemon and client processes.

**Decision**: Use POSIX shared memory (`/dev/shm/scarab_shm_v1`) with:
- `#[repr(C)]` structs for guaranteed memory layout
- `bytemuck::{Pod, Zeroable}` for safe zero-copy transmutation
- `AtomicU64` sequence numbers for lock-free synchronization
- Ring buffer for bulk terminal grid data

**Rationale**:
- Zero-copy: No serialization/deserialization overhead
- Lock-free: Client reads never block daemon writes
- Fast: Direct memory access, no system calls
- Simple: No complex messaging protocols

**Consequences**:
- All shared types must be `no_std` compatible
- Cannot use heap-allocated types (String, Vec) in shared memory
- Requires manual versioning strategy for protocol changes
- Platform-specific: Requires POSIX shared memory support

**Key Constraints**:
- SharedState must be `#[repr(C)]`
- Default grid: 200x100 cells (configurable)
- Image shared memory separate: `/dev/shm/scarab_img_shm_v1`

---

### ADR-003: Consolidate SharedState to Single Definition

**Date**: 2025-11-23 (GitHub Issue #1)
**Status**: Resolved

**Context**: Multiple conflicting SharedState definitions existed across crates causing compilation errors.

**Decision**: Consolidate to single canonical definition in `scarab-protocol` crate.

**Rationale**:
- Single source of truth for IPC protocol
- Prevents ABI mismatches between daemon and client
- Clean dependency graph (protocol is lowest-level crate)

**Implementation**: Removed duplicate definitions from daemon and client, both now import from `scarab_protocol::SharedState`

---

## 2. Plugin System Architecture

### ADR-004: Fusabi as Plugin Language

**Date**: Early 2025-11
**Status**: Accepted and Implemented

**Context**: Need a safe, performant scripting language for terminal plugins.

**Decision**: Adopt Fusabi (F# dialect for Rust) as the official plugin language with dual runtimes:
- **fusabi-vm**: Compiled bytecode (.fzb) for daemon plugins
- **fusabi-frontend**: Interpreted scripts (.fsx) for client UI plugins

**Rationale**:
- **Type Safety**: Compile-time checks prevent runtime errors
- **Familiar Syntax**: F# developers feel at home
- **Performance**: Compiled bytecode for hot paths
- **Hot Reload**: Interpreted scripts for rapid UI iteration
- **Official Support**: Fusabi is maintained by fusabi-lang organization

**Alternatives Considered**:
- Lua: Common in game engines but lacks strong typing
- Python: Too slow for terminal output filtering
- WASM: Too heavyweight, complex toolchain
- JavaScript: Weak typing, security concerns

**Consequences**:
- Dependency on fusabi-vm and fusabi-frontend crates
- Plugin authors must learn F# syntax (but examples provided)
- Two runtimes to maintain (VM for daemon, frontend for client)

---

### ADR-005: Hybrid Plugin Architecture (Daemon + Client)

**Date**: 2025-11
**Status**: Accepted and Implemented

**Decision**: Split plugin system into two layers:
- **Daemon Plugins** (.fzb bytecode): High-performance, run in daemon process
  - Hooks: on_output, on_input, on_pre_command, on_post_command
  - Use case: Output filtering, command monitoring, shell integration
- **Client Plugins** (.fsx scripts): Hot-reloadable, run in client process
  - Hooks: on_keypress, on_render, custom UI overlays
  - Use case: UI customization, keyboard shortcuts, visual enhancements

**Rationale**:
- Performance: Critical hooks (output filtering) run in compiled code
- Developer Experience: UI plugins can hot-reload without Rust recompilation
- Security: Client can't directly access PTY process
- Flexibility: Different use cases need different performance profiles

**Remote UI Protocol**: Daemon plugins can send RemoteCommands to client for UI updates (overlays, modals, notifications).

---

### ADR-006: Plugin API Hooks

**Date**: 2025-11-23 (GitHub Issue #4)
**Status**: Implemented

**Decision**: Define comprehensive hook system with 10+ lifecycle and event hooks:

**Lifecycle**: on_load, on_unload, on_attach, on_detach
**Terminal Events**: on_output, on_input, on_resize
**Command Tracking**: on_pre_command, on_post_command
**Remote UI**: on_remote_command, get_commands, get_menu

**Rationale**:
- Covers all major terminal lifecycle events
- Enables rich plugin ecosystem
- Plugins can intercept, modify, or block data flow
- Async-first design (all hooks return `Async<Result<T, string>>`)

**Safety Features**:
- Panic catching: Plugin panics don't crash daemon
- Timeouts: Hooks have execution time limits
- Error tracking: Failed hooks are logged and disabled

---

### ADR-007: Migration to fusabi-tui-runtime

**Date**: 2025-12-08 (Recent)
**Status**: Completed (v0.3.0)

**Context**: Initially used custom TUI implementation, but fusabi-tui-runtime was published to crates.io.

**Decision**: Migrate scarab-client to use fusabi-tui-runtime from crates.io instead of local path dependencies.

**Rationale**:
- Official release: Published crates are more stable
- Simpler dependencies: No workspace path dependencies
- Ecosystem growth: Encourages reuse across projects
- Version pinning: Can specify exact versions

**Consequences**:
- Removed local fusabi-tui-* dependencies
- Updated to published versions on crates.io
- Simplified workspace structure

**Related Commits**: #187 (chore: switch fusabi-tui-runtime deps to crates.io)

---

## 3. Migration to Bevy 0.15

### ADR-008: Bevy 0.15 Migration Strategy

**Date**: 2025-11-23
**Status**: Completed (Core), In Progress (Advanced UI)

**Context**: Bevy 0.15 introduced breaking changes to text rendering and color APIs.

**Decision**: Phased migration approach:
1. **Phase 1** (Completed): Migrate core rendering (text, colors, meshes)
2. **Phase 2** (In Progress): Migrate advanced UI (link hints, command palette, leader key menu)

**Key API Changes**:
- Text: `Text::from_section()` → `Text::from_sections([TextSection::new(...)])`
- Colors: `Color::rgba()` → `Color::srgba()` (linear to sRGB color space)
- UI Bundles: Bundle structure changes

**Rationale**:
- Prioritize core functionality first
- Temporarily disable advanced features to get core working
- Document migration path for future developers

**Consequences**:
- Created MIGRATION_GUIDE.md with API change examples
- Some UI features temporarily disabled during migration
- Warnings in codebase about deprecated APIs

---

## 4. Navigation and UI System

### ADR-009: Navigation System (scarab-nav)

**Date**: 2025-11-24
**Status**: Implemented, Under Audit

**Decision**: Implement comprehensive keyboard-driven navigation system with:
- **Link Hints** (Ctrl+Shift+O): Vim-style link activation
- **Pane Switching** (Ctrl+H/J/K/L): Vim-style pane navigation
- **Prompt Jumping** (Ctrl+Up/Down): Jump between shell prompts
- **Focusable Detection**: Automatic detection of clickable elements

**Rationale**:
- Keyboard-first UX: Aligns with terminal user expectations
- Vim-inspired: Familiar to power users
- Accessibility: Keyboard navigation for users who can't/won't use mouse

**Audit Findings** (Issue #40): Multiple focusable detection code paths need unification.

**Configuration**: User-configurable keymaps (Issue #38).

---

### ADR-010: Command Palette System (scarab-palette)

**Date**: 2025-11
**Status**: Implemented

**Decision**: Implement fuzzy-searchable command palette (Ctrl+Shift+P) modeled after VS Code.

**Features**:
- Fuzzy search with scoring algorithm
- Plugin-contributed commands via `GetCommands` hook
- Keyboard-driven selection
- Command palette state persistence

**Rationale**:
- Discoverability: Users can find commands without memorizing shortcuts
- Extensibility: Plugins can register custom commands
- Modern UX: Familiar pattern from VS Code, Sublime Text

---

## 5. Performance Optimizations

### ADR-011: VTE Parser Caching

**Date**: 2025-11-24 (Issue #14)
**Status**: Implemented

**Context**: VTE parsing is CPU-intensive for repetitive sequences.

**Decision**: Implement LRU cache for parsed VTE sequences:
- Cache size: 256 entries (configurable)
- Hit rate: 60-85% for typical workflows
- CPU reduction: 20-40% for repetitive output (e.g., `ls --color`)

**Rationale**:
- Many terminal sequences repeat (colors, cursor movements)
- Parsing overhead dominates CPU time for bulk output
- LRU eviction balances cache size vs hit rate

**Implementation**: `crates/scarab-daemon/src/vte/cache.rs`

---

### ADR-012: Texture Atlas Caching (cosmic-text)

**Date**: 2025-11
**Status**: Implemented

**Decision**: Use cosmic-text's texture atlas caching for glyph rendering.

**Rationale**:
- GPU textures are expensive to create
- Most glyphs repeat frequently (ASCII characters)
- Atlas packing minimizes texture switches

**Performance**:
- 60+ FPS at 200x100 cells
- <5ms frame time for static terminal content

---

## 6. Testing Strategy

### ADR-013: Multi-Layer Testing Approach

**Date**: 2025-11-23 (Issue #11, #13)
**Status**: Implemented

**Decision**: Comprehensive testing strategy with multiple layers:

**Unit Tests**: 332+ tests across workspace
- Core logic in each crate
- Fast iteration (<1 second)

**Integration Tests**:
- Daemon IPC tests
- Session persistence tests
- Plugin loading tests

**E2E Tests** (Issue #34):
- Real daemon + client interaction
- Terminal workflows (vim, htop, plugins)
- Stress tests

**Headless Tests** (ratatui-testlib):
- Env-gated: `SCARAB_TEST_RTL=1`
- Terminal rendering without GUI
- Snapshot testing with insta

**Smoke Tests**:
- Navigation smoke test: `just nav-smoke` (Nushell script)
- Quick verification of core workflows

**Rationale**:
- Catch bugs at multiple levels
- Fast feedback for developers
- High confidence in releases

**Current Status**: 332 tests passing, 1 failing (HTML export edge case)

---

### ADR-014: Justfile as Task Runner

**Date**: Early 2025-11
**Status**: Implemented

**Decision**: Use `just` (make alternative) for project task automation.

**Rationale**:
- Cross-platform: Works on Linux, macOS, Windows
- Better syntax than Makefiles
- Built-in variable expansion and scripting
- Familiar to Rust developers

**Task Categories**:
- Build: `just build`, `just build-release`
- Run: `just run`, `just run-bg`, `just fresh-run`
- Test: `just test`, `just test-all`, `just e2e`, `just integration`
- CI: `just ci` (format + clippy + test)
- Plugins: `just plugin-new`, `just plugin-build`, `just dev-mode`
- Install: `just install`, `just uninstall`

**Verification**: All recipes work (confirmed 2025-12-15)

---

## 7. Distribution and Packaging

### ADR-015: Homebrew Distribution

**Date**: 2025-11-24
**Status**: Configured, Pending SHA Checksums

**Decision**: Distribute Scarab via Homebrew tap for macOS and Linux users.

**Implementation**:
- Homebrew tap repository: `raibid-labs/homebrew-scarab`
- Formula template: `docs/HOMEBREW_SETUP.md`
- GitHub Actions workflow for releases

**Rationale**:
- Familiar install method for developers
- Automatic dependency management
- Easy updates via `brew upgrade`

**Status**: Infrastructure ready, needs SHA256 checksums for first release.

---

### ADR-016: GitHub Release Workflow

**Date**: 2025-11
**Status**: Implemented (6 platforms)

**Decision**: Automated release workflow building for:
- Linux x86_64 (glibc, musl)
- macOS x86_64, aarch64
- Windows x86_64
- FreeBSD x86_64

**Rationale**:
- Cross-platform support from day one
- Automated releases reduce manual work
- Binary distribution for users without Rust toolchain

**Implementation**: `.github/workflows/release.yml`

---

## 8. Historical Phase Summary

### Phase 1-4: Foundation (2025-11)

**Completed Work**:
- Core terminal emulation (VTE parser, rendering)
- Split-process architecture (daemon + client)
- Zero-copy IPC with shared memory
- Plugin system with Fusabi integration
- Session management and persistence
- Remote UI protocol
- Core plugins (scarab-nav, scarab-palette, scarab-session)

**Key Metrics**:
- ~80% of MVP features complete
- 332+ tests passing
- 3,393 lines of UI/UX implementation
- 75+ documentation files

### Phase 5: Integration & Polish (2025-12)

**Focus Areas**:
- Bevy 0.15 migration
- E2E testing
- Documentation updates
- Tutorial system
- Release preparation

**Current Status**: ~85% complete

---

## 9. Key Lessons Learned

### SharedState Consolidation
- **Problem**: Multiple definitions caused ABI mismatches
- **Solution**: Single source of truth in scarab-protocol
- **Lesson**: Define protocol separately from implementation

### Plugin System Complexity
- **Challenge**: Dual runtimes (VM + frontend) add complexity
- **Benefit**: Performance where needed, DX where wanted
- **Lesson**: Hybrid approach balances tradeoffs

### Migration Strategy
- **Approach**: Phased migration (core first, advanced features second)
- **Benefit**: Core functionality maintained during refactor
- **Lesson**: Disable non-essential features during major upgrades

### Testing Investment
- **Strategy**: Multi-layer testing from day one
- **Benefit**: High confidence in changes, fast iteration
- **Lesson**: Test infrastructure pays dividends

---

## 10. Future Directions

### Documented Future Work

**From CURRENT_STATUS_AND_NEXT_STEPS.md**:
- Scrollback UI (high priority)
- Copy/Paste enhancement
- Mouse support
- Theme system
- Tab/Pane management
- Interactive tutorial

**From ROADMAP.md**:
- Phase 6: Tabs, splits, window management
- Phase 7: macOS and Windows support
- Phase 8: Advanced rendering (ligatures, images, sixel)
- Phase 9: Multiplexing and remote sessions
- Phase 10: Beta release and ecosystem growth

---

## Document History

**Created**: 2025-12-15
**Sources**:
- IMPLEMENTATION_SUMMARY.md
- MIGRATION_GUIDE.md
- CURRENT_STATUS_AND_NEXT_STEPS.md
- WAVE1_TACTICAL_GUIDE.md
- ORCHESTRATION_PLAN.md
- GitHub Issues (#1-#50)
- Git commit history

**Archived Documents**: See `/docs/deprecated/` for historical implementation plans and execution summaries that were consolidated into this ADR.

---

## References

For current project status and active roadmap, see:
- [ROADMAP.md](/ROADMAP.md) - Strategic development roadmap
- [CLAUDE.md](/CLAUDE.md) - Architecture overview and build commands
- [TESTING.md](/TESTING.md) - Testing guide
- [README.md](/README.md) - User-facing documentation
