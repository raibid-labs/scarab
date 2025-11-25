# Migrating from Alacritty to Scarab

Complete guide for Alacritty users switching to Scarab Terminal.

## Quick Comparison

| Feature | Alacritty | Scarab | Notes |
|---------|-----------|--------|-------|
| **Performance** | Excellent | Excellent | Comparable rendering speed |
| **GPU Rendering** | ✅ OpenGL | ✅ Bevy (Vulkan/Metal/DX12) | Both GPU-accelerated |
| **Config Format** | YAML | TOML | Similar structure |
| **Hot Reload** | ✅ | ✅ | Both support live config updates |
| **Scrollback** | ✅ | ✅ | Scarab adds persistence |
| **Tabs** | ❌ | ✅ | Scarab has built-in tabs |
| **Splits** | ❌ | 🔄 Coming | Use tmux for now |
| **Plugins** | ❌ | ✅ F# (Fusabi) | Scarab's main differentiator |
| **Session Persistence** | ❌ | ✅ | Daemon survives client crashes |
| **Cross-platform** | ✅ All platforms | 🔄 Linux (macOS/Win planned) | |

---

## Configuration Migration

### File Location

**Alacritty**:
```
~/.config/alacritty/alacritty.yml
```

**Scarab**:
```
~/.config/scarab/config.toml
```

### Format Conversion

**Alacritty** uses YAML, **Scarab** uses TOML. Here's the mapping:

#### Window Settings

**Alacritty**:
```yaml
window:
  dimensions:
    columns: 80
    lines: 24
  padding:
    x: 2
    y: 2
  opacity: 0.95
```

**Scarab**:
```toml
[terminal]
columns = 80
rows = 24

[colors]
opacity = 0.95

# Note: Padding not yet supported in Scarab
```

#### Font Configuration

**Alacritty**:
```yaml
font:
  normal:
    family: "JetBrains Mono"
  size: 14.0
  offset:
    x: 0
    y: 0
  glyph_offset:
    x: 0
    y: 0
```

**Scarab**:
```toml
[font]
family = "JetBrains Mono"
size = 14.0
line_height = 1.2

# Note: Offsets handled automatically by cosmic-text
```

#### Colors

**Alacritty**:
```yaml
colors:
  primary:
    background: '#1e1e1e'
    foreground: '#d4d4d4'
  cursor:
    text: '#1e1e1e'
    cursor: '#d4d4d4'
  normal:
    black:   '#000000'
    red:     '#cd3131'
    green:   '#0dbc79'
    yellow:  '#e5e510'
    blue:    '#2472c8'
    magenta: '#bc3fbc'
    cyan:    '#11a8cd'
    white:   '#e5e5e5'
  bright:
    black:   '#666666'
    red:     '#f14c4c'
    green:   '#23d18b'
    yellow:  '#f5f543'
    blue:    '#3b8eea'
    magenta: '#d670d6'
    cyan:    '#29b8db'
    white:   '#ffffff'
```

**Scarab**:
```toml
[colors]
foreground = "#d4d4d4"
background = "#1e1e1e"
cursor = "#d4d4d4"

[colors.palette]
black = "#000000"
red = "#cd3131"
green = "#0dbc79"
yellow = "#e5e510"
blue = "#2472c8"
magenta = "#bc3fbc"
cyan = "#11a8cd"
white = "#e5e5e5"

bright_black = "#666666"
bright_red = "#f14c4c"
bright_green = "#23d18b"
bright_yellow = "#f5f543"
bright_blue = "#3b8eea"
bright_magenta = "#d670d6"
bright_cyan = "#29b8db"
bright_white = "#ffffff"
```

#### Keybindings

**Alacritty**:
```yaml
key_bindings:
  - { key: V,        mods: Control|Shift, action: Paste            }
  - { key: C,        mods: Control|Shift, action: Copy             }
  - { key: Key0,     mods: Control,       action: ResetFontSize    }
  - { key: Equals,   mods: Control,       action: IncreaseFontSize }
  - { key: Minus,    mods: Control,       action: DecreaseFontSize }
  - { key: N,        mods: Control|Shift, action: SpawnNewInstance }
```

**Scarab**:
```toml
[keybindings]
paste = "Ctrl+Shift+V"
copy_mode = "Ctrl+Shift+C"

[keybindings.custom]
"reset_font_size" = "Ctrl+0"
"increase_font_size" = "Ctrl+Plus"
"decrease_font_size" = "Ctrl+Minus"
"new_window" = "Ctrl+Shift+N"
```

#### Scrolling

**Alacritty**:
```yaml
scrolling:
  history: 10000
  multiplier: 3
```

**Scarab**:
```toml
[terminal]
scrollback_lines = 10000
scroll_multiplier = 3.0
```

#### Shell

**Alacritty**:
```yaml
shell:
  program: /bin/zsh
  args:
    - --login
```

**Scarab**:
```toml
[terminal]
default_shell = "/bin/zsh"

# Note: Shell args not yet supported
# Workaround: Use ~/.zshrc for initialization
```

---

## Automated Conversion Script

Use this script to convert Alacritty config to Scarab:

```bash
#!/bin/bash
# alacritty2scarab.sh - Convert Alacritty YAML to Scarab TOML

ALACRITTY_CONFIG="$HOME/.config/alacritty/alacritty.yml"
SCARAB_CONFIG="$HOME/.config/scarab/config.toml"

if [ ! -f "$ALACRITTY_CONFIG" ]; then
    echo "Error: Alacritty config not found at $ALACRITTY_CONFIG"
    exit 1
fi

echo "Converting Alacritty config to Scarab format..."

# Install yq if not available
if ! command -v yq &> /dev/null; then
    echo "Installing yq (YAML processor)..."
    sudo wget -qO /usr/local/bin/yq https://github.com/mikefarah/yq/releases/latest/download/yq_linux_amd64
    sudo chmod +x /usr/local/bin/yq
fi

# Extract values
FONT_FAMILY=$(yq eval '.font.normal.family' "$ALACRITTY_CONFIG")
FONT_SIZE=$(yq eval '.font.size' "$ALACRITTY_CONFIG")
COLUMNS=$(yq eval '.window.dimensions.columns' "$ALACRITTY_CONFIG")
ROWS=$(yq eval '.window.dimensions.lines' "$ALACRITTY_CONFIG")
SCROLLBACK=$(yq eval '.scrolling.history' "$ALACRITTY_CONFIG")
OPACITY=$(yq eval '.window.opacity' "$ALACRITTY_CONFIG")
BG=$(yq eval '.colors.primary.background' "$ALACRITTY_CONFIG")
FG=$(yq eval '.colors.primary.foreground' "$ALACRITTY_CONFIG")

# Generate Scarab config
cat > "$SCARAB_CONFIG" <<EOF
# Converted from Alacritty config on $(date)
# Original: $ALACRITTY_CONFIG

[terminal]
columns = ${COLUMNS:-80}
rows = ${ROWS:-24}
scrollback_lines = ${SCROLLBACK:-10000}

[font]
family = "${FONT_FAMILY:-JetBrains Mono}"
size = ${FONT_SIZE:-14.0}
line_height = 1.2

[colors]
background = "${BG:-#1e1e1e}"
foreground = "${FG:-#d4d4d4}"
opacity = ${OPACITY:-1.0}

[colors.palette]
# Extract from Alacritty config manually or use default
black = "$(yq eval '.colors.normal.black' "$ALACRITTY_CONFIG")"
red = "$(yq eval '.colors.normal.red' "$ALACRITTY_CONFIG")"
green = "$(yq eval '.colors.normal.green' "$ALACRITTY_CONFIG")"
yellow = "$(yq eval '.colors.normal.yellow' "$ALACRITTY_CONFIG")"
blue = "$(yq eval '.colors.normal.blue' "$ALACRITTY_CONFIG")"
magenta = "$(yq eval '.colors.normal.magenta' "$ALACRITTY_CONFIG")"
cyan = "$(yq eval '.colors.normal.cyan' "$ALACRITTY_CONFIG")"
white = "$(yq eval '.colors.normal.white' "$ALACRITTY_CONFIG")"

bright_black = "$(yq eval '.colors.bright.black' "$ALACRITTY_CONFIG")"
bright_red = "$(yq eval '.colors.bright.red' "$ALACRITTY_CONFIG")"
bright_green = "$(yq eval '.colors.bright.green' "$ALACRITTY_CONFIG")"
bright_yellow = "$(yq eval '.colors.bright.yellow' "$ALACRITTY_CONFIG")"
bright_blue = "$(yq eval '.colors.bright.blue' "$ALACRITTY_CONFIG")"
bright_magenta = "$(yq eval '.colors.bright.magenta' "$ALACRITTY_CONFIG")"
bright_cyan = "$(yq eval '.colors.bright.cyan' "$ALACRITTY_CONFIG")"
bright_white = "$(yq eval '.colors.bright.white' "$ALACRITTY_CONFIG")"

[keybindings]
# Standard Alacritty-style keybindings
copy_mode = "Ctrl+Shift+C"
paste = "Ctrl+Shift+V"

[keybindings.custom]
"increase_font_size" = "Ctrl+Plus"
"decrease_font_size" = "Ctrl+Minus"
"reset_font_size" = "Ctrl+0"

[ui]
cursor_style = "block"
cursor_blink = true

[sessions]
restore_on_startup = false
auto_save_interval = 300
EOF

echo "Conversion complete!"
echo "Config written to: $SCARAB_CONFIG"
echo ""
echo "Please review and adjust as needed."
echo "Some Alacritty features may not have direct equivalents."
```

**Usage**:
```bash
chmod +x alacritty2scarab.sh
./alacritty2scarab.sh
```

---

## Feature Mapping

### Features in Both

| Feature | Alacritty | Scarab | Notes |
|---------|-----------|--------|-------|
| GPU rendering | ✅ | ✅ | |
| True color | ✅ | ✅ | |
| Scrollback | ✅ | ✅ | Scarab adds persistence |
| Font fallback | ✅ | ✅ | |
| Hot reload | ✅ | ✅ | |
| Mouse support | ✅ | ✅ | |
| Hints mode | ✅ | ✅ | Scarab: Link hints |
| Vi mode | ✅ | ✅ | Scarab: Copy mode |

### Alacritty Features Not in Scarab (Yet)

| Feature | Status | Workaround |
|---------|--------|------------|
| Ligature decorations | 🔄 Planned | Basic ligatures work |
| Window padding | 🔄 Planned | None |
| Shell args | 🔄 Planned | Use shell rc files |
| Working directory | ✅ Available | `[sessions] working_directory` |
| Dynamic title | ✅ Available | Automatic via VTE |
| Hints (custom) | 🔄 Planned | Link hints only |
| IME support | 🔄 Planned | Works on some platforms |

### Scarab Features Not in Alacritty

| Feature | Description |
|---------|-------------|
| **Split Architecture** | Daemon + client for session persistence |
| **Built-in Tabs** | No need for tmux/screen for basic tabs |
| **Plugin System** | F# plugins (Fusabi) for extensibility |
| **Session Management** | Save/restore sessions with scrollback |
| **Command Palette** | Fuzzy search for commands |
| **Remote UI Protocol** | Daemon can control client UI |

---

## Migration Workflow

### Parallel Usage (Recommended)

Run both terminals during transition:

1. **Install Scarab** (keep Alacritty):
   ```bash
   # Build Scarab
   git clone https://github.com/raibid-labs/scarab.git
   cd scarab
   cargo build --release

   # Keep Alacritty as fallback
   ```

2. **Convert config**:
   ```bash
   ./alacritty2scarab.sh
   ```

3. **Test Scarab**:
   ```bash
   # Launch Scarab
   ./target/release/scarab-daemon &
   ./target/release/scarab-client

   # Test your workflow
   # - Run your usual commands
   # - Test keybindings
   # - Check plugins
   ```

4. **Gradually switch**:
   ```bash
   # Week 1: Use Scarab for development
   # Week 2: Use Scarab for everything except production
   # Week 3: Full switch, keep Alacritty as backup
   ```

5. **Make default** (optional):
   ```bash
   # Update desktop entry
   sudo update-alternatives --install /usr/bin/x-terminal-emulator \
       x-terminal-emulator /path/to/scarab-client 50

   # Or symlink
   sudo ln -s /path/to/scarab-client /usr/local/bin/terminal
   ```

---

## Common Issues

### Colors look different

**Cause**: Bevy rendering vs OpenGL may have slight color differences.

**Solution**:
```toml
[colors]
# Adjust gamma/brightness if needed
dim_opacity = 0.7  # Adjust for dimmed colors

# Or use exact Alacritty theme
theme = null
# Copy colors from Alacritty config
```

### Font looks different

**Cause**: cosmic-text vs fontconfig may render differently.

**Solution**:
```toml
[font]
# Try adjusting line height
line_height = 1.0  # vs Alacritty's default ~1.15

# Use thin strokes (macOS)
use_thin_strokes = true
```

### Keybindings don't work

**Cause**: Alacritty uses different key names.

**Solution**:
```toml
# Alacritty: Key0, Key1, etc.
# Scarab: 0, 1, etc.

# Alacritty: Equals
# Scarab: Plus or Equal

# Check Scarab docs for key names
```

### Missing features

**Solutions**:
- **Tabs**: Scarab has built-in tabs (use `Ctrl+Shift+T`)
- **Splits**: Use tmux inside Scarab (native splits coming)
- **Hints**: Use link hints (`Ctrl+Shift+O`)
- **Vi mode**: Use copy mode (`Ctrl+Shift+C`)

---

## Benefits of Switching

### 1. Session Persistence

**Scenario**: Client crashes or you close window accidentally.

**Alacritty**: Lost all terminal state and running processes.

**Scarab**: Daemon preserves state, just reconnect:
```bash
scarab-client  # Reconnects to daemon, session intact
```

### 2. Extensibility

**Alacritty**: Limited to config file customization.

**Scarab**: Write F# plugins for custom behavior:
```fsharp
// Auto-save sessions every 5 minutes
let on_post_command ctx cmd =
    async {
        ctx.SaveSession()
        return Ok ()
    }
```

### 3. Built-in Features

**Alacritty**: Need external tools for tabs/multiplexing.

**Scarab**: Built-in tabs, command palette, session management.

### 4. Modern Architecture

**Alacritty**: Monolithic binary.

**Scarab**: Modular daemon+client, easier to extend and maintain.

---

## Going Back to Alacritty

If Scarab doesn't work for you:

1. **Your config is safe**: Alacritty config unchanged
2. **Easy switch**: Just launch Alacritty again
3. **Export themes**: Convert Scarab themes back to Alacritty YAML

**Keep both**: No need to uninstall one to use the other!

---

## Getting Help

### Alacritty feature you need?

1. Check [Scarab roadmap](../../ROADMAP.md) - might be planned
2. File feature request: https://github.com/raibid-labs/scarab/issues
3. Write a plugin to add the feature
4. Contribute to Scarab development

### Found a bug?

Report at: https://github.com/raibid-labs/scarab/issues

Include:
- Alacritty config that worked
- Scarab config that doesn't
- Expected vs actual behavior

---

## See Also

- [Scarab Configuration Reference](../reference/configuration.md)
- [Alacritty Documentation](https://github.com/alacritty/alacritty)
- [Migration from iTerm2](./from-iterm2.md)
- [Migration from GNOME Terminal](./from-gnome-terminal.md)
