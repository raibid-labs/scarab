# Fusabi Language Reference

Fusabi is a lightweight, F#-inspired scripting language for hot-reloadable UI customization in the Scarab terminal emulator.

## Table of Contents

- [Overview](#overview)
- [Data Types](#data-types)
- [Syntax](#syntax)
- [Standard Library](#standard-library)
- [Bevy Integration](#bevy-integration)
- [Examples](#examples)

## Overview

Fusabi is designed for:
- **Hot-reloading**: Changes take effect in <100ms without recompilation
- **UI Scripting**: Create overlays, menus, and custom layouts
- **Keybindings**: Define custom key mappings
- **Themes**: Customize colors and styles
- **Performance**: Parse 1,000 LOC in <10ms

## Data Types

### Primitives

```fsharp
// Integers
let x = 42
let negative = -10

// Floats
let pi = 3.14159
let e = 2.71828

// Booleans
let is_true = true
let is_false = false

// Strings
let greeting = "Hello, world!"

// Nil (null/unit)
let nothing = nil
```

### Collections

```fsharp
// Lists
let numbers = [1, 2, 3, 4, 5]
let mixed = [1, "two", 3.0, true]

// Maps (records)
let person = {
    name = "Alice";
    age = 30;
    email = "alice@example.com"
}
```

## Syntax

### Variables and Bindings

```fsharp
// Top-level binding
let x = 42

// Local binding (let-in expression)
let result = let x = 10 in x + 20  // result = 30

// Multiple bindings
let x = 10
let y = 20
let sum = x + y
```

### Functions

```fsharp
// Named function
let add x y = x + y
let result = add 10 20  // 30

// Lambda (anonymous function)
let inc = fun x -> x + 1
let forty_two = inc 41  // 42

// Higher-order functions
let apply_twice f x = f (f x)
let result = apply_twice inc 40  // 42

// Recursion
let factorial n =
    if n <= 1 then
        1
    else
        n * factorial (n - 1)
```

### Conditionals

```fsharp
// If-then-else
let max x y =
    if x > y then x else y

// Nested conditionals
let sign x =
    if x > 0 then
        "positive"
    else if x < 0 then
        "negative"
    else
        "zero"
```

### Operators

```fsharp
// Arithmetic
let sum = 2 + 3        // 5
let diff = 10 - 4      // 6
let product = 3 * 4    // 12
let quotient = 15 / 3  // 5
let remainder = 17 % 5 // 2

// Comparison
let gt = 5 > 3         // true
let lt = 5 < 3         // false
let eq = 5 == 5        // true
let ne = 5 != 3        // true
let ge = 5 >= 5        // true
let le = 5 <= 5        // true

// Logical
let and_op = true && false   // false
let or_op = true || false    // true
let not_op = !true           // false

// String concatenation
let greeting = "Hello, " + "world!"
```

### List and Map Access

```fsharp
// List indexing
let first = [1, 2, 3][0]     // 1
let third = [1, 2, 3][2]     // 3

// Map field access
let name = { name = "Alice"; age = 30 }.name  // "Alice"

// Nested access
let config = {
    ui = {
        theme = "dark";
        font_size = 14
    }
}
let theme = config.ui.theme  // "dark"
```

### Comments

```fsharp
// Single-line comment

let x = 42  // End-of-line comment

// Multi-line comments not yet supported
```

## Standard Library

### String Functions

```fsharp
strlen "hello"              // 5
substr "hello" 0 3          // "hel"
concat "hello" " world"     // "hello world"
to_upper "hello"            // "HELLO"
to_lower "WORLD"            // "world"
trim "  spaces  "           // "spaces"
split "a,b,c" ","          // ["a", "b", "c"]
join ["a", "b", "c"] ","   // "a,b,c"
```

### List Functions

```fsharp
length [1, 2, 3]            // 3
head [1, 2, 3]              // 1
tail [1, 2, 3]              // [2, 3]
cons 0 [1, 2, 3]            // [0, 1, 2, 3]
append [1, 2] [3, 4]        // [1, 2, 3, 4]
reverse [1, 2, 3]           // [3, 2, 1]
nth [1, 2, 3] 1             // 2
take 2 [1, 2, 3, 4]         // [1, 2]
drop 2 [1, 2, 3, 4]         // [3, 4]
```

### Map Functions

```fsharp
let m = { x = 1; y = 2 }
keys m                      // ["x", "y"]
values m                    // [1, 2]
has_key m "x"               // true
```

### Math Functions

```fsharp
abs -42                     // 42
min 5 10                    // 5
max 5 10                    // 10
pow 2.0 8.0                 // 256.0
sqrt 16.0                   // 4.0
floor 3.7                   // 3
ceil 3.2                    // 4
round 3.5                   // 4
```

### Conversion Functions

```fsharp
to_int 3.14                 // 3
to_int "42"                 // 42
to_float 42                 // 42.0
to_float "3.14"             // 3.14
to_string 42                // "42"
```

### Type Checking

```fsharp
is_nil nil                  // true
is_bool true                // true
is_int 42                   // true
is_float 3.14               // true
is_string "hello"           // true
is_list [1, 2, 3]           // true
is_map { x = 1 }            // true
is_function (fun x -> x)    // true
```

### IO Functions

```fsharp
print "Hello"               // Prints without newline
println "World"             // Prints with newline
```

## Bevy Integration

When the `bevy-integration` feature is enabled, additional UI functions are available:

```fsharp
// Spawn UI elements
let button = ui_spawn_button "Click Me" (fun () -> println "Clicked!")
let text = ui_spawn_text "Hello, UI!"
let container = ui_spawn_container

// Set properties
ui_set_position button 100.0 200.0 0.0
ui_set_size button 150.0 50.0
ui_set_color button 1.0 0.0 0.0 1.0  // RGBA
```

Note: These functions require access to the Bevy World and are typically called from within Bevy systems.

## Examples

### Factorial

```fsharp
let factorial n =
    if n <= 1 then
        1
    else
        n * factorial (n - 1)

factorial 5  // 120
```

### Fibonacci

```fsharp
let fib n =
    if n <= 1 then
        n
    else
        fib (n - 1) + fib (n - 2)

fib 10  // 55
```

### List Processing

```fsharp
let sum_list lst =
    if length lst == 0 then
        0
    else
        head lst + sum_list (tail lst)

sum_list [1, 2, 3, 4, 5]  // 15
```

### Command Palette

```fsharp
let commands = [
    { name = "New Tab"; key = "t"; action = "new_tab" };
    { name = "Close Tab"; key = "q"; action = "close_tab" };
    { name = "Split Vertical"; key = "v"; action = "split_vertical" }
]

let find_command key cmds =
    if length cmds == 0 then
        nil
    else
        let cmd = head cmds
        if cmd.key == key then
            cmd
        else
            find_command key (tail cmds)

find_command "v" commands  // { name = "Split Vertical", ... }
```

### Theme Configuration

```fsharp
let theme = {
    name = "Monokai";
    colors = {
        background = "#272822";
        foreground = "#f8f8f2";
        black = "#272822";
        red = "#f92672";
        green = "#a6e22e";
        yellow = "#f4bf75";
        blue = "#66d9ef";
        magenta = "#ae81ff";
        cyan = "#a1efe4";
        white = "#f8f8f2"
    };
    font = {
        family = "JetBrains Mono";
        size = 14
    }
}
```

## Performance

Fusabi is optimized for fast parsing and hot-reloading:

- **Parse Speed**: 1,000 LOC in <10ms
- **Hot-Reload**: <100ms from file change to execution
- **Caching**: AST is cached and only reparsed when files change
- **File Watching**: Uses efficient notify-based file watcher

## Error Handling

Fusabi provides helpful error messages with line and column numbers:

```
Parse error at line 5, column 12: expected expression
Type error: expected int, got string
Runtime error: division by zero
Undefined variable: foo
Index out of bounds: 5 (length: 3)
```

## Future Enhancements

- Pattern matching
- Type inference
- Module system
- Async/await support
- More standard library functions
- Optimizations (tail-call optimization, JIT compilation)

---

For more examples, see the `examples/fusabi/` directory.
