# Plugin Development

Scarab uses the Fusabi scripting language for plugin development.

## Fusabi Language

Fusabi is a high-performance F# dialect designed for Rust integration.

- **Official Repository**: https://github.com/fusabi-lang/fusabi
- **Language**: F#-like syntax with Rust FFI
- **Type System**: ML-style with type inference

## Dual Runtime System

Scarab uses two Fusabi runtimes for different use cases:

| Runtime | File Type | Location | Purpose |
|---------|-----------|----------|---------|
| Fusabi VM | `.fzb` | Daemon | Compiled bytecode for high-performance hooks, output scanning, multiplexing logic |
| Fusabi Frontend | `.fsx` | Client | F# source parser/compiler for hot-reloadable UI scripts |

## Plugin Types

### Daemon Plugins (.fzb)

Compiled bytecode plugins that run in the daemon:

- **Output Filters**: Process terminal output
- **Command Hooks**: Intercept and modify commands
- **Multiplexing Logic**: Custom session management
- **Performance Critical**: Runs on every PTY output

Example use cases:
- Syntax highlighting
- Command completion
- Output redirection
- Session recording

### Client Plugins (.fsx)

Interpreted F# scripts that run in the client:

- **UI Extensions**: Custom widgets and overlays
- **Keybinding Handlers**: Dynamic input processing
- **Theme Scripts**: Dynamic color schemes
- **Hot-Reloadable**: No Rust recompilation needed

Example use cases:
- Custom status bars
- Notification overlays
- Dynamic themes
- Interactive widgets

## Plugin API

Plugin APIs are defined in `crates/scarab-plugin-api/`.

### Core Traits

```rust
pub trait Plugin {
    fn init(&mut self) -> Result<()>;
    fn on_output(&mut self, data: &[u8]) -> Result<Vec<u8>>;
    fn on_input(&mut self, data: &[u8]) -> Result<Vec<u8>>;
}
```

For complete API documentation, see the [API Reference](../reference/api.md).

## Development Workflow

### Creating a Daemon Plugin (.fzb)

1. Write Fusabi code (`.fs` source)
2. Compile to bytecode (`.fzb`)
3. Place in `~/.config/scarab/plugins/daemon/`
4. Restart daemon

### Creating a Client Plugin (.fsx)

1. Write Fusabi script (`.fsx`)
2. Place in `~/.config/scarab/plugins/client/`
3. Hot-reload (no restart needed)

## Plugin Configuration

Configure plugins in `~/.config/scarab/config.toml`:

```toml
[plugins.daemon]
enabled = ["syntax-highlight.fzb", "command-hooks.fzb"]

[plugins.client]
enabled = ["status-bar.fsx", "theme.fsx"]
```

## Plugin Development Status

- **Phase 7.0**: Fusabi integration (In Progress)
- **Phase 7.1**: Daemon plugin loader
- **Phase 7.2**: Client plugin loader
- **Phase 7.3**: Plugin API stabilization

## External Dependencies

- `fusabi-vm` - Official Fusabi VM runtime for `.fzb` bytecode
- `fusabi-frontend` - Official Fusabi compiler/parser for `.fsx` scripts

Both are from https://github.com/fusabi-lang/fusabi

## Resources

- [Fusabi Language Documentation](https://github.com/fusabi-lang/fusabi)
- [Plugin API Reference](../reference/api.md)
- [Configuration Guide](../user-guide/configuration.md)
