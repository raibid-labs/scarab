# Command Palette

Quick access to all Scarab commands via the command palette.

## Overview

The command palette provides fuzzy-searchable access to all Scarab commands, plugin actions, and settings.

## Opening the Command Palette

Press **Ctrl+Shift+P** to open the command palette.

## Usage

1. Press **Ctrl+Shift+P**
2. Type to filter commands (fuzzy matching)
3. Use **Up/Down** arrows to navigate
4. Press **Enter** to execute
5. Press **Escape** to close

## Command Categories

### Terminal Commands
- `terminal: new tab` - Create a new tab
- `terminal: close tab` - Close current tab
- `terminal: split horizontal` - Split pane horizontally
- `terminal: split vertical` - Split pane vertically
- `terminal: clear scrollback` - Clear scrollback buffer

### View Commands
- `view: toggle fullscreen` - Toggle fullscreen mode
- `view: zoom in` - Increase font size
- `view: zoom out` - Decrease font size
- `view: zoom reset` - Reset font size

### Session Commands
- `session: new` - Create new session
- `session: switch` - Switch to another session
- `session: save` - Save current session
- `session: load` - Load saved session

### Theme Commands
- `theme: switch` - Change color theme
- `theme: light` - Switch to light theme
- `theme: dark` - Switch to dark theme

### Plugin Commands

Plugins can register custom commands:

```fsharp
// Example plugin command
let getCommands () = [
    { Name = "git: status"
      Description = "Show git status"
      Action = fun () -> showGitStatus() }
]
```

## Fuzzy Search

The command palette uses fuzzy matching:

- `nwtb` matches `terminal: new tab`
- `thdrk` matches `theme: dark`
- `plns` matches `plugins: install`

## Keyboard Shortcuts

| Key | Action |
|-----|--------|
| **Ctrl+Shift+P** | Open command palette |
| **Up/Down** | Navigate commands |
| **Enter** | Execute command |
| **Escape** | Close palette |
| **Ctrl+N/P** | Navigate (alternative) |

## Configuration

Customize command palette appearance:

```toml
[ui.command_palette]
max_items = 10
width = 600
height = 400
fuzzy_threshold = 0.5
```

## See Also

- [Keybindings](./keybindings.md) - Keyboard shortcuts
- [Plugins](./plugins.md) - Plugin commands
- [Plugin Development](../developer-guide/plugins.md) - Create custom commands
