# Customization Guide

**Personalize Scarab to match your workflow and aesthetic**

---

## Table of Contents

1. [Configuration File Structure](#configuration-file-structure)
2. [Themes and Colors](#themes-and-colors)
3. [Font Configuration](#font-configuration)
4. [Custom Keybindings](#custom-keybindings)
5. [Performance Tuning](#performance-tuning)
6. [Plugin Configuration](#plugin-configuration)

---

## Configuration File Structure

Scarab uses TOML for configuration. The main config file is:
```
~/.config/scarab/config.toml
```

### Complete Configuration Template

Here's a comprehensive config with all available options:

```toml
# ~/.config/scarab/config.toml
# Scarab Terminal Configuration

[terminal]
# Terminal dimensions (in characters)
columns = 120
rows = 40

# Scrollback buffer (lines)
scrollback = 10000

# Shell to use (default: $SHELL or /bin/bash)
shell = "/bin/zsh"

# Shell arguments
shell_args = ["-l"]  # Login shell

# Working directory (default: $HOME)
working_directory = "~"

[font]
# Font family (must be installed on system)
family = "JetBrains Mono"

# Font size in points
size = 14.0

# Use bold variant for bold text
bold = true

# Use italic variant for italic text
italic = true

# Font features (OpenType features)
features = ["calt", "liga"]  # Enable ligatures

# Font hinting (none, slight, medium, full)
hinting = "slight"

# Subpixel rendering (none, rgb, bgr, vrgb, vbgr)
subpixel = "rgb"

[theme]
# Built-in theme name
# Options: dracula, monokai, solarized-dark, solarized-light,
#          nord, gruvbox-dark, gruvbox-light, one-dark, one-light
name = "dracula"

# Or define custom colors (overrides theme)
[theme.colors]
# foreground = "#f8f8f2"
# background = "#282a36"
# cursor = "#f8f8f0"
# selection_background = "#44475a"
# selection_foreground = "#f8f8f2"

# ANSI colors (0-15)
# black = "#21222c"
# red = "#ff5555"
# green = "#50fa7b"
# yellow = "#f1fa8c"
# blue = "#bd93f9"
# magenta = "#ff79c6"
# cyan = "#8be9fd"
# white = "#f8f8f2"
# bright_black = "#6272a4"
# bright_red = "#ff6e6e"
# bright_green = "#69ff94"
# bright_yellow = "#ffffa5"
# bright_blue = "#d6acff"
# bright_magenta = "#ff92df"
# bright_cyan = "#a4ffff"
# bright_white = "#ffffff"

[performance]
# Maximum FPS (0 = unlimited)
fps_limit = 60

# Enable VSync
vsync = true

# Texture atlas cache size (MB)
texture_cache_size = 256

# Enable GPU acceleration
gpu_acceleration = true

# Rendering backend (auto, vulkan, metal, dx12, opengl)
rendering_backend = "auto"

[keybindings]
# Custom keybindings
# Format: "key+modifier" = "action"
"ctrl+shift+c" = "copy"
"ctrl+shift+v" = "paste"
"ctrl+shift+o" = "link_hints"
"ctrl+shift+p" = "command_palette"
"ctrl+shift+f" = "search"
"ctrl+shift+n" = "new_window"
"ctrl+shift+t" = "new_tab"
"ctrl+shift+w" = "close_tab"
"ctrl+shift+q" = "quit"

# Navigation
"ctrl+shift+up" = "scroll_page_up"
"ctrl+shift+down" = "scroll_page_down"
"ctrl+shift+home" = "scroll_to_top"
"ctrl+shift+end" = "scroll_to_bottom"

[plugins]
# Auto-load plugins on startup
auto_load = true

# Plugin search paths
search_paths = [
    "~/.config/scarab/plugins",
    "~/.local/share/scarab/plugins",
    "/usr/share/scarab/plugins"
]

# Enabled plugins (empty = all)
enabled = []

# Disabled plugins
disabled = []

# Plugin-specific config
[plugins.git-status]
enabled = true
update_interval = 1000  # milliseconds
show_dirty_indicator = true

[plugins.notification-monitor]
enabled = true
min_duration = 5000  # Only notify for commands > 5s
sound_enabled = false

[ipc]
# IPC socket path
socket_path = "/tmp/scarab.sock"

# Shared memory path
shmem_path = "/scarab_shm_v1"

# Buffer size (bytes)
buffer_size = 4194304  # 4MB

[session]
# Session persistence
save_on_exit = true

# Session database path
database_path = "~/.local/share/scarab/sessions.db"

# Auto-restore last session
auto_restore = false

# Session history limit
history_limit = 100

[ui]
# Window opacity (0.0 - 1.0)
opacity = 1.0

# Window blur (requires compositor)
blur = false

# Window decorations
decorations = true

# Window padding (pixels)
padding = { x = 4, y = 4 }

# Cursor style (block, underline, beam)
cursor_style = "block"

# Cursor blink (milliseconds, 0 = no blink)
cursor_blink = 500

# Selection style (standard, block)
selection_style = "standard"

[mouse]
# Hide mouse when typing
hide_when_typing = true

# Mouse bindings
[mouse.bindings]
left = "select"
middle = "paste"
right = "extend_selection"
scroll_up = "scroll_up"
scroll_down = "scroll_down"

[logging]
# Log level (trace, debug, info, warn, error)
level = "info"

# Log file path
file = "/tmp/scarab.log"

# Log to stderr
stderr = true

# Enable performance profiling
profiling = false
```

---

## Themes and Colors

### Built-in Themes

Scarab includes several popular themes:

```toml
[theme]
name = "dracula"  # Dark theme with vibrant colors
```

Available themes:
- `dracula` - Dark theme with purple accent
- `monokai` - Classic dark theme
- `solarized-dark` - Precision colors, dark background
- `solarized-light` - Precision colors, light background
- `nord` - Arctic-inspired blue palette
- `gruvbox-dark` - Retro groovy dark theme
- `gruvbox-light` - Retro groovy light theme
- `one-dark` - Atom One Dark
- `one-light` - Atom One Light

### Custom Theme Example

Create a custom theme by defining colors:

```toml
[theme]
name = "custom"  # Optional, for reference

[theme.colors]
# Base colors
foreground = "#e0e0e0"
background = "#1a1a1a"
cursor = "#00ff00"

# Selection
selection_background = "#404040"
selection_foreground = "#ffffff"

# ANSI colors (0-7)
black = "#000000"
red = "#ff0000"
green = "#00ff00"
yellow = "#ffff00"
blue = "#0000ff"
magenta = "#ff00ff"
cyan = "#00ffff"
white = "#ffffff"

# Bright ANSI colors (8-15)
bright_black = "#808080"
bright_red = "#ff8080"
bright_green = "#80ff80"
bright_yellow = "#ffff80"
bright_blue = "#8080ff"
bright_magenta = "#ff80ff"
bright_cyan = "#80ffff"
bright_white = "#ffffff"
```

### Dynamic Theme Switching

Switch themes on the fly with the command palette:

```
Ctrl+Shift+P > theme dracula
Ctrl+Shift+P > theme monokai
```

Or create a keybinding:

```toml
[keybindings]
"ctrl+shift+1" = "theme dracula"
"ctrl+shift+2" = "theme monokai"
"ctrl+shift+3" = "theme solarized-dark"
```

---

## Font Configuration

### Installing Fonts

Scarab uses system-installed fonts. Install your preferred font:

**JetBrains Mono (Recommended):**
```bash
# Ubuntu/Debian
sudo apt install fonts-jetbrains-mono

# Arch Linux
sudo pacman -S ttf-jetbrains-mono

# macOS (via Homebrew)
brew tap homebrew/cask-fonts
brew install --cask font-jetbrains-mono

# Or download from: https://www.jetbrains.com/lp/mono/
```

**Other Popular Monospace Fonts:**
- **Fira Code** - Popular for ligatures
- **Hack** - Designed for source code
- **Source Code Pro** - Adobe's coding font
- **Cascadia Code** - Microsoft's terminal font
- **Inconsolata** - Readable monospace

### Font Configuration

```toml
[font]
family = "JetBrains Mono"
size = 14.0
bold = true
italic = true
```

### Font Features and Ligatures

Enable programming ligatures (if your font supports them):

```toml
[font]
family = "Fira Code"
size = 14.0

# Enable ligatures (OpenType features)
features = ["calt", "liga", "dlig"]
```

Common ligatures:
- `->` becomes →
- `=>` becomes ⇒
- `!=` becomes ≠
- `==` becomes ═
- `>=` becomes ≥

### Font Rendering Quality

Fine-tune font rendering:

```toml
[font]
# Hinting: none, slight, medium, full
# Use "slight" for modern LCD screens
hinting = "slight"

# Subpixel rendering: none, rgb, bgr
# Use "rgb" for most LCD screens
subpixel = "rgb"
```

### Different Font Sizes

You can dynamically change font size:

```toml
[keybindings]
"ctrl+plus" = "increase_font_size"
"ctrl+minus" = "decrease_font_size"
"ctrl+0" = "reset_font_size"
```

---

## Custom Keybindings

### Keybinding Syntax

Format: `"modifier+key" = "action"`

**Modifiers:**
- `ctrl` - Control key
- `shift` - Shift key
- `alt` - Alt/Option key
- `super` - Windows/Command key

**Combine modifiers:**
- `ctrl+shift+c`
- `ctrl+alt+t`
- `super+shift+enter`

### Available Actions

| Action | Description |
|--------|-------------|
| `copy` | Copy selection to clipboard |
| `paste` | Paste from clipboard |
| `search` | Open search overlay |
| `link_hints` | Trigger link hints |
| `command_palette` | Open command palette |
| `new_window` | Create new window |
| `new_tab` | Create new tab |
| `close_tab` | Close current tab |
| `next_tab` | Switch to next tab |
| `previous_tab` | Switch to previous tab |
| `scroll_page_up` | Scroll up one page |
| `scroll_page_down` | Scroll down one page |
| `scroll_to_top` | Jump to scrollback top |
| `scroll_to_bottom` | Jump to scrollback bottom |
| `increase_font_size` | Increase font size |
| `decrease_font_size` | Decrease font size |
| `reset_font_size` | Reset font size to default |
| `toggle_fullscreen` | Toggle fullscreen mode |
| `quit` | Quit Scarab |

### Example Keybindings

```toml
[keybindings]
# Vim-style navigation
"ctrl+h" = "select_left_pane"
"ctrl+j" = "select_below_pane"
"ctrl+k" = "select_above_pane"
"ctrl+l" = "select_right_pane"

# tmux-style prefix key (Ctrl+B)
# Note: Prefix keys require additional config
"ctrl+b" = "prefix"

# Quick theme switching
"alt+1" = "theme dracula"
"alt+2" = "theme monokai"
"alt+3" = "theme nord"

# Custom commands
"ctrl+shift+r" = "reload_config"
"ctrl+shift+d" = "toggle_debug_overlay"
```

### Unbinding Keys

To disable a default keybinding:

```toml
[keybindings]
"ctrl+shift+q" = "none"  # Disable quit shortcut
```

---

## Performance Tuning

### GPU Acceleration

Scarab uses Bevy for GPU-accelerated rendering. Optimize for your hardware:

```toml
[performance]
# Limit FPS to save power
fps_limit = 60  # 0 = unlimited

# Enable VSync (reduces tearing)
vsync = true

# Enable GPU acceleration
gpu_acceleration = true

# Rendering backend
# Options: auto, vulkan, metal, dx12, opengl
rendering_backend = "auto"
```

### Recommended Settings by Hardware

**High-End Desktop (Gaming PC):**
```toml
[performance]
fps_limit = 144  # or your monitor's refresh rate
vsync = true
texture_cache_size = 512
rendering_backend = "vulkan"
```

**Mid-Range Laptop:**
```toml
[performance]
fps_limit = 60
vsync = true
texture_cache_size = 256
rendering_backend = "auto"
```

**Low-End/Battery Saving:**
```toml
[performance]
fps_limit = 30
vsync = true
texture_cache_size = 128
rendering_backend = "opengl"
```

### Profiling Performance

Enable profiling to identify bottlenecks:

```toml
[logging]
profiling = true
```

Then check logs:
```bash
tail -f /tmp/scarab.log | grep PERF
```

---

## Plugin Configuration

### Managing Plugins

Control which plugins load:

```toml
[plugins]
# Auto-load all plugins
auto_load = true

# Or specify enabled plugins
enabled = [
    "git-status",
    "notification-monitor",
    "link-hints"
]

# Disable specific plugins
disabled = [
    "slow-plugin",
    "buggy-plugin"
]
```

### Plugin-Specific Settings

Each plugin can have its own config section:

```toml
[plugins.git-status]
enabled = true
update_interval = 1000  # Update every second
show_dirty_indicator = true
position = "top-right"

[plugins.notification-monitor]
enabled = true
min_duration = 5000  # Only notify for commands > 5s
sound_enabled = false
notification_style = "native"  # or "overlay"

[plugins.link-hints]
enabled = true
key_sequence = "home-row"  # or "alphabet", "numbers"
background_color = "#ffff00"
foreground_color = "#000000"
```

### Plugin Search Paths

Add custom plugin directories:

```toml
[plugins]
search_paths = [
    "~/.config/scarab/plugins",           # User plugins
    "~/projects/scarab-plugins",           # Development
    "~/.local/share/scarab/plugins",       # System
    "/usr/share/scarab/plugins"            # Global
]
```

---

## Advanced Configuration

### Per-Directory Overrides

Create `.scarab.toml` in any directory to override settings:

```toml
# ~/projects/rust-project/.scarab.toml

[terminal]
shell = "/bin/bash"  # Use bash for this project
working_directory = "~/projects/rust-project"

[theme]
name = "gruvbox-dark"  # Different theme for coding

[plugins]
enabled = ["git-status", "rust-analyzer"]
```

### Environment Variables

Scarab respects these environment variables:

| Variable | Default | Description |
|----------|---------|-------------|
| `SCARAB_CONFIG` | `~/.config/scarab/config.toml` | Config file path |
| `SCARAB_PLUGINS` | `~/.config/scarab/plugins` | Plugin directory |
| `SCARAB_LOG_LEVEL` | `info` | Logging level |
| `SHELL` | `/bin/bash` | Default shell |

Example:
```bash
export SCARAB_CONFIG=~/my-scarab-config.toml
export SCARAB_LOG_LEVEL=debug
scarab-client
```

### Configuration Validation

Validate your config before applying:

```bash
# Daemon will validate on startup
cargo run -p scarab-daemon -- --validate-config

# Or use command palette
Ctrl+Shift+P > config validate
```

---

## Configuration Examples

### Minimal Configuration

```toml
# Minimal config - use all defaults
[terminal]
columns = 100
rows = 30

[font]
family = "Monospace"
size = 12.0
```

### Power User Configuration

```toml
# Power user config with advanced features

[terminal]
columns = 200
rows = 60
scrollback = 100000
shell = "/bin/zsh"
shell_args = ["-l"]

[font]
family = "Fira Code"
size = 13.0
features = ["calt", "liga", "dlig"]
hinting = "slight"
subpixel = "rgb"

[theme]
name = "dracula"

[performance]
fps_limit = 144
vsync = true
texture_cache_size = 512
rendering_backend = "vulkan"

[plugins]
auto_load = true
enabled = ["git-status", "notification-monitor", "link-hints"]

[keybindings]
"ctrl+shift+o" = "link_hints"
"ctrl+shift+p" = "command_palette"
"ctrl+h" = "select_left_pane"
"ctrl+l" = "select_right_pane"

[ui]
opacity = 0.95
cursor_blink = 500
padding = { x = 8, y = 8 }
```

---

## Next Steps

- **[Workflow Integration](./03-workflows.md)** - Integrate Scarab into your development workflow
- **[Plugin Development](./03-plugin-development.md)** - Create custom plugins
- **[API Reference](../../docs/api/)** - Detailed API documentation

---

**Back to:** [Getting Started](./01-getting-started.md) | [README](../../README.md)
