# Migrating from Alacritty

A guide for Alacritty users switching to Scarab.

## Quick Links

For the complete migration guide, see:
- [From Alacritty](../../../migration/from-alacritty.md) - Detailed migration guide

## Overview

Alacritty and Scarab share similar design philosophies:
- GPU-accelerated rendering
- Minimalist design
- Configuration via text files
- Focus on performance

## Configuration Migration

### Alacritty Configuration Location

```
~/.config/alacritty/alacritty.yml
```

### Scarab Configuration Location

```
~/.config/scarab/config.toml
```

## Configuration Mapping

### Font

**Alacritty (YAML):**
```yaml
font:
  normal:
    family: "Fira Code"
  size: 14.0
```

**Scarab (TOML):**
```toml
[appearance]
font_family = "Fira Code"
font_size = 14.0
```

### Colors

**Alacritty (YAML):**
```yaml
colors:
  primary:
    background: '#1e1e1e'
    foreground: '#d4d4d4'
  normal:
    black: '#000000'
    red: '#cd3131'
```

**Scarab (TOML):**
```toml
[theme.colors]
background = "#1e1e1e"
foreground = "#d4d4d4"

[theme.colors.normal]
black = "#000000"
red = "#cd3131"
```

### Keybindings

**Alacritty (YAML):**
```yaml
key_bindings:
  - { key: V, mods: Control|Shift, action: Paste }
  - { key: C, mods: Control|Shift, action: Copy }
```

**Scarab (TOML):**
```toml
[keybindings]
paste = "Ctrl+Shift+V"
copy = "Ctrl+Shift+C"
```

### Window

**Alacritty (YAML):**
```yaml
window:
  dimensions:
    columns: 120
    lines: 40
  padding:
    x: 2
    y: 2
```

**Scarab (TOML):**
```toml
[window]
width = 1280
height = 720
padding_x = 2
padding_y = 2
```

## Converting Themes

Use the conversion tool (planned):

```bash
scarab-tools import-theme --from alacritty ~/.config/alacritty/alacritty.yml
```

Or manually convert using the mappings above.

## Feature Comparison

### Alacritty Features in Scarab

- GPU-accelerated rendering
- True color support
- Scrollback
- Mouse support
- Configuration via files

### Scarab Advantages

- **Plugin system** - Extend with F# scripts
- **Session persistence** - Sessions survive crashes
- **Command palette** - Quick command access
- **Link hints** - Keyboard-driven URL opening
- **Hot-reload config** - Changes apply instantly

### Missing from Scarab (Planned)

- Ligatures (in development)
- Hints for file paths (partially available via link hints)

## Migration Checklist

- [ ] Install Scarab dependencies
- [ ] Build Scarab from source
- [ ] Convert configuration from YAML to TOML
- [ ] Import color theme
- [ ] Map keybindings
- [ ] Test basic terminal operations
- [ ] Configure shell integration
- [ ] Set up plugins (optional)

## Getting Help

If you need assistance:

1. Check [Configuration Schema](../reference/config-schema.md)
2. Review [Troubleshooting](../reference/troubleshooting.md)
3. Compare with [Customization Guide](./customization.md)

## See Also

- [Configuration](../getting-started/configuration.md) - Configuration guide
- [Themes](./themes.md) - Theme configuration
- [Keybindings](./keybindings.md) - Keybinding reference
