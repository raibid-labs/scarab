# Configuration Reference

Complete guide to Scarab Terminal configuration options.

## Overview

Scarab uses TOML-based configuration with hot-reload support. Configuration files are loaded in order of precedence:

1. **Global config**: `~/.config/scarab/config.toml`
2. **Project override**: `.scarab.toml` (in current directory)
3. **Environment variables**: `SCARAB_*` prefix

Changes to configuration files are detected automatically and applied within 100ms.

## Quick Start

Generate default configuration:

```bash
scarab-daemon --print-config > ~/.config/scarab/config.toml
```

Minimal configuration (uses all defaults):

```toml
# ~/.config/scarab/config.toml
[terminal]
default_shell = "/bin/zsh"

[font]
family = "JetBrains Mono"
size = 14.0
```

## Configuration Sections

### [terminal]

Terminal emulator behavior settings.

```toml
[terminal]
# Shell to execute on startup
# Default: $SHELL environment variable, falls back to "/bin/zsh"
# Example: "/bin/bash", "/usr/bin/fish", "/bin/zsh"
default_shell = "/bin/zsh"

# Maximum scrollback buffer lines
# Default: 10000
# Range: 1000 - 100000
# Higher values use more memory (~1MB per 1000 lines)
scrollback_lines = 10000

# Enable alternate screen buffer (used by vim, less, etc)
# Default: true
# Set to false to disable alternate screen switching
alt_screen = true

# Scroll multiplier for mouse wheel events
# Default: 3.0
# Range: 0.5 - 10.0
# Higher values scroll faster
scroll_multiplier = 3.0

# Auto-scroll to bottom on new output
# Default: true
# Set to false to preserve scroll position when new text appears
auto_scroll = true

# Initial terminal dimensions
# Default: 80 columns, 24 rows
# Range: columns 20-500, rows 5-200
columns = 80
rows = 24
```

**Validation Rules**:
- `scrollback_lines`: Must be between 1,000 and 100,000
- `scroll_multiplier`: Must be between 0.5 and 10.0
- `columns`: Must be between 20 and 500
- `rows`: Must be between 5 and 200

---

### [font]

Font rendering configuration.

```toml
[font]
# Primary font family
# Default: "JetBrains Mono"
# Must be installed on system
# Recommended monospace fonts with ligature support:
#   - "JetBrains Mono"
#   - "Fira Code"
#   - "Cascadia Code"
#   - "SF Mono" (macOS)
#   - "Consolas" (Windows)
family = "JetBrains Mono"

# Font size in points
# Default: 14.0
# Range: 6.0 - 72.0
size = 14.0

# Line height multiplier
# Default: 1.2
# Range: 0.8 - 2.0
# 1.0 = tight spacing, 1.5 = spacious
line_height = 1.2

# Fallback fonts (checked in order if glyph not found)
# Default: ["Fira Code", "DejaVu Sans Mono", "Menlo"]
# Used for emoji, symbols, and non-Latin scripts
fallback = [
    "Fira Code",
    "DejaVu Sans Mono",
    "Menlo",
    "Noto Color Emoji"  # For emoji support
]

# Render bold text with bright colors
# Default: true
# When true, bold text uses bright color variants (8-15)
bold_is_bright = true

# Use thin strokes for font rendering (macOS only)
# Default: false
# Reduces font weight on Retina displays
use_thin_strokes = false
```

**Validation Rules**:
- `size`: Must be between 6.0 and 72.0
- `line_height`: Must be between 0.8 and 2.0
- `family`: Must be valid font name installed on system

**Font Discovery**: Scarab searches system font directories:
- Linux: `/usr/share/fonts`, `~/.local/share/fonts`
- macOS: `/Library/Fonts`, `~/Library/Fonts`
- Windows: `C:\Windows\Fonts`

---

### [colors]

Color scheme and theme configuration.

```toml
[colors]
# Built-in theme name
# Default: "dracula"
# Available themes: "dracula", "nord", "gruvbox", "solarized-dark",
#                   "solarized-light", "monokai", "one-dark", "tokyo-night"
# Set to null to use custom colors below
theme = "dracula"

# Custom foreground color (overrides theme)
# Default: null (uses theme)
# Format: "#RRGGBB" hex color
foreground = "#f8f8f2"

# Custom background color (overrides theme)
# Default: null (uses theme)
background = "#282a36"

# Cursor color
# Default: null (uses theme)
cursor = "#ffffff"

# Selection background color
# Default: null (uses theme)
selection_background = "#44475a"

# Selection foreground color
# Default: null (uses theme foreground)
selection_foreground = null

# Background opacity
# Default: 1.0 (opaque)
# Range: 0.0 - 1.0
# Requires compositor support (Linux/macOS)
opacity = 1.0

# Dim color opacity (for dimmed/inactive text)
# Default: 0.7
# Range: 0.0 - 1.0
dim_opacity = 0.7
```

**Custom Color Palette**:

Define 16-color ANSI palette (overrides theme):

```toml
[colors.palette]
# Normal colors (0-7)
black = "#21222c"
red = "#ff5555"
green = "#50fa7b"
yellow = "#f1fa8c"
blue = "#bd93f9"
magenta = "#ff79c6"
cyan = "#8be9fd"
white = "#f8f8f2"

# Bright colors (8-15)
bright_black = "#6272a4"
bright_red = "#ff6e6e"
bright_green = "#69ff94"
bright_yellow = "#ffffa5"
bright_blue = "#d6acff"
bright_magenta = "#ff92df"
bright_cyan = "#a4ffff"
bright_white = "#ffffff"
```

**Validation Rules**:
- All colors must be valid hex format: `#RRGGBB` or `#RRGGBBAA`
- `opacity`: Must be between 0.0 and 1.0
- `dim_opacity`: Must be between 0.0 and 1.0

**Theme Loading**: Custom themes can be added to `~/.config/scarab/themes/<name>.toml`

---

### [keybindings]

Keyboard shortcut customization.

```toml
[keybindings]
# Leader key for command sequences (vim-style)
# Default: "Space"
# Options: Any key name (e.g., "Space", "Escape", "Tab")
leader_key = "Space"

# Copy selected text
# Default: "Ctrl+Shift+C"
copy_mode = "Ctrl+Shift+C"

# Paste from clipboard
# Default: "Ctrl+Shift+V"
paste = "Ctrl+Shift+V"

# Open search mode
# Default: "Ctrl+Shift+F"
search = "Ctrl+Shift+F"

# Open command palette
# Default: "Ctrl+Shift+P"
command_palette = "Ctrl+Shift+P"

# Create new window
# Default: "Ctrl+Shift+N"
new_window = "Ctrl+Shift+N"

# Close current window
# Default: "Ctrl+Shift+W"
close_window = "Ctrl+Shift+W"

# Switch to next tab
# Default: "Ctrl+Tab"
next_tab = "Ctrl+Tab"

# Switch to previous tab
# Default: "Ctrl+Shift+Tab"
prev_tab = "Ctrl+Shift+Tab"

# Custom keybindings
[keybindings.custom]
# Format: "action" = "key combination"
"scroll_page_up" = "Shift+PageUp"
"scroll_page_down" = "Shift+PageDown"
"scroll_to_top" = "Shift+Home"
"scroll_to_bottom" = "Shift+End"
"increase_font_size" = "Ctrl+Plus"
"decrease_font_size" = "Ctrl+Minus"
"reset_font_size" = "Ctrl+0"
"toggle_fullscreen" = "F11"
"link_hints" = "Ctrl+Shift+O"
```

**Key Format**:
- Modifiers: `Ctrl`, `Shift`, `Alt`, `Super` (Cmd on macOS)
- Combine with `+`: `Ctrl+Shift+P`
- Special keys: `Space`, `Enter`, `Tab`, `Escape`, `Backspace`, `Delete`,
  `Home`, `End`, `PageUp`, `PageDown`, `F1`-`F12`, `Left`, `Right`, `Up`, `Down`

**Available Actions**:
See [keybindings.md](./keybindings.md) for complete list.

---

### [ui]

User interface behavior and visual effects.

```toml
[ui]
# Enable link hints (keyboard navigation to URLs)
# Default: true
link_hints = true

# Enable command palette
# Default: true
command_palette = true

# Enable UI animations
# Default: true
# Set to false to disable fade/slide transitions
animations = true

# Enable smooth scrolling
# Default: true
# Set to false for instant jumps
smooth_scroll = true

# Show tab bar
# Default: true
show_tabs = true

# Tab bar position
# Default: "top"
# Options: "top", "bottom", "left", "right"
tab_position = "top"

# Cursor style
# Default: "block"
# Options: "block", "beam", "underline"
cursor_style = "block"

# Enable cursor blinking
# Default: true
cursor_blink = true

# Cursor blink interval in milliseconds
# Default: 750
# Range: 100 - 2000
cursor_blink_interval = 750
```

**Validation Rules**:
- `tab_position`: Must be "top", "bottom", "left", or "right"
- `cursor_style`: Must be "block", "beam", or "underline"
- `cursor_blink_interval`: Must be between 100 and 2000

---

### [plugins]

Plugin system configuration.

```toml
[plugins]
# Enable plugins automatically on startup
# Default: []
# Plugin names without .fsx/.fzb extension
enabled = [
    "scarab-nav",
    "scarab-palette",
    "scarab-session"
]

# Plugin-specific configuration
[plugins.config]
# Configuration passed to plugins as JSON
# Each plugin defines its own config schema

[plugins.config.scarab-nav]
# Link hint characters (ordered by preference)
hint_chars = "asdfghjkl"
# Link highlight color
highlight_color = "#ff79c6"

[plugins.config.scarab-palette]
# Max recent commands shown
max_recent = 50
# Fuzzy search algorithm
fuzzy_search = true

[plugins.config.scarab-session]
# Auto-save interval in seconds
auto_save_interval = 300
# Session storage location
session_dir = "~/.local/share/scarab/sessions"
```

**Plugin Discovery**:

Plugins are loaded from:
1. `~/.config/scarab/plugins/` (user plugins)
2. `~/.local/share/scarab/plugins/` (installed plugins)
3. `/usr/share/scarab/plugins/` (system plugins)

**Plugin Types**:
- `.fsx` files: F# scripts (hot-reloadable, client or daemon)
- `.fzb` files: Compiled Fusabi bytecode (daemon only, high-performance)

---

### [sessions]

Session management and persistence.

```toml
[sessions]
# Restore last session on startup
# Default: false
restore_on_startup = false

# Auto-save interval in seconds
# Default: 300 (5 minutes)
# Range: 60 - 3600
# Set to 0 to disable auto-save
auto_save_interval = 300

# Save scrollback buffer in sessions
# Default: true
# Warning: Large scrollback can increase session file size
save_scrollback = true

# Default working directory for new sessions
# Default: null (uses current directory)
# Options: null, "home", or absolute path
working_directory = null
```

**Session Storage**:
- Location: `~/.local/share/scarab/sessions/`
- Format: SQLite database with JSON metadata
- Max sessions: 100 (oldest auto-deleted)

**Validation Rules**:
- `auto_save_interval`: Must be 0 (disabled) or 60-3600
- `working_directory`: Must be null, "home", or valid absolute path

---

## Advanced Configuration Examples

### High-Performance Setup

Optimized for low latency and high throughput:

```toml
[terminal]
scrollback_lines = 5000  # Reduced for lower memory
auto_scroll = true
scroll_multiplier = 5.0

[font]
size = 13.0
line_height = 1.0  # Tight spacing

[colors]
opacity = 1.0  # No transparency (faster)

[ui]
animations = false
smooth_scroll = false
cursor_blink = false
```

### Accessibility Setup

Enhanced readability and contrast:

```toml
[font]
size = 18.0
line_height = 1.5
bold_is_bright = true

[colors]
theme = "solarized-light"
opacity = 1.0

[ui]
cursor_style = "block"
cursor_blink = false
```

### Developer Setup

Tailored for coding workflows:

```toml
[terminal]
scrollback_lines = 50000
columns = 120
rows = 40

[font]
family = "JetBrains Mono"
size = 13.0
line_height = 1.3

[plugins]
enabled = [
    "scarab-nav",
    "scarab-palette",
    "git-status",
    "notification-monitor"
]

[plugins.config.git-status]
show_branch = true
show_dirty = true
position = "top-right"
```

### Minimal Setup

Zero-config with essential features only:

```toml
[terminal]
default_shell = "/bin/zsh"

[font]
family = "monospace"
size = 14.0

[colors]
theme = "dracula"
```

---

## Environment Variables

Override configuration via environment:

| Variable | Description | Example |
|----------|-------------|---------|
| `SCARAB_CONFIG` | Config file path | `~/.scarab.toml` |
| `SCARAB_SHELL` | Override `default_shell` | `/bin/bash` |
| `SCARAB_FONT_SIZE` | Override `font.size` | `16.0` |
| `SCARAB_THEME` | Override `colors.theme` | `nord` |
| `SCARAB_LOG_LEVEL` | Logging level | `debug` |
| `SCARAB_PLUGIN_DIR` | Additional plugin directory | `~/my-plugins` |

**Priority**: Environment variables > Project `.scarab.toml` > Global `config.toml` > Defaults

---

## Configuration Validation

Scarab validates configuration on load and provides helpful errors:

**Example validation error**:
```
Error: Invalid configuration at line 12
  font.size = 100.0
  ^^^^^^^^^^^^^^^^^^^^
  Font size must be between 6.0 and 72.0 (got 100.0)

Suggestion: Try a size between 12.0 and 18.0 for typical displays
```

**Validation on save**:

Enable real-time validation in your editor:

```bash
# Install scarab-lint (validates on file change)
cargo install scarab-lint

# Run in config directory
cd ~/.config/scarab
scarab-lint watch config.toml
```

---

## Hot Reload

Configuration changes apply instantly without restarting:

**Daemon hot-reloads**:
- `terminal.*` (requires new session)
- `plugins.*` (reloads affected plugins)
- `sessions.*`

**Client hot-reloads**:
- `font.*` (rebuilds texture atlas)
- `colors.*` (updates shader uniforms)
- `ui.*` (applies immediately)
- `keybindings.*` (updates key handlers)

**Requires restart**:
- None! All settings support hot-reload.

**Reload trigger**: File modification detected via `inotify` (Linux) or `FSEvents` (macOS).

---

## Troubleshooting

### Config not loading

**Symptom**: Changes don't apply

**Solutions**:
1. Check file location: `~/.config/scarab/config.toml`
2. Verify TOML syntax: `scarab-daemon --validate-config`
3. Check logs: `tail -f ~/.local/share/scarab/scarab.log`
4. Test with minimal config (see examples above)

### Invalid font

**Symptom**: Fallback font used instead of configured font

**Solutions**:
1. Verify font installed: `fc-list | grep "YourFont"`
2. Use exact font name from `fc-list` output
3. Add to `fallback` array for redundancy
4. Test with known font: `family = "monospace"`

### Theme not found

**Symptom**: Default theme used

**Solutions**:
1. Check theme name spelling (case-sensitive)
2. List available themes: `ls ~/.config/scarab/themes/`
3. Use custom colors instead (see [colors.palette])
4. Download themes: `scarab-daemon --download-themes`

### Keybinding conflicts

**Symptom**: Key combination doesn't work

**Solutions**:
1. Check for desktop environment conflicts (e.g., `Ctrl+Alt+T`)
2. Use alternative modifier: `Super` instead of `Ctrl`
3. Test in isolated environment (no DE shortcuts)
4. View current bindings: `scarab-daemon --list-keybindings`

---

## See Also

- [Keybindings Reference](./keybindings.md) - Complete keyboard shortcuts
- [Performance Tuning](./performance.md) - Optimization guide
- [Troubleshooting](./troubleshooting.md) - Common issues
- [FAQ](./faq.md) - Frequently asked questions
