# Scarab Parallel Orchestration Plan
**Generated:** 2025-12-02
**Source:** `codex-2025-12-02-roadmap-scarab.md`

## Executive Summary

This plan orchestrates the Scarab roadmap into **4 parallel work tracks** with clear dependencies, enabling maximum throughput while respecting architectural constraints.

## Dependency Graph

```
Phase 0 (Foundation) ─────┬────────────────────────────────────────────────┐
          │               │                                                │
          ▼               ▼                                                ▼
    ┌─────────────┐ ┌─────────────┐                               ┌───────────────┐
    │ TRACK A     │ │ TRACK B     │                               │ TRACK D       │
    │ Media/Images│ │ ECS/Grid    │                               │ Infrastructure│
    │ (Phase 1)   │ │ (Phase 2)   │                               │ (Phase 5, 6)  │
    └──────┬──────┘ └──────┬──────┘                               └───────┬───────┘
           │               │                                              │
           │               │      ┌─────────────┐                         │
           │               │      │ TRACK C     │                         │
           │               │      │ UI Layer    │                         │
           │               └─────▶│ (Phase 3)   │◀────────────────────────┘
           │                      └──────┬──────┘
           │                             │
           │                      ┌──────▼──────┐
           │                      │ Phase 4     │
           │                      │ Plugin Align│
           │                      └──────┬──────┘
           │                             │
           └─────────────────────────────┴─────────▶ INTEGRATION
```

## Track Definitions

### TRACK A: Media Pipeline (Phase 1)
**Owner:** `backend-architect` + `frontend-developer`
**Parallelizable internally:** Yes

| Task | Description | Files | Agent |
|------|-------------|-------|-------|
| A1 | Protocol extension: `SharedImageBuffer`, `ImagePlacement` structs | `scarab-protocol/src/lib.rs` | backend-architect |
| A2 | Daemon: iTerm2 image capture → per-pane store → shm blit | `scarab-daemon/src/images/*`, `vte.rs` | backend-architect |
| A3 | Client: Read image buffer, upload textures, render sprites | `scarab-client/src/images/*` | frontend-developer |
| A4 | Ligatures: Verify cosmic-text Harfbuzz, golden test | `scarab-client/Cargo.toml`, tests | test-writer-fixer |

**Internal Dependencies:** A1 → A2 → A3 (sequential); A4 is independent

---

### TRACK B: ECS Grid & Dirty Diff (Phase 2)
**Owner:** `frontend-developer`
**Parallelizable internally:** Partial

| Task | Description | Files | Agent |
|------|-------------|-------|-------|
| B1 | Define `TerminalChunk` component (64x32 cells) | `scarab-client/src/terminal/` | frontend-developer |
| B2 | System: SharedMemoryReader → dirty chunk marking | `scarab-client/src/integration/` | frontend-developer |
| B3 | Per-chunk mesh generation (replace whole-grid rebuild) | `scarab-client/src/rendering/` | frontend-developer |
| B4 | Update `TerminalMetrics` for chunk-aware hit-testing | `scarab-protocol/src/lib.rs` | frontend-developer |
| B5 | Headless test: assert only touched chunks rebuild | `tests/` | test-writer-fixer |

**Internal Dependencies:** B1 → B2 → B3 → B4; B5 depends on B3

---

### TRACK C: UI Layer (Phases 3 + 4)
**Owner:** `frontend-developer` + `ui-designer`
**Parallelizable internally:** Phase 3 → Phase 4 sequential

#### Phase 3: Ratatui Bridge

| Task | Description | Files | Agent |
|------|-------------|-------|-------|
| C1 | Create `scarab-ratatui-bridge` crate/module | `crates/scarab-ratatui-bridge/` | frontend-developer |
| C2 | `RatatuiSurface` component + buffer system | bridge crate | frontend-developer |
| C3 | Input mapping: Bevy → Ratatui `Event` | bridge crate | frontend-developer |
| C4 | Prototype: Command palette as Ratatui widget | bridge crate + client | ui-designer |
| C5 | Documentation: overlay plugin guide | `docs/` | frontend-developer |

#### Phase 4: Plugin Alignment

| Task | Description | Files | Agent |
|------|-------------|-------|-------|
| C6 | `ScarabPluginHostPlugin` with ECS events/resources | `scarab-client/src/` | backend-architect |
| C7 | Map Fusabi natives to ECS event dispatch | `scarab-client/src/scripting/` | backend-architect |
| C8 | Replace `Arc<Mutex<EventRegistry>>` with ECS | daemon + client | backend-architect |
| C9 | Port link-hints/palette/tutorial to Bevy plugin form | client | frontend-developer |

**Internal Dependencies:** C1→C2→C3→C4→C5 (Phase 3); then C6→C7→C8→C9 (Phase 4)

---

### TRACK D: Infrastructure (Phase 0, 5, 6)
**Owner:** `devops-automator` + `test-writer-fixer`
**Parallelizable internally:** Phase 0 first, then 5 & 6 parallel

#### Phase 0: Foundation (PREREQUISITE)

| Task | Description | Files | Agent |
|------|-------------|-------|-------|
| D1 | Stabilize headless harness (ScheduleRunner) | `tests/headless_*` | test-writer-fixer |
| D2 | Add telemetry/logging knobs for compositor | config/examples | devops-automator |

#### Phase 5: Shell Integration

| Task | Description | Files | Agent |
|------|-------------|-------|-------|
| D3 | Implement OSC 133 markers in daemon | `scarab-daemon/src/vte.rs` | backend-architect |
| D4 | Expose markers to client (gutter rendering) | `scarab-client/` | frontend-developer |
| D5 | Optional post-process shader (blur/glow toggle) | `scarab-client/src/rendering/` | frontend-developer |

#### Phase 6: Headless CI & Snapshots

| Task | Description | Files | Agent |
|------|-------------|-------|-------|
| D6 | `headless_runner.rs` with ScheduleRunnerPlugin | `tests/` | test-writer-fixer |
| D7 | Golden test: iTerm2 image placement | `tests/` | test-writer-fixer |
| D8 | Golden test: ligatures with Fira Code | `tests/` | test-writer-fixer |

**Internal Dependencies:** D1→D2 (Phase 0 first); then D3-D5 and D6-D8 can run parallel

---

## Parallel Execution Matrix

| Sprint | Track A | Track B | Track C | Track D |
|--------|---------|---------|---------|---------|
| **S0** | - | - | - | D1, D2 (Phase 0) |
| **S1** | A1 (protocol) | B1 (component) | C1 (bridge crate) | D3 (OSC 133) |
| **S2** | A2 (daemon) | B2-B3 (systems) | C2-C3 (surface, input) | D4-D5 (client markers) |
| **S3** | A3 (client) | B4-B5 (metrics, test) | C4-C5 (prototype, docs) | D6 (headless runner) |
| **S4** | A4 (ligatures) | - | C6-C7 (plugin host) | D7-D8 (golden tests) |
| **S5** | - | - | C8-C9 (ECS migration) | - |

---

## Risk Mitigation

| Risk | Impact | Mitigation |
|------|--------|------------|
| Image buffer size explosion | Memory pressure | Enforce 16-32MB cap, count limit, clear-on-reset |
| Chunked grid perf regression | Render slowdown | Profile mesh gen; fallback to coarse chunks |
| Bridge crate scope creep | Delays | Stay thin - accept any `ratatui::Widget`, no rewrites |
| ratatui-testlib issues 14-16 | Test gaps | Local headless harness fallback (D1) |

---

## Agent Assignments Summary

| Agent | Primary Tracks | Tasks |
|-------|----------------|-------|
| `backend-architect` | A, C, D | A1, A2, C6, C7, C8, D3 |
| `frontend-developer` | A, B, C, D | A3, B1-B5, C1-C5, C9, D4, D5 |
| `ui-designer` | C | C4 |
| `test-writer-fixer` | A, B, D | A4, B5, D1, D6, D7, D8 |
| `devops-automator` | D | D2 |

---

## Success Criteria

1. **Phase 0 Complete:** Headless harness passing, telemetry visible
2. **Phase 1 Complete:** iTerm2 images render end-to-end, ligature golden test passes
3. **Phase 2 Complete:** Only dirty chunks rebuild (verified by test)
4. **Phase 3 Complete:** Command palette overlay renders via Ratatui bridge
5. **Phase 4 Complete:** Fusabi plugins spawn ECS entities (link-hints ported)
6. **Phase 5 Complete:** OSC 133 markers visible in gutter
7. **Phase 6 Complete:** CI runs headless snapshots, image + ligature golden tests green

---

## Meta-Orchestrator Instructions

The meta-orchestrator should:
1. **Bootstrap:** Complete Track D Phase 0 (D1, D2) first
2. **Launch Parallel:** Start Tracks A, B, C, D simultaneously after Phase 0
3. **Sync Points:**
   - After A1: Track A can continue; validate protocol structs
   - After C5: Begin Phase 4 (C6+)
   - After D6: D7/D8 can run (depends on A3/A4 for content)
4. **Monitor:** Watch for blocked tasks, reallocate agents as needed
5. **Report:** Surface blockers and completion status to user
