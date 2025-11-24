# Fusabi Configuration Examples

This directory contains example Fusabi configuration files for Scarab terminal emulator.

**IMPORTANT**: These are examples of **future functionality**. The Fusabi config DSL is not yet implemented. See [FUSABI_CONFIG.md](../../FUSABI_CONFIG.md) for the roadmap.

## Current Configuration (TOML)

Scarab currently uses TOML for configuration. See `crates/scarab-config/examples/` for working TOML examples.

**Example: ~/.config/scarab/config.toml**
```toml
[terminal]
default_shell = "/bin/zsh"
columns = 120
rows = 40
scrollback_lines = 10000

[font]
family = "JetBrains Mono"
size = 14.0
line_height = 1.2
```

## Future Fusabi Configuration

When the Fusabi config DSL is implemented, you'll be able to write `~/.config/scarab/config.fsx` instead.

### Example Files

| File | Description |
|------|-------------|
| `minimal.fsx` | Simplest possible config - just sets a theme |
| `standard.fsx` | Common configuration options (terminal, font, plugins) |
| `advanced.fsx` | Advanced features: custom plugins, keybindings, dynamic config |
| `custom-theme.fsx` | Theme creation and customization examples |

### Quick Start (Future)

1. **Install Fusabi runtime** (when available):
   ```bash
   cargo install fusabi-cli
   ```

2. **Copy example to config directory**:
   ```bash
   cp examples/fusabi-config/standard.fsx ~/.config/scarab/config.fsx
   ```

3. **Edit the config**:
   ```bash
   $EDITOR ~/.config/scarab/config.fsx
   ```

4. **Restart Scarab** (or wait for hot-reload):
   ```bash
   scarab-client
   ```

## Configuration Features (Planned)

### Type-Safe Configuration
Fusabi's F# type system catches configuration errors at compile-time:
```fsharp
// Error: Type mismatch - columns expects u16, not string
|> withTerminal { columns = "invalid" }  // ❌ Won't compile
```

### Programmatic Configuration
Use the full power of F# for dynamic config:
```fsharp
// Conditional configuration based on hostname
let config =
    if Environment.MachineName = "workstation" then
        ScarabConfig.create() |> withTerminal { columns = 180u16 }
    else
        ScarabConfig.create() |> withTerminal { columns = 120u16 }
```

### Composable Modules
Import and extend configurations:
```fsharp
open MyThemes
open MyPlugins

let config =
    ScarabConfig.create()
        |> withTheme myCustomDarkTheme
        |> withPlugins workPlugins
```

### Hot-Reload
Changes to `.fsx` files reload without restarting Scarab:
- Edit `config.fsx`
- Save the file
- Scarab automatically applies changes
- No need to restart terminal

## Learning Resources

- **Fusabi Language**: https://github.com/fusabi-lang/fusabi
- **F# Syntax**: https://learn.microsoft.com/en-us/dotnet/fsharp/
- **Scarab Config Roadmap**: [FUSABI_CONFIG.md](../../FUSABI_CONFIG.md)
- **Scarab Docs**: [docs/](../../docs/)

## Migration from TOML

Your existing `config.toml` will continue to work after Fusabi config is implemented. Migration is optional:

```fsharp
// Before (TOML):
// [terminal]
// columns = 120
// rows = 40

// After (Fusabi):
|> withTerminal {
    columns = 120u16
    rows = 40u16
}
```

## Comparison with WezTerm (Lua Config)

### WezTerm (Lua)
```lua
local wezterm = require 'wezterm'
local config = {}

config.font = wezterm.font 'JetBrains Mono'
config.font_size = 14.0
config.color_scheme = 'Gruvbox Dark'

return config
```

### Scarab (Fusabi)
```fsharp
open Scarab.Config
open Scarab.Themes

let config =
    ScarabConfig.create()
        |> withFont {
            family = "JetBrains Mono"
            size = 14.0f
        }
        |> withTheme (gruvboxDark())

Scarab.export config
```

**Advantages of Fusabi:**
- **Type safety**: Catch errors at compile-time
- **Performance**: Compiles to bytecode (faster than Lua)
- **Functional**: F# semantics reduce bugs
- **Same language**: Config and plugins use Fusabi

## Contributing Themes and Plugins

Once Fusabi config is implemented, you can contribute your themes and plugins:

1. Create `my-theme.fsx` or `my-plugin.fsx`
2. Test locally in `~/.config/scarab/`
3. Submit PR to `scarab/community-plugins/`
4. Community users can import: `open Community.MyTheme`

## Implementation Status

See [FUSABI_CONFIG.md](../../FUSABI_CONFIG.md) for detailed roadmap:

- ✅ **Phase 1**: TOML config integration (current)
- 🔄 **Phase 2**: Fusabi config DSL (next)
- 📅 **Phase 3**: Pure Fusabi runtime (future)

## Help and Support

- **Issues**: https://github.com/raibid-labs/scarab/issues
- **Discussions**: https://github.com/raibid-labs/scarab/discussions
- **Discord**: (coming soon)

---

**Last Updated**: 2025-01-23
**Fusabi Config Status**: Planned (not yet implemented)
