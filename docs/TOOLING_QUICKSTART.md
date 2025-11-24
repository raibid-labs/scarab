# Plugin Development Tooling - Quick Start Guide

Get started with Fusabi plugin development in 5 minutes.

## Prerequisites

- Rust toolchain installed
- Just command runner: `cargo install just`
- cargo-watch (optional): `cargo install cargo-watch`

## Quick Commands

### Create New Plugin

```bash
just plugin-new my-plugin
```

This creates `examples/fusabi/my-plugin.fsx` with:
- Pre-filled metadata template
- All lifecycle hooks
- Best practices structure

### Build Plugin

```bash
# Build single plugin
just plugin-build examples/fusabi/my-plugin.fsx

# Build all plugins
just plugin-build-all
```

### Validate Plugin

```bash
# Validate single plugin
just plugin-validate examples/fusabi/my-plugin.fsx

# Validate all plugins
just plugin-validate-all
```

### Development Workflow

**Option 1: Watch Mode (Recommended)**

```bash
# Terminal 1: Auto-rebuild on changes
just plugin-watch

# Terminal 2: Run daemon
cargo run -p scarab-daemon

# Terminal 3: Run client
cargo run -p scarab-client
```

**Option 2: Manual Build**

```bash
# Edit plugin
vim examples/fusabi/my-plugin.fsx

# Build and validate
just plugin-build examples/fusabi/my-plugin.fsx
just plugin-validate examples/fusabi/my-plugin.fsx

# Test in Scarab
cargo run -p scarab-daemon
```

## Plugin Template

```fsharp
// @name my-plugin
// @version 0.1.0
// @description What my plugin does
// @author Your Name
// @api-version 0.1.0
// @min-scarab-version 0.1.0

/// Called when plugin loads
let on_load (ctx: PluginContext) =
    ctx.log Info "Plugin loaded!"
    Ok ()

/// Intercept terminal output
let on_output (line: string) (ctx: PluginContext) =
    // Process line
    Action.Continue

/// Plugin metadata
let metadata = {
    name = "my-plugin"
    version = "0.1.0"
    description = "What my plugin does"
    author = "Your Name"
}
```

## Available Commands

### Build Commands
- `just plugin-build FILE` - Build single plugin
- `just plugin-build-all` - Build all plugins
- `just plugin-clean` - Clean build artifacts

### Validation Commands
- `just plugin-validate FILE` - Validate single plugin
- `just plugin-validate-all` - Validate all plugins

### Development Commands
- `just plugin-watch` - Watch and auto-rebuild
- `just plugin-new NAME` - Create from template
- `just plugin-status` - Show development status

### Testing Commands
- `just plugin-test` - Test plugin loading
- `just plugin-ci` - Run all CI checks

## Directory Structure

```
examples/fusabi/
├── TEMPLATE.fsx        # Full-featured template
├── README.md           # Examples documentation
├── hello.fsx           # Hello world example
├── theme.fsx           # Theme customization
├── keybindings.fsx     # Input interception
└── ui_overlay.fsx      # UI overlay demo
```

## Common Tasks

### Check Plugin Status

```bash
just plugin-status
```

Output:
```
Fusabi Plugin Development Status
=================================

Example Plugins:
  .fsx files: 5
  .fzb files: 5

Plugin API Version: 0.1.0

Available Commands:
  ...
```

### Run All Validations

```bash
just plugin-ci
```

This runs:
1. Validate all plugins
2. Test plugin loading
3. Display summary

### Debug Plugin

```bash
# Enable debug mode in plugin
let config = { debug_mode = true }

let on_output line ctx =
    if config.debug_mode then
        ctx.log Debug (sprintf "Line: %s" line)
    Action.Continue
```

### Clean and Rebuild

```bash
just plugin-clean
just plugin-build-all
```

## IDE Setup

### VSCode

1. Install recommended extensions (prompted automatically)
2. Use tasks: Ctrl+Shift+P > "Tasks: Run Task"
   - "plugin: build all"
   - "plugin: validate all"
   - "plugin: watch"

3. Debug: F5 to start debugging daemon or client

### Other Editors

EditorConfig is enabled for all editors:
- Consistent indentation
- Line endings
- Charset

## Troubleshooting

### Scripts Not Executable

```bash
chmod +x scripts/build-plugin.sh
chmod +x scripts/plugin-validator.sh
```

### Validation Errors

**Missing Metadata**
```bash
# Add to top of .fsx file
// @name my-plugin
// @version 1.0.0
// @api-version 0.1.0
```

**Invalid Version**
```bash
# Use semver format
// @version 1.0.0  (correct)
// @version 1.0    (incorrect)
```

### Build Errors

**Verbose Output**
```bash
./scripts/build-plugin.sh -v examples/fusabi/my-plugin.fsx
```

**Check Syntax**
```bash
just plugin-validate examples/fusabi/my-plugin.fsx
```

## Next Steps

1. Read full guide: `docs/PLUGIN_DEVELOPMENT.md`
2. Study examples: `examples/fusabi/README.md`
3. Review template: `examples/fusabi/TEMPLATE.fsx`
4. Join development: Create PRs with new plugins!

## Resources

- **Full Documentation**: `/home/beengud/raibid-labs/scarab/docs/PLUGIN_DEVELOPMENT.md`
- **Examples**: `/home/beengud/raibid-labs/scarab/examples/fusabi/`
- **API Reference**: `/home/beengud/raibid-labs/scarab/crates/scarab-plugin-api/`
- **CI Configuration**: `/home/beengud/raibid-labs/scarab/.github/workflows/plugins.yml`

## Quick Reference

| Command | Description |
|---------|-------------|
| `just plugin-new NAME` | Create new plugin |
| `just plugin-build FILE` | Build plugin |
| `just plugin-build-all` | Build all plugins |
| `just plugin-validate FILE` | Validate plugin |
| `just plugin-validate-all` | Validate all |
| `just plugin-watch` | Watch mode |
| `just plugin-test` | Test loading |
| `just plugin-ci` | Full CI |
| `just plugin-status` | Show status |
| `just plugin-clean` | Clean builds |

## Getting Help

1. Check documentation in `docs/`
2. Review example plugins in `examples/fusabi/`
3. Run validation for specific errors
4. Check CI logs on GitHub

Happy plugin development!
