# Debugging Scarab Plugins

A comprehensive guide to debugging, troubleshooting, and profiling Scarab plugins.

## Introduction

Plugin debugging involves multiple layers:
- **Plugin logic** - Your F# code in .fsx files
- **Runtime execution** - Fusabi VM or Frontend interpreter
- **IPC communication** - Between daemon and client
- **Scarab internals** - How your plugin interacts with the terminal

This guide covers tools and techniques for debugging at each layer.

## Logging and Output

Logging is your primary debugging tool. Scarab provides a structured logging API that integrates with the main terminal logs.

### Using ctx.Log() Effectively

The `PluginContext` provides a `Log()` method for all output:

```fsharp
ctx.Log(level, message)
```

**Available log levels**:

| Level | Usage | Display |
|-------|-------|---------|
| `Debug` | Verbose debugging info | Gray/dim |
| `Info` | General information | White |
| `Warning` | Potential issues | Yellow |
| `Error` | Errors that don't crash | Red |

### Basic Logging Example

```fsharp
[<OnLoad>]
let onLoad (ctx: PluginContext) =
    async {
        ctx.Log Info "Plugin initialized"
        ctx.Log Debug "Configuration loaded"
        return Ok ()
    }

[<OnOutput>]
let onOutput (ctx: PluginContext) (line: string) =
    async {
        ctx.Log Debug (sprintf "Processing line: %s" line)

        if line.Contains("ERROR") then
            ctx.Log Warning (sprintf "Detected error in output: %s" line)

        return Continue
    }
```

### Structured Logging Best Practices

**DO** - Use consistent prefixes:
```fsharp
ctx.Log Debug "[clipboard-monitor] Monitoring clipboard..."
ctx.Log Info "[clipboard-monitor] Copied: 42 bytes"
```

**DO** - Include context:
```fsharp
ctx.Log Error (sprintf "[git-status] Failed to read .git directory: %s" ex.Message)
```

**DON'T** - Log on every iteration:
```fsharp
// Bad: Logs 1000x per second
for i in 0 .. 1000 do
    ctx.Log Debug (sprintf "Iteration %d" i)  // Too verbose!
```

**DO** - Use conditional logging:
```fsharp
// Good: Log summary
let mutable count = 0
for i in 0 .. 1000 do
    count <- count + 1

if count > 0 then
    ctx.Log Debug (sprintf "Processed %d items" count)
```

### Log Levels in Detail

#### Debug Level

**When to use**: Detailed information for troubleshooting during development.

```fsharp
ctx.Log Debug "Entering onKeyPress handler"
ctx.Log Debug (sprintf "Key code: %A, Modifiers: %A" key.Code key.Modifiers)
```

**Enable in config**:
```toml
[plugins.config]
log_level = "debug"
```

#### Info Level

**When to use**: Normal operations and important events.

```fsharp
ctx.Log Info "Plugin loaded successfully"
ctx.Log Info "Git repository detected"
```

#### Warning Level

**When to use**: Potential issues that don't prevent operation.

```fsharp
ctx.Log Warning "Config file not found, using defaults"
ctx.Log Warning "Clipboard access denied, feature disabled"
```

#### Error Level

**When to use**: Errors that affect functionality but don't crash.

```fsharp
ctx.Log Error "Failed to connect to database"
ctx.Log Error (sprintf "Parse error: %s" parseError)
```

### Viewing Logs in Scarab

#### Real-time Log Tailing

Watch logs as they happen:
```bash
tail -f ~/.local/share/scarab/plugins.log
```

Filter to specific plugin:
```bash
tail -f ~/.local/share/scarab/plugins.log | grep "\[my-plugin\]"
```

#### Log File Location

**Default path**: `~/.local/share/scarab/plugins.log`

**Custom path** (in config.toml):
```toml
[plugins.config]
log_file = "/tmp/scarab-debug.log"
```

#### Log Format

```
[2025-12-02 10:23:45.123] [INFO] [clipboard-monitor] Monitoring started
[2025-12-02 10:23:46.456] [DEBUG] [clipboard-monitor] Clipboard changed: 42 bytes
[2025-12-02 10:23:47.789] [WARN] [clipboard-monitor] Access denied to system clipboard
[2025-12-02 10:23:48.012] [ERROR] [clipboard-monitor] Failed to parse clipboard data
```

### Advanced Logging Techniques

#### Conditional Debug Logging

Enable verbose logging per-plugin:

```fsharp
let debugEnabled =
    ctx.Config.GetOpt<bool>("debug") |> Option.defaultValue false

let debugLog msg =
    if debugEnabled then
        ctx.Log Debug msg

// Usage
debugLog "This only appears if debug=true in config"
```

Config:
```toml
[plugins.my-plugin]
debug = true
```

#### Performance Logging

Track execution time:

```fsharp
let logTimed (name: string) (operation: unit -> 'a) : 'a =
    let sw = System.Diagnostics.Stopwatch.StartNew()
    let result = operation()
    sw.Stop()
    ctx.Log Debug (sprintf "[%s] took %dms" name sw.ElapsedMilliseconds)
    result

// Usage
let data = logTimed "load-config" (fun () ->
    loadConfigFromFile "config.json"
)
```

#### Structured Data Logging

Log complex data structures:

```fsharp
open System.Text.Json

let logObject (name: string) (obj: 'a) =
    let json = JsonSerializer.Serialize(obj, JsonSerializerOptions(WriteIndented = true))
    ctx.Log Debug (sprintf "%s = %s" name json)

// Usage
logObject "terminal-state" {| Width = 80; Height = 24; CursorX = 5; CursorY = 10 |}
```

## VSCode Debugging Setup

### Launch Configuration

The Scarab workspace includes debug configurations in `.vscode/launch.json`:

```json
{
  "type": "lldb",
  "request": "launch",
  "name": "Debug Scarab Daemon",
  "cargo": {
    "args": ["build", "--bin=scarab-daemon", "--package=scarab-daemon"]
  },
  "env": {
    "RUST_BACKTRACE": "1"
  }
}
```

### Debugging the Daemon (Backend Plugins)

1. **Set breakpoints** in Rust code
2. **Start debugging**: Press `F5` or select "Debug Scarab Daemon"
3. **Trigger your plugin hook**

### Debugging the Client (Frontend Plugins)

1. **Set breakpoints** in client code
2. **Start debugging**: Press `F5` or select "Debug Scarab Client"
3. **Plugin execution** triggers breakpoints

### Debugging Fusabi Code (Advanced)

Currently, debugging .fsx files requires printf-style debugging with `ctx.Log()`. Native breakpoint support for Fusabi is planned for future releases.

**Workaround** - Add strategic log points:

```fsharp
[<OnKeyPress>]
let onKeyPress (ctx: PluginContext) (key: KeyEvent) =
    async {
        ctx.Log Debug ">>> onKeyPress START"
        ctx.Log Debug (sprintf "    Key: %A" key)

        let handled =
            match key.Code with
            | Key.F1 ->
                ctx.Log Debug "    Matched F1"
                true
            | _ ->
                ctx.Log Debug "    No match"
                false

        ctx.Log Debug (sprintf "<<< onKeyPress END (handled=%b)" handled)

        return if handled then Handled else Continue
    }
```

## Common Issues and Solutions

### Issue 1: Plugin Won't Load

**Symptoms**:
- Plugin doesn't appear in plugin list
- No log messages from plugin
- Scarab starts but plugin inactive

**Debugging steps**:

1. **Check plugin is enabled**:
   ```toml
   [[plugins]]
   name = "my-plugin"
   enabled = true
   ```

2. **Verify file structure**:
   ```bash
   ls -R ~/.config/scarab/plugins/my-plugin/
   # Should show:
   # my-plugin.fsx
   # plugin.toml
   ```

3. **Check logs for load errors**:
   ```bash
   grep -i "error" ~/.local/share/scarab/plugins.log
   grep -i "my-plugin" ~/.local/share/scarab/plugins.log
   ```

4. **Validate manifest**:
   ```bash
   just plugin-validate my-plugin
   ```

**Common causes**:
- Syntax error in .fsx file
- Malformed plugin.toml
- Missing required metadata fields
- Wrong file permissions

### Issue 2: Hooks Not Firing

**Symptoms**:
- Plugin loads successfully
- But specific hooks never execute
- No errors in logs

**Debugging steps**:

1. **Add entry/exit logging**:
   ```fsharp
   [<OnOutput>]
   let onOutput (ctx: PluginContext) (line: string) =
       async {
           ctx.Log Debug "OnOutput called!"  // This should appear
           // ... rest of logic
       }
   ```

2. **Check hook registration** in plugin.toml:
   ```toml
   [hooks]
   on_output = true  # Must be enabled!
   ```

3. **Verify hook is triggerable**:
   - `OnOutput` - Type in terminal, run commands
   - `OnKeyPress` - Press keys
   - `OnResize` - Resize terminal window

4. **Verify plugin type matches runtime**:
   - Backend plugins (`backend`) run in daemon - have `OnOutput`, `OnPreCommand`
   - Frontend plugins (`frontend`) run in client - have `OnKeyPress`, `OnResize`

### Issue 3: UI Not Rendering

**Symptoms**:
- Overlay/notification calls succeed
- But nothing appears on screen

**Debugging steps**:

1. **Check z-index**:
   ```fsharp
   // Too low - might be behind other elements
   ctx.SetOverlay("test", "Hello", { ZIndex = 1.0f })

   // Better - above most UI
   ctx.SetOverlay("test", "Hello", { ZIndex = 100.0f })
   ```

2. **Verify coordinates**:
   ```fsharp
   let (cols, rows) = ctx.GetSize()
   ctx.Log Debug (sprintf "Terminal size: %dx%d" cols rows)

   // Ensure overlay is within bounds
   if x < cols && y < rows then
       ctx.SetOverlay(...)
   ```

3. **Check color format** (RGBA):
   ```fsharp
   // Wrong: RGB
   { Fg = 0xFFFFFF }

   // Correct: RGBA (alpha = FF for opaque)
   { Fg = 0xFFFFFFFF }
   ```

4. **Check if frontend plugin**:
   - Only frontend plugins can render UI
   - Backend plugins must send IPC messages

### Issue 4: Performance Problems

**Symptoms**:
- Terminal feels sluggish
- Typing has lag
- High CPU usage

**Debugging steps**:

1. **Profile with timing**:
   ```fsharp
   let sw = System.Diagnostics.Stopwatch.StartNew()

   // Your plugin logic here

   sw.Stop()
   if sw.ElapsedMilliseconds > 10 then
       ctx.Log Warning (sprintf "Slow operation: %dms" sw.ElapsedMilliseconds)
   ```

2. **Check hook frequency**:
   ```fsharp
   let mutable callCount = 0
   [<OnOutput>]
   let onOutput (ctx: PluginContext) (line: string) =
       async {
           callCount <- callCount + 1
           if callCount % 1000 = 0 then
               ctx.Log Debug (sprintf "OnOutput called %d times" callCount)
       }
   ```

3. **Optimize hot paths**:
   ```fsharp
   // Slow: Regex on every line
   let regex = Regex(@"ERROR: (.*)")

   // Fast: Simple string check first
   if line.Contains("ERROR:") then  // Cheap!
       // Only use regex if necessary
       let regex = Regex(@"ERROR: (.*)")
       if regex.IsMatch(line) then
           // ...
   ```

4. **Compile to bytecode**:
   ```bash
   # .fsx is interpreted (slower)
   # .fzb is compiled (2-5x faster)
   just plugin-build my-plugin
   ```

## Debug Mode Flags

### Enable Verbose Logging

**In config.toml**:
```toml
[plugins.config]
log_level = "debug"
verbose = true
```

**Environment variable**:
```bash
SCARAB_PLUGIN_DEBUG=1 cargo run -p scarab-daemon
```

### Dry Run Mode

Test plugin without side effects:

```bash
cargo run -p scarab-daemon -- --plugin-dry-run
```

**Effect**: Plugins load and hooks execute, but no actual actions performed.

### Plugin Isolation Mode

Run single plugin in isolation:

```bash
cargo run -p scarab-daemon -- --plugin-only my-plugin
```

## Troubleshooting Checklist

Use this checklist when debugging plugin issues:

### Loading Phase
- [ ] Plugin directory exists and is readable
- [ ] .fsx and plugin.toml files present
- [ ] plugin.toml syntax is valid
- [ ] Plugin name in toml matches directory name
- [ ] Plugin enabled in config.toml
- [ ] No syntax errors in .fsx file
- [ ] Scarab logs show plugin loaded

### Runtime Phase
- [ ] Hooks are registered in plugin.toml
- [ ] Plugin type (frontend/backend) matches hook usage
- [ ] OnLoad executes (check logs)
- [ ] Hooks have entry logging
- [ ] No early returns or exceptions
- [ ] Async blocks return proper types

### UI Phase (Frontend only)
- [ ] Overlay/notification IDs are unique
- [ ] Coordinates within terminal bounds
- [ ] Colors in RGBA format (0xRRGGBBAAu)
- [ ] Z-index high enough (> 50)
- [ ] Client process is running

### Performance Phase
- [ ] No loops in hot paths (OnOutput, OnInput)
- [ ] String operations optimized
- [ ] Regex compiled once, not per call
- [ ] No unnecessary allocations
- [ ] Consider compiling to .fzb

## Getting Help

When reporting issues, include:

1. **Plugin source** (relevant parts)
2. **Log output** (with debug level enabled)
3. **Steps to reproduce**
4. **Expected vs actual behavior**
5. **Scarab version**: `scarab --version`
6. **System info**: `uname -a`

## Quick Reference

### Essential Debug Commands

```bash
# Watch logs live
tail -f ~/.local/share/scarab/plugins.log

# Filter to specific plugin
tail -f ~/.local/share/scarab/plugins.log | grep "\[my-plugin\]"

# Enable debug logging
echo '[plugins.config]
log_level = "debug"' >> ~/.config/scarab/config.toml

# Validate plugin
just plugin-validate my-plugin

# Test plugin loading
cargo test -p scarab-plugin-api -- my-plugin
```

### Debug Logging Template

```fsharp
let debugLog =
    let enabled = ctx.Config.GetOpt<bool>("debug") |> Option.defaultValue false
    fun msg -> if enabled then ctx.Log Debug msg

[<OnKeyPress>]
let onKeyPress (ctx: PluginContext) (key: KeyEvent) =
    async {
        debugLog ">>> onKeyPress START"
        debugLog (sprintf "Key: %A" key)

        // Your logic here

        debugLog "<<< onKeyPress END"
        return Continue
    }
```

### Common Log Patterns

```fsharp
// Entry/exit logging
ctx.Log Debug ">>> functionName START"
// ... logic ...
ctx.Log Debug "<<< functionName END"

// Conditional logging
if result.IsError then
    ctx.Log Error (sprintf "Operation failed: %A" result)

// Performance logging
let sw = Stopwatch.StartNew()
// ... operation ...
ctx.Log Debug (sprintf "Operation took %dms" sw.ElapsedMilliseconds)

// State inspection
ctx.Log Debug (sprintf "State: %A" currentState)
```

## Next Steps

- **Master the API**: [../api-reference/](../api-reference/)
- **Study examples**: [/plugins/examples/](../../plugins/examples/)
- **Optimize performance**: [../tutorials/performance.md](../tutorials/performance.md)
- **Contribute**: Share your debugging tips with the community!
