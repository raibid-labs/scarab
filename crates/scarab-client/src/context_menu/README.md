# Context Menu System

Complete implementation of GitHub Issue #31 - Context menu system for Scarab terminal emulator.

## Overview

The context menu system provides right-click functionality with intelligent context detection, keyboard navigation, and extensible action dispatch. It leverages the existing Ratatui bridge for rendering and integrates seamlessly with Scarab's plugin system.

## Architecture

### Components

1. **mod.rs** - Core module with state management and event systems
   - `ContextMenuState` - Resource tracking current menu
   - `ContextMenuSurface` - Marker component for Bevy entity
   - `ShowContextMenuEvent` - Triggered on right-click
   - `ContextMenuItemSelected` - Fired when item is chosen
   - Systems for detection, positioning, input handling, and surface management

2. **overlay.rs** - Ratatui rendering implementation
   - Renders menus using Ratatui List and Block widgets
   - Highlights selected items with blue background
   - Displays shortcuts right-aligned in gray
   - Shows disabled items in dark gray
   - Renders separator lines

3. **actions.rs** - Action dispatch and execution
   - `ContextMenuAction` enum for all possible actions
   - Conversion from menu item IDs to actions
   - Action handlers for:
     - Copy/Paste (via arboard)
     - Open URL (via open crate)
     - Open File
     - Split panes
     - Search activation
     - Custom plugin actions

4. **plugin_items.rs** - Plugin integration (stub for future)
   - Framework for plugins to contribute menu items
   - Context-aware menu item providers

## Features Implemented

### Right-Click Detection
- Converts screen coordinates to terminal grid coordinates
- Emits `ShowContextMenuEvent` with cursor position
- Ready for URL/file path detection (TODO)

### Context-Aware Menus
- **Standard Menu**: Copy, Paste, Select All, Search, Split operations
- **URL Menu**: Open URL, Copy URL + standard operations
- **File Menu**: Open File, Copy Path + standard operations
- All menus automatically disable irrelevant items (e.g., Copy when no selection)

### Smart Positioning
- Edge detection prevents menus from going off-screen
- Automatic repositioning when near window edges
- Respects terminal grid boundaries

### Keyboard Navigation
- Up/Down arrows: Navigate between items
- Enter: Select highlighted item
- Escape: Close menu
- Automatically skips separator lines

### Mouse Integration
- Hover to highlight items
- Click to select items
- Smooth transition between keyboard and mouse

### Visual Design
- Bordered list with clean styling
- Selection indicator (">") for current item
- Right-aligned shortcuts in brackets
- Separator lines for logical grouping
- Disabled items shown in dark gray

## Usage

### Adding to Your Bevy App

```rust
use scarab_client::ContextMenuPlugin;

app.add_plugins(ContextMenuPlugin);
```

### Triggering Menu Programmatically

```rust
use scarab_client::{ShowContextMenuEvent, Position};

events.send(ShowContextMenuEvent {
    position: Position::new(50, 30),
    url: Some("https://example.com".to_string()),
    file_path: None,
    has_selection: false,
});
```

### Handling Action Events

```rust
use scarab_client::{DispatchContextMenuAction, ContextMenuAction};

fn handle_actions(mut events: EventReader<DispatchContextMenuAction>) {
    for event in events.read() {
        match &event.action {
            ContextMenuAction::Copy => { /* implement copy */ }
            ContextMenuAction::OpenUrl(url) => { /* open browser */ }
            _ => {}
        }
    }
}
```

## Integration Points

### With Ratatui Bridge
- Uses `RatatuiSurface` for overlay positioning
- Leverages `SurfaceBuffers` for rendering
- Integrates with `SurfaceFocus` for input routing
- Receives `SurfaceInputEvent` for keyboard/mouse

### With Mouse System
- Uses `scarab-mouse` context menu data structures
- `ContextMenu::standard/url_menu/file_menu` builders
- `MenuItem` with enabled state and shortcuts
- `Position` for grid coordinates

### With Clipboard (Future)
- Copy/Paste actions ready for integration
- Uses `arboard` for direct clipboard access
- TODO: Connect with `scarab-clipboard` plugin

### With Plugin System (Future)
- `plugin_items.rs` provides extension points
- Plugins can register menu item providers
- Context-specific menu items per plugin
- Action IDs prefixed with "plugin." route to plugins

## Dependencies Added

- `open = "5.0"` - For opening URLs and files in default applications

## Testing

### Unit Tests
- State management (show/hide)
- Menu positioning adjustment
- Action parsing from IDs
- Menu item rendering
- Navigation with separators

### Integration Tests
Located in `tests/context_menu_tests.rs`:
- Plugin initialization
- Event handling
- Context detection
- Edge adjustment
- Menu navigation
- Item selection

### Example Demo
`examples/context_menu_demo.rs`:
- Interactive demonstration
- Visual controls guide
- Action logging to console

Run with:
```bash
cargo run -p scarab-client --example context_menu_demo
```

## File Structure

```
crates/scarab-client/src/context_menu/
├── mod.rs              # Core module, state, events, systems
├── overlay.rs          # Ratatui rendering implementation
├── actions.rs          # Action dispatch and handlers
├── plugin_items.rs     # Plugin integration (stub)
└── README.md           # This file

tests/
└── context_menu_tests.rs  # Integration tests

examples/
└── context_menu_demo.rs   # Interactive demo
```

## Future Enhancements

### Short-term
1. URL detection at cursor position
2. File path detection using regex
3. Selection state integration
4. Data passing to action handlers (URLs, paths)

### Medium-term
1. Full clipboard plugin integration
2. Plugin menu item registration
3. Custom menu items via config
4. Configurable keybindings

### Long-term
1. Menu history/favorites
2. Fuzzy search in menu
3. Multi-level submenus
4. Custom menu themes
5. Animation effects

## Implementation Notes

### Why Ratatui Bridge?
- Consistent with command palette implementation
- Rich widget ecosystem (List, Block, borders)
- Terminal-native coordinate system
- No need to reinvent UI components

### Design Decisions
1. **Separate mod.rs from overlay.rs**: Keeps business logic separate from rendering
2. **Action enum**: Type-safe action dispatch vs. string IDs
3. **Smart positioning**: Better UX than clipped menus
4. **Keyboard + Mouse**: Accessible to all interaction styles
5. **Plugin-ready**: Extension points designed upfront

### Performance Considerations
- Menus only render when visible (dirty flag)
- Surface reused across invocations
- Event-driven updates (no polling)
- Efficient coordinate conversions
- Minimal allocations in hot path

## Credits

Implementation of GitHub Issue #31 for Scarab terminal emulator.
Follows established patterns from the command palette and Ratatui bridge systems.
