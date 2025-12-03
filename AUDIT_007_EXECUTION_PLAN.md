# Audit 007 Completion Sprint - Execution Plan

**Mission:** Complete 12 open GitHub issues from audit 007
**Sprint Start:** 2025-12-03
**Repository:** /home/beengud/raibid-labs/scarab

## Phase Overview

### Phase 1: Foundation (LAUNCH NOW - Parallel Execution)
**Status:** LAUNCHING
**ETA:** 2-3 hours
**Dependencies:** None - all tasks are independent

- Issue #61: Central documentation index + navigation doc consolidation
- Issue #62: Central TESTING.md guide with one-line commands
- Issue #71: Documentation portal (mdBook + rustdoc integration)
- Issue #65: Justfile targets for test commands

**Strategy:** Launch all 4 tasks in parallel as they don't depend on each other

### Phase 2: Testing Infrastructure (After Phase 1)
**Status:** WAITING
**ETA:** 3-4 hours
**Dependencies:** Phase 1 (justfile infrastructure)

- Issue #63: Expand ratatui-testlib coverage
- Issue #64: BevyTuiTestHarness (upstream tracking - update issue only)

**Strategy:** Sequential execution, #64 is just documentation update

### Phase 3: Plugin Development (After Phase 2 - Parallel)
**Status:** WAITING
**ETA:** 6-8 hours
**Dependencies:** Testing infrastructure ready

- Issue #66: Telemetry HUD plugin
- Issue #67: Diagnostics recorder/replay plugin
- Issue #68: Accessibility plugin
- Issue #69: Graphics inspector plugin
- Issue #70: Bevy UI inspector plugin

**Strategy:** All 5 plugins are independent, launch in parallel

### Phase 4: Release Planning (After Phase 3)
**Status:** WAITING
**ETA:** 1 hour
**Dependencies:** All previous phases complete

- Issue #72: v0.2.0-alpha stabilization sprint plan

**Strategy:** Update with actual completion status and timeline

## Issue Details

### Documentation Track (Priority 1)

#### Issue #61: Central documentation index
**Type:** Documentation
**Agent:** general-purpose
**Files to create/modify:**
- `/home/beengud/raibid-labs/scarab/docs/README.md` (central index)
- Consolidate navigation docs from scattered locations
- Create clear hierarchy
**Estimated time:** 1 hour

#### Issue #62: Central TESTING guide
**Type:** Documentation
**Agent:** general-purpose
**Files to modify:**
- `/home/beengud/raibid-labs/scarab/TESTING.md` (already exists, enhance it)
- Add one-line command reference table
- Add quickstart section at top
- Link to specialized test docs
**Estimated time:** 45 minutes

#### Issue #71: Documentation portal
**Type:** Documentation + Infrastructure
**Agent:** general-purpose
**Files to create:**
- `/home/beengud/raibid-labs/scarab/docs/book/` (mdBook structure)
- `/home/beengud/raibid-labs/scarab/docs/book/book.toml`
- `/home/beengud/raibid-labs/scarab/docs/book/src/SUMMARY.md`
- Integration scripts for rustdoc
**Dependencies:** mdBook (can be installed if not present)
**Estimated time:** 2 hours

### Testing Track (Priority 2)

#### Issue #65: Justfile test targets
**Type:** Build Infrastructure
**Agent:** systems-programming:rust-pro
**Files to modify:**
- `/home/beengud/raibid-labs/scarab/justfile` (already exists)
**Add targets:**
- `just test-golden` - Run golden tests
- `just test-ratatui` - Run ratatui-testlib tests
- `just test-headless` - Run headless harness tests
- `just test-all-visual` - Run all visual tests
**Estimated time:** 30 minutes

#### Issue #63: Expand ratatui-testlib coverage
**Type:** Testing
**Agent:** systems-programming:rust-pro
**Files to create/modify:**
- New test files in `/home/beengud/raibid-labs/scarab/crates/scarab-client/tests/`
- Add Kitty/Sixel/nav assertions
- Expand existing ratatui test suite
**Dependencies:** Phase 1 complete (justfile targets)
**Estimated time:** 3 hours

#### Issue #64: BevyTuiTestHarness upstream tracking
**Type:** Documentation (tracking only)
**Agent:** general-purpose
**Action:** Update issue with tracking info, link to upstream Bevy discussions
**Estimated time:** 15 minutes

### Plugin Track (Priority 3)

#### Issue #66: Telemetry HUD plugin
**Type:** Feature (Plugin)
**Agent:** systems-programming:rust-pro
**Files to create:**
- `/home/beengud/raibid-labs/scarab/plugins/telemetry-hud/`
- `telemetry-hud.fsx` - Main plugin code
- `plugin.toml` - Plugin manifest
- `README.md` - Plugin documentation
**Features:** Display FPS, cache stats, hint counts
**Pattern:** Follow scarab-nav plugin structure
**Estimated time:** 2 hours

#### Issue #67: Diagnostics recorder/replay plugin
**Type:** Feature (Plugin)
**Agent:** systems-programming:rust-pro
**Files to create:**
- `/home/beengud/raibid-labs/scarab/plugins/diagnostics-recorder/`
- Session recording/replay functionality
- Export terminal sessions
**Estimated time:** 2.5 hours

#### Issue #68: Accessibility plugin
**Type:** Feature (Plugin)
**Agent:** frontend-developer (for UI integration)
**Files to create:**
- `/home/beengud/raibid-labs/scarab/plugins/accessibility/`
- Screen reader support
- Text export for accessibility tools
**Estimated time:** 2 hours

#### Issue #69: Graphics inspector plugin
**Type:** Feature (Plugin)
**Agent:** systems-programming:rust-pro
**Files to create:**
- `/home/beengud/raibid-labs/scarab/plugins/graphics-inspector/`
- Sixel/Kitty/iTerm2 image detection
- Image metadata display
**Estimated time:** 2 hours

#### Issue #70: Bevy UI inspector plugin
**Type:** Feature (Plugin)
**Agent:** frontend-developer
**Files to create:**
- `/home/beengud/raibid-labs/scarab/plugins/bevy-ui-inspector/`
- ECS entity browser
- Component inspector overlay
**Estimated time:** 2.5 hours

### Release Track (Priority 4)

#### Issue #72: v0.2.0-alpha stabilization
**Type:** Planning
**Agent:** general-purpose
**Action:** Create sprint plan document with dependencies, timeline, testing checklist
**Dependencies:** ALL previous issues complete
**Estimated time:** 1 hour

## Technical Patterns to Follow

### Nushell Scripts (Not Bash!)
All new scripts must be written in Nushell (.nu extension):
```nushell
#!/usr/bin/env nu
# Example Nushell script
def main [] {
  echo "Scarab testing script"
  cargo test --workspace
}
```

### Plugin Structure
Follow the established pattern from existing plugins:
```
plugins/plugin-name/
├── plugin-name.fsx      # Main Fusabi script
├── plugin.toml          # Manifest
├── README.md            # Documentation
└── tests/               # Optional tests
    └── plugin_test.rs
```

### Documentation Style
- Use mdBook format for guides
- Link between documents extensively
- Include code examples
- Add "Last updated" footer with date

### Testing Patterns
- Use existing harness infrastructure
- Follow patterns from scarab-client/tests/
- Add to navigation smoke test if relevant
- Use headless mode for CI compatibility

## Success Criteria

### Phase 1 Complete When:
- [ ] Central docs index exists and is comprehensive
- [ ] TESTING.md has one-line command reference
- [ ] mdBook portal is set up and builds
- [ ] Justfile has all new test targets

### Phase 2 Complete When:
- [ ] Ratatui-testlib has Kitty/Sixel/nav tests
- [ ] Issue #64 updated with upstream tracking info
- [ ] All tests pass in CI

### Phase 3 Complete When:
- [ ] All 5 plugins exist and load successfully
- [ ] Each plugin has documentation
- [ ] Plugins follow established patterns
- [ ] Example usage is documented

### Phase 4 Complete When:
- [ ] v0.2.0-alpha sprint plan is documented
- [ ] Dependencies are mapped
- [ ] Timeline is realistic
- [ ] Testing checklist is comprehensive

## Agent Launch Sequence

### Immediate Launch (Phase 1 - Parallel):
1. Agent-Doc-Index: Issue #61 (general-purpose)
2. Agent-Testing-Guide: Issue #62 (general-purpose)
3. Agent-Doc-Portal: Issue #71 (general-purpose)
4. Agent-Justfile: Issue #65 (systems-programming:rust-pro)

### After Phase 1 (Phase 2):
5. Agent-Ratatui-Tests: Issue #63 (systems-programming:rust-pro)
6. Agent-Upstream-Track: Issue #64 (general-purpose)

### After Phase 2 (Phase 3 - Parallel):
7. Agent-Telemetry-HUD: Issue #66 (systems-programming:rust-pro)
8. Agent-Diagnostics: Issue #67 (systems-programming:rust-pro)
9. Agent-Accessibility: Issue #68 (frontend-developer)
10. Agent-Graphics: Issue #69 (systems-programming:rust-pro)
11. Agent-Bevy-Inspector: Issue #70 (frontend-developer)

### After Phase 3 (Phase 4):
12. Agent-Release-Plan: Issue #72 (general-purpose)

## Progress Tracking

**Phase 1:** 0/4 complete
**Phase 2:** 0/2 complete
**Phase 3:** 0/5 complete
**Phase 4:** 0/1 complete

**Overall:** 0/12 issues complete

---

**Created:** 2025-12-03
**Last Updated:** 2025-12-03
**Status:** Phase 1 launching now
