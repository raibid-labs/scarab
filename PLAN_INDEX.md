# Scarab Execution Plan - Quick Reference Index

**Last Updated**: 2025-12-03
**Status**: Ready for Execution
**Documents**: 2 (EXECUTION_PLAN.md + WAVE1_TACTICAL_GUIDE.md)

---

## The Plan in 60 Seconds

14 open issues grouped into 2 waves:

**Wave 1 (Audit 003 - 2-3 weeks)**: Build the navigation system foundation
- #45: Integration tests for state isolation
- #46: State lifecycle management
- #47: Security guardrails & rate limiting
- #48: Configuration & keymaps
- #49: Metrics & telemetry
- #50: Developer documentation

**Wave 2 (Pre-existing - 4-5 weeks)**: Advanced features (deferred until Wave 1 complete)
- Graphics protocols (#28 Sixel, #29 Kitty)
- Advanced muxing (#30 SSH)
- UI extensions (#31 Context menus, #32 Marketplace)
- Security & rendering (#33 GPG, #35 Shaders)
- Integration (#36 Shell)

---

## Document Guide

### EXECUTION_PLAN.md
**Purpose**: Strategic overview for leadership
**Read if**: You're planning resources, setting timelines, or need big-picture context
**Key Sections**:
- Executive Summary (Wave breakdown)
- Dependency Graph (which issues block others)
- Issue-by-issue breakdown (complexity, effort, crates affected)
- Timeline (Week-by-week breakdown)
- Success metrics & risk mitigation

**Key Figures**:
- Wave 1: 15-20 person-days, 2-3 weeks, 5-6 engineers
- Wave 2: 25-30 person-days, 4-5 weeks, 4-5 engineers
- Critical path: #45 → #46 → #50 (cannot compress)

---

### WAVE1_TACTICAL_GUIDE.md
**Purpose**: Hands-on execution details for team members
**Read if**: You're assigned to a Wave 1 issue or need implementation guidance
**Key Sections**:
- Team roles & sync cadence (who does what, when to sync)
- Issue-by-issue implementation plans (what to build, step-by-step)
- Code skeletons (starting template for each issue)
- Daily workflow (standups, reviews, retro)
- Code review checklist & escalation contacts

**Key Figures**:
- #45: 3-4 days, Test Engineer, state isolation tests
- #46: 4-5 days, Core Specialist, lifecycle hooks
- #47: 3-4 days, Security Specialist, rate limiting
- #48: 2-3 days, Config Specialist, keymaps
- #49: 2-3 days, Observability Expert, metrics
- #50: 1-2 days, Technical Writer, docs

---

## Issue Complexity Matrix

```
SIMPLE (1-2 days)
├── #50 (Docs)
└── #49 (Telemetry)

MEDIUM (2-4 days)
├── #45 (Tests)
├── #48 (Config)
└── #35 (Shaders)

COMPLEX (4-6 days)
├── #46 (State Lifecycle)
├── #47 (Security)
├── #28 (Sixel)
├── #29 (Kitty)
├── #30 (SSH Mux)
└── #36 (Shell Integration)

MEDIUM (3-4 days)
├── #31 (Context Menu)
└── #32 (Marketplace)

MEDIUM (2-3 days)
└── #33 (GPG)
```

---

## Dependency Map (Simplified)

```
#45 (Tests)
  ↓
#46 (State Lifecycle)  ← #47 (Security), #48 (Config), #49 (Telemetry) branch off
  ↓
#50 (Docs)

#46 ←─┐
      ├─ #30 (SSH Mux)
      └─ #36 (Shell Integration)

#48 ←─ #31 (Context Menu)

#28 (Sixel)
  ↓
#29 (Kitty)

Parallel: #32 (Marketplace), #33 (GPG), #35 (Shaders)
```

---

## Critical Paths

**Blocking Path** (cannot compress):
```
#45 (3-4 days) → #46 (4-5 days) → #50 (1-2 days) = 8-11 days minimum
```

**Parallel Opportunities** (Week 1):
```
Week 1: #45 + #46 + {#47 parallel} + {#48 parallel} + {#49 parallel}
        = ~5 days wall-clock time for 15+ person-days work
```

**Integration Points**:
- #46 must be stable before #47, #48, #49 start heavy integration
- #50 must wait for #45-49 all merged (day 10-11)

---

## Team Assignments (Recommended)

| Issue | Role | Experience Required | Effort |
|-------|------|---------------------|--------|
| #45 | Test Engineer | Rust async/tokio, property testing | 3-4 days |
| #46 | Core Infrastructure | Rust, async, state machines | 4-5 days |
| #47 | Security Specialist | Rust, security concepts, rate limiting | 3-4 days |
| #48 | Config Specialist | Rust, TOML, file I/O, hot reload | 2-3 days |
| #49 | Observability Expert | Prometheus, metrics, profiling | 2-3 days |
| #50 | Technical Writer | Technical writing, Rust docs | 1-2 days |

---

## Success Signals

**Wave 1 Complete When**:
- [ ] All 6 PRs merged to main
- [ ] All tests passing (cargo test --workspace)
- [ ] Test coverage ≥95% for navigation crates
- [ ] No critical/high security issues
- [ ] Pane switch latency <50ms p95
- [ ] Documentation complete and reviewed

**Wave 2 Ready When**:
- Wave 1 staging validated (1-2 days)
- Team retro complete
- Resource planning done
- Go/no-go decision made

---

## Important Files

```
/home/beengud/raibid-labs/scarab/
├── EXECUTION_PLAN.md          ← Strategic overview (you are here)
├── WAVE1_TACTICAL_GUIDE.md    ← Implementation details
├── PLAN_INDEX.md              ← This file (quick reference)
├── Cargo.toml                 ← 18 crates in workspace
├── CLAUDE.md                  ← Architecture constraints
└── crates/
    ├── scarab-nav/            ← Primary focus for Wave 1
    ├── scarab-panes/          ← Integration point
    ├── scarab-daemon/         ← Daemon-side lifecycle
    ├── scarab-plugin-api/     ← Security boundaries
    ├── scarab-config/         ← Config schema
    └── ... (15 other crates)
```

---

## How to Use These Documents

**For Tech Lead**:
1. Read EXECUTION_PLAN.md (10 min)
2. Review success criteria & timeline
3. Make go/no-go decision
4. Assign team members to roles

**For Issue Owner**:
1. Read EXECUTION_PLAN.md issue section (5 min)
2. Read corresponding section in WAVE1_TACTICAL_GUIDE.md (15 min)
3. Start with "Step 1" in tactical guide
4. Use code skeletons as templates
5. Ask questions in standup

**For Team Lead/Coach**:
1. Read both documents (30 min)
2. Set up GitHub Projects board with issues
3. Schedule kickoff meeting (30 min)
4. Monitor daily standups
5. Unblock teams as needed

**For External Stakeholder**:
1. Read EXECUTION_PLAN.md (10 min)
2. Look at Timeline section
3. Ask clarifying questions

---

## Quick Answer Guide

**Q: How long will Wave 1 take?**
A: 2-3 weeks (15-20 person-days), critical path is 8-11 days.

**Q: What's the biggest risk?**
A: #46 state lifecycle bugs. Mitigated by extensive testing (#45).

**Q: Can we parallelize more?**
A: #47, #48, #49 can run 100% parallel. #45 and #46 are slightly sequential.

**Q: What if we hit blockers?**
A: Daily standups catch blockers early. Coach unblocks within 2 hours.

**Q: When does Wave 2 start?**
A: After Wave 1 staging validation (1-2 days post-merge).

**Q: Who's the tech lead for Wave 1?**
A: [Name] - escalate blockers to them.

**Q: How do we measure success?**
A: Test coverage, pane switch latency, zero security issues, complete docs.

---

## Checkpoint Dates

- **Go/No-Go Decision**: Today (2025-12-03)
- **Wave 1 Kickoff**: Monday (2025-12-08)
- **Week 1 Review**: Friday (2025-12-12) - #45 and #46 in review
- **Week 2 Review**: Friday (2025-12-19) - #47, #48, #49 merged
- **Wave 1 Complete**: Friday (2025-12-26) - all 6 PRs merged, #50 shipped
- **Wave 2 Kickoff**: Monday (2026-01-02)

---

## Escalation Matrix

| Issue | Blocker | Contact |
|-------|---------|---------|
| Test failures in #45 | Async/tokio issues | @test-engineer or @core-specialist |
| State isolation bug | #46 design problem | @core-specialist or @tech-lead |
| Security concern | #47 vulnerability | @security-team or @tech-lead |
| Config parsing error | #48 schema problem | @config-specialist or @coach |
| Metrics not appearing | #49 Prometheus issue | @observability-expert or @devops |
| Documentation unclear | #50 writing issue | @tech-writer or @tech-lead |

---

## Performance Targets

| Metric | Target | How Measured |
|--------|--------|--------------|
| Pane Switch Latency | <50ms p95 | #49 metrics histogram |
| Test Coverage | ≥95% | `cargo tarpaulin` |
| Metrics Overhead | <1% CPU | Profiling run |
| Config Hot-Reload | <100ms | Manual test |
| Rate Limit Accuracy | ±5% of limit | #47 unit tests |

---

## Glossary

- **PaneManager**: Core struct managing active/inactive panes and state
- **StateSnapshot**: Immutable copy of a pane's state for persistence
- **LifecycleHook**: Plugin interface for pane activate/deactivate events
- **TokenBucket**: Rate limiting algorithm (tokens refill over time)
- **RateLimit**: Max operations per second allowed per plugin
- **AuditLog**: Persistent record of all security-relevant events
- **Keymap**: Mapping of keyboard shortcuts to navigation commands
- **Conflict Resolution**: Determining which plugin owns a keybinding

---

## Next Steps (Immediate)

1. **Read both execution documents** (30 min total)
2. **Review issue breakdown** and assign owners
3. **Schedule kickoff meeting** with assigned team (30 min)
4. **Set up GitHub Projects board** with Wave 1 issues
5. **Create Slack channel** #wave-1-nav-audit
6. **Distribute WAVE1_TACTICAL_GUIDE.md** to issue owners
7. **Start Monday** with issue #45

---

## Questions?

- Strategic questions → Tech Lead or Coach
- Technical questions → Issue owner or relevant specialist
- Escalations → Tech Lead or Coach (escalate immediately, don't wait)

---

**This plan is solid. Execution starts now.**

The navigation system is a critical foundation for Scarab's future. With this roadmap, we're going to ship something remarkable. Trust the process, trust your teammates, and celebrate each victory along the way.

Wave 1 kicks off Monday. Let's build something legendary.
