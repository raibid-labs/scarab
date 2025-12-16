# Phage Plugin MVP Design Document

**Created**: 2025-12-15
**Status**: Planning
**Author**: Claude Code

---

## Executive Summary

The Phage Plugin MVP will enable users to access Phage context management capabilities directly from the Scarab terminal through:
1. A **Status Bar Tab** representing Phage in the dock/status bar area
2. **Vimium-C style navigation** via scarab-nav for keyboard-first interaction
3. A **Vertical Menu** that grows from the tab when selected
4. The `phage init` command as the first menu option

---

## Architecture Overview

```
+------------------------------------------------------------------+
|                       SCARAB CLIENT (Bevy)                        |
| +--------------------------------------------------------------+ |
| |                        TERMINAL GRID                          | |
| |                                                               | |
| | [When Phage menu active]                                      | |
| |                                                               | |
| |    +---------------------------+                              | |
| |    |    Phage Menu (Overlay)   |                              | |
| |    +---------------------------+                              | |
| |    | > phage init              |                              | |
| |    |   phage status            |                              | |
| |    |   phage load              |                              | |
| |    |   phage context           |                              | |
| |    |   ------------------      |                              | |
| |    |   Chat with AI            |                              | |
| |    |   Fix Last Command        |                              | |
| |    +---------------------------+                              | |
| |                                                               | |
| +--------------------------------------------------------------+ |
| +--------------------------------------------------------------+ |
| |  DOCK: [scarab-tabs] [scarab-panes] [Phage] [other plugins]  | |
| +--------------------------------------------------------------+ |
| +--------------------------------------------------------------+ |
| |  STATUS BAR: [Terminal 1]                      [f] hints  [N]| |
| +--------------------------------------------------------------+ |
+------------------------------------------------------------------+
```

---

## Component Breakdown

### 1. phage-plugin-scarab (Existing Crate)

**Location**: `/home/beengud/raibid-labs/phage/crates/phage-plugin-scarab/`

The existing `phage-plugin-scarab` crate already implements:
- `PhagePlugin` struct implementing `Plugin` trait
- Menu items via `get_menu()` returning Chat, Explain Selection, Fix Last Command, Context Info
- `on_remote_command()` handling for menu actions
- `on_post_command()` hook for tracking command history
- Connection to Phage daemon via HTTP API

**Enhancement needed**: Add `phage init` as the first menu option.

### 2. Status Bar Tab Integration

The status bar currently uses `TabState` resource with tabs displayed on the left side. Phage needs to appear in the **Dock** (the plugin bar above the status bar), not the terminal tab bar.

**Key Files**:
- `scarab-client/src/ui/dock.rs` - DockPlugin system
- `scarab-client/src/ui/status_bar.rs` - StatusBarPlugin

The Dock system already:
- Displays plugins with emoji icons and names
- Registers items with scarab-nav via `UpdateLayout` protobuf
- Handles keyboard navigation (Tab, Enter, Alt+1-9)
- Shows plugin menus when activated

### 3. Link Hint Integration (scarab-nav)

The scarab-nav protocol uses:
- Protobuf `UpdateLayout` messages with `InteractiveElement` entries
- Unix socket communication (`/tmp/scarab-nav.sock`)
- Element types: BUTTON, INPUT, LINK, LIST_ITEM, TAB, TEXT

The dock already sends layout updates to scarab-nav:
```rust
let layout = UpdateLayout {
    window_id: "dock".to_string(),
    elements: vec![InteractiveElement {
        id: format!("dock-{}", plugin_name),
        r#type: ElementType::Button as i32,
        // position, size, description...
    }],
};
nav_connection.send_layout(layout);
```

### 4. Vertical Menu Component

The existing `PluginMenuPlugin` at `scarab-client/src/ui/plugin_menu.rs` provides:
- Overlay-based menu rendering
- Keyboard navigation (Up/Down/Enter/Escape)
- Submenu support
- Loading and error states

**Enhancement needed**: Change menu positioning from fixed to grow from dock item location.

### 5. State Management

**PhagePlugin State**:
```rust
pub struct PhagePlugin {
    daemon_url: String,
    client: reqwest::Client,
    metadata: PluginMetadata,
    command_history: CommandHistory,
    current_selection: Arc<RwLock<Option<String>>>,
}
```

**Menu State** (MenuState resource):
- `active: bool` - Whether menu is displayed
- `plugin_name: String` - Which plugin's menu
- `current_items: Vec<MenuItem>` - Menu items
- `selected_index: usize` - Cursor position
- `menu_stack: Vec<Vec<MenuItem>>` - Submenu navigation

---

## UI/UX Wireframes

### State 1: Phage Tab in Dock (Normal)
```
+------------------------------------------------------------------+
|                         TERMINAL GRID                             |
|  $ cargo build                                                    |
|     Compiling scarab v0.3.0                                       |
|                                                                   |
+------------------------------------------------------------------+
+------------------------------------------------------------------+
|  DOCK: [scarab-tabs] [scarab-panes] [Phage] [other]              |
|                                                                   |
+------------------------------------------------------------------+
```

### State 2: Hint Mode Active (Leader key pressed, e.g., 'f')
```
+------------------------------------------------------------------+
|                         TERMINAL GRID                             |
|  $ cargo build                                                    |
|     Compiling scarab v0.3.0                                       |
|                                                                   |
+------------------------------------------------------------------+
+------------------------------------------------------------------+
|  DOCK: [scarab-tabs] [scarab-panes] [Phage] [other]              |
|         [a]            [s]           [d]     [f]                 |
+------------------------------------------------------------------+
```

### State 3: Phage Tab Selected (hint 'd' typed or Enter on focused dock)
```
+------------------------------------------------------------------+
|                         TERMINAL GRID                             |
|                                                                   |
|           +---------------------------+                           |
|           |     Phage                 |                           |
|           +---------------------------+                           |
|           | > phage init         [i]  |                           |
|           |   phage status       [s]  |                           |
|           |   phage load         [l]  |                           |
|           |   phage context      [c]  |                           |
|           |   -------------------     |                           |
|           |   Chat             Ctrl+C |                           |
|           |   Explain Selection       |                           |
|           |   Fix Last Command  [f]   |                           |
|           |   Context Info            |                           |
|           +---------------------------+                           |
|           | Enter: Select  Esc: Close |                           |
|           +---------------------------+                           |
|                                                                   |
+------------------------------------------------------------------+
+------------------------------------------------------------------+
|  DOCK: [scarab-tabs] [scarab-panes] [Phage] [other]              |
|                                     *ACTIVE*                      |
+------------------------------------------------------------------+
```

---

## Integration Points with scarab-nav

1. **Dock Items Registration**: Already implemented in `send_dock_layout_to_nav()`

2. **Menu Items Registration**: Need to add when menu opens:
```rust
fn register_menu_items_with_nav(
    menu_state: &MenuState,
    menu_area: Rect,
    nav_connection: &mut NavConnection,
) {
    let elements: Vec<InteractiveElement> = menu_state.current_items
        .iter()
        .enumerate()
        .map(|(i, item)| InteractiveElement {
            id: format!("menu-item-{}", i),
            x: menu_area.x as u32,
            y: (menu_area.y + i as u16 * ITEM_HEIGHT) as u32,
            width: menu_area.width as u32,
            height: ITEM_HEIGHT as u32,
            r#type: ElementType::ListItem as i32,
            description: item.label.clone(),
            key_hint: String::new(),
        })
        .collect();

    nav_connection.send_layout(UpdateLayout {
        window_id: "plugin-menu".to_string(),
        elements,
    });
}
```

---

## Implementation Phases

### Phase 1: Plugin Skeleton Enhancement (1 day)
1. Add `phage init` to menu items in `phage-plugin-scarab/src/lib.rs`
2. Implement `init_cmd` handler in `on_remote_command`
3. Add workspace detection for smart defaults

### Phase 2: Status Bar Tab Integration (1 day)
1. Verify Phage appears in dock when loaded
2. Style the dock item with Phage branding (Matrix green #00FF00)
3. Test keyboard navigation (Alt+N where N is dock position)

### Phase 3: Link Hint Registration (0.5 days)
1. Verify dock items registered with scarab-nav
2. Test hint activation workflow
3. Ensure proper hint label assignment

### Phase 4: Vertical Menu Animation (1 day)
1. Modify `render_menu_system` to position menu above dock
2. Add smooth open/close transition
3. Register menu items with scarab-nav for in-menu hints

### Phase 5: Phage Init Command Integration (0.5 days)
1. Wire up menu action to execute `phage init`
2. Add interactive prompts for org/project if needed
3. Show success/failure notification

---

## Infrastructure Gaps Identified

1. **Menu Positioning**: Current menu uses fixed position. Need to calculate position based on triggering dock item.

2. **Menu Item Hints**: Menu items are not registered with scarab-nav for hint-based selection within the menu.

3. **Phage Daemon Discovery**: The plugin assumes `localhost:15702`. Need configuration or service discovery.

4. **Animation System**: No animation for menu open/close. Could add via Bevy tweening.

---

## Critical Files for Implementation

| File | Purpose |
|------|---------|
| `phage/crates/phage-plugin-scarab/src/lib.rs` | Core plugin - add `phage init` menu item |
| `scarab/crates/scarab-client/src/ui/plugin_menu.rs` | Menu renderer - update positioning |
| `scarab/crates/scarab-client/src/ui/dock.rs` | Dock system - hint integration reference |
| `scarab/crates/scarab-plugin-api/src/menu.rs` | Menu API definitions |
| `phage/crates/phage-cli/src/commands/init_cmd.rs` | Phage init command reference |

---

## Success Criteria

1. User can press hint key (e.g., 'f') and see hints on dock items
2. Typing Phage's hint opens the Phage menu
3. Menu appears above the dock, growing vertically
4. `phage init` is the first menu option
5. Selecting `phage init` creates `.phage/` directory structure
6. All interactions work with keyboard only (no mouse required)

---

## Related Issues

- #188: Remove scarab-nav duplication (prerequisite)
- #193: Phage plugin skeleton (this work)
- #194: Status bar tab integration
- #195: Link hint registration for menu items
- #196: Vertical menu positioning
- #197: Phage init command wrapper
