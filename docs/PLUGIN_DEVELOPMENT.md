# Fusabi Plugin Development Guide

> **DEPRECATED**: This document is outdated. See [Plugin Development Guide](./plugin-development/README.md) for current information.

Comprehensive guide for developing Scarab plugins using the Fusabi language.

## Table of Contents

- [Quick Start](#quick-start)
- [Plugin Structure](#plugin-structure)
- [Development Workflow](#development-workflow)
- [Build System](#build-system)
- [Testing Plugins](#testing-plugins)
- [API Reference](#api-reference)
- [Best Practices](#best-practices)
- [Troubleshooting](#troubleshooting)

## Quick Start

### 1. Create New Plugin

```bash
just plugin-new my-awesome-plugin
```

This creates `examples/fusabi/my-awesome-plugin.fsx` from the template.

### 2. Edit Plugin

Open the generated file and customize:

```fsharp
// @name my-awesome-plugin
// @version 0.1.0
// @description Does something awesome
// @author Your Name
// @api-version 0.1.0

let on_load (ctx: PluginContext) =
    printfn "Plugin loaded!"
    Ok ()
```

### 3. Build and Validate

```bash
# Build plugin
just plugin-build examples/fusabi/my-awesome-plugin.fsx

# Validate plugin
just plugin-validate examples/fusabi/my-awesome-plugin.fsx
```

### 4. Test in Scarab

```bash
# Terminal 1: Start daemon with plugin
cargo run -p scarab-daemon

# Terminal 2: Start client
cargo run -p scarab-client
```

## Plugin Structure

### Required Metadata

Every plugin must declare metadata at the top of the file:

```fsharp
// @name plugin-name              // Unique identifier (required)
// @version 1.0.0                 // Semver version (required)
// @description Short description // What it does (recommended)
// @author Your Name              // Plugin author (recommended)
// @homepage https://...          // Project URL (optional)
// @license MIT                   // License (optional)
// @api-version 0.1.0            // Plugin API version (required)
// @min-scarab-version 0.1.0     // Minimum Scarab version (recommended)
```

### Plugin Lifecycle

```fsharp
/// Called when plugin loads
let on_load (ctx: PluginContext) =
    // Initialize state, register commands
    Ok ()

/// Called when plugin unloads
let on_unload () =
    // Clean up resources
    Ok ()
```

### Event Hooks

```fsharp
/// Intercept terminal output
let on_output (line: string) (ctx: PluginContext) =
    // Process output
    Action.Continue

/// Intercept user input
let on_input (input: byte[]) (ctx: PluginContext) =
    // Process input
    Action.Continue

/// Before command execution
let on_pre_command (command: string) (ctx: PluginContext) =
    Action.Continue

/// After command completion
let on_post_command (command: string) (exit_code: int) (ctx: PluginContext) =
    Ok ()

/// Terminal resize
let on_resize (cols: uint16) (rows: uint16) (ctx: PluginContext) =
    Ok ()

/// Client attach
let on_attach (client_id: uint64) (ctx: PluginContext) =
    Ok ()

/// Client detach
let on_detach (client_id: uint64) (ctx: PluginContext) =
    Ok ()

/// Remote command from client
let on_remote_command (id: string) (ctx: PluginContext) =
    Ok ()
```

## Development Workflow

### Option 1: Watch Mode (Recommended)

Automatically rebuild on changes:

```bash
just plugin-watch
```

This watches `examples/fusabi/` and rebuilds all plugins when files change.

### Option 2: Manual Build

```bash
# Build single plugin
just plugin-build examples/fusabi/my-plugin.fsx

# Build all plugins
just plugin-build-all

# With output directory
./scripts/build-plugin.sh -o target/plugins examples/fusabi/my-plugin.fsx
```

### Option 3: Direct Script

```bash
# Build with verbose output
./scripts/build-plugin.sh -v examples/fusabi/my-plugin.fsx

# Validate only (no build)
./scripts/build-plugin.sh -V examples/fusabi/my-plugin.fsx

# Skip metadata validation
./scripts/build-plugin.sh -s examples/fusabi/my-plugin.fsx
```

## Build System

### Scripts

**`scripts/build-plugin.sh`** - Compiles .fsx to .fzb

```bash
# Options
-h, --help              Show help
-v, --verbose           Verbose output
-o, --output DIR        Output directory
-V, --validate-only     Validate without building
-s, --skip-metadata     Skip metadata checks
-a, --all               Build all plugins
```

**`scripts/plugin-validator.sh`** - Validates plugin structure

```bash
# Options
-h, --help              Show help
-v, --verbose           Verbose output
-s, --strict            Warnings as errors
-a, --all               Validate all plugins
-j, --json              JSON output
--api-version VERSION   Override API version
```

### Just Commands

```bash
just plugin-build FILE          # Build single plugin
just plugin-build-all           # Build all plugins
just plugin-validate FILE       # Validate single plugin
just plugin-validate-all        # Validate all plugins
just plugin-watch               # Watch and rebuild
just plugin-test                # Test plugin loading
just plugin-ci                  # Run all CI checks
just plugin-new NAME            # Create from template
just plugin-status              # Show status
just plugin-clean               # Clean build artifacts
```

## Testing Plugins

### Unit Tests

Test plugin loading logic:

```bash
cargo test -p scarab-daemon plugin -- --nocapture
```

### Integration Tests

Test with running daemon:

```bash
# Terminal 1: Start daemon
cargo run -p scarab-daemon

# Terminal 2: Start client
cargo run -p scarab-client

# Terminal 3: Send test commands
echo "test command" | nc -U /tmp/scarab.sock
```

### Validation Tests

```bash
# Validate plugin structure
just plugin-validate examples/fusabi/my-plugin.fsx

# Strict validation (warnings as errors)
./scripts/plugin-validator.sh --strict examples/fusabi/my-plugin.fsx

# Validate all
just plugin-validate-all
```

### Manual Testing

1. Enable debug mode in plugin:
   ```fsharp
   let config = { debug_mode = true }
   ```

2. Check daemon logs:
   ```bash
   tail -f /tmp/scarab-daemon.log
   ```

3. Test specific hooks by triggering events in the terminal

## API Reference

### Types

```fsharp
/// Plugin context with access to Scarab internals
type PluginContext = {
    notify: string -> unit
    log: LogLevel -> string -> unit
    register_command: string -> (unit -> unit) -> unit
    get_terminal_size: unit -> (uint16 * uint16)
    write_to_pty: byte[] -> unit
}

/// Action to control data flow in hooks
type Action =
    | Continue          // Pass through unchanged
    | Block            // Suppress/block
    | Modified of 'T   // Pass modified version

/// Log levels
type LogLevel =
    | Debug
    | Info
    | Warn
    | Error

/// Plugin metadata
type PluginMetadata = {
    name: string
    version: string
    description: string
    author: string
    homepage: string option
    api_version: string
    min_scarab_version: string
}
```

### Context Methods

```fsharp
// Show notification to user
ctx.notify "Hello from plugin!"

// Log message
ctx.log Info "Plugin initialized"

// Register command in palette
ctx.register_command "my-command" (fun () ->
    printfn "Command executed!"
)

// Get terminal size
let (cols, rows) = ctx.get_terminal_size ()

// Write to PTY
ctx.write_to_pty (System.Text.Encoding.UTF8.GetBytes "ls\n")
```

## Best Practices

### Performance

1. **Keep hooks fast** - Avoid blocking operations
2. **Use async/await** - For long-running operations
3. **Cache results** - Don't recompute on every call
4. **Profile your code** - Use Fusabi profiler

```fsharp
// Good: Fast check
let on_output line ctx =
    if line.StartsWith "ERROR" then
        ctx.notify "Error detected"
    Action.Continue

// Bad: Slow regex on every line
let on_output line ctx =
    let regex = Regex(@"complex.*pattern")
    if regex.IsMatch(line) then
        ctx.notify "Match found"
    Action.Continue
```

### Error Handling

Always return `Result` types and handle errors:

```fsharp
let on_load ctx =
    try
        // Initialization
        Ok ()
    with
    | ex ->
        ctx.log Error (sprintf "Failed to load: %s" ex.Message)
        Error ex.Message
```

### State Management

Prefer immutable state, use mutable sparingly:

```fsharp
// Good: Immutable
let process_line line state =
    { state with line_count = state.line_count + 1 }

// OK: Mutable when needed
let mutable is_recording = false

let toggle_recording () =
    is_recording <- not is_recording
```

### Documentation

Document your plugin thoroughly:

```fsharp
/// Parses git status output to detect repository state
/// Returns: Some RepoState if in git repo, None otherwise
let parse_git_status (output: string) : RepoState option =
    // Implementation
```

### Versioning

Follow Semantic Versioning:

- **Major** - Breaking changes to plugin API
- **Minor** - New features, backwards compatible
- **Patch** - Bug fixes, no API changes

```fsharp
// @version 1.2.3
//          | | |
//          | | +-- Patch (bug fixes)
//          | +---- Minor (new features)
//          +------ Major (breaking changes)
```

## Troubleshooting

### Plugin Not Loading

**Check metadata:**
```bash
just plugin-validate examples/fusabi/my-plugin.fsx
```

**Verify API version:**
```bash
grep "@api-version" examples/fusabi/my-plugin.fsx
```

**Check daemon logs:**
```bash
cargo run -p scarab-daemon 2>&1 | grep -i plugin
```

### Compilation Errors

**Verbose build:**
```bash
./scripts/build-plugin.sh -v examples/fusabi/my-plugin.fsx
```

**Check F# syntax:**
- Indentation must be consistent
- Types must match API definitions
- All functions must return correct types

### Runtime Errors

**Enable debug mode:**
```fsharp
let config = { debug_mode = true }

let on_output line ctx =
    if config.debug_mode then
        ctx.log Debug (sprintf "Processing: %s" line)
    Action.Continue
```

**Test in isolation:**
```bash
# Test just the plugin loading
cargo test -p scarab-daemon test_load_plugin -- --nocapture
```

### Performance Issues

**Profile your plugin:**
```fsharp
let on_output line ctx =
    let start = System.DateTime.Now
    // Your processing
    let elapsed = System.DateTime.Now - start
    if elapsed.TotalMilliseconds > 10.0 then
        ctx.log Warn (sprintf "Slow processing: %fms" elapsed.TotalMilliseconds)
    Action.Continue
```

**Check allocation:**
- Avoid creating objects in hot paths
- Reuse buffers and strings
- Use spans for byte operations

## IDE Setup

### VSCode

Recommended extensions (already configured in `.vscode/extensions.json`):

- `ionide.ionide-fsharp` - F# support
- `rust-lang.rust-analyzer` - Rust support
- `skellock.just` - Justfile support

### Tasks

Use VSCode tasks (Ctrl+Shift+P > "Tasks: Run Task"):

- **plugin: build all** - Build all plugins
- **plugin: validate all** - Validate all plugins
- **plugin: watch** - Watch mode

## CI/CD

### GitHub Actions

The `.github/workflows/plugins.yml` workflow automatically:

1. Validates plugin metadata
2. Checks F# syntax
3. Builds all plugins
4. Tests plugin loading
5. Checks API compatibility
6. Lints plugin code

### Pre-commit Hooks

Add to `.git/hooks/pre-commit`:

```bash
#!/bin/bash
echo "Validating plugins..."
just plugin-validate-all || exit 1
```

## Resources

- [Fusabi Language Guide](https://github.com/fusabi-lang/fusabi)
- [F# Documentation](https://fsharp.org/)
- [Plugin API Source](../crates/scarab-plugin-api/src/)
- [Example Plugins](../examples/fusabi/)

## Contributing

To contribute plugins:

1. Create plugin in `examples/fusabi/`
2. Add metadata and documentation
3. Run validation: `just plugin-validate-all`
4. Test thoroughly
5. Submit pull request

## License

See individual plugin files for license information.
