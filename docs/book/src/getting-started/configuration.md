# Configuration

Learn how to configure Scarab to match your preferences.

## Quick Links

For comprehensive configuration documentation, see:
- [Configuration Reference](../../../configuration.md) - Complete configuration documentation
- [Configuration Schema](../reference/config-schema.md) - Schema reference
- [Customization Guide](../../../CUSTOMIZATION.md) - Customization guide

## Configuration File Location

Scarab reads configuration from:
```
~/.config/scarab/config.toml
```

## Basic Configuration

### Minimal Configuration

Create `~/.config/scarab/config.toml`:

```toml
[terminal]
shell = "/bin/bash"
scrollback_lines = 10000

[appearance]
theme = "dark"
font_family = "monospace"
font_size = 14.0

[keybindings]
copy = "Ctrl+Shift+C"
paste = "Ctrl+Shift+V"
```

## Hot Reload

Scarab supports hot-reload for configuration changes. Edit your config file and changes will be applied immediately without restarting.

## Advanced Configuration

### F# DSL Configuration

For advanced users, Scarab supports configuration via Fusabi (F#) scripts:

```fsharp
// ~/.config/scarab/config.fsx
open Scarab.Config

config {
    terminal {
        shell = "/bin/bash"
        scrollback = 10000
    }

    appearance {
        theme = Theme.Dark
        font = Font.create "Fira Code" 14.0
    }
}
```

## Theme Configuration

Customize colors in your configuration:

```toml
[theme.colors]
background = "#1e1e1e"
foreground = "#d4d4d4"
cursor = "#ffffff"

[theme.colors.normal]
black = "#000000"
red = "#cd3131"
green = "#0dbc79"
yellow = "#e5e510"
blue = "#2472c8"
magenta = "#bc3fbc"
cyan = "#11a8cd"
white = "#e5e5e5"
```

## Next Steps

- Explore [Keybindings](../user-guide/keybindings.md)
- Learn about [Themes](../user-guide/themes.md)
- View [Configuration Schema](../reference/config-schema.md)
