# Scarab Plugin Registry System

The Scarab Plugin Registry provides a centralized marketplace for discovering, installing, and managing community plugins.

## Architecture Overview

The registry system consists of four main components:

1. **Remote Registry API** - Central server hosting plugin metadata and downloads
2. **Local Cache** - On-disk cache of registry manifest (`~/.config/scarab/registry/`)
3. **Plugin Installer** - Manages installation, updates, and removal
4. **Security Verifier** - Validates checksums and signatures

```
┌─────────────────────┐
│  Remote Registry    │
│  (registry.scarab)  │
└──────────┬──────────┘
           │ HTTPS
           ▼
┌─────────────────────┐
│   RegistryClient    │
│  (HTTP Client)      │
└──────────┬──────────┘
           │
           ▼
┌─────────────────────┐      ┌──────────────────┐
│  RegistryCache      │◄─────┤ RegistryManifest │
│  (Local Cache)      │      │  (manifest.json) │
└──────────┬──────────┘      └──────────────────┘
           │
           ▼
┌─────────────────────┐      ┌──────────────────┐
│  PluginInstaller    │◄─────┤ PluginVerifier   │
│  (Install/Remove)   │      │  (SHA256 + GPG)  │
└─────────────────────┘      └──────────────────┘
           │
           ▼
    ~/.config/scarab/plugins/
```

## CLI Usage

### Search for Plugins

```bash
# Search by name or description
scarab-plugin search notification

# Search results show:
# - Plugin name and version
# - Description
# - Author, downloads, and rating
# - Tags
```

### Install a Plugin

```bash
# Install latest version
scarab-plugin install auto-notify

# Install specific version
scarab-plugin install auto-notify 1.2.0
```

Installation process:
1. Fetch plugin metadata from registry
2. Download plugin file (.fzb or .fsx)
3. Verify SHA256 checksum
4. Verify GPG signature (if configured)
5. Install to `~/.config/scarab/plugins/<name>/`
6. Update installed plugins index

### Update Plugins

```bash
# Check for updates
scarab-plugin check-updates

# Update specific plugin
scarab-plugin update auto-notify

# Update all (future feature)
# scarab-plugin update --all
```

### Remove Plugins

```bash
scarab-plugin remove auto-notify
```

### List Installed Plugins

```bash
scarab-plugin list

# Output:
# Installed plugins (3):
#
#   auto-notify v1.2.0 [enabled]
#     Path: ~/.config/scarab/plugins/auto-notify/auto-notify.fzb
#
#   git-status v2.1.0 [disabled]
#     Path: ~/.config/scarab/plugins/git-status/git-status.fsx
```

### Plugin Information

```bash
scarab-plugin info auto-notify

# Shows:
# - Full description
# - Author and license
# - Homepage and repository
# - Statistics (downloads, rating)
# - Available versions
# - Installation status
```

### Sync Registry

```bash
# Manually sync with remote registry
scarab-plugin sync

# Cache is automatically synced if older than 24 hours
```

### Enable/Disable Plugins

```bash
# Disable a plugin (keeps it installed)
scarab-plugin disable auto-notify

# Re-enable a plugin
scarab-plugin enable auto-notify
```

## Registry JSON Schema

### Registry Manifest (`manifest.json`)

```json
{
  "version": "1.0.0",
  "metadata": {
    "name": "Scarab Official Registry",
    "description": "Official plugin registry for Scarab terminal emulator",
    "url": "https://registry.scarab.dev",
    "maintainer": "registry@scarab.dev"
  },
  "plugins": {
    "auto-notify": {
      "name": "auto-notify",
      "description": "Automatically send desktop notifications when long-running commands complete",
      "readme": "# Auto Notify\n\nThis plugin monitors command execution...",
      "author": "Jane Developer",
      "author_email": "jane@example.com",
      "homepage": "https://github.com/jane/scarab-auto-notify",
      "repository": "https://github.com/jane/scarab-auto-notify",
      "license": "MIT",
      "latest_version": "1.2.0",
      "versions": [
        {
          "version": "1.2.0",
          "download_url": "https://registry.scarab.dev/plugins/auto-notify/1.2.0/auto-notify.fzb",
          "checksum": "a7b3c5d9e1f2g4h6i8j0k2l4m6n8o0p2q4r6s8t0u2v4w6x8y0z2a4b6c8d0e2f4",
          "signature": "-----BEGIN PGP SIGNATURE-----\n...\n-----END PGP SIGNATURE-----",
          "changelog": "## v1.2.0\n- Added support for custom notification icons\n- Fixed bug with multi-line commands",
          "api_version": "0.1.0",
          "min_scarab_version": "0.1.0",
          "size": 15360,
          "released_at": 1700000000,
          "prerelease": false
        }
      ],
      "tags": ["notification", "productivity", "automation"],
      "stats": {
        "downloads": 5420,
        "downloads_recent": 342,
        "rating": 4.7,
        "rating_count": 23,
        "stars": 156
      },
      "created_at": 1680000000,
      "updated_at": 1700000000
    }
  },
  "updated_at": 1700000000
}
```

### Installed Plugins Index (`~/.config/scarab/registry/installed.json`)

```json
{
  "version": "1.0.0",
  "plugins": {
    "auto-notify": {
      "name": "auto-notify",
      "version": "1.2.0",
      "path": "/home/user/.config/scarab/plugins/auto-notify/auto-notify.fzb",
      "installed_at": 1700000000,
      "enabled": true,
      "config": {
        "threshold_seconds": 30,
        "notification_style": "urgent",
        "keywords": ["ERROR", "FAIL", "PANIC"]
      }
    }
  }
}
```

## Security Features

### SHA256 Checksum Verification

All plugin downloads are verified against SHA256 checksums:

```rust
// Automatic verification during installation
let verifier = PluginVerifier::new(security_config);
verifier.verify(&plugin_content, &plugin_entry, "1.2.0")?;
```

Configuration in `~/.config/scarab/registry.toml`:

```toml
[security]
require_checksum = true  # Mandatory checksum verification
require_signature = false  # Optional GPG signature verification
allow_unsigned = true  # Allow unsigned plugins (for development)
trusted_keys = []  # List of trusted GPG key IDs
```

### GPG Signature Verification

For enhanced security, plugins can be signed with GPG:

```toml
[security]
require_signature = true
allow_unsigned = false
trusted_keys = [
    "A7B3C5D9E1F2G4H6",  # Official Scarab registry key
    "I8J0K2L4M6N8O0P2"   # Community maintainer key
]
```

**Note**: GPG verification is currently a placeholder. Full implementation requires:
- Integration with `sequoia-pgp` or `gpgme`
- Key management and distribution system
- Key expiration and revocation checking

### Sandboxing Recommendations

While Scarab's plugin system uses Fusabi VM which provides some isolation, additional security measures are recommended:

1. **Review Plugin Source**: Always review plugins before installation, especially for sensitive operations
2. **Principle of Least Privilege**: Only grant necessary permissions to plugins
3. **Network Isolation**: Consider using firewall rules for plugins that don't need network access
4. **Resource Limits**: Configure CPU and memory limits for plugin execution
5. **Audit Logging**: Enable audit logging to track plugin behavior

Example plugin configuration with resource limits:

```toml
[[plugin]]
name = "auto-notify"
path = "~/.config/scarab/plugins/auto-notify/auto-notify.fzb"
enabled = true

[plugin.config]
# Plugin-specific settings
threshold_seconds = 30

[plugin.limits]
max_memory_mb = 100
max_cpu_percent = 5.0
network_access = false
filesystem_access = "read-only"  # read-only, read-write, none
```

## Registry Configuration

Create `~/.config/scarab/registry.toml`:

```toml
# Registry server URL
registry_url = "https://registry.scarab.dev"

# Local cache directory (default: ~/.config/scarab/registry)
cache_dir = "~/.config/scarab/registry"

# Plugin installation directory (default: ~/.config/scarab/plugins)
plugin_dir = "~/.config/scarab/plugins"

[security]
require_checksum = true
require_signature = false
allow_unsigned = true
trusted_keys = []
```

## API Endpoints

The registry server exposes the following endpoints:

### GET /v1/manifest.json
Returns the complete registry manifest with all available plugins.

**Response**: 200 OK
```json
{
  "version": "1.0.0",
  "metadata": {...},
  "plugins": {...}
}
```

### GET /v1/plugins/{name}/{version}/download
Downloads a specific plugin version.

**Headers**:
- `X-Plugin-Checksum`: SHA256 checksum
- `X-Plugin-Signature`: GPG signature (optional)

**Response**: 200 OK (binary content)

### GET /health
Health check endpoint.

**Response**: 200 OK

## Publishing Plugins

To publish your plugin to the registry:

1. **Prepare Plugin Package**:
   ```bash
   # Compile Fusabi plugin
   fusabi compile my-plugin.fsx -o my-plugin.fzb

   # Generate checksum
   sha256sum my-plugin.fzb > my-plugin.sha256

   # (Optional) Sign with GPG
   gpg --detach-sign --armor my-plugin.fzb
   ```

2. **Create Plugin Metadata** (`plugin.json`):
   ```json
   {
     "name": "my-awesome-plugin",
     "description": "Does awesome things in your terminal",
     "author": "Your Name",
     "author_email": "you@example.com",
     "homepage": "https://github.com/you/my-awesome-plugin",
     "repository": "https://github.com/you/my-awesome-plugin",
     "license": "MIT",
     "version": "1.0.0",
     "api_version": "0.1.0",
     "min_scarab_version": "0.1.0",
     "tags": ["productivity", "git", "automation"]
   }
   ```

3. **Submit to Registry**:
   ```bash
   # Using registry CLI (future feature)
   scarab-registry publish \
     --plugin my-plugin.fzb \
     --metadata plugin.json \
     --checksum my-plugin.sha256 \
     --signature my-plugin.fzb.asc
   ```

   Or submit via GitHub Pull Request to the registry repository.

## Example: Creating a Custom Registry

For private/enterprise deployments, you can host your own registry:

1. **Create Registry Server**:
   ```rust
   // Simple HTTP server serving manifest.json
   use axum::{Router, routing::get};

   #[tokio::main]
   async fn main() {
       let app = Router::new()
           .route("/v1/manifest.json", get(serve_manifest))
           .route("/v1/plugins/:name/:version/download", get(serve_plugin))
           .route("/health", get(|| async { "OK" }));

       axum::Server::bind(&"0.0.0.0:8080".parse().unwrap())
           .serve(app.into_make_service())
           .await
           .unwrap();
   }
   ```

2. **Configure Client**:
   ```toml
   # ~/.config/scarab/registry.toml
   registry_url = "https://registry.mycompany.com"
   ```

3. **Populate Registry**:
   ```bash
   # Add plugins to manifest.json
   # Host files at the configured download URLs
   ```

## Troubleshooting

### Cache Issues

```bash
# Clear cache and re-sync
rm -rf ~/.config/scarab/registry/manifest.json
scarab-plugin sync
```

### Checksum Verification Failed

```bash
# This indicates corrupted download or tampered plugin
# DO NOT bypass security checks!
# Try re-installing or contact plugin author
```

### Network Errors

```bash
# Check registry connectivity
curl https://registry.scarab.dev/health

# Use alternative registry
scarab-plugin --registry https://mirror.scarab.dev search <query>
```

## Future Enhancements

- [ ] Batch operations (update all, install multiple)
- [ ] Plugin dependencies and conflict resolution
- [ ] Plugin ratings and reviews from CLI
- [ ] Automatic update checks on Scarab startup
- [ ] Plugin screenshots and demos
- [ ] Plugin development templates (`scarab-plugin new`)
- [ ] Integration with GitHub Releases for auto-publishing
- [ ] Plugin analytics (usage statistics)
- [ ] Paid/premium plugins support
- [ ] Plugin collections/bundles

## Contributing

To contribute to the registry system:

1. Review the [Plugin API Documentation](./plugin-api.md)
2. Submit plugins via PR to the registry repository
3. Report issues on GitHub
4. Suggest features in discussions

## Security

Report security vulnerabilities to: security@scarab.dev

Do not open public issues for security problems.
