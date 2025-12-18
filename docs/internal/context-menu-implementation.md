# Context Menu UI Implementation Summary

## Overview
This document describes the implementation of context menu UI rendering and interaction in `scarab-mouse/src/bevy_plugin.rs`.

## Changes Made

### 1. Component Definitions (Lines 74-81)
Added marker components for context menu entities:
```rust
/// Component for context menu UI entities
#[derive(Component)]
struct ContextMenuEntity;

/// Component for context menu items
#[derive(Component)]
struct ContextMenuItem {
    index: usize,
}
```

### 2. MousePluginState Resource (Lines 67-75)
Added fields to track active context menu:
```rust
struct MousePluginState {
    shared_state: Arc<Mutex<MouseState>>,
    click_detector: ClickDetector,
    drag_start: Option<Position>,
    is_dragging: bool,
    active_menu: Option<ContextMenu>,      // NEW: Store active menu
    menu_mouse_pos: Option<Vec2>,          // NEW: Mouse position for menu
}
```

### 3. Plugin Registration (Lines 46-65)
Added `render_context_menu` system to the update chain:
```rust
.add_systems(
    Update,
    (
        handle_mouse_input,
        handle_scroll,
        update_selection_rendering,
        render_context_menu,          // NEW SYSTEM
        handle_context_menu_input,
    ).chain(),
)
```

### 4. Right Click Handler (Line 272-283)
Modified to store menu in plugin state:
```rust
// Change from:
let _menu = if let Some(item) = find_clickable_at(&state, pos) {
    // ...
};

// To:
let menu = if let Some(item) = find_clickable_at(&state, pos) {
    // ...
};
plugin_state.active_menu = Some(menu);  // Store menu for rendering
```

## TODO Line 272: Context Menu Rendering Implementation

### System Signature
```rust
fn render_context_menu(
    plugin_state: Res<MousePluginState>,
    mut commands: Commands,
    menu_query: Query<Entity, With<ContextMenuEntity>>,
    windows: Query<&Window>,
)
```

### Implementation Details

**Step 1: Clear existing menus**
```rust
for entity in &menu_query {
    commands.entity(entity).despawn_recursive();
}
```

**Step 2: Check if menu should be rendered**
```rust
let Some(menu) = &plugin_state.active_menu else {
    return;
};
```

**Step 3: Get window and cursor position**
```rust
let Ok(window) = windows.get_single() else { return };
let Some(cursor_pos) = window.cursor_position() else { return };
```

**Step 4: Calculate menu dimensions**
- Menu width: 280px
- Item height: 40px
- Separator height: 10px
- Clamp to screen bounds to prevent overflow

**Step 5: Spawn menu container with Bevy 0.15 API**
```rust
commands.spawn((
    ContextMenuEntity,
    Node {
        width: Val::Px(menu_width),
        height: Val::Px(menu_height),
        position_type: PositionType::Absolute,
        left: Val::Px(menu_x),
        top: Val::Px(menu_y),
        flex_direction: FlexDirection::Column,
        padding: UiRect::all(Val::Px(4.0)),
        ..default()
    },
    BackgroundColor(Color::srgba(0.2, 0.2, 0.2, 0.95)),
    BorderColor(Color::srgba(0.4, 0.4, 0.5, 0.8)),
))
```

**Step 6: Render menu items**
For each item in `menu.items`:
- If `item.separator`: render thin horizontal line
- Else: render interactive button with:
  - Label text on left
  - Shortcut text on right (if present)
  - Visual indication for selected item (highlighted background)
  - Grayed out appearance for disabled items

### Color Scheme
- Container background: `Color::srgba(0.2, 0.2, 0.2, 0.95)`
- Selected item: `Color::srgba(0.35, 0.4, 0.5, 0.9)`
- Normal item: `Color::srgba(0.25, 0.25, 0.25, 0.2)`
- Disabled item: `Color::srgba(0.15, 0.15, 0.15, 0.3)`
- Text (enabled): `Color::WHITE`
- Text (disabled): `Color::srgba(0.5, 0.5, 0.5, 0.7)`
- Shortcut text: `Color::srgba(0.6, 0.6, 0.7, 1.0)`

## TODO Line 468: Context Menu Interaction Implementation

### System Signature
```rust
fn handle_context_menu_input(
    mut plugin_state: ResMut<MousePluginState>,
    keyboard: Res<ButtonInput<KeyCode>>,
    ipc: Option<Res<MouseIpcSender>>,
)
```

### Implementation Details

**Step 1: Check if menu is visible**
```rust
let state = plugin_state.shared_state.lock();
if !state.context_menu_visible {
    return;
}

let Some(menu) = &mut plugin_state.active_menu else {
    return;
};
```

**Step 2: Handle Escape key - close menu**
```rust
if keyboard.just_pressed(KeyCode::Escape) {
    plugin_state.active_menu = None;
    plugin_state.menu_mouse_pos = None;
    drop(state);
    plugin_state.shared_state.lock().context_menu_visible = false;
    log::debug!("Context menu closed");
    return;
}
```

**Step 3: Handle arrow key navigation**
```rust
if keyboard.just_pressed(KeyCode::ArrowDown) {
    menu.select_next();  // Implemented in ContextMenu
}

if keyboard.just_pressed(KeyCode::ArrowUp) {
    menu.select_prev();  // Implemented in ContextMenu
}
```

**Step 4: Handle Enter key - execute action**
```rust
if keyboard.just_pressed(KeyCode::Enter) {
    if let Some(item) = menu.selected_item() {
        if item.enabled {
            let item_id = item.id.clone();
            log::info!("Executing context menu action: {}", item_id);
            execute_menu_action(&item_id, &ipc);

            // Close menu after execution
            plugin_state.active_menu = None;
            plugin_state.menu_mouse_pos = None;
            drop(state);
            plugin_state.shared_state.lock().context_menu_visible = false;
        }
    }
}
```

### Menu Actions Helper Function
```rust
fn execute_menu_action(action_id: &str, ipc: &Option<Res<MouseIpcSender>>) {
    match action_id {
        "copy" => { /* Copy selected text to clipboard */ },
        "paste" => {
            use arboard::Clipboard;
            if let Ok(mut clipboard) = Clipboard::new() {
                if let Ok(text) = clipboard.get_text() {
                    if let Some(ipc) = ipc {
                        ipc.0.send(ControlMessage::Input {
                            data: text.into_bytes(),
                        });
                    }
                }
            }
        },
        "select_all" => { /* Select all text */ },
        "clear_selection" => { /* Clear selection */ },
        "search" => { /* Open search overlay */ },
        "new_tab" => { /* Create new tab */ },
        "split_horizontal" | "split_vertical" => { /* Split panes */ },
        "open_url" | "copy_url" => { /* URL actions */ },
        "open_file" | "copy_path" => { /* File actions */ },
        _ => log::warn!("Unknown action: {}", action_id),
    }
}
```

## Integration Points

### Dependencies
- `bevy::prelude::*` - Bevy 0.15 ECS and UI components
- `crate::context_menu::ContextMenu` - Menu data structure
- `scarab_protocol::ControlMessage` - IPC messages
- `arboard::Clipboard` - Cross-platform clipboard access

### Key APIs Used
- **Bevy 0.15 UI**: `Node`, `BackgroundColor`, `Text::new()`, `TextFont`, `TextColor`
- **Context Menu**: `ContextMenu::standard()`, `ContextMenu::url_menu()`, `ContextMenu::file_menu()`
- **Navigation**: `menu.select_next()`, `menu.select_prev()`, `menu.selected_item()`

## Testing Checklist

- [ ] Right-click displays context menu at cursor position
- [ ] Menu items render correctly with labels and shortcuts
- [ ] Selected item is visually highlighted
- [ ] Disabled items are grayed out and non-interactive
- [ ] Arrow keys navigate through menu items
- [ ] Enter key executes selected action
- [ ] Escape key closes menu
- [ ] Menu closes after action execution
- [ ] Separators render as thin lines
- [ ] Menu stays within screen bounds
- [ ] Different menus for URLs and file paths
- [ ] Clipboard paste action works correctly

## Files Modified

1. `/home/beengud/raibid-labs/scarab/crates/scarab-mouse/src/bevy_plugin.rs`
   - Added `ContextMenuEntity` and `ContextMenuItem` components
   - Updated `MousePluginState` with `active_menu` and `menu_mouse_pos` fields
   - Added `render_context_menu` system
   - Implemented `handle_context_menu_input` system
   - Added `execute_menu_action` helper function
   - Modified `handle_right_click` to store menu in plugin state

## Implementation Status

✅ Component definitions
✅ State management fields
✅ System registration
✅ Menu rendering logic
✅ Keyboard interaction handling
✅ Action execution framework
⚠️  Pending: Full clipboard integration
⚠️  Pending: Search overlay integration
⚠️  Pending: Tab/split pane integration

## Notes

- The implementation follows Bevy 0.15 patterns seen in `scarab-client/src/ui/command_palette.rs` and `plugin_menu.rs`
- Uses lock-free pattern with `Option<ContextMenu>` to avoid holding mutex during rendering
- Menu navigation skips separator items automatically via `ContextMenu::select_next()`/`select_prev()`
- Proper cleanup on menu close (clears both `active_menu` and `context_menu_visible` flag)
- Color scheme matches Scarab's dark theme aesthetic
