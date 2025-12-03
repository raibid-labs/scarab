# Scarab Execution Plan - 14 Open Issues

**Date**: 2025-12-03
**Status**: Ready for Implementation
**Focus**: Audit 003 Priority Wave (Issues #45-50)

---

## Executive Summary

14 issues grouped into **2 sequential execution waves**:
- **Wave 1 (Audit 003 - Critical)**: Issues #45-50 (6 issues, 2-3 weeks)
- **Wave 2 (Pre-existing)**: Issues #28-36 (8 issues, 4-5 weeks)

Wave 1 must complete before Wave 2 begins due to foundational dependencies.

---

## Issue Dependency Graph

```
WAVE 1 (Audit 003 - Navigation Foundation)
├── #45 (Tests) ← Foundation
├── #46 (State Lifecycle) ← Depends on #45
├── #47 (Security) ← Depends on #46
├── #48 (Config) ← Parallel with #47
├── #49 (Telemetry) ← Parallel with #47, #48
└── #50 (Docs) ← Depends on all above

WAVE 2 (Pre-existing - UI & Advanced Features)
├── #28 (Sixel) ── Independent
├── #29 (Kitty) ── Independent
├── #30 (SSH Mux) ── Depends on #46 (state isolation)
├── #31 (Context Menu) ── Depends on #48 (config)
├── #32 (Marketplace) ── Parallel with others
├── #33 (GPG) ── Parallel with others
├── #35 (Shaders) ── Independent
└── #36 (Shell Integration) ── Depends on #46
```

---

## Wave 1: Audit 003 (Issues #45-50)

### Core Objective
Implement navigation system with robust state isolation, security boundaries, and extensibility.

### Issue Breakdown

#### #45: Test - State Isolation Integration Tests
**Complexity**: Medium
**Crate**: `scarab-nav`, `scarab-panes`, `scarab-session`
**Agent Type**: Test Engineer
**Dependencies**: None (foundation layer)

**What to build**:
- Integration tests for pane state isolation during tab switches
- Test cross-pane state leakage prevention
- Test concurrent navigation operations
- Test state rollback on navigation errors

**Acceptance Criteria**:
- 100% test coverage for navigation state transitions
- All tests pass in CI
- No race conditions detected under stress testing

**Estimated Effort**: 3-4 days

---

#### #46: Navigation - State Lifecycle Management
**Complexity**: Complex
**Crate**: `scarab-nav` (primary), `scarab-panes`, `scarab-daemon`
**Agent Type**: Core Infrastructure Specialist
**Dependencies**: #45 (test foundation established)

**What to build**:
- State lifecycle hooks: `on_pane_activate`, `on_pane_deactivate`, `on_tab_switch`
- Plugin notification system for state changes
- State snapshot/restore mechanism for error recovery
- Navigation event ring buffer (lockfree)

**Key Files to Modify**:
- `/home/beengud/raibid-labs/scarab/crates/scarab-nav/src/lib.rs`
- `/home/beengud/raibid-labs/scarab/crates/scarab-daemon/src/orchestrator.rs`

**Acceptance Criteria**:
- All lifecycle hooks firing correctly
- State isolation verified by #45 tests
- Zero panics under pane switch stress (100+ rapid switches)

**Estimated Effort**: 4-5 days

---

#### #47: Security - Plugin Navigation API Guardrails
**Complexity**: Complex
**Crate**: `scarab-plugin-api`, `scarab-nav`
**Agent Type**: Security/Architecture Specialist
**Dependencies**: #46 (state lifecycle in place)

**What to build**:
- Rate limiting on navigation commands (per-plugin quotas)
- Capability model: plugins declare navigation permissions
- Sandboxing: plugins can't directly manipulate other panes' state
- Audit logging for all plugin navigation calls
- Exception handling: prevent crashed plugins from breaking navigation

**Key Files to Modify**:
- `/home/beengud/raibid-labs/scarab/crates/scarab-plugin-api/src/lib.rs`
- `/home/beengud/raibid-labs/scarab/crates/scarab-nav/src/plugin_integration.rs`

**Acceptance Criteria**:
- Plugin rate limit enforcement (configurable, default 100 ops/sec)
- All navigation calls logged with plugin ID and timestamp
- Malicious plugin cannot crash navigation system
- Security audit checklist signed off

**Estimated Effort**: 3-4 days

---

#### #48: Config - Expose Navigation Keymaps & Plugin Conflict Resolution
**Complexity**: Medium
**Crate**: `scarab-config`, `scarab-nav`
**Agent Type**: Configuration/UX Specialist
**Dependencies**: None (can run parallel with #47)

**What to build**:
- Navigation keymap configuration system (JSON/TOML)
- Plugin conflict detection: two plugins requesting same keybinding
- Conflict resolution UI: priority-based plugin binding order
- Hot-reload support for keymap changes without restart

**Key Files to Modify**:
- `/home/beengud/raibid-labs/scarab/crates/scarab-config/src/lib.rs`
- `/home/beengud/raibid-labs/scarab/crates/scarab-config/examples/default_config.toml`

**Config Structure Example**:
```toml
[navigation]
keymaps.ctrl_h = "navigate.previous_pane"
keymaps.ctrl_l = "navigate.next_pane"
plugin_priority = ["plugin-a", "plugin-b", "plugin-c"]

[plugins.plugin-a.navigation]
allowed_commands = ["tab_switch", "pane_focus"]
rate_limit_ops_per_sec = 50
```

**Acceptance Criteria**:
- Config schema validated
- Keymap conflicts detected and reported
- Hot-reload tested without crashes
- Default config ships with sensible bindings

**Estimated Effort**: 2-3 days

---

#### #49: Telemetry - Pane Switch & Plugin Navigation Metrics
**Complexity**: Medium
**Crate**: `scarab-daemon`, `scarab-nav`
**Agent Type**: Observability/Metrics Specialist
**Dependencies**: None (can run parallel with #47, #48)

**What to build**:
- Metrics collection for pane switch latency (histogram)
- Plugin navigation command counts (per plugin)
- State isolation violation attempts (counter)
- Rate limit hits (per plugin, per command)
- Export via Prometheus endpoint (if enabled)

**Key Files to Modify**:
- `/home/beengud/raibid-labs/scarab/crates/scarab-daemon/src/metrics.rs`
- `/home/beengud/raibid-labs/scarab/crates/scarab-nav/src/lib.rs`

**Metrics Exposed**:
```
scarab_pane_switch_latency_ms (histogram, p50/p95/p99)
scarab_plugin_nav_commands_total (counter, labeled by plugin)
scarab_state_isolation_violations (counter)
scarab_plugin_rate_limit_hits (counter, labeled by plugin)
```

**Acceptance Criteria**:
- Metrics collection <1% CPU overhead
- Prometheus scrape endpoint responds
- Dashboard template provided for Grafana

**Estimated Effort**: 2-3 days

---

#### #50: Docs - Update Navigation Developer Guide
**Complexity**: Simple
**Crate**: Documentation
**Agent Type**: Technical Writer
**Dependencies**: #45-49 all merged (writing about complete system)

**What to build**:
- Developer guide: navigation lifecycle, plugin hooks, state isolation
- API reference: all navigation plugin APIs
- Architecture diagram: navigation flow, state machine
- Troubleshooting guide: common issues and fixes
- Code examples: building a navigation plugin

**Key Files to Create/Modify**:
- `/home/beengud/raibid-labs/scarab/docs/navigation-guide.md`
- `/home/beengud/raibid-labs/scarab/docs/api-reference-navigation.md`
- `/home/beengud/raibid-labs/scarab/docs/examples/nav-plugin-template.fsx`

**Acceptance Criteria**:
- Guide covers all new features from #45-49
- API reference auto-generated from doc comments
- Examples build and run without errors
- Reviewed by team lead

**Estimated Effort**: 1-2 days

---

## Wave 2: Pre-existing Features (Issues #28-36)

### Core Objective
Implement graphics protocols, advanced muxing, and UI extensions.

### Issue Breakdown

#### #28: Sixel Graphics Protocol
**Complexity**: Complex
**Crate**: `scarab-daemon` (VTE parser), `scarab-client` (Bevy rendering)
**Agent Type**: Graphics/Terminal Protocol Specialist
**Dependencies**: None
**Est. Effort**: 5-6 days

---

#### #29: Kitty Graphics Protocol
**Complexity**: Complex
**Crate**: `scarab-daemon`, `scarab-client`
**Agent Type**: Graphics/Terminal Protocol Specialist
**Dependencies**: #28 (similar rendering pipeline)
**Est. Effort**: 4-5 days

---

#### #30: SSH Multiplexing
**Complexity**: Complex
**Crate**: `scarab-daemon`, `scarab-session`
**Agent Type**: Networking Specialist
**Dependencies**: #46 (state isolation required for clean mux boundaries)
**Est. Effort**: 4-5 days

---

#### #31: Context Menu System
**Complexity**: Medium
**Crate**: `scarab-client` (Bevy UI), `scarab-nav` (menu keybinds)
**Agent Type**: UI/UX Specialist
**Dependencies**: #48 (keymap config)
**Est. Effort**: 3-4 days

---

#### #32: Plugin Marketplace UI
**Complexity**: Medium
**Crate**: `scarab-client`, `scarab-config`
**Agent Type**: Frontend/UI Specialist
**Dependencies**: None (independent)
**Est. Effort**: 3-4 days

---

#### #33: GPG Signature Verification
**Complexity**: Medium
**Crate**: `scarab-plugin-compiler` (during plugin load)
**Agent Type**: Security Specialist
**Dependencies**: None (independent)
**Est. Effort**: 2-3 days

---

#### #35: Post-process Shaders
**Complexity**: Medium
**Crate**: `scarab-client` (Bevy rendering)
**Agent Type**: Graphics/Shader Specialist
**Dependencies**: None (independent)
**Est. Effort**: 3-4 days

---

#### #36: Deep Shell Integration
**Complexity**: Complex
**Crate**: `scarab-daemon` (shell hooks), `scarab-nav` (command tracking)
**Agent Type**: Shell Integration Specialist
**Dependencies**: #46 (state isolation)
**Est. Effort**: 4-5 days

---

## Execution Timeline

### Wave 1: 2-3 Weeks

**Week 1**:
- Mon-Wed: #45 (Tests) - Test Engineer
- Wed-Fri: #46 (State Lifecycle) - Core Specialist
- Mon-Fri: #47 (Security) - Security Specialist [parallel]
- Mon-Fri: #48 (Config) - Config Specialist [parallel]
- Mon-Fri: #49 (Telemetry) - Observability Specialist [parallel]

**Week 2**:
- Mon-Tue: Final integration and bug fixes for #45-49
- Wed-Fri: #50 (Documentation) - Technical Writer

**Week 3**:
- Mon: Audit review and final polish
- Tue-Fri: Pre-staging for Wave 2

### Wave 2: 4-5 Weeks (Sequential, with parallelization where possible)

**Week 1**:
- Mon-Fri: #28 (Sixel) + #32 (Marketplace) [parallel]
- Mon-Fri: #33 (GPG) [light workload, parallel]

**Week 2**:
- Mon-Fri: #29 (Kitty) + #35 (Shaders) [parallel]

**Week 3**:
- Mon-Fri: #30 (SSH Mux) + #36 (Shell Integration) [parallel]

**Week 4**:
- Mon-Fri: #31 (Context Menu) + final integration

**Week 5**:
- Mon-Fri: Testing, bug fixes, performance tuning

---

## Agent Assignments Summary

| Issue | Agent Type | Primary Crate | Effort |
|-------|-----------|--------------|--------|
| #45 | Test Engineer | scarab-nav | 3-4 days |
| #46 | Core Infrastructure | scarab-nav | 4-5 days |
| #47 | Security Specialist | scarab-plugin-api | 3-4 days |
| #48 | Configuration | scarab-config | 2-3 days |
| #49 | Observability | scarab-daemon | 2-3 days |
| #50 | Technical Writer | docs | 1-2 days |
| #28 | Graphics Protocol | scarab-daemon | 5-6 days |
| #29 | Graphics Protocol | scarab-client | 4-5 days |
| #30 | Networking | scarab-session | 4-5 days |
| #31 | UI/UX | scarab-client | 3-4 days |
| #32 | Frontend | scarab-client | 3-4 days |
| #33 | Security | scarab-plugin-compiler | 2-3 days |
| #35 | Graphics/Shaders | scarab-client | 3-4 days |
| #36 | Shell Integration | scarab-daemon | 4-5 days |

---

## Success Metrics

**Wave 1 Success**:
- All 6 issues merged to main
- All tests passing (>95% coverage)
- Zero critical security issues in audit
- Documentation complete and reviewed
- Navigation system stable under 1000 pane switch/sec

**Wave 2 Success**:
- All 8 issues merged to main
- Graphics protocols rendering correctly
- Marketplace UI functional with 5+ test plugins
- Shell integration hooks working for bash/zsh/fish
- Performance: <50ms latency for complex operations

---

## Risk Mitigation

| Risk | Mitigation |
|------|-----------|
| State isolation bugs in #46 | Extensive property-based testing (#45) |
| Plugin security bypass | Security code review + fuzzing harness |
| Performance regression | Benchmark suite with regression detection |
| Config parsing issues | Schema validation + migration tooling |
| Documentation decay | Auto-generated API docs from code |

---

## Prerequisites Checklist

Before starting Wave 1:
- [ ] All team members trained on navigation architecture
- [ ] Test infrastructure set up (tokio-test, criterion benchmarks)
- [ ] CI/CD pipeline ready for new crates
- [ ] Security review process established
- [ ] Metrics infrastructure deployed (Prometheus optional)

---

## Rollback Strategy

If critical issues emerge:
- Wave 1: Roll back individual issues (non-blocking)
- Wave 2: Can pause at issue boundaries without cascade failures
- Hot-fix path: Critical security bugs bypass backlog queue

---

## Next Steps

1. **Approve this plan** with team lead
2. **Assign primary agents** to Wave 1 issues
3. **Schedule kickoff meeting** (30min team sync)
4. **Create tracking board** in GitHub Projects
5. **Begin Week 1 of Wave 1** (Monday)

---

**Prepared by**: Performance Coach
**Last Updated**: 2025-12-03
**Status**: Ready for Execution
