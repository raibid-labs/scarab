# Migration Guides

Migrate to Scarab from other terminal emulators.

## Overview

Scarab provides migration guides for users switching from popular terminal emulators.

## Available Guides

- [From Alacritty](./from-alacritty.md) - Migrate from Alacritty
- [From iTerm2](./from-iterm2.md) - Migrate from iTerm2 (macOS)
- [From GNOME Terminal](./from-gnome-terminal.md) - Migrate from GNOME Terminal

## Quick Links

Detailed migration guides in the docs directory:
- [From Alacritty](../../../migration/from-alacritty.md)
- [From iTerm2](../../../migration/from-iterm2.md)
- [From GNOME Terminal](../../../migration/from-gnome-terminal.md)

## General Migration Steps

### 1. Export Configuration

Export your current terminal configuration for reference.

### 2. Install Scarab

Follow the [Installation](../getting-started/installation.md) guide.

### 3. Configure Scarab

Adapt your previous configuration to Scarab's format. See [Configuration](../getting-started/configuration.md).

### 4. Import Theme

Convert and import your color theme. See [Themes](./themes.md).

### 5. Set Up Keybindings

Map your familiar keybindings. See [Keybindings](./keybindings.md).

### 6. Test and Iterate

Test your configuration and make adjustments as needed.

## Common Configuration Mappings

### Shell Configuration

Most terminals → Scarab:
```toml
[terminal]
shell = "/bin/bash"  # or /bin/zsh, /bin/fish, etc.
```

### Font Configuration

```toml
[appearance]
font_family = "Your Font"
font_size = 14.0
```

### Theme Configuration

```toml
[appearance]
theme = "dark"  # or your custom theme

[theme.colors]
background = "#1e1e1e"
foreground = "#d4d4d4"
# ... additional colors
```

### Keybindings

```toml
[keybindings]
copy = "Ctrl+Shift+C"
paste = "Ctrl+Shift+V"
# ... additional bindings
```

## Feature Comparison

### Available in Scarab

- GPU-accelerated rendering
- Plugin system (F#/Fusabi)
- Session persistence
- Split-process architecture
- Hot-reload configuration
- Command palette
- Link hints

### Planned Features

- Tabs and panes (in development)
- macOS support
- Windows support
- Image protocols (Sixel, Kitty)

## Getting Help

If you encounter issues during migration:

1. Check the [Troubleshooting](../reference/troubleshooting.md) guide
2. Review the [FAQ](../reference/faq.md)
3. Open an issue on GitHub
4. Ask in GitHub Discussions

## See Also

- [Configuration](../getting-started/configuration.md) - Configuration guide
- [Customization](./customization.md) - Customization options
- [Troubleshooting](../reference/troubleshooting.md) - Common issues
