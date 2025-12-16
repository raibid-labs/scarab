# Refactoring Opportunities
## Technical Audit - December 15, 2025

### Executive Summary

This report identifies opportunities to improve code organization, reduce complexity, and enhance maintainability across the Scarab codebase. The analysis focuses on **actionable refactorings** with measurable impact.

---

## High-Impact Opportunities (P0-P1)

### 1. Split Large Test Files (P0)

**Issue**: Several test files exceed 1,000 lines, making them difficult to navigate and maintain.

**Files**:
- `scarab-client/src/navigation/tests.rs` - **2,306 lines**
- `scarab-daemon/tests/plugin_integration.rs` - **1,140 lines**
- `scarab-client/tests/headless_harness.rs` - **722 lines**

**Recommendation**:

**navigation/tests.rs** → Split into:
```
navigation/
├── mod.rs
└── tests/
    ├── mod.rs
    ├── focusable_tests.rs      # ~500 lines
    ├── link_detection_tests.rs # ~500 lines
    ├── keyboard_nav_tests.rs   # ~500 lines
    ├── overlay_tests.rs        # ~400 lines
    └── integration_tests.rs    # ~400 lines
```

**plugin_integration.rs** → Split by scenario:
```
tests/
├── plugin_integration/
    ├── mod.rs
    ├── lifecycle_tests.rs      # ~300 lines
    ├── input_handling_tests.rs # ~300 lines
    ├── remote_commands_tests.rs# ~300 lines
    └── error_handling_tests.rs # ~240 lines
```

**Impact**:
- Improves test discoverability
- Enables parallel test runs
- Easier to add new tests

**Effort**: Medium (2-3 hours)
**Priority**: P0

---

### 2. Modularize scarab-panes (P0)

**Issue**: Entire crate is a single 1,067-line file.

**Current**:
```
scarab-panes/
└── src/
    └── lib.rs (1,067 lines)
```

**Proposed**:
```
scarab-panes/
└── src/
    ├── lib.rs          # Public API, re-exports
    ├── types.rs        # Pane, PaneId, Direction types
    ├── layout.rs       # Layout algorithm
    ├── split.rs        # Split operations
    ├── resize.rs       # Resize logic
    ├── focus.rs        # Focus management
    └── manager.rs      # PaneManager impl
```

**Extraction Strategy**:
1. Extract types to `types.rs` (~100 lines)
2. Extract layout algorithm to `layout.rs` (~250 lines)
3. Extract split operations to `split.rs` (~200 lines)
4. Extract resize logic to `resize.rs` (~200 lines)
5. Extract focus management to `focus.rs` (~150 lines)
6. Keep manager impl in `manager.rs` (~167 lines)

**Benefits**:
- Each module < 300 lines
- Clear separation of concerns
- Easier testing of individual components

**Effort**: Medium (3-4 hours)
**Priority**: P0

---

### 3. Extract Copy Mode from Plugin API (P1)

**Issue**: `scarab-plugin-api/src/copy_mode/mod.rs` is **1,343 lines** and implements a full feature, not just an API.

**Current Structure**:
```
scarab-plugin-api/
└── src/
    ├── lib.rs
    ├── types.rs
    ├── host_bindings.rs
    └── copy_mode/
        └── mod.rs (1,343 lines)
```

**Proposed**:
```
scarab-copy-mode/          # New crate
└── src/
    ├── lib.rs
    ├── cursor.rs
    ├── selection.rs
    ├── search.rs
    ├── clipboard.rs
    └── modes.rs

scarab-plugin-api/
└── src/
    └── copy_mode.rs      # Trait definition only (~100 lines)
```

**Migration Plan**:
1. Create new `scarab-copy-mode` crate
2. Move implementation from plugin-api
3. Keep only trait definitions in plugin-api
4. Update client to use new crate

**Benefits**:
- Plugin API stays focused on interfaces
- Copy mode can evolve independently
- Reduces plugin-api complexity

**Effort**: Medium-High (4-6 hours)
**Priority**: P1

---

### 4. Document IPC Synchronization Protocol (P0)

**Issue**: The lock-free IPC pattern is complex but not documented.

**Current**:
- Daemon writes to shared memory with atomic sequence numbers
- Client reads with SeqLock pattern
- No comprehensive documentation of the protocol

**Locations**:
- `scarab-daemon/src/ipc.rs` - 1,236 lines
- `scarab-client/src/integration.rs` - Reader implementation
- `scarab-protocol/src/lib.rs` - Shared state definitions

**Recommendation**:

Create documentation at:
```
docs/architecture/
└── IPC_SYNCHRONIZATION.md
```

**Contents**:
1. **Overview**: Lock-free shared memory architecture
2. **Memory Layout**: SharedState structure explanation
3. **Synchronization**: Atomic sequence number protocol
4. **Writer Protocol**: Daemon-side steps
5. **Reader Protocol**: Client-side steps
6. **Correctness**: Memory ordering guarantees
7. **Error Handling**: Stale reads, error mode
8. **Performance**: Benchmarks and optimization notes

**Code Example**:
```rust
// WRITER (Daemon)
// 1. Increment sequence (marks write start)
let seq = state.sequence_number.fetch_add(1, Ordering::SeqCst);

// 2. Update cells
for (i, cell) in updates.iter().enumerate() {
    state.cells[i] = *cell;
}

// 3. Write fence + increment again (marks write end)
atomic::fence(Ordering::Release);
state.sequence_number.store(seq + 1, Ordering::Release);

// READER (Client)
// 1. Read sequence (should be even)
let seq1 = state.sequence_number.load(Ordering::Acquire);

// 2. Read cells
let snapshot = state.cells[..].to_vec();

// 3. Read sequence again
let seq2 = state.sequence_number.load(Ordering::Acquire);

// 4. Verify sequences match and are even
if seq1 == seq2 && seq1 % 2 == 0 {
    // Valid read
} else {
    // Retry or use stale data
}
```

**Impact**:
- Aids future contributors
- Documents critical correctness requirements
- Prevents bugs from misunderstanding

**Effort**: Low (2-3 hours for comprehensive docs)
**Priority**: P0

---

### 5. Consolidate Small Crates (P1)

**Issue**: Several crates are very small (<500 LoC) and could be consolidated.

#### 5a. Merge scarab-palette into scarab-themes

**Current**:
- `scarab-palette` - ~200 LoC (color palette management)
- `scarab-themes` - ~984 LoC (theme system)

**Rationale**: Both deal with color management. Themes already depend on palettes.

**Proposed**:
```
scarab-themes/
└── src/
    ├── lib.rs
    ├── theme.rs
    ├── manager.rs
    ├── palette.rs      # <-- Moved from scarab-palette
    └── plugin.rs
```

**Migration**:
1. Move `scarab-palette/src/*` to `scarab-themes/src/palette.rs`
2. Update imports in daemon
3. Remove `scarab-palette` from workspace

**Benefits**:
- One less crate to maintain
- Clearer dependency graph
- Palette and theme are conceptually related

**Effort**: Low (1-2 hours)
**Priority**: P1

#### 5b. Merge scarab-clipboard into scarab-mouse

**Current**:
- `scarab-clipboard` - ~150 LoC
- `scarab-mouse` - ~400 LoC

**Rationale**: Both handle user interaction. Mouse operations often involve clipboard (copy on select).

**Proposed**:
```
scarab-input/          # Rename from scarab-mouse
└── src/
    ├── lib.rs
    ├── mouse.rs       # <-- Original mouse code
    ├── clipboard.rs   # <-- Moved from scarab-clipboard
    └── selection.rs   # <-- Shared selection logic
```

**Alternative**: Keep separate if clipboard logic grows significantly.

**Benefits**:
- Unified input handling
- Shared selection state
- Reduces crate count

**Effort**: Low-Medium (2-3 hours)
**Priority**: P1

---

## Medium-Impact Opportunities (P2)

### 6. Extract Test Utilities Crate (P2)

**Issue**: Test setup code is duplicated across client and daemon tests.

**Current Duplication**:
- `scarab-client/tests/headless_harness.rs` - 722 lines
- `scarab-client/tests/bevy_harness_stubs.rs` - 200+ lines
- `scarab-daemon/tests/` - Similar setup code

**Proposed**:
```
scarab-test-utils/
└── src/
    ├── lib.rs
    ├── mock_terminal.rs    # Mock SharedState
    ├── bevy_harness.rs     # Bevy app test setup
    ├── daemon_harness.rs   # Daemon test setup
    └── assertions.rs       # Common test assertions
```

**Usage**:
```rust
// In test files
use scarab_test_utils::{MockTerminal, BevyHarness};

#[test]
fn test_rendering() {
    let harness = BevyHarness::new();
    let terminal = MockTerminal::with_size(80, 24);
    // ...
}
```

**Benefits**:
- Reduces ~1,500 lines of duplication
- Consistent test setup across crates
- Easier to improve test infrastructure

**Effort**: Medium (4-5 hours)
**Priority**: P2

---

### 7. Split scarab-config (P2)

**Issue**: Mixes multiple concerns in one crate.

**Current**:
- Config file parsing (1,021 LoC in config.rs)
- Plugin registry client (registry/)
- Security/verification (735 LoC in security.rs)

**Proposed**:
```
scarab-config/          # Core config
└── src/
    ├── lib.rs
    ├── config.rs       # Main config parsing
    ├── watcher.rs      # File watching
    └── validation.rs   # Config validation

scarab-registry/        # New crate for marketplace
└── src/
    ├── lib.rs
    ├── client.rs       # Registry API client
    ├── security.rs     # Signature verification
    └── installer.rs    # Plugin installation
```

**Benefits**:
- Clearer separation of concerns
- Registry can be optional feature
- Config crate has minimal dependencies

**Effort**: Medium-High (5-6 hours)
**Priority**: P2

---

### 8. Modularize VTE Parser (P2)

**Issue**: `scarab-daemon/src/vte.rs` is **1,218 lines** with complex state machine logic.

**Current**:
```
scarab-daemon/src/vte.rs (1,218 lines)
```

**Proposed**:
```
scarab-daemon/src/vte/
├── mod.rs              # Public API
├── parser.rs           # Core VTE state machine (~400 lines)
├── actions.rs          # Terminal actions (~300 lines)
├── escape_sequences.rs # Escape sequence handling (~300 lines)
└── osc.rs             # OSC sequence parsing (~218 lines)
```

**Benefits**:
- Each module < 400 lines
- Easier to test individual components
- Clearer separation of VTE logic

**Effort**: Medium (3-4 hours)
**Priority**: P2

---

### 9. Create Shared Config Utilities (P2)

**Issue**: TOML parsing patterns are repeated across config, themes, and plugins.

**Current**:
- `scarab-config/src/config.rs` - Custom TOML parsing
- `scarab-themes/src/manager.rs` - Theme TOML parsing
- Each plugin crate loads config differently

**Proposed**:

Add to `scarab-config/src/utils.rs`:
```rust
pub struct ConfigFile<T> {
    path: PathBuf,
    value: T,
}

impl<T: DeserializeOwned> ConfigFile<T> {
    pub fn load(path: impl AsRef<Path>) -> Result<Self>;
    pub fn reload(&mut self) -> Result<bool>;
    pub fn validate(&self) -> Result<()>;
}

pub trait Validatable {
    fn validate(&self) -> Result<()>;
}
```

**Usage**:
```rust
let config = ConfigFile::<ScarabConfig>::load("config.toml")?;
config.validate()?;
```

**Benefits**:
- Consistent config loading
- Shared validation logic
- Reduces ~200 lines of duplication

**Effort**: Low-Medium (2-3 hours)
**Priority**: P2

---

### 10. Extract Marketplace to Separate Crate (P2)

**Issue**: Marketplace is large feature embedded in scarab-client.

**Current**:
- `scarab-client/src/marketplace/overlay.rs` - 717 LoC
- `scarab-client/src/marketplace/installer.rs` - 500+ LoC
- `scarab-client/src/marketplace/mod.rs` - Entry point

**Proposed** (if marketplace grows):
```
scarab-marketplace/
└── src/
    ├── lib.rs
    ├── ui.rs           # UI overlay
    ├── installer.rs    # Plugin installation
    └── api.rs          # Registry API
```

**Benefits**:
- Reduces scarab-client complexity
- Optional feature for minimal builds
- Can be developed independently

**Caveat**: Only worth it if marketplace becomes core feature.

**Effort**: Medium-High (6-8 hours)
**Priority**: P2 (conditional on growth)

---

## Low-Impact Opportunities (P3)

### 11. Extract Ratatui Bridge (P3)

**Issue**: Ratatui bridge could be useful for other Bevy+Ratatui projects.

**Current**:
- `scarab-client/src/ratatui_bridge/` - Complete integration (~2,000 LoC)

**Proposed**:
```
bevy-ratatui-bridge/    # External crate on crates.io
└── src/
    ├── lib.rs
    ├── surface.rs
    ├── renderer.rs
    └── input.rs
```

**Benefits**:
- Community contribution
- Easier to maintain as separate project
- Could gain external contributors

**Caveat**: Only worth it if there's external demand.

**Effort**: High (8-10 hours + ongoing maintenance)
**Priority**: P3

---

### 12. Standardize Error Types (P3)

**Issue**: Mixed error handling patterns across crates.

**Current**:
- Some crates use `anyhow::Result`
- Some use custom error types with `thiserror`
- Inconsistent error context

**Proposed**:

Add to `scarab-protocol/src/error.rs`:
```rust
#[derive(Debug, thiserror::Error)]
pub enum ScarabError {
    #[error("IPC error: {0}")]
    Ipc(String),

    #[error("Plugin error: {0}")]
    Plugin(String),

    #[error("Configuration error: {0}")]
    Config(String),

    // ...
}

pub type Result<T> = std::result::Result<T, ScarabError>;
```

**Benefits**:
- Consistent error handling
- Better error messages
- Easier to handle errors at boundaries

**Caveat**: Large refactoring, may not be worth it.

**Effort**: High (10+ hours)
**Priority**: P3

---

## Architecture-Level Refactorings

### 13. Consider fusabi-tui-runtime Migration Completion (P1)

**Issue**: Project migrated to fusabi-tui-runtime, but ratatui bridge still exists.

**Current State**:
- Core uses `fusabi-tui-*` dependencies
- Ratatui bridge provides compatibility
- Tests still use `ratatui-testlib`

**Questions**:
1. Does `fusabi-tui-runtime` provide all ratatui functionality?
2. Can ratatui bridge be removed post-migration?
3. Should tests migrate to fusabi-tui testing tools?

**Investigation Needed**:
- Audit `fusabi-tui-core` vs `ratatui_bridge`
- Identify overlapping functionality
- Create migration plan

**Recommendation**:
- Document what features require ratatui bridge
- If fusabi-tui is complete, deprecate ratatui bridge
- Otherwise, clearly document ratatui as "legacy compatibility"

**Effort**: Medium (requires investigation + documentation)
**Priority**: P1

---

### 14. Consider Extracting VTE Parser to Separate Crate (P3)

**Issue**: VTE parsing logic is buried in scarab-daemon.

**Potential**:
- VTE parser could be generic library
- Other projects might use it
- Better separation of concerns

**Proposed**:
```
scarab-vte/             # New crate
└── src/
    ├── lib.rs          # Generic VTE parser
    ├── parser.rs
    └── actions.rs

scarab-daemon/
└── src/
    └── vte_adapter.rs  # Scarab-specific integration
```

**Benefits**:
- Reusable VTE parsing
- Easier to test in isolation
- Community contribution potential

**Caveat**: Only worth it if VTE parser is sufficiently generic.

**Effort**: High (10+ hours)
**Priority**: P3

---

## Summary of Recommendations

### Priority 0 (Immediate - High Impact)

| # | Refactoring | Effort | Lines Affected | Impact |
|---|-------------|--------|----------------|--------|
| 1 | Split large test files | Medium | ~4,000 | High |
| 2 | Modularize scarab-panes | Medium | ~1,067 | High |
| 4 | Document IPC protocol | Low | - | Critical |

**Total Effort**: ~8-12 hours
**Total Impact**: Significantly improves maintainability

### Priority 1 (High Impact)

| # | Refactoring | Effort | Lines Affected | Impact |
|---|-------------|--------|----------------|--------|
| 3 | Extract copy mode | Medium-High | ~1,343 | High |
| 5a | Merge palette → themes | Low | ~200 | Medium |
| 5b | Merge clipboard → mouse | Low-Medium | ~150 | Medium |
| 13 | Complete fusabi-tui migration | Medium | ~2,000 | High |

**Total Effort**: ~12-18 hours
**Total Impact**: Reduces complexity, clearer architecture

### Priority 2 (Medium Impact)

| # | Refactoring | Effort | Lines Affected | Impact |
|---|-------------|--------|----------------|--------|
| 6 | Extract test utilities | Medium | ~1,500 | Medium |
| 7 | Split scarab-config | Medium-High | ~1,756 | Medium |
| 8 | Modularize VTE parser | Medium | ~1,218 | Medium |
| 9 | Shared config utilities | Low-Medium | ~200 | Low |

**Total Effort**: ~14-20 hours
**Total Impact**: Reduces duplication, improves organization

### Priority 3 (Nice to Have)

| # | Refactoring | Effort | Lines Affected | Impact |
|---|-------------|--------|----------------|--------|
| 11 | Extract ratatui bridge | High | ~2,000 | External |
| 12 | Standardize errors | High | ~5,000+ | Medium |
| 14 | Extract VTE parser | High | ~1,218 | External |

**Total Effort**: ~25+ hours
**Total Impact**: Long-term maintenance benefits

---

## Implementation Strategy

### Phase 1: Quick Wins (Week 1)
1. Document IPC protocol (P0) - **2-3 hours**
2. Merge palette → themes (P1) - **1-2 hours**
3. Merge clipboard → mouse (P1) - **2-3 hours**

**Total**: ~6-8 hours
**Impact**: Immediate improvement in clarity

### Phase 2: Core Refactorings (Week 2)
4. Split large test files (P0) - **2-3 hours**
5. Modularize scarab-panes (P0) - **3-4 hours**
6. Shared config utilities (P2) - **2-3 hours**

**Total**: ~7-10 hours
**Impact**: Better organization

### Phase 3: Major Extractions (Week 3-4)
7. Extract copy mode (P1) - **4-6 hours**
8. Extract test utilities (P2) - **4-5 hours**
9. Modularize VTE parser (P2) - **3-4 hours**

**Total**: ~11-15 hours
**Impact**: Significant complexity reduction

### Phase 4: Strategic (As Needed)
10. Complete fusabi-tui migration (P1) - **Investigation + execution**
11. Split scarab-config (P2) - **5-6 hours**
12. Other P3 items - **As project evolves**

---

## Metrics

### Refactoring Potential

| Category | Current LoC | After Refactoring | Reduction |
|----------|-------------|-------------------|-----------|
| Test Files | ~4,000 | ~4,000 (better organized) | 0 (organizational only) |
| Monolithic Files | ~3,628 | ~3,628 (modularized) | 0 (organizational only) |
| Duplicate Code | ~2,000 | ~300 | **-1,700 (-85%)** |
| **Total** | ~9,628 | ~7,928 | **-1,700 (-18%)** |

### Code Quality Improvements

- **Modularity**: 7 monolithic files → 35+ focused modules
- **Testability**: Improved isolation of components
- **Maintainability**: Clearer separation of concerns
- **Discoverability**: Better file organization

---

## Conclusion

The refactoring opportunities are **well-scoped** and **highly actionable**. The recommended approach:

1. **Start with P0 items** - Quick wins with high impact
2. **Progress through P1** - Core architectural improvements
3. **Selectively implement P2** - Based on maintenance burden
4. **Defer P3** - Only if external demand justifies effort

**Estimated Total Effort**: ~40-60 hours for P0-P2
**Estimated Impact**: **-1,700 LoC**, better organization, reduced complexity

**Overall Assessment**: The codebase is **already well-structured**. These refactorings are optimizations, not critical fixes.

**Grade**: **A-** (solid foundation with room for polish)
