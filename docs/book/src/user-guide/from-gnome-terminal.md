# Migrating from GNOME Terminal

A guide for GNOME Terminal users switching to Scarab.

## Quick Links

For the complete migration guide, see:
- [From GNOME Terminal](../../../migration/from-gnome-terminal.md) - Detailed migration guide

## Overview

GNOME Terminal is the default terminal for many Linux distributions. Scarab offers more advanced features while maintaining compatibility.

## Configuration Migration

### GNOME Terminal Configuration

GNOME Terminal uses dconf/gsettings:
```bash
gsettings list-recursively org.gnome.Terminal.Legacy.Profile
```

### Scarab Configuration

Scarab uses TOML files:
```
~/.config/scarab/config.toml
```

## Configuration Mapping

### Font

**GNOME Terminal:**
```bash
gsettings set org.gnome.Terminal.Legacy.Profile:/.../ font 'Monospace 14'
```

**Scarab:**
```toml
[appearance]
font_family = "Monospace"
font_size = 14.0
```

### Colors

**GNOME Terminal:**
- GUI-based color picker
- Preset color schemes

**Scarab:**
```toml
[theme.colors]
background = "#300a24"  # GNOME Terminal purple
foreground = "#ffffff"

[theme.colors.normal]
black = "#2e3436"
red = "#cc0000"
green = "#4e9a06"
yellow = "#c4a000"
blue = "#3465a4"
magenta = "#75507b"
cyan = "#06989a"
white = "#d3d7cf"
```

### Shell

**GNOME Terminal:**
- Uses user's default shell
- Can override in preferences

**Scarab:**
```toml
[terminal]
shell = "/bin/bash"  # or your preferred shell
```

## Feature Comparison

### GNOME Terminal Features in Scarab

- Multiple profiles → Sessions
- Color schemes → Themes
- Keyboard shortcuts → Keybindings
- Scrollback → Scrollback buffer
- Transparency → Window opacity

### Scarab Advantages

- **Plugin system** - Extensible with F#
- **Command palette** - Quick access to commands
- **Link hints** - Keyboard URL opening
- **Session persistence** - Survive crashes
- **Hot-reload config** - Instant changes
- **GPU acceleration** - Better performance

### GNOME Terminal Features Not in Scarab

- GUI preferences - Scarab uses config files
- Desktop integration - Scarab is standalone
- Nautilus integration - Not applicable

## Migration Checklist

- [ ] Install Scarab
- [ ] Note your GNOME Terminal preferences
- [ ] Create Scarab configuration
- [ ] Set up color scheme
- [ ] Configure keybindings
- [ ] Test basic operations
- [ ] Configure shell integration

## Exporting GNOME Terminal Settings

### View Current Settings

```bash
# List all profiles
gsettings get org.gnome.Terminal.ProfilesList list

# Get default profile ID
gsettings get org.gnome.Terminal.ProfilesList default

# View profile settings
dconf dump /org/gnome/terminal/legacy/profiles:/
```

### Common Settings

```bash
# Font
gsettings get org.gnome.Terminal.Legacy.Profile:/.../ font

# Colors
gsettings get org.gnome.Terminal.Legacy.Profile:/.../ use-theme-colors
gsettings get org.gnome.Terminal.Legacy.Profile:/.../ foreground-color
gsettings get org.gnome.Terminal.Legacy.Profile:/.../ background-color

# Scrollback
gsettings get org.gnome.Terminal.Legacy.Profile:/.../ scrollback-lines
```

## Example Configuration

Based on GNOME Terminal defaults:

```toml
[terminal]
shell = "/bin/bash"
scrollback_lines = 10000

[appearance]
font_family = "Monospace"
font_size = 12.0
theme = "gnome-default"

[theme.colors]
background = "#300a24"
foreground = "#ffffff"
cursor = "#ffffff"

[theme.colors.normal]
black = "#2e3436"
red = "#cc0000"
green = "#4e9a06"
yellow = "#c4a000"
blue = "#3465a4"
magenta = "#75507b"
cyan = "#06989a"
white = "#d3d7cf"

[theme.colors.bright]
black = "#555753"
red = "#ef2929"
green = "#8ae234"
yellow = "#fce94f"
blue = "#729fcf"
magenta = "#ad7fa8"
cyan = "#34e2e2"
white = "#eeeeec"
```

## Getting Help

If you need assistance:

1. Check [Configuration Guide](../getting-started/configuration.md)
2. Review [Troubleshooting](../reference/troubleshooting.md)
3. See [Customization](./customization.md)

## See Also

- [Configuration](../getting-started/configuration.md) - Configuration guide
- [Themes](./themes.md) - Theme configuration
- [Keybindings](./keybindings.md) - Keybinding reference
