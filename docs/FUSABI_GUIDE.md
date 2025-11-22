# Fusabi Quick Start Guide

Get started with Fusabi scripting in Scarab terminal.

## Installation

Fusabi is built into Scarab. No additional installation required!

## Your First Script

Create a file `~/.config/scarab/scripts/hello.fsx`:

```fsharp
let greeting = "Hello, Fusabi!"
println greeting
```

Save the file and Fusabi will automatically detect and run it!

## Hot Reloading

Edit your script and save it. Changes take effect immediately (typically <100ms) without restarting Scarab.

```fsharp
// Edit this file and save - changes apply instantly!
let message = "Hot reload works!"
println message
```

## Common Use Cases

### 1. Custom Keybindings

Create `~/.config/scarab/scripts/keybindings.fsx`:

```fsharp
let leader = "<Space>"

// Window management
let window_bindings = [
    { keys = [leader; "w"; "v"]; command = "split_vertical" };
    { keys = [leader; "w"; "h"]; command = "split_horizontal" };
    { keys = [leader; "w"; "q"]; command = "close_window" }
]

// Tab management
let tab_bindings = [
    { keys = [leader; "t"; "n"]; command = "new_tab" };
    { keys = [leader; "t"; "q"]; command = "close_tab" }
]
```

### 2. Custom Theme

Create `~/.config/scarab/scripts/theme.fsx`:

```fsharp
let theme = {
    name = "My Theme";
    colors = {
        background = "#1e1e2e";
        foreground = "#cdd6f4";
        cursor = "#89b4fa"
    };
    font = {
        family = "JetBrains Mono";
        size = 14
    }
}
```

### 3. Command Palette

Create `~/.config/scarab/scripts/commands.fsx`:

```fsharp
let commands = [
    { name = "New Tab"; key = "t" };
    { name = "Split Vertical"; key = "v" };
    { name = "Search"; key = "/" };
    { name = "Settings"; key = "," }
]

// Show command palette on <Space>
println "Command palette loaded"
```

### 4. Status Line

Create `~/.config/scarab/scripts/statusline.fsx`:

```fsharp
let format_status mode branch =
    let mode_indicator =
        if mode == "normal" then
            "NORMAL"
        else if mode == "insert" then
            "INSERT"
        else
            "VISUAL"
    in
    concat mode_indicator (concat " | " branch)

let status = format_status "normal" "main"
println status
```

## Script Structure

A typical Fusabi script follows this pattern:

```fsharp
// 1. Define data
let config = {
    width = 600;
    height = 400
}

// 2. Define functions
let process_data x =
    x * 2

// 3. Define actions
let commands = [
    { name = "Action 1"; handler = fun () -> println "Action 1" }
]

// 4. Initialize
println "Script loaded!"
```

## Tips and Best Practices

### 1. Keep Scripts Small

Break large configurations into multiple files:

```
~/.config/scarab/scripts/
  ├── theme.fsx
  ├── keybindings.fsx
  ├── commands.fsx
  └── statusline.fsx
```

### 2. Use Comments

Document your scripts:

```fsharp
// Theme configuration for Scarab
// Last updated: 2024-01-20

let theme = {
    // Catppuccin Mocha palette
    background = "#1e1e2e";  // Base
    foreground = "#cdd6f4"   // Text
}
```

### 3. Test Incrementally

Start simple and add complexity:

```fsharp
// Start with this
let greeting = "Hello"
println greeting

// Then add more
let format_greeting name =
    concat "Hello, " (concat name "!")

println (format_greeting "World")
```

### 4. Use the Standard Library

Leverage built-in functions:

```fsharp
// String manipulation
let uppercase_name = to_upper "alice"

// List processing
let numbers = [1, 2, 3, 4, 5]
let doubled = reverse (take 3 numbers)

// Math operations
let distance = sqrt (pow 3.0 2.0 + pow 4.0 2.0)
```

### 5. Handle Errors

Use conditionals to validate data:

```fsharp
let safe_divide x y =
    if y == 0 then
        0
    else
        x / y

let result = safe_divide 10 0  // Returns 0 instead of error
```

## Debugging Scripts

### Print Debugging

```fsharp
let debug_value name value =
    println (concat name (concat ": " (to_string value)))

let x = 42
debug_value "x" x  // Prints: x: 42
```

### Type Checking

```fsharp
let validate_config cfg =
    if is_map cfg then
        println "Config is valid"
    else
        println "Config is invalid!"

validate_config { x = 1 }
```

### Error Messages

Fusabi provides helpful error messages:

```
Parse error at line 5, column 12: expected expression
Type error: expected int, got string
Undefined variable: foo
```

## Performance Considerations

### 1. Avoid Deep Recursion

Fusabi doesn't have tail-call optimization yet:

```fsharp
// This might overflow with large n
let factorial n =
    if n <= 1 then 1 else n * factorial (n - 1)

// Better: use iteration when possible
let factorial_iter n =
    let rec helper acc i =
        if i > n then acc else helper (acc * i) (i + 1)
    helper 1 1
```

### 2. Cache Expensive Computations

```fsharp
// Computed once, reused many times
let expensive_config = {
    processed_data = process_large_dataset data;
    computed_value = expensive_computation input
}
```

### 3. Keep Hot Path Fast

Code that runs frequently should be simple:

```fsharp
// Fast path for common case
let quick_check x =
    if x > 0 then true else false

// Complex logic for rare cases
let detailed_analysis x =
    // ...
```

## Next Steps

1. Read the [Language Reference](FUSABI_LANGUAGE.md)
2. Explore [examples/fusabi/](../examples/fusabi/)
3. Join the community and share your scripts!

## Resources

- **Language Reference**: Complete syntax and standard library
- **Examples**: Real-world scripts for inspiration
- **Issue Tracker**: Report bugs or request features

Happy scripting!
