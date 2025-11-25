# scarab-panes

Pane/split management plugin for Scarab Terminal.

## Features

- **Pane Splitting**: Split panes horizontally or vertically
- **Pane Navigation**: Move focus between panes using keyboard shortcuts
- **Pane Closing**: Close individual panes (prevents closing the last pane)
- **Pane Resizing**: Resize panes dynamically (coming soon)
- **Layout Management**: Automatic layout recalculation on terminal resize
- **PTY Sessions**: Each pane maintains its own PTY session (future)

## Keybindings

| Keybinding | Action |
|------------|--------|
| `Ctrl+Shift+-` | Split pane horizontally |
| `Ctrl+Shift+\|` | Split pane vertically |
| `Ctrl+Shift+W` | Close current pane |
| `Ctrl+Shift+Up` | Focus pane above |
| `Ctrl+Shift+Down` | Focus pane below |
| `Ctrl+Shift+Left` | Focus pane to the left |
| `Ctrl+Shift+Right` | Focus pane to the right |
| `Ctrl+Shift+Alt+Arrows` | Resize pane (coming soon) |

## Command Palette Integration

The plugin provides the following commands:

- **Split Pane Horizontally**: Split current pane horizontally
- **Split Pane Vertically**: Split current pane vertically
- **Close Pane**: Close current pane
- **Navigate Up/Down/Left/Right**: Focus adjacent panes
- **Zoom Pane**: Toggle pane zoom (fullscreen) (coming soon)

## Architecture

### Data Structures

- **Pane**: Represents a single pane with layout info and PTY handle
- **PaneLayout**: Position and size information (x, y, width, height)
- **SplitDirection**: Horizontal or Vertical split orientation
- **PluginState**: Manages the pane collection, active pane, and layout

### Layout Algorithm

When splitting a pane:
1. Calculate new dimensions (50/50 split)
2. Update existing pane layout
3. Create new pane with calculated dimensions
4. Mark new pane as focused

When closing a pane:
1. Remove pane from collection
2. Recalculate layout to distribute space
3. Focus another pane if current was active

### Integration Points

1. **portable-pty**: Each pane owns a PTY master/slave pair (future)
2. **scarab-protocol**: Uses Pane-related ControlMessages for client-daemon communication
3. **scarab-tabs**: Tabs can contain multiple panes
4. **scarab-daemon**: PTY process management per pane

## Usage

### In Daemon

```rust
use scarab_panes::PanesPlugin;

// Register the plugin with terminal size
let plugin = PanesPlugin::with_size(cols, rows);
plugin_manager
    .register_plugin(Box::new(plugin))
    .await?;
```

### Configuration

Add to your `~/.config/scarab/config.toml`:

```toml
[panes]
default_split = "vertical"  # or "horizontal"
border_style = "rounded"    # or "square", "none"
border_color = "#4A90E2"

[plugins]
enabled = ["scarab-panes"]
```

## Layout Examples

### Horizontal Split
```
┌────────────────┐
│                │
│   Pane 1       │
│                │
├────────────────┤
│                │
│   Pane 2       │
│                │
└────────────────┘
```

### Vertical Split
```
┌────────┬───────┐
│        │       │
│ Pane 1 │ Pane 2│
│        │       │
└────────┴───────┘
```

### Complex Layout
```
┌────────┬───────┐
│        │       │
│ Pane 1 │ Pane 2│
│        ├───────┤
│        │ Pane 3│
└────────┴───────┘
```

## Future Enhancements

- [ ] PTY session per pane (currently placeholder)
- [ ] Advanced layout algorithms (tree-based splits)
- [ ] Pane resizing with mouse/keyboard
- [ ] Pane zooming (maximize/restore)
- [ ] Pane swapping
- [ ] Saved pane layouts
- [ ] Pane border rendering in client
- [ ] Synchronized scrolling across panes
- [ ] Broadcast input to multiple panes
- [ ] Pane titles with status information

## Testing

Run tests with:

```bash
cargo test -p scarab-panes
```

Current test coverage:
- Horizontal/vertical splitting
- Pane closing
- Pane focus management
- Layout recalculation on resize
- Last pane protection

## Technical Notes

### PTY Management (Future)

Each pane will maintain:
- PTY master handle (for reading/writing)
- PTY slave handle (for spawning shell)
- VTE parser state (independent terminal state)
- Scrollback buffer (per-pane history)

### Rendering Strategy

The client will receive pane layout updates and:
1. Calculate pane boundaries
2. Render borders between panes
3. Draw content for each pane from shared memory
4. Highlight focused pane with visual indicator

## License

MIT OR Apache-2.0
