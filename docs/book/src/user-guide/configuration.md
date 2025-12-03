# Configuration

Scarab uses a hierarchical configuration system with TOML files.

## Configuration Files

For complete configuration documentation, see:
- [Configuration Guide](../../configuration.md)
- [Configuration Schema Reference](../reference/config-schema.md)

## Quick Start

Configuration files are located in standard XDG directories:

```
~/.config/scarab/config.toml        # User configuration
/etc/scarab/config.toml              # System-wide defaults
```

## Basic Configuration

```toml
[appearance]
theme = "ayu-dark"
font_family = "JetBrains Mono"
font_size = 14.0

[behavior]
shell = "/bin/zsh"
scrollback_lines = 10000

[keybindings]
new_tab = "Ctrl+Shift+T"
close_tab = "Ctrl+Shift+W"
```

## Advanced Topics

- Custom color schemes
- Plugin configuration
- Per-profile settings
- Environment variables

For detailed options, see the [Configuration Schema](../reference/config-schema.md).
