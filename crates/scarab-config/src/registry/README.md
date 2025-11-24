# Scarab Registry Module

Internal module implementing the plugin marketplace and registry system.

## Module Structure

```
registry/
├── mod.rs          - Public API and RegistryManager
├── types.rs        - Core data types (PluginEntry, InstalledPlugin, etc.)
├── manifest.rs     - Registry manifest handling
├── cache.rs        - Local cache management
├── client.rs       - HTTP client for remote registry API
├── installer.rs    - Plugin installation and removal
└── security.rs     - Checksum and signature verification
```

## Architecture

```
┌─────────────────────────────────────────────────────────┐
│                    RegistryManager                      │
│  (Main facade combining all registry functionality)    │
└────────┬──────────────┬──────────────┬─────────────┬────┘
         │              │              │             │
    ┌────▼────┐   ┌─────▼─────┐  ┌────▼────┐  ┌────▼────┐
    │  Cache  │   │  Client   │  │Installer│  │Verifier │
    │ (Local) │   │ (Remote)  │  │(Install)│  │(Security)│
    └─────────┘   └───────────┘  └─────────┘  └─────────┘
         │              │              │             │
         ▼              ▼              ▼             ▼
    manifest.json   HTTP API     plugins/     SHA256+GPG
```

## Usage Example

```rust
use scarab_config::registry::*;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize registry manager
    let mut manager = RegistryManager::new()?;

    // Sync with remote registry (downloads manifest)
    manager.sync().await?;

    // Search for plugins
    let filter = PluginFilter {
        query: Some("notification".to_string()),
        min_rating: Some(4.0),
        sort: SortOrder::Popular,
        limit: Some(10),
        ..Default::default()
    };
    let results = manager.search(&filter)?;

    // Install a plugin
    let installed = manager.install("auto-notify", None).await?;
    println!("Installed {} v{}", installed.name, installed.version);

    // Check for updates
    let updates = manager.check_updates()?;
    for (name, current, latest) in updates {
        println!("{}: {} -> {}", name, current, latest);
    }

    // Update a plugin
    manager.update("auto-notify").await?;

    // List installed plugins
    let installed = manager.list_installed()?;
    for plugin in installed {
        println!("{} v{} [{}]",
            plugin.name,
            plugin.version,
            if plugin.enabled { "enabled" } else { "disabled" }
        );
    }

    Ok(())
}
```

## Components

### RegistryManager

Main entry point combining all functionality. Provides high-level operations:
- `sync()` - Synchronize with remote registry
- `search()` - Search plugins with filters
- `install()` - Install plugin from registry
- `update()` - Update plugin to latest version
- `remove()` - Remove installed plugin
- `list_installed()` - List all installed plugins
- `check_updates()` - Check for available updates

### RegistryCache

Manages local cache of registry manifest:
- Stores manifest at `~/.config/scarab/registry/manifest.json`
- Provides fast local search without network requests
- Tracks cache age and staleness
- Supports cache invalidation

### RegistryClient

HTTP client for remote registry API:
- Fetches manifest from `/v1/manifest.json`
- Downloads plugins from `/v1/plugins/{name}/{version}/download`
- Includes checksums and signatures in response headers
- Configurable timeout and retry logic

### PluginInstaller

Manages plugin installation lifecycle:
- Installs plugins to `~/.config/scarab/plugins/{name}/`
- Maintains installation index at `plugins/installed.json`
- Supports enable/disable without removal
- Handles plugin-specific configuration

### PluginVerifier

Security verification:
- SHA256 checksum verification (mandatory)
- GPG signature verification (optional)
- Plugin format validation (.fzb vs .fsx)
- Configurable security policies

## Data Types

### PluginEntry
Complete plugin metadata from registry:
- Name, description, author
- Version history
- Download URLs and checksums
- Statistics (downloads, ratings)
- Tags and categories

### InstalledPlugin
Locally installed plugin metadata:
- Installation path and timestamp
- Enabled/disabled status
- Plugin-specific configuration
- Version information

### PluginFilter
Search filter criteria:
- Query string (name/description/author)
- Tag filtering
- Minimum rating
- Sort order (Popular, Rating, Recent, Name)
- Result limit

## Security Model

1. **Checksum Verification**:
   - All downloads verified against SHA256 checksums
   - Prevents corrupted or tampered downloads
   - Configurable via `security.require_checksum`

2. **GPG Signatures** (Optional):
   - Detached signatures for plugin files
   - Trusted key management
   - Configurable via `security.require_signature`

3. **Format Validation**:
   - Validates file format (.fzb bytecode or .fsx source)
   - Checks for valid Fusabi magic numbers

## Configuration

Registry behavior configured via `RegistryConfig`:

```rust
let config = RegistryConfig {
    registry_url: "https://registry.scarab.dev".to_string(),
    cache_dir: PathBuf::from("~/.config/scarab/registry"),
    plugin_dir: PathBuf::from("~/.config/scarab/plugins"),
    security: SecurityConfig {
        require_checksum: true,
        require_signature: false,
        trusted_keys: vec![],
        allow_unsigned: true,
    },
};

let manager = RegistryManager::with_config(config)?;
```

## Error Handling

All operations return `Result<T, ConfigError>`:

- `ConfigError::NotFound` - Plugin not in registry or not installed
- `ConfigError::SecurityError` - Checksum/signature verification failed
- `ConfigError::NetworkError` - HTTP request failed
- `ConfigError::ValidationError` - Invalid manifest or plugin format
- `ConfigError::IoError` - File system operation failed

## Testing

Run module tests:

```bash
cargo test -p scarab-config --lib registry
```

Key test scenarios:
- Cache persistence and reload
- Search filtering (query, tags, rating)
- Install/remove lifecycle
- Checksum verification
- Format validation

## Performance Considerations

1. **Cache-First**: Always search local cache before network requests
2. **Lazy Sync**: Only sync if cache is stale (>24 hours)
3. **Parallel Downloads**: Use async for concurrent operations
4. **Index Optimization**: In-memory HashMap for fast lookups

## Future Improvements

- [ ] Batch operations (install/update multiple plugins)
- [ ] Download resume for large plugins
- [ ] Dependency resolution
- [ ] Plugin conflict detection
- [ ] Automatic update scheduling
- [ ] Mirror/fallback registries
- [ ] Bandwidth throttling
- [ ] Plugin analytics opt-in
