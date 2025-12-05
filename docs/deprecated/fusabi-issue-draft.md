# Feature Request: Configuration DSL Support for Scarab Terminal

## Context

**Project**: [Scarab Terminal](https://github.com/raibid-labs/scarab) - A high-performance, split-process terminal emulator built in Rust
**Current Fusabi Version**: 0.5.0 (upgrading to 0.8.0)
**Use Case**: Replacing TOML-based configuration with Fusabi-based configuration DSL

We've successfully integrated Fusabi as the core scripting and configuration language for Scarab Terminal. However, several F# language features are needed to fully support our configuration use case.

## Current Status

**What Works:**
- ✅ Basic let bindings with integers and strings
- ✅ Module declarations and comments
- ✅ Attributes (e.g., `[<Plugin>]`)
- ✅ Lists and functions with async support
- ✅ Pattern matching and Sets

**What's Missing:**
- ❌ Floating-point literals → `CompileError(UnsupportedFloat)`
- ❌ Record types and expressions → `ParseError(UnexpectedToken)`
- ❌ Module imports (`open` statements)
- ❌ Option types (Some/None)
- ❌ Map types for dictionaries
- ❌ Anonymous records (`{| ... |}`)
- ❌ Enum types

## Requested Features (Priority Order)

### 1. Floating-Point Numbers (CRITICAL)

**Current Error**: `CompileError(UnsupportedFloat)`

**Use Case**:
```fsharp
let font_size = 14.0           // Font size in points
let line_height = 1.2          // Line height multiplier
let opacity = 0.85             // Window transparency
```

**Why Critical**: Font sizes, opacity values, and scaling factors are inherently floating-point. Without this, we cannot configure visual properties.

---

### 2. Record Types (CRITICAL)

**Current Error**: `ParseError(UnexpectedToken { expected: "=", found: Open })`

**Use Case**:
```fsharp
type FontConfig = {
    Family: string
    Size: float
    LineHeight: float
    Fallback: string list
}

let font = {
    Family = "JetBrains Mono"
    Size = 14.0
    LineHeight = 1.2
    Fallback = ["Fira Code"; "Menlo"]
}
```

**Why Critical**: Configuration is naturally hierarchical. Records provide type-safe structured data that maps directly to Rust structs.

---

### 3. Module Imports (`open`) (HIGH)

**Use Case**:
```fsharp
module ScarabConfig

open Scarab.Config
open System.Text.RegularExpressions

let theme = Themes.Dracula
```

**Why Needed**: Access to types, functions, and constants from standard libraries and Scarab API.

---

### 4. Option Types (HIGH)

**Use Case**:
```fsharp
let theme = Some "dracula"        // Optional theme
let foreground = None             // Use default
let working_dir = None            // Use current directory
```

**Why Needed**: Many config fields are optional. Option types provide type-safe optionality that maps to Rust's `Option<T>`.

---

### 5. Map Types (MEDIUM)

**Use Case**:
```fsharp
let custom_keybindings = Map.ofList [
    ("OpenConfig", "Ctrl+,")
    ("ReloadConfig", "Ctrl+Shift+R")
    ("ToggleFullscreen", "F11")
]
```

**Why Needed**: Key-value configuration for custom keybindings and per-plugin settings.

---

### 6. Enum Types (MEDIUM)

**Use Case**:
```fsharp
type TabPosition = Top | Bottom | Left | Right
type CursorStyle = Block | Beam | Underline

let tab_position = TabPosition.Top
let cursor_style = CursorStyle.Block
```

**Why Needed**: Type-safe enumerated configuration options that map to Rust enums.

---

### 7. Anonymous Records (LOW)

**Use Case**:
```fsharp
let plugin_config = Map.ofList [
    ("url-detector", {| IgnoreLocalhost = true; AutoOpen = false |})
]
```

**Why Needed**: Convenient for inline configuration without defining types.

---

## Current Workaround

Until these features are implemented, we use:

1. **Simplified configs** that compile:
   ```fsharp
   let scrollback_lines = 10000
   let terminal_columns = 80
   ```

2. **Default fallback**: Config validates syntax, actual values use Rust defaults

3. **TOML backwards compatibility**: Full feature support via legacy `config.toml`

4. **Clear messaging**: "⚠️ Fusabi config loader is WIP - using defaults"

## Long-Term Vision

Once these features are available, Scarab will have:

- **Type-safe configuration** - Records and enums prevent invalid configs
- **Programmable configs** - Functions for dynamic configuration (time-based themes, DPI-aware fonts)
- **Compile-time validation** - Catch config errors before runtime
- **Hot-reload support** - Edit .fsx and reload instantly
- **Composable modules** - Import and reuse config modules

**Example of the future**:
```fsharp
module ScarabConfig

open Scarab.Config

[<TerminalConfig>]
let terminal = {
    DefaultShell = env "SHELL" |> Option.defaultValue "/bin/zsh"
    ScrollbackLines = 10_000
    Columns = 80
    Rows = 24
}

[<FontConfig>]
let font = {
    Family = "JetBrains Mono"
    Size = 14.0
    LineHeight = 1.2
    Fallback = ["Fira Code"; "DejaVu Sans Mono"]
}

[<ColorConfig>]
let colors = {
    Theme = Some "dracula"
    Opacity = 1.0
    DimOpacity = 0.7
    Palette = defaultDraculaPalette
}

[<Validate>]
let validate cfg =
    if cfg.Font.Size < 6.0 then Error "Font too small"
    else Ok cfg

export config  // Export to Rust
```

## Impact

This would make Fusabi's configuration capabilities on par with:
- **Lua** (neovim configs)
- **Dhall** (kubernetes configs)
- **Nix** (system configs)

But with **static type safety** and **F# expressiveness**.

## Questions

1. Are these features planned for future Fusabi versions?
2. What's the timeline for float and record support?
3. Would you accept PRs to help implement these features?
4. Are there design constraints we should be aware of?

## Resources

- **Scarab Terminal**: https://github.com/raibid-labs/scarab
- **Feature Request Doc**: https://github.com/raibid-labs/scarab/blob/main/docs/fusabi-feature-request.md
- **Example Plugin** (working today): https://github.com/raibid-labs/scarab/blob/main/plugins/examples/url-detector/url-detector.fsx

---

Thank you for building Fusabi! It's already powering Scarab's plugin system beautifully. These features would unlock the full potential of Fusabi as a configuration language.
