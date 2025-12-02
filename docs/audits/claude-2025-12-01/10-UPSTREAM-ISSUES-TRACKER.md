# Upstream Issues Tracker
**Date Created:** December 1, 2025
**Last Updated:** December 1, 2025

---

## Overview

This document tracks feature requests filed in upstream repositories based on Scarab's audit findings. All issues have been filed with clear problem statements, proposed solutions, and code examples.

---

## Fusabi-lang/fusabi Issues

**Repository:** https://github.com/fusabi-lang/fusabi

### Critical Priority

| Issue # | Title | Status | Priority | Link |
|---------|-------|--------|----------|------|
| #148 | Implement event hook system for terminal lifecycle events | Filed | CRITICAL | https://github.com/fusabi-lang/fusabi/issues/148 |
| #149 | Add API to query terminal state and process information | Filed | CRITICAL | https://github.com/fusabi-lang/fusabi/issues/149 |

**Impact:** These two issues block most plugin use cases in Scarab. Without event hooks and terminal state queries, plugins are limited to basic keybindings only.

### High Priority

| Issue # | Title | Status | Priority | Link |
|---------|-------|--------|----------|------|
| #150 | Add API for programmatic pane/window control | Filed | HIGH | https://github.com/fusabi-lang/fusabi/issues/150 |
| #151 | Add API for status bar and UI element formatting | Filed | HIGH | https://github.com/fusabi-lang/fusabi/issues/151 |

**Impact:** These enable advanced automation and visual customization in Scarab plugins.

### Medium Priority

| Issue # | Title | Status | Priority | Link |
|---------|-------|--------|----------|------|
| #152 | Add typed configuration schema with validation | Filed | MEDIUM | https://github.com/fusabi-lang/fusabi/issues/152 |
| #153 | Allow plugins to register custom commands in command palette | Filed | MEDIUM | https://github.com/fusabi-lang/fusabi/issues/153 |
| #154 | Add standard library modules for common plugin operations | Filed | MEDIUM | https://github.com/fusabi-lang/fusabi/issues/154 |

**Impact:** Quality of life improvements. Can work around these with manual implementations.

---

## Raibid-labs/ratatui-testlib Issues

**Repository:** https://github.com/raibid-labs/ratatui-testlib

### High Priority

| Issue # | Title | Status | Priority | Link |
|---------|-------|--------|----------|------|
| #9 | Add Bevy ECS integration for testing Bevy+Ratatui applications | Filed | HIGH | https://github.com/raibid-labs/ratatui-testlib/issues/9 |
| #10 | Support headless testing without display server | Filed | HIGH | https://github.com/raibid-labs/ratatui-testlib/issues/10 |

**Impact:** These two issues are critical for Scarab's testing infrastructure. Issue #9 enables frontend testing entirely, and #10 is required for CI/CD.

### Medium Priority

| Issue # | Title | Status | Priority | Link |
|---------|-------|--------|----------|------|
| #11 | Add assertions for UI component positioning and layout | Filed | MEDIUM | https://github.com/raibid-labs/ratatui-testlib/issues/11 |
| #12 | Snapshot testing for Bevy ECS component state | Filed | MEDIUM | https://github.com/raibid-labs/ratatui-testlib/issues/12 |

**Impact:** Quality of life for complex UI testing. Can work around with manual assertions.

### Low Priority

| Issue # | Title | Status | Priority | Link |
|---------|-------|--------|----------|------|
| #13 | Add performance profiling and benchmarking utilities | Filed | LOW | https://github.com/raibid-labs/ratatui-testlib/issues/13 |

**Impact:** Nice to have. Can use Criterion separately for benchmarking.

---

## Summary Statistics

**Total Issues Filed:** 12
- Fusabi: 7 issues
- Ratatui-testlib: 5 issues

**By Priority:**
- CRITICAL: 2 (Fusabi #148, #149)
- HIGH: 4 (Fusabi #150, #151 | Ratatui-testlib #9, #10)
- MEDIUM: 5 (Fusabi #152, #153, #154 | Ratatui-testlib #11, #12)
- LOW: 1 (Ratatui-testlib #13)

---

## Status Definitions

| Status | Description |
|--------|-------------|
| Filed | Issue has been filed, awaiting maintainer response |
| Acknowledged | Maintainer has acknowledged the issue |
| Planned | Maintainer has added to roadmap |
| In Progress | Work has started on the issue |
| Implemented | Feature has been implemented |
| Released | Feature is available in a release |
| Rejected | Issue was rejected by maintainer |
| Closed | Issue was closed (any reason) |

---

## Maintainer Responses

### Fusabi Issues

**Issue #148 - Event System:**
- Status: Filed
- Response: (awaiting)
- Notes:

**Issue #149 - Terminal Queries:**
- Status: Filed
- Response: (awaiting)
- Notes:

**Issue #150 - Programmatic Control:**
- Status: Filed
- Response: (awaiting)
- Notes:

**Issue #151 - UI Formatting:**
- Status: Filed
- Response: (awaiting)
- Notes:

**Issue #152 - Config Schema:**
- Status: Filed
- Response: (awaiting)
- Notes:

**Issue #153 - Command Palette:**
- Status: Filed
- Response: (awaiting)
- Notes:

**Issue #154 - Stdlib Enhancements:**
- Status: Filed
- Response: (awaiting)
- Notes:

### Ratatui-testlib Issues

**Issue #9 - Bevy Integration:**
- Status: Filed
- Response: (awaiting)
- Notes:

**Issue #10 - Headless Mode:**
- Status: Filed
- Response: (awaiting)
- Notes:

**Issue #11 - Position Assertions:**
- Status: Filed
- Response: (awaiting)
- Notes:

**Issue #12 - Component Snapshots:**
- Status: Filed
- Response: (awaiting)
- Notes:

**Issue #13 - Performance Benchmarking:**
- Status: Filed
- Response: (awaiting)
- Notes:

---

## Next Steps

### Immediate Actions
1. Monitor issue responses for next 7 days
2. Respond to any maintainer questions promptly
3. Offer to contribute PRs for accepted features

### Contribution Offers
- Fusabi: Ready to contribute event system, terminal queries, and example plugins
- Ratatui-testlib: Ready to contribute Bevy integration and headless mode support

### Fallback Plans
If critical issues are rejected or delayed:
- **Fusabi Events (#148, #149):** Could fork temporarily or use FFI to Lua
- **Ratatui-testlib Bevy (#9, #10):** Build separate harness in Scarab repo

---

## Related Documents

- [06-RATATUI-TESTLIB-ISSUES.md](/home/beengud/raibid-labs/scarab/docs/audits/claude-2025-12-01/06-RATATUI-TESTLIB-ISSUES.md) - Original feature requests for ratatui-testlib
- [07-FUSABI-ISSUES.md](/home/beengud/raibid-labs/scarab/docs/audits/claude-2025-12-01/07-FUSABI-ISSUES.md) - Original feature requests for Fusabi

---

## Update Log

**2025-12-01:**
- Filed all 12 issues across both repositories
- Created tracking document
- All issues include code examples and use cases from Scarab

---

**Document:** 10-UPSTREAM-ISSUES-TRACKER.md
**Maintained by:** Scarab Team
**Update Frequency:** Weekly (or when maintainers respond)
