# Plugin Logging and Notifications System

## Overview

Scarab provides a comprehensive bidirectional logging and notification system that allows plugins to communicate with the daemon and all connected clients. This enables plugins to:

- **Log messages** at different severity levels for debugging and monitoring
- **Send notifications** to users for important events and status updates
- **Integrate seamlessly** with Rust's standard logging infrastructure
- **Provide rich UI feedback** through the client's notification overlay system

## Architecture

The logging and notification system follows Scarab's split-process architecture:

```
┌─────────────────┐
│     Plugin      │
│   (in Daemon)   │
└────────┬────────┘
         │ ctx.log()
         │ ctx.notify()
         ▼
┌─────────────────┐
│ Plugin Context  │
│  (Queue Cmds)   │
└────────┬────────┘
         │ RemoteCommand::PluginLog
         │ RemoteCommand::PluginNotify
         ▼
┌─────────────────┐
│ Plugin Manager  │
│ (Process Queue) │
└────────┬────────┘
         │ DaemonMessage::PluginLog
         │ DaemonMessage::PluginNotification
         ▼
┌─────────────────┐
│  IPC Broadcast  │
│  (All Clients)  │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│ Client UI Layer │
│ (Notifications) │
└─────────────────┘
```

## Logging API

### Log Levels

The system supports four log levels, matching Rust's standard `log` crate:

| Level | Purpose | Example Use Case |
|-------|---------|------------------|
| `Error` | Critical errors requiring attention | Plugin initialization failed |
| `Warn` | Warning conditions that should be reviewed | Configuration deprecated |
| `Info` | Informational messages about normal operation | Plugin loaded successfully |
| `Debug` | Detailed debugging information | Processing 1000th line |

### Using ctx.log()

**Rust API:**

```rust
use scarab_plugin_api::context::LogLevel;

impl Plugin for MyPlugin {
    async fn on_load(&mut self, ctx: &mut PluginContext) -> Result<()> {
        // Log at different levels
        ctx.log(LogLevel::Info, "Plugin initialized");
        ctx.log(LogLevel::Debug, "Configuration loaded from ~/.config");
        ctx.log(LogLevel::Warn, "Using default theme");
        ctx.log(LogLevel::Error, "Failed to connect to service");

        Ok(())
    }
}
```

**Fusabi (F#) API:**

```fsharp
let on_load (ctx: PluginContext) : Async<Result<unit, string>> =
    async {
        // Log at different levels
        ctx.Log(LogLevel.Info, "Plugin initialized")
        ctx.Log(LogLevel.Debug, "Configuration loaded")
        ctx.Log(LogLevel.Warn, "Using default theme")
        ctx.Log(LogLevel.Error, "Failed to connect")

        return Ok ()
    }
```

### Behavior

When you call `ctx.log()`:

1. **Local Logging**: Message is sent to the daemon's logging system (Rust `log` crate)
2. **Client Forwarding**: Message is queued as a `RemoteCommand::PluginLog`
3. **Broadcast**: Daemon broadcasts `DaemonMessage::PluginLog` to all connected clients
4. **Client Display**: Clients log to their console using Bevy's logging system

## Notification API

### Notification Levels

Notifications have four severity levels that control visual styling:

| Level | Color | Icon | Use Case |
|-------|-------|------|----------|
| `Success` | Green | ✓ OK | Operation completed successfully |
| `Info` | Blue | ℹ INFO | General information |
| `Warning` | Orange | ⚠ WARN | Non-critical issues |
| `Error` | Red | ✗ ERROR | Critical failures |

### Using ctx.notify()

**Rust API:**

```rust
use scarab_plugin_api::context::NotifyLevel;

impl Plugin for MyPlugin {
    async fn on_output(&self, data: &str, ctx: &PluginContext) -> Result<Action> {
        if data.contains("error") {
            ctx.notify("Build Failed", "Compilation errors detected", NotifyLevel::Error);
        }

        if data.contains("success") {
            ctx.notify("Build Complete", "All tests passed!", NotifyLevel::Success);
        }

        Ok(Action::Continue)
    }
}
```

**Fusabi (F#) API:**

```fsharp
let on_output (data: string) (ctx: PluginContext) : Async<Result<Action, string>> =
    async {
        if data.Contains("error") then
            ctx.Notify("Build Failed", "Compilation errors detected", NotifyLevel.Error)

        if data.Contains("success") then
            ctx.Notify("Build Complete", "All tests passed!", NotifyLevel.Success)

        return Ok Action.Continue
    }
```

### Convenience Methods

For common notification levels, convenience methods are available:

**Rust:**
```rust
ctx.notify_info("Title", "Body");
ctx.notify_success("Title", "Body");
ctx.notify_warning("Title", "Body");
ctx.notify_error("Title", "Body");
```

**Fusabi:**
```fsharp
ctx.NotifyInfo("Title", "Body")
ctx.NotifySuccess("Title", "Body")
ctx.NotifyWarning("Title", "Body")
ctx.NotifyError("Title", "Body")
```

### Notification UI Behavior

When a notification is sent:

1. **Appears in top-right corner** of the client window
2. **Auto-dismisses after 5 seconds** (configurable in future versions)
3. **Stacks vertically** when multiple notifications are active
4. **Color-coded background** based on severity level
5. **Shows title and body text** with appropriate styling

## Complete Example Plugin

Here's a comprehensive example demonstrating both logging and notifications:

```fsharp
(*
 * monitor-plugin.fsx
 * Monitors terminal output and sends appropriate notifications
 *)

open Scarab.PluginApi

let metadata = {
    Name = "output-monitor"
    Version = "1.0.0"
    Description = "Monitors terminal output and notifies user of important events"
    Author = "Your Name"
    ApiVersion = "0.1.0"
    MinScarabVersion = "0.1.0"
}

let on_load (ctx: PluginContext) : Async<Result<unit, string>> =
    async {
        ctx.Log(LogLevel.Info, "Output Monitor plugin loaded")
        ctx.NotifySuccess("Plugin Ready", "Output monitoring is now active")
        ctx.SetData("events_detected", "0")
        return Ok ()
    }

let on_output (data: string) (ctx: PluginContext) : Async<Result<Action, string>> =
    async {
        // Detect build errors
        if data.Contains("error:") || data.Contains("Error:") then
            ctx.Log(LogLevel.Warn, sprintf "Build error detected: %s" data)
            ctx.NotifyError("Build Error", "Check terminal for details")

            // Track events
            let count = ctx.GetData("events_detected") |> Option.defaultValue "0" |> int
            ctx.SetData("events_detected", string (count + 1))

        // Detect successful builds
        if data.Contains("Build succeeded") || data.Contains("Finished") then
            ctx.Log(LogLevel.Info, "Build completed successfully")
            ctx.NotifySuccess("Build Complete", "Your build finished successfully")

        // Detect long-running operations
        if data.Contains("Compiling") then
            ctx.Log(LogLevel.Debug, sprintf "Compilation started: %s" data)

        // Detect test results
        if data.Contains("test result:") then
            if data.Contains("ok") then
                ctx.NotifySuccess("Tests Passed", "All tests completed successfully")
            else
                ctx.NotifyWarning("Tests Failed", "Some tests did not pass")

        return Ok Action.Continue
    }

let on_remote_command (id: string) (ctx: PluginContext) : Async<Result<unit, string>> =
    async {
        match id with
        | "monitor.stats" ->
            let events = ctx.GetData("events_detected") |> Option.defaultValue "0"
            ctx.Log(LogLevel.Info, sprintf "Detected %s events since load" events)
            ctx.NotifyInfo("Monitor Statistics", sprintf "%s events detected" events)
            return Ok ()
        | _ ->
            return Ok ()
    }

let get_commands () : ModalItem list =
    [
        {
            Id = "monitor.stats"
            Label = "📊 Show Monitor Stats"
            Description = Some "Display how many events have been detected"
        }
    ]

Plugin.Register {
    Metadata = metadata
    OnLoad = on_load
    OnUnload = None
    OnOutput = Some on_output
    OnInput = None
    OnResize = None
    OnAttach = None
    OnDetach = None
    OnPreCommand = None
    OnPostCommand = None
    OnRemoteCommand = Some on_remote_command
    GetCommands = get_commands
}
```

## Protocol Details

### ControlMessage Enum (Client to Daemon)

```rust
pub enum ControlMessage {
    // ... existing variants ...

    PluginLog {
        plugin_name: String,
        level: LogLevel,
        message: String,
    },
    PluginNotify {
        title: String,
        body: String,
        level: NotifyLevel,
    },
}
```

### DaemonMessage Enum (Daemon to Client)

```rust
pub enum DaemonMessage {
    // ... existing variants ...

    PluginLog {
        plugin_name: String,
        level: LogLevel,
        message: String,
    },
    PluginNotification {
        title: String,
        body: String,
        level: NotifyLevel,
    },
}
```

### RemoteCommand Enum (Plugin API)

```rust
pub enum RemoteCommand {
    // ... existing variants ...

    PluginLog {
        plugin_name: String,
        level: LogLevel,
        message: String,
    },
    PluginNotify {
        title: String,
        body: String,
        level: NotifyLevel,
    },
}
```

## Implementation Details

### Plugin Context (scarab-plugin-api)

**Location**: `/home/beengud/raibid-labs/scarab/crates/scarab-plugin-api/src/context.rs`

- Lines 164-183: `log()` method implementation
- Lines 185-216: `notify()` and convenience methods

### Plugin Manager (scarab-daemon)

**Location**: `/home/beengud/raibid-labs/scarab/crates/scarab-daemon/src/plugin_manager/mod.rs`

- Lines 175-260: `process_pending_commands()` with log/notify handlers
- Lines 178-196: Level conversion functions

### Client UI (scarab-client)

**Location**: `/home/beengud/raibid-labs/scarab/crates/scarab-client/src/ui/overlays.rs`

- Lines 107-121: Plugin log handling
- Lines 122-124: Plugin notification handling
- Lines 130-218: Notification UI rendering
- Lines 220-249: Auto-dismiss and stacking logic

## Best Practices

### When to Use Logging

- **Debug**: Detailed information for troubleshooting (line-by-line processing, state transitions)
- **Info**: Normal operational messages (plugin loaded, configuration applied)
- **Warn**: Recoverable issues (deprecated config, fallback to defaults)
- **Error**: Critical failures (initialization failed, resource unavailable)

### When to Use Notifications

- **Success**: User actions completed successfully (build finished, tests passed)
- **Info**: Important status updates (long operation started/finished)
- **Warning**: Issues requiring attention but not blocking (outdated dependencies, performance degradation)
- **Error**: Critical failures requiring immediate action (build failed, service unreachable)

### Performance Considerations

1. **Avoid notification spam**: Don't send notifications for every event
2. **Use logging for debugging**: Prefer logs over notifications for verbose output
3. **Batch related events**: Accumulate events and send a summary notification
4. **Respect user attention**: Notifications should be actionable, not just informative

### Example: Rate Limiting

```fsharp
let on_output (data: string) (ctx: PluginContext) : Async<Result<Action, string>> =
    async {
        // Always log (filtered by log level configuration)
        ctx.Log(LogLevel.Debug, sprintf "Processing: %s" data)

        // Only notify every 100 errors to avoid spam
        if data.Contains("error") then
            let errorCount = ctx.GetData("error_count") |> Option.defaultValue "0" |> int
            ctx.SetData("error_count", string (errorCount + 1))

            if errorCount % 100 = 0 then
                ctx.NotifyWarning("Errors Detected", sprintf "%d errors in output" errorCount)

        return Ok Action.Continue
    }
```

## Testing Your Plugin

Use the example plugin at `examples/plugins/logging-demo.fsx` to test the system:

1. **Load the plugin**:
   ```bash
   cargo run -p scarab-daemon
   # In another terminal:
   cargo run -p scarab-client
   ```

2. **Trigger test commands** via command palette (Ctrl+P):
   - "Test All Notifications" - See all notification levels
   - "Test All Log Levels" - Send logs at all levels
   - "Show Plugin Stats" - Display plugin statistics

3. **Monitor output**:
   - Daemon logs: Check terminal running scarab-daemon
   - Client logs: Check terminal running scarab-client
   - Notifications: Watch top-right corner of client window

## Troubleshooting

### Notifications Not Appearing

1. Check that client is connected to daemon
2. Verify plugin is loaded and enabled
3. Check client console for errors
4. Ensure notification level is not filtered

### Logs Not Visible

1. Check log level configuration (RUST_LOG environment variable)
2. Verify plugin is calling ctx.log() correctly
3. Check daemon console output
4. Enable debug logging: `RUST_LOG=debug cargo run -p scarab-daemon`

### Notification UI Issues

1. Check for JavaScript/UI errors in client console
2. Verify Bevy rendering is working correctly
3. Test with example plugin first to isolate issues
4. Check notification stacking (multiple notifications should stack vertically)

## Future Enhancements

Planned improvements for the notification system:

- [ ] Configurable auto-dismiss duration
- [ ] User-dismissible notifications (click to close)
- [ ] Notification history panel
- [ ] Sound/haptic feedback for notifications
- [ ] Priority levels for notification ordering
- [ ] Notification grouping by plugin
- [ ] Desktop system notifications (OS integration)
- [ ] Notification filtering/muting per plugin

## See Also

- [Plugin API Documentation](../crates/scarab-plugin-api/README.md)
- [Fusabi Language Guide](./FUSABI_GUIDE.md)
- [Example Plugins](../examples/plugins/)
- [Plugin Development Guide](../examples/plugin-template/README.md)
