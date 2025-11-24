# Scarab Plugin Development Quick Start

This guide will get you writing Scarab plugins in 5 minutes.

## Prerequisites

- Scarab terminal emulator installed
- Basic F# syntax knowledge (or willingness to learn - it's quite readable!)
- Text editor

## Your First Plugin (3 Steps)

### Step 1: Create the plugin file

Create `~/.config/scarab/plugins/my-first-plugin.fsx`:

```fsharp
open Scarab.PluginApi

let metadata = {
    Name = "my-first-plugin"
    Version = "1.0.0"
    Description = "My first Scarab plugin"
    Author = "Your Name"
    Homepage = None
    ApiVersion = "0.1.0"
    MinScarabVersion = "0.1.0"
}

let on_load (ctx: PluginContext) : Async<Result<unit, string>> =
    async {
        ctx.Log(LogLevel.Info, "Hello from my first plugin!")
        return Ok ()
    }

Plugin.Register {
    Metadata = metadata
    OnLoad = on_load
    OnUnload = None
    OnOutput = None
    OnInput = None
    OnResize = None
    OnAttach = None
    OnDetach = None
    OnPreCommand = None
    OnPostCommand = None
    OnRemoteCommand = None
    GetCommands = fun () -> []
}
```

### Step 2: Enable the plugin

Add to `~/.config/scarab/config.toml`:

```toml
[plugins]
enabled = ["my-first-plugin.fsx"]
```

### Step 3: Test it

Start Scarab and check the logs - you should see "Hello from my first plugin!"

## Common Plugin Patterns

### Pattern 1: Intercept Output (Highlight Errors)

```fsharp
let on_output (line: string) (ctx: PluginContext) : Async<Result<Action, string>> =
    async {
        if line.Contains("ERROR") then
            ctx.Log(LogLevel.Warn, sprintf "Found error: %s" line)
            // Draw red indicator
            let (_, y) = ctx.GetCursor()
            ctx.QueueCommand(RemoteCommand.DrawOverlay {
                Id = 1UL
                X = 0us
                Y = y
                Text = "[!]"
                Style = { Fg = 0xFFFFFFFFu; Bg = 0xFF0000FFu; ZIndex = 50.0f }
            })
        return Ok Action.Continue
    }
```

### Pattern 2: Custom Keyboard Shortcut

```fsharp
let on_input (input: byte[]) (ctx: PluginContext) : Async<Result<Action, string>> =
    async {
        // Ctrl+X (0x18)
        if input = [|0x18uy|] then
            ctx.Notify("You pressed Ctrl+X!")
            return Ok (Action.Modify [||])  // Consume the key
        else
            return Ok Action.Continue  // Pass through
    }
```

### Pattern 3: Track Command Duration

```fsharp
let mutable startTime = DateTime.Now

let on_pre_command (cmd: string) (ctx: PluginContext) : Async<Result<Action, string>> =
    async {
        startTime <- DateTime.Now
        return Ok Action.Continue
    }

let on_post_command (cmd: string) (exitCode: int) (ctx: PluginContext) : Async<Result<unit, string>> =
    async {
        let duration = DateTime.Now - startTime
        if duration.TotalSeconds > 5.0 then
            ctx.Notify(sprintf "Command took %.1f seconds" duration.TotalSeconds)
        return Ok ()
    }
```

### Pattern 4: Add Command Palette Entry

```fsharp
let getCommands () : ModalItem list =
    [
        {
            Id = "my-plugin.action"
            Label = "Do Something Cool"
            Description = Some "Press this to trigger an action"
        }
    ]

let on_remote_command (id: string) (ctx: PluginContext) : Async<Result<unit, string>> =
    async {
        if id = "my-plugin.action" then
            ctx.Notify("Action triggered!")
        return Ok ()
    }
```

### Pattern 5: Configuration

In your plugin:
```fsharp
let on_load (ctx: PluginContext) : Async<Result<unit, string>> =
    async {
        match ctx.Config.GetOpt<string>("greeting") with
        | Some msg -> ctx.Log(LogLevel.Info, msg)
        | None -> ctx.Log(LogLevel.Info, "No greeting configured")
        return Ok ()
    }
```

In `~/.config/scarab/plugins.toml`:
```toml
[plugin.my-first-plugin]
greeting = "Hello, configured world!"
```

## Plugin Template

Save this as a starting point for new plugins:

```fsharp
(*
 * template-plugin.fsx
 *
 * Name: Template Plugin
 * Version: 1.0.0
 * Description: Template for new plugins
 * Author: Your Name
 * API Version: 0.1.0
 *)

open Scarab.PluginApi

// Metadata
let metadata = {
    Name = "template-plugin"
    Version = "1.0.0"
    Description = "Template for new plugins"
    Author = "Your Name"
    Homepage = None
    ApiVersion = "0.1.0"
    MinScarabVersion = "0.1.0"
}

// Hooks
let on_load (ctx: PluginContext) : Async<Result<unit, string>> =
    async {
        ctx.Log(LogLevel.Info, "Template plugin loaded")
        return Ok ()
    }

let on_unload (ctx: PluginContext) : Async<Result<unit, string>> =
    async {
        ctx.Log(LogLevel.Info, "Template plugin unloaded")
        return Ok ()
    }

// Register
Plugin.Register {
    Metadata = metadata
    OnLoad = on_load
    OnUnload = on_unload
    OnOutput = None
    OnInput = None
    OnResize = None
    OnAttach = None
    OnDetach = None
    OnPreCommand = None
    OnPostCommand = None
    OnRemoteCommand = None
    GetCommands = fun () -> []
}
```

## Debugging Tips

### Enable Debug Logging

In `~/.config/scarab/config.toml`:
```toml
[plugins.global]
log_level = "debug"
log_file = "~/.local/share/scarab/plugins.log"
```

Then tail the log:
```bash
tail -f ~/.local/share/scarab/plugins.log
```

### Test Hook Execution

Add logging to every hook:
```fsharp
let on_input (input: byte[]) (ctx: PluginContext) : Async<Result<Action, string>> =
    async {
        ctx.Log(LogLevel.Debug, sprintf "Input: %A" input)
        // Your logic here...
        return Ok Action.Continue
    }
```

### Inspect Terminal State

```fsharp
let inspect (ctx: PluginContext) =
    let (cols, rows) = ctx.GetSize()
    let (x, y) = ctx.GetCursor()
    ctx.Log(LogLevel.Debug, sprintf "Terminal: %dx%d, Cursor: (%d,%d)" cols rows x y)

    // Dump first 5 lines
    for i in 0us .. 4us do
        match ctx.GetLine(i) with
        | Some line -> ctx.Log(LogLevel.Debug, sprintf "Line %d: %s" i line)
        | None -> ()
```

## Key Bindings Quick Reference

Common control characters:
- Ctrl+A = 0x01
- Ctrl+B = 0x02
- ...
- Ctrl+Z = 0x1A
- Escape = 0x1B
- Ctrl+P = 0x10

Alt combinations (Escape + key):
- Alt+F = [0x1B, byte 'f']
- Alt+1 = [0x1B, byte '1']

## Next Steps

1. Read the full [README.md](README.md) for comprehensive API documentation
2. Study the example plugins in this directory
3. Join the Scarab community for questions and sharing
4. Check out [Fusabi language docs](https://github.com/fusabi-lang/fusabi) for advanced F# features

## Common Gotchas

1. **Always return a Result**: Hooks return `Result<T, string>`, not just `T`
2. **Use Async**: All hooks are async - wrap your code in `async { ... }`
3. **Unique overlay IDs**: Use a consistent range for your plugin (e.g., 5000-5999)
4. **Clean up on unload**: Clear overlays, timers, and resources
5. **Handle None cases**: Many API functions return `Option<T>`

## Getting Help

- Examples: `/examples/plugins/`
- API Reference: `/crates/scarab-plugin-api/src/`
- Core Plugins: `/crates/scarab-{palette,nav,platform}/`
- Issues: GitHub Issues
- Community: Discord/Forum

Happy plugin development!
