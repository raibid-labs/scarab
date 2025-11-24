# Scarab Plugin Examples

This directory contains example plugins for the Scarab terminal emulator, written in Fusabi (F# dialect for Rust).

## Overview

Scarab's plugin system allows you to extend the terminal with custom behavior through **Fusabi scripts** (.fsx files) or compiled bytecode (.fzb files). These examples demonstrate common plugin patterns and the available API.

## Example Plugins

### 1. hello-plugin.fsx
**Simplest possible plugin** - demonstrates basic structure and logging.
- Shows how to register a plugin
- Uses the `on_load` hook
- Accesses terminal state and environment variables

### 2. output-filter.fsx
**Output interception and highlighting** - detects patterns in terminal output.
- Uses the `on_output` hook to intercept each line
- Regex pattern matching for errors, warnings, and success messages
- Draws colored overlays to highlight important output
- Configuration-driven behavior

### 3. custom-keybind.fsx
**Keyboard shortcuts and command palette integration** - intercepts keys.
- Uses the `on_input` hook to detect key combinations
- Implements a modal "command mode" (like Vim)
- Shows help, stats, and inserts timestamps
- Registers commands in the command palette
- Uses the `on_remote_command` hook for palette integration

### 4. git-status.fsx
**Environment awareness and command tracking** - displays git status.
- Uses `on_pre_command` and `on_post_command` hooks
- Runs external processes (git commands)
- Displays persistent status overlay in top-right corner
- Updates automatically when git commands are run
- Responds to terminal resize events

### 5. notification-monitor.fsx
**Long-running command notifications** - tracks execution time.
- Tracks command start/end times
- Sends notifications for commands exceeding a threshold
- Demonstrates stateful plugin design with mutable dictionaries
- Configuration for customizable threshold

## Plugin Anatomy

Every Fusabi plugin has the same basic structure:

```fsharp
open Scarab.PluginApi

// 1. Metadata (required)
let metadata = {
    Name = "my-plugin"
    Version = "1.0.0"
    Description = "What this plugin does"
    Author = "Your Name"
    Homepage = Some "https://github.com/..."
    ApiVersion = "0.1.0"
    MinScarabVersion = "0.1.0"
}

// 2. Hook implementations (optional)
let on_load (ctx: PluginContext) : Async<Result<unit, string>> =
    async {
        ctx.Log(LogLevel.Info, "Plugin loaded!")
        return Ok ()
    }

// 3. Plugin registration (required)
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

## Available Hooks

### Lifecycle Hooks

- **on_load**: Called when plugin is loaded
  ```fsharp
  let on_load (ctx: PluginContext) : Async<Result<unit, string>>
  ```

- **on_unload**: Called when plugin is being unloaded (cleanup resources)
  ```fsharp
  let on_unload (ctx: PluginContext) : Async<Result<unit, string>>
  ```

### Terminal Event Hooks

- **on_output**: Intercept output before it's displayed
  ```fsharp
  let on_output (line: string) (ctx: PluginContext) : Async<Result<Action, string>>
  ```
  Returns:
  - `Action.Continue` - pass through unchanged
  - `Action.Stop` - block this line from other plugins/display
  - `Action.Modify bytes` - replace line with modified data

- **on_input**: Intercept keyboard/mouse input
  ```fsharp
  let on_input (input: byte[]) (ctx: PluginContext) : Async<Result<Action, string>>
  ```
  Returns same Actions as on_output

- **on_resize**: Called when terminal is resized
  ```fsharp
  let on_resize (cols: uint16) (rows: uint16) (ctx: PluginContext) : Async<Result<unit, string>>
  ```

### Command Hooks

- **on_pre_command**: Called before a shell command executes
  ```fsharp
  let on_pre_command (command: string) (ctx: PluginContext) : Async<Result<Action, string>>
  ```

- **on_post_command**: Called after command completes
  ```fsharp
  let on_post_command (command: string) (exitCode: int) (ctx: PluginContext) : Async<Result<unit, string>>
  ```

### Client Hooks

- **on_attach**: Called when a client connects
  ```fsharp
  let on_attach (clientId: uint64) (ctx: PluginContext) : Async<Result<unit, string>>
  ```

- **on_detach**: Called when a client disconnects
  ```fsharp
  let on_detach (clientId: uint64) (ctx: PluginContext) : Async<Result<unit, string>>
  ```

### Remote UI Hooks

- **on_remote_command**: Handle commands triggered from command palette
  ```fsharp
  let on_remote_command (id: string) (ctx: PluginContext) : Async<Result<unit, string>>
  ```

- **get_commands**: Register commands in the command palette
  ```fsharp
  let getCommands () : ModalItem list =
      [
          { Id = "my-plugin.action"; Label = "Do Something"; Description = Some "Help text" }
      ]
  ```

## Plugin Context API

The `PluginContext` provides access to terminal state and remote commands:

### Logging
```fsharp
ctx.Log(LogLevel.Info, "message")      // Info level
ctx.Log(LogLevel.Debug, "message")     // Debug level
ctx.Log(LogLevel.Warn, "message")      // Warning level
ctx.Log(LogLevel.Error, "message")     // Error level
ctx.Notify("User notification")        // Send notification to user
```

### Terminal State
```fsharp
let (cols, rows) = ctx.GetSize()       // Terminal dimensions
let (x, y) = ctx.GetCursor()           // Cursor position
let line = ctx.GetLine(5)              // Get text at row 5
let cell = ctx.GetCell(10, 5)          // Get cell at (10, 5)
ctx.SetCell(10, 5, cell)               // Set cell at (10, 5)
```

### Environment & Data
```fsharp
let user = ctx.GetEnv("USER")          // Get environment variable
ctx.SetData("key", "value")            // Store plugin-specific data
let value = ctx.GetData("key")         // Retrieve plugin-specific data
```

### Configuration
```fsharp
// Read from [plugin.my-plugin] section in config
let enabled = ctx.Config.GetOpt<bool>("enabled")
let threshold = ctx.Config.GetOpt<float>("threshold_seconds")
let pattern = ctx.Config.GetOpt<string>("pattern")
```

### Remote Commands

Queue commands to be sent to the client for UI updates:

#### Draw Overlay
```fsharp
ctx.QueueCommand(RemoteCommand.DrawOverlay {
    Id = 12345UL                       // Unique identifier
    X = 0us                             // Column position
    Y = 0us                             // Row position
    Text = "[INFO]"                     // Text to display
    Style = {
        Fg = 0xFFFFFFFFu                // RGBA foreground (white)
        Bg = 0xFF0000FFu                // RGBA background (red)
        ZIndex = 100.0f                 // Z-order for layering
    }
})
```

#### Clear Overlays
```fsharp
ctx.QueueCommand(RemoteCommand.ClearOverlays {
    Id = Some 12345UL                   // Clear specific overlay
})
ctx.QueueCommand(RemoteCommand.ClearOverlays {
    Id = None                           // Clear all overlays
})
```

#### Show Modal (Command Palette)
```fsharp
ctx.QueueCommand(RemoteCommand.ShowModal {
    Title = "Choose Action"
    Items = [
        { Id = "action-1"; Label = "First Option"; Description = Some "Help text" }
        { Id = "action-2"; Label = "Second Option"; Description = None }
    ]
})
```

#### Hide Modal
```fsharp
ctx.QueueCommand(RemoteCommand.HideModal)
```

## Color Format

Colors use 32-bit RGBA format (0xRRGGBBAA):
- `0xFFFFFFFF` - White (opaque)
- `0x000000FF` - Black (opaque)
- `0xFF0000FF` - Red (opaque)
- `0x00FF00FF` - Green (opaque)
- `0x0000FFFF` - Blue (opaque)
- `0xFFFF00FF` - Yellow (opaque)
- `0xFF00FFFF` - Magenta (opaque)
- `0x00FFFFFF` - Cyan (opaque)
- `0xFFFFFF80` - White (50% transparent)

## Configuration

Plugins can read configuration from the Scarab config file:

```toml
# ~/.config/scarab/config.toml

[plugins]
enabled = ["output-filter", "git-status", "notification-monitor"]

[plugin.output-filter]
enabled = true

[plugin.git-status]
show_dirty = true
position = "top-right"

[plugin.notification-monitor]
threshold_seconds = 10.0
```

## Installation & Loading

### Loading from Config
Add plugins to your `~/.config/scarab/config.toml`:

```toml
[plugins]
enabled = ["hello-plugin", "output-filter"]
```

### Hot Reloading (.fsx scripts)
Since .fsx files are interpreted, they can be reloaded without restarting:

```bash
# Send reload command via IPC
scarab-client --reload-plugins
```

### Compiled Plugins (.fzb bytecode)
For performance-critical plugins, compile to bytecode:

```bash
# Compile .fsx to .fzb (using Fusabi compiler)
fusabi compile my-plugin.fsx -o my-plugin.fzb
```

Then reference in config:
```toml
[plugins]
enabled = ["my-plugin.fzb"]
```

## Best Practices

1. **Always handle errors**: Return `Ok ()` or `Error "reason"` from hooks
2. **Log appropriately**: Use Debug for verbose, Info for important events
3. **Clean up on unload**: Clear overlays, cancel timers, close resources
4. **Use unique IDs**: For overlays, use a consistent range (e.g., 10000-10999)
5. **Be efficient**: The `on_output` hook is called frequently - keep it fast
6. **Test with edge cases**: Empty terminal, resize, rapid input
7. **Document configuration**: Explain all config options in comments

## Plugin Development Workflow

1. **Start with hello-plugin.fsx** as a template
2. **Add your hooks** incrementally (start with on_load)
3. **Test frequently** by loading in Scarab
4. **Use logging liberally** during development
5. **Refine and optimize** once behavior is correct
6. **Add configuration** for customization
7. **Compile to .fzb** if performance is critical

## Debugging Tips

```fsharp
// Log terminal state
ctx.Log(LogLevel.Debug, sprintf "Size: %A" (ctx.GetSize()))

// Log input bytes
ctx.Log(LogLevel.Debug, sprintf "Input: %A" input)

// Dump current line
match ctx.GetLine(0) with
| Some line -> ctx.Log(LogLevel.Debug, sprintf "Line 0: %s" line)
| None -> ()

// Check configuration
ctx.Log(LogLevel.Debug, sprintf "Config: %A" ctx.Config)
```

## Resources

- **Fusabi Language Docs**: https://github.com/fusabi-lang/fusabi
- **Scarab Plugin API**: /crates/scarab-plugin-api/
- **Core Plugins**: /crates/scarab-{palette,nav,platform}/
- **Protocol Reference**: /crates/scarab-protocol/src/lib.rs

## Contributing

Submit your plugins as pull requests! Good examples help everyone learn.

## License

These examples are released under the same license as Scarab (MIT/Apache-2.0).
