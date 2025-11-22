# Scarab Configuration System

Comprehensive TOML-based configuration with hot-reload, validation, and zero-config startup.

## Features

- **Zero-Config Startup**: Works perfectly with sensible defaults
- **TOML Format**: Human-readable, easy to edit
- **Hot-Reload**: Changes apply instantly (<100ms)
- **Global + Local**: Per-directory config overrides
- **Validation**: Helpful error messages with suggestions
- **IDE Support**: JSON Schema for autocomplete
- **Type-Safe**: Strongly typed Rust API

## Quick Start

### Zero Configuration

Scarab works out of the box with no configuration needed:

```bash
scarab  # Just works!
```

### Create Your Config

```bash
# Create default config at ~/.config/scarab/config.toml
scarab config init

# Edit with your favorite editor
$EDITOR ~/.config/scarab/config.toml
```

### Minimal Configuration

```toml
# ~/.config/scarab/config.toml
[font]
family = "JetBrains Mono"
size = 14.0

[colors]
theme = "dracula"
```

## Configuration Locations

Scarab searches for configuration in this order:

1. **Local Config**: `.scarab.toml` in current directory (walks up)
2. **Global Config**: `~/.config/scarab/config.toml`
3. **Defaults**: Built-in sensible defaults

Local configs override global settings, enabling per-project customization.

## Configuration Structure

### Terminal Settings

```toml
[terminal]
default_shell = "/bin/zsh"          # Default: $SHELL
scrollback_lines = 10000            # Range: 100-100,000
alt_screen = true                   # Enable alternate screen
scroll_multiplier = 3.0             # Scroll speed
auto_scroll = true                  # Auto-scroll on output
```

### Font Configuration

```toml
[font]
family = "JetBrains Mono"           # Primary font
size = 14.0                         # Range: 6.0-72.0
line_height = 1.2                   # Range: 0.5-3.0
fallback = ["Fira Code", "Menlo"]   # Fallback fonts
bold_is_bright = true               # Bright colors for bold
use_thin_strokes = false            # macOS thin strokes
```

### Color Themes

#### Using Predefined Themes

```toml
[colors]
theme = "dracula"  # or "nord", "gruvbox", "monokai"
```

#### Custom Colors

```toml
[colors]
foreground = "#f8f8f2"
background = "#282a36"
cursor = "#f8f8f2"
selection_background = "#44475a"
opacity = 1.0                       # Range: 0.0-1.0
dim_opacity = 0.7                   # Dim inactive windows
```

#### Custom Color Palette

```toml
[colors.palette]
black = "#21222c"
red = "#ff5555"
green = "#50fa7b"
yellow = "#f1fa8c"
blue = "#bd93f9"
magenta = "#ff79c6"
cyan = "#8be9fd"
white = "#f8f8f2"
bright_black = "#6272a4"
bright_red = "#ff6e6e"
# ... (16 colors total)
```

### Keybindings

```toml
[keybindings]
leader_key = "Space"
copy_mode = "Ctrl+Shift+C"
paste = "Ctrl+Shift+V"
search = "Ctrl+Shift+F"
command_palette = "Ctrl+Shift+P"
new_window = "Ctrl+Shift+N"
close_window = "Ctrl+Shift+W"
next_tab = "Ctrl+Tab"
prev_tab = "Ctrl+Shift+Tab"

# Custom keybindings
[keybindings.custom]
split_vertical = "Ctrl+Shift+|"
split_horizontal = "Ctrl+Shift+-"
zoom_in = "Ctrl+="
```

### UI Configuration

```toml
[ui]
link_hints = true                   # Enable link hints
command_palette = true              # Enable command palette
animations = true                   # UI animations
smooth_scroll = true                # Smooth scrolling
show_tabs = true                    # Show tab bar
tab_position = "top"                # "top", "bottom", "left", "right"
cursor_style = "block"              # "block", "beam", "underline"
cursor_blink = true                 # Enable cursor blinking
cursor_blink_interval = 750         # Blink interval (ms)
```

### Plugin Configuration

```toml
[plugins]
enabled = ["auto-notify", "git-status"]

[plugins.config.auto-notify]
keywords = ["ERROR", "FAIL", "PANIC"]
notification_style = "urgent"
min_runtime_seconds = 30

[plugins.config.git-status]
show_branch = true
show_dirty = true
update_interval = 2000
```

### Session Management

```toml
[sessions]
restore_on_startup = false          # Restore last session
auto_save_interval = 300            # Auto-save (seconds)
save_scrollback = true              # Save scrollback history
working_directory = "/path/to/dir"  # Default working directory
```

## Hot-Reload

Changes to configuration files are automatically detected and applied:

```rust
use scarab_config::prelude::*;

// Create watcher
let config = ScarabConfig::default();
let mut watcher = ConfigWatcher::new(config)?;

// Register callback for config changes
watcher.on_change(Box::new(|new_config| {
    println!("Config updated: font size = {}", new_config.font.size);
}));

// Start watching
watcher.start()?;
```

Hot-reload typically completes in **<100ms**.

## Validation

Configuration is validated on load with helpful error messages:

```toml
[font]
size = 100.0  # Too large!
```

```
Error: Font size 100.0 is out of range.
Valid range: 6.0 to 72.0
Suggestion: Try 12.0 or 14.0 for readability
```

### Auto-Fix

Some validation errors can be automatically fixed:

```rust
use scarab_config::prelude::*;

let config = load_config_from_file("config.toml")?;
let fixed = ConfigValidator::auto_fix(config);
```

## Per-Directory Configs

Create `.scarab.toml` in any directory to override global settings:

```toml
# /path/to/project/.scarab.toml
[terminal]
default_shell = "/usr/local/bin/node"

[plugins]
enabled = ["project-specific-plugin"]

[sessions]
working_directory = "/path/to/project"
```

Only specify settings you want to override. Scarab walks up the directory tree
to find the nearest `.scarab.toml`.

## Example Configurations

See the `examples/` directory for complete configurations:

- **`minimal.toml`**: Bare minimum configuration
- **`default.toml`**: Complete default configuration with comments
- **`nord-theme.toml`**: Nord color scheme
- **`gruvbox-dark.toml`**: Gruvbox dark theme
- **`poweruser.toml`**: Advanced configuration with all features
- **`light-theme.toml`**: Solarized light theme
- **`local-override.toml`**: Example per-directory config

## IDE Support

For IDE autocomplete and validation, install the JSON Schema:

### VS Code

Add to `.vscode/settings.json`:

```json
{
  "yaml.schemas": {
    "./crates/scarab-config/schema.json": ["**/.scarab.toml", "**/config.toml"]
  }
}
```

### JetBrains IDEs

Settings → Languages & Frameworks → Schemas and DTDs → JSON Schema Mappings

Add `schema.json` and map to `*.toml` in the scarab config directory.

## Programmatic Usage

### Loading Configuration

```rust
use scarab_config::prelude::*;

// Load with default discovery
let loader = ConfigLoader::new();
let config = loader.load()?;  // Merges global + local

// Load from specific file
let config = ConfigLoader::from_file("my-config.toml")?;

// Get default config
let config = ScarabConfig::default();
```

### Saving Configuration

```rust
use scarab_config::prelude::*;

let loader = ConfigLoader::new();
let config = ScarabConfig::default();

// Save to global config
loader.save_global(&config)?;

// Save to specific path
loader.save_to("custom.toml", &config)?;
```

### Validation

```rust
use scarab_config::prelude::*;

let config = load_config()?;

// Validate
match ConfigValidator::validate(&config) {
    Ok(_) => println!("Config is valid!"),
    Err(e) => {
        eprintln!("Validation error: {}", e);
        eprintln!("{}", e.help_text());
    }
}

// Auto-fix common issues
let fixed = ConfigValidator::auto_fix(config);
```

### Hot-Reload

```rust
use scarab_config::prelude::*;

let config = ScarabConfig::default();
let mut watcher = ConfigWatcher::new(config)?;

// Register callbacks
watcher.on_change(Box::new(|config| {
    apply_font_changes(&config.font);
}));

watcher.on_change(Box::new(|config| {
    apply_color_changes(&config.colors);
}));

// Start watching for changes
watcher.start()?;

// Get current config
let current = watcher.get_config();

// Manual reload
watcher.reload()?;

// Stop watching
watcher.stop();
```

## Migration

Configuration version migration is automatic:

```rust
use scarab_config::prelude::*;

// Old configs are automatically migrated to new format
let config = ConfigLoader::new().load()?;
```

## Testing

```bash
# Run all tests
cargo test

# Run integration tests
cargo test --test integration_tests

# Test config validation
cargo test validation

# Test hot-reload
cargo test watcher
```

## Performance

- **Config Load**: <10ms (cold), <1ms (cached)
- **Hot-Reload**: <100ms from file change to callback
- **Validation**: <1ms for typical configs
- **Memory**: ~1KB per config instance

## Troubleshooting

### Config Not Loading

```bash
# Check config locations
scarab config locations

# Validate config
scarab config validate

# Show current config
scarab config show
```

### Invalid Colors

Colors must be in `#RRGGBB` or `#RRGGBBAA` format:

```toml
foreground = "#ff5555"      # Valid
foreground = "ff5555"        # Invalid (missing #)
foreground = "#ff55"         # Invalid (too short)
foreground = "#gggggg"       # Invalid (not hex)
```

### Font Not Found

Ensure the font family name is exact:

```bash
# List available fonts (macOS)
fc-list : family | sort | uniq

# Or use Font Book.app
```

## API Reference

See the [generated documentation](https://docs.rs/scarab-config) for complete API reference.

## Contributing

Contributions welcome! Please:

1. Add tests for new features
2. Update schema.json for new config options
3. Add examples for new features
4. Update this README

## License

MIT OR Apache-2.0
