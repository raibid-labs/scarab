# Themes

Customize the visual appearance of Scarab with themes.

## Quick Links

For complete customization documentation, see:
- [Customization Guide](../../../CUSTOMIZATION.md) - Complete customization guide

## Built-in Themes

Scarab includes several built-in themes:

- `dark` - Dark theme (default)
- `light` - Light theme
- `solarized-dark` - Solarized Dark
- `solarized-light` - Solarized Light
- `monokai` - Monokai
- `dracula` - Dracula

### Selecting a Theme

Edit `~/.config/scarab/config.toml`:

```toml
[appearance]
theme = "dracula"
```

Changes apply immediately with hot-reload.

## Custom Themes

Create custom color schemes:

```toml
[theme.colors]
background = "#1e1e1e"
foreground = "#d4d4d4"
cursor = "#ffffff"
cursor_text = "#000000"
selection_bg = "#264f78"
selection_fg = "#ffffff"

# Normal colors (0-7)
[theme.colors.normal]
black = "#000000"
red = "#cd3131"
green = "#0dbc79"
yellow = "#e5e510"
blue = "#2472c8"
magenta = "#bc3fbc"
cyan = "#11a8cd"
white = "#e5e5e5"

# Bright colors (8-15)
[theme.colors.bright]
black = "#666666"
red = "#f14c4c"
green = "#23d18b"
yellow = "#f5f543"
blue = "#3b8eea"
magenta = "#d670d6"
cyan = "#29b8db"
white = "#ffffff"
```

## Theme Files

Save themes as separate files in `~/.config/scarab/themes/`:

```toml
# ~/.config/scarab/themes/my-theme.toml
name = "My Custom Theme"
author = "Your Name"

[colors]
background = "#282c34"
foreground = "#abb2bf"
# ... rest of colors
```

Load with:

```toml
[appearance]
theme = "my-theme"
```

## Advanced Theme Configuration

### True Color Support

Scarab supports 24-bit true color:

```toml
[theme.colors]
background = "#1e1e1e"  # RGB hex notation
foreground = "rgb(212, 212, 212)"  # RGB function
```

### Per-Element Styling

Customize specific UI elements:

```toml
[theme.ui]
tab_bar_bg = "#333333"
tab_bar_fg = "#ffffff"
tab_active_bg = "#007acc"
tab_active_fg = "#ffffff"
tab_inactive_bg = "#2d2d2d"
tab_inactive_fg = "#cccccc"

status_bar_bg = "#007acc"
status_bar_fg = "#ffffff"

plugin_dock_bg = "#252526"
plugin_dock_fg = "#cccccc"
```

### Dynamic Themes (Fusabi)

Create dynamic themes with F#:

```fsharp
// ~/.config/scarab/themes/dynamic.fsx
open Scarab.Config
open System

// Time-based theme switching
let getTheme () =
    let hour = DateTime.Now.Hour
    if hour >= 6 && hour < 18 then
        Theme.Light
    else
        Theme.Dark

theme (getTheme())
```

## Theme Switching

### Command Palette

Press **Ctrl+Shift+P** and type:
```
theme: <theme-name>
```

### Keybinding

Add a keybinding for quick theme switching:

```toml
[keybindings]
toggle_theme = "Ctrl+Shift+T"
```

## Importing Themes

### From Alacritty

Convert Alacritty themes:

```bash
scarab-tools import-theme --from alacritty ~/.config/alacritty/theme.yml
```

### From iTerm2

Convert iTerm2 themes:

```bash
scarab-tools import-theme --from iterm2 ~/Downloads/theme.itermcolors
```

## See Also

- [Customization](./customization.md) - General customization
- [Fonts](./fonts.md) - Font configuration
- [Configuration Schema](../reference/config-schema.md) - Complete reference
