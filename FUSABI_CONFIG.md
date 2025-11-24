# Fusabi Configuration Architecture

## Vision: Fusabi as Scarab's Configuration Language

Scarab uses **Fusabi** (F# dialect) as its configuration language, similar to how WezTerm uses Lua. This provides:

- **Type-safe configuration** - Compile-time guarantees via F# type system
- **Hot-reload capability** - `.fsx` scripts reload without restarting
- **Dual execution modes** - Compiled `.fzb` for performance, interpreted `.fsx` for development
- **Programmatic config** - Full programming language for dynamic configuration
- **Plugin ecosystem** - Same language for config and extensions

## Architecture Layers

```
┌─────────────────────────────────────────────────────────┐
│           User Configuration (.fsx/.fzb)                │
│  - config.fsx (main config, like WezTerm's config.lua) │
│  - themes/gruvbox.fsx (theme definitions)               │
│  - plugins/git-prompt.fsx (custom plugins)              │
└─────────────────────────────────────────────────────────┘
                          ↓
┌─────────────────────────────────────────────────────────┐
│         Fusabi Config API (scarab-config)               │
│  - ScarabConfig builder DSL in Fusabi                   │
│  - Type-safe configuration construction                 │
│  - Validation and defaults                              │
└─────────────────────────────────────────────────────────┘
                          ↓
┌─────────────────────────────────────────────────────────┐
│              TOML Serialization Layer                   │
│  - Fusabi config → TOML (for compatibility)             │
│  - TOML → ScarabConfig struct (Rust)                    │
└─────────────────────────────────────────────────────────┘
                          ↓
┌─────────────────────────────────────────────────────────┐
│            Rust Core (daemon + client)                  │
│  - ScarabConfig struct consumed by Rust                 │
│  - Plugin hooks invoke Fusabi VM/interpreter            │
└─────────────────────────────────────────────────────────┘
```

## Configuration Flow

### Phase 1: Current State (TOML-based)
**Status**: ✅ Implemented in this commit

- User creates `~/.config/scarab/config.toml`
- Daemon and client load via `ConfigLoader::from_file()`
- Applied to terminal dimensions, shell, fonts, colors, etc.

**Example: config.toml**
```toml
[terminal]
default_shell = "/bin/zsh"
columns = 120
rows = 40
scrollback_lines = 10000

[font]
family = "JetBrains Mono"
size = 14.0

[colors]
theme = "gruvbox-dark"
```

### Phase 2: Fusabi Config DSL (Next Step)
**Status**: 🔄 Planned

Instead of TOML, users write `config.fsx`:

```fsharp
// ~/.config/scarab/config.fsx
open Scarab.Config

let config =
    ScarabConfig.create()
        |> withTerminal {
            shell = "/bin/zsh"
            columns = 120u16
            rows = 40u16
            scrollback = 10000u
        }
        |> withFont {
            family = "JetBrains Mono"
            size = 14.0f
            lineHeight = 1.2f
        }
        |> withTheme (Themes.gruvboxDark())
        |> withPlugins [
            Plugins.gitPrompt()
            Plugins.urlHighlighter()
        ]

// Export configuration
Scarab.export config
```

**Execution Flow:**
1. User launches `scarab-client`
2. Client invokes `fusabi-frontend` to interpret `config.fsx`
3. Fusabi script returns `ScarabConfig` data structure
4. Client serializes to TOML (for daemon compatibility)
5. Daemon reads TOML config as before

### Phase 3: Pure Fusabi Runtime (Future)
**Status**: 📅 Roadmap

- Daemon and client communicate ScarabConfig via shared memory (no TOML)
- `.fsx` scripts can be hot-reloaded without restart
- Fusabi VM runs config in both daemon and client

## Fusabi Config API Design

### Base Configuration Module

```fsharp
// Fusabi standard library for Scarab config
module Scarab.Config

type TerminalSettings = {
    shell: string
    columns: uint16
    rows: uint16
    scrollback: uint
    altScreen: bool
    scrollMultiplier: float32
    autoScroll: bool
}

type FontSettings = {
    family: string
    size: float32
    lineHeight: float32
    fallback: string list
}

type ColorScheme = {
    background: Color
    foreground: Color
    cursor: Color
    black: Color
    red: Color
    green: Color
    yellow: Color
    blue: Color
    magenta: Color
    cyan: Color
    white: Color
    // ... bright variants
}

type ScarabConfig = {
    terminal: TerminalSettings
    font: FontSettings
    colors: ColorScheme
    keybindings: Keybinding list
    ui: UiSettings
    plugins: Plugin list
    sessions: SessionSettings
}

// Builder API
let create : unit -> ScarabConfig
let withTerminal : TerminalSettings -> ScarabConfig -> ScarabConfig
let withFont : FontSettings -> ScarabConfig -> ScarabConfig
let withTheme : ColorScheme -> ScarabConfig -> ScarabConfig
let withPlugin : Plugin -> ScarabConfig -> ScarabConfig
let export : ScarabConfig -> unit
```

### Theme System

```fsharp
module Scarab.Themes

// Built-in themes (ship with scarab)
let gruvboxDark : unit -> ColorScheme
let gruvboxLight : unit -> ColorScheme
let nord : unit -> ColorScheme
let draculaTheme : unit -> ColorScheme
let monokaiPro : unit -> ColorScheme

// Custom theme builder
let customTheme (name: string) (colors: ColorScheme) : ColorScheme
```

### Plugin System

```fsharp
module Scarab.Plugins

// Base plugin interface
type Plugin = {
    name: string
    version: string
    onLoad: Context -> unit
    onOutput: string -> Context -> Action
    onInput: byte[] -> Context -> Action
    onResize: uint16 -> uint16 -> Context -> unit
    onUnload: unit -> unit
}

// Built-in plugins
let gitPrompt : unit -> Plugin
let urlHighlighter : unit -> Plugin
let outputLogger : string -> Plugin // path to log file

// Custom plugin loader
let loadPlugin : string -> Plugin // path to .fsx or .fzb
```

### Keybinding DSL

```fsharp
module Scarab.Keybindings

type Modifier = Ctrl | Shift | Alt | Super
type Key = Char of char | F of int | Enter | Tab | Escape | ...

type Keybinding = {
    modifiers: Modifier list
    key: Key
    action: Action
}

// Action types
type Action =
    | CopySelection
    | PasteClipboard
    | ScrollUp of int
    | ScrollDown of int
    | SplitHorizontal
    | SplitVertical
    | NewTab
    | CloseTab
    | NextTab
    | PrevTab
    | ShowCommandPalette
    | ShowLinkHints
    | Custom of (Context -> unit)

// Builder
let bind : Modifier list -> Key -> Action -> Keybinding
```

## Example Configurations

### Minimal Config

```fsharp
// ~/.config/scarab/config.fsx
open Scarab.Config
open Scarab.Themes

let config =
    ScarabConfig.create()
        |> withTheme (gruvboxDark())

Scarab.export config
```

### Advanced Config with Custom Plugins

```fsharp
// ~/.config/scarab/config.fsx
open Scarab.Config
open Scarab.Themes
open Scarab.Plugins
open Scarab.Keybindings

// Custom plugin: Auto-save terminal output
let autoSavePlugin =
    let logPath = "/home/user/terminal-logs"
    {
        name = "auto-save"
        version = "1.0.0"
        onLoad = fun ctx ->
            ctx.log Info "Auto-save plugin loaded"
        onOutput = fun line ctx ->
            System.IO.File.AppendAllText(logPath, line + "\n")
            Continue
        onInput = fun _ _ -> Continue
        onResize = fun _ _ _ -> ()
        onUnload = fun () -> ()
    }

// Custom keybinding: Clear terminal and run 'ls'
let customClear ctx =
    ctx.sendInput "\x0cls\n" // Ctrl+L then 'ls\n'

let config =
    ScarabConfig.create()
        |> withTerminal {
            shell = "/usr/bin/fish"
            columns = 140u16
            rows = 50u16
            scrollback = 50000u
            altScreen = true
            scrollMultiplier = 3.0f
            autoScroll = true
        }
        |> withFont {
            family = "Fira Code"
            size = 13.0f
            lineHeight = 1.3f
            fallback = ["JetBrains Mono"; "monospace"]
        }
        |> withTheme (nord())
        |> withPlugins [
            gitPrompt()
            urlHighlighter()
            autoSavePlugin
        ]
        |> withKeybinding (bind [Ctrl; Shift] (Char 'K') (Custom customClear))

Scarab.export config
```

### Theme Customization

```fsharp
// ~/.config/scarab/themes/my-theme.fsx
open Scarab.Config
open Scarab.Themes

let myCustomTheme =
    customTheme "my-dark-theme" {
        background = rgb 0x1e 0x1e 0x1e
        foreground = rgb 0xd4 0xd4 0xd4
        cursor = rgb 0xff 0xff 0xff
        black = rgb 0x00 0x00 0x00
        red = rgb 0xcc 0x66 0x66
        green = rgb 0xb5 0xbd 0x68
        yellow = rgb 0xf0 0xc6 0x74
        blue = rgb 0x81 0xa2 0xbe
        magenta = rgb 0xb2 0x94 0xbb
        cyan = rgb 0x8a 0xbe 0xb7
        white = rgb 0xff 0xff 0xff
        // ... bright variants
    }

// Use in main config
let config =
    ScarabConfig.create()
        |> withTheme myCustomTheme

Scarab.export config
```

## Implementation Roadmap

### ✅ Phase 1: TOML Integration (This Commit)
- [x] Add `columns` and `rows` to `TerminalConfig`
- [x] Load config in daemon via `ConfigLoader::from_file()`
- [x] Apply config to PTY dimensions and shell
- [x] Load config in client
- [x] Make `ScarabConfig` a Bevy `Resource`
- [x] Calculate window size from terminal dimensions

### 🔄 Phase 2: Fusabi Config API (Next)
1. **Define Fusabi Config Module** (`scarab-fusabi-config` crate)
   - Create Fusabi standard library for config
   - Implement builder API in Fusabi
   - Add type definitions for all config structures

2. **Fusabi → TOML Bridge**
   - Implement serialization from Fusabi config to TOML
   - Create `fusabi_config_loader` that:
     - Interprets `config.fsx`
     - Extracts `ScarabConfig` data
     - Serializes to TOML

3. **Config Loading Flow**
   - Check for `config.fsx` first, fall back to `config.toml`
   - Cache compiled `.fzb` bytecode for faster loads
   - Hot-reload detection for `.fsx` changes

### 📅 Phase 3: Pure Fusabi Runtime (Future)
1. **IPC Protocol Extension**
   - Add `ConfigUpdate` message to protocol
   - Shared memory config section

2. **Hot-Reload System**
   - File watcher for `config.fsx` changes
   - Incremental config updates (no full restart)
   - Validation before applying changes

3. **Plugin Marketplace**
   - Community Fusabi plugins
   - Package manager for `.fsx` scripts
   - Sandboxed plugin execution

## Benefits of Fusabi Config

### vs TOML
- **Programmatic**: Conditionals, loops, functions for dynamic config
- **Type-safe**: Compile-time checks prevent invalid configs
- **Composable**: Import and extend configs from modules
- **Reactive**: Configs can respond to environment variables, system state

### vs Lua (WezTerm approach)
- **Performance**: Fusabi compiles to bytecode, faster than interpreted Lua
- **Functional**: F# semantics (immutable, pure functions) reduce bugs
- **Ecosystem**: Same language for plugins and config (no context switching)
- **Modern**: Pattern matching, discriminated unions, type inference

## Migration Guide for Users

### Current (TOML) Users
Your existing `~/.config/scarab/config.toml` will continue to work. No changes required.

### Migrating to Fusabi Config
1. Install Fusabi runtime: `cargo install fusabi-cli`
2. Create `~/.config/scarab/config.fsx`
3. Use the Fusabi DSL (see examples above)
4. Remove old `config.toml` (optional)

### Hybrid Approach (Recommended)
- Keep `config.toml` for base config
- Use `config.fsx` for advanced features (plugins, dynamic keybindings)
- Fusabi config merges with TOML (Fusabi wins on conflicts)

## Technical Details

### Fusabi VM Integration Points

**Daemon Side:**
- Load `.fzb` plugins at startup
- Execute plugin hooks on terminal events
- No interpreted `.fsx` (performance critical)

**Client Side:**
- Interpret `config.fsx` for hot-reload
- Compile to `.fzb` for caching
- Load `.fsx` UI plugins (themes, keybindings)

### Security Considerations
- Fusabi scripts run in sandboxed VM (no arbitrary syscalls)
- Plugin API restricted to terminal operations
- File I/O limited to config directories
- Network access disabled by default

### Performance Targets
- Config load time: < 50ms (interpreted `.fsx`)
- Config reload time: < 10ms (compiled `.fzb` cache)
- Plugin overhead: < 1ms per terminal event

## Next Steps

1. **Implement Fusabi Config Module** - Create `scarab-fusabi-config` crate
2. **Write Standard Library** - Fusabi API for `ScarabConfig` construction
3. **Build Fusabi → TOML Bridge** - Serialization layer
4. **Update Documentation** - User guide for Fusabi config
5. **Create Example Configs** - Ship default `config.fsx` templates
6. **Test Hot-Reload** - Verify config changes apply without restart

---

**Related Files:**
- `/home/beengud/raibid-labs/scarab/crates/scarab-config/` - Rust config system
- `/home/beengud/raibid-labs/scarab/crates/scarab-daemon/src/plugin_manager/fusabi_adapter.rs` - Fusabi plugin adapter
- `/home/beengud/raibid-labs/scarab/ROADMAP.md` - Overall project roadmap
