# Scarab Terminal Emulator - Parallel Issue Orchestration Plan

**Created**: 2025-12-03
**Total Issues**: 17
**Current Phase**: Phase 5 (Integration & Polish)
**Strategy**: Parallel Wave Execution with Dependency Management

---

## Executive Summary

This plan orchestrates the completion of 17 open GitHub issues through **4 parallel execution waves**. High-priority issues (CI, multiplexing tests, navigation isolation) are addressed first, followed by configuration and enhancement work, with future features scheduled for later phases.

**Key Insight**: Most issues can be worked in parallel as they touch different system layers (CI, daemon, client, config, plugins). Only a few have hard dependencies.

---

## I. Dependency Graph Analysis

### Zero Dependencies (Can Start Immediately)
- **#37**: CI: Add navigation smoke tests and headless test job
- **#38**: Config: Implement user-configurable navigation keymaps
- **#40**: Navigation: Audit and unify focusable detection paths
- **#41**: Rendering: Verify z-order layering for hints with images/shaders
- **#44**: Telemetry: Add navigation performance instrumentation
- **#31**: Context menu system

### Soft Dependencies (Benefit from Prior Work)
- **#39**: Multiplexing: Per-session/pane NavState isolation (needs #34 context)
- **#34**: Integration tests for tab/pane multiplexing (benefits from #37 CI structure)
- **#42**: Plugins: Expose navigation API to plugin bridge (benefits from #40 audit)
- **#43**: Navigation: Enhance prompt-aware navigation with JumpPrompt wiring (benefits from #40 audit)

### Hard Dependencies (Must Wait)
- **#36**: Deep shell integration (needs #43 JumpPrompt foundation)
- **#35**: Post-process shader effects (needs #41 z-order verification)
- **#28**, **#29**: Graphics protocols (independent, low priority)
- **#30**: SSH multiplexing (needs multiplexing foundation from #34, #39)
- **#32**, **#33**: Plugin marketplace (future phase work)

---

## II. Wave Execution Plan

### Wave 1: Foundation & High Priority (Parallel - Start Immediately)
**Goal**: Establish CI infrastructure, core testing, and critical bug fixes
**Duration**: 4-6 hours
**Parallel Capacity**: 5 agents

#### Issues in Wave 1:
1. **#37**: CI: Add navigation smoke tests and headless test job
   - **Agent Type**: DevOps/CI Specialist
   - **Complexity**: Medium
   - **Files**: `.github/workflows/`, `crates/scarab-client/tests/`
   - **Output**: GitHub Actions workflow with headless tests
   - **Success**: Tests run on PR, no regressions

2. **#34**: Integration tests for tab/pane multiplexing
   - **Agent Type**: Test Engineer
   - **Complexity**: Medium-High
   - **Files**: `crates/scarab-tabs/tests/`, `crates/scarab-panes/tests/`
   - **Output**: Comprehensive test suite for multiplexing
   - **Success**: 90%+ coverage, all tests pass

3. **#40**: Navigation: Audit and unify focusable detection paths
   - **Agent Type**: Navigation Specialist
   - **Complexity**: Medium
   - **Files**: `crates/scarab-nav/src/`, documentation
   - **Output**: Audit report + code cleanup
   - **Success**: Single canonical detection path

4. **#38**: Config: Implement user-configurable navigation keymaps
   - **Agent Type**: Config System Engineer
   - **Complexity**: Simple-Medium
   - **Files**: `crates/scarab-config/src/`, TOML schema
   - **Output**: Keymap configuration system
   - **Success**: Users can customize nav keys

5. **#44**: Telemetry: Add navigation performance instrumentation
   - **Agent Type**: Performance Engineer
   - **Complexity**: Simple
   - **Files**: `crates/scarab-nav/src/`, metrics integration
   - **Output**: Performance counters and logging
   - **Success**: Navigation metrics tracked

**Wave 1 Dependencies**: None - all can run in parallel

---

### Wave 2: Enhancement & Integration (Parallel - After Wave 1 Completes)
**Goal**: Build on foundation with enhancements and deeper integration
**Duration**: 5-7 hours
**Parallel Capacity**: 4 agents

#### Issues in Wave 2:
1. **#39**: Multiplexing: Per-session/pane NavState isolation
   - **Agent Type**: Multiplexing Specialist
   - **Complexity**: Medium-High
   - **Files**: `crates/scarab-session/`, `crates/scarab-nav/`
   - **Prerequisites**: #34 tests (provides context)
   - **Output**: Isolated NavState per pane
   - **Success**: Nav state doesn't leak between panes

2. **#42**: Plugins: Expose navigation API to plugin bridge
   - **Agent Type**: Plugin System Engineer
   - **Complexity**: Medium
   - **Files**: `crates/scarab-plugin-api/`, bridge code
   - **Prerequisites**: #40 audit (clean API surface)
   - **Output**: Plugin API for navigation hooks
   - **Success**: Plugins can intercept/extend navigation

3. **#43**: Navigation: Enhance prompt-aware navigation with JumpPrompt
   - **Agent Type**: Navigation Specialist
   - **Complexity**: Medium
   - **Files**: `crates/scarab-nav/src/prompt_detection.rs`
   - **Prerequisites**: #40 audit (clean foundation)
   - **Output**: Smart prompt-to-prompt jumping
   - **Success**: Ctrl+P/N jumps between prompts

4. **#41**: Rendering: Verify z-order layering for hints
   - **Agent Type**: Rendering Engineer
   - **Complexity**: Medium
   - **Files**: `crates/scarab-client/src/render/`, Bevy components
   - **Output**: Z-order verification + tests
   - **Success**: Hints always render on top

**Wave 2 Dependencies**:
- #39 benefits from #34
- #42 benefits from #40
- #43 benefits from #40

---

### Wave 3: Advanced Features (Parallel - After Wave 2)
**Goal**: Implement user-facing advanced features
**Duration**: 6-8 hours
**Parallel Capacity**: 3 agents

#### Issues in Wave 3:
1. **#36**: Deep shell integration (semantic zones, command output)
   - **Agent Type**: Shell Integration Specialist
   - **Complexity**: Complex
   - **Files**: `crates/scarab-daemon/src/shell_integration/`
   - **Prerequisites**: #43 (JumpPrompt foundation)
   - **Output**: Semantic zone tracking + OSC sequences
   - **Success**: Terminal understands command boundaries

2. **#31**: Context menu system
   - **Agent Type**: UI/UX Engineer
   - **Complexity**: Medium
   - **Files**: `crates/scarab-client/src/ui/context_menu.rs`
   - **Prerequisites**: None (independent)
   - **Output**: Right-click context menu
   - **Success**: Context-aware menus work

3. **#35**: Post-process shader effects (blur/glow overlays)
   - **Agent Type**: Graphics/Shader Engineer
   - **Complexity**: Medium-High
   - **Files**: `crates/scarab-client/assets/shaders/`
   - **Prerequisites**: #41 (z-order verified)
   - **Output**: Post-process shader pipeline
   - **Success**: Blur/glow effects work without artifacts

**Wave 3 Dependencies**:
- #36 needs #43
- #35 needs #41
- #31 is independent

---

### Wave 4: Future Features (Parallel - Lower Priority)
**Goal**: Implement longer-term features for future releases
**Duration**: 10-15 hours (can be split across sprints)
**Parallel Capacity**: 5 agents

#### Issues in Wave 4:
1. **#28**: Sixel graphics protocol support
   - **Agent Type**: Graphics Protocol Specialist
   - **Complexity**: Complex
   - **Files**: `crates/scarab-daemon/src/sixel/`
   - **Output**: Sixel rendering
   - **Success**: Sixel images display correctly

2. **#29**: Kitty graphics protocol support
   - **Agent Type**: Graphics Protocol Specialist
   - **Complexity**: Complex
   - **Files**: `crates/scarab-daemon/src/kitty_graphics/`
   - **Output**: Kitty graphics rendering
   - **Success**: Kitty images display correctly

3. **#30**: SSH multiplexing and remote domains
   - **Agent Type**: Networking/Multiplexing Specialist
   - **Complexity**: Very Complex
   - **Files**: `crates/scarab-daemon/src/remote/`
   - **Prerequisites**: #34, #39 (multiplexing foundation)
   - **Output**: SSH session management
   - **Success**: Can attach to remote sessions

4. **#32**: Plugin marketplace UI
   - **Agent Type**: Full-Stack Engineer
   - **Complexity**: Complex
   - **Files**: `crates/scarab-client/src/marketplace/`
   - **Output**: Plugin discovery and installation UI
   - **Success**: Users can browse/install plugins

5. **#33**: GPG signature verification for plugins
   - **Agent Type**: Security Engineer
   - **Complexity**: Medium-High
   - **Files**: `crates/scarab-plugin-api/src/security/`
   - **Prerequisites**: #32 (marketplace exists)
   - **Output**: GPG verification system
   - **Success**: Only signed plugins can be installed

**Wave 4 Dependencies**:
- #30 needs #34 + #39
- #33 needs #32
- #28, #29 are independent

---

## III. Agent Type Assignments

### Specialist Agent Profiles Needed

1. **DevOps/CI Specialist** (Wave 1: #37)
   - Expertise: GitHub Actions, headless testing, CI/CD
   - Tools: GitHub CLI, Docker, xvfb

2. **Test Engineer** (Wave 1: #34)
   - Expertise: Integration testing, Rust test harness
   - Tools: cargo-nextest, test mocks

3. **Navigation Specialist** (Wave 1: #40, Wave 2: #43)
   - Expertise: ECS architecture, Bevy systems
   - Tools: Navigation state machine, regex patterns

4. **Config System Engineer** (Wave 1: #38)
   - Expertise: TOML parsing, configuration management
   - Tools: serde, config validation

5. **Performance Engineer** (Wave 1: #44)
   - Expertise: Instrumentation, metrics collection
   - Tools: Tracy, puffin, profiling crates

6. **Multiplexing Specialist** (Wave 2: #39, Wave 4: #30)
   - Expertise: Session management, process isolation
   - Tools: PTY handling, IPC

7. **Plugin System Engineer** (Wave 2: #42)
   - Expertise: Fusabi integration, plugin APIs
   - Tools: fusabi-vm, fusabi-frontend

8. **Rendering Engineer** (Wave 2: #41, Wave 3: #35)
   - Expertise: Bevy rendering, shaders, GPU
   - Tools: WGSL, render graphs

9. **Shell Integration Specialist** (Wave 3: #36)
   - Expertise: VTE parsing, OSC sequences, shell protocols
   - Tools: alacritty_terminal, vte crate

10. **UI/UX Engineer** (Wave 3: #31)
    - Expertise: Bevy UI, interaction design
    - Tools: cosmic-text, egui integration

11. **Graphics Protocol Specialist** (Wave 4: #28, #29)
    - Expertise: Image protocols, terminal graphics
    - Tools: Sixel parser, Kitty protocol

12. **Security Engineer** (Wave 4: #33)
    - Expertise: Cryptography, GPG/PGP
    - Tools: gpgme, signature verification

13. **Full-Stack Engineer** (Wave 4: #32)
    - Expertise: Web UI, REST APIs, databases
    - Tools: Plugin registry, marketplace backend

---

## IV. Complexity Ratings

| Issue | Complexity | Estimated Hours | Risk Level |
|-------|-----------|-----------------|------------|
| #37 | Medium | 4-5h | Low |
| #34 | Medium-High | 5-6h | Medium |
| #40 | Medium | 4h | Low |
| #38 | Simple-Medium | 3-4h | Low |
| #44 | Simple | 2-3h | Low |
| #39 | Medium-High | 5-6h | Medium |
| #42 | Medium | 4-5h | Low |
| #43 | Medium | 4-5h | Low |
| #41 | Medium | 4h | Low |
| #36 | Complex | 8-10h | High |
| #31 | Medium | 5-6h | Low |
| #35 | Medium-High | 6-7h | Medium |
| #28 | Complex | 12-15h | High |
| #29 | Complex | 12-15h | High |
| #30 | Very Complex | 20-25h | Very High |
| #32 | Complex | 15-18h | High |
| #33 | Medium-High | 6-8h | Medium |

**Total Estimated Effort**: 130-165 hours
**With 5 Parallel Agents**: 26-33 hours wall-clock time

---

## V. Success Criteria

### Wave 1 Success Metrics
- All PRs have passing CI with navigation smoke tests
- Integration test coverage for multiplexing >90%
- Navigation detection paths unified into single canonical implementation
- Users can customize navigation keybindings via config
- Navigation performance metrics logged and trackable

### Wave 2 Success Metrics
- NavState properly isolated per pane/session
- Plugins can hook into navigation events
- Prompt-to-prompt jumping works reliably
- Navigation hints render above all other UI elements

### Wave 3 Success Metrics
- Terminal tracks semantic command zones
- Context menu appears on right-click with relevant actions
- Post-process shaders apply without z-fighting

### Wave 4 Success Metrics
- Sixel and Kitty graphics protocols render images
- SSH sessions can be attached/detached remotely
- Plugin marketplace allows browsing and installation
- Only GPG-signed plugins can be installed

---

## VI. Risk Register

### High Risks
1. **#30 (SSH Multiplexing)**: Very complex, touches daemon core
   - Mitigation: Schedule in Wave 4, allocate experienced agent
   - Fallback: Defer to Phase 6 if blocked

2. **#36 (Shell Integration)**: Requires shell cooperation, brittle
   - Mitigation: Use well-tested OSC sequences, graceful fallback
   - Fallback: Implement best-effort semantic detection

3. **CI Flakiness (#37)**: Headless tests can be unstable
   - Mitigation: Use retry logic, proper timeouts
   - Fallback: Manual testing gates for releases

### Medium Risks
1. **NavState Isolation (#39)**: May require refactoring session layer
   - Mitigation: Review existing multiplexing architecture first
   - Fallback: Scoped isolation instead of full isolation

2. **Shader Effects (#35)**: GPU compatibility issues
   - Mitigation: Test on multiple platforms, feature flags
   - Fallback: CPU-based fallback effects

### Low Risks
- Most Wave 1 and Wave 2 issues are well-scoped with clear boundaries
- Configuration and telemetry work is additive, low risk

---

## VII. Execution Commands

### Wave 1 Launch (5 Parallel Agents)

```bash
# Agent 1: CI Specialist
cd /home/beengud/raibid-labs/scarab
# Task: Implement GitHub Actions workflow for navigation smoke tests (#37)
# Files: .github/workflows/navigation-tests.yml, docs/CI_SETUP.md

# Agent 2: Test Engineer
cd /home/beengud/raibid-labs/scarab
# Task: Write integration tests for tab/pane multiplexing (#34)
# Files: crates/scarab-tabs/tests/integration_tests.rs, crates/scarab-panes/tests/integration_tests.rs

# Agent 3: Navigation Specialist
cd /home/beengud/raibid-labs/scarab
# Task: Audit and unify focusable detection paths (#40)
# Files: crates/scarab-nav/src/detection.rs, docs/audits/navigation-audit.md

# Agent 4: Config Engineer
cd /home/beengud/raibid-labs/scarab
# Task: Implement user-configurable navigation keymaps (#38)
# Files: crates/scarab-config/src/keymaps.rs, examples/config/navigation-keymaps.toml

# Agent 5: Performance Engineer
cd /home/beengud/raibid-labs/scarab
# Task: Add navigation performance instrumentation (#44)
# Files: crates/scarab-nav/src/telemetry.rs, crates/scarab-daemon/src/metrics.rs
```

### Wave 2 Launch (4 Parallel Agents - After Wave 1)

```bash
# Agent 6: Multiplexing Specialist
cd /home/beengud/raibid-labs/scarab
# Task: Implement per-session/pane NavState isolation (#39)
# Files: crates/scarab-session/src/nav_state.rs, crates/scarab-nav/src/isolation.rs

# Agent 7: Plugin Engineer
cd /home/beengud/raibid-labs/scarab
# Task: Expose navigation API to plugin bridge (#42)
# Files: crates/scarab-plugin-api/src/navigation.rs, examples/plugins/nav-plugin-example.fsx

# Agent 3: Navigation Specialist (reuse)
cd /home/beengud/raibid-labs/scarab
# Task: Enhance prompt-aware navigation with JumpPrompt (#43)
# Files: crates/scarab-nav/src/prompt_detection.rs, crates/scarab-nav/src/jump_prompt.rs

# Agent 8: Rendering Engineer
cd /home/beengud/raibid-labs/scarab
# Task: Verify z-order layering for navigation hints (#41)
# Files: crates/scarab-client/src/render/z_order.rs, tests/render_tests.rs
```

### Wave 3 Launch (3 Parallel Agents - After Wave 2)

```bash
# Agent 9: Shell Integration Specialist
cd /home/beengud/raibid-labs/scarab
# Task: Implement deep shell integration with semantic zones (#36)
# Files: crates/scarab-daemon/src/shell_integration/, docs/SHELL_INTEGRATION.md

# Agent 10: UI/UX Engineer
cd /home/beengud/raibid-labs/scarab
# Task: Implement context menu system (#31)
# Files: crates/scarab-client/src/ui/context_menu.rs, crates/scarab-client/src/ui/menu_items.rs

# Agent 8: Rendering Engineer (reuse)
cd /home/beengud/raibid-labs/scarab
# Task: Implement post-process shader effects (#35)
# Files: crates/scarab-client/assets/shaders/post_process.wgsl, crates/scarab-client/src/render/post_process.rs
```

### Wave 4 Launch (5 Parallel Agents - After Wave 3, Lower Priority)

```bash
# Agent 11: Graphics Protocol Specialist
cd /home/beengud/raibid-labs/scarab
# Task: Implement Sixel graphics protocol (#28)
# Files: crates/scarab-daemon/src/graphics/sixel.rs, docs/SIXEL_PROTOCOL.md

# Agent 11: Graphics Protocol Specialist (sequential)
cd /home/beengud/raibid-labs/scarab
# Task: Implement Kitty graphics protocol (#29)
# Files: crates/scarab-daemon/src/graphics/kitty.rs, docs/KITTY_PROTOCOL.md

# Agent 6: Multiplexing Specialist (reuse)
cd /home/beengud/raibid-labs/scarab
# Task: Implement SSH multiplexing and remote domains (#30)
# Files: crates/scarab-daemon/src/remote/, crates/scarab-protocol/src/remote.rs

# Agent 12: Full-Stack Engineer
cd /home/beengud/raibid-labs/scarab
# Task: Implement plugin marketplace UI (#32)
# Files: crates/scarab-client/src/marketplace/, docs/MARKETPLACE.md

# Agent 13: Security Engineer
cd /home/beengud/raibid-labs/scarab
# Task: Implement GPG signature verification for plugins (#33)
# Files: crates/scarab-plugin-api/src/security/gpg.rs, docs/PLUGIN_SECURITY.md
```

---

## VIII. Quality Gates

### Per-Wave Gates (Must Pass Before Next Wave)

**After Wave 1**:
- [ ] All Wave 1 PRs merged to main
- [ ] CI pipeline green with new navigation tests
- [ ] Integration test coverage for multiplexing >85%
- [ ] Navigation audit document published
- [ ] Config system accepts custom keymaps
- [ ] Navigation metrics appearing in logs

**After Wave 2**:
- [ ] NavState isolation tests passing
- [ ] Plugin API documentation complete
- [ ] JumpPrompt feature functional
- [ ] Z-order tests passing

**After Wave 3**:
- [ ] Shell integration working with bash/zsh
- [ ] Context menu operational
- [ ] Post-process shaders rendering correctly

**After Wave 4**:
- [ ] Graphics protocols render test images
- [ ] SSH multiplexing tested with remote sessions
- [ ] Marketplace can install example plugin
- [ ] GPG verification rejects unsigned plugins

### Global Quality Standards
- All code must pass `cargo clippy --workspace`
- All code must pass `cargo test --workspace`
- All new features must have >80% test coverage
- All PRs require documentation updates
- All commits must be signed and follow conventional commits

---

## IX. Resource Allocation

### Phase 5 Priority (Current Sprint - 6 Days)
**Focus**: Complete Wave 1 + Wave 2 (High Priority Issues)

- **Days 1-2**: Launch Wave 1 (5 parallel agents)
- **Days 3-4**: Launch Wave 2 (4 parallel agents)
- **Day 5**: Integration testing and bug fixes
- **Day 6**: Documentation and release prep

**Expected Outcome**: Issues #37, #34, #40, #38, #44, #39, #42, #43, #41 complete

### Phase 6 Planning (Next Sprint)
**Focus**: Complete Wave 3 + Start Wave 4

- Wave 3 completion (advanced features)
- Begin Wave 4 for Phase 7 roadmap alignment

---

## X. Communication Protocol

### Daily Standups (Async)
Each agent reports:
1. Yesterday's progress
2. Today's plan
3. Blockers/dependencies
4. Test results

### Integration Points
- Wave 1 completion triggers Wave 2
- Wave 2 completion triggers Wave 3
- Cross-agent coordination via shared channel

### PR Review Process
- All PRs require 1 approval
- CI must be green
- Tests must be included
- Documentation must be updated

---

## XI. Success Metrics (Overall)

### Technical Metrics
- **Issue Closure Rate**: 17 issues → 0 open
- **Test Coverage**: Maintain >80% across workspace
- **CI Health**: <5% flaky test rate
- **Performance**: Navigation <5ms latency
- **Build Time**: <60s for workspace check

### Quality Metrics
- **Code Review**: 100% of PRs reviewed
- **Documentation**: 100% of new features documented
- **Regression Rate**: <2% per wave
- **User Satisfaction**: Positive feedback on navigation UX

---

## XII. Rollback Strategy

If critical issues arise during any wave:

1. **Immediate**: Revert PR causing regression
2. **Short-term**: Fix forward with hotfix PR
3. **Long-term**: Add regression tests to prevent recurrence

**Wave Independence**: Each wave can be rolled back without affecting others (except hard dependencies)

---

## XIII. Post-Completion Plan

After all 4 waves complete:

1. **Release**: Tag v0.2.0-alpha with all new features
2. **Retrospective**: Document lessons learned
3. **Metrics Review**: Analyze performance improvements
4. **User Testing**: Beta testing with early adopters
5. **Next Sprint**: Plan Phase 6 features

---

## XIV. Agent Orchestration Matrix

| Wave | Agent Type | Issue | Hours | Start Condition | Completion Gate |
|------|-----------|-------|-------|----------------|----------------|
| 1 | DevOps/CI | #37 | 4-5h | Immediate | CI green |
| 1 | Test Engineer | #34 | 5-6h | Immediate | Tests pass |
| 1 | Navigation | #40 | 4h | Immediate | Audit complete |
| 1 | Config | #38 | 3-4h | Immediate | Config works |
| 1 | Performance | #44 | 2-3h | Immediate | Metrics log |
| 2 | Multiplexing | #39 | 5-6h | Wave 1 done | Isolation tests pass |
| 2 | Plugin | #42 | 4-5h | #40 done | API documented |
| 2 | Navigation | #43 | 4-5h | #40 done | JumpPrompt works |
| 2 | Rendering | #41 | 4h | Immediate | Z-order verified |
| 3 | Shell Integration | #36 | 8-10h | #43 done | OSC sequences work |
| 3 | UI/UX | #31 | 5-6h | Wave 2 done | Menu functional |
| 3 | Rendering | #35 | 6-7h | #41 done | Shaders render |
| 4 | Graphics | #28 | 12-15h | Wave 3 done | Sixel images show |
| 4 | Graphics | #29 | 12-15h | #28 done | Kitty images show |
| 4 | Multiplexing | #30 | 20-25h | #39 done | SSH sessions work |
| 4 | Full-Stack | #32 | 15-18h | Wave 3 done | Marketplace UI done |
| 4 | Security | #33 | 6-8h | #32 done | GPG verification works |

---

## XV. Final Recommendations

### Immediate Actions (Today)
1. Launch Wave 1 with 5 parallel agents
2. Set up daily async standups
3. Create feature branches for each issue
4. Enable draft PRs for early feedback

### This Week (Days 1-6)
1. Complete Wave 1 and Wave 2
2. Daily integration testing
3. Documentation updates
4. Prepare for Wave 3 launch

### Next Sprint
1. Complete Wave 3
2. Begin Wave 4 (graphics protocols can be deferred)
3. User testing of navigation features
4. Performance benchmarking

### Strategic Decisions
- **Prioritize Wave 1+2**: These are high-impact, low-risk
- **Wave 3 as stretch goal**: Advanced features, can slip to next sprint
- **Wave 4 as Phase 6 work**: Long-term features, plan carefully
- **Quality over speed**: Better to ship solid Wave 1+2 than rush all 4

---

**Orchestration Ready**: This plan is executable immediately with clear agent assignments, dependencies, and success criteria. Each wave is independent enough for parallel work while respecting necessary sequencing.

**Expected Timeline**:
- Wave 1: 2 days
- Wave 2: 2 days
- Wave 3: 2-3 days
- Wave 4: 5-7 days (can be phased)

**Total**: 11-14 days with full parallel execution, ~25-30 days sequential

---

**Status**: READY FOR EXECUTION
**Approval**: Awaiting orchestrator launch command
**Next Step**: Launch Wave 1 agents with specified tasks

