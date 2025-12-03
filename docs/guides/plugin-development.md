# Plugin Development Guide

> **DEPRECATED**: This document is outdated. See [Plugin Development Guide](../plugin-development/README.md) for current information.

Learn how to build powerful plugins for Scarab terminal emulator.

## Table of Contents

- [Getting Started](#getting-started)
- [Plugin Structure](#plugin-structure)
- [Hook System](#hook-system)
- [Plugin Context](#plugin-context)
- [Examples](#examples)
- [Best Practices](#best-practices)
- [Publishing](#publishing)

## Getting Started

### Prerequisites

- Rust 1.70+
- Scarab installed
- Basic understanding of Rust

### Create a New Plugin

```bash
cargo new --lib my-scarab-plugin
cd my-scarab-plugin
```

### Add Dependencies

```toml
[dependencies]
scarab-plugin-api = "0.1"
serde = { version = "1.0", features = ["derive"] }

[lib]
crate-type = ["cdylib"]
```

## Plugin Structure

### Basic Plugin

```rust
use scarab_plugin_api::{Plugin, PluginMetadata, PluginContext, Action, HookType};

pub struct MyPlugin {
    enabled: bool,
}

impl Plugin for MyPlugin {
    fn metadata() -> PluginMetadata {
        PluginMetadata {
            name: "my-plugin".to_string(),
            version: "0.1.0".to_string(),
            description: "My awesome plugin".to_string(),
            author: "Your Name".to_string(),
            hooks: vec![HookType::OnInput, HookType::OnOutput],
        }
    }

    fn on_init(&mut self, ctx: &mut PluginContext) -> Result<(), String> {
        ctx.log_info("Plugin initialized!");
        self.enabled = true;
        Ok(())
    }

    fn on_input(&mut self, ctx: &mut PluginContext, input: &str) -> Result<Action, String> {
        if input.starts_with("/my-command") {
            ctx.send_output("Command executed!\n");
            return Ok(Action::Consume); // Prevent input from reaching shell
        }
        Ok(Action::PassThrough)
    }

    fn on_output(&mut self, ctx: &mut PluginContext, output: &str) -> Result<String, String> {
        // Modify output before rendering
        let modified = output.replace("error", "\x1b[31merror\x1b[0m");
        Ok(modified)
    }
}

// Export the plugin
scarab_plugin_api::export_plugin!(MyPlugin);
```

## Hook System

Plugins can hook into various terminal events:

### Available Hooks

#### `on_init`

Called when the plugin is loaded.

```rust
fn on_init(&mut self, ctx: &mut PluginContext) -> Result<(), String> {
    // Initialize plugin state
    ctx.set_config("my_key", "my_value")?;
    Ok(())
}
```

#### `on_input`

Process user input before it reaches the shell.

```rust
fn on_input(&mut self, ctx: &mut PluginContext, input: &str) -> Result<Action, String> {
    if input == "/help\n" {
        ctx.send_output("Available commands:\n  /help - Show this help\n");
        return Ok(Action::Consume);  // Don't send to shell
    }
    Ok(Action::PassThrough)  // Normal processing
}
```

**Actions:**
- `Action::PassThrough` - Continue normal processing
- `Action::Consume` - Stop processing (don't send to shell)
- `Action::Modified(String)` - Send modified input

#### `on_output`

Process shell output before rendering.

```rust
fn on_output(&mut self, ctx: &mut PluginContext, output: &str) -> Result<String, String> {
    // Syntax highlight errors
    let highlighted = output.replace("ERROR", "\x1b[1;31mERROR\x1b[0m");
    Ok(highlighted)
}
```

#### `on_resize`

Handle terminal resize events.

```rust
fn on_resize(&mut self, ctx: &mut PluginContext, cols: u16, rows: u16) -> Result<(), String> {
    ctx.log_info(&format!("Terminal resized to {}x{}", cols, rows));
    Ok(())
}
```

#### `on_command`

Handle custom commands from the command palette.

```rust
fn on_command(&mut self, ctx: &mut PluginContext, cmd: &str) -> Result<(), String> {
    match cmd {
        "my-plugin:toggle" => {
            self.enabled = !self.enabled;
            ctx.log_info(&format!("Plugin {}", if self.enabled { "enabled" } else { "disabled" }));
        }
        _ => return Err(format!("Unknown command: {}", cmd)),
    }
    Ok(())
}
```

## Plugin Context

The `PluginContext` provides access to terminal state and utilities:

### Configuration

```rust
// Get config value
let value: String = ctx.get_config("my_key")?;

// Set config value
ctx.set_config("my_key", "new_value")?;

// Get terminal dimensions
let (cols, rows) = ctx.get_size();
```

### Output

```rust
// Send output to terminal
ctx.send_output("Hello from plugin!\n");

// Send formatted output
ctx.send_output(&format!("Value: {}\n", value));

// Send ANSI escape sequences
ctx.send_output("\x1b[1;32mGreen text\x1b[0m\n");
```

### Logging

```rust
// Log levels
ctx.log_info("Informational message");
ctx.log_warn("Warning message");
ctx.log_error("Error message");
ctx.log_debug("Debug message (only in debug builds)");
```

### State Management

```rust
// Store plugin state
ctx.set_state("counter", &42)?;

// Retrieve plugin state
let counter: i32 = ctx.get_state("counter")?;

// Remove state
ctx.remove_state("counter");
```

## Examples

### Example 1: Git Status Plugin

Shows git status in the prompt.

```rust
use scarab_plugin_api::*;
use std::process::Command;

pub struct GitStatusPlugin;

impl Plugin for GitStatusPlugin {
    fn metadata() -> PluginMetadata {
        PluginMetadata {
            name: "git-status".to_string(),
            version: "0.1.0".to_string(),
            description: "Show git status in prompt".to_string(),
            author: "Scarab Team".to_string(),
            hooks: vec![HookType::OnOutput],
        }
    }

    fn on_output(&mut self, ctx: &mut PluginContext, output: &str) -> Result<String, String> {
        // Check if this is a prompt
        if !output.contains("$ ") {
            return Ok(output.to_string());
        }

        // Get git status
        let status = Command::new("git")
            .args(&["status", "--short"])
            .output()
            .ok()
            .filter(|o| o.status.success())
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .unwrap_or_default();

        if status.is_empty() {
            return Ok(output.to_string());
        }

        // Count changes
        let changes = status.lines().count();
        let indicator = format!("\x1b[33m[{}]\x1b[0m ", changes);

        // Insert before prompt
        let modified = output.replace("$ ", &format!("{}$ ", indicator));
        Ok(modified)
    }
}

scarab_plugin_api::export_plugin!(GitStatusPlugin);
```

### Example 2: Command History Plugin

Saves command history with timestamps.

```rust
use scarab_plugin_api::*;
use std::fs::OpenOptions;
use std::io::Write;

pub struct HistoryPlugin {
    history_file: String,
}

impl Plugin for HistoryPlugin {
    fn metadata() -> PluginMetadata {
        PluginMetadata {
            name: "command-history".to_string(),
            version: "0.1.0".to_string(),
            description: "Enhanced command history".to_string(),
            author: "Scarab Team".to_string(),
            hooks: vec![HookType::OnInput],
        }
    }

    fn on_init(&mut self, ctx: &mut PluginContext) -> Result<(), String> {
        self.history_file = format!("{}/.scarab_history", std::env::var("HOME").unwrap());
        ctx.log_info(&format!("History file: {}", self.history_file));
        Ok(())
    }

    fn on_input(&mut self, ctx: &mut PluginContext, input: &str) -> Result<Action, String> {
        // Ignore empty commands
        if input.trim().is_empty() {
            return Ok(Action::PassThrough);
        }

        // Write to history file
        let timestamp = chrono::Utc::now().to_rfc3339();
        let entry = format!("{} | {}\n", timestamp, input.trim());

        if let Ok(mut file) = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.history_file)
        {
            let _ = file.write_all(entry.as_bytes());
        }

        Ok(Action::PassThrough)
    }
}

scarab_plugin_api::export_plugin!(HistoryPlugin);
```

### Example 3: Syntax Highlighting Plugin

Highlights output based on patterns.

```rust
use scarab_plugin_api::*;
use regex::Regex;

pub struct SyntaxHighlightPlugin {
    patterns: Vec<(Regex, String)>,
}

impl Plugin for SyntaxHighlightPlugin {
    fn metadata() -> PluginMetadata {
        PluginMetadata {
            name: "syntax-highlight".to_string(),
            version: "0.1.0".to_string(),
            description: "Syntax highlighting for terminal output".to_string(),
            author: "Scarab Team".to_string(),
            hooks: vec![HookType::OnOutput],
        }
    }

    fn on_init(&mut self, ctx: &mut PluginContext) -> Result<(), String> {
        self.patterns = vec![
            (Regex::new(r"\bERROR\b").unwrap(), "\x1b[1;31m$0\x1b[0m".to_string()),
            (Regex::new(r"\bWARN\b").unwrap(), "\x1b[1;33m$0\x1b[0m".to_string()),
            (Regex::new(r"\bINFO\b").unwrap(), "\x1b[1;36m$0\x1b[0m".to_string()),
            (Regex::new(r"\bSUCCESS\b").unwrap(), "\x1b[1;32m$0\x1b[0m".to_string()),
        ];
        Ok(())
    }

    fn on_output(&mut self, ctx: &mut PluginContext, output: &str) -> Result<String, String> {
        let mut result = output.to_string();
        for (pattern, replacement) in &self.patterns {
            result = pattern.replace_all(&result, replacement.as_str()).to_string();
        }
        Ok(result)
    }
}

scarab_plugin_api::export_plugin!(SyntaxHighlightPlugin);
```

## Best Practices

### 1. Error Handling

Always return descriptive errors:

```rust
fn on_input(&mut self, ctx: &mut PluginContext, input: &str) -> Result<Action, String> {
    let parsed = parse_command(input)
        .map_err(|e| format!("Failed to parse command: {}", e))?;

    Ok(Action::PassThrough)
}
```

### 2. Performance

Avoid blocking operations:

```rust
// BAD: Blocking network request
fn on_output(&mut self, ctx: &mut PluginContext, output: &str) -> Result<String, String> {
    let data = reqwest::blocking::get("https://api.example.com").unwrap();
    // ...
}

// GOOD: Use async or background threads
fn on_init(&mut self, ctx: &mut PluginContext) -> Result<(), String> {
    // Spawn background task for network requests
    std::thread::spawn(|| {
        // Do work asynchronously
    });
    Ok(())
}
```

### 3. Resource Cleanup

Clean up resources in `on_deactivate`:

```rust
fn on_deactivate(&mut self, ctx: &mut PluginContext) -> Result<(), String> {
    // Close files, connections, etc.
    self.cleanup_resources();
    Ok(())
}
```

### 4. Configuration

Use the plugin config system:

```rust
#[derive(Serialize, Deserialize)]
struct MyConfig {
    enabled: bool,
    max_items: usize,
}

fn on_init(&mut self, ctx: &mut PluginContext) -> Result<(), String> {
    let config: MyConfig = ctx.get_plugin_config()
        .unwrap_or(MyConfig {
            enabled: true,
            max_items: 100,
        });

    self.enabled = config.enabled;
    Ok(())
}
```

### 5. Testing

Write tests for your plugin:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plugin_init() {
        let mut plugin = MyPlugin::default();
        let mut ctx = PluginContext::new_test();

        assert!(plugin.on_init(&mut ctx).is_ok());
    }

    #[test]
    fn test_command_processing() {
        let mut plugin = MyPlugin::default();
        let mut ctx = PluginContext::new_test();

        let action = plugin.on_input(&mut ctx, "/mycommand\n").unwrap();
        assert_eq!(action, Action::Consume);
    }
}
```

## Publishing

### 1. Build Release

```bash
cargo build --release
```

### 2. Package Plugin

Create a `plugin.toml`:

```toml
[plugin]
name = "my-plugin"
version = "0.1.0"
description = "My awesome plugin"
author = "Your Name <your.email@example.com>"
license = "MIT"
repository = "https://github.com/yourusername/my-plugin"

[dependencies]
scarab-plugin-api = "0.1"

[capabilities]
hooks = ["on_input", "on_output"]
network = false
filesystem = false
```

### 3. Test Locally

```bash
scarab plugin install ./target/release/libmy_plugin.so
```

### 4. Publish

```bash
scarab plugin publish
```

## Debugging

### Enable Debug Logging

```bash
RUST_LOG=debug scarab
```

### Use Plugin Logging

```rust
ctx.log_debug(&format!("Debug info: {:?}", data));
```

### Attach Debugger

```bash
# Build with debug symbols
cargo build --release --profile release-with-debug

# Run with debugger
lldb -- scarab
```

## Resources

- [Plugin API Reference](../api/plugin-api.md)
- [Example Plugins](https://github.com/scarab-terminal/plugins)
- [Discord Community](https://discord.gg/scarab)

## Getting Help

- [GitHub Discussions](https://github.com/yourusername/scarab/discussions)
- [Plugin Development Issues](https://github.com/yourusername/scarab/issues?q=is%3Aissue+is%3Aopen+label%3Aplugin)
