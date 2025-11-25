# Hooks Reference

Hooks are the entry points where Scarab calls your plugin code. This reference documents all available hooks with examples, performance characteristics, and best practices.

## Quick Reference

| Hook | Type | Frequency | Performance Impact | Use Case |
|------|------|-----------|-------------------|----------|
| `OnLoad` | Init | Once | Low | Plugin initialization |
| `OnUnload` | Cleanup | Once | Low | Resource cleanup |
| `OnOutput` | Backend | Per line | **High** | Process terminal output |
| `OnInput` | Backend | Per input | Medium | Intercept user input |
| `OnPreCommand` | Backend | Per command | Low | Before command runs |
| `OnPostCommand` | Backend | Per command | Low | After command completes |
| `OnResize` | Event | Occasional | Low | Terminal resized |
| `OnAttach` | Event | Occasional | Low | Client connected |
| `OnDetach` | Event | Occasional | Low | Client disconnected |
| `OnRemoteCommand` | Frontend | Variable | Low | Handle remote commands |

## Initialization Hooks

### OnLoad

Called when the plugin is loaded at startup or when enabled.

**Signature:**
```fsharp
[<OnLoad>]
let onLoad (ctx: PluginContext) : Async<Result<unit, PluginError>>
```

**Parameters:**
- `ctx` - Plugin context for accessing terminal state

**Returns:**
- `Ok ()` - Plugin loaded successfully
- `Error msg` - Plugin failed to load

**Example:**
```fsharp
[<OnLoad>]
let onLoad (ctx: PluginContext) =
    async {
        ctx.Log Info "My Plugin loaded!"

        // Initialize plugin state
        ctx.SetData "initialized" "true"

        // Get initial terminal size
        let (cols, rows) = ctx.GetSize()
        ctx.SetData "last_cols" (string cols)
        ctx.SetData "last_rows" (string rows)

        // Show startup notification
        ctx.NotifySuccess "Plugin Ready" "My Plugin is active"

        return Ok ()
    }
```

**Best Practices:**
- Keep initialization fast (< 100ms)
- Load configuration from `ctx.Config`
- Show user feedback if initialization is visible
- Return `Error` if plugin can't function
- Don't perform blocking I/O

**Use Cases:**
- Load plugin configuration
- Initialize data structures
- Connect to external services
- Show startup notifications
- Register commands

### OnUnload

Called when the plugin is being unloaded (Scarab shutdown or plugin disabled).

**Signature:**
```fsharp
[<OnUnload>]
let onUnload (ctx: PluginContext) : Async<Result<unit, PluginError>>
```

**Example:**
```fsharp
[<OnUnload>]
let onUnload (ctx: PluginContext) =
    async {
        ctx.Log Info "My Plugin unloading..."

        // Clean up resources
        closeConnections()
        saveState()

        // Clear overlays
        ctx.QueueCommand (RemoteCommand.ClearOverlays { Id = None })

        ctx.Log Info "My Plugin unloaded"

        return Ok ()
    }
```

**Best Practices:**
- Clean up resources (files, connections, etc.)
- Save plugin state if needed
- Don't throw exceptions
- Keep it fast (< 500ms)

**Use Cases:**
- Close file handles
- Disconnect from services
- Save session data
- Clear UI elements
- Release locks

## Terminal Output Hooks

### OnOutput

**Backend only.** Called for **every line** of terminal output before it's displayed.

**⚠️ Performance Critical:** This hook is called at very high frequency. Optimize carefully!

**Signature:**
```fsharp
[<OnOutput>]
let onOutput (ctx: PluginContext) (line: string) : Async<Action>
```

**Parameters:**
- `ctx` - Plugin context
- `line` - Line of terminal output (newline stripped)

**Returns:**
- `Continue` - Pass output to next plugin/display
- `Stop` - Block output from being displayed
- `Modify bytes` - Modify output before displaying

**Example:**
```fsharp
open System.Text.RegularExpressions

let urlRegex = Regex(@"https?://\S+", RegexOptions.Compiled)

[<OnOutput>]
let onOutput (ctx: PluginContext) (line: string) =
    async {
        // Quick check before expensive regex
        if not (line.Contains "http") then
            return Continue

        // Detect URLs
        if urlRegex.IsMatch(line) then
            let urls =
                urlRegex.Matches(line)
                |> Seq.cast<Match>
                |> Seq.map (fun m -> m.Value)
                |> Seq.toList

            ctx.Log Debug (sprintf "Found %d URLs" (List.length urls))

            // Notify user
            if urls.Length > 0 then
                ctx.NotifyInfo "URL Detected" (List.head urls)

        // Always pass through
        return Continue
    }
```

**Best Practices:**
- ✅ Return early if line doesn't match expected pattern
- ✅ Use compiled regex (`RegexOptions.Compiled`)
- ✅ Pre-filter with simple string checks before regex
- ✅ Use `ctx.Log Debug` sparingly
- ✅ Batch notifications (don't notify on every match)
- ❌ Don't perform blocking I/O
- ❌ Don't call external APIs on every line
- ❌ Don't use `Modify` unless necessary (performance cost)

**Performance:**
- **Budget:** < 50μs per line
- **Called:** 100-10,000+ times/second
- **Impact:** Directly affects terminal responsiveness

**Use Cases:**
- URL detection and highlighting
- Error detection and alerts
- Log parsing and colorization
- Pattern matching and tracking
- Command output analysis

## Terminal Input Hooks

### OnInput

**Backend only.** Called when user sends input to the terminal (before sent to PTY).

**Signature:**
```fsharp
[<OnInput>]
let onInput (ctx: PluginContext) (input: byte[]) : Async<Action>
```

**Parameters:**
- `ctx` - Plugin context
- `input` - Raw input bytes from user

**Returns:**
- `Continue` - Pass input to PTY
- `Stop` - Block input from reaching PTY
- `Modify bytes` - Modify input before sending to PTY

**Example:**
```fsharp
[<OnInput>]
let onInput (ctx: PluginContext) (input: byte[]) =
    async {
        // Detect Ctrl+R (0x12)
        if input.Length = 1 && input.[0] = 0x12uy then
            ctx.Log Info "Ctrl+R detected - showing history"

            // Show custom history search
            let! history = loadHistory()
            ctx.NotifyInfo "History Search" "Use arrow keys to navigate"

            // Block default Ctrl+R behavior
            return Stop

        // Pass through all other input
        return Continue
    }
```

**Best Practices:**
- Return `Continue` for most input
- Use `Stop` sparingly (only for overrides)
- Log intercepts for debugging
- Handle multi-byte sequences carefully (UTF-8)

**Common Input Bytes:**
- `0x03` - Ctrl+C
- `0x04` - Ctrl+D
- `0x12` - Ctrl+R
- `0x0A` - Enter
- `0x1B` - Escape

**Use Cases:**
- Custom keyboard shortcuts
- Input validation/sanitization
- Macro expansion
- Command history replacement
- Security filtering

## Command Hooks

### OnPreCommand

**Backend only.** Called before a command is executed.

**Signature:**
```fsharp
[<OnPreCommand>]
let onPreCommand (ctx: PluginContext) (command: string) : Async<Action>
```

**Parameters:**
- `ctx` - Plugin context
- `command` - Command about to be executed

**Returns:**
- `Continue` - Allow command to execute
- `Stop` - Block command execution
- `Modify bytes` - Modify command before execution

**Example:**
```fsharp
[<OnPreCommand>]
let onPreCommand (ctx: PluginContext) (command: string) =
    async {
        // Store start time for timing
        ctx.SetData "cmd_start" (DateTime.Now.ToString("o"))
        ctx.SetData "cmd_text" command

        // Log command
        ctx.Log Info (sprintf "Executing: %s" command)

        // Warn about dangerous commands
        if command.StartsWith("rm -rf /") then
            ctx.NotifyWarning "Dangerous Command" "Are you sure?"

        return Continue
    }
```

**Best Practices:**
- Use for timing command execution
- Track command history
- Warn about dangerous commands
- Don't block unless absolutely necessary

**Use Cases:**
- Command timing/profiling
- Command history tracking
- Security warnings
- Pre-execution validation
- Resource preparation

### OnPostCommand

**Backend only.** Called after a command completes.

**Signature:**
```fsharp
[<OnPostCommand>]
let onPostCommand (ctx: PluginContext) (command: string) (exitCode: int) : Async<unit>
```

**Parameters:**
- `ctx` - Plugin context
- `command` - Command that was executed
- `exitCode` - Command exit code (0 = success)

**Returns:**
- `()` - No return value (notification only)

**Example:**
```fsharp
[<OnPostCommand>]
let onPostCommand (ctx: PluginContext) (command: string) (exitCode: int) =
    async {
        // Calculate duration
        match ctx.GetData "cmd_start" with
        | Some startStr ->
            let start = DateTime.Parse(startStr)
            let duration = (DateTime.Now - start).TotalMilliseconds

            ctx.Log Info (sprintf "Command '%s' took %.0fms (exit: %d)" command duration exitCode)

            // Notify if slow or failed
            if duration > 5000.0 then
                ctx.NotifyWarning "Slow Command" (sprintf "Took %.1fs" (duration / 1000.0))
            elif exitCode <> 0 then
                ctx.NotifyError "Command Failed" (sprintf "Exit code: %d" exitCode)

        | None ->
            ctx.Log Debug "No start time found"

        return ()
    }
```

**Best Practices:**
- Check exit code for errors
- Calculate command duration
- Clear stored state
- Don't show notifications for every command

**Use Cases:**
- Command timing
- Error detection
- Statistics tracking
- Success/failure notifications
- Performance analysis

## Terminal Event Hooks

### OnResize

Called when the terminal is resized.

**Signature:**
```fsharp
[<OnResize>]
let onResize (ctx: PluginContext) (cols: u16) (rows: u16) : Async<unit>
```

**Parameters:**
- `ctx` - Plugin context
- `cols` - New width in columns
- `rows` - New height in rows

**Example:**
```fsharp
[<OnResize>]
let onResize (ctx: PluginContext) (cols: u16) (rows: u16) =
    async {
        ctx.Log Info (sprintf "Terminal resized to %dx%d" cols rows)

        // Update stored size
        ctx.SetData "cols" (string cols)
        ctx.SetData "rows" (string rows)

        // Reposition UI elements
        repositionOverlays cols rows

        return ()
    }
```

**Best Practices:**
- Update UI positions/sizes
- Recalculate layouts
- Don't show notifications on every resize

**Use Cases:**
- UI repositioning
- Layout recalculation
- Adaptive rendering
- Size tracking

### OnAttach

Called when a client attaches to the daemon session.

**Signature:**
```fsharp
[<OnAttach>]
let onAttach (ctx: PluginContext) (clientId: u64) : Async<unit>
```

**Parameters:**
- `ctx` - Plugin context
- `clientId` - Unique client identifier

**Example:**
```fsharp
[<OnAttach>]
let onAttach (ctx: PluginContext) (clientId: u64) =
    async {
        ctx.Log Info (sprintf "Client %d attached" clientId)

        // Send current state to new client
        syncStateToClient clientId

        return ()
    }
```

**Use Cases:**
- Multi-client sync
- State synchronization
- Welcome messages
- Session tracking

### OnDetach

Called when a client detaches from the daemon session.

**Signature:**
```fsharp
[<OnDetach>]
let onDetach (ctx: PluginContext) (clientId: u64) : Async<unit>
```

**Example:**
```fsharp
[<OnDetach>]
let onDetach (ctx: PluginContext) (clientId: u64) =
    async {
        ctx.Log Info (sprintf "Client %d detached" clientId)

        // Clean up client-specific state
        cleanupClientState clientId

        return ()
    }
```

**Use Cases:**
- Cleanup client state
- Session tracking
- Resource management

## Frontend-Specific Hooks

### OnRemoteCommand

**Frontend only.** Called when backend sends a command to frontend.

**Signature:**
```fsharp
[<OnRemoteCommand>]
let onRemoteCommand (ctx: PluginContext) (id: string) : Async<unit>
```

**Parameters:**
- `ctx` - Plugin context
- `id` - Command identifier sent by backend

**Example:**
```fsharp
// Backend (.fzb) sends command
[<OnOutput>]
let onOutput (ctx: PluginContext) (line: string) =
    async {
        if line.Contains "BUILD SUCCESS" then
            ctx.SendToFrontend "show-success-animation"
        return Continue
    }

// Frontend (.fsx) receives command
[<OnRemoteCommand>]
let onRemoteCommand (ctx: PluginContext) (id: string) =
    async {
        match id with
        | "show-success-animation" ->
            showSuccessAnimation()
        | _ ->
            ctx.Log Warn (sprintf "Unknown command: %s" id)

        return ()
    }
```

**Use Cases:**
- Backend → Frontend communication
- Hybrid plugin coordination
- Dynamic UI updates
- Event forwarding

## Hook Combinations

### Timing Pattern

```fsharp
[<OnPreCommand>]
let onPreCommand ctx cmd =
    async {
        ctx.SetData "start" (DateTime.Now.ToString("o"))
        return Continue
    }

[<OnPostCommand>]
let onPostCommand ctx cmd exitCode =
    async {
        match ctx.GetData "start" with
        | Some startStr ->
            let duration = DateTime.Now - DateTime.Parse(startStr)
            ctx.Log Info (sprintf "Duration: %A" duration)
        | None -> ()

        return ()
    }
```

### Output Detection + Notification

```fsharp
[<OnOutput>]
let onOutput ctx line =
    async {
        if line.Contains "error" then
            ctx.NotifyError "Error Detected" line
        return Continue
    }
```

### State Tracking

```fsharp
[<OnLoad>]
let onLoad ctx =
    async {
        ctx.SetData "counter" "0"
        return Ok ()
    }

[<OnOutput>]
let onOutput ctx line =
    async {
        let count =
            match ctx.GetData "counter" with
            | Some c -> int c
            | None -> 0

        ctx.SetData "counter" (string (count + 1))
        return Continue
    }
```

## Performance Benchmarks

Measured on a typical workstation:

| Hook | Avg Latency | Max Frequency | Budget |
|------|-------------|---------------|--------|
| OnLoad | 10ms | 1/session | 1s |
| OnOutput | 50μs | 10k/sec | 100μs |
| OnInput | 100μs | 100/sec | 1ms |
| OnPreCommand | 200μs | 10/sec | 5ms |
| OnPostCommand | 200μs | 10/sec | 5ms |
| OnResize | 500μs | 1/min | 10ms |

## Next Steps

- **[PluginContext Reference](plugin-context.md)** - Available methods
- **[RemoteUI Components](remote-ui.md)** - UI building blocks
- **[Example Plugins](../../plugins/examples/)** - Working examples
