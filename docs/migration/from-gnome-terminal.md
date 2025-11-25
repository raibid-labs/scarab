# Migrating from GNOME Terminal to Scarab

Guide for GNOME Terminal users switching to Scarab on Linux.

## Quick Comparison

| Feature | GNOME Terminal | Scarab | Notes |
|---------|----------------|--------|-------|
| **Platform** | Linux | Linux (macOS/Win planned) | Both Linux-native |
| **Performance** | Good | Excellent | Scarab is GPU-accelerated |
| **Configuration** | GUI + dconf | TOML file | Scarab easier to version control |
| **Tabs** | ✅ | ✅ | Both support tabs |
| **Splits** | ❌ | 🔄 Coming | Use tmux for now |
| **Profiles** | ✅ | ✅ Sessions | Similar concept |
| **Transparency** | ✅ | ✅ | Both support transparency |
| **Color Schemes** | ✅ Built-in | ✅ Themes + custom | |
| **Keyboard Shortcuts** | ✅ Configurable | ✅ Configurable | Different format |
| **VTE Compatibility** | ✅ (Native) | ✅ (Alacritty VTE) | Both excellent |
| **Desktop Integration** | ✅ GNOME only | ✅ All DEs | Scarab more universal |
| **Session Persistence** | ❌ | ✅ SQLite | Scarab advantage |
| **Extensibility** | ❌ Limited | ✅ F# Plugins | Scarab major advantage |

---

## Why Switch?

### Performance

**GNOME Terminal**: CPU-based rendering via GTK.

**Scarab**: GPU-accelerated rendering via Bevy + wgpu.

**Real-world impact**:
```bash
# Large output (e.g., cat large-file.txt)
# GNOME Terminal: Stutters, high CPU
# Scarab: Smooth 60 FPS, low CPU
```

### Configuration Management

**GNOME Terminal**: Settings in dconf (binary format).
```bash
dconf dump /org/gnome/terminal/ > gnome-terminal.conf
# Not human-readable, hard to version control
```

**Scarab**: Plain TOML file.
```toml
# ~/.config/scarab/config.toml
# Easy to read, edit, and version with git
[font]
family = "DejaVu Sans Mono"
size = 12.0
```

### Desktop Environment Independence

**GNOME Terminal**: Designed for GNOME, looks out of place elsewhere.

**Scarab**: Works on any DE/WM:
- GNOME
- KDE Plasma
- XFCE
- i3/sway
- dwm/bspwm
- Standalone (no DE)

### Session Persistence

**GNOME Terminal**: Sessions lost on crash/logout.

**Scarab**: Daemon preserves terminal state.
```bash
# GNOME Terminal crashes → lost everything
# Scarab client crashes → daemon preserves state
scarab-client  # Reconnect, state intact
```

---

## Configuration Migration

### Export GNOME Terminal Settings

```bash
# Export all settings
dconf dump /org/gnome/terminal/ > ~/gnome-terminal.dconf

# Export specific profile
PROFILE_ID=$(gsettings get org.gnome.Terminal.ProfilesList default | tr -d "'")
dconf dump /org/gnome/terminal/legacy/profiles:/:$PROFILE_ID/ > ~/profile.dconf
```

### Profile Mapping

**GNOME Terminal Profile** → **Scarab Config**

#### Font Settings

**GNOME Terminal** (dconf):
```ini
[/]
font='Monospace 12'
use-system-font=false
```

**Scarab** (TOML):
```toml
[font]
family = "Monospace"  # Or "DejaVu Sans Mono", "Liberation Mono"
size = 12.0
```

#### Colors

**GNOME Terminal** (dconf):
```ini
[/]
use-theme-colors=false
foreground-color='rgb(211,215,207)'
background-color='rgb(46,52,54)'
palette=['rgb(0,0,0)', 'rgb(204,0,0)', 'rgb(78,154,6)', ...]
```

**Scarab** (TOML):
```toml
[colors]
foreground = "#d3d7cf"
background = "#2e3436"

[colors.palette]
black = "#000000"
red = "#cc0000"
green = "#4e9a06"
yellow = "#c4a000"
blue = "#3465a4"
magenta = "#75507b"
cyan = "#06989a"
white = "#d3d7cf"

bright_black = "#555753"
bright_red = "#ef2929"
bright_green = "#8ae234"
bright_yellow = "#fce94f"
bright_blue = "#729fcf"
bright_magenta = "#ad7fa8"
bright_cyan = "#34e2e2"
bright_white = "#eeeeec"
```

#### Scrollback

**GNOME Terminal** (dconf):
```ini
[/]
scrollback-lines=10000
scrollback-unlimited=false
scroll-on-output=false
scroll-on-keystroke=true
```

**Scarab** (TOML):
```toml
[terminal]
scrollback_lines = 10000  # Max: 100000
auto_scroll = false       # scroll-on-output
# scroll-on-keystroke: Always enabled in Scarab
```

#### Cursor

**GNOME Terminal** (dconf):
```ini
[/]
cursor-shape='block'
cursor-blink-mode='on'
```

**Scarab** (TOML):
```toml
[ui]
cursor_style = "block"  # Options: "block", "beam", "underline"
cursor_blink = true
cursor_blink_interval = 750  # milliseconds
```

#### Transparency

**GNOME Terminal** (dconf):
```ini
[/]
use-transparent-background=true
background-transparency-percent=15
```

**Scarab** (TOML):
```toml
[colors]
opacity = 0.85  # 100% - 15% = 85%
# Range: 0.0 (fully transparent) to 1.0 (opaque)
```

---

## Automated Conversion Script

```bash
#!/bin/bash
# gnome-terminal2scarab.sh - Convert GNOME Terminal profile to Scarab config

set -e

PROFILE_ID=$(gsettings get org.gnome.Terminal.ProfilesList default | tr -d "'")
DCONF_PATH="/org/gnome/terminal/legacy/profiles:/:$PROFILE_ID/"
SCARAB_CONFIG="$HOME/.config/scarab/config.toml"

echo "Converting GNOME Terminal profile to Scarab config..."

# Helper function to get dconf value
get_dconf() {
    dconf read "${DCONF_PATH}$1" | sed "s/'//g"
}

# Extract font
FONT=$(get_dconf "font")
FONT_FAMILY=$(echo "$FONT" | sed 's/ [0-9]*$//')
FONT_SIZE=$(echo "$FONT" | grep -o '[0-9]*$')

# Extract colors
FG_RGB=$(get_dconf "foreground-color" | sed 's/rgb(\(.*\))/\1/')
BG_RGB=$(get_dconf "background-color" | sed 's/rgb(\(.*\))/\1/')

# Convert RGB to hex
rgb_to_hex() {
    IFS=',' read -r r g b <<< "$1"
    printf "#%02x%02x%02x" "$r" "$g" "$b"
}

FG_HEX=$(rgb_to_hex "$FG_RGB")
BG_HEX=$(rgb_to_hex "$BG_RGB")

# Extract scrollback
SCROLLBACK=$(get_dconf "scrollback-lines")
SCROLL_ON_OUTPUT=$(get_dconf "scroll-on-output")

# Extract cursor
CURSOR_SHAPE=$(get_dconf "cursor-shape")
CURSOR_BLINK=$(get_dconf "cursor-blink-mode")

# Extract transparency
TRANSPARENCY=$(get_dconf "background-transparency-percent")
if [ -n "$TRANSPARENCY" ]; then
    OPACITY=$(awk "BEGIN {print 1 - ($TRANSPARENCY / 100)}")
else
    OPACITY=1.0
fi

# Generate Scarab config
mkdir -p ~/.config/scarab

cat > "$SCARAB_CONFIG" <<EOF
# Converted from GNOME Terminal profile on $(date)
# Profile ID: $PROFILE_ID

[terminal]
default_shell = "/bin/bash"
scrollback_lines = ${SCROLLBACK:-10000}
auto_scroll = $([ "$SCROLL_ON_OUTPUT" = "true" ] && echo "true" || echo "false")
columns = 80
rows = 24

[font]
family = "${FONT_FAMILY:-Monospace}"
size = ${FONT_SIZE:-12}.0
line_height = 1.2
fallback = ["DejaVu Sans Mono", "Liberation Mono", "Noto Color Emoji"]

[colors]
foreground = "$FG_HEX"
background = "$BG_HEX"
opacity = $OPACITY

# Extract full palette from GNOME Terminal
# Note: This requires manual extraction of all 16 colors
# For now, using GNOME Terminal's default "Tango" palette:
[colors.palette]
black = "#000000"
red = "#cc0000"
green = "#4e9a06"
yellow = "#c4a000"
blue = "#3465a4"
magenta = "#75507b"
cyan = "#06989a"
white = "#d3d7cf"

bright_black = "#555753"
bright_red = "#ef2929"
bright_green = "#8ae234"
bright_yellow = "#fce94f"
bright_blue = "#729fcf"
bright_magenta = "#ad7fa8"
bright_cyan = "#34e2e2"
bright_white = "#eeeeec"

[ui]
cursor_style = "${CURSOR_SHAPE:-block}"
cursor_blink = $([ "$CURSOR_BLINK" = "on" ] && echo "true" || echo "false")
cursor_blink_interval = 750

[keybindings]
# GNOME Terminal-style keybindings
copy_mode = "Ctrl+Shift+C"
paste = "Ctrl+Shift+V"
search = "Ctrl+Shift+F"

[keybindings.custom]
"new_tab" = "Ctrl+Shift+T"
"close_tab" = "Ctrl+Shift+W"
"next_tab" = "Ctrl+PageDown"
"prev_tab" = "Ctrl+PageUp"
"increase_font_size" = "Ctrl+Plus"
"decrease_font_size" = "Ctrl+Minus"
"reset_font_size" = "Ctrl+0"

[sessions]
restore_on_startup = false
auto_save_interval = 300
EOF

echo "Conversion complete!"
echo "Config written to: $SCARAB_CONFIG"
echo ""
echo "Note: Color palette uses default Tango theme."
echo "To extract exact colors from your profile, edit $SCARAB_CONFIG manually."
```

**Usage**:
```bash
chmod +x gnome-terminal2scarab.sh
./gnome-terminal2scarab.sh
```

---

## Keybinding Migration

### Default GNOME Terminal Shortcuts

| Action | GNOME Terminal | Scarab |
|--------|----------------|--------|
| **Copy** | `Ctrl+Shift+C` | `Ctrl+Shift+C` ✅ Same |
| **Paste** | `Ctrl+Shift+V` | `Ctrl+Shift+V` ✅ Same |
| **New Tab** | `Ctrl+Shift+T` | `Ctrl+Shift+T` ✅ Same |
| **Close Tab** | `Ctrl+Shift+W` | `Ctrl+Shift+W` ✅ Same |
| **Next Tab** | `Ctrl+PageDown` | `Ctrl+Tab` ⚠️ Different |
| **Previous Tab** | `Ctrl+PageUp` | `Ctrl+Shift+Tab` ⚠️ Different |
| **Switch to Tab 1-9** | `Alt+1-9` | `Ctrl+1-9` ⚠️ Different |
| **New Window** | `Ctrl+Shift+N` | `Ctrl+Shift+N` ✅ Same |
| **Find** | `Ctrl+Shift+F` | `Ctrl+F` ⚠️ Different |
| **Zoom In** | `Ctrl++` | `Ctrl++` ✅ Same |
| **Zoom Out** | `Ctrl+-` | `Ctrl+-` ✅ Same |
| **Reset Zoom** | `Ctrl+0` | `Ctrl+0` ✅ Same |
| **Full Screen** | `F11` | `F11` ✅ Same |

### Use GNOME Terminal Keybindings in Scarab

```toml
[keybindings]
copy_mode = "Ctrl+Shift+C"
paste = "Ctrl+Shift+V"
search = "Ctrl+Shift+F"  # Match GNOME Terminal
new_window = "Ctrl+Shift+N"
close_window = "Ctrl+Shift+W"

[keybindings.custom]
"new_tab" = "Ctrl+Shift+T"
"next_tab" = "Ctrl+PageDown"   # Match GNOME Terminal
"prev_tab" = "Ctrl+PageUp"     # Match GNOME Terminal
"tab_1" = "Alt+1"               # Match GNOME Terminal
"tab_2" = "Alt+2"
"tab_3" = "Alt+3"
"tab_4" = "Alt+4"
"tab_5" = "Alt+5"
"tab_6" = "Alt+6"
"tab_7" = "Alt+7"
"tab_8" = "Alt+8"
"tab_9" = "Alt+9"
"increase_font_size" = "Ctrl+Plus"
"decrease_font_size" = "Ctrl+Minus"
"reset_font_size" = "Ctrl+0"
"toggle_fullscreen" = "F11"
```

---

## Feature Comparison

### GNOME Terminal Features in Scarab

| Feature | GNOME Terminal | Scarab | Notes |
|---------|----------------|--------|-------|
| **Multiple tabs** | ✅ | ✅ | |
| **Split terminal** | ❌ | 🔄 Planned | Use tmux for now |
| **Color schemes** | ✅ Few built-in | ✅ Many themes | Easier customization |
| **Profiles** | ✅ | ✅ Sessions | Similar concept |
| **Transparency** | ✅ | ✅ | Requires compositor |
| **Background image** | ✅ | ❌ | Not planned |
| **Scrollbar** | ✅ | ❌ | Use keybindings |
| **Menu bar** | ✅ | ❌ | Use command palette |
| **Right-click menu** | ✅ | 🔄 Planned | |
| **Bold is bright** | ✅ | ✅ | `[font] bold_is_bright = true` |
| **Audible bell** | ✅ | 🔄 Plugin-based | |
| **Visual bell** | ✅ | 🔄 Plugin-based | |
| **Custom commands** | ❌ | ✅ Plugins | Scarab advantage |
| **Text reflow** | ✅ | ✅ | |
| **Hyperlink detection** | ✅ | ✅ Link hints | |

### Scarab Unique Features

Not available in GNOME Terminal:

1. **Session Persistence**: Survives crashes
2. **Plugin System**: F# plugins for extensibility
3. **Command Palette**: Fuzzy search commands
4. **GPU Acceleration**: Faster rendering
5. **Zero-Copy IPC**: Daemon + client architecture
6. **Hot-Reload Config**: Changes apply instantly
7. **Remote UI Protocol**: Daemon controls client UI

---

## Desktop Integration

### Application Launcher

**GNOME Terminal**: Shows in app grid automatically.

**Scarab**: Create desktop entry.

```bash
# Create desktop file
cat > ~/.local/share/applications/scarab.desktop <<'EOF'
[Desktop Entry]
Name=Scarab Terminal
Comment=GPU-accelerated terminal with F# plugins
Exec=/path/to/scarab-client
Icon=utilities-terminal
Type=Application
Categories=System;TerminalEmulator;
Keywords=terminal;shell;prompt;command;
StartupNotify=true
Terminal=false
EOF

# Update desktop database
update-desktop-database ~/.local/share/applications/
```

### Default Terminal

```bash
# Set Scarab as default terminal
sudo update-alternatives --install /usr/bin/x-terminal-emulator \
    x-terminal-emulator /path/to/scarab-client 50

# Configure default (interactive)
sudo update-alternatives --config x-terminal-emulator

# Or set manually
gsettings set org.gnome.desktop.default-applications.terminal exec '/path/to/scarab-client'
```

### Keyboard Shortcut

**GNOME**: Settings → Keyboard → Custom Shortcuts

```
Name: Scarab Terminal
Command: /path/to/scarab-client
Shortcut: Ctrl+Alt+T
```

**Or via dconf**:
```bash
# Add custom keybinding
CUSTOM_KEYBINDINGS_PATH="/org/gnome/settings-daemon/plugins/media-keys/custom-keybindings"
CUSTOM_KB="${CUSTOM_KEYBINDINGS_PATH}/scarab/"

dconf write "${CUSTOM_KB}name" "'Scarab Terminal'"
dconf write "${CUSTOM_KB}command" "'/path/to/scarab-client'"
dconf write "${CUSTOM_KB}binding" "'<Ctrl><Alt>t'"

# Register the keybinding
dconf write "${CUSTOM_KEYBINDINGS_PATH}/custom-list" "['scarab']"
```

---

## Common Issues

### Colors look different

**Cause**: GNOME Terminal uses GTK theme colors by default.

**Solution**:
```bash
# Export GNOME Terminal colors
dconf dump /org/gnome/terminal/legacy/profiles:/ > ~/colors.dconf

# Extract hex colors manually
# Or use GNOME Terminal's palette names:

[colors]
theme = "tango"  # GNOME Terminal's default
```

### Font rendering looks different

**Cause**: GNOME Terminal uses system font rendering (fontconfig + cairo).
Scarab uses cosmic-text (HarfBuzz).

**Solution**:
```toml
[font]
# Adjust line height to match GNOME Terminal
line_height = 1.0  # GNOME Terminal typically uses 1.0

# Use same font
family = "Monospace"  # System default

# Or match exactly
family = "DejaVu Sans Mono"
size = 12.0
```

### Transparency doesn't work

**Cause**: Requires compositor (same as GNOME Terminal).

**Solution**:
```bash
# Ensure compositor is running
# GNOME: Built-in compositor
# Other DEs: picom, compton, etc.

# Check compositor
ps aux | grep -i composit

# Enable in Scarab
[colors]
opacity = 0.9  # 90% opaque
```

### Missing menu bar

**Scarab**: No menu bar (minimalist design).

**Solution**: Use command palette instead.
```
Ctrl+Shift+P  # Open command palette
Type command name
Enter to execute
```

---

## Configuration Examples

### Minimal (GNOME Terminal defaults)

```toml
# ~/.config/scarab/config.toml
[terminal]
default_shell = "/bin/bash"
scrollback_lines = 10000

[font]
family = "Monospace"
size = 12.0

[colors]
theme = "tango"  # GNOME Terminal default

[keybindings]
copy_mode = "Ctrl+Shift+C"
paste = "Ctrl+Shift+V"
```

### Power User (Enhanced GNOME Terminal)

```toml
[terminal]
scrollback_lines = 50000
columns = 120
rows = 40

[font]
family = "Fira Code"
size = 11.0
line_height = 1.2
enable_ligatures = true

[colors]
theme = "dracula"
opacity = 0.95

[keybindings]
# GNOME Terminal-compatible
copy_mode = "Ctrl+Shift+C"
paste = "Ctrl+Shift+V"
search = "Ctrl+Shift+F"

[keybindings.custom]
"next_tab" = "Ctrl+PageDown"
"prev_tab" = "Ctrl+PageUp"
"new_tab" = "Ctrl+Shift+T"

[plugins]
enabled = [
    "scarab-nav",
    "scarab-palette",
    "scarab-session",
]

[sessions]
restore_on_startup = true
auto_save_interval = 300
```

---

## Migration Checklist

- [ ] Export GNOME Terminal profile (dconf)
- [ ] Run conversion script
- [ ] Review generated Scarab config
- [ ] Adjust colors if needed
- [ ] Test keybindings
- [ ] Create desktop launcher
- [ ] Set as default terminal (optional)
- [ ] Test with daily workflow
- [ ] Keep GNOME Terminal as backup (first week)
- [ ] Fully switch after testing

---

## Advantages of Scarab

### Performance

**GNOME Terminal**:
```bash
# Large file output
cat large.log  # Stutters, high CPU

# Many lines
yes | head -100000  # Lags
```

**Scarab**:
```bash
# Same operations
# Smooth 60 FPS, low CPU
# GPU-accelerated rendering
```

### Extensibility

**GNOME Terminal**: Limited to built-in features.

**Scarab**: Write plugins in F#:
```fsharp
// Highlight git status in prompt
// Show notifications for long commands
// Custom keybindings
// Remote session monitoring
```

### Session Management

**GNOME Terminal**: Lost on crash.

**Scarab**: Daemon preserves state.
```bash
# Client crashes
# Restart:
scarab-client  # All sessions intact
```

---

## Going Back

If Scarab doesn't work for you:

```bash
# GNOME Terminal is still installed
# Just launch it again

# Or set as default
sudo update-alternatives --config x-terminal-emulator
# Select gnome-terminal
```

Your GNOME Terminal settings are unchanged!

---

## Getting Help

### Issues Migrating?

1. Check [Troubleshooting Guide](../reference/troubleshooting.md)
2. Search [GitHub Issues](https://github.com/raibid-labs/scarab/issues)
3. Ask in [Discussions](https://github.com/raibid-labs/scarab/discussions)

### Missing GNOME Terminal Feature?

File feature request with:
- GNOME Terminal feature name
- How you use it
- Desired Scarab behavior

---

## See Also

- [Configuration Reference](../reference/configuration.md)
- [Keybindings Reference](../reference/keybindings.md)
- [Plugin Development](../development/plugins.md)
- [Migration from Alacritty](./from-alacritty.md)
- [Migration from iTerm2](./from-iterm2.md)
