# Scarab Meta-Orchestration Status Report
**Generated:** 2025-12-02
**Orchestrator:** Meta-Coach AI
**Sprint Start:** Day 0

## Executive Summary

The Scarab terminal emulator project is ready to execute a 4-track parallel development roadmap. The foundation (multiplexing architecture Phase 1) is COMPLETE, providing stable infrastructure for all future work. This report analyzes current codebase state, identifies immediate next steps, and assigns initial agent tasks.

## Codebase Health Assessment

### Architecture Status: GREEN
- **Multiplexing Foundation:** COMPLETE (commits 662d519 → 105c1c6)
  - Session/Tab/Pane hierarchy implemented
  - PaneOrchestrator for parallel PTY reading
  - Compositor blitting active pane @ 60fps
  - IPC commands wired to SessionManager

- **Split Architecture:** OPERATIONAL
  - Daemon owns PTY processes and grid state
  - Client renders via shared memory (zero-copy)
  - Unix socket for control messages

- **Protocol Layer:** STABLE
  - `SharedState` with `#[repr(C)]` layout
  - Atomic sequence numbers for sync
  - Control/Daemon message enums defined

### Current Capabilities
- Multiple sessions/tabs/panes functional
- Parallel PTY reading (all panes, not just active)
- VTE parsing with Alacritty backend
- Bevy 0.15 rendering pipeline
- Plugin API (Fusabi scripting) infrastructure
- IPC command routing complete

### Missing Critical Features (from ROADMAP-AI.md)
1. **Image Rendering Pipeline** - Parser exists, end-to-end pipeline missing
2. **Ligatures Verification** - cosmic-text supports it, needs testing
3. **Headless Test Harness** - No stable CI infrastructure yet
4. **Ratatui UI Bridge** - Overlay system not ECS-native
5. **ECS Grid Chunking** - Whole-grid rebuild on every frame

## Track Status Analysis

### Track D (Phase 0): Foundation Infrastructure - PRIORITY 1

**Status:** NOT STARTED (BLOCKING ALL OTHER TRACKS)

This phase must complete FIRST before launching parallel work on Tracks A, B, C.

| Task | Status | Blocker | Agent Needed |
|------|--------|---------|--------------|
| D1: Stabilize headless harness | NOT STARTED | No ScheduleRunner test setup | test-writer-fixer |
| D2: Telemetry/logging knobs | NOT STARTED | No compositor visibility | devops-automator |

**Critical Path:** D1 + D2 must complete before S1 sprint begins.

**Risk Assessment:** MEDIUM
- No existing headless test infrastructure found in `/crates/scarab-client/tests/`
- ratatui-testlib issues #14-#16 may not be resolved
- Need local fallback harness (ScheduleRunner + snapshot)

**Immediate Actions Required:**
1. Spawn `test-writer-fixer` agent to audit current test infrastructure
2. Create minimal headless harness using Bevy's ScheduleRunnerPlugin
3. Add telemetry for compositor blit operations (fps, dirty regions)
4. Establish snapshot baseline for future golden tests

---

### Track A: Media Pipeline (Phase 1)

**Status:** READY TO START (after Phase 0)

**Dependencies Met:**
- Protocol layer stable
- Daemon VTE parser operational
- Client Bevy rendering functional

**Current State Analysis:**

| Component | Status | Evidence |
|-----------|--------|----------|
| iTerm2 Parser | EXISTS | `scarab-daemon/src/images/iterm2.rs` |
| ImagePlacement struct | DEFINED | `scarab-protocol/src/lib.rs` lines 565-587 |
| Shared memory image buffer | MISSING | Protocol only has `cells` array |
| Client image rendering | MISSING | No `ImagesPlugin` implementation found |
| Ligature support | UNKNOWN | cosmic-text feature unclear |

**Gaps Identified:**
1. **Protocol Extension (A1):** `SharedImageBuffer` struct not in protocol
2. **Daemon Integration (A2):** iTerm2 parser not wired to VTE
3. **Client Rendering (A3):** No Bevy texture upload system
4. **Ligature Testing (A4):** No golden test for Fira Code

**Agent Assignment:**
- A1: `backend-architect` (protocol design)
- A2: `backend-architect` (daemon integration)
- A3: `frontend-developer` (Bevy textures/sprites)
- A4: `test-writer-fixer` (golden test with headless harness)

**Blockers:** A4 requires D1 (headless harness) to complete first

---

### Track B: ECS Grid & Dirty Diff (Phase 2)

**Status:** READY TO START (after Phase 0)

**Current Rendering Analysis:**

From `/crates/scarab-client/src/main.rs`:
- Client uses `IntegrationPlugin` for terminal rendering
- Shared memory read via `SharedMemoryReader` resource
- No evidence of chunked grid or dirty region tracking

**Problem:** Entire grid likely rebuilt every frame (performance issue)

**Required Architecture:**
```rust
// New ECS components needed
struct TerminalChunk {
    chunk_id: u32,          // e.g., 0-15 for 4x4 grid of chunks
    origin: (u16, u16),     // Top-left cell position
    mesh_handle: Handle<Mesh>,
    dirty: bool,
}

// Dirty region marking system
fn mark_dirty_chunks(
    reader: Res<SharedMemoryReader>,
    mut chunks: Query<&mut TerminalChunk>,
) {
    // Only mark chunks that overlap dirty region
}
```

**Agent Assignment:**
- B1-B4: `frontend-developer` (ECS architecture)
- B5: `test-writer-fixer` (headless chunk rebuild verification)

**Blockers:** B5 requires D1 (headless harness)

---

### Track C: UI Layer (Phases 3 + 4)

**Status:** PARTIALLY STARTED

**Current Plugin System Analysis:**

From `/crates/scarab-client/src/main.rs`:
- `AdvancedUIPlugin` exists (search, indicators)
- `TutorialPlugin` exists
- `ScriptingPlugin` exists
- Plugin system uses `Arc<Mutex<EventRegistry>>` (not ECS-native)

**Gap:** Plugins not integrated with Bevy ECS properly

**Phase 3 (Ratatui Bridge) - NOT STARTED:**
- No `scarab-ratatui-bridge` crate found
- Overlays likely not Ratatui-based
- Command palette implementation unknown

**Phase 4 (Plugin Alignment) - PARTIAL:**
- Plugin infrastructure exists but not ECS-aligned
- Event system uses mutexes instead of Bevy events/resources
- No `ScarabPluginHostPlugin` found

**Agent Assignment:**
- C1-C5: `frontend-developer` + `ui-designer` (Ratatui bridge)
- C6-C9: `backend-architect` (plugin host refactor)

**Blockers:** None (can start in S1), but C4 benefits from D1 for testing

---

### Track D: Infrastructure (Phases 5 + 6)

**Phase 0:** BLOCKING (see above)

**Phase 5 (Shell Integration) - NOT STARTED:**
- OSC 133 markers not implemented
- No gutter rendering found
- No post-process shader support

**Phase 6 (Headless CI) - NOT STARTED:**
- No golden tests found
- No `headless_runner.rs` exists
- Depends on A3/A4 for image/ligature content

**Agent Assignment:**
- D3: `backend-architect` (OSC 133 daemon)
- D4-D5: `frontend-developer` (client markers, shader)
- D6-D8: `test-writer-fixer` (CI infrastructure)

**Blockers:**
- D6-D8 requires A3/A4 to complete first (need content to snapshot)

---

## Dependency Matrix

```
Phase 0 (D1+D2)
    ├─── BLOCKS ───┐
    │              │
    ▼              ▼
  Track A      Track B
    │              │
    │              └──── B5 needs D1
    │
    └──── A4 needs D1

Track C (independent start)
    └──── C4 benefits from D1

Track D Phase 6
    └──── D6-D8 needs A3+A4
```

## Sprint Plan

### Sprint 0 (IMMEDIATE - 1-2 days)
**Goal:** Complete Phase 0 Foundation

**Tasks:**
1. **D1: Headless Harness** (test-writer-fixer)
   - Audit `/crates/scarab-client/tests/` directory
   - Create `tests/headless_runner.rs` with ScheduleRunnerPlugin
   - Implement grid snapshot function
   - Add basic smoke test (render "Hello World")

2. **D2: Telemetry Knobs** (devops-automator)
   - Add compositor fps logging (INFO level)
   - Add sequence_number change logging
   - Add dirty region size tracking
   - Create example config showing telemetry options

**Success Criteria:**
- Can run `cargo test --test headless_runner` and get grid snapshot
- Daemon logs show fps and blit activity

**Estimated Time:** 8-16 hours

---

### Sprint 1 (After S0 - 2-3 days)
**Goal:** Launch all 4 tracks in parallel

**Track A (Media Pipeline):**
- A1: Protocol extension (backend-architect)
- A2: Daemon iTerm2 integration (backend-architect)

**Track B (ECS Grid):**
- B1: TerminalChunk component design (frontend-developer)
- B2: Dirty chunk marking system (frontend-developer)

**Track C (UI Layer):**
- C1: Create `scarab-ratatui-bridge` module (frontend-developer)
- C2: RatatuiSurface component (frontend-developer)

**Track D (Infrastructure):**
- D3: OSC 133 markers daemon (backend-architect)

**Success Criteria:**
- A1: `SharedImageBuffer` defined in protocol
- B1: `TerminalChunk` component compiles
- C1: Bridge crate scaffolded
- D3: OSC 133 parsing works

---

### Sprint 2 (3-4 days)

**Track A:**
- A3: Client image rendering (frontend-developer)
- A4: Ligature golden test (test-writer-fixer)

**Track B:**
- B3: Per-chunk mesh generation (frontend-developer)
- B4: TerminalMetrics chunk-aware (frontend-developer)

**Track C:**
- C3: Input mapping Bevy → Ratatui (frontend-developer)
- C4: Command palette prototype (ui-designer)

**Track D:**
- D4: Client OSC markers (frontend-developer)
- D5: Post-process shader (frontend-developer)

---

### Sprint 3 (3-4 days)

**Track B:**
- B5: Headless chunk test (test-writer-fixer)

**Track C:**
- C5: Ratatui overlay docs (frontend-developer)
- C6: ScarabPluginHostPlugin (backend-architect)
- C7: Fusabi → ECS events (backend-architect)

**Track D:**
- D6: Headless runner enhancement (test-writer-fixer)
- D7: iTerm2 image golden test (test-writer-fixer)
- D8: Ligature golden test (test-writer-fixer)

---

### Sprint 4-5 (Cleanup)

**Track C:**
- C8: Replace Arc<Mutex> with ECS (backend-architect)
- C9: Port plugins to Bevy form (frontend-developer)

---

## Risk Register

| Risk | Severity | Probability | Mitigation |
|------|----------|-------------|------------|
| Image buffer memory explosion | HIGH | MEDIUM | Enforce 16-32MB cap, count limit |
| Chunked grid perf regression | MEDIUM | LOW | Profile mesh gen, coarse chunk fallback |
| Ratatui bridge scope creep | MEDIUM | MEDIUM | Stay thin, accept any Widget |
| ratatui-testlib blocked | LOW | HIGH | Local headless harness (D1) |
| cosmic-text ligatures broken | MEDIUM | LOW | Feature flag, manual test |
| Parallel track merge conflicts | MEDIUM | MEDIUM | Clear file boundaries, frequent sync |

## Agent Resource Allocation

### Immediate (Sprint 0):
- `test-writer-fixer`: D1 (headless harness) - 6 hours
- `devops-automator`: D2 (telemetry) - 2 hours

### Sprint 1 (Parallel):
- `backend-architect`: A1, A2, D3 (12 hours)
- `frontend-developer`: B1, B2, C1, C2 (16 hours)
- `ui-designer`: Standby (waiting for C4)
- `test-writer-fixer`: Standby (waiting for A4)

### Ongoing:
- Meta-orchestrator: Daily sync, blocker resolution

## Coordination Checkpoints

### Daily Standup (Async):
- Each agent reports: completed tasks, blockers, next 24hr plan
- Meta-orchestrator identifies cross-track dependencies
- Resolve merge conflicts proactively

### Sprint Boundary (End of S0, S1, S2, S3):
- Review success criteria
- Adjust resource allocation
- Validate track synchronization points

### Integration Points:
- After A1: Validate protocol structs with backend + frontend
- After B1: Review ECS component design
- After C1: Architecture review (Ratatui integration)
- After D1: All agents can use headless harness

## Next Actions (IMMEDIATE)

### 1. Bootstrap Phase 0

I will spawn two agents to complete Track D Phase 0:

**Agent 1: test-writer-fixer**
```
TASK: D1 - Stabilize headless test harness
DELIVERABLE: /crates/scarab-client/tests/headless_runner.rs
REQUIREMENTS:
- Use Bevy ScheduleRunnerPlugin
- Implement grid snapshot function
- Add smoke test rendering "Hello World"
- Document snapshot format
TIME BOX: 6 hours
```

**Agent 2: devops-automator**
```
TASK: D2 - Add telemetry and logging knobs
DELIVERABLE: Enhanced logging in compositor + example config
REQUIREMENTS:
- Compositor fps logging (INFO level)
- Sequence number change tracking
- Dirty region size metrics
- Example config with telemetry options
TIME BOX: 2 hours
```

### 2. Validation Gate

Before proceeding to Sprint 1, verify:
- [ ] `cargo test --test headless_runner` passes
- [ ] Daemon logs show fps metrics
- [ ] Grid snapshot can be written/read
- [ ] No regressions in existing tests

### 3. Launch Sprint 1

Once Phase 0 is complete, spawn 3 agents simultaneously:
- `backend-architect`: A1 + A2 + D3
- `frontend-developer`: B1 + B2 + C1 + C2
- `test-writer-fixer`: Monitor harness stability

## Success Metrics

### Phase 0 (Foundation):
- Headless harness passing smoke test
- Telemetry visible in daemon logs

### Phase 1 (Images):
- iTerm2 image renders in client
- Ligature golden test passes

### Phase 2 (ECS Grid):
- Only dirty chunks rebuild (verified by test)

### Phase 3 (Ratatui Bridge):
- Command palette overlay renders

### Phase 4 (Plugin Alignment):
- Link-hints plugin works as Bevy plugin

### Phase 5 (Shell Integration):
- OSC 133 markers visible in gutter

### Phase 6 (CI):
- Headless snapshots green in CI

---

## Files Requiring Immediate Attention

### Phase 0 (Sprint 0):
1. `/crates/scarab-client/tests/headless_runner.rs` - CREATE
2. `/crates/scarab-daemon/src/main.rs` - MODIFY (add telemetry)
3. `/crates/scarab-config/examples/telemetry.toml` - CREATE

### Sprint 1 Files:
4. `/crates/scarab-protocol/src/lib.rs` - MODIFY (add SharedImageBuffer)
5. `/crates/scarab-daemon/src/images/mod.rs` - MODIFY (wire iTerm2)
6. `/crates/scarab-client/src/terminal/` - MODIFY (add TerminalChunk)
7. `/crates/scarab-ratatui-bridge/` - CREATE (new module/crate)
8. `/crates/scarab-daemon/src/vte.rs` - MODIFY (OSC 133)

---

## Recommendation

**PROCEED WITH PHASE 0 IMMEDIATELY**

The codebase is healthy and ready for parallel work, but Track D Phase 0 (headless harness + telemetry) is the critical bottleneck. Without it:
- Track A (ligatures) cannot verify results
- Track B (chunked grid) cannot measure perf
- Track C (overlays) cannot test rendering
- Track D Phase 6 (CI) has no foundation

**Estimated Timeline:**
- Sprint 0 (Phase 0): 1-2 days
- Sprint 1-2 (Parallel tracks): 6-8 days
- Sprint 3 (Integration): 3-4 days
- Sprint 4-5 (Polish): 3-4 days

**Total:** 13-18 days to complete all 6 phases

**Confidence Level:** HIGH (architecture is sound, tasks are well-defined)

---

## Orchestrator Notes

This is a well-structured roadmap with clear dependencies. The main risk is attempting parallel work before Phase 0 completes. By enforcing the D1+D2 gate, we ensure all downstream work has stable testing infrastructure.

The agent assignments leverage specialization:
- `backend-architect`: Protocol, daemon, plugins
- `frontend-developer`: Client, ECS, rendering
- `ui-designer`: Ratatui widgets, overlays
- `test-writer-fixer`: Harness, golden tests, CI
- `devops-automator`: Logging, telemetry, config

This distribution balances load and minimizes context switching.

**Ready to execute.**
