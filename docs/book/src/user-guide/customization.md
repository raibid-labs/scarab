# Customization

Personalize Scarab to match your preferences and workflow.

## Quick Links

For complete customization documentation, see:
- [Customization Guide](../../../CUSTOMIZATION.md) - Complete customization guide
- [Configuration Reference](../reference/config-schema.md) - Configuration schema

## Overview

Scarab is highly customizable through TOML configuration files and F# scripts.

## Configuration File

Edit `~/.config/scarab/config.toml` to customize Scarab.

### Basic Customization

```toml
[terminal]
shell = "/bin/bash"
scrollback_lines = 10000

[appearance]
theme = "dark"
font_family = "Fira Code"
font_size = 14.0
line_height = 1.2

[window]
width = 1280
height = 720
opacity = 1.0
```

## Hot Reload

Configuration changes are applied immediately without restarting:

1. Edit `~/.config/scarab/config.toml`
2. Save the file
3. Changes take effect instantly

## Customization Areas

### Themes

See [Themes](./themes.md) for:
- Built-in themes
- Custom color schemes
- Theme switching

### Fonts

See [Fonts](./fonts.md) for:
- Font selection
- Font rendering options
- Ligature support

### Keybindings

Customize keyboard shortcuts:

```toml
[keybindings]
copy = "Ctrl+Shift+C"
paste = "Ctrl+Shift+V"
new_tab = "Ctrl+Shift+T"
close_tab = "Ctrl+Shift+W"

[keybindings.navigation]
focus_next = "Tab"
focus_prev = "Shift+Tab"
```

See [Keybindings](./keybindings.md) for complete reference.

### Plugins

Enable and configure plugins:

```toml
[plugins]
enabled = ["scarab-nav", "scarab-palette", "git-status"]

[plugins.config.git-status]
show_branch = true
show_dirty = true
```

See [Plugins](./plugins.md) for plugin documentation.

## Advanced Customization

### F# Configuration DSL

For complex configurations, use Fusabi (F#) syntax:

```fsharp
// ~/.config/scarab/config.fsx
open Scarab.Config

config {
    terminal {
        shell = "/bin/bash"
        scrollback = 10000
    }

    appearance {
        theme = Theme.custom {
            background = Color.hex "#1e1e1e"
            foreground = Color.hex "#d4d4d4"
        }
        font = Font.create "Fira Code" 14.0
    }

    keybindings {
        bind "Ctrl+Shift+N" (Command.NewWindow)
        bind "Ctrl+Shift+T" (Command.NewTab)
    }
}
```

### Per-Session Overrides

Override settings for specific sessions:

```toml
[sessions.development]
shell = "/bin/zsh"
working_directory = "~/projects"

[sessions.development.appearance]
theme = "light"
font_size = 16.0
```

## See Also

- [Themes](./themes.md) - Theme customization
- [Fonts](./fonts.md) - Font configuration
- [Keybindings](./keybindings.md) - Keyboard shortcuts
- [Configuration Schema](../reference/config-schema.md) - Complete reference
