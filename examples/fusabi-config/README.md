# Fusabi Configuration Examples

This directory contains example Fusabi configuration files (`.fsx`) for the Scarab terminal emulator.

**Status**: ✅ **Implemented**. You can use `~/.config/scarab/config.fsx`.

## Configuration Syntax

Scarab uses the **Fusabi** language (an F# dialect) for configuration. The configuration file must evaluate to a **record** containing the configuration sections you wish to override.

### Basic Structure

```fsharp
// ~/.config/scarab/config.fsx

// Define sections using F# record syntax
let terminal = {
    DefaultShell = "/bin/zsh";
    ScrollbackLines = 10000;
    AltScreen = true
}

let font = {
    Family = "JetBrains Mono";
    Size = 14.0
}

// Return the configuration object at the end of the script
{
    terminal = terminal;
    font = font
}
```

### Example Files

| File | Description |
|------|-------------|
| `minimal.fsx` | Simplest possible config |
| `standard.fsx` | Common configuration options (terminal, font, colors) |

### Quick Start

1. **Copy example to config directory**:
   ```bash
   mkdir -p ~/.config/scarab
   cp examples/fusabi-config/standard.fsx ~/.config/scarab/config.fsx
   ```

2. **Edit the config**:
   ```bash
   $EDITOR ~/.config/scarab/config.fsx
   ```

3. **Restart Scarab**:
   The new configuration will be loaded on startup.

## Available Configuration Sections

All sections are optional. If omitted, defaults are used.

- **terminal**: Shell, scrollback, dimensions
- **font**: Family, size, line height, fallbacks
- **colors**: Theme, custom colors, opacity, palette
- **ui**: Link hints, animations, tabs, cursor style
- **keybindings**: Leader key, shortcuts
- **plugins**: Enabled plugins
- **sessions**: Auto-save, restore settings

See `standard.fsx` for a comprehensive list of fields.

## Comparison with WezTerm (Lua Config)

### WezTerm (Lua)
```lua
local config = {}
config.font = wezterm.font 'JetBrains Mono'
config.font_size = 14.0
return config
```

### Scarab (Fusabi)
```fsharp
let font = {
    Family = "JetBrains Mono";
    Size = 14.0
}
{ font = font }
```

**Advantages of Fusabi:**
- **Type safety**: Catch errors at compile-time (e.g., assigning string to integer field)
- **Performance**: Compiles to bytecode
- **Consistent**: Same language for config and plugins

## Implementation Status

- ✅ **Phase 1**: TOML config integration (Legacy)
- ✅ **Phase 2**: Fusabi config loader (Active)
- 📅 **Phase 3**: Advanced DSL and Host Functions (Planned)

Scarab will automatically prefer `config.fsx` if present. If not, it falls back to defaults (or `config.toml` if supported).