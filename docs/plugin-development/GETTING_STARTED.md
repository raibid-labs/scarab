# Plugin Development Guide

Welcome to Scarab plugin development! This guide will help you build powerful plugins using Fusabi, a high-performance F# dialect for Rust.

## Table of Contents

1. [Overview](#overview)
2. [Quick Start](#quick-start)
3. [Plugin Architecture](#plugin-architecture)
4. [Plugin API Reference](#plugin-api-reference)
5. [Development Workflow](#development-workflow)
6. [Best Practices](#best-practices)
7. [Publishing Plugins](#publishing-plugins)

---

## Overview

### What are Scarab Plugins?

Scarab plugins extend terminal functionality using the Fusabi scripting language. Plugins can:

- Intercept and modify terminal output
- Handle keyboard input and create custom keybindings
- Draw overlays and UI elements
- Register custom commands in the command palette
- React to terminal events (resize, command execution, etc.)
- Maintain state across terminal sessions

### Plugin Types

Scarab supports two plugin runtimes:

| Runtime | File Type | Location | Use Case |
|---------|-----------|----------|----------|
| **Frontend** | `.fsx` | Client | UI scripting, overlays, menus, hot-reloadable |
| **Backend** | `.fzb` | Daemon | High-performance hooks, output scanning, compiled bytecode |

For most use cases, start with **frontend** plugins. They're easier to develop and support hot reloading.

### Current API Version

**v0.1.0** - See [API_VERSION](../../crates/scarab-plugin-api/src/lib.rs) for compatibility info.

---

## Quick Start

### 1. Create a New Plugin

```bash
just plugin-new my-plugin frontend
```

This creates a plugin scaffold at `plugins/my-plugin/`:

```
plugins/my-plugin/
├── my-plugin.fsx      # Plugin source code
├── plugin.toml        # Plugin manifest
└── README.md          # Documentation
```

### 2. Edit Your Plugin

Open `plugins/my-plugin/my-plugin.fsx`:

```fsharp
module my_plugin

open Scarab.PluginApi

[<Plugin>]
let metadata = {
    Name = "my-plugin"
    Version = "0.1.0"
    Description = "My awesome plugin"
    Author = "Your Name"
    Homepage = Some "https://github.com/yourusername/my-plugin"
    ApiVersion = "0.1.0"
    MinScarabVersion = "0.1.0"
    Emoji = Some "🚀"
    Color = Some "#FF5733"
    Catchphrase = Some "Making terminals awesome!"
}

[<OnLoad>]
let onLoad (ctx: PluginContext) =
    async {
        ctx.Log Info "My plugin loaded successfully!"
        return Ok ()
    }
```

### 3. Enable Hot Reload

```bash
just dev-mode my-plugin
```

This watches for changes and automatically reloads your plugin. Keep this running while developing.

### 4. Test Your Plugin

1. Start Scarab daemon: `just daemon`
2. Start Scarab client: `just client`
3. Your plugin loads automatically
4. Check logs for "My plugin loaded successfully!"

---

## Plugin Architecture

### Plugin Structure

Every plugin consists of:

1. **Metadata** - Plugin information and capabilities
2. **Hooks** - Event handlers for terminal events
3. **Commands** - Custom commands for the command palette
4. **Menu Items** - Actions for the Scarab Dock

### Minimal Plugin Example

```fsharp
module hello_plugin

open Scarab.PluginApi

[<Plugin>]
let metadata = {
    Name = "hello-plugin"
    Version = "1.0.0"
    Description = "Simple hello world plugin"
    Author = "Scarab Examples"
    Homepage = Some "https://github.com/scarab-terminal/scarab"
    ApiVersion = "0.1.0"
    MinScarabVersion = "0.1.0"
}

[<OnLoad>]
let onLoad (ctx: PluginContext) =
    async {
        // Get terminal info
        let (cols, rows) = ctx.GetSize()
        ctx.Log Info (sprintf "Terminal size: %dx%d" cols rows)

        // Access environment
        match ctx.GetEnv "USER" with
        | Some user -> ctx.Log Info (sprintf "Running for user: %s" user)
        | None -> ()

        return Ok ()
    }
```

---

## Plugin API Reference

### Available Hooks

Plugins can implement these hooks to respond to terminal events:

#### `[<OnLoad>]`
Called when the plugin is first loaded.

```fsharp
[<OnLoad>]
let onLoad (ctx: PluginContext) : Async<Result<unit, string>> =
    async {
        ctx.Log Info "Plugin initialized"
        return Ok ()
    }
```

#### `[<OnUnload>]`
Called when the plugin is being unloaded. Clean up resources here.

```fsharp
[<OnUnload>]
let onUnload (ctx: PluginContext) : Async<Result<unit, string>> =
    async {
        ctx.Log Info "Cleaning up..."
        return Ok ()
    }
```

#### `[<OnOutput>]`
Called for each line of terminal output before it's displayed.

```fsharp
[<OnOutput>]
let onOutput (ctx: PluginContext) (line: string) : Async<Result<Action, string>> =
    async {
        // Detect errors
        if line.Contains "error" then
            ctx.NotifyError "Error Detected" line

        return Continue  // or Stop, or Modify data
    }
```

**Return values:**
- `Continue` - Pass through to next plugin
- `Stop` - Stop processing chain
- `Modify data` - Transform the output

#### `[<OnInput>]`
Called when user input is received, before it reaches the PTY.

```fsharp
[<OnInput>]
let onInput (ctx: PluginContext) (input: byte[]) : Async<Result<Action, string>> =
    async {
        // Intercept Ctrl+K
        if input = [|0x0Buy|] then
            ctx.NotifyInfo "Hotkey" "Ctrl+K pressed!"
            return Modify [||]  // Consume the input

        return Continue  // Pass through
    }
```

#### `[<OnPreCommand>]`
Called before a command is executed.

```fsharp
[<OnPreCommand>]
let onPreCommand (ctx: PluginContext) (command: string) : Async<Result<Action, string>> =
    async {
        if command.StartsWith "git" then
            ctx.Log Debug (sprintf "Git command: %s" command)

        return Continue
    }
```

#### `[<OnPostCommand>]`
Called after a command completes.

```fsharp
[<OnPostCommand>]
let onPostCommand (ctx: PluginContext) (command: string) (exitCode: int) =
    async {
        if exitCode <> 0 then
            ctx.NotifyWarning "Command Failed" (sprintf "Exit code: %d" exitCode)

        return Ok ()
    }
```

#### `[<OnResize>]`
Called when the terminal is resized.

```fsharp
[<OnResize>]
let onResize (ctx: PluginContext) (cols: u16) (rows: u16) =
    async {
        ctx.Log Info (sprintf "Terminal resized to %dx%d" cols rows)
        return Ok ()
    }
```

#### `[<OnRemoteCommand>]`
Called when a menu action or command palette item is triggered.

```fsharp
[<OnRemoteCommand>]
let onRemoteCommand (ctx: PluginContext) (id: string) =
    async {
        match id with
        | "show-stats" ->
            let (cols, rows) = ctx.GetSize()
            ctx.NotifyInfo "Stats" (sprintf "Size: %dx%d" cols rows)
        | _ ->
            ctx.Log Warn (sprintf "Unknown command: %s" id)

        return Ok ()
    }
```

### Plugin Context API

The `PluginContext` provides access to terminal state and actions:

#### Terminal State

```fsharp
// Get terminal dimensions
let (cols, rows) = ctx.GetSize()

// Get cursor position
let (x, y) = ctx.GetCursor()

// Read a specific line
match ctx.GetLine 5 with
| Some text -> ctx.Log Info text
| None -> ()

// Access environment variables
match ctx.GetEnv "HOME" with
| Some home -> ctx.Log Info home
| None -> ()
```

#### Logging

```fsharp
// Log levels: Error, Warn, Info, Debug
ctx.Log Error "Something went wrong!"
ctx.Log Warn "This is a warning"
ctx.Log Info "Informational message"
ctx.Log Debug "Debug details"
```

#### Notifications

```fsharp
// Send user notifications (shown as overlays)
ctx.NotifyInfo "Title" "Message body"
ctx.NotifySuccess "Success!" "Operation completed"
ctx.NotifyWarning "Warning" "Something to watch"
ctx.NotifyError "Error" "Something failed"
```

#### Plugin Data Storage

```fsharp
// Store plugin-specific state
ctx.SetData "last_command" "git status"
ctx.SetData "counter" "42"

// Retrieve stored data
match ctx.GetData "last_command" with
| Some cmd -> ctx.Log Info cmd
| None -> ()
```

#### Configuration

```fsharp
// Read from plugin.toml [config] section
match ctx.Config.GetOpt<bool> "enabled" with
| Some true -> ctx.Log Info "Feature enabled"
| Some false -> ctx.Log Info "Feature disabled"
| None -> ctx.Log Info "Using default"

// Required config value (throws if missing)
let threshold = ctx.Config.Get<int> "threshold"
```

### Menu System

Plugins can register menu items for the Scarab Dock:

```fsharp
// In your plugin's GetMenu function
let getMenu () =
    [
        MenuItem.new "Refresh" (MenuAction.Remote "refresh")
            |> MenuItem.withIcon "🔄"
            |> MenuItem.withShortcut "Ctrl+R"

        MenuItem.new "Settings" (MenuAction.SubMenu [
            MenuItem.new "Enable" (MenuAction.Remote "toggle-enable")
            MenuItem.new "Disable" (MenuAction.Remote "toggle-disable")
        ])

        MenuItem.new "Run Command" (MenuAction.Command "echo Hello")
    ]
```

**Menu Action Types:**

- `MenuAction.Command "cmd"` - Execute shell command
- `MenuAction.Remote "id"` - Trigger `OnRemoteCommand` with ID
- `MenuAction.SubMenu items` - Nested menu

### Remote Commands

Queue UI commands to be executed on the client:

```fsharp
// Draw an overlay
ctx.QueueCommand(RemoteCommand.DrawOverlay {
    Id = 1UL
    X = 0us
    Y = 10us
    Text = "[INFO] Processing..."
    Style = {
        Fg = 0xFFFFFFFFu  // White text
        Bg = 0x0000FFFFu  // Blue background
        ZIndex = 50.0f
    }
})

// Clear overlays
ctx.QueueCommand(RemoteCommand.ClearOverlays { Id = None })

// Show modal dialog
ctx.QueueCommand(RemoteCommand.ShowModal {
    Title = "Choose Action"
    Items = [
        { Id = "opt1"; Label = "Option 1"; Description = Some "First option" }
        { Id = "opt2"; Label = "Option 2"; Description = Some "Second option" }
    ]
})

// Update theme
ctx.QueueCommand(RemoteCommand.ApplyTheme {
    PluginName = "my-plugin"
    ThemeName = "dark"
})
```

---

## Development Workflow

### Hot Reload Development

The fastest way to iterate on plugins:

```bash
# Terminal 1: Start hot reload watcher
just dev-mode my-plugin

# Terminal 2: Run Scarab daemon
just daemon

# Terminal 3: Run Scarab client
just client
```

Now edit `plugins/my-plugin/my-plugin.fsx` and save. The plugin reloads automatically!

### Building Plugins

```bash
# Build a specific plugin
just plugin-build my-plugin

# Build all plugins
just plugin-build-all

# Validate plugin manifest
just plugin-validate plugins/my-plugin/plugin.toml
```

### Testing Plugins

```bash
# Run plugin-specific tests
just plugin-test my-plugin

# Test plugin loading
just plugin-test-loading

# Full plugin CI
just plugin-ci
```

### Debugging

**View plugin logs:**

```bash
# Check daemon logs
tail -f /tmp/scarab-daemon.log

# Or run daemon in foreground
just daemon
```

**Common debugging patterns:**

```fsharp
[<OnLoad>]
let onLoad (ctx: PluginContext) =
    async {
        // Log startup info
        ctx.Log Info "=== Plugin Starting ==="
        ctx.Log Info (sprintf "API Version: %s" metadata.ApiVersion)

        // Dump environment
        match ctx.GetEnv "PATH" with
        | Some path -> ctx.Log Debug (sprintf "PATH: %s" path)
        | None -> ()

        // Test configuration
        ctx.Log Debug "Testing config..."
        match ctx.Config.GetOpt<string> "test_key" with
        | Some val -> ctx.Log Info (sprintf "Config loaded: %s" val)
        | None -> ctx.Log Warn "No config found"

        return Ok ()
    }
```

### Plugin Configuration

Edit `plugins/my-plugin/plugin.toml`:

```toml
[plugin]
name = "my-plugin"
version = "0.1.0"
runtime = "frontend"

[plugin.metadata]
description = "My awesome plugin"
author = "Your Name"
license = "MIT"
homepage = "https://github.com/yourusername/my-plugin"

# Enable hooks (comment out unused ones)
[hooks]
on_load = true
on_output = true
# on_input = true
# on_resize = true
# on_key_press = true

# Plugin-specific configuration
[config]
enabled = true
notify_on_detection = false
max_items = 10
```

Access config in your plugin:

```fsharp
let enabled = ctx.Config.GetOpt<bool> "enabled" |> Option.defaultValue true
let maxItems = ctx.Config.GetOpt<int> "max_items" |> Option.defaultValue 10
```

### User Configuration

Users enable your plugin in `~/.config/scarab/config.toml`:

```toml
[[plugins]]
name = "my-plugin"
enabled = true

# Plugin-specific settings override plugin.toml [config]
[plugins.config]
enabled = true
max_items = 20
custom_setting = "value"
```

---

## Best Practices

### Performance Considerations

**1. Minimize work in hot paths**

```fsharp
// BAD: Heavy regex on every output line
[<OnOutput>]
let onOutput (ctx: PluginContext) (line: string) =
    async {
        let heavyRegex = System.Text.RegularExpressions.Regex(@"complex\s+pattern")
        if heavyRegex.IsMatch line then
            // ...
        return Continue
    }

// GOOD: Compile regex once at module level
let heavyRegex = System.Text.RegularExpressions.Regex(
    @"complex\s+pattern",
    System.Text.RegularExpressions.RegexOptions.Compiled
)

[<OnOutput>]
let onOutput (ctx: PluginContext) (line: string) =
    async {
        if heavyRegex.IsMatch line then
            // ...
        return Continue
    }
```

**2. Use Continue when possible**

```fsharp
// Return Continue quickly if no work needed
[<OnOutput>]
let onOutput (ctx: PluginContext) (line: string) =
    async {
        // Quick early return
        if not (line.Contains "keyword") then
            return Continue

        // Only do work if needed
        doExpensiveProcessing line
        return Continue
    }
```

**3. Debounce notifications**

```fsharp
let mutable lastNotification = System.DateTime.MinValue

[<OnOutput>]
let onOutput (ctx: PluginContext) (line: string) =
    async {
        let now = System.DateTime.Now

        // Only notify once per second
        if (now - lastNotification).TotalSeconds > 1.0 then
            ctx.NotifyInfo "Alert" "Multiple matches found"
            lastNotification <- now

        return Continue
    }
```

### State Management

**1. Use mutable state sparingly**

```fsharp
// For simple counters or flags
let mutable eventCount = 0

[<OnOutput>]
let onOutput (ctx: PluginContext) (line: string) =
    async {
        eventCount <- eventCount + 1
        return Continue
    }
```

**2. Use context data storage for persistence**

```fsharp
// Store state that should survive plugin reloads
[<OnOutput>]
let onOutput (ctx: PluginContext) (line: string) =
    async {
        // Get current count
        let count =
            match ctx.GetData "count" with
            | Some s -> int s
            | None -> 0

        // Increment and store
        ctx.SetData "count" (string (count + 1))

        return Continue
    }
```

**3. Initialize state in OnLoad**

```fsharp
[<OnLoad>]
let onLoad (ctx: PluginContext) =
    async {
        // Initialize plugin state
        ctx.SetData "initialized" "true"
        ctx.SetData "count" "0"
        ctx.SetData "last_command" ""

        return Ok ()
    }
```

### Error Handling

**1. Always handle errors gracefully**

```fsharp
[<OnOutput>]
let onOutput (ctx: PluginContext) (line: string) =
    async {
        try
            // Risky operation
            let result = parseComplexData line
            ctx.Log Info (sprintf "Parsed: %s" result)
        with
        | ex ->
            ctx.Log Error (sprintf "Parse error: %s" ex.Message)
            // Don't crash the plugin - return Continue

        return Continue
    }
```

**2. Validate configuration early**

```fsharp
[<OnLoad>]
let onLoad (ctx: PluginContext) =
    async {
        // Validate required config
        match ctx.Config.GetOpt<string> "api_key" with
        | Some key when key.Length > 0 ->
            ctx.Log Info "API key configured"
        | _ ->
            ctx.Log Error "Missing API key in config!"
            return Error "Configuration error: api_key required"

        return Ok ()
    }
```

**3. Use Result types for operations that can fail**

```fsharp
let parseGitBranch (line: string) : Result<string, string> =
    if line.StartsWith "On branch " then
        Ok (line.Substring(10).Trim())
    else
        Error "Not a branch line"

[<OnOutput>]
let onOutput (ctx: PluginContext) (line: string) =
    async {
        match parseGitBranch line with
        | Ok branch ->
            ctx.Log Info (sprintf "Branch: %s" branch)
        | Error msg ->
            ctx.Log Debug msg

        return Continue
    }
```

### Resource Cleanup

```fsharp
// Track resources created during plugin lifetime
let mutable overlayIds : uint64 list = []

[<OnLoad>]
let onLoad (ctx: PluginContext) =
    async {
        overlayIds <- []
        return Ok ()
    }

[<OnOutput>]
let onOutput (ctx: PluginContext) (line: string) =
    async {
        if line.Contains "error" then
            let id = System.Random().NextUInt64()
            overlayIds <- id :: overlayIds

            ctx.QueueCommand(RemoteCommand.DrawOverlay {
                Id = id
                X = 0us
                Y = 0us
                Text = "[ERROR]"
                Style = defaultStyle
            })

        return Continue
    }

[<OnUnload>]
let onUnload (ctx: PluginContext) =
    async {
        // Clean up all overlays
        for id in overlayIds do
            ctx.QueueCommand(RemoteCommand.ClearOverlays { Id = Some id })

        ctx.Log Info "Cleanup complete"
        return Ok ()
    }
```

---

## Publishing Plugins

### 1. Package Your Plugin

```bash
just plugin-package my-plugin
```

This creates `dist/plugins/my-plugin.tar.gz` containing:
- `my-plugin.fsx` - Source code
- `plugin.toml` - Manifest
- `README.md` - Documentation

### 2. Create Documentation

Your `README.md` should include:

```markdown
# My Plugin

Description of what your plugin does.

## Features

- Feature 1
- Feature 2
- Feature 3

## Installation

```bash
# Clone or download plugin
cd ~/.config/scarab/plugins
tar -xzf my-plugin.tar.gz

# Enable in config
echo '[[plugins]]' >> ~/.config/scarab/config.toml
echo 'name = "my-plugin"' >> ~/.config/scarab/config.toml
echo 'enabled = true' >> ~/.config/scarab/config.toml
```

## Configuration

Add to `~/.config/scarab/config.toml`:

```toml
[[plugins]]
name = "my-plugin"
enabled = true

[plugins.config]
setting1 = "value1"
setting2 = 42
```

## Usage

Describe how to use the plugin.

## License

MIT
```

### 3. Version Your Plugin

Follow [Semantic Versioning](https://semver.org/):

- `1.0.0` - Major release
- `1.1.0` - New features (minor)
- `1.1.1` - Bug fixes (patch)

Update both `plugin.toml` and your metadata:

```toml
# plugin.toml
[plugin]
version = "1.1.0"
```

```fsharp
// my-plugin.fsx
let metadata = {
    Name = "my-plugin"
    Version = "1.1.0"
    // ...
}
```

### 4. Share Your Plugin

- Create a GitHub repository
- Add to the [Scarab Plugin Registry](https://github.com/raibid-labs/scarab-plugins) (coming soon)
- Share on the Scarab Discord/forum

---

## Examples

### Example 1: URL Detector

Detects URLs in terminal output and highlights them.

```fsharp
module url_detector

open Scarab.PluginApi
open System.Text.RegularExpressions

[<Plugin>]
let metadata = {
    Name = "url-detector"
    Version = "0.1.0"
    Description = "Detects and highlights URLs"
    Author = "Scarab Team"
    ApiVersion = "0.1.0"
    MinScarabVersion = "0.1.0"
    Emoji = Some "🔗"
}

let urlPattern = Regex(
    @"https?://[^\s]+",
    RegexOptions.Compiled
)

[<OnOutput>]
let onOutput (ctx: PluginContext) (line: string) =
    async {
        let matches = urlPattern.Matches line

        if matches.Count > 0 then
            let urls = [for m in matches -> m.Value]
            ctx.Log Info (sprintf "Found %d URLs" matches.Count)

            // Notify user
            let urlList = String.concat ", " urls
            ctx.NotifyInfo "URLs Detected" urlList

        return Continue
    }
```

### Example 2: Git Status Monitor

Tracks git branch and status changes.

```fsharp
module git_status

open Scarab.PluginApi
open System.Text.RegularExpressions

[<Plugin>]
let metadata = {
    Name = "git-status"
    Version = "0.1.0"
    Description = "Monitors git branch and status"
    Author = "Scarab Team"
    ApiVersion = "0.1.0"
    MinScarabVersion = "0.1.0"
    Emoji = Some "🌿"
    Color = Some "#F05032"
}

let branchPattern = Regex(@"On branch (\S+)", RegexOptions.Compiled)

[<OnLoad>]
let onLoad (ctx: PluginContext) =
    async {
        ctx.SetData "git_branch" "unknown"
        return Ok ()
    }

[<OnOutput>]
let onOutput (ctx: PluginContext) (line: string) =
    async {
        let m = branchPattern.Match line

        if m.Success then
            let branch = m.Groups.[1].Value
            ctx.SetData "git_branch" branch
            ctx.NotifyInfo "Git Branch" (sprintf "⎇ %s" branch)

        return Continue
    }
```

### Example 3: Command Timer

Times command execution and reports duration.

```fsharp
module command_timer

open Scarab.PluginApi
open System

[<Plugin>]
let metadata = {
    Name = "command-timer"
    Version = "0.1.0"
    Description = "Times command execution"
    Author = "Scarab Team"
    ApiVersion = "0.1.0"
    MinScarabVersion = "0.1.0"
    Emoji = Some "⏱️"
}

let mutable commandStart = DateTime.MinValue

[<OnPreCommand>]
let onPreCommand (ctx: PluginContext) (command: string) =
    async {
        commandStart <- DateTime.Now
        ctx.Log Debug (sprintf "Command started: %s" command)
        return Continue
    }

[<OnPostCommand>]
let onPostCommand (ctx: PluginContext) (command: string) (exitCode: int) =
    async {
        let duration = DateTime.Now - commandStart
        let ms = duration.TotalMilliseconds

        ctx.Log Info (sprintf "Command took %.2fms" ms)

        if ms > 1000.0 then
            ctx.NotifyWarning "Slow Command" (sprintf "Took %.2fs" (ms / 1000.0))

        return Ok ()
    }
```

---

## Next Steps

- Explore [example plugins](../../plugins/examples/) for more patterns
- Read the [API Reference](./api-reference/) for detailed documentation
- Join the [Scarab Discord](https://discord.gg/scarab) for help
- Check out [Fusabi Language Guide](../FUSABI_LANGUAGE.md)

Happy plugin development!
