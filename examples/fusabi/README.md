# Fusabi Plugin Examples

This directory contains example Fusabi plugins for the Scarab terminal emulator.

## Plugin Structure

Fusabi plugins are written in F# syntax (`.fsx` files) and compiled to bytecode (`.fzb` files) for execution.

### Directory Layout

```
examples/fusabi/
├── README.md              # This file
├── TEMPLATE.fsx           # Plugin template for creating new plugins
├── hello.fsx              # Simple hello world example
├── theme.fsx              # Theme customization example
├── keybindings.fsx        # Custom keybindings example
└── ui_overlay.fsx         # UI overlay example
```

## Plugin Metadata

All plugins must include metadata comments at the top of the file:

```fsharp
// @name plugin-name
// @version 1.0.0
// @description Short description of what the plugin does
// @author Your Name
// @homepage https://github.com/yourusername/plugin-name (optional)
// @license MIT (optional)
// @api-version 0.1.0
// @min-scarab-version 0.1.0
```

## Plugin Lifecycle Hooks

Plugins can implement the following lifecycle hooks:

- `on_load` - Called when plugin is loaded
- `on_unload` - Called when plugin is unloaded

## Event Hooks

Plugins can hook into terminal events:

- `on_output` - Intercept terminal output before display
- `on_input` - Intercept user input before sending to PTY
- `on_pre_command` - Called before command execution
- `on_post_command` - Called after command completes
- `on_resize` - Called when terminal is resized
- `on_attach` - Called when client attaches
- `on_detach` - Called when client detaches
- `on_remote_command` - Handle remote commands from client

## Building Plugins

### Build Single Plugin

```bash
just plugin-build examples/fusabi/hello.fsx
```

### Build All Plugins

```bash
just plugin-build-all
```

### Watch for Changes

```bash
just plugin-watch
```

## Validating Plugins

### Validate Single Plugin

```bash
just plugin-validate examples/fusabi/hello.fsx
```

### Validate All Plugins

```bash
just plugin-validate-all
```

## Creating New Plugins

### From Template

```bash
just plugin-new my-plugin
```

This creates `examples/fusabi/my-plugin.fsx` from the template.

### Manual Creation

1. Copy `TEMPLATE.fsx` to a new file
2. Update the metadata at the top
3. Implement your plugin logic
4. Build and validate:

```bash
just plugin-build examples/fusabi/my-plugin.fsx
just plugin-validate examples/fusabi/my-plugin.fsx
```

## Plugin Development Workflow

1. **Create** - Create new plugin from template
2. **Develop** - Write plugin logic with hot-reload support
3. **Build** - Compile .fsx to .fzb bytecode
4. **Validate** - Check metadata and structure
5. **Test** - Load in Scarab daemon and test functionality
6. **Deploy** - Install to `~/.config/scarab/plugins/`

## Plugin API

### Context Object

The `PluginContext` provides access to Scarab internals:

```fsharp
type PluginContext = {
    // Send notification to client
    notify: string -> unit

    // Log message
    log: LogLevel -> string -> unit

    // Register command
    register_command: string -> (unit -> unit) -> unit

    // Get terminal size
    get_terminal_size: unit -> (uint16 * uint16)

    // Send data to PTY
    write_to_pty: byte[] -> unit
}
```

### Action Types

Hooks return actions to control data flow:

```fsharp
type Action =
    | Continue          // Pass through unchanged
    | Block            // Suppress (don't process)
    | Modified of 'T   // Pass modified version
```

## Testing Plugins

### Unit Tests

```bash
cargo test -p scarab-daemon plugin
```

### Integration Tests

```bash
# Start daemon with plugin
cargo run -p scarab-daemon -- --plugin examples/fusabi/hello.fzb

# In another terminal, start client
cargo run -p scarab-client
```

## Examples Explained

### hello.fsx

Simple example demonstrating:
- Basic metadata
- Plugin initialization
- Console output

### theme.fsx

Demonstrates:
- Color scheme customization
- UI configuration
- Theme switching

### keybindings.fsx

Demonstrates:
- Input interception
- Custom key combinations
- Command binding

### ui_overlay.fsx

Demonstrates:
- Custom UI rendering
- Remote commands
- Client-side integration

## Best Practices

1. **Metadata** - Always include complete metadata
2. **Error Handling** - Return `Result` types, handle errors gracefully
3. **Performance** - Keep hooks fast, avoid blocking operations
4. **State** - Use mutable state sparingly, prefer functional style
5. **Testing** - Validate plugins before deploying
6. **Documentation** - Comment your code, especially public functions

## Plugin Installation

### Development

Plugins in `examples/fusabi/` are automatically discovered during development.

### Production

Install plugins to:
```
~/.config/scarab/plugins/
```

Or system-wide:
```
/usr/share/scarab/plugins/
```

## Troubleshooting

### Plugin Not Loading

1. Check metadata is complete and valid
2. Validate plugin: `just plugin-validate your-plugin.fsx`
3. Check Scarab logs for errors
4. Verify API version compatibility

### Compilation Errors

1. Check F# syntax
2. Ensure all types match API definitions
3. Run with verbose flag: `./scripts/build-plugin.sh -v your-plugin.fsx`

### Runtime Errors

1. Enable debug mode in plugin
2. Check daemon logs
3. Test plugin in isolation

## Resources

- [Fusabi Language Documentation](https://github.com/fusabi-lang/fusabi)
- [Scarab Plugin API Documentation](../../docs/plugin-api.md)
- [F# Language Guide](https://fsharp.org/)

## Contributing

To contribute example plugins:

1. Create plugin in `examples/fusabi/`
2. Add metadata and documentation
3. Validate: `just plugin-validate-all`
4. Submit pull request

## License

Example plugins are provided as-is under the MIT license.
