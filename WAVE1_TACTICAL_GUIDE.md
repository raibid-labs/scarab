# Wave 1 Tactical Guide - Audit 003 Navigation System

**Status**: Ready for Execution
**Duration**: 2-3 weeks
**Team Size**: 5-6 engineers
**Critical Path**: #45 → #46 → #50

---

## Team Roles for Wave 1

```
Test Engineer         → #45 (State Isolation Tests)
Core Specialist       → #46 (Lifecycle Management)
Security Specialist   → #47 (Rate Limiting & Sandboxing)
Config Specialist     → #48 (Keymaps & Conflict Resolution)
Observability Expert  → #49 (Metrics & Telemetry)
Technical Writer      → #50 (Documentation - Week 2)
```

**Sync Cadence**: Daily standup (9:15am), Friday retro (4pm)

---

## Issue #45: State Isolation Integration Tests (3-4 days)

### Your Mission
Build a rock-solid test foundation that proves pane state never leaks during navigation.

### Architecture Overview

```
Navigation Events:
  user input (Ctrl+H) → Client → Daemon → Navigation::switch_pane(id)
                                            ↓
                                      PaneManager
                                            ↓
                                      State Snapshot
                                            ↓
                                      Pane Deactivate Hooks
                                            ↓
                                      Pane Activate Hooks
                                            ↓
                                      State Restore
```

### Implementation Plan

**Step 1: Test Harness (Day 1 afternoon)**
- Create `/home/beengud/raibid-labs/scarab/crates/scarab-nav/tests/integration/mod.rs`
- Set up test fixtures: multi-pane sessions, plugin mocks
- Implement `TestSession` helper struct for spawning nav systems

**Step 2: Core Tests (Day 2-3)**
- `test_pane_state_isolation_on_switch()` - verify states don't cross panes
- `test_concurrent_navigation_safe()` - 100 concurrent switches, no panics
- `test_plugin_hooks_fire_correctly()` - capture all lifecycle events
- `test_state_rollback_on_error()` - navigation fails gracefully

**Step 3: Stress & Property Tests (Day 4 morning)**
- `quickcheck`: generate random pane switch sequences (1000+ permutations)
- Verify invariant: "no pane state should be accessible from sibling pane"
- Run under sanitizers (thread-sanitizer, address-sanitizer)

### Code Skeleton

```rust
// crates/scarab-nav/tests/integration/mod.rs

#[tokio::test]
async fn test_pane_state_isolation_on_switch() {
    let mut session = TestSession::new(3); // 3 panes

    // Set unique state in each pane
    session.pane(0).set_state("data_a");
    session.pane(1).set_state("data_b");
    session.pane(2).set_state("data_c");

    // Switch to pane 1, verify only pane 1 state accessible
    session.navigate_to_pane(1).await;
    assert_eq!(session.current_state(), "data_b");
    assert!(!session.current_state().contains("data_a"));
    assert!(!session.current_state().contains("data_c"));

    // Switch to pane 2, verify isolation still holds
    session.navigate_to_pane(2).await;
    assert_eq!(session.current_state(), "data_c");
}

#[tokio::test]
async fn test_concurrent_navigation_safe() {
    let session = Arc::new(TestSession::new(10));
    let mut handles = vec![];

    for i in 0..100 {
        let sess = Arc::clone(&session);
        let h = tokio::spawn(async move {
            sess.navigate_to_pane(i % 10).await;
        });
        handles.push(h);
    }

    // All should complete without panic
    for h in handles {
        h.await.expect("thread panicked");
    }
}
```

### Acceptance Criteria
- [ ] 100% coverage of `Navigation::switch_pane()` code paths
- [ ] All tests pass with `--release` (optimizations enabled)
- [ ] Stress test runs 100+ rapid switches without panics
- [ ] Property-based test generates 1000+ random sequences
- [ ] CI runs tests under thread-sanitizer

### Success Signal
**Green checkmark on GitHub**: All test jobs pass, coverage ≥95%

---

## Issue #46: State Lifecycle Management (4-5 days)

### Your Mission
Build the nervous system of the navigation system—hooks that fire at the right time, state that's captured and restored cleanly.

### Architecture Overview

```
┌─────────────────────────────────────────┐
│      Navigation System (Daemon)         │
├─────────────────────────────────────────┤
│                                         │
│  ┌──────────────────────────────────┐  │
│  │   PaneManager                    │  │
│  │                                  │  │
│  │  ┌─────────────┐  ┌───────────┐ │  │
│  │  │  Active     │  │  Inactive │ │  │
│  │  │  Pane       │  │  Pane(s)  │ │  │
│  │  │             │  │           │ │  │
│  │  └─────────────┘  └───────────┘ │  │
│  └──────────────────────────────────┘  │
│                 ↑                       │
│                 │ Event Loop            │
│                 │                       │
│  ┌──────────────────────────────────┐  │
│  │  Lifecycle Hooks (Plugin-facing) │  │
│  │                                  │  │
│  │  • on_pane_activate              │  │
│  │  • on_pane_deactivate            │  │
│  │  • on_tab_switch                 │  │
│  │  • on_state_save                 │  │
│  │  • on_state_restore              │  │
│  └──────────────────────────────────┘  │
└─────────────────────────────────────────┘
```

### Implementation Plan

**Step 1: State Snapshot Mechanism (Day 1-2)**
- Create `StateSnapshot` struct in `scarab-nav/src/state.rs`
  ```rust
  pub struct StateSnapshot {
      pane_id: PaneId,
      timestamp: u64,
      grid: Vec<u8>,          // VTE grid
      plugin_state: Map<PluginId, Vec<u8>>, // Plugin-owned state
      cursor_pos: (u16, u16),
      mode_flags: u16,        // Insert/replace/etc
  }
  ```
- Implement `PaneManager::snapshot_state()` and `restore_state()`
- Ensure serialization is deterministic (for testing)

**Step 2: Lifecycle Hooks (Day 2-3)**
- Define hook traits in `scarab-plugin-api/src/lifecycle.rs`:
  ```rust
  pub trait NavigationHook: Send + Sync {
      async fn on_pane_activate(&self, pane_id: PaneId);
      async fn on_pane_deactivate(&self, pane_id: PaneId);
      async fn on_tab_switch(&self, from: TabId, to: TabId);
      async fn on_state_save(&self, snapshot: &StateSnapshot);
      async fn on_state_restore(&self, snapshot: &StateSnapshot);
  }
  ```
- Register plugins as listeners when they load
- Call hooks at correct points in `navigate_to_pane()` flow

**Step 3: Error Recovery (Day 3-4)**
- If hook panics, catch it, log, continue (never crash nav)
- Implement hook timeout: if hook takes >1s, cancel it
- Rollback state if restoration fails

**Step 4: Event Ring Buffer (Day 4)**
- Lock-free ringbuffer for navigation events (optional, nice-to-have)
- Pre-allocate 10MB for typical workload
- Use for debugging/telemetry downstream

### Code Skeleton

```rust
// crates/scarab-nav/src/lifecycle.rs

pub struct LifecycleManager {
    hooks: Vec<Arc<dyn NavigationHook>>,
    event_buffer: RingBuffer<NavEvent>,
}

impl LifecycleManager {
    pub async fn fire_pane_activate(&self, pane_id: PaneId) {
        for hook in &self.hooks {
            match tokio::time::timeout(
                Duration::from_secs(1),
                hook.on_pane_activate(pane_id),
            ).await {
                Ok(_) => {},
                Err(_) => log::warn!("Hook timeout for pane_activate"),
            }
        }
    }
}

// crates/scarab-nav/src/lib.rs

pub struct PaneManager {
    lifecycle: LifecycleManager,
    state_snapshots: LRUCache<PaneId, StateSnapshot>,
}

impl PaneManager {
    pub async fn switch_pane(&mut self, target: PaneId) -> Result<()> {
        // Save current pane state
        let current = self.active_pane_id();
        let snapshot = self.snapshot_state(current)?;
        self.state_snapshots.insert(current, snapshot);

        // Fire deactivate hooks
        self.lifecycle.fire_pane_deactivate(current).await;

        // Switch (update pointer)
        self.set_active_pane(target);

        // Fire activate hooks
        self.lifecycle.fire_pane_activate(target).await;

        Ok(())
    }
}
```

### Acceptance Criteria
- [ ] All lifecycle hooks fire in correct order (verified by #45 tests)
- [ ] State snapshots are accurate and deterministic
- [ ] Pane switch latency <50ms (measured by #49 metrics)
- [ ] Crashing plugin doesn't crash navigation
- [ ] State recovery works after daemon restart

### Success Signal
**#45 integration tests all pass** without modification

---

## Issue #47: Security - Plugin Navigation API Guardrails (3-4 days)

### Your Mission
Build the security boundary that prevents malicious plugins from:
- Manipulating panes they don't own
- Flooding navigation commands
- Crashing the system
- Accessing other plugins' state

### Implementation Plan

**Step 1: Rate Limiting (Day 1)**
- Create `RateLimiter` in `scarab-nav/src/security.rs`
  ```rust
  pub struct PluginRateLimiter {
      limits: Map<PluginId, TokenBucket>,
      default_limit: u32, // ops/sec
  }
  ```
- Token bucket algorithm: 100 ops/sec default
- Configurable per-plugin via #48 config
- Reject with `NavigationError::RateLimited` when exceeded

**Step 2: Capability Model (Day 1-2)**
- In plugin manifest (or descriptor), declare permissions:
  ```toml
  [plugin.my-nav-plugin]
  navigation_permissions = [
      "pane.focus",
      "tab.switch",
  ]
  # NOT allowed: "pane.kill", "plugin.load"
  ```
- Check capability at hook registration time
- Return error if plugin lacks permission
- Log all permission checks

**Step 3: State Sandboxing (Day 2-3)**
- Plugin can only access its own state bucket
- Create `PluginStateVault` that wraps `StateSnapshot`
  ```rust
  pub struct PluginStateVault {
      owner_plugin: PluginId,
      state: Vec<u8>,
  }

  impl StateVault {
      pub fn read(&self, pane: PaneId) -> Result<Vec<u8>> {
          // Only allow read if plugin owns this state
      }

      pub fn write(&self, pane: PaneId, data: Vec<u8>) -> Result<()> {
          // Only allow write if plugin owns this state
      }
  }
  ```

**Step 4: Audit Logging (Day 3-4)**
- Log all navigation commands: `[PLUGIN: plugin-a] [CMD: pane.focus(3)] [RESULT: OK]`
- Include timestamp, plugin ID, command, result
- Write to `~/.scarab/nav-audit.log` (rotate every 10MB)
- Queryable for security investigations

### Code Skeleton

```rust
// crates/scarab-nav/src/security.rs

pub struct SecurityContext {
    plugin_id: PluginId,
    capabilities: Set<Permission>,
    rate_limiter: Arc<RateLimiter>,
}

impl SecurityContext {
    pub fn check_rate_limit(&self) -> Result<()> {
        if self.rate_limiter.allow_operation()? {
            Ok(())
        } else {
            Err(NavigationError::RateLimited)
        }
    }

    pub fn check_permission(&self, perm: Permission) -> Result<()> {
        if self.capabilities.contains(&perm) {
            Ok(())
        } else {
            Err(NavigationError::PermissionDenied(perm))
        }
    }
}

pub async fn navigate_with_security(
    ctx: &SecurityContext,
    target: PaneId,
) -> Result<()> {
    ctx.check_rate_limit()?;
    ctx.check_permission(Permission::PaneFocus)?;

    log_audit_event(&ctx.plugin_id, "navigate_pane", &target);

    // actual navigation...
    Ok(())
}
```

### Acceptance Criteria
- [ ] Rate limiter blocks plugins exceeding 100 ops/sec
- [ ] Audit log captures 100% of navigation calls
- [ ] Plugin can't access state it doesn't own (verified by tests)
- [ ] Security review sign-off from team lead
- [ ] No performance regression (<1% overhead)

### Success Signal
**Security audit passes** with zero findings (critical/high)

---

## Issue #48: Config - Navigation Keymaps & Plugin Conflict Resolution (2-3 days)

### Your Mission
Make navigation keybindings user-configurable and resolve plugin conflicts elegantly.

### Implementation Plan

**Step 1: Config Schema (Day 1 morning)**
- Define TOML schema in `/home/beengud/raibid-labs/scarab/crates/scarab-config/examples/default_config.toml`:
  ```toml
  [navigation]
  # Keymaps: key → action
  keymaps.ctrl_h = "navigate.previous_pane"
  keymaps.ctrl_l = "navigate.next_pane"
  keymaps.ctrl_j = "navigate.pane_below"
  keymaps.ctrl_k = "navigate.pane_above"

  # Plugin priority: first match wins on conflict
  plugin_priority = [
      "built-in-nav",
      "tmux-compat",
      "custom-nav-plugin",
  ]
  ```
- Create Rust types that deserialize this config
- Validate at startup (unknown actions, duplicate keys)

**Step 2: Conflict Detection (Day 1 afternoon)**
- When plugins register keybindings, check if already bound
  ```rust
  pub fn register_keymap(
      plugin_id: &PluginId,
      bindings: &[(KeyCombo, Action)],
  ) -> Result<()> {
      for (key, action) in bindings {
          if self.keymap.contains_key(key) {
              // Conflict detected
              let priority = self.plugin_priority.index(plugin_id)?;
              let existing = self.keymap.get(key)?;
              let existing_priority = self.plugin_priority.index(&existing.owner)?;

              if priority > existing_priority {
                  log::info!("Keymap override: {:?} by {}", key, plugin_id);
                  self.keymap.insert(*key, binding(plugin_id, action));
              } else {
                  return Err(NavError::KeybindConflict(key.clone()));
              }
          }
      }
      Ok(())
  }
  ```

**Step 3: Hot-Reload (Day 2 morning)**
- Watch config file for changes (using `notify` crate)
- Re-parse and reload keymaps without daemon restart
- Test: modify config, press a key, verify new binding works

**Step 4: Default Config (Day 2 afternoon)**
- Ship sensible defaults (vim-like + tmux-compatible)
- Provide example configs (Vim, Emacs, default)
- Document in `WAVE1_TACTICAL_GUIDE.md`

### Code Skeleton

```rust
// crates/scarab-config/src/navigation.rs

#[derive(Deserialize)]
pub struct NavigationConfig {
    pub keymaps: Map<String, String>,  // "ctrl_h" → "navigate.previous_pane"
    pub plugin_priority: Vec<PluginId>,
}

pub struct KeymapRegistry {
    config: NavigationConfig,
    resolved: Map<KeyCombo, (PluginId, Action)>,
}

impl KeymapRegistry {
    pub fn reload_from_config(&mut self, config: NavigationConfig) -> Result<()> {
        self.config = config;
        self.resolved.clear();
        self.build_resolved_keymap()?;
        Ok(())
    }

    fn build_resolved_keymap(&mut self) -> Result<()> {
        for (key_str, action_str) in &self.config.keymaps {
            let key = KeyCombo::from_str(key_str)?;
            let action = Action::from_str(action_str)?;
            self.resolved.insert(key, ("built-in", action));
        }
        Ok(())
    }
}
```

### Acceptance Criteria
- [ ] Config parses without errors
- [ ] Keymap conflicts detected and reported
- [ ] Hot-reload works without crashes
- [ ] Default config ships with project
- [ ] Documentation provided

### Success Signal
**User can customize keybindings in 5 minutes** (config edit + reload)

---

## Issue #49: Telemetry - Pane Switch & Plugin Navigation Metrics (2-3 days)

### Your Mission
Instrument the navigation system so we can measure performance, detect bottlenecks, and debug issues in production.

### Implementation Plan

**Step 1: Metrics Collection (Day 1)**
- Define metrics in `scarab-daemon/src/metrics/navigation.rs`:
  ```rust
  pub struct NavMetrics {
      pane_switch_latency: Histogram,      // ms
      plugin_nav_commands: Counter,         // total
      state_isolation_violations: Counter,  // attempts
      rate_limit_hits: Counter,             // per plugin
  }
  ```
- Use `prometheus` crate for types
- Collect latency in `PaneManager::switch_pane()` using `Instant`

**Step 2: Plugin Command Tracking (Day 1-2)**
- Increment counter for each plugin navigation command
- Label by: plugin_id, command_type (pane.focus, tab.switch, etc)
- Example: `scarab_plugin_nav_commands_total{plugin="plugin-a",command="pane.focus"} 42`

**Step 3: Rate Limit Monitoring (Day 2)**
- Increment counter each time rate limiter rejects
- Label by plugin_id
- Helps identify aggressive plugins

**Step 4: Prometheus Endpoint (Day 2-3)**
- Expose HTTP endpoint at `127.0.0.1:9191/metrics`
- Scrape interval: 10s (reasonable for real-time monitoring)
- Can disable in config if not needed

### Code Skeleton

```rust
// crates/scarab-daemon/src/metrics/navigation.rs

use prometheus::{Histogram, Counter, HistogramOpts, CounterOpts, Registry};

pub struct NavMetrics {
    pub pane_switch_latency: Histogram,
    pub plugin_nav_commands: Counter,
    pub state_isolation_violations: Counter,
    pub rate_limit_hits: Counter,
}

impl NavMetrics {
    pub fn new(registry: &Registry) -> Result<Self> {
        let pane_switch_latency = Histogram::with_opts(
            HistogramOpts::new("scarab_pane_switch_latency_ms", "Pane switch latency")
                .buckets(vec![1.0, 5.0, 10.0, 50.0, 100.0, 500.0]),
        )?;
        registry.register(Box::new(pane_switch_latency.clone()))?;

        Ok(NavMetrics {
            pane_switch_latency,
            // ...
        })
    }
}

// In PaneManager::switch_pane()
let start = Instant::now();
// ... do the switch ...
let latency_ms = start.elapsed().as_secs_f64() * 1000.0;
self.metrics.pane_switch_latency.observe(latency_ms);
```

### Acceptance Criteria
- [ ] Metrics exposed on HTTP endpoint
- [ ] <1% CPU overhead from metrics collection
- [ ] Prometheus scrape succeeds
- [ ] Grafana dashboard template provided
- [ ] Example queries documented

### Success Signal
**Prometheus scrapes metrics**, Grafana shows live pane switch latency

---

## Issue #50: Documentation - Navigation Developer Guide (1-2 days)

### Your Mission
Help future developers understand and extend the navigation system.

### Implementation Plan

**Step 1: Architecture Guide (Day 1 morning)**
- Create `/home/beengud/raibid-labs/scarab/docs/navigation-guide.md`
- Sections:
  1. Overview (what is navigation, why it matters)
  2. Architecture diagram (state machine, lifecycle hooks)
  3. Key concepts (pane, tab, state snapshot, lifecycle)
  4. Plugin integration (how to write a nav plugin)
  5. Security model (capabilities, rate limiting, audit)
  6. Troubleshooting (common issues, debugging)

**Step 2: API Reference (Day 1 afternoon)**
- Create `/home/beengud/raibid-labs/scarab/docs/api-reference-navigation.md`
- Auto-generate from doc comments using `cargo doc`
- Include:
  - `PaneManager` methods
  - `NavigationHook` trait
  - Config schema
  - Error types

**Step 3: Code Examples (Day 2 morning)**
- Create `/home/beengud/raibid-labs/scarab/examples/nav-plugin-simple.rs`
  - Simple plugin that responds to pane activate events
  - Shows how to implement `NavigationHook`
  - Prints log message on each event
- Create `/home/beengud/raibid-labs/scarab/examples/nav-plugin-advanced.rs`
  - Plugin that tracks pane state and provides telemetry
  - Shows how to use `StateSnapshot`
  - Shows error handling

**Step 4: Review & Polish (Day 2 afternoon)**
- Team lead reviews for clarity and accuracy
- Fix any broken links or missing details
- Add any missing sections

### Acceptance Criteria
- [ ] Guide covers all issues #45-49
- [ ] Code examples compile and run
- [ ] API reference is complete and accurate
- [ ] Reviewed by team lead
- [ ] No typos or broken links

### Success Signal
**New engineer can write a nav plugin from scratch** using only the guide

---

## Daily Workflow for Wave 1 Teams

### Morning (9:15 AM Standup)
- What did you ship yesterday?
- What are you shipping today?
- Any blockers?
- 5 min per person, max 30 min total

### During Day
- Work on your issue
- Unblock teammates (code reviews, pair programming)
- Update tracking board (GitHub Projects)

### Evening (4 PM Friday Retro, other days ad-hoc)
- Celebrate wins
- Reflect on what went well
- Identify improvements for next week
- 30 min max

---

## Code Review Checklist for Wave 1

Before merging, ensure:
- [ ] Tests pass locally and in CI
- [ ] Code follows Rust fmt and clippy
- [ ] Benchmark scores don't regress
- [ ] Security review (for #47)
- [ ] Documentation updated
- [ ] Commit messages are clear

---

## Dependency Coordination

**Critical Path**:
1. #45 (Tests) must merge first → enables #46
2. #46 (Lifecycle) must be solid → enables #47
3. #47, #48, #49 can be parallel → depend only on earlier issues
4. #50 (Docs) last → documents everything

**Integration Points**:
- #46 result is main "product" of Wave 1
- #47 adds security layer to #46
- #48 consumes hooks from #46
- #49 instruments #46
- #50 explains #45-49

---

## Communication Channels

- **Blockers**: Post in team Slack immediately, don't wait for standup
- **Code review**: GitHub PR, tag code owner, 4-hour turnaround SLA
- **Questions**: Discussion in PR/commit, or ask in Slack
- **Escalations**: Tag tech lead or coach

---

## Success Criteria Summary

| Issue | Metric | Target |
|-------|--------|--------|
| #45 | Test Coverage | ≥95% |
| #45 | Stress Tests | 100+ switches without panic |
| #46 | Pane Switch Latency | <50ms p95 |
| #46 | Hook Reliability | 100% fire correctly |
| #47 | Rate Limit Accuracy | ±5% of configured limit |
| #47 | Audit Log Completeness | 100% of commands logged |
| #48 | Config Parse Time | <10ms |
| #48 | Hot Reload Uptime | 100% (no crashes) |
| #49 | Metrics Overhead | <1% CPU |
| #49 | Prometheus Scrape | <100ms |
| #50 | Documentation | Complete, reviewed |

---

## Escalation Contacts

- **Tech Lead**: @tech-lead
- **Security**: @security-team
- **Performance**: @perf-coach
- **Documentation**: @tech-writer

---

**Wave 1 kicks off Monday. You've got this.**

Every issue is a victory waiting to happen. Stay focused, stay humble, and keep the team in sync. This is going to be legendary.
