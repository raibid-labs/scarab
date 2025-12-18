# Plugin Registry Quick Start Guide

Get started with the Scarab Plugin Registry in 5 minutes.

## Installation

The plugin registry is built into the `scarab-config` crate and will be available after building Scarab:

```bash
# Build the workspace (includes scarab-plugin CLI)
cargo build --release

# The scarab-plugin binary will be at:
# target/release/scarab-plugin
```

## First-Time Setup

1. **Initialize Plugin Directory**:
   ```bash
   mkdir -p ~/.config/scarab/plugins
   mkdir -p ~/.config/scarab/registry
   ```

2. **Sync with Registry**:
   ```bash
   scarab-plugin sync
   ```

   This downloads the registry manifest (list of all available plugins) to your local cache.

3. **Verify Setup**:
   ```bash
   scarab-plugin search notification
   ```

   You should see a list of notification-related plugins.

## Common Tasks

### Discover Plugins

Search by keyword:
```bash
scarab-plugin search git
scarab-plugin search productivity
scarab-plugin search theme
```

Get detailed information:
```bash
scarab-plugin info auto-notify
```

### Install Your First Plugin

```bash
# Install latest version
scarab-plugin install auto-notify

# Install specific version
scarab-plugin install auto-notify 1.2.0
```

The plugin will be installed to `~/.config/scarab/plugins/auto-notify/`

### Verify Installation

```bash
scarab-plugin list
```

Output:
```
Installed plugins (1):

  auto-notify v1.2.0 [enabled]
    Path: ~/.config/scarab/plugins/auto-notify/auto-notify.fzb
```

### Enable/Disable Plugins

```bash
# Disable a plugin temporarily
scarab-plugin disable auto-notify

# Re-enable it
scarab-plugin enable auto-notify
```

### Update Plugins

Check for updates:
```bash
scarab-plugin check-updates
```

Update a specific plugin:
```bash
scarab-plugin update auto-notify
```

### Remove Plugins

```bash
scarab-plugin remove auto-notify
```

## Configuration

Create `~/.config/scarab/registry.toml` to customize settings:

```toml
# Registry server URL (change for private registries)
registry_url = "https://registry.scarab.dev"

# Cache directory
cache_dir = "~/.config/scarab/registry"

# Plugin installation directory
plugin_dir = "~/.config/scarab/plugins"

[security]
# Require SHA256 checksum verification (recommended)
require_checksum = true

# Require GPG signatures (optional, more secure)
require_signature = false

# Allow unsigned plugins (disable for production)
allow_unsigned = true

# Trusted GPG key IDs (if using signatures)
trusted_keys = []
```

## Plugin Configuration

After installing a plugin, configure it in `~/.config/scarab/plugins.toml`:

```toml
[[plugin]]
name = "auto-notify"
path = "~/.config/scarab/plugins/auto-notify/auto-notify.fzb"
enabled = true

[plugin.config]
# Plugin-specific settings
threshold_seconds = 30  # Notify after 30 seconds
notification_style = "urgent"
keywords = ["ERROR", "FAIL", "PANIC"]
```

Restart Scarab for changes to take effect.

## Programmatic Usage

You can also use the registry API directly in Rust:

```rust
use scarab_config::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    // Create registry manager
    let mut manager = RegistryManager::new()?;

    // Sync with remote registry
    manager.sync().await?;

    // Search for plugins
    let filter = PluginFilter {
        query: Some("git".to_string()),
        min_rating: Some(4.0),
        ..Default::default()
    };
    let results = manager.search(&filter)?;

    for plugin in results {
        println!("{}: {}", plugin.name, plugin.description);
    }

    // Install a plugin
    let installed = manager.install("auto-notify", None).await?;
    println!("Installed: {} v{}", installed.name, installed.version);

    // List installed plugins
    let installed = manager.list_installed()?;
    println!("Installed {} plugins", installed.len());

    Ok(())
}
```

## Security Best Practices

1. **Always verify checksums**: Keep `require_checksum = true`
2. **Review before installing**: Check plugin source code if possible
3. **Use stable versions**: Avoid prereleases in production
4. **Keep plugins updated**: Run `check-updates` regularly
5. **Enable signatures for production**: Set `require_signature = true` and configure `trusted_keys`

## Troubleshooting

### "Registry cache is stale" message

This is normal. The cache auto-syncs if older than 24 hours. You can manually sync:
```bash
scarab-plugin sync
```

### "Plugin not found" error

The plugin may not be in the registry yet. Search to verify:
```bash
scarab-plugin search <name>
```

### Network errors

Check connectivity to the registry:
```bash
curl https://registry.scarab.dev/health
```

If the official registry is down, you can use a mirror (if configured).

### Checksum verification failed

**DO NOT bypass this check!** This indicates:
- Corrupted download
- Tampered plugin file
- Registry server issue

Try reinstalling. If the problem persists, report it as a security issue.

## Next Steps

- **Create your own plugin**: See [Plugin Development Guide](./plugin-development.md)
- **Publish to registry**: See [Publishing Plugins](./plugin-registry.md#publishing-plugins)
- **Host private registry**: See [Custom Registry Guide](./plugin-registry.md#example-creating-a-custom-registry)
- **Advanced configuration**: See [Full Registry Documentation](./plugin-registry.md)

## Getting Help

- **Documentation**: https://docs.scarab.dev
- **GitHub Issues**: https://github.com/raibid-labs/scarab/issues
- **Discord**: https://discord.gg/scarab
- **Email**: support@scarab.dev
