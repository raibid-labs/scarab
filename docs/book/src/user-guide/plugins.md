# Plugins

Extend Scarab with plugins written in Fusabi (F#).

## Quick Links

For complete plugin documentation, see:
- [Plugin Development Guide](../../../plugin-development/README.md) - Complete plugin guide
- [Plugin API Reference](../../../plugin-api.md) - API documentation
- [Example Plugins](../../../plugin-development/tutorials/) - Plugin tutorials

## Overview

Scarab's plugin system allows you to extend functionality using Fusabi, an F# dialect for Rust.

## Plugin Types

### Frontend Plugins (.fsx)
- Run in client GUI process
- Hot-reloadable (no Rust recompilation)
- UI overlays, custom keybindings
- Interpreted scripts

### Backend Plugins (.fzb)
- Run in daemon process
- Pre-compiled bytecode
- Output filtering, process monitoring
- High performance

## Built-in Plugins

Scarab includes several built-in plugins:

- **scarab-nav** - Link hints and keyboard navigation
- **scarab-palette** - Command palette
- **scarab-session** - Session management

## Installing Plugins

### From Plugin Directory

Place plugins in `~/.config/scarab/plugins/`:

```bash
# Frontend plugin
~/.config/scarab/plugins/my-plugin.fsx

# Backend plugin (compiled)
~/.config/scarab/plugins/my-plugin.fzb
```

### Enable in Configuration

Edit `~/.config/scarab/config.toml`:

```toml
[plugins]
enabled = ["scarab-nav", "scarab-palette", "my-plugin"]
```

## Managing Plugins

### Listing Plugins

Via command palette:
- Press **Ctrl+Shift+P**
- Type `plugins: list`

### Enabling/Disabling

```toml
[plugins]
enabled = ["scarab-nav", "scarab-palette"]
disabled = ["problematic-plugin"]
```

### Plugin Configuration

Configure plugin settings:

```toml
[plugins.config.git-status]
show_branch = true
show_dirty = true
position = "top-right"

[plugins.config.notification-monitor]
min_duration = 5  # seconds
show_desktop_notification = true
```

## Creating Your First Plugin

See [Installing Plugins](./installing-plugins.md) for step-by-step instructions.

Quick example:

```fsharp
// ~/.config/scarab/plugins/hello.fsx
open Scarab.PluginApi

let metadata = {
    Name = "hello-plugin"
    Version = "1.0.0"
    Description = "My first Scarab plugin"
    Author = "Your Name"
    Homepage = None
    ApiVersion = "0.1.0"
    MinScarabVersion = "0.1.0"
}

let on_load (ctx: PluginContext) =
    async {
        ctx.Log(LogLevel.Info, "Hello from my plugin!")
        return Ok ()
    }

Plugin.Register {
    Metadata = metadata
    OnLoad = on_load
    // ... other hooks
}
```

## Plugin Examples

Browse example plugins in the repository:
- `examples/plugins/hello-plugin.fsx` - Minimal example
- `examples/plugins/git-status.fsx` - Git integration
- `examples/plugins/notification-monitor.fsx` - Desktop notifications
- `examples/plugins/custom-keybind.fsx` - Custom shortcuts

## See Also

- [Installing Plugins](./installing-plugins.md) - Plugin installation
- [Managing Plugins](./managing-plugins.md) - Plugin management
- [Plugin Development](../developer-guide/plugins.md) - Create plugins
- [Plugin API](../developer-guide/plugin-api.md) - API reference
