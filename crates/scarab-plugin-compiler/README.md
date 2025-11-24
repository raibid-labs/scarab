# Scarab Plugin Compiler

A command-line tool for compiling Fusabi (F#-like) plugin source files (.fsx) to bytecode (.fzb) for the Scarab terminal emulator.

## Overview

This compiler integrates the official [Fusabi](https://github.com/fusabi-lang/fusabi) language frontend to provide:

- **Lexical analysis** - Tokenization of Fusabi source code
- **Parsing** - AST generation with error reporting
- **Type checking** - Optional Hindley-Milner type inference
- **Bytecode generation** - Compilation to Fusabi VM bytecode
- **Metadata validation** - Plugin metadata extraction and validation

## Installation

Build from the Scarab workspace:

```bash
cargo build -p scarab-plugin-compiler
```

The compiled binary will be available at:
- Debug: `~/.cargo/target/debug/scarab-plugin-compiler`
- Release: `~/.cargo/target/release/scarab-plugin-compiler`

## Usage

### Basic Compilation

```bash
scarab-plugin-compiler examples/fusabi/hello.fsx
```

This will produce `examples/fusabi/hello.fzb` bytecode file.

### Options

```
USAGE:
    scarab-plugin-compiler [OPTIONS] <INPUT>

OPTIONS:
    -o, --output <OUTPUT>       Output .fzb file path (default: same as input)
    -v, --verbose               Enable verbose compilation output
    --validate-metadata         Validate plugin metadata (@name, @version, @description)
    --skip-type-check           Skip type inference (faster compilation)
    --print-ast                 Print abstract syntax tree for debugging
    --disassemble               Print bytecode disassembly
    -h, --help                  Print help information
```

### Examples

**Compile with verbose output:**
```bash
scarab-plugin-compiler -v examples/fusabi/hello.fsx
```

**Compile to custom location:**
```bash
scarab-plugin-compiler -o target/plugins/hello.fzb examples/fusabi/hello.fsx
```

**Debug compilation with AST and disassembly:**
```bash
scarab-plugin-compiler --print-ast --disassemble examples/fusabi/hello.fsx
```

**Fast compilation without type checking:**
```bash
scarab-plugin-compiler --skip-type-check examples/fusabi/hello.fsx
```

## Plugin Metadata

Plugins should include metadata comments at the top of the source file:

```fsharp
// @name my-plugin
// @version 0.1.0
// @description A description of what this plugin does
// @author Your Name
// @api-version 0.1.0
// @min-scarab-version 0.1.0
```

Required metadata:
- `@name` - Plugin identifier
- `@version` - Semantic version
- `@description` - Brief description

Optional metadata:
- `@author` - Plugin author
- `@api-version` - Scarab plugin API version
- `@min-scarab-version` - Minimum Scarab version required

## Fusabi Language Support

The compiler uses **fusabi-frontend v0.5.0** which supports a subset of F# syntax:

### Supported Features

- Let bindings: `let x = 42 in expr`
- Lambda functions: `fun x -> x + 1`
- Function application: `add 10 20`
- Binary operations: `+`, `-`, `*`, `/`, `==`, `!=`, `<`, `>`, `<=`, `>=`
- Literals: integers, floats, strings, booleans
- Conditional expressions: `if condition then expr1 else expr2`
- Pattern matching: `match expr with | pattern -> expr`
- Recursive functions: `let rec factorial n = ...`

### Current Limitations

The Fusabi frontend v0.5.0 has limited support for:
- Type declarations (use type inference instead)
- Module definitions
- Record types
- Discriminated unions
- Advanced pattern matching

### Example Plugin

```fsharp
// @name hello
// @version 0.1.0
// @description Simple hello world example

let add x y = x + y in
let result = add 10 20 in
result
```

## Bytecode File Format

Compiled `.fzb` files contain:

1. **Header** (length-prefixed):
   - Magic bytes: `FZB\x01`
   - Format version: `1`
   - Plugin metadata (serialized)

2. **Bytecode chunk** (bincode-serialized):
   - Instructions array
   - Constants pool
   - Chunk name (optional)

## Integration with Build Script

Use the provided build script for batch compilation:

```bash
# Compile single plugin
./scripts/build-plugin.sh examples/fusabi/hello.fsx

# Compile all plugins in a directory
./scripts/build-plugin.sh --all

# Verbose compilation with validation
./scripts/build-plugin.sh -v --validate-metadata examples/fusabi/hello.fsx
```

## Error Handling

The compiler provides detailed error messages for:

- **Lexer errors** - Invalid tokens or characters
- **Parser errors** - Syntax errors with position information
- **Type errors** - Type mismatches (when type checking is enabled)
- **Metadata errors** - Missing or invalid plugin metadata
- **IO errors** - File read/write failures

Example error output:
```
Error: Parser error: UnexpectedToken {
    expected: "in",
    found: Ident("println"),
    pos: Position { line: 11, column: 1, offset: 233 }
}
```

## Development

### Building

```bash
cargo build -p scarab-plugin-compiler
```

### Testing

```bash
# Run unit tests
cargo test -p scarab-plugin-compiler

# Test compilation on examples
./scripts/build-plugin.sh --all
```

### Dependencies

- **fusabi-frontend** v0.5.0 - Parser, type checker, compiler
- **fusabi-vm** v0.5.0 - Bytecode definitions and constants
- **clap** v4.5 - Command-line argument parsing
- **colored** v2.1 - Terminal output coloring
- **anyhow** v1.0 - Error handling
- **bincode** v1.3 - Bytecode serialization
- **serde** v1.0 - Serialization framework

## Troubleshooting

### Compiler binary not found

If the build script reports "Fusabi compiler not found", build it explicitly:

```bash
cargo build -p scarab-plugin-compiler
```

### Parser errors

Ensure your Fusabi code uses proper F# expression syntax:
- All let bindings must end with `in`
- The final expression should not have `in`
- Use `fun x -> expr` for lambda functions

### Type errors

If type checking fails, you can:
1. Fix the type error in your code
2. Use `--skip-type-check` to bypass type inference
3. Add explicit type annotations (if supported)

## See Also

- [Fusabi Language Documentation](https://github.com/fusabi-lang/fusabi)
- [Scarab Plugin API Documentation](../scarab-plugin-api/README.md)
- [Plugin Build Script](../../scripts/build-plugin.sh)
