# Scarab Configuration Guide

This guide covers all configuration options for Scarab terminal emulator.

## Configuration File

Scarab reads configuration from `~/.config/scarab/config.toml` (or platform equivalent).

You can also create project-local configuration by placing a `.scarab.toml` file in your project directory. Local configuration overrides global settings.

## Configuration Structure

The configuration file uses TOML format and is organized into sections:

```toml
[terminal]
# Terminal emulator settings

[font]
# Font configuration

[colors]
# Color scheme and theme

[keybindings]
# Keyboard shortcuts

[ui]
# UI behavior and appearance

[plugins]
# Plugin management

[sessions]
# Session persistence

[telemetry]
# Logging and observability

[navigation]
# Navigation system settings
```

## Terminal Configuration

Controls terminal emulator behavior and PTY settings.

```toml
[terminal]
# Default shell to launch (defaults to $SHELL environment variable)
default_shell = "/bin/zsh"

# Number of scrollback lines to keep in history
scrollback_lines = 10000

# Enable alternate screen buffer support
alt_screen = true

# Scroll speed multiplier (higher = faster)
scroll_multiplier = 3.0

# Auto-scroll to bottom when new output appears
auto_scroll = true

# Initial terminal size (columns x rows)
columns = 80
rows = 24
```

### Default Values

| Option | Default | Description |
|--------|---------|-------------|
| `default_shell` | `$SHELL` or `/bin/zsh` | Shell executable path |
| `scrollback_lines` | `10000` | Lines of scrollback history |
| `alt_screen` | `true` | Enable alternate screen buffer |
| `scroll_multiplier` | `3.0` | Scroll speed multiplier |
| `auto_scroll` | `true` | Auto-scroll on new output |
| `columns` | `80` | Initial terminal width |
| `rows` | `24` | Initial terminal height |

## Font Configuration

Configure font family, size, and rendering options.

```toml
[font]
# Primary font family
family = "JetBrains Mono"

# Font size in points
size = 14.0

# Line height multiplier
line_height = 1.2

# Fallback fonts (tried in order if glyph not found)
fallback = ["Fira Code", "DejaVu Sans Mono", "Menlo"]

# Use bright variants for bold text
bold_is_bright = true

# Use thin stroke rendering (macOS only)
use_thin_strokes = false
```

### Default Values

| Option | Default | Description |
|--------|---------|-------------|
| `family` | `"JetBrains Mono"` | Primary font family |
| `size` | `14.0` | Font size in points |
| `line_height` | `1.2` | Line height multiplier |
| `fallback` | `["Fira Code", ...]` | Fallback font list |
| `bold_is_bright` | `true` | Bright colors for bold |
| `use_thin_strokes` | `false` | Thin stroke rendering |

## Color Configuration

Theme and color palette settings.

```toml
[colors]
# Theme name (built-in themes: "dracula", "nord", "monokai", etc.)
theme = "dracula"

# Custom colors (override theme)
foreground = "#f8f8f2"
background = "#282a36"
cursor = "#f8f8f2"
selection_background = "#44475a"
selection_foreground = "#f8f8f2"

# Window transparency (1.0 = opaque, 0.0 = fully transparent)
opacity = 1.0

# Dim text opacity (for dimmed colors)
dim_opacity = 0.7
```

### Color Palette

Define the 16-color ANSI palette:

```toml
[colors.palette]
# Normal colors
black = "#21222c"
red = "#ff5555"
green = "#50fa7b"
yellow = "#f1fa8c"
blue = "#bd93f9"
magenta = "#ff79c6"
cyan = "#8be9fd"
white = "#f8f8f2"

# Bright colors
bright_black = "#6272a4"
bright_red = "#ff6e6e"
bright_green = "#69ff94"
bright_yellow = "#ffffa5"
bright_blue = "#d6acff"
bright_magenta = "#ff92df"
bright_cyan = "#a4ffff"
bright_white = "#ffffff"
```

### Default Theme

The default theme is "dracula" with the palette shown above.

## Keybindings Configuration

Global keyboard shortcuts for terminal actions.

```toml
[keybindings]
# Leader key for command sequences
leader_key = "Space"

# Copy mode entry (visual selection)
copy_mode = "Ctrl+Shift+C"

# Paste from clipboard
paste = "Ctrl+Shift+V"

# Search mode
search = "Ctrl+Shift+F"

# Command palette
command_palette = "Ctrl+Shift+P"

# New window
new_window = "Ctrl+Shift+N"

# Close window
close_window = "Ctrl+Shift+W"

# Next tab
next_tab = "Ctrl+Tab"

# Previous tab
prev_tab = "Ctrl+Shift+Tab"
```

### Custom Keybindings

Add custom keybindings for actions:

```toml
[keybindings.custom]
"split_horizontal" = "Ctrl+Shift+H"
"split_vertical" = "Ctrl+Shift+V"
"zoom_in" = "Ctrl+Plus"
"zoom_out" = "Ctrl+Minus"
```

### Default Keybindings

| Action | Default | Description |
|--------|---------|-------------|
| `leader_key` | `Space` | Leader key prefix |
| `copy_mode` | `Ctrl+Shift+C` | Enter copy mode |
| `paste` | `Ctrl+Shift+V` | Paste clipboard |
| `search` | `Ctrl+Shift+F` | Search mode |
| `command_palette` | `Ctrl+Shift+P` | Command palette |
| `new_window` | `Ctrl+Shift+N` | New window |
| `close_window` | `Ctrl+Shift+W` | Close window |
| `next_tab` | `Ctrl+Tab` | Next tab |
| `prev_tab` | `Ctrl+Shift+Tab` | Previous tab |

## UI Configuration

UI appearance and behavior settings.

```toml
[ui]
# Enable link hints (Vimium-style navigation)
link_hints = true

# Enable command palette
command_palette = true

# Enable UI animations
animations = true

# Enable smooth scrolling
smooth_scroll = true

# Show tab bar
show_tabs = true

# Tab bar position: "top", "bottom", "left", "right"
tab_position = "top"

# Cursor style: "block", "beam", "underline"
cursor_style = "block"

# Enable cursor blinking
cursor_blink = true

# Cursor blink interval in milliseconds
cursor_blink_interval = 750

# Custom window icon (path to PNG file)
window_icon = "/path/to/icon.png"

# Case-sensitive search by default
search_case_sensitive = false

# Use regex in search by default
search_use_regex = false
```

### Default Values

| Option | Default | Description |
|--------|---------|-------------|
| `link_hints` | `true` | Enable link hints |
| `command_palette` | `true` | Enable command palette |
| `animations` | `true` | Enable animations |
| `smooth_scroll` | `true` | Smooth scrolling |
| `show_tabs` | `true` | Show tab bar |
| `tab_position` | `"top"` | Tab bar position |
| `cursor_style` | `"block"` | Cursor appearance |
| `cursor_blink` | `true` | Cursor blinking |
| `cursor_blink_interval` | `750` | Blink interval (ms) |
| `window_icon` | `None` | Custom icon path |
| `search_case_sensitive` | `false` | Case-sensitive search |
| `search_use_regex` | `false` | Regex search mode |

## Plugin Configuration

Plugin management and per-plugin settings.

```toml
[plugins]
# List of enabled plugins (by name)
enabled = ["git-status", "tmux-integration", "url-preview"]

# Per-plugin configuration
[plugins.config.git-status]
show_branch = true
refresh_interval = 5000

[plugins.config.tmux-integration]
prefix_key = "Ctrl+B"
```

### Default Values

| Option | Default | Description |
|--------|---------|-------------|
| `enabled` | `[]` | Empty list (no plugins) |
| `config` | `{}` | No plugin-specific config |

### Plugin Registry Configuration

Configure the plugin registry and security settings:

```toml
[plugins.registry]
# Remote registry URL
registry_url = "https://registry.scarab.dev"

# Local cache directory
cache_dir = "~/.config/scarab/registry"

# Plugin installation directory
plugin_dir = "~/.config/scarab/plugins"

[plugins.registry.security]
# Require SHA256 checksum verification
require_checksum = true

# Require GPG signature verification
require_signature = false

# Trusted GPG key fingerprints (40-character hex strings)
trusted_keys = [
    "ABCD1234ABCD1234ABCD1234ABCD1234ABCD1234"
]

# Allow unsigned plugins (dangerous!)
allow_unsigned = true

# Path to additional keyring file (OpenPGP format)
keyring_path = "~/.config/scarab/keyring.gpg"

# Require signatures from specific key IDs only
require_key_match = true

# Maximum allowed signature age in days (0 = no limit)
max_signature_age_days = 365
```

### Plugin Security Defaults

| Option | Default | Description |
|--------|---------|-------------|
| `require_checksum` | `true` | Verify SHA256 checksums |
| `require_signature` | `false` | Verify GPG signatures |
| `trusted_keys` | `[]` | No trusted keys |
| `allow_unsigned` | `true` | Allow unsigned plugins |
| `keyring_path` | `None` | No custom keyring |
| `require_key_match` | `true` | Strict key matching |
| `max_signature_age_days` | `365` | 1 year maximum age |

## Session Configuration

Session persistence and restoration settings.

```toml
[sessions]
# Restore previous session on startup
restore_on_startup = false

# Auto-save session interval in seconds
auto_save_interval = 300

# Save scrollback buffer in session
save_scrollback = true

# Default working directory (overrides shell's default)
working_directory = "/home/user/projects"
```

### Default Values

| Option | Default | Description |
|--------|---------|-------------|
| `restore_on_startup` | `false` | Restore sessions on start |
| `auto_save_interval` | `300` | Save every 5 minutes |
| `save_scrollback` | `true` | Include scrollback |
| `working_directory` | `None` | Use shell default |

## Telemetry Configuration

Logging and observability settings. All telemetry is opt-in and disabled by default to avoid performance impact.

```toml
[telemetry]
# Log compositor FPS every N seconds (0 = disabled)
fps_log_interval_secs = 0

# Log sequence number changes in compositor
log_sequence_changes = false

# Log dirty region sizes when blitting to shared memory
log_dirty_regions = false

# Log pane lifecycle events
log_pane_events = false
```

### Default Values

| Option | Default | Description |
|--------|---------|-------------|
| `fps_log_interval_secs` | `0` | Disabled |
| `log_sequence_changes` | `false` | Disabled |
| `log_dirty_regions` | `false` | Disabled |
| `log_pane_events` | `false` | Disabled |

### Environment Variable Overrides

Telemetry can be enabled via environment variables (overrides config file):

```bash
# Log FPS every 5 seconds
export SCARAB_LOG_FPS=5

# Enable sequence number logging
export SCARAB_LOG_SEQUENCE=1

# Enable dirty region logging
export SCARAB_LOG_DIRTY=1

# Enable pane lifecycle logging
export SCARAB_LOG_PANES=1
```

## Navigation Configuration

Keyboard navigation system settings (Vimium-style link hints).

```toml
[navigation]
# Navigation style: "vimium", "cosmos", "spacemacs"
style = "vimium"

# Allow plugins to enter hint mode
allow_plugin_hint_mode = true

# Allow plugins to register focusable elements
allow_plugin_focusables = true
```

### Navigation Keybindings

Custom keybindings for navigation actions:

```toml
[navigation.keybindings]
enter_hints = "Ctrl+F"
cancel = "Escape"
prev_prompt = "Ctrl+Up"
next_prompt = "Ctrl+Down"
```

See the [Navigation Documentation](navigation.md) for complete details on navigation modes, keymaps, and usage.

### Navigation Styles

Scarab supports three navigation style presets:

#### Vimium Style (Default)

Browser-inspired navigation with single-key hints:

- `F` or `Ctrl+F` - Enter hint mode
- `Escape` - Cancel/exit modes
- `Ctrl+Up/Down` - Navigate prompts
- `a-z` - Type hint labels

#### Cosmos Style

Space-based leader key approach:

- `F` - Enter hint mode
- `Escape` - Cancel/exit modes
- `Ctrl+Up/Down` - Navigate prompts

*Note: Full leader key pattern is planned but not yet implemented.*

#### Spacemacs Style

SPC prefix pattern:

- `F` - Enter hint mode
- `Escape` - Cancel/exit modes
- `Ctrl+Up/Down` - Navigate prompts

*Note: Full SPC prefix pattern is planned but not yet implemented.*

### Default Values

| Option | Default | Description |
|--------|---------|-------------|
| `style` | `"vimium"` | Navigation keymap style |
| `allow_plugin_hint_mode` | `true` | Plugins can trigger hints |
| `allow_plugin_focusables` | `true` | Plugins can add focusables |

### Plugin Capability Limits

Plugins must declare capabilities in their manifest to use navigation features:

```toml
[plugin.capabilities]
# Enter hint mode programmatically
can_enter_hint_mode = true

# Register custom focusable elements
can_register_focusables = true

# Execute navigation actions
can_trigger_actions = true
```

**Enforcement:**
- Operations are rejected if the plugin lacks required capabilities
- Rejections are logged to telemetry (`nav.plugin_actions_rejected`)
- Sandboxed plugins have stricter restrictions than trusted plugins

**Rate Limits:**
- Action rate limit: 10 actions/second per plugin
- Max focusables: 50 focusables per plugin at any time
- Burst allowance: Small burst over limit is tolerated

**Bounds Validation:**
- Coordinates must be non-negative
- Width and height must be > 0 and < 1000
- Focusables should be within the current viewport
- Out-of-bounds focusables are rejected silently

**Conflict Resolution:**

Set `allow_plugin_hint_mode = false` to prevent plugins from triggering hint mode if you have a conflicting plugin. Set `allow_plugin_focusables = false` to prevent plugins from adding custom focusable elements.

## Environment Variables

Scarab respects the following environment variables:

| Variable | Description | Default |
|----------|-------------|---------|
| `SCARAB_CONFIG` | Override config file path | `~/.config/scarab/config.toml` |
| `SCARAB_LOG` | Log level | `info` |
| `SCARAB_LOG_FPS` | Log FPS interval (seconds) | `0` (disabled) |
| `SCARAB_LOG_SEQUENCE` | Log sequence changes | `false` |
| `SCARAB_LOG_DIRTY` | Log dirty regions | `false` |
| `SCARAB_LOG_PANES` | Log pane events | `false` |
| `SHELL` | Default shell executable | `/bin/bash` or `/bin/zsh` |
| `EDITOR` | Text editor for file navigation | `vim` or `nano` |
| `XDG_CONFIG_HOME` | Config directory (Linux) | `~/.config` |
| `XDG_RUNTIME_DIR` | Runtime directory (Linux) | `/tmp` |
| `TMPDIR` | Temporary directory | `/tmp` |

## Default Values Summary

All configuration options have sensible defaults. A minimal config file only needs to override what you want to change.

### Minimal Config

An empty configuration file uses all defaults:

```toml
# Uses all default values
```

### Common Customizations

Override just the settings you care about:

```toml
[font]
family = "Fira Code"
size = 16.0

[colors]
theme = "nord"

[terminal]
default_shell = "/bin/fish"
```

## Example Configurations

### Minimal Config

Use all defaults:

```toml
# Empty config - uses all defaults
```

### Developer Setup

Optimized for development work:

```toml
[terminal]
default_shell = "/bin/zsh"
scrollback_lines = 50000

[font]
family = "JetBrains Mono"
size = 13.0
line_height = 1.3

[colors]
theme = "dracula"
opacity = 0.95

[ui]
link_hints = true
smooth_scroll = true
cursor_blink = false

[sessions]
restore_on_startup = true
auto_save_interval = 180

[telemetry]
fps_log_interval_secs = 5
log_pane_events = true
```

### Spacemacs User

Space-based leader key workflow:

```toml
[navigation]
style = "spacemacs"

[navigation.keybindings]
enter_hints = "SPC f"
prev_prompt = "SPC p"
next_prompt = "SPC n"

[keybindings]
leader_key = "Space"
copy_mode = "SPC y"
search = "SPC s"
command_palette = "SPC SPC"
```

### High Security

Strict plugin security requirements:

```toml
[plugins]
enabled = []  # No plugins

[plugins.registry.security]
require_checksum = true
require_signature = true
allow_unsigned = false
require_key_match = true
max_signature_age_days = 90

trusted_keys = [
    "1234567890ABCDEF1234567890ABCDEF12345678"
]
```

### Performance Monitoring

Enable all telemetry for debugging:

```toml
[telemetry]
fps_log_interval_secs = 1
log_sequence_changes = true
log_dirty_regions = true
log_pane_events = true
```

Run with environment variables:

```bash
SCARAB_LOG_FPS=1 \
SCARAB_LOG_SEQUENCE=1 \
SCARAB_LOG_DIRTY=1 \
SCARAB_LOG_PANES=1 \
scarab
```

### Accessibility

High contrast with larger fonts:

```toml
[font]
family = "DejaVu Sans Mono"
size = 18.0
line_height = 1.5
bold_is_bright = true

[colors]
theme = "high-contrast"
opacity = 1.0

[ui]
cursor_style = "block"
cursor_blink = false
animations = false
```

## Configuration Discovery

Scarab searches for configuration in the following order:

1. **Environment override**: `$SCARAB_CONFIG` (if set)
2. **Local config**: `.scarab.toml` in current directory (walks up directory tree)
3. **Global config**: `~/.config/scarab/config.toml` (platform-specific)

Local configuration files override global settings.

### Platform-Specific Paths

| Platform | Config Directory |
|----------|------------------|
| Linux | `~/.config/scarab/` |
| macOS | `~/Library/Application Support/scarab/` |
| Windows | `%APPDATA%\scarab\` |

## Validation

Scarab validates configuration on load and reports errors:

```bash
$ scarab
ERROR: Invalid configuration: font.size must be positive
ERROR: Unknown color theme: "invalid-theme"
```

### Common Validation Errors

- **Invalid TOML syntax**: Check brackets, quotes, and commas
- **Unknown theme**: Use built-in theme or define custom colors
- **Invalid key combination**: Check modifier key syntax (Ctrl+Shift+...)
- **Invalid path**: Ensure file paths exist and are accessible
- **Out of range values**: Check numeric values are within valid ranges

## Advanced Topics

### Merging Configuration

When both global and local configs exist, they are merged:

1. Global config is loaded first
2. Local config overrides matching sections
3. Arrays and maps are extended (not replaced)
4. Custom keybindings are merged

Example:

```toml
# Global: ~/.config/scarab/config.toml
[plugins]
enabled = ["git-status"]

[plugins.config.git-status]
show_branch = true
```

```toml
# Local: /project/.scarab.toml
[plugins]
enabled = ["tmux-integration"]  # Extends global list

[plugins.config.git-status]
refresh_interval = 1000  # Adds to global config
```

Result:
```toml
[plugins]
enabled = ["git-status", "tmux-integration"]

[plugins.config.git-status]
show_branch = true
refresh_interval = 1000
```

### Hot Reloading

Scarab watches the configuration file for changes and reloads automatically. Most settings take effect immediately without restart.

Settings that require restart:
- `terminal.default_shell`
- Plugin list changes
- Window-level settings

### Configuration from Fusabi Scripts

Advanced users can load configuration from Fusabi scripts (.fsx files) for dynamic configuration:

```fsharp
// config.fsx - Fusabi configuration script
let config = {
    Font = {
        Family = "JetBrains Mono"
        Size = if isDarkMode() then 14.0 else 16.0
    }
    Colors = {
        Theme = if time.Hour < 18 then "light" else "dark"
    }
}
```

See the [Fusabi Guide](FUSABI_GUIDE.md) for details.

## Troubleshooting

### Config Not Loading

1. Check file location: `~/.config/scarab/config.toml`
2. Verify TOML syntax: `toml-lint config.toml`
3. Check file permissions: Must be readable by user
4. Look for error messages in `scarab --debug`

### Settings Not Taking Effect

1. Check if setting requires restart
2. Verify setting is in correct section
3. Check for local config override
4. Enable debug logging: `SCARAB_LOG=debug scarab`

### Performance Issues

1. Disable animations: `ui.animations = false`
2. Reduce scrollback: `terminal.scrollback_lines = 1000`
3. Disable telemetry: All telemetry options to `false`
4. Reduce font size: `font.size = 12.0`

## Related Documentation

- [Navigation System](navigation.md) - Detailed navigation configuration
- [Plugin Development Guide](plugin-development/README.md) - Plugin capabilities and API
- [Telemetry Quick Reference](TELEMETRY_QUICK_REFERENCE.md) - Telemetry metrics
- [Fusabi Guide](FUSABI_GUIDE.md) - Fusabi scripting for configuration

## See Also

- **Config Location**: `~/.config/scarab/config.toml`
- **Example Configs**: `examples/configs/` directory
- **Config Schema**: JSON schema available for IDE autocomplete
- **Config Validation**: `scarab --validate-config`
