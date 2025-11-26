# Fusabi Configuration Architecture

## Vision: Fusabi as Scarab's Native Configuration Language

Scarab uses **Fusabi** (F# dialect) as its primary configuration language, similar to how WezTerm uses Lua. This provides:

- **Type-safe configuration** - Compile-time guarantees via F# type system
- **Hot-reload capability** - `.fsx` scripts reload without restarting
- **Dual execution modes** - Compiled `.fzb` for performance, interpreted `.fsx` for development
- **Programmatic config** - Full programming language for dynamic configuration
- **Unified ecosystem** - Same language for config and plugins (no context switching)
- **Direct Rust integration** - No serialization overhead, Fusabi values map directly to Rust structs

## Architecture: Native Records

```
┌─────────────────────────────────────────────────────────┐
│           User Configuration (.fsx)                     │
│  - config.fsx (main config, like WezTerm's config.lua)  │
└─────────────────────────────────────────────────────────┘
                          ↓
┌─────────────────────────────────────────────────────────┐
│         Fusabi Interpreter/VM (fusabi-frontend)         │
│  - Compile config.fsx script                            │
│  - Execute script                                       │
│  - Script evaluates to a Record value                   │
└─────────────────────────────────────────────────────────┘
                          ↓
┌─────────────────────────────────────────────────────────┐
│            Rust Core (scarab-config)                    │
│  - Receives Value::Record from VM                       │
│  - Extracts fields (terminal, font, etc.)               │
│  - Converts Value types to Rust types                   │
│  - Applies to ScarabConfig struct                       │
└─────────────────────────────────────────────────────────┘
```

**Key Principle**: The configuration script is a program that *returns* the configuration object. This avoids the need for complex host function registration for basic configuration.

## Configuration Flow

### Phase 1: Bootstrap with TOML (Legacy)
**Status**: ✅ Implemented (Deprecated)

**Purpose**: Temporary fallback while Fusabi config was being developed.

```toml
[terminal]
default_shell = "/bin/zsh"
```

### Phase 2: Native Fusabi Records (Current)
**Status**: ✅ Implemented

Users write `~/.config/scarab/config.fsx` that returns a record:

```fsharp
// ~/.config/scarab/config.fsx

let terminal = {
    DefaultShell = "/bin/zsh";
    ScrollbackLines = 10000;
    Columns = 120
}

let font = {
    Family = "JetBrains Mono";
    Size = 14.0
}

// Return the configuration object
{
    terminal = terminal;
    font = font
}
```

**Execution Flow:**
1. Daemon/client checks for `~/.config/scarab/config.fsx`
2. If found, loads and compiles using `fusabi-frontend`
3. Executes in `fusabi-vm`
4. The final expression in the script is returned as the result
5. `FusabiConfigLoader` inspects this result record and populates `ScarabConfig`

### Phase 3: Ergonomic DSL (Future)
**Status**: 📅 Planned

Provide F#-style builder API or DSL for nicer syntax:

```fsharp
open Scarab.Config

ScarabConfig.create()
    |> withTerminal { ... }
    |> withFont { ... }
```

This will require implementing Fusabi modules (`Scarab.Config`) that provide these helper functions, potentially backed by host functions or pure Fusabi code.

## Implementation Roadmap

### ✅ Phase 1: TOML Bootstrap (Complete)
- [x] Load TOML config in daemon and client

### ✅ Phase 2: Native Records (Complete)
- [x] Integrate `fusabi-frontend` and `fusabi-vm` 0.12.0
- [x] Implement `FusabiConfigLoader`
- [x] Implement value extraction for all config sections
- [x] Verify record return strategy works with VM

### 📅 Phase 3: DSL & Host Functions (Future)
- [ ] Implement host functions for advanced plugin interaction
- [ ] Create `Scarab.Config` prelude module
- [ ] Support `|>` pipe syntax for config (requires module support in Fusabi)

## Migration Guide

### From TOML to Fusabi

**Before (config.toml):**
```toml
[terminal]
default_shell = "/bin/zsh"
columns = 120
```

**After (config.fsx):**
```fsharp
let terminal = {
    DefaultShell = "/bin/zsh";
    Columns = 120
}
{ terminal = terminal }
```

## Comparison with WezTerm (Lua Config)

### WezTerm (Lua)
```lua
local config = {}
config.font = wezterm.font 'JetBrains Mono'
return config
```

### Scarab (Fusabi)
```fsharp
let font = { Family = "JetBrains Mono" }
{ font = font }
```

**Advantages:**
- **Type Safety**: Fusabi catches type errors at compile time
- **Performance**: Compiled bytecode execution
- **Consistency**: Use the same language for plugins and config

---

**Related Files:**
- `crates/scarab-config/src/fusabi_loader.rs` - Implementation of the loader
- `examples/fusabi-config/` - Example configuration files