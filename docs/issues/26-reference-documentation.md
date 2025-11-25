# Issue #26: Complete Reference Documentation

## 🎯 Goal
Fill gaps in reference documentation with complete configuration guide, keybindings reference, and troubleshooting guide.

## 🐛 Problem

Current documentation has significant gaps:

### Configuration Reference
- ⚠️ Many TOML options have PLACEHOLDER descriptions
- ❌ Missing examples for advanced configuration
- ❌ No validation rules documented
- ❌ Missing default values

### Keybindings Reference
- ⚠️ Mentions features not yet implemented
- ❌ No comprehensive list of all shortcuts
- ❌ Missing platform-specific variations (macOS vs Linux)
- ❌ No customization examples

### Missing Sections
- ❌ No troubleshooting guide
- ❌ No FAQ section
- ❌ No performance tuning guide
- ❌ No migration guides from other terminals

## 💡 Proposed Solution

Create comprehensive reference documentation:

### 1. Complete Configuration Reference
**File**: `docs/reference/configuration.md`

### 2. Complete Keybindings Reference
**File**: `docs/reference/keybindings.md`

### 3. Troubleshooting Guide
**File**: `docs/reference/troubleshooting.md`

### 4. Performance Tuning Guide
**File**: `docs/reference/performance.md`

### 5. FAQ Document
**File**: `docs/reference/faq.md`

### 6. Migration Guides
**Files**:
- `docs/migration/from-alacritty.md`
- `docs/migration/from-iterm2.md`
- `docs/migration/from-gnome-terminal.md`

## 📋 Implementation Tasks

### Phase 1: Configuration Reference (1 day)

**File**: `docs/reference/configuration.md`

Complete documentation for all TOML sections:

```toml
# ~/.config/scarab/config.toml

[terminal]
# Shell to use (default: $SHELL environment variable)
shell = "/bin/zsh"

# Font family (default: monospace)
font_family = "SF Mono"

# Font size in points (default: 14.0, min: 6.0, max: 72.0)
font_size = 14.0

# Line height multiplier (default: 1.0, range: 0.8-2.0)
line_height = 1.2

# Enable font ligatures (default: true)
# Requires font with ligature support (Fira Code, JetBrains Mono)
font_ligatures = true

# Scrollback lines (default: 10000, max: 100000)
scrollback_lines = 10000

# Working directory for new sessions
# Options: "home", "inherit", or absolute path
# Default: "inherit" (uses current directory)
working_directory = "inherit"

[theme]
# Theme name (see ~/.config/scarab/themes/ for available themes)
name = "default"

# Or define custom colors inline
[theme.colors]
background = "#1e1e1e"
foreground = "#d4d4d4"
cursor = "#ffffff"
selection = "#264f78"

# Normal colors
black = "#000000"
red = "#cd3131"
green = "#0dbc79"
yellow = "#e5e510"
blue = "#2472c8"
magenta = "#bc3fbc"
cyan = "#11a8cd"
white = "#e5e5e5"

# Bright colors
bright_black = "#666666"
bright_red = "#f14c4c"
bright_green = "#23d18b"
bright_yellow = "#f5f543"
bright_blue = "#3b8eea"
bright_magenta = "#d670d6"
bright_cyan = "#29b8db"
bright_white = "#ffffff"

[gpu]
# GPU backend: "auto", "metal" (macOS), "vulkan" (Linux/Windows)
# Default: "auto" (detects best option)
backend = "auto"

# Enable vsync (default: true)
vsync = true

# Target FPS (default: 60, max: 144)
target_fps = 60

# MSAA samples (1, 2, 4, 8, 16)
# Higher = better quality but slower
# Default: 4
msaa_samples = 4

[plugins]
# Enable plugin system (default: true)
enabled = true

# Plugin search paths (checked in order)
search_paths = [
    "~/.config/scarab/plugins",
    "~/.local/share/scarab/plugins"
]

# Auto-load plugins on startup
auto_load = ["scarab-nav", "scarab-palette", "scarab-session"]

# Plugin logging level: "error", "warn", "info", "debug"
log_level = "info"

[keybindings]
# Format: "modifiers+key" = "action"
# Modifiers: Ctrl, Shift, Alt, Super (Cmd on macOS)

# Copy/paste
"Ctrl+Shift+c" = "copy"
"Ctrl+Shift+v" = "paste"

# Search
"Ctrl+f" = "search"

# Scrollback
"Shift+PageUp" = "scroll_page_up"
"Shift+PageDown" = "scroll_page_down"
"Shift+Home" = "scroll_to_top"
"Shift+End" = "scroll_to_bottom"

# Command palette
"Ctrl+Shift+p" = "command_palette"

# Link hints
"Ctrl+Shift+o" = "link_hints"

# Font size
"Ctrl+Plus" = "increase_font_size"
"Ctrl+Minus" = "decrease_font_size"
"Ctrl+0" = "reset_font_size"

[ipc]
# Shared memory size in MB (default: 16)
shm_size = 16

# Socket path (default: /tmp/scarab-{uid}.sock)
socket_path = "/tmp/scarab.sock"

[logging]
# Log level: "error", "warn", "info", "debug", "trace"
level = "info"

# Log file path (default: ~/.local/share/scarab/scarab.log)
file = "~/.local/share/scarab/scarab.log"

# Max log file size in MB (default: 10)
max_size = 10

# Number of rotated log files to keep (default: 3)
max_backups = 3
```

### Phase 2: Keybindings Reference (half day)

**File**: `docs/reference/keybindings.md`

Complete table of all keybindings:

| Action | macOS | Linux/Windows | Customizable |
|--------|-------|---------------|--------------|
| **Editing** |
| Copy | `Cmd+C` | `Ctrl+Shift+C` | ✅ |
| Paste | `Cmd+V` | `Ctrl+Shift+V` | ✅ |
| Select All | `Cmd+A` | `Ctrl+Shift+A` | ✅ |
| **Navigation** |
| Scroll Page Up | `Shift+PageUp` | `Shift+PageUp` | ✅ |
| Scroll Page Down | `Shift+PageDown` | `Shift+PageDown` | ✅ |
| Scroll to Top | `Shift+Home` | `Shift+Home` | ✅ |
| Scroll to Bottom | `Shift+End` | `Shift+End` | ✅ |
| **Search** |
| Find | `Cmd+F` | `Ctrl+F` | ✅ |
| Find Next | `Enter` | `Enter` | ✅ |
| Find Previous | `Shift+Enter` | `Shift+Enter` | ✅ |
| **Features** |
| Command Palette | `Cmd+Shift+P` | `Ctrl+Shift+P` | ✅ |
| Link Hints | `Cmd+Shift+O` | `Ctrl+Shift+O` | ✅ |
| **View** |
| Increase Font | `Cmd++` | `Ctrl++` | ✅ |
| Decrease Font | `Cmd+-` | `Ctrl+-` | ✅ |
| Reset Font | `Cmd+0` | `Ctrl+0` | ✅ |
| Toggle Fullscreen | `Cmd+Enter` | `F11` | ✅ |

### Phase 3: Troubleshooting Guide (1 day)

**File**: `docs/reference/troubleshooting.md`

Common issues and solutions:

#### Installation Issues
- Command not found after install
- Permission denied errors
- Missing dependencies

#### Performance Issues
- High CPU usage
- Stuttering/lag
- Memory leaks
- GPU not being used

#### Display Issues
- Fonts not rendering correctly
- Colors appear wrong
- Ligatures not working
- Emoji not displaying

#### Plugin Issues
- Plugins not loading
- Plugin errors in logs
- Fusabi compilation failures

#### IPC/Connection Issues
- Daemon not starting
- Client can't connect to daemon
- Shared memory errors

### Phase 4: Performance Tuning Guide (half day)

**File**: `docs/reference/performance.md`

Topics:
- GPU backend selection
- VTE cache tuning
- Scrollback buffer limits
- Font rendering optimization
- Plugin performance impact
- Benchmarking tools

### Phase 5: FAQ (half day)

**File**: `docs/reference/faq.md`

Common questions:
- What makes Scarab different from Alacritty/iTerm2?
- Why split architecture (daemon + client)?
- What is Fusabi?
- Can I use my existing shell config?
- Does it work on Wayland?
- How do I create a plugin?
- Can I sync config across machines?

### Phase 6: Migration Guides (1 day)

#### From Alacritty
**File**: `docs/migration/from-alacritty.md`

- Config file comparison
- Keybinding translation
- Missing features (what to expect)
- Configuration conversion script

#### From iTerm2
**File**: `docs/migration/from-iterm2.md`

- Feature parity table
- Unique iTerm2 features and alternatives
- Theme conversion
- Profile migration

#### From GNOME Terminal
**File**: `docs/migration/from-gnome-terminal.md`

- Settings mapping
- Profile import
- Keyboard shortcuts

## 🎨 Documentation Structure

```
docs/
├── reference/
│   ├── configuration.md          ← Complete TOML reference
│   ├── keybindings.md           ← All keyboard shortcuts
│   ├── troubleshooting.md       ← Common issues + solutions
│   ├── performance.md           ← Tuning guide
│   └── faq.md                   ← Frequently asked questions
└── migration/
    ├── from-alacritty.md        ← Alacritty migration guide
    ├── from-iterm2.md           ← iTerm2 migration guide
    └── from-gnome-terminal.md   ← GNOME Terminal migration guide
```

## 🧪 Validation

### Automated Checks
- [ ] All TOML options have descriptions
- [ ] All default values documented
- [ ] All keybindings listed
- [ ] Links work
- [ ] Code examples are valid

### Manual Review
- [ ] Config examples work when copied
- [ ] Troubleshooting steps solve issues
- [ ] Migration guides are accurate
- [ ] FAQ answers are clear

## 📊 Success Criteria

- [ ] Zero PLACEHOLDER values remain
- [ ] Every config option documented
- [ ] All keybindings listed with platform variants
- [ ] Troubleshooting covers top 20 issues
- [ ] Migration guides for 3 popular terminals
- [ ] FAQ answers 15+ common questions

## 🔗 Related Issues

- Issue #25: Interactive Tutorial (references this documentation)
- Issue #27: Plugin Development Documentation (separate focus)

---

**Priority**: 🟡 HIGH
**Effort**: 2 days
**Assignee**: Technical Writer
