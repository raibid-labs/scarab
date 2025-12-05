# Parallel Orchestration Plan - High Priority Work
**Date:** December 1, 2025
**Duration:** 6 weeks (Weeks 1-6 from Action Plan)
**Strategy:** Maximum parallelization with 4 concurrent agents

---

## Executive Summary

We'll orchestrate **4 parallel workstreams** using specialized agents to complete all critical/high priority work in 6 weeks instead of sequential 12 weeks.

**Workstreams:**
1. **Testing POC & Harness** (Weeks 1-4) - CRITICAL PATH
2. **Safe SharedState Refactoring** (Weeks 3-4) - Parallel to testing
3. **Upstream Issue Filing** (Week 1) - Non-blocking
4. **Documentation Updates** (Weeks 5-6) - Cleanup phase

---

## Agent Assignments

### Agent 1: test-writer-fixer (PRIMARY - CRITICAL PATH)
**Responsibility:** Frontend testing infrastructure
**Timeline:** Weeks 1-4
**Deliverables:**
- Headless Bevy POC
- Reusable test harness
- 30+ UI component tests

### Agent 2: backend-architect (PARALLEL)
**Responsibility:** Safe abstractions and architecture
**Timeline:** Weeks 3-4 (starts after POC proves viable)
**Deliverables:**
- TerminalStateReader trait
- Safe SharedState wrapper
- System refactoring

### Agent 3: rapid-prototyper (UPSTREAM)
**Responsibility:** Issue creation and documentation
**Timeline:** Week 1 (immediate, non-blocking)
**Deliverables:**
- Filed issues in fusabi-lang/fusabi
- Filed issues in ratatui-testlib
- Issue tracking setup

### Agent 4: frontend-developer (INTEGRATION)
**Responsibility:** UI test coverage
**Timeline:** Weeks 3-6 (works with test harness)
**Deliverables:**
- Command palette tests
- Link hints tests
- Overlay tests
- All UI component tests

---

## Week-by-Week Orchestration

### Week 1: POC + Issue Filing (Parallel Start)

**Agent 1 (test-writer-fixer):** Headless Bevy POC
```
Priority: P0 (BLOCKING)
Tasks:
  - Create tests/headless_poc.rs
  - Test 1: Bevy runs with MinimalPlugins
  - Test 2: Component querying works
  - Test 3: Assets can be mocked
  - Test 4: Integration plugin loads
  - Document findings
```

**Agent 3 (rapid-prototyper):** Upstream Issues
```
Priority: P2 (Non-blocking)
Tasks:
  - File 5 issues in ratatui-testlib
  - File 7 issues in fusabi-lang/fusabi
  - Create tracking document
  - Link to audit documents
```

**Blockers:** None - both can start immediately

---

### Week 2: Harness Foundation

**Agent 1 (test-writer-fixer):** Test Harness
```
Priority: P0 (BLOCKING)
Dependencies: Week 1 POC must pass
Tasks:
  - Create tests/harness/mod.rs
  - Implement HeadlessTestHarness
  - Mock SharedState reader
  - Mock Assets<Image>
  - Write 5 example tests
  - Document harness API
```

**Agent 3 (rapid-prototyper):** Documentation Prep
```
Priority: P3
Tasks:
  - Draft testing guide outline
  - Collect architecture diagrams needs
  - Prepare CLAUDE.md updates
```

**Blockers:** Agent 1 blocked if POC fails (fallback: rendering abstraction)

---

### Week 3: UI Tests (Part 1) + Safe Abstractions

**Agent 1 (test-writer-fixer):** Command Palette Tests
```
Priority: P0
Dependencies: Week 2 harness ready
Tasks:
  - tests/ui/command_palette_tests.rs
  - 5+ tests (opens, shows commands, filters, executes)
  - Snapshot tests
```

**Agent 4 (frontend-developer):** Link Hints Tests
```
Priority: P0
Dependencies: Week 2 harness ready
Tasks:
  - tests/ui/link_hints_tests.rs
  - 5+ tests (detection, positioning, activation)
  - Visual assertions
```

**Agent 2 (backend-architect):** Safe SharedState (START)
```
Priority: P1
Dependencies: None (independent work)
Tasks:
  - Design TerminalStateReader trait
  - Implement for SharedState
  - Add safety checks (bounds, magic numbers)
  - Create MockTerminalState for tests
```

**Parallelization:** 3 agents working simultaneously
**Risk:** If harness not ready, Agents 1+4 blocked

---

### Week 4: UI Tests (Part 2) + Safe Abstractions Complete

**Agent 1 (test-writer-fixer):** Overlay Tests
```
Priority: P0
Tasks:
  - tests/ui/overlays_tests.rs
  - tests/ui/selection_tests.rs
  - 10+ tests total
```

**Agent 4 (frontend-developer):** Scroll & Tutorial Tests
```
Priority: P0
Tasks:
  - tests/ui/scroll_tests.rs
  - tests/ui/tutorial_tests.rs
  - 10+ tests total
```

**Agent 2 (backend-architect):** System Refactoring
```
Priority: P1
Dependencies: Week 3 trait design complete
Tasks:
  - Refactor sync_terminal_state_system
  - Refactor update_terminal_rendering_system
  - Refactor all unsafe SharedState access
  - Add validation
  - PR and review
```

**Deliverable Checkpoint:** 30+ UI tests + Safe abstractions complete

---

### Week 5: Integration Tests + Documentation

**Agent 1 (test-writer-fixer):** Integration Tests
```
Priority: P1
Tasks:
  - tests/integration/full_lifecycle.rs
  - tests/integration/multi_component.rs
  - tests/integration/event_chains.rs
```

**Agent 4 (frontend-developer):** Snapshot Tests
```
Priority: P1
Tasks:
  - tests/snapshots/layout_snapshots.rs
  - tests/snapshots/mesh_snapshots.rs
  - Integrate insta
  - Generate golden files
```

**Agent 3 (rapid-prototyper):** Documentation
```
Priority: P2
Tasks:
  - docs/testing/GUIDE.md (how to write tests)
  - Update CLAUDE.md (17 crates, new structure)
  - Create architecture diagrams
  - Document headless testing
```

**Agent 2 (backend-architect):** Performance Tests
```
Priority: P2
Tasks:
  - tests/performance/rendering_bench.rs
  - FPS assertion tests
  - Memory usage checks
```

---

### Week 6: Polish + CI Integration

**ALL AGENTS:** Final Sprint
```
Agent 1 (test-writer-fixer):
  - Fix any failing tests
  - Achieve 70%+ coverage
  - Add missing test cases

Agent 2 (backend-architect):
  - Performance optimization
  - Code review responses
  - Final validation

Agent 3 (rapid-prototyper):
  - Complete documentation
  - CI/CD configuration
  - README updates

Agent 4 (frontend-developer):
  - UI polish tests
  - Edge case coverage
  - Integration with CI
```

**Deliverable:** Production-ready testing infrastructure

---

## Parallel Execution Strategy

### Critical Path (Cannot Parallelize)
```
Week 1: POC → Week 2: Harness → Weeks 3-4: Tests
```
**Duration:** 4 weeks
**Agent:** test-writer-fixer (primary)

### Parallel Track 1 (Safe Abstractions)
```
Week 3: Design → Week 4: Refactor
```
**Duration:** 2 weeks (starts Week 3)
**Agent:** backend-architect
**Blocker:** None (independent)

### Parallel Track 2 (Upstream Issues)
```
Week 1: File issues → Track upstream response
```
**Duration:** 1 week (non-blocking)
**Agent:** rapid-prototyper

### Parallel Track 3 (Documentation)
```
Week 5-6: Docs + CI
```
**Duration:** 2 weeks (after tests ready)
**Agent:** rapid-prototyper + backend-architect

---

## Resource Allocation

### Week 1-2: 2 Agents
- Agent 1: test-writer-fixer (100%)
- Agent 3: rapid-prototyper (50%)

### Week 3-4: 3 Agents (PEAK)
- Agent 1: test-writer-fixer (100%)
- Agent 2: backend-architect (100%)
- Agent 4: frontend-developer (100%)

### Week 5-6: 4 Agents
- Agent 1: test-writer-fixer (75%)
- Agent 2: backend-architect (75%)
- Agent 3: rapid-prototyper (100%)
- Agent 4: frontend-developer (75%)

---

## Risk Management

### Risk 1: POC Fails (Week 1)
**Impact:** CRITICAL (blocks everything)
**Probability:** LOW (Bevy supports headless)
**Mitigation:**
- Allocate extra time in Week 1
- Have rendering abstraction as fallback
- Consult Bevy community if issues

### Risk 2: Harness More Complex Than Expected (Week 2)
**Impact:** HIGH (delays UI tests)
**Probability:** MEDIUM
**Mitigation:**
- Start with minimal harness
- Iterate and improve
- Agent 4 can help Agent 1 if needed

### Risk 3: Safe Abstractions Break Systems (Week 4)
**Impact:** MEDIUM (delays refactoring)
**Probability:** MEDIUM
**Mitigation:**
- Extensive testing before refactor
- Incremental migration
- Easy to rollback

### Risk 4: Test Coverage Below Target (Week 6)
**Impact:** MEDIUM (delays completion)
**Probability:** LOW
**Mitigation:**
- Monitor coverage weekly
- Prioritize high-value tests
- Accept 70% vs 80% if needed

---

## Communication & Synchronization

### Daily Standups (Async)
Each agent posts progress:
- What was completed
- What's in progress
- Any blockers

### Weekly Checkpoints
- Week 1: POC results review
- Week 2: Harness API review
- Week 3: Test count checkpoint
- Week 4: Safe abstractions PR review
- Week 5: Integration review
- Week 6: Final review

### Coordination Points
- **Week 2 → Week 3:** Harness must be ready for Agents 1+4
- **Week 3 → Week 4:** Safe trait design must be approved
- **Week 4 → Week 5:** UI tests must be 80% complete

---

## Success Metrics

### Week 2 Checkpoint
- [ ] POC proves headless testing viable
- [ ] Harness can spawn and query components
- [ ] 5+ example tests passing

### Week 4 Checkpoint
- [ ] 30+ UI component tests passing
- [ ] Safe SharedState abstraction complete
- [ ] Tests run in < 10 seconds

### Week 6 Completion
- [ ] 50+ total tests (UI + integration)
- [ ] 70%+ code coverage
- [ ] All systems use safe abstractions
- [ ] Documentation complete
- [ ] CI enforces testing

---

## Agent Launch Commands

### Week 1 (Immediate)

**Agent 1 - Test Writer (POC):**
```bash
Launch: test-writer-fixer
Task: Create headless Bevy POC in tests/headless_poc.rs
Prove: Bevy runs without window, components queryable, assets mockable
Deliverable: 4 passing POC tests + findings doc
```

**Agent 3 - Upstream Issues:**
```bash
Launch: rapid-prototyper
Task: File issues in fusabi-lang/fusabi and ratatui-testlib
Source: docs/audits/claude-2025-12-01/06-*.md and 07-*.md
Deliverable: All issues filed with tracking doc
```

### Week 2

**Agent 1 - Test Harness:**
```bash
Launch: test-writer-fixer
Task: Build HeadlessTestHarness in tests/harness/mod.rs
API: new(), update(), query(), send_event(), assertions
Deliverable: Reusable harness + 5 example tests
```

### Week 3

**Agent 1 - Command Palette Tests:**
```bash
Launch: test-writer-fixer
Task: Write tests/ui/command_palette_tests.rs
Coverage: Opens, filters, executes, snapshots
Deliverable: 5+ tests
```

**Agent 4 - Link Hints Tests:**
```bash
Launch: frontend-developer
Task: Write tests/ui/link_hints_tests.rs
Coverage: Detection, positioning, activation
Deliverable: 5+ tests
```

**Agent 2 - Safe Abstractions:**
```bash
Launch: backend-architect
Task: Design TerminalStateReader trait + implementation
Safety: Bounds checking, validation
Deliverable: Trait + SharedState impl + MockState
```

---

## Deliverables Timeline

```
Week 1: ✅ POC + Issues filed
Week 2: ✅ Test harness ready
Week 3: ✅ 15+ UI tests + Safe trait designed
Week 4: ✅ 30+ UI tests + Refactoring complete
Week 5: ✅ Integration tests + Documentation
Week 6: ✅ 50+ tests + CI + Polish
```

---

## Next Actions (Execute Now)

1. **Launch Agent 1 (test-writer-fixer)** - POC creation
2. **Launch Agent 3 (rapid-prototyper)** - Issue filing
3. **Set up progress tracking** - Weekly checkpoints
4. **Monitor Agent 1 POC** - Critical path dependency

---

**Document:** 09-PARALLEL-ORCHESTRATION.md
**Status:** Ready to execute
**Expected Duration:** 6 weeks (vs 12 sequential)
**Speedup:** 2x through parallelization
