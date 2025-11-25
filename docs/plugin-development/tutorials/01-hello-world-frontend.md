# Tutorial 1: Hello World (Frontend Plugin)

In this tutorial, you'll create your first Scarab plugin: a frontend plugin that displays a notification when Scarab starts.

**What you'll learn:**
- Creating a frontend (.fsx) plugin
- Using the `just dev-mode` hot-reload workflow
- Sending notifications to users
- Plugin metadata and configuration

**Time:** 15 minutes

## Prerequisites

- Scarab installed and working
- VSCode with recommended extensions (optional but recommended)
- `cargo-watch` installed: `cargo install cargo-watch`

## Step 1: Create the Plugin

Run the plugin scaffold generator:

```bash
just plugin-new hello-notification frontend
```

This creates a new plugin directory:

```
plugins/hello-notification/
├── hello-notification.fsx    # Plugin source code
├── plugin.toml               # Plugin configuration
└── README.md                 # Plugin documentation
```

## Step 2: Understand the Generated Code

Open `plugins/hello-notification/hello-notification.fsx`:

```fsharp
module hello_notification

open Scarab.PluginApi

[<Plugin>]
let metadata = {
    Name = "hello-notification"
    Version = "0.1.0"
    Description = "TODO: Add description"
    Author = "Your Name"
}

[<OnLoad>]
let onLoad (ctx: PluginContext) =
    ctx.Log Info "hello-notification loaded!"
    async { return Ok () }

[<OnKeyPress>]
let onKeyPress (ctx: PluginContext) (key: KeyEvent) =
    // TODO: Handle key presses
    async { return Continue }
```

Let's break this down:

### Plugin Metadata

```fsharp
[<Plugin>]
let metadata = { ... }
```

The `[<Plugin>]` attribute tells Scarab this is plugin metadata. Every plugin must define:
- `Name` - Unique identifier
- `Version` - Semantic version (semver)
- `Description` - What your plugin does
- `Author` - Your name

### Hook Functions

Functions decorated with hook attributes are called at specific points in the terminal lifecycle:

- `[<OnLoad>]` - Called when the plugin loads
- `[<OnKeyPress>]` - Called when a key is pressed
- `[<OnOutput>]` - Called when terminal output appears
- `[<OnInput>]` - Called when user sends input

All hooks are **async** and must return a `Result` or `Action`.

### The Plugin Context

The `PluginContext` provides access to:
- `ctx.Log()` - Log messages
- `ctx.Notify()` - Show notifications
- `ctx.GetCell()` - Read terminal cells
- `ctx.GetEnv()` - Access environment variables
- Many more (see [API Reference](../api-reference/plugin-context.md))

## Step 3: Add a Welcome Notification

Let's make the plugin show a notification when Scarab starts. Replace the `onLoad` function:

```fsharp
[<OnLoad>]
let onLoad (ctx: PluginContext) =
    // Log to the console
    ctx.Log Info "Hello Notification plugin loaded!"

    // Show a notification to the user
    ctx.Notify
        "Welcome to Scarab!"
        "This notification was sent by a plugin"
        NotifyLevel.Success

    async { return Ok () }
```

And update the metadata:

```fsharp
[<Plugin>]
let metadata = {
    Name = "hello-notification"
    Version = "0.1.0"
    Description = "Shows a welcome notification when Scarab starts"
    Author = "Your Name"
}
```

## Step 4: Remove Unused Hooks

Since we're not handling key presses, remove the `onKeyPress` function entirely. Your complete plugin should look like:

```fsharp
module hello_notification

open Scarab.PluginApi

[<Plugin>]
let metadata = {
    Name = "hello-notification"
    Version = "0.1.0"
    Description = "Shows a welcome notification when Scarab starts"
    Author = "Your Name"
}

[<OnLoad>]
let onLoad (ctx: PluginContext) =
    ctx.Log Info "Hello Notification plugin loaded!"

    ctx.Notify
        "Welcome to Scarab!"
        "This notification was sent by a plugin"
        NotifyLevel.Success

    async { return Ok () }
```

## Step 5: Configure the Plugin

Open `plugins/hello-notification/plugin.toml` and update it:

```toml
[plugin]
name = "hello-notification"
version = "0.1.0"
runtime = "frontend"

[plugin.metadata]
description = "Shows a welcome notification when Scarab starts"
author = "Your Name"
license = "MIT"

[hooks]
on_load = true
```

Note: We set `on_load = true` to enable the OnLoad hook. Only enabled hooks will be called.

## Step 6: Test with Hot Reload

Start the development server:

```bash
just dev-mode hello-notification
```

You should see:
```
🔄 Starting dev mode for hello-notification
   Watching: plugins/hello-notification
   Press Ctrl+C to stop
```

In another terminal, start Scarab:

```bash
just run-bg
```

You should see your notification appear in the Scarab window!

## Step 7: Make Live Changes

With `dev-mode` still running, edit `hello-notification.fsx` and change the notification message:

```fsharp
ctx.Notify
    "Hello from Plugin Land!"
    "Hot reload is working!"
    NotifyLevel.Info
```

**Save the file.** The plugin will automatically recompile and reload. Restart Scarab to see your changes.

## Step 8: Add Personality

Scarab plugins support optional personality attributes. Update your metadata:

```fsharp
[<Plugin>]
let metadata = {
    Name = "hello-notification"
    Version = "0.1.0"
    Description = "Shows a welcome notification when Scarab starts"
    Author = "Your Name"
    Emoji = Some "👋"
    Color = Some "#4CAF50"
    Catchphrase = Some "Welcome to your terminal!"
}
```

These attributes are used in the plugin list UI and logs for visual identification.

## Complete Plugin

Here's your complete first plugin:

```fsharp
module hello_notification

open Scarab.PluginApi

[<Plugin>]
let metadata = {
    Name = "hello-notification"
    Version = "0.1.0"
    Description = "Shows a welcome notification when Scarab starts"
    Author = "Your Name"
    Emoji = Some "👋"
    Color = Some "#4CAF50"
    Catchphrase = Some "Welcome to your terminal!"
}

[<OnLoad>]
let onLoad (ctx: PluginContext) =
    ctx.Log Info "Hello Notification plugin loaded!"

    ctx.Notify
        "Welcome to Scarab!"
        "This notification was sent by a plugin"
        NotifyLevel.Success

    async { return Ok () }
```

## What You Learned

- ✅ Creating frontend plugins with `just plugin-new`
- ✅ Plugin structure: metadata, hooks, and context
- ✅ Using `just dev-mode` for hot reloading
- ✅ Sending notifications with `ctx.Notify()`
- ✅ Configuring enabled hooks in `plugin.toml`
- ✅ Adding personality with emojis and colors

## Next Steps

→ **[Tutorial 2: Hello World (Backend)](02-hello-world-backend.md)** - Build a backend plugin that processes terminal output

→ **[Plugin API Deep Dive](03-plugin-api-deep-dive.md)** - Learn all available methods

→ **[Frontend UI with RemoteUI](05-frontend-ui-remoteui.md)** - Build complex UIs

## Troubleshooting

### Plugin not loading?

Check that:
1. Plugin is in `~/.config/scarab/config.toml`:
   ```toml
   [[plugins]]
   name = "hello-notification"
   enabled = true
   ```

2. `on_load = true` in `plugin.toml`

3. No syntax errors (check `dev-mode` output)

### Notification not showing?

Frontend plugins run in the client. Make sure:
- The client is running (not just the daemon)
- No other plugins are blocking notifications
- Check the client logs for errors

### Hot reload not working?

- Ensure `cargo-watch` is installed
- Check that `dev-mode` is watching the right directory
- Restart `dev-mode` if it gets stuck

## Challenge

Try these enhancements:

1. **Random greeting** - Show a different message each time
2. **Time-based greeting** - "Good morning" vs "Good evening"
3. **Multiple notifications** - Show 3 tips when starting
4. **Dismissable notification** - Add a "Don't show again" option

Happy coding!
