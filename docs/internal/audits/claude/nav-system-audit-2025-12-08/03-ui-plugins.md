# UI Plugin System Analysis

## Overview

The Scarab UI system is implemented as a bundle of 12 plugins loaded via `AdvancedUIPlugin`. A critical issue was discovered where `lib.rs` was exporting a **stub** version instead of the real implementation.

---

## Critical Issue: Stub Export (FIXED in PR #144)

### Problem

**File: `lib.rs` line 57 (before fix):**
```rust
pub use ui_stub::AdvancedUIPlugin;  // WRONG - exports stub
```

**File: `ui_stub.rs` lines 10-16:**
```rust
impl Plugin for AdvancedUIPlugin {
    fn build(&self, _app: &mut App) {
        warn!("AdvancedUIPlugin temporarily disabled during Bevy 0.15 migration");
        // Does NOTHING
    }
}
```

### Impact

All 12 UI plugins were disabled:
- Status bar invisible
- Command palette non-functional
- Link hints broken
- Keybindings not processed
- Leader key menus unavailable
- Animations disabled
- Visual selection broken
- Search overlay unavailable
- Dock hidden

### Fix Applied

**File: `lib.rs` line 57 (after fix):**
```rust
pub use ui::AdvancedUIPlugin;  // CORRECT - exports real implementation
```

---

## Plugin Dependency Graph

```
AdvancedUIPlugin (ui/mod.rs)
│
├── LinkHintsPlugin (link_hints.rs)
│   └── Depends on: SharedMemoryReader, TerminalMetrics
│
├── CommandPalettePlugin (command_palette.rs)
│   └── Depends on: IpcChannel, RemoteMessageEvent
│
├── LeaderKeyPlugin (leader_key.rs)
│   └── Depends on: None (self-contained)
│
├── KeybindingsPlugin (keybindings.rs)
│   └── Depends on: None (self-contained)
│
├── AnimationsPlugin (animations.rs)
│   └── Depends on: Time resource
│
├── VisualSelectionPlugin (visual_selection.rs)
│   └── Depends on: SharedMemoryReader, TerminalMetrics
│
├── RemoteUiPlugin (overlays.rs)
│   └── Depends on: RemoteMessageEvent
│
├── PluginMenuPlugin (plugin_menu.rs)
│   └── Depends on: RemoteMessageEvent
│
├── ScrollIndicatorPlugin (scroll_indicator.rs)
│   └── Depends on: ScrollbackState
│
├── ScrollbackSelectionPlugin (scrollback_selection.rs)
│   └── Depends on: SharedMemoryReader, TerminalMetrics
│
├── SearchOverlayPlugin (search_overlay.rs)
│   └── Depends on: ScrollbackState
│
├── DockPlugin (dock.rs)
│   └── Depends on: NavConnection (socket)
│
└── StatusBarPlugin (status_bar.rs)
    └── Depends on: DaemonMessageReceiver
```

---

## Individual Plugin Status

### 1. LinkHintsPlugin ✓ Complete

**File:** `ui/link_hints.rs`

**Resources:**
- `LinkDetector` - Regex patterns for URLs, files, emails
- `LinkHintsState` - Current hint display state

**Events:**
- `LinkActivatedEvent` - Hint selected

**Systems:**
- `detect_links_system` - Scan terminal on Ctrl+K
- `show_hints_system` - Render hint labels
- `handle_hint_input_system` - Process hint selection
- `activate_link_system` - Open URLs/files

**Status:** Fully implemented

---

### 2. CommandPalettePlugin ⚠️ Partial

**File:** `ui/command_palette.rs`

**Resources:**
- `CommandRegistry` - Available commands
- `CommandPaletteState` - Open/search state

**Events:**
- `CommandExecutedEvent`
- `ShowRemoteModalEvent`

**Systems:**
- `toggle_palette_system` - Ctrl+P to open
- `handle_palette_input_system` - Navigation
- `render_palette_system` - UI rendering
- `execute_command_system` - Run commands
- `handle_remote_modal_system` - Daemon commands

**Stub Commands (lines 461-484):**
```rust
"reload_config" => {
    // TODO: Implement configuration reload
    println!("Reload config not yet implemented");
}
"help" => {
    // TODO: Show help documentation
    println!("Help not yet implemented");
}
```

---

### 3. LeaderKeyPlugin ✓ Complete

**File:** `ui/leader_key.rs`

**Resources:**
- `LeaderKeyState`
- `LeaderKeyMenus`

**Systems (chained):**
- `handle_leader_key_system`
- `handle_menu_navigation_system`
- `render_menu_system`
- `timeout_check_system`

**Status:** Fully implemented (Spacemacs-style menus)

---

### 4. KeybindingsPlugin ✓ Complete

**File:** `ui/keybindings.rs`

**Resources:**
- `KeyBindingConfig`

**Events:**
- `KeyBindingTriggeredEvent`

**Systems:**
- `handle_keybindings_system`

**Default Bindings:**
- Copy/Paste/Cut: Ctrl+C/V/X
- Undo/Redo: Ctrl+Z/Y
- Search: Ctrl+F
- Command Palette: Ctrl+P
- Link Hints: Ctrl+K

**Status:** Fully implemented

---

### 5. AnimationsPlugin ⚠️ Partial

**File:** `ui/animations.rs`

**Components:**
- `AnimationState` (FadeIn, FadeOut, SlideIn, SlideOut)
- `FadeAnimation`
- `SlideAnimation`

**Systems:**
- `update_fade_animations_system`
- `update_slide_animations_system`
- `cleanup_finished_animations_system`

**Placeholder (line 173):**
```rust
// leaving this as a placeholder for future easing functions
```

**Status:** Core animations work, easing incomplete

---

### 6. VisualSelectionPlugin ✓ Complete

**File:** `ui/visual_selection.rs`

**Components:**
- `SelectionRegion`
- `SelectionMode` (Character, Line, Block)

**Resources:**
- `SelectionState`

**Events:**
- `SelectionChangedEvent`
- `SelectionCopiedEvent`

**Status:** Fully implemented

---

### 7. RemoteUiPlugin ✓ Complete

**File:** `ui/overlays.rs`

**Components:**
- `RemoteOverlay`
- `NotificationUI`
- `PluginLogDisplay`

**Events:**
- `HideModalEvent`

**Systems:**
- `handle_remote_messages`
- `update_notifications`
- `handle_hide_modal`

**TODO (line 130):**
```rust
// TODO: Could also display in an on-screen log panel
```

**Status:** Working, logging UI optional

---

### 8. PluginMenuPlugin ⚠️ Partial

**File:** `ui/plugin_menu.rs`

**Resources:**
- `MenuState` (with submenu stack)

**Events:**
- `ShowPluginMenuEvent`
- `MenuActionEvent`

**Commented Code (lines 37-46):**
```rust
// fn handle_menu_request_system(...) {
//     // Currently disabled pending menu request protocol
// }
```

**Status:** Menus work, dynamic loading incomplete

---

### 9. ScrollIndicatorPlugin ✓ Complete

**File:** `ui/scroll_indicator.rs`

**Components:**
- `ScrollIndicator`

**Resources:**
- `ScrollIndicatorConfig`

**Systems:**
- `spawn_scroll_indicator`
- `despawn_scroll_indicator`
- `update_scroll_indicator`

**Status:** Fully implemented

---

### 10. ScrollbackSelectionPlugin ✓ Complete

**File:** `ui/scrollback_selection.rs`

**Resources:**
- `ScrollbackSelectionState`

**Events:**
- `ScrollbackMouseSelection`

**Systems:**
- `handle_mouse_selection`

**Status:** Fully implemented

---

### 11. SearchOverlayPlugin ✓ Complete

**File:** `ui/search_overlay.rs`

**Components:**
- `SearchOverlay`
- `SearchInputBox`
- `SearchResultsText`

**Resources:**
- `SearchOverlayConfig`

**Systems:**
- Spawn/despawn on Ctrl+F
- Update search results

**Status:** Fully implemented

---

### 12. DockPlugin ✓ Complete

**File:** `ui/dock.rs`

**Components:**
- `DockContainer`
- `DockItem`
- `DockItemBounds`

**Resources:**
- `DockState`
- `DockConfig`
- `NavConnection`

**Status:** Fully implemented with nav socket integration

---

### 13. StatusBarPlugin ⚠️ Partial

**File:** `ui/status_bar.rs`

**Resources:**
- `StatusBarState`
- `StatusUpdateTimer`
- `DaemonMessageReceiver`

**Components:**
- `StatusBarContainer`
- `StatusBarLeft`
- `StatusBarRight`

**Disabled Styling (lines 308-342):**
```rust
// Colors, bold, italic, underline, strikethrough
// Currently NOT applied to Text nodes
// Pending proper Bevy 0.15 text styling API
```

**Status:** Basic display works, rich styling disabled

---

### 14. FusabiTuiPlugin ✗ Stub

**File:** `ui/fusabi_widgets.rs`

**Status:** Complete stub, does nothing

**Code (lines 78-82):**
```rust
impl Plugin for FusabiTuiPlugin {
    fn build(&self, _app: &mut App) {
        // integration pending
    }
}
```

---

## Stub/Placeholder Summary

| File | Issue | Lines | Priority |
|------|-------|-------|----------|
| `ui_stub.rs` | Entire file is deprecated stub | All | P0 - Delete |
| `fusabi_widgets.rs` | No-op plugin | 78-82 | P1 - Implement or remove |
| `command_palette.rs` | 2 unimplemented commands | 461-484 | P2 - Implement |
| `status_bar.rs` | Styling disabled | 308-342 | P2 - Enable |
| `plugin_menu.rs` | Menu request handler commented | 37-46 | P2 - Restore |
| `overlays.rs` | Log panel TODO | 130 | P3 - Optional |
| `animations.rs` | Easing placeholder | 173 | P3 - Optional |

---

## UIConfig Resource

**File:** `ui/mod.rs` lines 67-91

```rust
#[derive(Resource, Clone)]
pub struct UIConfig {
    pub link_hints_enabled: bool,       // true
    pub command_palette_enabled: bool,  // true
    pub leader_key_enabled: bool,       // true
    pub animations_enabled: bool,       // true
    pub leader_key_timeout_ms: u64,     // 1000
    pub fuzzy_search_threshold: f64,    // 0.3
    pub dock_enabled: bool,             // true
}
```

All features enabled by default.

---

## Recommendations

### P0 - Immediate

1. **Delete `ui_stub.rs`** - No longer needed
2. **Verify all plugins load** - Add startup logging

### P1 - Short Term

3. **Implement FusabiTuiPlugin** or remove from bundle
4. **Enable StatusBar styling** - Complete Bevy 0.15 migration

### P2 - Medium Term

5. **Implement reload_config command**
6. **Implement help command**
7. **Restore plugin menu request handler**

### P3 - Optional

8. **Add on-screen log panel** for debugging
9. **Complete animation easing functions**
