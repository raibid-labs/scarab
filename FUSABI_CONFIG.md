# Fusabi Configuration Architecture

## Vision: Fusabi as Scarab's Native Configuration Language

Scarab uses **Fusabi** (F# dialect) as its primary configuration language, similar to how WezTerm uses Lua. This provides:

- **Type-safe configuration** - Compile-time guarantees via F# type system
- **Hot-reload capability** - `.fsx` scripts reload without restarting
- **Dual execution modes** - Compiled `.fzb` for performance, interpreted `.fsx` for development
- **Programmatic config** - Full programming language for dynamic configuration
- **Unified ecosystem** - Same language for config and plugins (no context switching)
- **Direct Rust integration** - No serialization overhead, Fusabi calls Rust host functions directly

## Architecture: Direct Fusabi → Rust

```
┌─────────────────────────────────────────────────────────┐
│           User Configuration (.fsx)                     │
│  - config.fsx (main config, like WezTerm's config.lua) │
│  - themes/gruvbox.fsx (theme definitions)               │
│  - plugins/git-prompt.fsx (custom plugins)              │
└─────────────────────────────────────────────────────────┘
                          ↓
┌─────────────────────────────────────────────────────────┐
│         Fusabi Interpreter/VM (fusabi-frontend)         │
│  - Execute config.fsx script                            │
│  - Call Scarab host functions (implemented in Rust)     │
│  - Return ScarabConfig struct directly                  │
└─────────────────────────────────────────────────────────┘
                          ↓
┌─────────────────────────────────────────────────────────┐
│      Rust Host Functions (scarab-fusabi-config)         │
│  - config_create() -> ScarabConfig                      │
│  - config_set_terminal(config, shell, cols, rows)       │
│  - config_set_font(config, family, size)                │
│  - config_add_plugin(config, plugin)                    │
│  - config_export(config) -> Returns to Rust             │
└─────────────────────────────────────────────────────────┘
                          ↓
┌─────────────────────────────────────────────────────────┐
│            Rust Core (daemon + client)                  │
│  - Receives ScarabConfig struct directly from Fusabi    │
│  - No serialization, no TOML, no intermediate format    │
│  - Apply config to terminal, UI, plugins                │
└─────────────────────────────────────────────────────────┘
```

**Key Principle**: Fusabi scripts call Rust functions directly via the host function API. There is **no TOML bridge**, no serialization layer, no intermediate format. Fusabi is the native configuration format.

## Configuration Flow

### Phase 1: Bootstrap with TOML (Current State)
**Status**: ✅ Implemented

**Purpose**: Temporary fallback while Fusabi config is being developed.

```rust
// daemon/src/main.rs (current implementation)
let config_path = PathBuf::from(&home_dir).join(".config/scarab/config.toml");

let config = if config_path.exists() {
    ConfigLoader::from_file(&config_path)?
} else {
    ScarabConfig::default()
};
```

**TOML Example** (temporary, will be replaced):
```toml
[terminal]
default_shell = "/bin/zsh"
columns = 120
rows = 40
scrollback_lines = 10000

[font]
family = "JetBrains Mono"
size = 14.0
```

**This is temporary scaffolding.** TOML will be deprecated once Fusabi config is implemented.

### Phase 2: Pure Fusabi Configuration (Next Step)
**Status**: 🔄 In Progress (architecture defined, implementation next)

Users write `~/.config/scarab/config.fsx` that calls Rust host functions:

```fsharp
// ~/.config/scarab/config.fsx
open Scarab.Config

// Create base config by calling Rust host function
let config = hostCall "config_create" []

// Set terminal settings via Rust host function
let config = hostCall "config_set_terminal" [
    config
    "/bin/zsh"      // shell
    120u16          // columns
    40u16           // rows
    10000u          // scrollback
]

// Set font via Rust host function
let config = hostCall "config_set_font" [
    config
    "JetBrains Mono"  // family
    14.0f             // size
    1.2f              // line height
]

// Set theme via Rust host function
let config = hostCall "config_set_theme" [
    config
    "gruvbox-dark"
]

// Export config back to Rust (returns ScarabConfig struct)
hostCall "config_export" [config]
```

**Execution Flow:**
1. Daemon/client checks for `~/.config/scarab/config.fsx`
2. If found, invoke `fusabi-frontend` interpreter
3. Fusabi script executes, calling Rust host functions
4. Host functions build `ScarabConfig` struct in Rust memory
5. `config_export` returns the struct to Rust code
6. **No serialization, no TOML, direct struct transfer**

### Phase 3: Ergonomic Fusabi API (Future)
**Status**: 📅 Planned (requires Fusabi language features)

Provide F#-style builder API that wraps host function calls:

```fsharp
// ~/.config/scarab/config.fsx
open Scarab.Config
open Scarab.Themes
open Scarab.Plugins

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

Scarab.export config
```

This is **syntactic sugar** over Phase 2's host function calls. Under the hood:
- `ScarabConfig.create()` → `hostCall "config_create" []`
- `withTerminal {...}` → `hostCall "config_set_terminal" [...]`
- `Themes.gruvboxDark()` → `hostCall "theme_gruvbox_dark" []`

**If Fusabi lacks features** for this ergonomic API (e.g., record syntax, pipe operators, modules), we **file issues on fusabi-lang/fusabi** requesting those features.

## Host Functions API (Rust Side)

These functions are implemented in Rust and callable from Fusabi:

### Configuration Construction

```rust
// scarab-fusabi-config/src/host_functions.rs

use fusabi_vm::{HostRegistry, Value};
use scarab_config::{ScarabConfig, TerminalConfig, FontConfig, ColorConfig};

pub fn register_config_host_functions(registry: &mut HostRegistry) {
    // Create empty config with defaults
    registry.register("config_create", |_args| {
        let config = Box::new(ScarabConfig::default());
        Value::UserData(config)
    });

    // Set terminal configuration
    registry.register("config_set_terminal", |args| {
        let mut config = args[0].as_userdata_mut::<ScarabConfig>();
        let shell = args[1].as_string();
        let cols = args[2].as_int() as u16;
        let rows = args[3].as_int() as u16;
        let scrollback = args[4].as_int() as u32;

        config.terminal.default_shell = shell.to_string();
        config.terminal.columns = cols;
        config.terminal.rows = rows;
        config.terminal.scrollback_lines = scrollback;

        Value::UserData(Box::new(config.clone()))
    });

    // Set font configuration
    registry.register("config_set_font", |args| {
        let mut config = args[0].as_userdata_mut::<ScarabConfig>();
        let family = args[1].as_string();
        let size = args[2].as_float() as f32;
        let line_height = args[3].as_float() as f32;

        config.font.family = family.to_string();
        config.font.size = size;
        config.font.line_height = line_height;

        Value::UserData(Box::new(config.clone()))
    });

    // Set color theme by name
    registry.register("config_set_theme", |args| {
        let mut config = args[0].as_userdata_mut::<ScarabConfig>();
        let theme_name = args[1].as_string();

        // Load built-in theme
        config.colors = match theme_name {
            "gruvbox-dark" => ColorConfig::gruvbox_dark(),
            "nord" => ColorConfig::nord(),
            "dracula" => ColorConfig::dracula(),
            _ => ColorConfig::default(),
        };

        Value::UserData(Box::new(config.clone()))
    });

    // Export configuration (return to Rust)
    registry.register("config_export", |args| {
        // Simply return the config as-is
        args[0].clone()
    });
}
```

### Theme Functions

```rust
// Built-in theme constructors
registry.register("theme_gruvbox_dark", |_args| {
    let theme = ColorConfig::gruvbox_dark();
    Value::UserData(Box::new(theme))
});

registry.register("theme_nord", |_args| {
    let theme = ColorConfig::nord();
    Value::UserData(Box::new(theme))
});

// Custom theme builder
registry.register("theme_custom", |args| {
    let name = args[0].as_string();
    let bg = args[1].as_string();  // hex color
    let fg = args[2].as_string();
    // ... more colors

    let theme = ColorConfig {
        background: Color::from_hex(bg),
        foreground: Color::from_hex(fg),
        // ... build custom theme
    };

    Value::UserData(Box::new(theme))
});
```

### Plugin Functions

```rust
registry.register("plugin_git_prompt", |_args| {
    let plugin = GitPromptPlugin::new();
    Value::UserData(Box::new(plugin))
});

registry.register("config_add_plugin", |args| {
    let mut config = args[0].as_userdata_mut::<ScarabConfig>();
    let plugin = args[1].as_userdata::<Box<dyn Plugin>>();

    config.plugins.enabled.push(plugin.metadata().name.clone());

    Value::UserData(Box::new(config.clone()))
});
```

## Configuration Loading (Rust Side)

### Daemon and Client Main

```rust
// daemon/src/main.rs and client/src/main.rs

use fusabi_frontend::Interpreter;
use scarab_fusabi_config::register_config_host_functions;

fn load_config() -> Result<ScarabConfig> {
    let home_dir = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());

    // Priority 1: Check for config.fsx (Fusabi)
    let config_fsx = PathBuf::from(&home_dir).join(".config/scarab/config.fsx");
    if config_fsx.exists() {
        println!("Loading Fusabi config from: {}", config_fsx.display());
        return load_fusabi_config(&config_fsx);
    }

    // Priority 2: Check for config.toml (legacy fallback)
    let config_toml = PathBuf::from(&home_dir).join(".config/scarab/config.toml");
    if config_toml.exists() {
        println!("Loading TOML config (legacy): {}", config_toml.display());
        println!("Consider migrating to config.fsx for better features");
        return ConfigLoader::from_file(&config_toml);
    }

    // Priority 3: Use defaults
    println!("No config found, using defaults");
    Ok(ScarabConfig::default())
}

fn load_fusabi_config(path: &Path) -> Result<ScarabConfig> {
    // Create Fusabi interpreter
    let mut interpreter = Interpreter::new();

    // Register Scarab host functions
    let mut registry = HostRegistry::new();
    register_config_host_functions(&mut registry);
    interpreter.set_host_registry(registry);

    // Read and execute config.fsx
    let script = std::fs::read_to_string(path)?;
    let result = interpreter.eval(&script)?;

    // Extract ScarabConfig from Fusabi return value
    let config = result.as_userdata::<ScarabConfig>()?;

    Ok(config)
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    // Load configuration (Fusabi or TOML)
    let config = load_config()?;

    // Use config as before
    println!("Shell: {}", config.terminal.default_shell);
    println!("Dimensions: {}x{}", config.terminal.columns, config.terminal.rows);

    // ... rest of daemon/client initialization
}
```

## Example Configurations

### Minimal Config

```fsharp
// ~/.config/scarab/config.fsx
// Simplest possible config - just theme

let config = hostCall "config_create" []
let config = hostCall "config_set_theme" [config; "gruvbox-dark"]
hostCall "config_export" [config]
```

### Standard Config

```fsharp
// ~/.config/scarab/config.fsx
// Common configuration options

let config = hostCall "config_create" []

// Terminal settings
let config = hostCall "config_set_terminal" [
    config
    "/bin/zsh"
    120u16
    40u16
    10000u
]

// Font settings
let config = hostCall "config_set_font" [
    config
    "JetBrains Mono"
    14.0f
    1.2f
]

// Theme
let config = hostCall "config_set_theme" [config; "gruvbox-dark"]

// Export
hostCall "config_export" [config]
```

### Advanced Config (Future - requires Fusabi language features)

```fsharp
// ~/.config/scarab/config.fsx
// This syntax requires Fusabi to support:
// - Record syntax
// - Pipe operators
// - Module system
// - Function composition
//
// If Fusabi doesn't support these, we file issues on fusabi-lang/fusabi

open Scarab.Config
open Scarab.Themes
open Scarab.Plugins

// Custom plugin
let autoSavePlugin = {
    name = "auto-save"
    version = "1.0.0"
    onOutput = fun line ctx ->
        System.IO.File.AppendAllText("/tmp/terminal.log", line + "\n")
        Continue
}

let config =
    ScarabConfig.create()
        |> withTerminal {
            shell = "/usr/bin/fish"
            columns = 140u16
            rows = 50u16
            scrollback = 50000u
        }
        |> withFont {
            family = "Fira Code"
            size = 13.0f
            lineHeight = 1.3f
        }
        |> withTheme (Themes.nord())
        |> withPlugins [
            Plugins.gitPrompt()
            Plugins.urlHighlighter()
            autoSavePlugin
        ]

Scarab.export config
```

**Note**: If the ergonomic syntax above doesn't work due to missing Fusabi features, we fall back to Phase 2's host function calls and **create issues on fusabi-lang/fusabi** requesting the features we need.

## Implementation Roadmap

### ✅ Phase 1: TOML Bootstrap (Complete)
- [x] Add `columns` and `rows` to `TerminalConfig`
- [x] Load TOML config in daemon and client
- [x] Apply config to PTY dimensions and shell
- [x] Make `ScarabConfig` a Bevy `Resource`
- [x] Calculate window size from terminal dimensions

**Status**: TOML works as temporary fallback. Will remain supported as legacy format.

### 🔄 Phase 2: Direct Fusabi Host Functions (Next)

**Priority 1: Core Host Functions**
- [ ] Create `scarab-fusabi-config` crate
- [ ] Implement `config_create()` host function
- [ ] Implement `config_set_terminal()` host function
- [ ] Implement `config_set_font()` host function
- [ ] Implement `config_set_theme()` host function
- [ ] Implement `config_export()` host function
- [ ] Register host functions with Fusabi interpreter

**Priority 2: Config Loading**
- [ ] Implement `load_fusabi_config()` in daemon
- [ ] Implement `load_fusabi_config()` in client
- [ ] Test with minimal config example
- [ ] Test with standard config example
- [ ] Verify config priority: `.fsx` → `.toml` → defaults

**Priority 3: Built-in Themes**
- [ ] Implement `theme_gruvbox_dark()` host function
- [ ] Implement `theme_nord()` host function
- [ ] Implement `theme_dracula()` host function
- [ ] Implement `theme_custom()` for user themes
- [ ] Add theme functions to host registry

**Priority 4: Plugin Integration**
- [ ] Implement `plugin_git_prompt()` host function
- [ ] Implement `plugin_url_highlighter()` host function
- [ ] Implement `config_add_plugin()` host function
- [ ] Test plugin loading from Fusabi config

**Blockers / Issues to File on Fusabi:**
- Check if `fusabi-frontend` supports `UserData` for passing Rust structs
- Check if host functions can mutate Rust structs in-place
- Check if Fusabi supports returning complex Rust types

If any of these are missing, **file issues on fusabi-lang/fusabi**.

### 📅 Phase 3: Ergonomic Builder API (Future)

**Depends on Fusabi Language Features:**
- [ ] Record syntax: `{ field = value }`
- [ ] Pipe operators: `config |> withTerminal {...}`
- [ ] Module system: `open Scarab.Config`
- [ ] List syntax: `[item1; item2]`
- [ ] Function composition

**Implementation:**
- [ ] Create Fusabi standard library module `Scarab.Config`
- [ ] Implement builder functions that wrap host function calls
- [ ] Create `Scarab.Themes` module with theme constructors
- [ ] Create `Scarab.Plugins` module with plugin builders
- [ ] Ship standard library with Scarab installation

**If Fusabi lacks required features:**
→ **File feature requests on fusabi-lang/fusabi**
→ Provide use cases and examples from Scarab config needs
→ Continue using Phase 2's host function API until features land

### 📅 Phase 4: Hot-Reload (Future)
- [ ] File watcher for `config.fsx` changes
- [ ] Reload config without restarting daemon/client
- [ ] Validate config before applying
- [ ] Graceful fallback if reload fails
- [ ] Apply delta updates (only changed settings)

## Dependency on Fusabi Features

### Required Features (Phase 2)
These are **essential** and if missing, we file issues:

1. **Host Function API**: Register Rust functions callable from Fusabi
2. **UserData Support**: Pass Rust structs between Fusabi and Rust
3. **Basic Types**: Strings, integers, floats, booleans
4. **Function Calls**: `hostCall "function_name" [args]`

### Desired Features (Phase 3)
These make configs **ergonomic** and if missing, we file enhancement requests:

1. **Record Syntax**: `{ field = value; field2 = value2 }`
2. **Pipe Operator**: `value |> function |> function2`
3. **Module System**: `open Module` to import definitions
4. **List Syntax**: `[item1; item2; item3]`
5. **Pattern Matching**: `match value with | Pattern -> result`
6. **Type Inference**: Infer types without annotations

### Enhancement Features (Phase 4)
These are **nice-to-have** for advanced use cases:

1. **File I/O**: Read/write files from Fusabi (sandboxed)
2. **Environment Variables**: Access `$HOME`, `$SHELL`, etc.
3. **Conditional Compilation**: `#if DEBUG` style directives
4. **Async Support**: `async/await` for plugin operations

**Process for Missing Features:**
1. Try to implement config with current Fusabi features
2. If blocked, check if workaround exists
3. If no workaround, file detailed issue on `fusabi-lang/fusabi`
4. Include Scarab use case in issue description
5. Continue with fallback approach (Phase 2 API) until feature lands

## Migration Guide

### From TOML to Fusabi

**Before (config.toml):**
```toml
[terminal]
default_shell = "/bin/zsh"
columns = 120
rows = 40
```

**After (config.fsx - Phase 2 API):**
```fsharp
let config = hostCall "config_create" []
let config = hostCall "config_set_terminal" [config; "/bin/zsh"; 120u16; 40u16; 10000u]
hostCall "config_export" [config]
```

**After (config.fsx - Phase 3 API, if Fusabi supports it):**
```fsharp
open Scarab.Config

ScarabConfig.create()
    |> withTerminal { shell = "/bin/zsh"; columns = 120u16; rows = 40u16 }
    |> Scarab.export
```

**Migration Steps:**
1. Keep existing `config.toml` as backup
2. Create new `config.fsx` with desired settings
3. Test by launching `scarab-client`
4. If config.fsx works, optionally delete config.toml
5. TOML will remain supported as legacy fallback

## Benefits Over TOML

| Feature | TOML | Fusabi |
|---------|------|--------|
| Type Safety | ❌ Runtime errors | ✅ Compile-time checks |
| Programmatic | ❌ Static only | ✅ Functions, conditionals, loops |
| Composable | ❌ Limited includes | ✅ Module system, imports |
| Dynamic | ❌ No logic | ✅ Respond to env vars, system state |
| Plugin Language | ❌ Different (Rust/Fusabi) | ✅ Same language |
| Hot-Reload | ⚠️ Requires full restart | ✅ Live config updates |
| IDE Support | ⚠️ Basic syntax | ✅ Full F# tooling |

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

### Scarab (Fusabi - Phase 2)
```fsharp
let config = hostCall "config_create" []
let config = hostCall "config_set_font" [config; "JetBrains Mono"; 14.0f; 1.2f]
let config = hostCall "config_set_theme" [config; "gruvbox-dark"]
hostCall "config_export" [config]
```

### Scarab (Fusabi - Phase 3)
```fsharp
open Scarab.Config

ScarabConfig.create()
    |> withFont { family = "JetBrains Mono"; size = 14.0f }
    |> withTheme (Themes.gruvboxDark())
    |> Scarab.export
```

**Advantages over WezTerm/Lua:**
- **Type Safety**: F# catches errors at compile-time, Lua fails at runtime
- **Performance**: Fusabi compiles to bytecode, faster than interpreted Lua
- **Functional**: Immutable by default, reduces state bugs
- **Same Language**: Configs and plugins both use Fusabi (WezTerm mixes Lua config with Rust plugins)
- **Modern Syntax**: Pattern matching, discriminated unions, pipe operators

## Next Steps

1. **Start Phase 2 Implementation** - Create `scarab-fusabi-config` crate
2. **Test Fusabi Host Functions** - Verify we can call Rust from Fusabi
3. **Identify Missing Fusabi Features** - Document what we need
4. **File Issues on fusabi-lang/fusabi** - Request critical features
5. **Implement Minimal Config Example** - Get `.fsx` loading working
6. **Deprecate TOML** - Mark as legacy, encourage Fusabi migration

## Contributing

When contributing config features:

1. **Always prefer Fusabi-native solutions** - No custom DSLs or intermediate formats
2. **File Fusabi issues for missing features** - Don't work around, fix upstream
3. **Test with Phase 2 API** - Ensure host function approach works
4. **Document Fusabi requirements** - Make dependencies explicit
5. **Maintain TOML fallback** - Keep legacy support for migration period

---

**Related Files:**
- `crates/scarab-config/` - Rust config structs (target for Fusabi)
- `crates/scarab-daemon/src/plugin_manager/fusabi_adapter.rs` - Fusabi VM integration
- `examples/fusabi-config/` - Example Fusabi configurations
- `ROADMAP.md` - Overall project roadmap

**External Dependencies:**
- https://github.com/fusabi-lang/fusabi - Fusabi language and runtime
- File feature requests here if Fusabi lacks needed capabilities
