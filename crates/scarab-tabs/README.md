# scarab-tabs

Tab management plugin for Scarab Terminal.

## Features

- **Tab Creation**: Create new tabs with auto-generated or custom titles
- **Tab Switching**: Navigate between tabs using keyboard shortcuts or commands
- **Tab Closing**: Close individual tabs (prevents closing the last tab)
- **Tab Reordering**: Move tabs to different positions
- **Tab Listing**: View all open tabs with active indicator
- **Persistence**: Tab state can be saved/restored (future integration with scarab-session)

## Keybindings

| Keybinding | Action |
|------------|--------|
| `Ctrl+Shift+T` | Create new tab |
| `Ctrl+Shift+W` | Close current tab |
| `Ctrl+Tab` | Switch to next tab |
| `Ctrl+Shift+Tab` | Switch to previous tab |
| `Ctrl+1-9` | Switch to tab by number (1-9) |

## Command Palette Integration

The plugin provides the following commands for the Command Palette (Ctrl+P):

- **New Tab**: Create a new tab
- **Close Tab**: Close current tab
- **Next Tab**: Switch to next tab
- **Previous Tab**: Switch to previous tab
- **List Tabs**: Show all open tabs
- **Rename Tab**: Rename current tab (coming soon)

## Architecture

### Data Structures

- **Tab**: Represents a single tab with metadata (ID, title, session, working directory, creation time)
- **PluginState**: Manages the tab collection, active tab index, and ID generation

### State Management

- Tab state is maintained in-memory by the plugin
- Each tab has a unique u64 ID
- The active tab is tracked by index
- Tab operations are atomic and thread-safe using `parking_lot::Mutex`

### Integration Points

1. **scarab-plugin-api**: Uses standard Plugin trait
2. **scarab-protocol**: Will use Tab-related ControlMessages for daemon communication
3. **scarab-session**: Future integration for tab persistence
4. **scarab-panes**: Tabs can contain multiple panes

## Usage

### In Daemon

```rust
use scarab_tabs::TabsPlugin;

// Register the plugin
plugin_manager
    .register_plugin(Box::new(TabsPlugin::new()))
    .await?;
```

### Configuration

Add to your `~/.config/scarab/config.toml`:

```toml
[ui]
show_tabs = true
tab_position = "top"  # or "bottom", "left", "right"

[plugins]
enabled = ["scarab-tabs"]
```

## Future Enhancements

- [ ] Tab persistence (save/restore on restart)
- [ ] Tab reordering via drag-and-drop
- [ ] Custom tab titles with template variables (e.g., `{cwd}`, `{command}`)
- [ ] Tab grouping/workspaces
- [ ] Tab icons based on running process
- [ ] Tab color coding
- [ ] Split tab state to separate client/daemon components
- [ ] Integration with scarab-panes for full workspace management

## Testing

Run tests with:

```bash
cargo test -p scarab-tabs
```

Current test coverage:
- Tab creation
- Tab closing
- Tab switching (next/prev/by-index)
- Tab reordering
- Last tab protection

## License

MIT OR Apache-2.0
