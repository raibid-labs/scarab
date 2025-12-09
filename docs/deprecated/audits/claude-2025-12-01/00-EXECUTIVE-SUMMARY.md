# Scarab Terminal Emulator - Technical Audit Executive Summary
**Date:** December 1, 2025
**Auditor:** Claude (Sonnet 4.5)
**Scope:** Deep technical audit covering architecture, testing, dependencies, and feature completeness

---

## TL;DR - Critical Findings

**Overall Assessment:** ⚠️ **Good Foundation, Significant Testing Gaps**

### 🔴 **CRITICAL PRIORITY ISSUES**

1. **Frontend/Client Cannot Be Tested Without Manual Verification**
   - Scarab-client rendering pipeline is tightly coupled to Bevy ECS
   - No headless testing capability for UI components
   - User must manually run the client to verify any UI changes
   - **BLOCKING**: This prevents reliable automated QA for the GUI

2. **Ratatui-testlib Integration Incomplete**
   - Currently only used in daemon for basic VTE tests
   - Client has NO integration with ratatui-testlib
   - Ratatui-testlib itself is in early development (v0.1.0)
   - **Missing**: Bevy integration features needed for Scarab

3. **Fusabi Feature Gap vs WezTerm's Lua**
   - Fusabi lacks critical configuration APIs that WezTerm's Lua provides
   - No event system for lifecycle hooks
   - Limited programmatic control over terminal state
   - **Impact**: Plugin developers cannot achieve feature parity with wezterm

### 🟡 **HIGH PRIORITY ISSUES**

4. **Rendering Pipeline Testability**
   - Mesh generation requires Bevy `Assets<Image>` (GPU context)
   - No pure data-only rendering mode
   - Cannot unit test rendering logic
   - Grade: C for testability

5. **Unsafe Shared Memory Access Patterns**
   - Multiple systems use raw pointer dereference
   - No abstraction layer for safe SharedState access
   - Potential UB if daemon crashes/unmaps memory

6. **Documentation Gaps**
   - CLAUDE.md (5KB) significantly outdated vs actual workspace (17 crates vs documented 5)
   - No comprehensive testing guide
   - Missing architecture diagrams for rendering pipeline

---

## Audit Metrics

### Codebase Size
```
Workspace Crates:    17 (vs 5 documented)
Source Files:        ~100+ Rust files
Test Files:          24 test modules
Lines of Code:       ~50,000+ (estimated)
Documentation Files: 20+ markdown files
```

### Test Coverage (Estimated)
```
Overall:               ~40-50%
Business Logic:        ~70% (good)
Rendering Logic:       ~5% (critical gap)
Integration/E2E:       ~20%
Frontend UI:           ~0% (BLOCKING)
```

### Architecture Quality
```
Modularity:            A- (excellent plugin system)
Separation of Concerns: B+ (good daemon/client split)
Testability:           C (rendering pipeline issues)
Documentation:         B (good but outdated)
Code Quality:          A- (clean, well-structured)
```

---

## Key Findings Summary

### ✅ **Strengths**

1. **Solid Core Architecture**
   - Clean daemon/client separation
   - Zero-copy IPC via shared memory
   - Lock-free synchronization with atomic sequences
   - Excellent modularity with 17 workspace crates

2. **Comprehensive Plugin System**
   - Well-designed plugin API
   - Dual runtime (client .fsx + daemon .fzb)
   - Remote UI protocol for daemon-client communication
   - Active plugin ecosystem (nav, palette, session)

3. **Good Business Logic Testing**
   - 487+ test cases for logic
   - Unit tests for link detection, fuzzy search, keybindings
   - Good test coverage for non-rendering code

4. **Performance Focus**
   - Benchmarks for VTE parsing, IPC throughput, rendering
   - Profiling integration (Tracy, puffin)
   - Optimized builds with LTO

### ⚠️ **Critical Weaknesses**

1. **Frontend Testing Impossible Without Manual Verification**
   - **THE CORE ISSUE**: No way to test client rendering headlessly
   - Bevy coupling prevents automated UI testing
   - Developer must run client manually to verify changes
   - This violates the audit's primary goal

2. **Rendering Pipeline Tight Coupling**
   - `generate_terminal_mesh()` returns Bevy-specific types
   - Requires GPU context (Assets<Image>) for atlas
   - Cannot mock or stub for testing
   - No intermediate data representation

3. **Ratatui-testlib Not Solving the Problem**
   - Currently only used for daemon VTE testing
   - Designed for PTY/terminal testing, NOT GUI testing
   - Client integration doesn't exist yet
   - Ratatui-testlib's Bevy features still in design phase

4. **Fusabi Capability Gaps**
   - Missing 12+ event types that wezterm provides
   - No status bar formatting API
   - No programmatic pane control
   - No command palette augmentation
   - Limited to basic keybindings

5. **Architectural Issues**
   - UI rendering uses Bevy primitives directly (no abstraction)
   - Unsafe shared memory access in multiple systems
   - Dirty region optimization disabled (always full redraw)
   - Fixed 4096x4096 atlas can overflow

---

## The Core Problem: Frontend Testing Loop Cannot Be Closed

### Current State
```
Developer makes UI change
    ↓
Must manually run scarab-client
    ↓
Visual inspection required
    ↓
No automated verification
```

### Desired State
```
Developer makes UI change
    ↓
Run: cargo test
    ↓
Automated tests verify:
  - Tab bar visible?
  - Command palette renders?
  - Link hints positioned correctly?
  - Overlays within bounds?
    ↓
PASS/FAIL without manual verification
```

### Why This Is Hard

The client uses **Bevy**, a game engine that:
- Requires a window/GPU context to render
- Tightly couples rendering to ECS systems
- Has no built-in headless testing
- Makes UI assertions difficult without actual rendering

**The Gap:** We need a way to:
1. Run Bevy systems without opening a window
2. Capture rendered output (mesh data, UI layout)
3. Assert on UI state (component positions, visibility)
4. Verify against expected snapshots

---

## Proposed Solutions

### 🎯 **Solution 1: Headless Bevy Test Harness** (RECOMMENDED)

Create a test harness that:
- Runs Bevy with `MinimalPlugins` (no window)
- Mocks `Assets<Image>` for atlas operations
- Captures mesh generation output to data structures
- Provides assertions on UI component state

**Pros:**
- Native Bevy testing
- No architecture changes needed
- Can test actual rendering logic

**Cons:**
- Requires Bevy expertise
- May need mocking infrastructure
- Some GPU operations might still require stubs

**Effort:** 2-3 weeks (Medium)

---

### 🎯 **Solution 2: Rendering Abstraction Layer** (ARCHITECTURAL)

Refactor rendering to separate:
```rust
// Pure data (testable)
pub struct MeshData {
    positions: Vec<[f32; 3]>,
    uvs: Vec<[f32; 2]>,
    colors: Vec<[f32; 4]>,
    indices: Vec<u32>,
}

// Pure function
fn generate_mesh_data(state: &SharedState) -> MeshData

// Bevy adapter
impl From<MeshData> for Mesh { ... }
```

**Pros:**
- Enables pure unit testing
- Better architecture long-term
- Future-proof for other backends

**Cons:**
- Significant refactoring required
- May impact performance slightly
- Needs careful design

**Effort:** 3-4 weeks (Large)

---

### 🎯 **Solution 3: Extend Ratatui-testlib for Bevy** (HYBRID)

Enhance ratatui-testlib with:
- Bevy ECS integration (query components)
- Mesh data extraction
- UI component assertions
- Snapshot testing for Bevy UIs

**Pros:**
- Leverages existing tool
- Reusable for other Bevy TUIs
- Could benefit ratatui-testlib project

**Cons:**
- Ratatui-testlib still in early development
- May not fit its core mission
- Adds complexity to dependency

**Effort:** 4-5 weeks (Large + External Dependency)

---

## Recommendations by Priority

### 🔴 **IMMEDIATE (Week 1-2)**

1. **Spike: Headless Bevy Testing**
   - Create proof-of-concept test that:
     - Runs Bevy without window
     - Captures mesh data
     - Asserts on component state
   - **Deliverable:** `tests/headless_bevy_poc.rs`

2. **Document Rendering Architecture**
   - Create architecture diagram
   - Document mesh generation flow
   - Identify abstraction points
   - **Deliverable:** `docs/architecture/rendering-pipeline.md`

3. **Update CLAUDE.md**
   - Reflect actual 17 crates
   - Update build commands
   - Document testing strategy
   - **Deliverable:** Updated `CLAUDE.md`

### 🟡 **SHORT-TERM (Week 3-6)**

4. **Implement Headless Test Harness**
   - Based on POC, build reusable harness
   - Add to `scarab-client/tests/`
   - Write example tests for:
     - Command palette rendering
     - Link hints positioning
     - Overlay bounds checking
   - **Deliverable:** `TestHarness` in `scarab-client`

5. **Safe SharedState Access Layer**
   - Create `TerminalStateReader` trait
   - Implement for SharedState
   - Refactor systems to use trait
   - **Deliverable:** `scarab-client/src/integration/safe_reader.rs`

6. **Expand Fusabi Plugin API**
   - Add event hooks (focus, resize, etc.)
   - Add programmatic commands
   - Update plugin examples
   - **Deliverable:** `scarab-plugin-api` v0.2.0

### 🟢 **MEDIUM-TERM (Week 7-12)**

7. **Rendering Abstraction Layer** (if viable)
   - Extract pure mesh generation
   - Create Bevy adapters
   - Migrate existing code
   - **Deliverable:** Refactored rendering module

8. **Ratatui-testlib Bevy Integration**
   - Contribute Bevy features upstream
   - Integrate into scarab test suite
   - Document integration patterns
   - **Deliverable:** Enhanced ratatui-testlib support

9. **Comprehensive Test Suite**
   - Achieve 80% code coverage
   - Add snapshot tests for rendering
   - Performance regression tests
   - **Deliverable:** Full test suite

### 🔵 **LONG-TERM (3-6 months)**

10. **Fusabi Feature Parity with WezTerm Lua**
    - Implement missing event system
    - Add status bar APIs
    - Programmatic pane control
    - **Deliverable:** Fusabi v2.0 with full event system

11. **Visual Regression Testing**
    - Screenshot comparison tests
    - Pixel-perfect UI verification
    - CI integration
    - **Deliverable:** Visual test suite

12. **Performance Benchmarking Dashboard**
    - Track metrics over time
    - Detect regressions automatically
    - Public dashboard
    - **Deliverable:** CI performance tracking

---

## Success Metrics

### 🎯 **Primary Goal: Close the Frontend Testing Loop**

**Definition of Success:**
- Developers can run `cargo test` and verify UI changes
- Tab bar, overlays, command palette tested automatically
- No manual client launches needed for verification
- Test suite catches UI regressions

**Measurement:**
- Frontend test coverage > 70%
- UI component tests exist for all major features
- CI blocks PRs with failing UI tests

### 📊 **Secondary Goals**

1. **Code Coverage**
   - Overall: > 70%
   - Rendering: > 60%
   - Integration: > 50%

2. **Documentation**
   - All crates documented
   - Architecture diagrams complete
   - Testing guide published

3. **Fusabi Capabilities**
   - Event system implemented
   - 10+ new plugin hooks
   - Feature parity with 80% of wezterm Lua capabilities

---

## Risk Assessment

### 🔴 **HIGH RISK**

1. **Headless Bevy Testing May Not Be Viable**
   - Bevy may require GPU for certain operations
   - Mocking might be too complex
   - **Mitigation:** Prototype early, have backup plans

2. **Rendering Refactor May Break Things**
   - Large changes = high regression risk
   - Performance impact unknown
   - **Mitigation:** Incremental changes, extensive benchmarking

### 🟡 **MEDIUM RISK**

3. **Ratatui-testlib Development Stalls**
   - External dependency
   - Early stage project
   - **Mitigation:** Don't block on it, build our own if needed

4. **Fusabi Upstream Changes**
   - API changes could break plugins
   - Limited control over upstream
   - **Mitigation:** Pin versions, maintain compatibility layer

### 🟢 **LOW RISK**

5. **Documentation Effort**
   - Low technical risk
   - Clear deliverables
   - **Mitigation:** N/A

---

## Conclusion

Scarab has a **solid architectural foundation** with excellent modularity and a well-designed plugin system. However, the **frontend testing gap is critical** and must be addressed to achieve the project's quality goals.

**The core issue:** Developers cannot verify UI changes without manually running the client. This creates a testing loop that cannot be closed through automation.

**Recommended approach:**
1. **Immediate:** Prove headless Bevy testing is viable (POC)
2. **Short-term:** Build test harness and add UI tests
3. **Medium-term:** Consider rendering abstraction for long-term maintainability
4. **Long-term:** Expand Fusabi and ratatui-testlib capabilities

**Expected timeline to close testing loop:** 4-6 weeks

**Expected timeline for comprehensive testing:** 3-4 months

---

## Document Index

This audit consists of multiple detailed documents:

1. **00-EXECUTIVE-SUMMARY.md** (this document) - High-level findings and recommendations
2. **01-RENDERING-ARCHITECTURE.md** - Detailed rendering pipeline analysis
3. **02-TESTING-INFRASTRUCTURE.md** - Current test coverage and gaps
4. **03-FUSABI-VS-WEZTERM.md** - Feature comparison and gap analysis
5. **04-RATATUI-TESTLIB-ANALYSIS.md** - Capabilities and integration requirements
6. **05-FRONTEND-TESTING-SOLUTIONS.md** - Detailed proposals for closing the loop
7. **06-RATATUI-TESTLIB-ISSUES.md** - Feature requests for ratatui-testlib
8. **07-FUSABI-ISSUES.md** - Feature requests for fusabi-lang/fusabi
9. **08-ACTION-PLAN.md** - Week-by-week implementation roadmap

---

**Generated:** 2025-12-01 by Claude Sonnet 4.5
**Total Analysis Time:** ~2 hours
**Codebase Review:** ~100 files
**Research Conducted:** WezTerm, Ratatui, Bevy testing patterns
**Recommendations:** 12 actionable items with timelines
