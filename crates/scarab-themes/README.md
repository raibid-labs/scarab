# Scarab Themes

Professional theme system for Scarab Terminal with 10+ built-in themes, import/export, and hot-reload support.

## Features

- **13 Built-in Themes**: Dracula, Solarized (Light/Dark), Nord, Monokai, One Dark/Light, Gruvbox (Light/Dark), Tokyo Night, Catppuccin (Mocha/Latte), Synthwave
- **Live Preview**: Preview themes without applying
- **Hot Reload**: Change themes without restarting
- **Multiple Formats**: TOML, JSON, Base16 YAML support
- **Custom Themes**: Create and save your own themes
- **Command Palette Integration**: Quick access to all theme operations
- **Smart Search**: Filter by variant (light/dark) or tags

## Usage

### Via Command Palette

Press `Ctrl+Shift+P` and type:

- `Theme: Select` - Choose from all themes
- `Theme: Preview` - Preview without applying
- `Theme: List Dark` - Show only dark themes
- `Theme: List Light` - Show only light themes
- `Theme: Import` - Import theme from file
- `Theme: Export` - Export current theme
- `Theme: Create Custom` - Create theme from current colors

### Via Configuration

Edit `~/.config/scarab/config.toml`:

```toml
[colors]
theme = "dracula"  # or any built-in theme ID
```

## Built-in Themes

### Dark Themes

- **dracula** - Vibrant purple accents (default)
- **solarized-dark** - Classic precision colors
- **nord** - Arctic north-bluish palette
- **monokai** - Warm, rich colors
- **one-dark** - Atom's popular theme
- **gruvbox-dark** - Retro groove colors
- **tokyo-night** - Modern Tokyo-inspired
- **catppuccin-mocha** - Soothing pastels
- **synthwave** - Retro cyberpunk neon

### Light Themes

- **solarized-light** - Classic precision colors
- **one-light** - Atom's light variant
- **gruvbox-light** - Retro groove light
- **catppuccin-latte** - Soothing pastel light

## Custom Themes

### Create from TOML

Create `~/.config/scarab/themes/my-theme.toml`:

```toml
[metadata]
id = "my-theme"
name = "My Custom Theme"
author = "Your Name"
description = "My personal theme"
variant = "dark"
tags = ["custom", "dark"]

[colors]
foreground = "#ffffff"
background = "#1e1e1e"
cursor = "#ffffff"
selection_background = "#444444"

[colors.palette]
black = "#000000"
red = "#ff0000"
green = "#00ff00"
yellow = "#ffff00"
blue = "#0000ff"
magenta = "#ff00ff"
cyan = "#00ffff"
white = "#ffffff"
bright_black = "#808080"
bright_red = "#ff8080"
bright_green = "#80ff80"
bright_yellow = "#ffff80"
bright_blue = "#8080ff"
bright_magenta = "#ff80ff"
bright_cyan = "#80ffff"
bright_white = "#ffffff"
```

### Import Base16 Theme

Download any Base16 theme YAML and import:

```bash
# Download Base16 theme
curl -o ~/.config/scarab/themes/ocean.yaml \
  https://raw.githubusercontent.com/chriskempson/base16-schemes/master/ocean.yaml

# Restart client or use "Theme: Import" command
```

## Exporting Themes

Export any theme to share:

```bash
# Via command palette: "Theme: Export"
# Or manually access from ~/.config/scarab/themes/
```

Exported themes can be shared and imported by other users.

## Plugin API

Use themes programmatically:

```rust
use scarab_themes::{ThemeManager, ThemePlugin};

let mut manager = ThemeManager::new();
manager.initialize()?;

// List all themes
for theme in manager.all_themes() {
    println!("{}: {}", theme.id(), theme.name());
}

// Apply theme
manager.set_active_theme("dracula")?;

// Preview theme
manager.set_preview_theme("nord")?;
manager.clear_preview();

// Import theme
let theme = manager.import_theme("path/to/theme.toml")?;

// Export theme
manager.export_theme("dracula", "export.json", ThemeFormat::Json)?;
```

## Architecture

```
scarab-themes/
├── src/
│   ├── theme.rs        # Core theme data structures
│   ├── manager.rs      # Theme management logic
│   ├── plugin.rs       # Plugin implementation
│   ├── themes/         # Built-in theme definitions
│   │   ├── dracula.rs
│   │   ├── nord.rs
│   │   └── ...
│   └── format/         # Import/export formats
│       ├── toml.rs
│       ├── json.rs
│       └── base16.rs
```

## Contributing

To add a new built-in theme:

1. Create `src/themes/mytheme.rs`
2. Define theme using `Theme` struct
3. Add to `src/themes/mod.rs`
4. Test with `cargo test -p scarab-themes`

## License

MIT OR Apache-2.0
