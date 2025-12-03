# Managing Plugins

Manage your installed Scarab plugins.

## Quick Links

For complete plugin documentation, see:
- [Plugin Development Guide](../../../plugin-development/README.md) - Complete plugin guide

## Listing Plugins

### Via Command Palette

1. Press **Ctrl+Shift+P**
2. Type `plugins: list`
3. View all installed plugins

### Via Configuration File

Check `~/.config/scarab/config.toml`:

```toml
[plugins]
enabled = ["scarab-nav", "scarab-palette", "my-plugin"]
```

## Enabling/Disabling Plugins

### Enable a Plugin

Add to the `enabled` array:

```toml
[plugins]
enabled = ["scarab-nav", "scarab-palette", "new-plugin"]
```

### Disable a Plugin

Remove from `enabled` or add to `disabled`:

```toml
[plugins]
enabled = ["scarab-nav", "scarab-palette"]
disabled = ["problematic-plugin"]
```

Changes apply on next startup (daemon) or immediately (client).

## Updating Plugins

### Manual Update

1. Download new plugin version
2. Replace old file in `~/.config/scarab/plugins/`
3. Restart daemon (for backend plugins)

```bash
# Update plugin
cp git-status-v2.fsx ~/.config/scarab/plugins/git-status.fsx

# Restart daemon
cargo run --release -p scarab-daemon
```

### Auto-Update (Future Feature)

Plugin auto-update is planned for a future release.

## Uninstalling Plugins

### Step 1: Disable Plugin

Remove from `enabled` array:

```toml
[plugins]
enabled = ["scarab-nav", "scarab-palette"]  # removed unwanted-plugin
```

### Step 2: Delete Plugin File

```bash
rm ~/.config/scarab/plugins/unwanted-plugin.fsx
```

## Plugin Configuration

### View Plugin Settings

Check configuration section:

```toml
[plugins.config.git-status]
show_branch = true
show_dirty = true
position = "top-right"
```

### Modify Settings

Edit values and save:

```toml
[plugins.config.git-status]
show_branch = false  # Hide branch name
show_dirty = true
position = "top-left"  # Change position
```

Changes apply immediately with hot-reload.

## Plugin Dependencies

Some plugins may have dependencies on other plugins:

```toml
[plugins.dependencies.advanced-nav]
requires = ["scarab-nav"]
```

Scarab will:
- Load dependencies first
- Warn if dependencies are missing
- Refuse to load plugin without dependencies

## Plugin Priority

Control plugin load order:

```toml
[plugins]
enabled = ["scarab-nav", "scarab-palette", "my-plugin"]

[plugins.priority]
scarab-nav = 100      # Load first (higher priority)
my-plugin = 50        # Load after scarab-nav
scarab-palette = 10   # Load last
```

## Troubleshooting

### Plugin Not Loading

1. Check plugin is in `enabled` array
2. Verify file exists in `~/.config/scarab/plugins/`
3. Check logs for errors:
   ```bash
   RUST_LOG=debug cargo run -p scarab-daemon
   ```

### Plugin Conflicts

If plugins conflict:

1. Disable one plugin
2. Check configuration for conflicts
3. Report issue to plugin author

### Performance Issues

If a plugin causes performance issues:

1. Disable the plugin temporarily
2. Check plugin configuration
3. Monitor system resources
4. Report to plugin author

## Plugin Development

Want to create your own plugin? See:

- [Plugin Development Guide](../developer-guide/plugins.md)
- [Plugin API Reference](../developer-guide/plugin-api.md)
- [Example Plugins](../developer-guide/plugin-examples.md)

## See Also

- [Installing Plugins](./installing-plugins.md) - Plugin installation
- [Plugins Overview](./plugins.md) - Plugin system overview
- [Plugin Development](../developer-guide/plugins.md) - Create plugins
