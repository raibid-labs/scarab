# Migrating from iTerm2 to Scarab

Guide for iTerm2 users transitioning to Scarab Terminal on macOS (and Linux).

## Quick Comparison

| Feature | iTerm2 | Scarab | Notes |
|---------|--------|--------|-------|
| **Platform** | macOS only | Linux (macOS planned Phase 7) | |
| **Performance** | Very Good | Excellent | GPU-accelerated |
| **Tabs** | ✅ | ✅ | Both support tabs |
| **Splits** | ✅ | 🔄 Coming | Use tmux for now |
| **Profiles** | ✅ | ✅ Sessions | Similar concept |
| **Triggers** | ✅ | ✅ Plugins | Scarab uses F# plugins |
| **Shell Integration** | ✅ | 🔄 Coming | Basic support via plugins |
| **Search** | ✅ | ✅ | Both have regex support |
| **Hotkey Window** | ✅ | 🔄 Planned | Visor-style dropdown |
| **Status Bar** | ✅ | 🔄 Plugin-based | Use overlays |
| **Python API** | ✅ | ✅ F# API | Scarab uses Fusabi (F#) |
| **Session Restoration** | ✅ | ✅ | Scarab uses SQLite |
| **tmux Integration** | ✅ | ✅ | Works inside Scarab |

---

## Important Note: macOS Support

**Current Status**: Scarab currently supports **Linux only**.

**macOS Support**: Planned for **Phase 7** (Q3 2025).

**For macOS Users**:
- This guide prepares you for future macOS release
- You can test on Linux VM or dual-boot system
- Configuration will be compatible across platforms

**Stay Updated**: Watch https://github.com/raibid-labs/scarab for macOS announcements.

---

## Feature Mapping

### Profiles vs Sessions

**iTerm2 Profiles**:
```
Preferences → Profiles
- Default
- Development
- SSH Remote
- Personal
```

**Scarab Sessions**:
```toml
# Each session is like an iTerm2 profile instance
[sessions]
restore_on_startup = true
auto_save_interval = 300
```

**Create Sessions**:
```bash
# iTerm2: Cmd+T with profile
# Scarab:
scarab-client --new-session "development"
scarab-client --new-session "ssh-remote"

# Or use command palette: Ctrl+Shift+P → "New Session"
```

### Triggers vs Plugins

**iTerm2 Triggers** (Preferences → Profiles → Advanced → Triggers):
```
Regex: ERROR
Action: Highlight text
Color: Red
```

**Scarab Plugin**:
```fsharp
// ~/.config/scarab/plugins/highlight-errors.fsx
open Scarab.PluginApi

let on_output ctx line =
    async {
        if line.Contains("ERROR") then
            // Highlight in red
            ctx.QueueCommand(RemoteCommand.DrawOverlay {
                Id = 1000UL
                X = 0us
                Y = (ctx.GetCursorY())
                Text = line
                Style = {
                    Fg = 0xFFFFFFFFu
                    Bg = 0xFF0000FFu
                    ZIndex = 100.0f
                }
            })
        return Ok line
    }

Plugin.Register {
    Metadata = { (*...*) }
    OnOutput = Some on_output
    (*...*)
}
```

**Enable**:
```toml
[plugins]
enabled = ["highlight-errors"]
```

### Shell Integration

**iTerm2**: Automatic shell integration with `imgcat`, `it2dl`, etc.

**Scarab**: Plugin-based integration (in development):

```fsharp
// Shell integration plugin
let on_post_command ctx cmd =
    async {
        // Track command duration
        let duration = ctx.GetCommandDuration()
        if duration > 5000 then  // 5 seconds
            ctx.ShowNotification($"Command took {duration}ms")

        // Update status bar
        ctx.QueueCommand(RemoteCommand.DrawOverlay {
            Id = 2000UL
            X = (ctx.GetCols() - 20us)
            Y = 0us
            Text = $"⏱ {duration}ms"
            Style = { (*...*) }
        })

        return Ok ()
    }
```

### Hotkey Window (Visor)

**iTerm2**: Preferences → Keys → Hotkey Window

**Scarab**: Planned feature (Phase 8). Workaround:

```bash
# Use window manager shortcut
# i3/sway config:
bindsym $mod+grave exec --no-startup-id scarab-client --dropdown

# Or script:
#!/bin/bash
# toggle-scarab.sh
if wmctrl -l | grep -q "Scarab"; then
    wmctrl -c "Scarab"
else
    scarab-client &
fi
```

### Status Bar Components

**iTerm2**: Status Bar Configuration → Add Component

**Scarab**: Plugin-based overlays:

```fsharp
// ~/.config/scarab/plugins/status-bar.fsx
let update_status_bar ctx =
    async {
        // Git branch
        let branch = get_git_branch()
        ctx.QueueCommand(RemoteCommand.DrawOverlay {
            Id = 3000UL
            X = 0us
            Y = 0us
            Text = $"  {branch}"
            Style = { (*...*) }
        })

        // CPU usage
        let cpu = get_cpu_usage()
        ctx.QueueCommand(RemoteCommand.DrawOverlay {
            Id = 3001UL
            X = 20us
            Y = 0us
            Text = $" {cpu}%"
            Style = { (*...*) }
        })

        // Time
        let time = DateTime.Now.ToString("HH:mm")
        let cols = ctx.GetCols()
        ctx.QueueCommand(RemoteCommand.DrawOverlay {
            Id = 3002UL
            X = (cols - 10us)
            Y = 0us
            Text = $" {time}"
            Style = { (*...*) }
        })
    }

// Update every second
let on_load ctx =
    async {
        ctx.SetInterval(update_status_bar, 1000)
        return Ok ()
    }
```

---

## Configuration Conversion

### Color Schemes

**iTerm2**: Preferences → Profiles → Colors → Color Presets

**Export iTerm2 colors**:
```bash
# iTerm2 stores colors in plist
# Location: ~/Library/Preferences/com.googlecode.iterm2.plist

# Or export preset:
# Preferences → Profiles → Colors → Color Presets → Export
```

**Convert to Scarab**:
```toml
# ~/.config/scarab/config.toml
[colors]
foreground = "#d4d4d4"  # iTerm2: Foreground Color
background = "#1e1e1e"  # iTerm2: Background Color
cursor = "#d4d4d4"      # iTerm2: Cursor Color
selection_background = "#264f78"  # iTerm2: Selection Color

[colors.palette]
# ANSI Colors (iTerm2 → Scarab)
black = "#000000"        # iTerm2: ANSI Black Color
red = "#cd3131"          # iTerm2: ANSI Red Color
green = "#0dbc79"        # iTerm2: ANSI Green Color
yellow = "#e5e510"       # iTerm2: ANSI Yellow Color
blue = "#2472c8"         # iTerm2: ANSI Blue Color
magenta = "#bc3fbc"      # iTerm2: ANSI Magenta Color
cyan = "#11a8cd"         # iTerm2: ANSI Cyan Color
white = "#e5e5e5"        # iTerm2: ANSI White Color

# Bright ANSI Colors
bright_black = "#666666"   # iTerm2: ANSI Bright Black Color
bright_red = "#f14c4c"     # iTerm2: ANSI Bright Red Color
bright_green = "#23d18b"   # iTerm2: ANSI Bright Green Color
bright_yellow = "#f5f543"  # iTerm2: ANSI Bright Yellow Color
bright_blue = "#3b8eea"    # iTerm2: ANSI Bright Blue Color
bright_magenta = "#d670d6" # iTerm2: ANSI Bright Magenta Color
bright_cyan = "#29b8db"    # iTerm2: ANSI Bright Cyan Color
bright_white = "#ffffff"   # iTerm2: ANSI Bright White Color
```

### Font Settings

**iTerm2**: Preferences → Profiles → Text → Font

**Scarab**:
```toml
[font]
# iTerm2: Font → Family
family = "SF Mono"  # Or "Menlo", "Monaco"

# iTerm2: Font → Size
size = 14.0

# iTerm2: Text → Vertical spacing
line_height = 1.2

# iTerm2: Text → Use ligatures
enable_ligatures = true

# iTerm2: Text → Anti-aliasing (macOS only)
use_thin_strokes = true  # For Retina displays
```

### Keybindings

**iTerm2**: Preferences → Keys → Key Bindings

**Common iTerm2 bindings** → **Scarab**:

```toml
[keybindings]
# iTerm2: ⌘C (Copy)
copy_mode = "Cmd+C"

# iTerm2: ⌘V (Paste)
paste = "Cmd+V"

# iTerm2: ⌘F (Find)
search = "Cmd+F"

# iTerm2: ⌘T (New Tab)
new_window = "Cmd+T"

# iTerm2: ⌘W (Close Tab)
close_window = "Cmd+W"

# iTerm2: ⌘N (New Window)
new_window = "Cmd+N"

# iTerm2: ⌘K (Clear Buffer)
[keybindings.custom]
"clear_scrollback" = "Cmd+K"

# iTerm2: ⌘+ (Increase Font)
"increase_font_size" = "Cmd+Plus"

# iTerm2: ⌘- (Decrease Font)
"decrease_font_size" = "Cmd+Minus"

# iTerm2: ⌘0 (Reset Font)
"reset_font_size" = "Cmd+0"

# iTerm2: ⌘D (Split Vertically)
"split_vertical" = "Cmd+D"  # Coming soon

# iTerm2: ⌘⇧D (Split Horizontally)
"split_horizontal" = "Cmd+Shift+D"  # Coming soon

# iTerm2: ⌘] (Next Pane)
"focus_next_pane" = "Cmd+]"  # Coming soon

# iTerm2: ⌘[ (Previous Pane)
"focus_prev_pane" = "Cmd+["  # Coming soon

# iTerm2: ⌘⌥→ (Next Tab)
"next_tab" = "Cmd+Alt+Right"

# iTerm2: ⌘⌥← (Previous Tab)
"prev_tab" = "Cmd+Alt+Left"
```

### Window Settings

**iTerm2**: Preferences → Profiles → Window

```toml
# iTerm2: Transparency
[colors]
opacity = 0.95  # 0.0-1.0 (iTerm2: 0-100%)

# iTerm2: Blur
# Not yet supported in Scarab

# iTerm2: Columns × Rows
[terminal]
columns = 120
rows = 40

# iTerm2: Screen (position)
# Not configurable in Scarab (use WM rules)
```

---

## iTerm2 Features Not in Scarab

### Native macOS Integration

**Not Available** (until Phase 7):
- Touch Bar support
- macOS Services integration
- Notification Center
- Quick Look integration
- Cocoa text system

**Workarounds**:
- Use standard terminal features
- Plugins for custom notifications
- X11/Wayland on macOS (experimental)

### Advanced Features

| Feature | Status | Alternative |
|---------|--------|-------------|
| **Coprocesses** | ❌ Not planned | Use plugins for automation |
| **Captured Output** | 🔄 Plugin-based | Write output filter plugin |
| **Timestamps** | 🔄 Plugin-based | Write overlay plugin |
| **Composer** | ❌ Not planned | Use clipboard |
| **Python API** | ✅ F# API instead | More powerful plugin system |
| **Shell Integration** | 🔄 Partial | Plugin-based integration |
| **Instant Replay** | ❌ Not planned | Use `script` command |

### Unique iTerm2 Features

**Badges**:
- iTerm2: Show custom badge in corner
- Scarab: Use overlay plugin (similar)

**Automatic Profile Switching**:
- iTerm2: Switch profile based on hostname/user
- Scarab: Use session management + plugins

**Password Manager**:
- iTerm2: Built-in password manager
- Scarab: Use system password manager (1Password, etc.)

---

## Scarab Features Not in iTerm2

### Split Architecture

**Daemon + Client** allows:
```bash
# Client crash doesn't kill terminal
# On iTerm2: Lost if app crashes
# On Scarab: Daemon keeps running

# Reconnect to session
scarab-client --attach my-session
```

### Plugin System (Fusabi)

**Type-safe F# plugins**:
```fsharp
// iTerm2: Python API (runtime errors possible)
// Scarab: F# with compile-time checking

// Example: Git integration
let on_post_command ctx cmd =
    async {
        if cmd.StartsWith("git") then
            let status = run_git_status()
            ctx.UpdateOverlay(status)
        return Ok ()
    }
```

### Cross-Platform

**Scarab** (when complete):
- Linux ✅
- macOS 🔄 Planned
- Windows 🔄 Planned

**iTerm2**: macOS only

### Zero-Copy IPC

**Performance benefits**:
- Shared memory between daemon and client
- No data copying overhead
- <1μs latency for updates

---

## Migration Strategy

### For Current macOS Users

**Option 1: Wait for macOS Support** (recommended)
```bash
# Stay on iTerm2 until Scarab Phase 7
# Watch: https://github.com/raibid-labs/scarab/releases
```

**Option 2: Dual Boot / VM**
```bash
# Install Linux on:
# - Virtual machine (Parallels, VMware, VirtualBox)
# - Dual boot partition
# - Separate Linux machine

# Test Scarab on Linux
# Prepare configuration for future macOS release
```

**Option 3: Contribute**
```bash
# Help port Scarab to macOS
# See: docs/CONTRIBUTING.md
# macOS-specific work needed:
# - Metal backend (Bevy already supports)
# - macOS keyboard handling
# - .app bundle creation
# - Codesigning
```

### For Linux Users (Coming from iTerm2)

**You're in luck!** Scarab works on Linux now.

```bash
# Install Scarab
git clone https://github.com/raibid-labs/scarab.git
cd scarab
cargo build --release

# Convert iTerm2 config (manually)
# Use config examples from this guide

# Start using Scarab
./target/release/scarab-daemon &
./target/release/scarab-client
```

---

## Configuration Template for ex-iTerm2 Users

Complete Scarab config mimicking iTerm2 defaults:

```toml
# ~/.config/scarab/config.toml
# iTerm2-inspired configuration

[terminal]
default_shell = "/bin/zsh"
columns = 80
rows = 24
scrollback_lines = 1000  # iTerm2 default: unlimited (use 10000 for Scarab)
auto_scroll = true

[font]
family = "SF Mono"  # macOS default
size = 12.0         # iTerm2 default
line_height = 1.2
use_thin_strokes = true  # For Retina displays
enable_ligatures = true

[colors]
# iTerm2 "Solarized Dark" theme
theme = "solarized-dark"
opacity = 1.0

# Or custom colors:
# foreground = "#839496"
# background = "#002b36"
# cursor = "#839496"
# selection_background = "#073642"

[keybindings]
# iTerm2-style keybindings (macOS)
copy_mode = "Cmd+C"
paste = "Cmd+V"
search = "Cmd+F"
command_palette = "Cmd+Shift+P"
new_window = "Cmd+T"
close_window = "Cmd+W"
next_tab = "Cmd+Alt+Right"
prev_tab = "Cmd+Alt+Left"

[keybindings.custom]
"increase_font_size" = "Cmd+Plus"
"decrease_font_size" = "Cmd+Minus"
"reset_font_size" = "Cmd+0"
"clear_scrollback" = "Cmd+K"
"toggle_fullscreen" = "Cmd+Ctrl+F"
"split_vertical" = "Cmd+D"        # Coming soon
"split_horizontal" = "Cmd+Shift+D"  # Coming soon

[ui]
cursor_style = "block"  # iTerm2 default
cursor_blink = true
cursor_blink_interval = 750
animations = true
smooth_scroll = true
show_tabs = true
tab_position = "top"

[plugins]
enabled = [
    "shell-integration",  # iTerm2-style shell integration
    "status-bar",         # Status bar components
    "triggers",           # iTerm2-style triggers
]

[plugins.config.status-bar]
components = [
    { type = "git-branch", position = "left" },
    { type = "cpu", position = "left" },
    { type = "time", position = "right" },
]

[sessions]
restore_on_startup = true   # Like iTerm2 "Restore Windows"
auto_save_interval = 300
save_scrollback = true
```

---

## Common Questions

### Will my iTerm2 scripts work?

**No**, Scarab uses F# (Fusabi), not Python.

**Migration Path**:
1. Identify what your Python script does
2. Write equivalent F# plugin
3. Or: Call Python script from F# plugin:

```fsharp
open System.Diagnostics

let run_python_script script_path =
    let psi = ProcessStartInfo("python3", script_path)
    psi.RedirectStandardOutput <- true
    use proc = Process.Start(psi)
    let output = proc.StandardOutput.ReadToEnd()
    proc.WaitForExit()
    output
```

### Can I keep using iTerm2?

**Yes!** No need to uninstall iTerm2.

- Use iTerm2 for production work
- Use Scarab for development/testing
- Switch when comfortable

### What about tmux integration?

**iTerm2's tmux integration**: Special protocol for tight tmux integration.

**Scarab**: Standard tmux works inside Scarab, but no special integration yet.

**Recommendation**: Use Scarab's built-in tabs/sessions instead of tmux when macOS support arrives.

---

## Reporting Issues

Found something that works in iTerm2 but not Scarab?

**Report at**: https://github.com/raibid-labs/scarab/issues

**Include**:
- iTerm2 version and settings
- Expected behavior (what iTerm2 does)
- Actual behavior (what Scarab does)
- Config files

**macOS-specific issues**: Tag with `macos` label (for future reference).

---

## See Also

- [Scarab Configuration Reference](../reference/configuration.md)
- [Plugin Development Guide](../development/plugins.md)
- [Migration from Alacritty](./from-alacritty.md)
- [Migration from GNOME Terminal](./from-gnome-terminal.md)
- [iTerm2 Documentation](https://iterm2.com/documentation.html)
