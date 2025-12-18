# Code Duplication Report
## Technical Audit - December 15, 2025

### Executive Summary

This report identifies code duplication across the Scarab codebase, with a focus on the **scarab-nav** relationship with its upstream repository. The analysis reveals **one critical duplication issue** and several opportunities for code sharing.

---

## Critical Finding: scarab-nav Duplication

### Context

The Scarab monorepo contains a `scarab-nav` crate at:
```
/home/beengud/raibid-labs/scarab/crates/scarab-nav/
```

An upstream repository exists at:
```
/home/beengud/raibid-labs/scarab-nav/
```

According to CLAUDE.md:
> "scarab-nav was extracted so plugins could implement the vimium-c like navigation"

### Comparison Analysis

#### Monorepo Version (scarab/crates/scarab-nav)

**Structure**:
```
scarab-nav/
├── Cargo.toml
└── src/
    └── lib.rs (295 lines)
```

**Cargo.toml**:
```toml
[package]
name = "scarab-nav"
version.workspace = true  # 0.3.0

[dependencies]
scarab-nav-protocol = "0.1.0"  # ⚠️ OLD VERSION
scarab-plugin-api = { path = "../scarab-plugin-api" }
anyhow = { workspace = true }
serde = { workspace = true }
tokio = { workspace = true }
regex = "1.10"
async-trait = "0.1"
prost = "0.12"
log = "0.4"
```

**Implementation**: Full plugin implementation with:
- Socket-based protocol listener
- URL detection via regex
- Link hints generation
- Keyboard navigation (Alt+f trigger)

#### Upstream Version (/home/beengud/raibid-labs/scarab-nav)

**Structure**:
```
scarab-nav/
├── Cargo.toml (workspace)
├── crates/
│   ├── scarab-nav-protocol/  # v0.2.0 ⚠️ NEWER
│   └── scarab-nav-client/    # v0.2.0 ⚠️ NEWER
├── proto/
├── docs/
└── README.md
```

**Cargo.toml** (workspace):
```toml
[workspace]
members = [
    "crates/scarab-nav-protocol",
    "crates/scarab-nav-client",
]

[workspace.package]
version = "0.2.0"  # ⚠️ NEWER THAN MONOREPO
```

**scarab-nav-protocol v0.2.0**:
- Protobuf definitions for navigation protocol
- Published to crates.io (presumably)

**scarab-nav-client v0.2.0**:
- Client library for TUI apps to report interactive elements
- Ratatui integration
- Fusabi TUI integration
- Plugin runtime support

### Key Differences

| Aspect | Monorepo (0.3.0) | Upstream (0.2.0) |
|--------|------------------|------------------|
| **Purpose** | Plugin implementation | Protocol + Client library |
| **Structure** | Single crate | Workspace with 2 crates |
| **Protocol Version** | Imports v0.1.0 | Defines v0.2.0 |
| **Scope** | Scarab-specific | Generic TUI navigation |
| **Features** | Plugin impl only | Protocol + Client helpers |

### Version Mismatch Issue

**CRITICAL**: The monorepo's `scarab-nav` depends on `scarab-nav-protocol = "0.1.0"`, but the upstream has evolved to **v0.2.0**.

**Impact**:
- Protocol incompatibility between versions
- Maintenance burden of keeping two versions in sync
- Potential runtime failures if protocol changes

---

## Duplication Analysis

### Type 1: Complete Duplication (CRITICAL)

**scarab-nav plugin implementation**

**Location 1**: `/home/beengud/raibid-labs/scarab/crates/scarab-nav/src/lib.rs`
**Location 2**: Conceptually duplicated by upstream design

**Issue**: The monorepo contains a **Scarab-specific plugin wrapper** that should instead:
1. Import `scarab-nav-protocol` from upstream
2. Implement plugin using the protocol
3. Not maintain its own protocol implementation

**Recommendation**:
```toml
# In scarab-daemon/Cargo.toml
[dependencies]
scarab-nav-protocol = "0.2.0"  # Use latest from upstream
```

**Action**: Remove `scarab/crates/scarab-nav` and implement the plugin directly in `scarab-daemon` using the upstream protocol.

---

### Type 2: Ratatui Bridge Duplication

**Pattern**: Ratatui integration code

**Locations**:
1. `scarab-client/src/ratatui_bridge/` - Complete bridge implementation
2. Upstream `scarab-nav-client` has ratatui integration
3. Both implement Bevy ↔ Ratatui conversion

**Files**:
- `scarab-client/src/ratatui_bridge/input.rs` - Key conversion
- `scarab-client/src/ratatui_bridge/renderer.rs` - Buffer rendering
- Similar logic likely exists in `fusabi-tui-runtime`

**Analysis**:
The ratatui bridge in scarab-client is a **compatibility layer** for legacy widgets. It's intentionally duplicated because:
- Scarab needs Bevy-specific integration
- fusabi-tui-runtime provides a different abstraction
- The bridge is documented as "legacy support"

**Recommendation**: **Keep as-is**, but:
- Mark clearly as "compatibility layer"
- Document that new code should use `fusabi-tui-*`
- Consider extracting to `bevy-ratatui-bridge` crate for reuse

---

### Type 3: Plugin Boilerplate

**Pattern**: Common plugin implementation patterns

**Occurrences**:
- `scarab-nav/src/lib.rs` - Plugin trait impl
- `scarab-palette/src/lib.rs` - Plugin trait impl
- `scarab-clipboard/src/lib.rs` - Plugin trait impl
- `scarab-mouse/src/lib.rs` - Plugin trait impl

**Duplication**:
Each plugin implements similar patterns:
```rust
pub struct XxxPlugin {
    metadata: PluginMetadata,
    state: Arc<Mutex<PluginState>>,
}

impl Plugin for XxxPlugin {
    fn metadata(&self) -> &PluginMetadata { ... }
    async fn on_input(&mut self, input: &[u8], ctx: &PluginContext) -> Result<Action> { ... }
}
```

**Analysis**: This is **acceptable duplication** because:
- It follows Rust's trait pattern
- Each plugin has unique state
- Boilerplate is minimal (~20 lines)

**Recommendation**: **No action needed**. This is idiomatic Rust.

---

### Type 4: IPC Pattern Duplication

**Pattern**: Shared memory access patterns

**Locations**:
1. `scarab-daemon/src/ipc.rs` - Writer implementation (1,236 LoC)
2. `scarab-client/src/integration.rs` - Reader implementation
3. Both use `shared_memory` crate with atomic sequence numbers

**Sample Code** (daemon):
```rust
// Writer side
let seq = state.sequence_number.fetch_add(1, Ordering::SeqCst);
// Update cells
state.sequence_number.store(seq + 1, Ordering::Release);
```

**Sample Code** (client):
```rust
// Reader side
let seq = state.sequence_number.load(Ordering::Acquire);
// Read cells
```

**Analysis**: This is **necessary duplication** because:
- Reader and writer have different concerns
- Abstraction would add overhead
- Patterns are well-documented

**Recommendation**: **No action needed**. Document the synchronization protocol in both files.

---

### Type 5: Configuration Parsing Duplication

**Pattern**: TOML parsing and validation

**Locations**:
1. `scarab-config/src/config.rs` - Main config parsing (1,021 LoC)
2. `scarab-themes/src/manager.rs` - Theme config parsing (347 LoC)
3. `scarab-config/src/registry/` - Registry config parsing

**Duplication**:
- Multiple TOML deserialization paths
- Similar validation logic
- Repeated error handling patterns

**Analysis**: **Moderate duplication**. Could benefit from:
- Shared validation utilities
- Common error types
- Helper functions for file loading

**Recommendation**:
- Extract common patterns to `scarab-config/src/utils.rs`
- Create `ConfigFile<T>` generic loader
- Priority: **P2** (low impact, but improves maintainability)

---

### Type 6: Test Infrastructure Duplication

**Pattern**: Test setup and harness code

**Locations**:
1. `scarab-client/tests/headless_harness.rs` - 722 LoC
2. `scarab-client/tests/bevy_harness_stubs.rs`
3. `scarab-daemon/tests/` - Similar test infrastructure
4. Multiple crates have test utility modules

**Duplication**:
- Mock terminal state setup
- Bevy app initialization
- Shared memory mocking
- Common assertions

**Analysis**: **Significant duplication**, but this is **test code**.

**Recommendation**:
- Extract to `scarab-test-utils` crate
- Share between client and daemon tests
- Priority: **P2** (improves test maintainability)

---

### Type 7: Repeated TODO Comments

**Pattern**: Unimplemented features with similar TODOs

**Findings** (from grep analysis):
```
scarab-mouse/src/bevy_plugin.rs:155:    // TODO: Get actual modifiers from Bevy input system
scarab-mouse/src/bevy_plugin.rs:292:        // TODO: Spawn context menu UI entity
scarab-mouse/src/bevy_plugin.rs:384:    // TODO: Get actual character at position from terminal grid
scarab-mouse/src/bevy_plugin.rs:395:    // TODO: Get actual terminal dimensions
```

Multiple TODOs for:
- Getting terminal dimensions
- Extracting text from grid
- OSC 133 support
- Waiting for ratatui-testlib v0.5.0

**Analysis**: These represent **incomplete features**, not duplication.

**Recommendation**:
- Track TODOs in GitHub issues
- Create milestone for ratatui-testlib v0.5.0 migration
- Priority: **P3** (tracking only)

---

## Duplication with Upstream Dependencies

### fusabi-tui-runtime

The project migrated to `fusabi-tui-runtime` but still maintains:
- `ratatui_bridge` module in scarab-client
- Tests using `ratatui-testlib`

**Question**: Does `fusabi-tui-runtime` provide equivalent functionality?

**Investigation Needed**:
- Compare `fusabi-tui-core` with `scarab-client/src/ratatui_bridge/`
- Identify overlapping functionality
- Determine if bridge can be removed post-migration

**Recommendation**:
- Audit `fusabi-tui-*` capabilities
- Document migration path from ratatui bridge
- Priority: **P1** (affects architecture)

---

## Recommendations

### Priority 0 (Critical - Immediate Action)

1. **Remove scarab-nav from monorepo**
   - Delete `/home/beengud/raibid-labs/scarab/crates/scarab-nav/`
   - Update workspace members
   - Import `scarab-nav-protocol = "0.2.0"` from crates.io
   - Implement navigation plugin directly in daemon

2. **Fix scarab-nav-protocol version mismatch**
   - Update all references from v0.1.0 → v0.2.0
   - Test protocol compatibility

### Priority 1 (High Impact)

3. **Audit fusabi-tui-runtime overlap**
   - Compare with ratatui bridge
   - Document what to use when
   - Create migration guide

4. **Extract test utilities**
   - Create `scarab-test-utils` crate
   - Share between client/daemon tests
   - Reduces ~1,500 lines of duplication

### Priority 2 (Medium Impact)

5. **Create shared config utilities**
   - Extract TOML parsing helpers
   - Generic `ConfigFile<T>` loader
   - Reduces ~200 lines of duplication

6. **Document IPC patterns**
   - Add synchronization protocol docs
   - Explain SeqLock pattern
   - Aids future contributors

### Priority 3 (Low Impact)

7. **Consider extracting ratatui bridge**
   - If other projects need Bevy ↔ Ratatui
   - Publish as `bevy-ratatui-bridge`
   - Low priority unless external demand

8. **Track TODOs in issues**
   - Convert TODO comments to GitHub issues
   - Organize by milestone
   - Improves project visibility

---

## Metrics

### Duplication Summary

| Type | Severity | Lines Affected | Action Required |
|------|----------|----------------|-----------------|
| scarab-nav | **CRITICAL** | ~295 | Remove from monorepo |
| Test Utils | Medium | ~1,500 | Extract to crate |
| Config Parsing | Low | ~200 | Share utilities |
| Plugin Boilerplate | None | ~100 | Keep as-is |
| IPC Patterns | None | - | Document only |
| Ratatui Bridge | Intentional | ~2,000 | Keep as compatibility layer |

### Total Estimated Duplication: **~2,000 lines** (excluding intentional duplication)

### Reduction Potential: **~1,700 lines** (85%)

---

## Conclusion

The most critical issue is the **scarab-nav duplication** with version mismatch. This should be resolved immediately by:

1. Removing the monorepo's scarab-nav crate
2. Using the upstream protocol from crates.io
3. Implementing the navigation plugin in scarab-daemon

Other duplication is either **intentional** (ratatui bridge, plugin boilerplate) or **low-impact** (config utilities, test code).

**Overall Assessment**: The codebase has **minimal problematic duplication** aside from the scarab-nav issue. Once resolved, the duplication level is **acceptable** for a project of this size.

**Grade**: **B** (would be A after scarab-nav cleanup)
