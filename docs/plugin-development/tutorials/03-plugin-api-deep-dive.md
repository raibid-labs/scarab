# Tutorial 3: Plugin API Deep Dive

This tutorial provides a comprehensive overview of the Scarab Plugin API, covering all major concepts and features.

## Table of Contents

1. [Plugin Architecture](#plugin-architecture)
2. [The Plugin Trait](#the-plugin-trait)
3. [PluginContext](#plugincontext)
4. [Hooks Lifecycle](#hooks-lifecycle)
5. [Async Programming](#async-programming)
6. [Error Handling](#error-handling)
7. [Configuration](#configuration)
8. [Best Practices](#best-practices)

## Plugin Architecture

Scarab plugins follow a trait-based architecture:

```
Plugin Trait (interface)
    ↓
Your Plugin Implementation
    ↓
Fusabi Runtime (VM or Interpreter)
    ↓
Scarab Core (daemon or client)
```

## The Plugin Trait

Every plugin must implement the `Plugin` trait with metadata:

```fsharp
[<Plugin>]
let metadata = {
    Name = "my-plugin"
    Version = "0.1.0"
    Description = "What it does"
    Author = "Your Name"
    Homepage = Some "https://..."
    ApiVersion = "0.1.0"
    MinScarabVersion = "0.1.0"
    Emoji = Some "🚀"
    Color = Some "#4CAF50"
    Catchphrase = Some "Make it awesome!"
}
```

See **[API Reference: Plugin Context](../api-reference/plugin-context.md)** for complete details.

## PluginContext

The context provides access to:

- **Terminal State** - Grid, cursor, size
- **Environment** - Variables, config
- **Storage** - Per-plugin data
- **Logging** - Debug, info, warn, error
- **Notifications** - User alerts
- **Commands** - RemoteUI interactions

See **[API Reference: Plugin Context](../api-reference/plugin-context.md)** for all methods.

## Hooks Lifecycle

```
OnLoad
  ↓
[OnOutput/OnInput/OnPreCommand/etc - repeated]
  ↓
OnUnload
```

See **[API Reference: Hooks](../api-reference/hooks.md)** for complete hook documentation.

## Async Programming

All hooks are async:

```fsharp
[<OnLoad>]
let onLoad (ctx: PluginContext) =
    async {
        // Async operations
        let! data = fetchData()
        ctx.Log Info "Data loaded"
        return Ok ()
    }
```

## Error Handling

Use Result types:

```fsharp
[<OnLoad>]
let onLoad (ctx: PluginContext) =
    async {
        try
            let! result = riskyOperation()
            return Ok ()
        with
        | ex ->
            ctx.Log Error ex.Message
            return Error (PluginError.InitializationError ex.Message)
    }
```

## Configuration

Access config from `plugin.toml`:

```fsharp
// Get required value
let threshold: int = ctx.Config.Get "threshold"

// Get optional value
let apiKey: string option = ctx.Config.GetOpt "api_key"
```

## Best Practices

1. **Performance** - Keep hooks fast (< 16ms for frontend, < 50μs for backend output)
2. **Async** - Use async properly, don't block
3. **Errors** - Handle gracefully, don't crash
4. **Logging** - Use appropriate levels
5. **Testing** - Write tests for your plugins

## Next Steps

→ **[Tutorial 4: Real Plugin (URL Shortener)](04-real-plugin-url-shortener.md)**

→ **[Frontend UI with RemoteUI](05-frontend-ui-remoteui.md)**

→ **[Backend Processing with Hooks](06-backend-hooks.md)**
