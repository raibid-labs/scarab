# Fusabi Feature Requests for Scarab Configuration

This document outlines the Fusabi language features needed to fully support Scarab's configuration system.

## Current Status

**What Works:**
- ✅ Basic let bindings with integers
- ✅ Module declarations
- ✅ Comments
- ✅ Attributes (e.g., `[<Plugin>]`)
- ✅ String literals
- ✅ Lists
- ✅ Functions with async support
- ✅ Pattern matching
- ✅ Sets (Set.ofList)

**What's Missing:**
- ❌ Floating-point literals (e.g., `14.0`, `1.2`)
- ❌ Record types and record expressions
- ❌ Module imports (`open` statements)
- ❌ Option types (Some/None)
- ❌ Map.ofList for dictionaries
- ❌ Anonymous records (`{| ... |}`)
- ❌ Enum types (e.g., `TabPosition.Top`)
- ❌ System library access (System.DateTime)
- ❌ Mutable bindings (`<-` operator)
- ❌ Error/Ok result types

## Required Features for Configuration

### 1. Floating-Point Numbers

**Priority:** HIGH
**Current Error:** `CompileError(UnsupportedFloat)`

**Use Case:**
```fsharp
let font = {
    Size = 14.0        // Font size in points
    LineHeight = 1.2   // Line height multiplier
}

let colors = {
    Opacity = 1.0      // Window transparency
    DimOpacity = 0.7   // Dimmed window transparency
}
```

**Why Needed:** Font sizes, opacity values, and scaling factors are inherently floating-point.

---

### 2. Record Types

**Priority:** HIGH
**Current Error:** `ParseError(UnexpectedToken { expected: "=", found: Open })`

**Use Case:**
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

**Why Needed:** Configuration is naturally hierarchical. Records provide type-safe structured data.

---

### 3. Module Imports (`open`)

**Priority:** MEDIUM
**Current Error:** `ParseError(UnexpectedToken)`

**Use Case:**
```fsharp
module ScarabConfig

open Scarab.Config
open System.Text.RegularExpressions

let urlPattern = @"https?://.*"
let urlRegex = Regex(urlPattern)
```

**Why Needed:** Access to types, functions, and constants from standard libraries and Scarab API.

---

### 4. Option Types

**Priority:** MEDIUM

**Use Case:**
```fsharp
let colors = {
    Theme = Some "dracula"         // Optional theme
    Foreground = None              // Use default
    Background = Some "#282a36"    // Override
}

let sessions = {
    WorkingDirectory = None  // Use current directory
}
```

**Why Needed:** Many config fields are optional. Option types provide type-safe optionality.

---

### 5. Map Types

**Priority:** MEDIUM

**Use Case:**
```fsharp
let keybindings = {
    Custom = Map.ofList [
        ("OpenConfig", "Ctrl+,")
        ("ReloadConfig", "Ctrl+Shift+R")
        ("ToggleFullscreen", "F11")
    ]
}

let plugins = {
    Config = Map.ofList [
        ("url-detector", {| IgnoreLocalhost = true |})
        ("git-status", {| ShowBranch = true |})
    ]
}
```

**Why Needed:** Key-value configuration (custom keybindings, per-plugin settings).

---

### 6. Anonymous Records

**Priority:** LOW
**Workaround:** Use named records for now

**Use Case:**
```fsharp
let pluginConfig = Map.ofList [
    ("url-detector", {|
        IgnoreLocalhost = true
        AutoOpen = false
        NotifyOnDetect = true
    |})
]
```

**Why Needed:** Convenient for inline configuration objects without defining types.

---

### 7. Enum Types

**Priority:** MEDIUM

**Use Case:**
```fsharp
type TabPosition = Top | Bottom | Left | Right
type CursorStyle = Block | Beam | Underline

let ui = {
    TabPosition = TabPosition.Top
    CursorStyle = CursorStyle.Block
}
```

**Why Needed:** Type-safe enumerated configuration options.

---

### 8. System Library Access

**Priority:** LOW
**Workaround:** Implement minimal subset in Fusabi stdlib

**Use Case:**
```fsharp
// Dynamic configuration based on time
let dynamicTheme () =
    let hour = System.DateTime.Now.Hour
    if hour >= 6 && hour < 18 then
        "solarized-light"
    else
        "dracula"
```

**Why Needed:** Dynamic configuration based on environment (time, screen resolution, etc.).

---

### 9. Mutable Bindings

**Priority:** LOW
**Workaround:** Use immutable updates for now

**Use Case:**
```fsharp
let mutable font = defaultFont
font.Size <- dynamicFontSize()  // Update based on screen resolution
```

**Why Needed:** Apply dynamic configuration updates.

---

### 10. Result Types (Error/Ok)

**Priority:** LOW
**Workaround:** Use simple success/failure pattern

**Use Case:**
```fsharp
[<OnConfigValidate>]
let validate () =
    if font.Size < 6.0 || font.Size > 72.0 then
        Error "Font size must be between 6.0 and 72.0"
    else
        Ok ()
```

**Why Needed:** Type-safe error handling in config validation hooks.

---

## Prioritized Implementation Order

1. **Floating-Point Numbers** - Blocks config entirely
2. **Record Types** - Essential for structured config
3. **Module Imports** - Needed for API access
4. **Option Types** - Common in config
5. **Enum Types** - Type-safe choices
6. **Map Types** - Key-value configs
7. **Anonymous Records** - Nice-to-have convenience
8. **System Library** - Advanced dynamic configs
9. **Mutable Bindings** - Advanced scenarios
10. **Result Types** - Better error handling

---

## Workarounds (Current Approach)

Until these features are implemented, Scarab uses:

1. **Simplified .fsx configs** that compile successfully:
   ```fsharp
   let scrollback_lines = 10000
   let terminal_columns = 80
   let terminal_rows = 24
   ```

2. **Default fallback**: Compiled config validates syntax, but actual values use Rust defaults

3. **TOML backwards compatibility**: Users can still use `config.toml` with full feature support

4. **Clear messaging**: "⚠️ Fusabi config loader is WIP - using defaults"

---

## Long-Term Vision

Once these features are available, Scarab configuration will be:

- **Type-safe** - Record types and enums prevent invalid configs
- **Programmable** - Functions for dynamic configuration
- **Validated** - Compile-time + runtime validation
- **Composable** - Import and reuse config modules
- **Hot-reloadable** - Edit .fsx and reload instantly

Example of the future:

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
let font = dynamicFont()  // Function returns config based on screen DPI

[<ColorConfig>]
let colors = themeFor(System.DateTime.Now)  // Time-based theme switching

[<Validate>]
let validate cfg =
    if cfg.Font.Size < 6.0 then Error "Font too small"
    else Ok cfg
```

This makes configuration both powerful and safe.
