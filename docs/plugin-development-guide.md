# Scarab Plugin Development Guide

> **DEPRECATED**: This document is outdated. See [Plugin Development Guide](./plugin-development/README.md) for current information.

Complete guide to developing plugins for the Scarab terminal emulator.

## Table of Contents

1. [Introduction](#introduction)
2. [Getting Started](#getting-started)
3. [Plugin Architecture](#plugin-architecture)
4. [API Reference](#api-reference)
5. [Hook System](#hook-system)
6. [Configuration](#configuration)
7. [Best Practices](#best-practices)
8. [Testing](#testing)
9. [Deployment](#deployment)
10. [Troubleshooting](#troubleshooting)

## Introduction

Scarab plugins allow you to extend the terminal emulator's functionality by intercepting and modifying terminal events, input/output, and implementing custom behaviors.

### What Can Plugins Do?

- Intercept and modify terminal output before display
- Process keyboard input before it reaches the PTY
- Execute code before/after commands
- React to terminal events (resize, attach, detach)
- Access and modify terminal grid state
- Store persistent data
- Send notifications to users

### Plugin Types

Scarab supports two plugin types:

1. **Compiled Plugins (.fzb)** - Fusabi bytecode, high performance
2. **Script Plugins (.fsx)** - F# scripts, easier development

## Getting Started

### Prerequisites

- Rust 1.70 or later
- Scarab 0.1.0 or later
- Basic understanding of async Rust

### Creating Your First Plugin

1. **Clone the Template**

```bash
cp -r examples/plugin-template my-first-plugin
cd my-first-plugin
```

2. **Update Metadata**

Edit `Cargo.toml`:
```toml
[package]
name = "my-first-plugin"
version = "0.1.0"
edition = "2021"
```

3. **Implement Plugin Logic**

Edit `src/lib.rs`:
```rust
use scarab_plugin_api::*;
use async_trait::async_trait;

pub struct MyFirstPlugin {
    metadata: PluginMetadata,
}

impl MyFirstPlugin {
    pub fn new() -> Self {
        Self {
            metadata: PluginMetadata::new(
                "my-first-plugin",
                "0.1.0",
                "My first Scarab plugin",
                "Your Name",
            ),
        }
    }
}

#[async_trait]
impl Plugin for MyFirstPlugin {
    fn metadata(&self) -> &PluginMetadata {
        &self.metadata
    }

    async fn on_load(&mut self, ctx: &mut PluginContext) -> Result<()> {
        ctx.log(LogLevel::Info, "My first plugin loaded!");
        Ok(())
    }

    async fn on_output(&mut self, line: &str, ctx: &PluginContext) -> Result<Action> {
        if line.contains("Hello") {
            ctx.notify("Found 'Hello' in output!");
        }
        Ok(Action::Continue)
    }
}
```

4. **Build**

```bash
cargo build --release
```

5. **Install**

```bash
mkdir -p ~/.config/scarab/plugins
cp target/release/libmy_first_plugin.so ~/.config/scarab/plugins/
```

6. **Configure**

Create/edit `~/.config/scarab/plugins.toml`:
```toml
[[plugin]]
name = "my-first-plugin"
path = "~/.config/scarab/plugins/libmy_first_plugin.so"
enabled = true
```

7. **Test**

Start Scarab and run:
```bash
echo "Hello, World!"
# You should see a notification
```

## Plugin Architecture

### Plugin Lifecycle

```
┌─────────────┐
│   Created   │
└──────┬──────┘
       │
       ▼
┌─────────────┐
│  on_load()  │ ◄─── Initialize state, validate config
└──────┬──────┘
       │
       ▼
┌─────────────────────────┐
│   Hook Execution Loop   │ ◄─── Process events
│                         │
│  • on_output()          │
│  • on_input()           │
│  • on_pre_command()     │
│  • on_post_command()    │
│  • on_resize()          │
│  • on_attach()          │
│  • on_detach()          │
└──────┬──────────────────┘
       │
       ▼
┌─────────────┐
│ on_unload() │ ◄─── Cleanup resources
└──────┬──────┘
       │
       ▼
┌─────────────┐
│  Destroyed  │
└─────────────┘
```

### Hook Execution Order

When an event occurs, Scarab calls hooks in this order:

1. **Pre-hooks** (can prevent action)
   - `on_pre_command()` - Before command execution

2. **Main hooks** (can modify data)
   - `on_output()` - Output processing
   - `on_input()` - Input processing

3. **Post-hooks** (notification only)
   - `on_post_command()` - After command completes

4. **Event hooks** (notification)
   - `on_resize()`, `on_attach()`, `on_detach()`

### Plugin Isolation

Each plugin runs with:
- **Timeout**: 1 second max per hook (configurable)
- **Panic catching**: Panics don't crash daemon
- **Failure tracking**: Auto-disable after 3 consecutive failures
- **Memory isolation**: Separate state per plugin

## API Reference

### Plugin Trait

```rust
#[async_trait]
pub trait Plugin: Send + Sync {
    fn metadata(&self) -> &PluginMetadata;

    async fn on_load(&mut self, ctx: &mut PluginContext) -> Result<()>;
    async fn on_unload(&mut self) -> Result<()>;

    async fn on_output(&mut self, line: &str, ctx: &PluginContext) -> Result<Action>;
    async fn on_input(&mut self, input: &[u8], ctx: &PluginContext) -> Result<Action>;
    async fn on_pre_command(&mut self, command: &str, ctx: &PluginContext) -> Result<Action>;
    async fn on_post_command(&mut self, command: &str, exit_code: i32, ctx: &PluginContext) -> Result<()>;
    async fn on_resize(&mut self, cols: u16, rows: u16, ctx: &PluginContext) -> Result<()>;
    async fn on_attach(&mut self, client_id: u64, ctx: &PluginContext) -> Result<()>;
    async fn on_detach(&mut self, client_id: u64, ctx: &PluginContext) -> Result<()>;
}
```

### PluginMetadata

```rust
pub struct PluginMetadata {
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
    pub homepage: Option<String>,
    pub api_version: String,
    pub min_scarab_version: String,
}

impl PluginMetadata {
    pub fn new(name: impl Into<String>, version: impl Into<String>,
               description: impl Into<String>, author: impl Into<String>) -> Self;
    pub fn with_homepage(self, homepage: impl Into<String>) -> Self;
    pub fn is_compatible(&self, current_api_version: &str) -> bool;
}
```

### PluginContext

```rust
pub struct PluginContext {
    pub config: PluginConfigData,
    pub state: Arc<Mutex<SharedState>>,
    pub logger_name: String,
}

impl PluginContext {
    // Terminal access
    pub fn get_cell(&self, x: u16, y: u16) -> Option<Cell>;
    pub fn set_cell(&self, x: u16, y: u16, cell: Cell) -> bool;
    pub fn get_line(&self, y: u16) -> Option<String>;
    pub fn get_size(&self) -> (u16, u16);
    pub fn get_cursor(&self) -> (u16, u16);

    // Environment
    pub fn get_env(&self, key: &str) -> Option<String>;

    // Plugin data storage
    pub fn set_data(&self, key: impl Into<String>, value: impl Into<String>);
    pub fn get_data(&self, key: &str) -> Option<String>;

    // Logging and notifications
    pub fn log(&self, level: LogLevel, message: &str);
    pub fn notify(&self, message: &str);
}
```

### Action

```rust
pub enum Action {
    Continue,           // Pass to next plugin
    Stop,              // Stop processing chain
    Modify(Vec<u8>),   // Modify data and continue
}
```

## Hook System

### Output Hook (on_output)

Called before terminal output is displayed.

```rust
async fn on_output(&mut self, line: &str, ctx: &PluginContext) -> Result<Action> {
    // Example: Syntax highlighting
    if line.starts_with("error:") {
        // Modify to add color codes
        let colored = format!("\x1b[31m{}\x1b[0m", line);
        return Ok(Action::Modify(colored.into_bytes()));
    }

    Ok(Action::Continue)
}
```

**Use Cases:**
- Syntax highlighting
- Output filtering
- Text transformation
- Keyword detection

### Input Hook (on_input)

Called after keyboard input is received.

```rust
async fn on_input(&mut self, input: &[u8], ctx: &PluginContext) -> Result<Action> {
    // Example: Vim-style escape from insert mode
    if input == b"\x1b" {  // ESC key
        self.mode = Mode::Normal;
        ctx.notify("-- NORMAL --");
    }

    Ok(Action::Continue)
}
```

**Use Cases:**
- Keyboard shortcuts
- Input validation
- Text expansion
- Modal editing

### Pre-Command Hook (on_pre_command)

Called before a command is executed.

```rust
async fn on_pre_command(&mut self, command: &str, ctx: &PluginContext) -> Result<Action> {
    // Example: Dangerous command warning
    if command.starts_with("rm -rf") {
        ctx.notify("⚠️  Dangerous command detected!");
        // Could return Action::Stop to prevent execution
    }

    Ok(Action::Continue)
}
```

**Use Cases:**
- Command logging
- Safety checks
- Command aliasing
- Permission enforcement

### Post-Command Hook (on_post_command)

Called after a command completes.

```rust
async fn on_post_command(&mut self, command: &str, exit_code: i32, ctx: &PluginContext) -> Result<()> {
    // Example: Track failed commands
    if exit_code != 0 {
        self.failed_commands.push(command.to_string());
        ctx.notify(&format!("Command failed: {} (exit code {})", command, exit_code));
    }

    Ok(())
}
```

**Use Cases:**
- Command statistics
- Error tracking
- Notifications
- History management

### Resize Hook (on_resize)

Called when terminal size changes.

```rust
async fn on_resize(&mut self, cols: u16, rows: u16, ctx: &PluginContext) -> Result<()> {
    // Example: Adjust UI elements
    if cols < 80 {
        ctx.notify("Terminal too narrow for optimal display");
    }

    Ok(())
}
```

### Attach/Detach Hooks

Called when clients connect/disconnect.

```rust
async fn on_attach(&mut self, client_id: u64, ctx: &PluginContext) -> Result<()> {
    ctx.notify(&format!("Client {} joined the session", client_id));
    Ok(())
}

async fn on_detach(&mut self, client_id: u64, ctx: &PluginContext) -> Result<()> {
    ctx.notify(&format!("Client {} left the session", client_id));
    Ok(())
}
```

## Configuration

### Plugin Configuration File

`~/.config/scarab/plugins.toml`:

```toml
[[plugin]]
name = "auto-notify"
path = "~/.config/scarab/plugins/auto-notify.so"
enabled = true

[plugin.config]
keywords = ["ERROR", "FAIL", "PANIC", "FATAL"]
notification_style = "urgent"
max_notifications_per_minute = 5

[[plugin]]
name = "vim-mode"
path = "~/.config/scarab/plugins/vim-mode.so"
enabled = false  # Disabled
```

### Accessing Configuration

```rust
async fn on_load(&mut self, ctx: &mut PluginContext) -> Result<()> {
    // Get required config value
    let keywords: Vec<String> = ctx.config.get("keywords")?;

    // Get optional config with default
    let style: String = ctx.config.get_opt("notification_style")
        .unwrap_or_else(|| "normal".to_string());

    // Get optional config
    if let Some(max) = ctx.config.get_opt::<u32>("max_notifications_per_minute") {
        self.rate_limit = max;
    }

    Ok(())
}
```

## Best Practices

### Performance

1. **Keep hooks fast** - Target <1ms execution time
2. **Avoid blocking I/O** - Use async operations
3. **Cache expensive computations** - Store in plugin state
4. **Minimize allocations** - Reuse buffers where possible

```rust
// ❌ Bad: Blocking I/O
async fn on_output(&mut self, line: &str, ctx: &PluginContext) -> Result<Action> {
    std::fs::write("/tmp/log", line)?;  // Blocks!
    Ok(Action::Continue)
}

// ✅ Good: Async I/O
async fn on_output(&mut self, line: &str, ctx: &PluginContext) -> Result<Action> {
    tokio::fs::write("/tmp/log", line).await?;
    Ok(Action::Continue)
}
```

### Error Handling

1. **Never panic** - Return `Result` instead
2. **Provide context** - Include helpful error messages
3. **Log errors** - Help users debug issues

```rust
// ❌ Bad: Panics
async fn on_load(&mut self, ctx: &mut PluginContext) -> Result<()> {
    let config = ctx.config.get("required").unwrap();  // Panics!
    Ok(())
}

// ✅ Good: Returns error
async fn on_load(&mut self, ctx: &mut PluginContext) -> Result<()> {
    let config = ctx.config.get("required")
        .map_err(|e| PluginError::ConfigError(
            format!("Missing required config 'required': {}", e)
        ))?;
    Ok(())
}
```

### State Management

1. **Keep state minimal** - Only store what you need
2. **Use Arc/Mutex for sharing** - Thread-safe state
3. **Clean up in on_unload** - Release resources

```rust
pub struct MyPlugin {
    metadata: PluginMetadata,
    // Minimal state
    notification_count: u32,
    last_command: Option<String>,
}
```

### Logging

Use appropriate log levels:

```rust
ctx.log(LogLevel::Error, "Failed to parse config");  // User must see
ctx.log(LogLevel::Warn, "Deprecated feature used");   // User should see
ctx.log(LogLevel::Info, "Plugin initialized");         // User may see
ctx.log(LogLevel::Debug, "Processing line 42");        // Developer info
```

## Testing

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_context() -> PluginContext {
        let state = Arc::new(Mutex::new(SharedState::new(80, 24)));
        PluginContext::new(Default::default(), state, "test")
    }

    #[tokio::test]
    async fn test_output_hook() {
        let mut plugin = MyPlugin::new();
        let ctx = create_test_context();

        let result = plugin.on_output("ERROR: test", &ctx).await;
        assert!(result.is_ok());
    }
}
```

### Integration Tests

```rust
#[tokio::test]
async fn test_plugin_lifecycle() {
    let mut manager = PluginManager::new(create_test_context());
    let plugin = Box::new(MyPlugin::new());

    // Test loading
    assert!(manager.register_plugin(plugin).await.is_ok());
    assert_eq!(manager.enabled_count(), 1);

    // Test hook dispatch
    let output = manager.dispatch_output("test").await.unwrap();
    assert_eq!(output, "test");

    // Test unloading
    assert!(manager.unload_all().await.is_ok());
    assert_eq!(manager.enabled_count(), 0);
}
```

## Deployment

### Building for Release

```bash
# Build optimized binary
cargo build --release

# Strip debug symbols (optional)
strip target/release/libmyplugin.so
```

### Installation

```bash
# System-wide (requires root)
sudo cp target/release/libmyplugin.so /usr/local/share/scarab/plugins/

# User-specific
mkdir -p ~/.config/scarab/plugins
cp target/release/libmyplugin.so ~/.config/scarab/plugins/
```

### Distribution

Create a release package:

```bash
# Create archive
tar czf myplugin-v1.0.0.tar.gz \
    target/release/libmyplugin.so \
    README.md \
    LICENSE \
    example-config.toml
```

## Troubleshooting

### Plugin Not Loading

**Symptom**: Plugin doesn't appear in `scarab plugins list`

**Solutions**:
1. Check file permissions: `chmod +x ~/.config/scarab/plugins/myplugin.so`
2. Verify path in `plugins.toml`
3. Check Scarab logs: `scarab --log-level debug`
4. Verify API version compatibility

### Plugin Disabled Automatically

**Symptom**: Plugin shows as disabled after running

**Cause**: 3 consecutive failures triggered auto-disable

**Solutions**:
1. Check plugin logs for errors
2. Fix underlying issue
3. Re-enable in `plugins.toml`
4. Restart Scarab

### Hook Not Called

**Symptom**: Hook code never executes

**Solutions**:
1. Verify hook is implemented (not just default)
2. Check plugin is enabled
3. Check hook execution order in logs
4. Verify no earlier plugin returns `Action::Stop`

### Performance Issues

**Symptom**: Terminal feels slow with plugin

**Solutions**:
1. Profile hook execution time
2. Move expensive work to background tasks
3. Cache results where possible
4. Consider disabling plugin temporarily

### Memory Leaks

**Symptom**: Memory usage grows over time

**Solutions**:
1. Implement `on_unload` cleanup
2. Use weak references where appropriate
3. Clear caches periodically
4. Profile memory usage with valgrind

## Advanced Topics

### Hot Reloading

Plugins can be reloaded without restarting Scarab:

```bash
scarab plugins reload my-plugin
```

### Plugin Dependencies

If your plugin depends on another:

```toml
[plugin.requires]
plugins = ["base-plugin"]
min_versions = ["1.0.0"]
```

### Custom Notifications

Implement rich notifications:

```rust
ctx.notify_with_level(NotificationLevel::Urgent, "Critical error!");
```

### Performance Metrics

Track plugin performance:

```rust
let start = Instant::now();
// ... plugin work ...
let elapsed = start.elapsed();
ctx.log(LogLevel::Debug, &format!("Hook took {:?}", elapsed));
```

## Resources

- [API Documentation](https://docs.rs/scarab-plugin-api)
- [Example Plugins](../examples/)
- [Scarab GitHub](https://github.com/yourusername/scarab)
- [Plugin Registry](https://scarab-plugins.io)

## Contributing

Found a bug or want to improve this guide? Contributions welcome!

1. Fork the repository
2. Create a feature branch
3. Submit a pull request

## License

This guide is part of the Scarab project and is licensed under the same terms.
