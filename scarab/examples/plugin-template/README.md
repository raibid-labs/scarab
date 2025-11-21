# Scarab Plugin Template

This is a template for creating your own Scarab terminal emulator plugins.

## Quick Start

1. Copy this directory:
   ```bash
   cp -r examples/plugin-template my-plugin
   cd my-plugin
   ```

2. Update `Cargo.toml`:
   - Change `name` to your plugin name
   - Update version and other metadata

3. Update `src/lib.rs`:
   - Rename `ExamplePlugin` to your plugin name
   - Update the `PluginMetadata` in the constructor
   - Implement the hooks you need

4. Build your plugin:
   ```bash
   cargo build --release
   ```

5. Install your plugin:
   ```bash
   mkdir -p ~/.config/scarab/plugins
   cp target/release/libscarab_plugin_example.so ~/.config/scarab/plugins/
   ```

6. Configure your plugin in `~/.config/scarab/plugins.toml`:
   ```toml
   [[plugin]]
   name = "my-plugin"
   path = "~/.config/scarab/plugins/libmy_plugin.so"
   enabled = true

   [plugin.config]
   # Your custom configuration here
   ```

## Plugin Structure

```rust
use scarab_plugin_api::*;

pub struct MyPlugin {
    metadata: PluginMetadata,
    // Your state here
}

impl Plugin for MyPlugin {
    fn metadata(&self) -> &PluginMetadata {
        &self.metadata
    }

    // Implement hooks you need
    async fn on_output(&mut self, line: &str, ctx: &PluginContext) -> Result<Action> {
        // Your logic here
        Ok(Action::Continue)
    }
}
```

## Available Hooks

- `on_load(&mut self, ctx: &mut PluginContext)` - Called when plugin loads
- `on_unload(&mut self)` - Called when plugin unloads
- `on_output(&mut self, line: &str, ctx: &PluginContext)` - Before output displayed
- `on_input(&mut self, input: &[u8], ctx: &PluginContext)` - After input received
- `on_pre_command(&mut self, command: &str, ctx: &PluginContext)` - Before command executes
- `on_post_command(&mut self, command: &str, exit_code: i32, ctx: &PluginContext)` - After command completes
- `on_resize(&mut self, cols: u16, rows: u16, ctx: &PluginContext)` - Terminal resized
- `on_attach(&mut self, client_id: u64, ctx: &PluginContext)` - Client attached
- `on_detach(&mut self, client_id: u64, ctx: &PluginContext)` - Client detached

## Hook Actions

Hooks can return different actions:

- `Action::Continue` - Pass to next plugin
- `Action::Stop` - Stop processing, don't call remaining plugins
- `Action::Modify(Vec<u8>)` - Modify data and continue

## Plugin Context

The `PluginContext` provides access to terminal state:

```rust
// Get terminal size
let (cols, rows) = ctx.get_size();

// Get cell at position
let cell = ctx.get_cell(x, y);

// Set cell at position
ctx.set_cell(x, y, cell);

// Get line of text
let line = ctx.get_line(y);

// Get environment variable
let value = ctx.get_env("HOME");

// Store/retrieve plugin data
ctx.set_data("key", "value");
let value = ctx.get_data("key");

// Logging
ctx.log(LogLevel::Info, "message");

// Send notification
ctx.notify("Important message");
```

## Configuration

Access plugin configuration through `PluginContext`:

```rust
// In on_load
async fn on_load(&mut self, ctx: &mut PluginContext) -> Result<()> {
    // Get required config
    let threshold: u32 = ctx.config.get("threshold")?;

    // Get optional config
    let enabled: bool = ctx.config.get_opt("enabled").unwrap_or(true);

    Ok(())
}
```

## Best Practices

1. **Keep hooks fast** - Aim for <1ms execution time
2. **Handle errors gracefully** - Don't panic, return `Result`
3. **Use logging** - Help users debug issues
4. **Document configuration** - Clearly document config options
5. **Test thoroughly** - Write tests for your plugin logic
6. **Version carefully** - Follow semver for compatibility

## Example Plugins

See `examples/` directory for more examples:
- `auto-notify` - Notification on error keywords
- `vim-mode` - Vim-style keybindings
- `session-logger` - Log all terminal output

## Troubleshooting

### Plugin not loading

1. Check file permissions: `chmod +x ~/.config/scarab/plugins/myplugin.so`
2. Check Scarab logs: `scarab --log-level debug`
3. Verify plugin path in `plugins.toml`

### Plugin disabled automatically

Plugins are auto-disabled after 3 consecutive failures. Check:
1. Plugin logs for errors
2. API version compatibility
3. Configuration validity

### Hook not being called

1. Verify hook is implemented (not just default)
2. Check plugin is enabled: `scarab plugins list`
3. Check hook execution order in logs

## Resources

- [Scarab Plugin API Documentation](../../docs/plugin-api.md)
- [Plugin Development Guide](../../docs/plugin-development-guide.md)
- [API Reference](https://docs.rs/scarab-plugin-api)
