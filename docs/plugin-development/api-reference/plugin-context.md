# PluginContext API Reference

The `PluginContext` is the primary interface between your plugin and Scarab. It provides access to terminal state, logging, notifications, and more.

## Overview

Every hook function receives a `PluginContext` as its first parameter:

```fsharp
[<OnLoad>]
let onLoad (ctx: PluginContext) =
    ctx.Log Info "Plugin loaded!"
    async { return Ok () }
```

## Properties

### config: PluginConfigData

Access plugin-specific configuration from `plugin.toml`:

```fsharp
// Get required configuration value
let threshold: int = ctx.Config.Get "threshold"

// Get optional configuration value
let apiKey: string option = ctx.Config.GetOpt "api_key"
```

### state: Arc<Mutex<PluginSharedState>>

Access shared terminal state (advanced usage):

```fsharp
// Most plugins use convenience methods instead of direct state access
let (cols, rows) = ctx.GetSize()
```

## Terminal State Methods

### GetSize() -> (u16, u16)

Get terminal dimensions:

```fsharp
let (cols, rows) = ctx.GetSize()
ctx.Log Info (sprintf "Terminal size: %dx%d" cols rows)
```

**Returns:** Tuple of (columns, rows)

### GetCursor() -> (u16, u16)

Get cursor position:

```fsharp
let (x, y) = ctx.GetCursor()
ctx.Log Debug (sprintf "Cursor at (%d, %d)" x y)
```

**Returns:** Tuple of (x, y) coordinates

### GetCell(x: u16, y: u16) -> Cell option

Get cell contents at position:

```fsharp
match ctx.GetCell 10 5 with
| Some cell ->
    ctx.Log Debug (sprintf "Cell at (10,5): '%c'" cell.C)
| None ->
    ctx.Log Warn "Cell out of bounds"
```

**Returns:** `Some cell` if position is valid, `None` otherwise

**Cell structure:**
```fsharp
type Cell = {
    C: char              // Character
    Fg: (u8, u8, u8)    // Foreground RGB
    Bg: (u8, u8, u8)    // Background RGB
    Bold: bool
    Italic: bool
    Underline: bool
}
```

### SetCell(x: u16, y: u16, cell: Cell) -> bool

Set cell contents at position:

```fsharp
let cell = {
    C = '✓'
    Fg = (0, 255, 0)
    Bg = (0, 0, 0)
    Bold = true
    Italic = false
    Underline = false
}

if ctx.SetCell 0 0 cell then
    ctx.Log Debug "Cell updated"
else
    ctx.Log Warn "Cell out of bounds"
```

**Returns:** `true` if successful, `false` if out of bounds

### GetLine(y: u16) -> string option

Get entire line of text:

```fsharp
match ctx.GetLine 0 with
| Some line ->
    ctx.Log Info (sprintf "First line: %s" line)
| None ->
    ctx.Log Warn "Line out of bounds"
```

**Returns:** `Some text` if row is valid, `None` otherwise

**Note:** Trailing whitespace is trimmed

## Environment Methods

### GetEnv(key: string) -> string option

Get environment variable:

```fsharp
match ctx.GetEnv "USER" with
| Some username ->
    ctx.Log Info (sprintf "Hello, %s!" username)
| None ->
    ctx.Log Warn "USER environment variable not set"
```

**Common environment variables:**
- `USER` - Current username
- `HOME` - User home directory
- `SHELL` - Current shell path
- `PWD` - Present working directory
- `PATH` - Executable search paths

## Data Storage Methods

### SetData(key: string, value: string)

Store plugin-specific data:

```fsharp
// Store start time
let startTime = DateTime.Now.ToString("o")
ctx.SetData "start_time" startTime

// Store state
ctx.SetData "last_command" "git status"
```

**Use cases:**
- Storing state between hooks
- Caching computed values
- Tracking session data

### GetData(key: string) -> string option

Retrieve stored data:

```fsharp
match ctx.GetData "start_time" with
| Some startTime ->
    let start = DateTime.Parse(startTime)
    let duration = DateTime.Now - start
    ctx.Log Info (sprintf "Elapsed: %d ms" duration.TotalMilliseconds)
| None ->
    ctx.Log Warn "No start time found"
```

**Note:** Data is stored per-plugin and persists across hook invocations within the same session.

## Logging Methods

### Log(level: LogLevel, message: string)

Log a message:

```fsharp
ctx.Log Error "Something went wrong!"
ctx.Log Warn "This might be a problem"
ctx.Log Info "Plugin is working"
ctx.Log Debug "Detailed debugging info"
```

**Log Levels:**
- `Error` - Errors that prevent plugin functionality
- `Warn` - Warnings about potential issues
- `Info` - General informational messages
- `Debug` - Detailed debugging information

**Best practices:**
- Use `Error` for actual errors
- Use `Warn` for unexpected but handled situations
- Use `Info` for important events
- Use `Debug` for development/troubleshooting (disabled in production)

## Notification Methods

### Notify(title: string, body: string, level: NotifyLevel)

Show a notification to the user:

```fsharp
ctx.Notify "Build Complete" "All tests passed!" NotifyLevel.Success
ctx.Notify "URL Detected" "https://example.com" NotifyLevel.Info
ctx.Notify "Slow Command" "Command took 15 seconds" NotifyLevel.Warning
ctx.Notify "Build Failed" "Compilation error" NotifyLevel.Error
```

**Notification Levels:**
- `Success` - Green, checkmark icon
- `Info` - Blue, info icon
- `Warning` - Orange, warning icon
- `Error` - Red, error icon

**Best practices:**
- Keep titles short (< 30 chars)
- Keep bodies concise (< 100 chars)
- Don't spam notifications (rate limit to user actions)
- Use appropriate levels for visual hierarchy

### Convenience Methods

```fsharp
// These are shortcuts for common notification types
ctx.NotifySuccess "Title" "Body"
ctx.NotifyInfo "Title" "Body"
ctx.NotifyWarning "Title" "Body"
ctx.NotifyError "Title" "Body"
```

## Command Queue Methods

### QueueCommand(cmd: RemoteCommand)

Queue a command to send to client/daemon:

```fsharp
// Draw an overlay
ctx.QueueCommand (RemoteCommand.DrawOverlay {
    Id = 1UL
    X = 10us
    Y = 1us
    Text = "Status: Active"
    Style = OverlayStyle.Success
})

// Show a modal
ctx.QueueCommand (RemoteCommand.ShowModal {
    Title = "Command Palette"
    Items = commandList
})

// Clear overlays
ctx.QueueCommand (RemoteCommand.ClearOverlays { Id = Some 1UL })
```

**Available commands:**
- `DrawOverlay` - Draw text overlay at position
- `ClearOverlays` - Clear specific or all overlays
- `ShowModal` - Show modal dialog with items
- `PluginLog` - Send log message (use `ctx.Log` instead)
- `PluginNotify` - Send notification (use `ctx.Notify` instead)

**Note:** Most plugins use the convenience methods (`ctx.Notify`, etc.) instead of queueing commands directly.

## Complete Example

Here's a plugin using most PluginContext methods:

```fsharp
module example

open Scarab.PluginApi
open System

[<Plugin>]
let metadata = {
    Name = "example"
    Version = "0.1.0"
    Description = "Demonstrates PluginContext API"
    Author = "Scarab Team"
    Homepage = Some "https://github.com/raibid-labs/scarab"
    ApiVersion = "0.1.0"
    MinScarabVersion = "0.1.0"
    Emoji = Some "📚"
    Color = Some "#9C27B0"
    Catchphrase = Some "Learn by example!"
}

[<OnLoad>]
let onLoad (ctx: PluginContext) =
    async {
        // Log plugin startup
        ctx.Log Info "Example plugin loading..."

        // Get environment info
        match ctx.GetEnv "USER" with
        | Some user ->
            ctx.NotifyInfo "Welcome" (sprintf "Hello, %s!" user)
        | None ->
            ctx.NotifyWarning "User Unknown" "Could not determine username"

        // Get terminal size
        let (cols, rows) = ctx.GetSize()
        ctx.Log Info (sprintf "Terminal size: %dx%d" cols rows)

        // Store initialization time
        ctx.SetData "init_time" (DateTime.Now.ToString("o"))

        return Ok ()
    }

[<OnOutput>]
let onOutput (ctx: PluginContext) (line: string) =
    async {
        // Check first line content
        match ctx.GetLine 0 with
        | Some firstLine ->
            ctx.Log Debug (sprintf "First line: %s" firstLine)
        | None ->
            ctx.Log Warn "Could not read first line"

        // Process output
        if line.Contains("error") then
            ctx.NotifyError "Error Detected" line

        return Continue
    }

[<OnPreCommand>]
let onPreCommand (ctx: PluginContext) (command: string) =
    async {
        // Store command start time
        ctx.SetData "cmd_start" (DateTime.Now.ToString("o"))

        // Log command
        ctx.Log Info (sprintf "Running: %s" command)

        return Continue
    }

[<OnPostCommand>]
let onPostCommand (ctx: PluginContext) (command: string) (exitCode: int) =
    async {
        // Calculate duration
        match ctx.GetData "cmd_start" with
        | Some startStr ->
            let start = DateTime.Parse(startStr)
            let duration = (DateTime.Now - start).TotalMilliseconds

            ctx.Log Info (sprintf "Command took %.0fms (exit: %d)" duration exitCode)

            // Notify if slow
            if duration > 5000.0 then
                ctx.NotifyWarning "Slow Command" (sprintf "Took %.1fs" (duration / 1000.0))

        | None ->
            ctx.Log Warn "No start time found"

        return ()
    }
```

## Error Handling

All hooks should handle errors gracefully:

```fsharp
[<OnLoad>]
let onLoad (ctx: PluginContext) =
    async {
        try
            // Risky operation
            let data = performRiskyOperation()
            ctx.Log Info "Operation succeeded"
            return Ok ()
        with
        | ex ->
            ctx.Log Error (sprintf "Failed to load: %s" ex.Message)
            return Error (PluginError.InitializationError ex.Message)
    }
```

**Best practices:**
- Catch and log all exceptions
- Return meaningful error messages
- Don't crash the plugin system
- Use Result types for error handling

## Performance Considerations

### Do's
- ✅ Use async/await properly
- ✅ Return early from hooks when possible
- ✅ Cache computed values with SetData/GetData
- ✅ Use Debug logging sparingly
- ✅ Batch UI updates when possible

### Don'ts
- ❌ Don't block on I/O operations
- ❌ Don't call Log on every output line
- ❌ Don't spam notifications
- ❌ Don't do expensive computations in OnOutput
- ❌ Don't hold locks for long periods

## Next Steps

- **[Hooks Reference](hooks.md)** - All available hooks
- **[RemoteUI Components](remote-ui.md)** - Building UIs
- **[Tutorial 3: API Deep Dive](../tutorials/03-plugin-api-deep-dive.md)** - Comprehensive guide

## Getting Help

Questions about the API? Check:
- Example plugins in `plugins/examples/`
- Source code in `crates/scarab-plugin-api/`
- GitHub Discussions for community help
