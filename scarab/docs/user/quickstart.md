# Quick Start Guide

Get up and running with Scarab in 5 minutes!

## First Launch

After [installation](installation.md), launch Scarab:

```bash
scarab
```

You'll see Scarab's welcome screen with your default shell.

## Basic Usage

### Creating Sessions

```bash
# Launch with default session
scarab

# Create a named session
scarab session create my-work

# Attach to existing session
scarab session attach my-work

# List all sessions
scarab session list
```

### Keyboard Shortcuts

| Action | Shortcut |
|--------|----------|
| New Tab | `Cmd+T` (macOS) / `Ctrl+Shift+T` (Linux/Windows) |
| Close Tab | `Cmd+W` / `Ctrl+Shift+W` |
| Next Tab | `Cmd+]` / `Ctrl+PageDown` |
| Previous Tab | `Cmd+[` / `Ctrl+PageUp` |
| Split Horizontally | `Cmd+D` / `Ctrl+Shift+D` |
| Split Vertically | `Cmd+Shift+D` / `Ctrl+Shift+Alt+D` |
| Command Palette | `Cmd+Shift+P` / `Ctrl+Shift+P` |
| Settings | `Cmd+,` / `Ctrl+,` |

See [Keybindings](keybindings.md) for the full list.

## Configuration

### Quick Config

Edit your config file:

**macOS/Linux:**
```bash
nano ~/.config/scarab/config.toml
```

**Windows:**
```powershell
notepad %APPDATA%\scarab\config.toml
```

### Essential Settings

```toml
[terminal]
# Your preferred shell
shell = "/bin/zsh"

# Terminal size
initial_cols = 120
initial_rows = 40

[appearance]
# Font settings
font_family = "JetBrains Mono"
font_size = 14.0

# Theme
[appearance.theme]
name = "dracula"

[appearance.theme.colors]
background = "#282a36"
foreground = "#f8f8f2"
```

See [Configuration Guide](configuration.md) for all options.

## Common Tasks

### Changing the Shell

```toml
[terminal]
shell = "/bin/zsh"  # or "/bin/bash", "/usr/bin/fish", etc.
```

### Changing the Font

```toml
[appearance]
font_family = "JetBrains Mono"
font_size = 14.0
```

### Setting a Theme

```toml
[appearance.theme]
name = "dracula"  # or "solarized-dark", "gruvbox", etc.
```

### Customizing Keybindings

```toml
[keybindings]
new_tab = "ctrl+t"
close_tab = "ctrl+w"
split_horizontal = "ctrl+shift+h"
split_vertical = "ctrl+shift+v"
```

## Session Management

Scarab includes powerful session management:

### Create and Persist Sessions

```bash
# Create a session that persists after closing
scarab session create work-project

# Do your work...

# Detach (session keeps running)
scarab session detach

# Later, reattach
scarab session attach work-project
```

### List Sessions

```bash
scarab session list
```

Output:
```
NAME          STATUS    CREATED             LAST ATTACHED
work-project  attached  2024-01-15 09:30    2024-01-15 14:45
side-project  detached  2024-01-14 16:20    2024-01-14 18:30
```

### Delete Sessions

```bash
scarab session delete work-project
```

## Plugins

Scarab supports powerful plugins:

### Install a Plugin

```bash
scarab plugin install git-status
```

### Enable a Plugin

```toml
[plugins]
enabled = ["git-status", "syntax-highlight"]
```

### List Available Plugins

```bash
scarab plugin list
```

See [Plugin Guide](plugins.md) for more.

## Command Palette

Press `Cmd+Shift+P` (or `Ctrl+Shift+P`) to open the command palette:

- Type to search commands
- Use arrow keys to navigate
- Press Enter to execute

Available commands:
- `Session: Create`
- `Session: Attach`
- `Session: Detach`
- `Session: Delete`
- `Theme: Select`
- `Settings: Open`
- `Plugins: Manage`

## Tips & Tricks

### 1. Search in Terminal

Press `Cmd+F` (or `Ctrl+F`) to search the current buffer.

### 2. Copy/Paste

- **Copy**: Select text, then `Cmd+C` / `Ctrl+Shift+C`
- **Paste**: `Cmd+V` / `Ctrl+Shift+V`

### 3. Zoom

- **Zoom In**: `Cmd++` / `Ctrl++`
- **Zoom Out**: `Cmd+-` / `Ctrl+-`
- **Reset**: `Cmd+0` / `Ctrl+0`

### 4. Full Screen

Press `Cmd+Enter` (or `F11`) to toggle full screen.

### 5. Tab Management

Right-click on tabs for quick actions:
- Rename
- Close
- Close Others
- Close to the Right

## Troubleshooting

### Colors Look Wrong

Check your `TERM` environment variable:

```bash
echo $TERM
```

Should be `xterm-256color` or similar. Set in your shell config:

```bash
export TERM=xterm-256color
```

### Font Not Found

Install the font system-wide:

**macOS:**
```bash
brew tap homebrew/cask-fonts
brew install font-jetbrains-mono
```

**Linux:**
```bash
sudo apt install fonts-jetbrains-mono
```

### Performance Issues

Try disabling GPU rendering:

```toml
[rendering]
use_gpu = false
```

## Next Steps

- [Configuration Reference](configuration.md) - Detailed config options
- [Keybindings](keybindings.md) - All keyboard shortcuts
- [Plugins](plugins.md) - Extend functionality
- [Developer Guide](../developer/architecture.md) - Build plugins

## Getting Help

- [GitHub Discussions](https://github.com/yourusername/scarab/discussions)
- [Issue Tracker](https://github.com/yourusername/scarab/issues)
- [Discord](https://discord.gg/scarab)
