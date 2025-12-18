# Crate Structure Analysis
## Technical Audit - December 15, 2025

### Executive Summary

The Scarab workspace contains **16 crates** organized into a clean domain-driven architecture. The crate structure is generally well-designed with clear separation of concerns between client, daemon, and shared libraries. However, there are opportunities for consolidation and extraction of shared functionality.

---

## Workspace Overview

### Crates by Category

#### Core Runtime (2 crates)
- **scarab-client** - Bevy-based GUI client (112,353 total LoC)
- **scarab-daemon** - Headless server with PTY ownership

#### IPC & Protocol (2 crates)
- **scarab-protocol** - Shared memory definitions (`#[repr(C)]`, no_std)
- **scarab-plugin-api** - Plugin trait definitions and host bindings

#### Feature Plugins (7 crates)
- **scarab-nav** - Navigation system (extracted from upstream)
- **scarab-clipboard** - Clipboard integration
- **scarab-mouse** - Mouse interaction
- **scarab-tabs** - Tab management
- **scarab-panes** - Pane/split management
- **scarab-themes** - Theme system
- **scarab-telemetry-hud** - Performance metrics overlay

#### Infrastructure (5 crates)
- **scarab-config** - Configuration system with plugin registry
- **scarab-session** - Session management (local + SSH)
- **scarab-platform** - Platform abstraction layer
- **scarab-plugin-compiler** - Fusabi plugin compiler
- **scarab-palette** - Color palette management

---

## Code Size Analysis

### Largest Files (>1000 LoC)
1. `scarab-client/src/navigation/tests.rs` - **2,306 lines**
2. `scarab-plugin-api/src/copy_mode/mod.rs` - **1,343 lines**
3. `scarab-daemon/src/ipc.rs` - **1,236 lines**
4. `scarab-daemon/src/vte.rs` - **1,218 lines**
5. `scarab-daemon/tests/plugin_integration.rs` - **1,140 lines**
6. `scarab-panes/src/lib.rs` - **1,067 lines**
7. `scarab-config/src/config.rs` - **1,021 lines**

**Finding**: Several files exceed recommended size limits (500-800 LoC). The navigation tests file at 2,306 lines should be split into multiple test modules.

### Crate Complexity

| Crate | Files | Approx LoC | Complexity |
|-------|-------|------------|------------|
| scarab-client | 50+ | ~15,000 | High |
| scarab-daemon | 20+ | ~8,000 | High |
| scarab-plugin-api | 15+ | ~4,000 | Medium |
| scarab-protocol | 3 | ~1,500 | Low |
| scarab-panes | 1 | ~1,067 | Medium |
| scarab-config | 5+ | ~2,500 | Medium |

---

## Architecture Patterns

### Positive Patterns

1. **Clear Process Boundary**
   - Protocol crate is `#![no_std]` with `#[repr(C)]`
   - Zero-copy shared memory via `bytemuck`
   - Atomic sequence numbers for lock-free sync

2. **Plugin Architecture**
   - Clean separation: `scarab-plugin-api` defines interfaces
   - Individual feature crates are small and focused
   - Async-first design with `async-trait`

3. **Dependency Discipline**
   - Workspace-level dependency management
   - Core crates have minimal dependencies
   - Protocol crate has only 3 dependencies (bytemuck, rkyv, serde)

### Areas for Improvement

1. **Crate Granularity**
   - Some crates are too small (scarab-palette: ~200 LoC)
   - Could consolidate: palette + themes into one crate
   - Could consolidate: clipboard + mouse into one crate

2. **Test Organization**
   - Large test files should be split into modules
   - `scarab-client/src/navigation/tests.rs` at 2,306 lines is excessive

3. **Documentation Consistency**
   - Some crates lack module-level documentation
   - Missing examples in several plugin crates

---

## Crate-by-Crate Analysis

### scarab-client (P1 - Needs Refactoring)

**Size**: ~15,000 LoC across 50+ files
**Dependencies**: 30+ direct dependencies

**Issues**:
- Ratatui bridge is well-documented but only used internally
- Navigation system has 2,306 lines of tests - needs modularization
- Marketplace overlay is 717 LoC - could be extracted to separate crate

**Recommendation**:
- Split navigation tests into multiple modules
- Consider extracting marketplace to `scarab-marketplace` crate
- Extract ratatui bridge to standalone crate for reuse

### scarab-daemon (P1 - Review Required)

**Size**: ~8,000 LoC across 20+ files
**Dependencies**: 27+ direct dependencies

**Issues**:
- VTE parser is 1,218 lines - consider extracting VTE logic
- IPC module is 1,236 lines - complex, needs documentation
- Plugin integration test is 1,140 lines - split into scenarios

**Recommendation**:
- Document IPC module's synchronization strategy
- Split VTE parser into sub-modules
- Consider extracting VTE logic to `scarab-vte` crate

### scarab-protocol (P0 - Well Designed)

**Size**: ~1,500 LoC across 3 files
**Dependencies**: Minimal (3 deps)

**Strengths**:
- Clean `#![no_std]` design
- Proper `#[repr(C)]` alignment
- Safe zero-copy with bytemuck

**No changes recommended** - this crate is exemplary.

### scarab-plugin-api (P2 - Consider Splitting)

**Size**: ~4,000 LoC across 15+ files
**Dependencies**: 15+ direct dependencies

**Issues**:
- Copy mode module is 1,343 lines - could be its own crate
- Mixes trait definitions with complex implementations
- Host bindings are 958 lines - complex

**Recommendation**:
- Extract copy mode to `scarab-copy-mode` crate
- Split into `-api` (traits) and `-impl` (implementations)

### scarab-config (P2 - Consolidation Candidate)

**Size**: ~2,500 LoC across 5+ files
**Dependencies**: Heavy (includes crypto, HTTP for registry)

**Issues**:
- Config file parsing is 1,021 lines
- Registry security is 735 lines
- Mixes concerns: config parsing + plugin registry + security

**Recommendation**:
- Split into `scarab-config` (core) and `scarab-registry` (plugin marketplace)
- Consider if registry belongs in daemon or client

### scarab-panes (P1 - Needs Decomposition)

**Size**: 1,067 LoC in single file
**Dependencies**: Minimal

**Issues**:
- Entire crate is one file
- Complex pane management logic with no sub-modules

**Recommendation**:
- Split into modules: layout, split, resize, focus
- Add integration tests

### scarab-nav (P0 - CRITICAL: Duplication Issue)

**Size**: ~295 LoC (single file)
**Dependencies**: Links to `scarab-nav-protocol` 0.1.0

**CRITICAL FINDING**:
The `/home/beengud/raibid-labs/scarab/crates/scarab-nav` is a **stub plugin** that delegates to the upstream `scarab-nav` repository at `/home/beengud/raibid-labs/scarab-nav/`.

The upstream repo contains:
- `scarab-nav-protocol` (v0.2.0) - Protocol definitions
- `scarab-nav-client` (v0.2.0) - Client library

**Issue**: The monorepo contains a thin wrapper that imports `scarab-nav-protocol` v0.1.0, but the upstream has evolved to v0.2.0. This creates a version mismatch.

**Recommendation**:
- **P0**: Remove the `scarab-nav` crate from monorepo entirely
- Import `scarab-nav-client` and `scarab-nav-protocol` as external dependencies
- The upstream repo is the source of truth

### Small Crates (Consolidation Candidates)

#### scarab-palette (P2)
- **Size**: ~200 LoC
- **Recommendation**: Merge into `scarab-themes` (both deal with colors)

#### scarab-clipboard (P2)
- **Size**: ~150 LoC
- **Recommendation**: Merge into `scarab-mouse` (both handle user interaction)

#### scarab-telemetry-hud (P2)
- **Size**: ~650 LoC
- **Recommendation**: Keep separate but consider making optional feature

---

## Dependency Graph Analysis

### Workspace Dependencies (workspace.dependencies)

**Fusabi Dependencies** (6):
- fusabi-vm (0.17.0) - VM runtime
- fusabi-frontend (0.17.0) - Compiler
- bevy-fusabi (0.1.4) - Bevy integration
- fusabi-plugin-runtime (0.1.1) - Plugin loader
- fusabi-stdlib-ext (0.1.1) - Standard library
- fusabi-tui-* (0.1.x) - TUI runtime (3 crates)

**Status**: All Fusabi deps are on crates.io (good!)

**Ratatui Dependencies**:
- `ratatui-testlib` - Git dependency (branch: main)

**Issue**: `ratatui-testlib` is still a git dependency. Should be published to crates.io.

### Circular Dependencies

**None detected** - Good separation of concerns.

### Dependency Duplication

Multiple crates depend on:
- `parking_lot` (5 crates)
- `log` (15+ crates)
- `async-trait` (8 crates)
- `serde` (all crates)

**Recommendation**: These are workspace dependencies, correctly managed.

---

## Migration Status: Fusabi TUI Runtime

The project recently migrated from `ratatui` to `fusabi-tui-runtime`.

**Analysis**:
```bash
# Files still referencing ratatui:
- 42 files contain "ratatui" references
- Most are in tests and the ratatui_bridge module
- The ratatui_bridge is a compatibility layer
```

**Status**:
- Core runtime has migrated to `fusabi-tui-*`
- Ratatui bridge exists for legacy widget support
- Tests still use `ratatui-testlib`

**Recommendation**:
- Keep ratatui bridge as compatibility layer
- Mark it clearly as "legacy support"
- Ensure all new code uses `fusabi-tui-*`

---

## Recommendations Summary

### Priority 0 (Immediate Action Required)

1. **Remove scarab-nav from monorepo** - Use upstream crates
2. **Update scarab-nav-protocol to v0.2.0** - Version mismatch

### Priority 1 (High Impact)

3. **Split large test files** - Especially navigation tests (2,306 LoC)
4. **Document IPC synchronization** - Critical for maintainability
5. **Modularize scarab-panes** - Single 1,067 line file

### Priority 2 (Medium Impact)

6. **Consolidate small crates**:
   - Merge palette → themes
   - Merge clipboard → mouse
7. **Extract copy mode** - From plugin-api to separate crate
8. **Split scarab-config** - Separate registry concerns

### Priority 3 (Low Impact)

9. **Publish ratatui-testlib** - Remove git dependency
10. **Add module docs** - Improve discoverability
11. **Extract marketplace** - To separate crate if it grows

---

## Metrics

### Crate Count: 16
- **Core**: 2
- **Protocol**: 2
- **Plugins**: 7
- **Infrastructure**: 5

### Lines of Code: ~112,353 total
- **Client**: ~15,000
- **Daemon**: ~8,000
- **Plugin API**: ~4,000
- **Tests**: ~8,000
- **Other**: ~77,353

### Dependency Health
- **Workspace deps**: Well managed
- **Git deps**: 1 (ratatui-testlib)
- **Circular deps**: 0
- **Duplicate deps**: Properly managed via workspace

---

## Conclusion

The crate structure is **fundamentally sound** with clear separation of concerns and good dependency discipline. The main issues are:

1. **Code organization** - Some files are too large
2. **scarab-nav duplication** - Should use upstream repo
3. **Small crate consolidation** - Reduce maintenance overhead

Overall Grade: **B+** (Good architecture with room for optimization)
