# Scarab Technical Audit - Action Plan
**Date:** December 1, 2025
**Timeline:** 12 weeks to complete all critical issues

---

## Quick Reference

### Critical Path (Must Do)
1. ✅ **Week 1-2:** Prove headless Bevy testing works (POC)
2. ✅ **Week 3-4:** Build reusable test harness
3. ✅ **Week 5-6:** Write comprehensive UI tests
4. ⏸️ **Week 7-8:** Safe SharedState abstraction
5. ⏸️ **Week 9-12:** Documentation and Fusabi enhancements

### Success Criteria
- [ ] Frontend tests run without manual client launch
- [ ] `cargo test` verifies tab bars, overlays, command palette
- [ ] CI blocks PRs with failing UI tests
- [ ] Test coverage > 70% overall, > 60% rendering

---

## Week-by-Week Roadmap

### 🔴 **WEEK 1: Headless Bevy POC**

**Goal:** Prove headless testing is viable

**Tasks:**
1. [ ] Create `crates/scarab-client/tests/headless_poc.rs`
2. [ ] Test 1: Bevy runs with `MinimalPlugins` (no window)
3. [ ] Test 2: Spawn and query basic components
4. [ ] Test 3: Mock `Assets<Image>` successfully
5. [ ] Test 4: Run integration plugin (LinkHints or CommandPalette)
6. [ ] Document findings in `docs/testing/headless-bevy-poc.md`

**Deliverable:** POC tests passing in CI

**Success Criteria:**
- Tests pass without `DISPLAY` environment variable
- Can query spawned Bevy components
- Assets can be mocked
- No GPU errors

**Risk Mitigation:**
- If Bevy requires GPU: Document limitations, plan for rendering abstraction
- If Assets can't be mocked: Build custom mock Assets implementation

---

### 🔴 **WEEK 2: Test Harness Foundation**

**Goal:** Build reusable harness for all tests

**Tasks:**
1. [ ] Create `crates/scarab-client/tests/harness/mod.rs`
2. [ ] Implement `HeadlessTestHarness` struct
   - `new()` - Setup Bevy with minimal plugins
   - `update()` - Run one Bevy frame
   - `query<T: Component>()` - Query ECS
   - `send_event<E: Event>()` - Trigger events
3. [ ] Add mock `SharedState` reader
4. [ ] Add mock `Assets<Image>` for atlas
5. [ ] Write 5 example tests:
   - Spawn and query Node
   - Trigger and verify event
   - Query multiple components
   - Assert component count
   - Assert component properties

**Deliverable:** Reusable `HeadlessTestHarness`

**Documentation:**
- `tests/harness/README.md` - Usage guide
- Inline doc comments
- Example tests

---

### 🔴 **WEEK 3: UI Component Tests (Part 1)**

**Goal:** Test half of major UI components

**Tasks:**
1. [ ] **Command Palette Tests** (`tests/ui/command_palette_tests.rs`)
   - Opens on Ctrl+Shift+P
   - Shows correct number of commands
   - Filters by search query
   - Executes selected command

2. [ ] **Link Hints Tests** (`tests/ui/link_hints_tests.rs`)
   - Detects URLs in grid
   - Spawns hint entities
   - Positions hints correctly
   - Activates on key press

3. [ ] **Overlays Tests** (`tests/ui/overlays_tests.rs`)
   - Renders daemon overlays
   - Positions within bounds
   - Clears when dismissed
   - Z-index ordering correct

**Deliverable:** 15+ tests for 3 components

**Coverage Target:** 60% of UI code

---

### 🔴 **WEEK 4: UI Component Tests (Part 2)**

**Goal:** Complete UI test coverage

**Tasks:**
1. [ ] **Visual Selection Tests** (`tests/ui/selection_tests.rs`)
   - Character mode selection
   - Line mode selection
   - Block mode selection
   - Selection rendering

2. [ ] **Scroll Indicator Tests** (`tests/ui/scroll_tests.rs`)
   - Appears when scrolled
   - Shows correct position
   - Hides when at live view

3. [ ] **Tutorial Tests** (`tests/ui/tutorial_tests.rs`)
   - Displays on first run
   - Advances through steps
   - Validates completion
   - Dismissable

**Deliverable:** 30+ total UI tests

**Coverage Target:** 70% of UI code

---

### 🟡 **WEEK 5: Integration & Snapshot Tests**

**Goal:** Add higher-level integration tests

**Tasks:**
1. [ ] **Integration Tests** (`tests/integration/`)
   - Full app lifecycle (startup → shutdown)
   - Multi-component interactions
   - Event chains (e.g., Ctrl+Shift+P → type → Enter → command runs)

2. [ ] **Snapshot Tests** (`tests/snapshots/`)
   - Component layout snapshots (with `insta`)
   - Mesh data snapshots (vertex counts, bounds)
   - UI state snapshots (for regression detection)

3. [ ] **Performance Tests** (`tests/performance/`)
   - Rendering benchmarks
   - FPS assertions (must maintain 60+ FPS)
   - Memory usage checks

**Deliverable:** 10+ integration tests, 20+ snapshots

---

### 🟡 **WEEK 6: Safe SharedState Access Layer**

**Goal:** Eliminate unsafe pointer dereference

**Tasks:**
1. [ ] Design `TerminalStateReader` trait
   ```rust
   pub trait TerminalStateReader {
       fn cells(&self) -> &[Cell];
       fn cursor_pos(&self) -> (u16, u16);
       fn sequence(&self) -> u64;
       fn is_valid(&self) -> bool;
   }
   ```

2. [ ] Implement for `SharedState` (with safety checks)
3. [ ] Create `MockTerminalState` for tests
4. [ ] Refactor all systems to use trait instead of raw pointers
5. [ ] Add validation (bounds checking, magic number verification)

**Deliverable:** Safe abstraction layer

**Coverage Target:** 100% of SharedState access

---

### 🟡 **WEEK 7: Documentation**

**Goal:** Comprehensive testing and architecture docs

**Tasks:**
1. [ ] **Testing Guide** (`docs/testing/GUIDE.md`)
   - How to write headless tests
   - Common patterns and examples
   - Troubleshooting
   - CI integration

2. [ ] **Architecture Diagrams**
   - Rendering pipeline flowchart
   - Component interaction diagrams
   - IPC data flow

3. [ ] **Update CLAUDE.md**
   - Reflect 17 crates (not 5)
   - Update test commands
   - Document headless testing

**Deliverable:** Complete testing documentation

---

### 🟢 **WEEK 8: Fusabi API Enhancements (Phase 1)**

**Goal:** Add critical Fusabi features

**Tasks:**
1. [ ] File issues in fusabi-lang/fusabi (from 07-FUSABI-ISSUES.md)
2. [ ] If upstream accepts, contribute:
   - Event system foundation
   - Terminal query API basics

3. [ ] Update scarab-plugin-api to expose new capabilities
4. [ ] Write example plugins using new APIs
5. [ ] Update plugin documentation

**Deliverable:** Enhanced plugin capabilities

---

### 🟢 **WEEK 9-10: Rendering Optimization**

**Goal:** Enable dirty region tracking and fix atlas

**Tasks:**
1. [ ] **Enable Dirty Region Optimization**
   - Fix `mark_full_redraw()` calls
   - Implement cell-level dirty tracking
   - Benchmark performance improvement

2. [ ] **Dynamic Atlas Expansion**
   - Implement multi-atlas support
   - Fallback to new atlas when full
   - Test with emoji-heavy content

3. [ ] **Performance Profiling**
   - Run Tracy profiler
   - Identify bottlenecks
   - Optimize hot paths

**Deliverable:** 2x rendering performance improvement

---

### 🟢 **WEEK 11: Ratatui-testlib Integration**

**Goal:** Enhance ratatui-testlib for Bevy (if viable)

**Tasks:**
1. [ ] File issues in ratatui-testlib (from 06-RATATUI-TESTLIB-ISSUES.md)
2. [ ] If upstream interested, contribute:
   - Bevy ECS integration
   - Headless mode support
   - Position assertions

3. [ ] Integrate ratatui-testlib into scarab test suite
4. [ ] Document integration patterns

**Deliverable:** Enhanced testing capabilities

**Fallback:** Use internal harness only

---

### 🟢 **WEEK 12: Final Polish & Release**

**Goal:** Prepare for release with comprehensive testing

**Tasks:**
1. [ ] **Code Coverage Analysis**
   - Run `cargo tarpaulin`
   - Identify gaps
   - Add missing tests
   - Target: 80% overall

2. [ ] **CI/CD Hardening**
   - Add matrix testing (Linux, macOS, Windows)
   - Add performance regression checks
   - Add test result reporting

3. [ ] **Release Prep**
   - Changelog updates
   - Version bumps
   - Release notes
   - Blog post draft

**Deliverable:** Production-ready testing infrastructure

---

## Parallel Tracks

### Track A: Critical Testing (Weeks 1-6)
**Owner:** Primary developer
**Focus:** Close the frontend testing loop
**Blockers:** None (can start immediately)

### Track B: Architecture Improvements (Weeks 6-10)
**Owner:** Can be second developer or same
**Focus:** Safe abstractions, performance
**Blockers:** Track A completion (needs test coverage first)

### Track C: Ecosystem Contributions (Weeks 8-11)
**Owner:** Community/team collaboration
**Focus:** Upstream improvements
**Blockers:** Depends on upstream response

---

## Risk Management

### HIGH RISK: Headless Bevy Doesn't Work

**Mitigation:**
- Week 1 POC explicitly tests this
- If fails: Pivot to rendering abstraction layer (add 2-3 weeks)
- Fallback: Visual snapshot testing with screenshots (requires X11, harder CI)

### MEDIUM RISK: Upstream Dependencies Unresponsive

**Mitigation:**
- Don't block on fusabi-lang or ratatui-testlib
- Build internal solutions if needed
- Contribute upstream when possible (best effort)

### LOW RISK: Performance Regression

**Mitigation:**
- Benchmark early and often
- Continuous performance monitoring in CI
- Rendering abstraction can help isolate performance

---

## Success Metrics

### Week 4 Checkpoint
- [ ] 30+ frontend tests passing
- [ ] Tests run in < 10 seconds
- [ ] No manual client launch needed

### Week 8 Checkpoint
- [ ] 50+ total tests
- [ ] Safe SharedState abstraction complete
- [ ] Documentation published

### Week 12 Completion
- [ ] 80% code coverage
- [ ] All critical UI testable
- [ ] CI enforces testing
- [ ] Fusabi enhancements live (or planned)

---

## Resources Needed

### Time
- **1 full-time developer:** Weeks 1-6 (critical path)
- **0.5 FTE:** Weeks 7-12 (parallel tracks OK)

### Tools
- None (all Rust tooling already in place)

### External Dependencies
- Upstream response (fusabi, ratatui-testlib) - nice to have, not blocking

---

## Next Steps (This Week)

### Monday-Tuesday
- [ ] Review audit with team
- [ ] Approve action plan
- [ ] Assign owner

### Wednesday
- [ ] Create Week 1 task branch
- [ ] Begin POC implementation

### Thursday-Friday
- [ ] Complete POC tests
- [ ] Document findings
- [ ] Plan Week 2 work

---

## Long-Term Vision (Beyond 12 Weeks)

### Months 4-6
- Visual regression testing (screenshot comparison)
- Fuzzing for input handling
- Property-based testing for state machines
- Continuous benchmarking dashboard

### Months 7-12
- Fusabi feature parity with wezterm (if upstream accepts)
- Plugin ecosystem growth
- Performance optimization (target: 120 FPS)
- macOS and Windows support

---

**Document:** 08-ACTION-PLAN.md
**Status:** Ready to execute
**Start Date:** Week of December 2, 2025
**Expected Completion:** Week of February 24, 2026 (12 weeks)
