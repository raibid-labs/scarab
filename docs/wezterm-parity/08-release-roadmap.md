# WezTerm Parity: Release Roadmap & Milestones

**Document:** Release Planning for v0.2.0
**Date:** December 2, 2025

## Version Strategy

The WezTerm parity work will be released as **v0.2.0**, representing a significant feature addition. The release will follow Scarab's established release process with alpha → beta → stable progression.

## Milestone Overview

```
v0.2.0-alpha.1 ──► v0.2.0-alpha.2 ──► v0.2.0-alpha.3 ──► v0.2.0-beta.1 ──► v0.2.0
    │                   │                   │                  │              │
    │                   │                   │                  │              │
    WS-1 + WS-2         WS-3 + WS-4         WS-5 + WS-6      Stabilize     Release
   (Foundations)       (User Features)    (Advanced)         (Polish)
```

## Detailed Milestones

### Milestone 1: v0.2.0-alpha.1 — Foundations

**Target Date:** Week 2-3
**Theme:** Object Model & Event System Infrastructure

**Deliverables:**
- [WS-1] Object Model Infrastructure (Phase 1-3)
  - [ ] `ObjectHandle` and `ObjectType` definitions
  - [ ] `ObjectRegistry` trait and implementations
  - [ ] `WindowProxy` with basic methods
  - [ ] `PaneProxy` with basic methods
  - [ ] IPC query/response messages

- [WS-2] Event System (Phase 1-3)
  - [ ] `EventType` enum with 20+ event types
  - [ ] `EventRegistry` with priority dispatch
  - [ ] Window events (focus, resize)
  - [ ] Pane events (title, output)
  - [ ] Backward compatibility with existing hooks

**PR Strategy:**
- PR 1: `feat(plugin-api): add object handle infrastructure`
- PR 2: `feat(plugin-api): add event registry system`
- PR 3: `feat(daemon): implement object registry for sessions`
- PR 4: `feat(client): implement object registry for Bevy entities`
- PR 5: `feat(protocol): add query/response IPC messages`

**Release Criteria:**
- [ ] Object handles can be created and resolved
- [ ] Basic events fire correctly
- [ ] No regressions in existing plugin system
- [ ] All tests pass

**CHANGELOG Preview:**
```markdown
## [0.2.0-alpha.1] - YYYY-MM-DD

### Added
- Object model infrastructure with Window, Pane, Tab proxies
- Event system with 20+ event types
- Object handles for cross-process object references
- `EventRegistry` for plugin event subscriptions
```

---

### Milestone 2: v0.2.0-alpha.2 — User Features

**Target Date:** Week 4-5
**Theme:** Status Bar API & Key Tables

**Deliverables:**
- [WS-1] Object Model (Phase 4-5)
  - [ ] `TabProxy` implementation
  - [ ] Object navigation (`pane.Tab()`, `tab.Window()`)
  - [ ] Updated `PluginContext` with object access

- [WS-2] Event System (Phase 4-5)
  - [ ] `UpdateStatus` event with render item return
  - [ ] Custom events (`On`/`Emit`)
  - [ ] Tab events (created, closed, switched)

- [WS-3] Status Bar API (Full)
  - [ ] `RenderItem` enum
  - [ ] `window.SetRightStatus()` / `SetLeftStatus()`
  - [ ] Periodic and event-driven updates
  - [ ] Built-in status components

- [WS-4] Key Tables (Phase 1-4)
  - [ ] `KeyTableStack` with resolution algorithm
  - [ ] `LeaderKeyState` with timeout
  - [ ] `ActivateKeyTable` / `PopKeyTable` actions
  - [ ] Config parsing for key tables

**PR Strategy:**
- PR 6: `feat(plugin-api): add RenderItem and format API`
- PR 7: `feat(client): implement status bar rendering`
- PR 8: `feat(client): add key table stack system`
- PR 9: `feat(config): add key table parsing`
- PR 10: `feat(plugin-api): complete object model navigation`

**Release Criteria:**
- [ ] Status bar can be customized from Fusabi scripts
- [ ] Leader key + key tables work
- [ ] Mode indicator displays correctly
- [ ] Documentation for new APIs

**CHANGELOG Preview:**
```markdown
## [0.2.0-alpha.2] - YYYY-MM-DD

### Added
- Status bar rendering API with `RenderItem` types
- Key tables for modal editing (resize, activate modes)
- Leader key support with configurable timeout
- Mode indicator in status bar
- Built-in status components: clock, cwd, process name
```

---

### Milestone 3: v0.2.0-alpha.3 — Advanced Features

**Target Date:** Week 6-7
**Theme:** Image Protocols & Copy Mode

**Deliverables:**
- [WS-4] Key Tables (Phase 5)
  - [ ] Default key tables (copy_mode, search_mode)
  - [ ] Mode indicator integration

- [WS-5] Image Protocols
  - [ ] iTerm2 protocol parser (OSC 1337)
  - [ ] Image shared memory region
  - [ ] Bevy sprite rendering
  - [ ] `imgcat` utility
  - [ ] Image lifecycle management (scroll, delete)
  - [ ] Optional: Kitty basic support

- [WS-6] Copy Mode
  - [ ] `CopyModeState` resource
  - [ ] Navigation (hjkl, word, page)
  - [ ] Selection modes (cell, line, block)
  - [ ] Text extraction and clipboard
  - [ ] Search with match highlighting

**PR Strategy:**
- PR 11: `feat(daemon): add iTerm2 image protocol parser`
- PR 12: `feat(protocol): add image shared memory region`
- PR 13: `feat(client): implement image rendering`
- PR 14: `feat(client): add copy mode system`
- PR 15: `feat(client): implement copy mode search`
- PR 16: `chore: add imgcat utility`

**Release Criteria:**
- [ ] `imgcat` displays images inline
- [ ] Copy mode navigation works
- [ ] Selection and yank to clipboard work
- [ ] Search highlights matches

**CHANGELOG Preview:**
```markdown
## [0.2.0-alpha.3] - YYYY-MM-DD

### Added
- iTerm2 image protocol support (inline images)
- Copy mode with vim-like navigation
- Visual, line, and block selection modes
- Copy mode search with highlighting
- `imgcat` utility for testing images
```

---

### Milestone 4: v0.2.0-beta.1 — Stabilization

**Target Date:** Week 8
**Theme:** Polish, Testing, Documentation

**Deliverables:**
- Integration testing across all workstreams
- Performance profiling and optimization
- API documentation (rustdoc)
- User documentation (guides)
- Example configurations
- Bug fixes from alpha testing

**PR Strategy:**
- PR 17: `test: add integration tests for object model`
- PR 18: `test: add integration tests for key tables`
- PR 19: `docs: add WezTerm migration guide`
- PR 20: `perf: optimize image rendering`
- PR 21-N: Bug fix PRs as needed

**Release Criteria:**
- [ ] All integration tests pass
- [ ] Performance targets met
- [ ] Documentation complete
- [ ] No known critical bugs
- [ ] API stable for this release

**CHANGELOG Preview:**
```markdown
## [0.2.0-beta.1] - YYYY-MM-DD

### Added
- Comprehensive integration test suite
- WezTerm migration guide

### Changed
- Optimized image rendering performance

### Fixed
- [List of bugs fixed from alpha testing]
```

---

### Milestone 5: v0.2.0 — Stable Release

**Target Date:** Week 9-10
**Theme:** Release

**Deliverables:**
- Release candidate testing (if needed)
- Final bug fixes
- Release notes
- Announcement preparation
- Package manager updates

**Release Criteria:**
- [ ] RC tested for 1 week (if applicable)
- [ ] Zero known critical bugs
- [ ] All documentation finalized
- [ ] CHANGELOG complete
- [ ] Package managers updated

**CHANGELOG Preview:**
```markdown
## [0.2.0] - YYYY-MM-DD

### Summary
This release brings WezTerm-inspired programmability to Scarab, including:
- Full object model with Window, Pane, Tab proxies
- Rich event system with 20+ event types
- Programmable status bars
- Modal editing with key tables
- iTerm2 image protocol support
- Vim-like copy mode

### Added
[Full list from alpha/beta releases]

### Changed
[Full list from alpha/beta releases]

### Fixed
[Full list from alpha/beta releases]
```

---

## PR Workflow Summary

### Branch Naming for This Work

```
feature/ws1-object-model-phase1
feature/ws1-object-model-phase2
feature/ws2-event-system
feature/ws3-status-bar-api
feature/ws4-key-tables
feature/ws5-image-iterm2
feature/ws5-image-kitty
feature/ws6-copy-mode
feature/ws6-copy-mode-search
```

### PR Size Guidelines

- **Small PRs** (< 200 lines): Single-reviewer, quick turnaround
- **Medium PRs** (200-500 lines): Standard review, 1-2 days
- **Large PRs** (> 500 lines): Split if possible, detailed review

### Squash Merge Commit Examples

```
feat(plugin-api): add object handle infrastructure (#42)

Implements handle-based proxies for Window, Pane, and Tab objects.
Includes ObjectHandle struct with type, id, and generation fields.
Adds ObjectRegistry trait with client/daemon implementations.

Part of WS-1: Object Model Infrastructure

🤖 Generated with Claude Code
Co-Authored-By: Claude <noreply@anthropic.com>
```

---

## Milestone Tracking

### GitHub Milestones

Create these milestones in GitHub:

| Milestone | Due Date | Description |
|-----------|----------|-------------|
| `v0.2.0-alpha.1` | Week 3 | Object Model + Event System |
| `v0.2.0-alpha.2` | Week 5 | Status Bar + Key Tables |
| `v0.2.0-alpha.3` | Week 7 | Images + Copy Mode |
| `v0.2.0-beta.1` | Week 8 | Stabilization |
| `v0.2.0` | Week 10 | Stable Release |

### Progress Tracking

Use GitHub Projects or this checklist:

**v0.2.0-alpha.1 Progress:**
- [ ] WS-1 Phase 1: Core Infrastructure
- [ ] WS-1 Phase 2: Window Object
- [ ] WS-1 Phase 3: Pane Object
- [ ] WS-2 Phase 1: Event Infrastructure
- [ ] WS-2 Phase 2: Core Events
- [ ] WS-2 Phase 3: Tab/Pane Events

**v0.2.0-alpha.2 Progress:**
- [ ] WS-1 Phase 4-5: Tab + Integration
- [ ] WS-2 Phase 4-5: Status + Custom Events
- [ ] WS-3: Status Bar API (Full)
- [ ] WS-4 Phase 1-4: Key Tables

**v0.2.0-alpha.3 Progress:**
- [ ] WS-4 Phase 5: Default Tables
- [ ] WS-5: Image Protocols
- [ ] WS-6: Copy Mode

---

## Risk Mitigation

### Schedule Risks

| Risk | Mitigation |
|------|------------|
| Workstream takes longer than expected | Alpha releases can slip; beta/stable dates are firm |
| Blocking dependencies between workstreams | WS-5 (images) is fully parallel, can absorb delays |
| Complex IPC changes | Early prototyping in alpha.1 |

### Technical Risks

| Risk | Mitigation |
|------|------------|
| Object model performance | Profile early, cache aggressively |
| Image memory exhaustion | LRU cache with configurable limits |
| Key table conflicts | Clear documentation, validation in parser |

### Communication

- **Weekly status update** in GitHub Discussions
- **Alpha release notes** for each alpha
- **Beta announcement** to gather wider testing
- **Stable release announcement** to all channels

---

## References

- [Release Process](../RELEASE_PROCESS.md)
- [Versioning Strategy](../VERSIONING.md)
- [Contributing Guidelines](../../CONTRIBUTING.md)
- [Workstream Details](./07-workstreams.md)
