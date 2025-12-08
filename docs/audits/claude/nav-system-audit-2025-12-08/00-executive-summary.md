# Navigation System Technical Audit - Executive Summary

**Date:** 2025-12-08
**Auditor:** Claude Opus 4.5
**Scope:** Navigation, Input Handling, and UI Plugin Architecture
**Severity Rating:** HIGH - Critical architectural issues requiring immediate attention

---

## Critical Findings

### 1. CRITICAL: Orphaned Navigation System
The `route_nav_input()` system in `input/nav_input.rs` is **exported but never registered as a Bevy system**. This means:
- Navigation mode switching (Hints, Copy, Search) is non-functional
- `NavInputRouter` and `ModeStack` resources are created but never used
- 900+ lines of navigation code are effectively dead code

### 2. CRITICAL: Duplicate Type Definitions
Two competing `NavMode` and `NavAction` enums exist with incompatible definitions:
- `input/nav_input.rs` - Input-focused (6 modes, 20+ actions)
- `navigation/mod.rs` - ECS-focused (4 modes, 8 actions)

This creates namespace collisions and semantic confusion.

### 3. HIGH: AdvancedUIPlugin Was a Stub (Fixed)
The `lib.rs` was exporting a stub `AdvancedUIPlugin` from `ui_stub.rs` that did nothing. This has been fixed in PR #144 but indicates:
- Status bar, command palette, link hints were all disabled
- No tests caught this regression

### 4. MEDIUM: No NavAction Handler
`NavActionEvent` is defined and emitted but **no system processes it**. Actions like `Open(url)`, `NextPane`, `JumpPrompt` are never executed.

---

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────────┐
│                     INPUT LAYER (Fragmented)                     │
├─────────────────────────────────────────────────────────────────┤
│ ipc.rs                 │ input/nav_input.rs │ ratatui_bridge/   │
│ - handle_keyboard_input│ - route_nav_input  │ - handle_keyboard │
│ - handle_character_inp │   (NOT REGISTERED) │ - handle_mouse    │
│ ✓ REGISTERED           │ ✗ ORPHANED         │ ✓ REGISTERED      │
└─────────────────────────────────────────────────────────────────┘
                              ↓
┌─────────────────────────────────────────────────────────────────┐
│                   NAVIGATION LAYER (Incomplete)                  │
├─────────────────────────────────────────────────────────────────┤
│ navigation/mod.rs      │ navigation/focusable.rs                │
│ - NavStateRegistry     │ - FocusableDetector                    │
│ - Pane lifecycle       │ - URL/path/email detection             │
│ ✓ REGISTERED           │ ✓ REGISTERED                           │
│                        │                                         │
│ NavActionEvent         │ EnterHintModeEvent                      │
│ ✗ NO HANDLER           │ ✓ HAS HANDLERS                          │
└─────────────────────────────────────────────────────────────────┘
                              ↓
┌─────────────────────────────────────────────────────────────────┐
│                      UI LAYER (12 Plugins)                       │
├─────────────────────────────────────────────────────────────────┤
│ ✓ StatusBarPlugin      │ ✓ CommandPalettePlugin                 │
│ ✓ LinkHintsPlugin      │ ✓ LeaderKeyPlugin                      │
│ ✓ SearchOverlayPlugin  │ ✓ AnimationsPlugin                     │
│ ✓ DockPlugin           │ ✗ FusabiTuiPlugin (stub)               │
└─────────────────────────────────────────────────────────────────┘
```

---

## Impact Assessment

| Issue | Impact | Users Affected | Fix Complexity |
|-------|--------|----------------|----------------|
| route_nav_input not registered | Hint/Copy/Search modes broken | All | Low |
| Duplicate NavMode/NavAction | Maintenance nightmare | Developers | Medium |
| No NavAction handler | Actions don't execute | All | Low |
| ui_stub.rs exists | Confusion, future regressions | Developers | Low |
| Input ordering undefined | Race conditions possible | Edge cases | Medium |

---

## Recommended Actions

### Immediate (P0)
1. **Delete or consolidate** `input/nav_input.rs` types with `navigation/mod.rs`
2. **Implement NavAction handler** system in NavigationPlugin
3. **Remove ui_stub.rs** entirely

### Short-term (P1)
4. **Add system ordering** constraints for input handlers
5. **Add integration tests** for navigation flow
6. **Document expected architecture** for future contributors

### Medium-term (P2)
7. **Unify input handling** into single coherent system
8. **Implement FusabiTuiPlugin** (currently stub)
9. **Complete StatusBar styling** (colors disabled)

---

## Files Requiring Changes

| File | Action | Priority |
|------|--------|----------|
| `input/nav_input.rs` | Delete or refactor | P0 |
| `navigation/mod.rs` | Add NavAction handler | P0 |
| `main.rs` | Remove orphan resource insertions | P0 |
| `ui_stub.rs` | Delete entirely | P1 |
| `lib.rs` | Clean up exports | P1 |

---

## Detailed Reports

- [01-navigation-architecture.md](./01-navigation-architecture.md) - Full navigation system analysis
- [02-input-handling.md](./02-input-handling.md) - Input flow and system registration
- [03-ui-plugins.md](./03-ui-plugins.md) - UI plugin dependency graph and status
- [04-recommendations.md](./04-recommendations.md) - Detailed fix recommendations

---

## Metrics

- **Total Lines Audited:** ~15,000
- **Dead Code Identified:** ~1,200 lines (nav_input.rs)
- **Missing Systems:** 2 (route_nav_input, NavAction handler)
- **Stub/Placeholder Code:** 3 files
- **Type Conflicts:** 2 (NavMode, NavAction)
