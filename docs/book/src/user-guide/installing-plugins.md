# Installing Plugins

Learn how to install and configure Scarab plugins.

## Quick Links

For complete plugin documentation, see:
- [Plugin Development Guide](../../../plugin-development/README.md) - Complete plugin guide

## Plugin Location

Plugins are stored in:
```
~/.config/scarab/plugins/
```

Create the directory if it doesn't exist:
```bash
mkdir -p ~/.config/scarab/plugins/
```

## Installing a Plugin

### Step 1: Download the Plugin

Copy the plugin file to the plugins directory:

```bash
# Frontend plugin (.fsx)
cp my-plugin.fsx ~/.config/scarab/plugins/

# Backend plugin (.fzb)
cp my-plugin.fzb ~/.config/scarab/plugins/
```

### Step 2: Enable the Plugin

Edit `~/.config/scarab/config.toml`:

```toml
[plugins]
enabled = ["scarab-nav", "scarab-palette", "my-plugin"]
```

### Step 3: Reload Configuration

Configuration hot-reloads automatically. For backend plugins, restart the daemon:

```bash
# Restart daemon to load backend plugins
cargo run --release -p scarab-daemon
```

## Plugin Types

### Frontend Plugins (.fsx)

Frontend plugins run in the client and can:
- Create UI overlays
- Add custom keybindings
- React to terminal events
- Hot-reload without restart

Install by copying `.fsx` file and enabling in config.

### Backend Plugins (.fzb)

Backend plugins run in the daemon and can:
- Filter terminal output
- Monitor processes
- Integrate with shell
- High performance (compiled bytecode)

Require daemon restart after installation.

## Configuration

Configure plugin settings:

```toml
[plugins.config.my-plugin]
option1 = "value"
option2 = 42
option3 = true
```

## Example: Installing Git Status Plugin

1. Create the plugin:

```bash
cat > ~/.config/scarab/plugins/git-status.fsx << 'EOF'
open Scarab.PluginApi

let metadata = {
    Name = "git-status"
    Version = "1.0.0"
    Description = "Display git status"
    Author = "You"
    Homepage = None
    ApiVersion = "0.1.0"
    MinScarabVersion = "0.1.0"
}

// ... plugin implementation
EOF
```

2. Enable in config:

```toml
[plugins]
enabled = ["git-status"]

[plugins.config.git-status]
show_branch = true
show_dirty = true
```

3. Plugin loads automatically!

## Troubleshooting

### Plugin Not Loading

Check Scarab logs:
```bash
RUST_LOG=debug cargo run -p scarab-daemon
```

Common issues:
- Plugin file not in `~/.config/scarab/plugins/`
- Plugin not listed in `enabled` array
- Syntax errors in plugin code
- Missing dependencies

### Plugin Errors

View plugin-specific logs in the daemon output:
```
[INFO] Loading plugin: my-plugin
[ERROR] Plugin 'my-plugin' failed: <error message>
```

## See Also

- [Managing Plugins](./managing-plugins.md) - Plugin management
- [Plugins Overview](./plugins.md) - Plugin system overview
- [Plugin Development](../developer-guide/plugins.md) - Create plugins
