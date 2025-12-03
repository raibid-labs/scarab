# Configuration Schema

Complete reference for Scarab configuration options.

## Overview

Scarab uses TOML configuration files located in:
- `~/.config/scarab/config.toml` (User configuration)
- `/etc/scarab/config.toml` (System defaults)

For general configuration information, see the [Configuration Guide](../user-guide/configuration.md).

## Full Configuration Schema

```toml
# Appearance settings
[appearance]
theme = "ayu-dark"          # Theme name
font_family = "monospace"   # Font family
font_size = 14.0            # Font size in points
line_spacing = 1.2          # Line spacing multiplier
cursor_style = "block"      # block | underline | beam
cursor_blink = true         # Enable cursor blinking

[appearance.colors]
# Custom color scheme (overrides theme)
foreground = "#e6e1cf"
background = "#0f1419"
# ... additional color definitions

# Behavior settings
[behavior]
shell = "/bin/bash"         # Default shell
scrollback_lines = 10000    # Scrollback buffer size
word_separators = " ,│`|:\"';()[]{}<>" # Word boundary chars
auto_save_session = true    # Save session on exit
restore_session = true      # Restore session on start

# Keybindings
[keybindings]
new_tab = "Ctrl+Shift+T"
close_tab = "Ctrl+Shift+W"
next_tab = "Ctrl+Tab"
previous_tab = "Ctrl+Shift+Tab"
split_horizontal = "Ctrl+Shift+H"
split_vertical = "Ctrl+Shift+V"
close_pane = "Ctrl+Shift+Q"
focus_up = "Ctrl+Shift+Up"
focus_down = "Ctrl+Shift+Down"
focus_left = "Ctrl+Shift+Left"
focus_right = "Ctrl+Shift+Right"
copy = "Ctrl+Shift+C"
paste = "Ctrl+Shift+V"
search = "Ctrl+Shift+F"

# Navigation settings
[navigation]
wraparound = false          # Wrap focus at boundaries
focus_follows_mouse = false # Focus pane on mouse hover
visual_bell = true          # Flash screen on bell
audio_bell = false          # Play sound on bell

# Plugin configuration
[plugins.daemon]
enabled = []                # List of .fzb plugins to load
plugin_dir = "~/.config/scarab/plugins/daemon"

[plugins.client]
enabled = []                # List of .fsx scripts to load
plugin_dir = "~/.config/scarab/plugins/client"
hot_reload = true           # Auto-reload on file change

# IPC settings
[ipc]
socket_path = "/tmp/scarab.sock"
shared_memory_path = "/scarab_shm_v1"
buffer_size = 8192          # Shared memory buffer size (bytes)

# Logging and debugging
[logging]
level = "info"              # trace | debug | info | warn | error
file = "~/.local/share/scarab/scarab.log"
console = false             # Log to console
```

## Data Types

- **String**: Quoted text (`"value"`)
- **Integer**: Whole number (`42`)
- **Float**: Decimal number (`1.5`)
- **Boolean**: `true` or `false`
- **Array**: List of values (`["item1", "item2"]`)

## Color Format

Colors can be specified as:
- Hex: `"#RRGGBB"` or `"#RRGGBBAA"`
- RGB: `{ r = 255, g = 128, b = 0 }`
- RGBA: `{ r = 255, g = 128, b = 0, a = 255 }`

## Keybinding Syntax

Modifiers:
- `Ctrl` - Control key
- `Shift` - Shift key
- `Alt` - Alt key
- `Super` - Super/Windows/Command key

Combine with `+`: `"Ctrl+Shift+T"`

Special keys:
- Arrow keys: `Up`, `Down`, `Left`, `Right`
- Function keys: `F1` through `F12`
- Others: `Tab`, `Enter`, `Escape`, `Backspace`, `Delete`

## Validation

Configuration is validated on startup. Invalid values will:
1. Log a warning
2. Fall back to default value
3. Continue startup (fail-safe)

## Configuration Priority

1. Command-line arguments (highest)
2. User config (`~/.config/scarab/config.toml`)
3. System config (`/etc/scarab/config.toml`)
4. Built-in defaults (lowest)

## Environment Variables

Override specific settings:

```bash
SCARAB_SHELL=/bin/zsh
SCARAB_LOG_LEVEL=debug
SCARAB_SOCKET_PATH=/custom/path.sock
```

## Profiles

Create per-profile configurations:

```toml
[profiles.work]
shell = "/bin/bash"
font_size = 12.0

[profiles.personal]
shell = "/bin/zsh"
font_size = 14.0
```

Activate profile:
```bash
scarab-client --profile work
```

## Related Documentation

- [Configuration Guide](../user-guide/configuration.md)
- [Keybindings](../user-guide/keybindings.md)
- [Plugin Development](../developer-guide/plugins.md)
