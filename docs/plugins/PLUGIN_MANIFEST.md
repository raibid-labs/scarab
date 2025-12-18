# Plugin Manifest Schema

Scarab plugins must declare their capabilities, dependencies, and requirements through a plugin manifest. This ensures security, compatibility, and proper resource allocation.

## Overview

The plugin manifest is a TOML file that describes:
- Plugin metadata (name, version, author)
- API version compatibility
- Required capabilities (what the plugin can do)
- Required modules from fusabi-stdlib-ext
- Visual branding (emoji, color, catchphrase)

## Manifest Location

The manifest can be included in two ways:

1. **Standalone file**: `plugin-name.manifest.toml` alongside `plugin-name.fsx`
2. **Embedded**: As a TOML frontmatter in the `.fsx` file itself

## Schema

### Required Fields

```toml
name = "my-plugin"           # Unique plugin identifier
version = "1.0.0"            # Semantic version
description = "What it does" # Short description
author = "Your Name"         # Plugin author
api-version = "0.1.0"        # Compatible API version
min-scarab-version = "0.1.0" # Minimum Scarab version
```

### Optional Fields

```toml
homepage = "https://..."     # Plugin homepage/repository
emoji = "🚀"                 # Visual icon (Unicode emoji)
color = "#FF5733"            # Theme color (hex code)
catchphrase = "Awesome!"     # Plugin motto/tagline
```

### Capabilities

Plugins must declare what they intend to do. Available capabilities:

```toml
capabilities = [
    "output-filtering",      # Intercept/modify terminal output
    "input-filtering",       # Intercept/modify user input
    "shell-execution",       # Execute shell commands
    "file-system",           # Read/write files
    "network",               # Make network requests
    "clipboard",             # Access clipboard
    "process-spawn",         # Spawn child processes
    "terminal-control",      # Modify terminal state
    "ui-overlay",            # Draw UI overlays
    "menu-registration",     # Register menu items
    "command-registration",  # Register command palette commands
]
```

### Required Modules

Declare which fusabi-stdlib-ext modules the plugin needs:

```toml
required-modules = [
    "terminal",   # Terminal I/O operations
    "gpu",        # GPU rendering utilities
    "fs",         # File system operations
    "net",        # Network operations
    "process",    # Process management
    "text",       # Text processing utilities
    "config",     # JSON/TOML parsing
]
```

## Validation

The manifest is validated at plugin load time:

1. **API Version Check**: Plugin's `api-version` must match major version and not exceed minor version
2. **Capability Check**: All requested capabilities must be supported
3. **Module Availability**: All required modules must be available in fusabi-stdlib-ext

## Example Manifest

```toml
name = "git-status"
version = "1.2.0"
description = "Display git repository status in terminal"
author = "Scarab Team"
homepage = "https://github.com/raibid-labs/scarab"
api-version = "0.1.0"
min-scarab-version = "0.1.0"

emoji = "🌳"
color = "#F05032"
catchphrase = "Git good or git going!"

capabilities = [
    "shell-execution",
    "ui-overlay",
    "terminal-control",
]

required-modules = [
    "terminal",
    "process",
    "text",
]
```

## Using in Plugins

### Rust Plugin

```rust
use scarab_plugin_api::{PluginManifest, Capability, FusabiModule};
use std::collections::HashSet;

fn create_manifest() -> PluginManifest {
    let mut capabilities = HashSet::new();
    capabilities.insert(Capability::OutputFiltering);
    capabilities.insert(Capability::UiOverlay);

    let mut modules = HashSet::new();
    modules.insert(FusabiModule::Terminal);
    modules.insert(FusabiModule::Text);

    PluginManifest {
        name: "my-plugin".to_string(),
        version: "1.0.0".to_string(),
        description: "My awesome plugin".to_string(),
        author: "Me".to_string(),
        homepage: Some("https://example.com".to_string()),
        api_version: "0.1.0".to_string(),
        min_scarab_version: "0.1.0".to_string(),
        capabilities,
        required_modules: modules,
        emoji: Some("🔌".to_string()),
        color: Some("#FF5733".to_string()),
        catchphrase: Some("Plugin power!".to_string()),
    }
}
```

### Fusabi Plugin

For Fusabi (F#) plugins, include the manifest as a separate `.toml` file or as TOML frontmatter:

```fsharp
(*
[manifest]
name = "my-plugin"
version = "1.0.0"
description = "My awesome plugin"
author = "Me"
api-version = "0.1.0"
min-scarab-version = "0.1.0"
capabilities = ["output-filtering", "ui-overlay"]
required-modules = ["terminal", "text"]
*)

open Scarab.PluginApi

// Plugin code here...
```

## Migration Guide

Existing plugins should add a manifest file. The plugin loader will:

1. Look for `plugin-name.manifest.toml`
2. If not found, look for TOML frontmatter in `.fsx`
3. If not found, use default capabilities (minimal permissions)

For backward compatibility, plugins without manifests get:
- No special capabilities (safe mode)
- Only basic terminal I/O module access
- Warning logged during load

## Security Model

The manifest enforces a **capabilities-based security model**:

- Plugins can only use declared capabilities
- Runtime checks ensure plugins don't exceed permissions
- Users can review manifests before installing plugins
- Scarab can sandbox plugins based on capabilities

## Best Practices

1. **Principle of Least Privilege**: Only request capabilities you actually need
2. **Accurate Versioning**: Use semantic versioning correctly
3. **Clear Descriptions**: Help users understand what your plugin does
4. **Module Minimalism**: Only require modules you use
5. **Visual Identity**: Add emoji/color for better UX

## Related Documentation

- [Plugin Development Guide](../examples/plugins/README.md)
- [Fusabi Integration](./FUSABI_CONFIG.md)
- [Plugin API Reference](../crates/scarab-plugin-api/README.md)
