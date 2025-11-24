# Scarab Plugin API Reference

Complete API reference for the Scarab plugin system.

## Module Overview

- `scarab_plugin_api::plugin` - Core plugin trait and metadata
- `scarab_plugin_api::context` - Plugin execution context
- `scarab_plugin_api::config` - Configuration loading and discovery
- `scarab_plugin_api::error` - Error types
- `scarab_plugin_api::types` - Common types and enums

## Core Traits

### Plugin

The main trait that all plugins must implement.

```rust
#[async_trait]
pub trait Plugin: Send + Sync {
    fn metadata(&self) -> &PluginMetadata;
    
    /// Get list of commands provided by this plugin
    fn get_commands(&self) -> Vec<ModalItem> { Vec::new() }

    async fn on_load(&mut self, ctx: &mut PluginContext) -> Result<()>;
    async fn on_unload(&mut self) -> Result<()>;
    async fn on_output(&mut self, line: &str, ctx: &PluginContext) -> Result<Action>;
    async fn on_input(&mut self, input: &[u8], ctx: &PluginContext) -> Result<Action>;
    async fn on_pre_command(&mut self, command: &str, ctx: &PluginContext) -> Result<Action>;
    async fn on_post_command(&mut self, command: &str, exit_code: i32, ctx: &PluginContext) -> Result<()>;
    async fn on_resize(&mut self, cols: u16, rows: u16, ctx: &PluginContext) -> Result<()>;
    async fn on_attach(&mut self, client_id: u64, ctx: &PluginContext) -> Result<()>;
    async fn on_detach(&mut self, client_id: u64, ctx: &PluginContext) -> Result<()>;
    
    /// Hook called when a remote command is selected/triggered by the client
    async fn on_remote_command(&mut self, id: &str, ctx: &PluginContext) -> Result<()> { Ok(()) }
}
```

All methods except `metadata()` have default implementations that return `Ok(())`, `Ok(Action::Continue)`, or empty collections.

## Types

### PluginMetadata

Plugin identification and version information.

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
```

**Methods:**

```rust
impl PluginMetadata {
    pub fn new(
        name: impl Into<String>,
        version: impl Into<String>,
        description: impl Into<String>,
        author: impl Into<String>,
    ) -> Self;

    pub fn with_homepage(self, homepage: impl Into<String>) -> Self;
    pub fn with_api_version(self, version: impl Into<String>) -> Self;
    pub fn with_min_scarab_version(self, version: impl Into<String>) -> Self;
    pub fn is_compatible(&self, current_api_version: &str) -> bool;
}
```

**Example:**

```rust
let metadata = PluginMetadata::new(
    "my-plugin",
    "1.0.0",
    "Description of my plugin",
    "Author Name <email@example.com>",
)
.with_homepage("https://github.com/user/plugin")
.with_api_version("0.1.0");
```

### ModalItem

A command or item to be displayed in the Command Palette or other modals.

```rust
pub struct ModalItem {
    pub id: String,
    pub label: String,
    pub description: Option<String>,
}
```

### Action

Return value for hooks that can modify data.

```rust
pub enum Action {
    Continue,         // Pass to next plugin unchanged
    Stop,            // Stop processing, don't call remaining plugins
    Modify(Vec<u8>), // Modify data and continue to next plugin
}
```

**Methods:**

```rust
impl Action {
    pub fn is_modify(&self) -> bool;
    pub fn is_stop(&self) -> bool;
}
```

### HookType

Enumeration of all hook types.

```rust
pub enum HookType {
    PreOutput,
    PostInput,
    PreCommand,
    PostCommand,
    OnResize,
    OnAttach,
    OnDetach,
}
```

**Methods:**

```rust
impl HookType {
    pub fn all() -> &'static [HookType];
    pub fn name(&self) -> &'static str;
}
```

### Cell

Represents a single terminal grid cell.

```rust
pub struct Cell {
    pub c: char,
    pub fg: (u8, u8, u8),  // RGB foreground
    pub bg: (u8, u8, u8),  // RGB background
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
}
```

**Example:**

```rust
let red_cell = Cell {
    c: 'X',
    fg: (255, 0, 0),
    bg: (0, 0, 0),
    bold: true,
    italic: false,
    underline: false,
};
```

### PluginInfo

Runtime information about a loaded plugin.

```rust
pub struct PluginInfo {
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
    pub homepage: Option<String>,
    pub api_version: String,
    pub min_scarab_version: String,
    pub enabled: bool,
    pub failure_count: u32,
}
```

## Context

### PluginContext

Provides access to terminal state and services.

```rust
pub struct PluginContext {
    pub config: PluginConfigData,
    pub state: Arc<Mutex<SharedState>>,
    pub logger_name: String,
}
```

**Methods:**

```rust
impl PluginContext {
    // Terminal grid access
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

**Example:**

```rust
// Get terminal size
let (cols, rows) = ctx.get_size();

// Read cell at cursor position
let (x, y) = ctx.get_cursor();
if let Some(cell) = ctx.get_cell(x, y) {
    println!("Character at cursor: {}", cell.c);
}

// Store persistent data
ctx.set_data("last_command", "ls -la");

// Retrieve data
if let Some(cmd) = ctx.get_data("last_command") {
    ctx.log(LogLevel::Info, &format!("Last command: {}", cmd));
}
```

### SharedState

Terminal state accessible to plugins.

```rust
pub struct SharedState {
    pub cells: Vec<Cell>,
    pub cols: u16,
    pub rows: u16,
    pub cursor: (u16, u16),
    pub env: HashMap<String, String>,
    pub data: HashMap<String, String>,
}
```

**Methods:**

```rust
impl SharedState {
    pub fn new(cols: u16, rows: u16) -> Self;
    pub fn get_cell(&self, x: u16, y: u16) -> Option<Cell>;
    pub fn set_cell(&mut self, x: u16, y: u16, cell: Cell) -> bool;
    pub fn get_line(&self, y: u16) -> Option<String>;
}
```

### LogLevel

Logging levels.

```rust
pub enum LogLevel {
    Error,
    Warn,
    Info,
    Debug,
}
```

## Configuration

### PluginConfig

Configuration loaded from TOML file.

```rust
pub struct PluginConfig {
    pub name: String,
    pub path: PathBuf,
    pub enabled: bool,
    pub config: PluginConfigData,
}
```

**Methods:**

```rust
impl PluginConfig {
    pub fn from_file(path: impl AsRef<Path>) -> Result<Vec<Self>>;
    pub fn expanded_path(&self) -> PathBuf;
}
```

### PluginConfigData

Plugin-specific configuration data.

```rust
pub struct PluginConfigData {
    #[serde(flatten)]
    pub data: HashMap<String, toml::Value>,
}
```

**Methods:**

```rust
impl PluginConfigData {
    pub fn get<T: for<'de> Deserialize<'de>>(&self, key: &str) -> Result<T>;
    pub fn get_opt<T: for<'de> Deserialize<'de>>(&self, key: &str) -> Option<T>;
}
```

**Example:**

```rust
// Get required config
let threshold: u32 = ctx.config.get("threshold")?;

// Get optional config with default
let enabled: bool = ctx.config.get_opt("enabled").unwrap_or(true);

// Get complex config
#[derive(Deserialize)]
struct Colors {
    fg: (u8, u8, u8),
    bg: (u8, u8, u8),
}
let colors: Colors = ctx.config.get("colors")?;
```

### PluginDiscovery

Plugin discovery system.

```rust
pub struct PluginDiscovery {
    search_paths: Vec<PathBuf>,
}
```

**Methods:**

```rust
impl PluginDiscovery {
    pub fn new() -> Self;
    pub fn default_plugin_dir() -> PathBuf;
    pub fn default_config_path() -> PathBuf;
    pub fn add_path(&mut self, path: impl Into<PathBuf>);
    pub fn discover(&self) -> Vec<PathBuf>;
    pub fn load_config(&self, path: Option<&Path>) -> Result<Vec<PluginConfig>>;
    pub fn ensure_plugin_dir() -> Result<PathBuf>;
    pub fn create_default_config() -> Result<PathBuf>;
}
```

**Example:**

```rust
let mut discovery = PluginDiscovery::new();
discovery.add_path("/custom/plugin/path");

// Discover all plugins
let plugins = discovery.discover();

// Load configuration
let configs = discovery.load_config(None)?;
```

## Error Handling

### PluginError

All plugin errors.

```rust
pub enum PluginError {
    LoadError(String),
    VersionIncompatible { required: String, actual: String },
    Timeout(u64),
    Panic(String),
    Disabled,
    ConfigError(String),
    IoError(std::io::Error),
    TomlError(toml::de::Error),
    InvalidMetadata(String),
    NotFound(String),
    Other(anyhow::Error),
}
```

### Result

Plugin result type.

```rust
pub type Result<T> = std::result::Result<T, PluginError>;
```

## Constants

### API_VERSION

Current plugin API version.

```rust
pub const API_VERSION: &str = "0.1.0";
```

Use for version compatibility checking:

```rust
if !metadata.is_compatible(scarab_plugin_api::API_VERSION) {
    return Err(PluginError::VersionIncompatible {
        required: scarab_plugin_api::API_VERSION.to_string(),
        actual: metadata.api_version.clone(),
    });
}
```

## Performance Guidelines

### Hook Execution Constraints

- **Timeout**: 1 second (default, configurable)
- **Max CPU overhead**: <1% per plugin
- **Memory**: Reasonable limits enforced

### Optimization Tips

1. **Minimize allocations** in hot paths
2. **Cache** expensive computations
3. **Use async I/O** for blocking operations
4. **Profile** your hooks regularly

```rust
// ❌ Bad: Allocates every call
async fn on_output(&mut self, line: &str, ctx: &PluginContext) -> Result<Action> {
    let keywords = vec!["ERROR", "WARN", "INFO"];  // Allocates!
    // ...
}

// ✅ Good: Allocate once in struct
struct MyPlugin {
    keywords: Vec<String>,  // Allocated in new()
}

async fn on_output(&mut self, line: &str, ctx: &PluginContext) -> Result<Action> {
    // Use self.keywords
}
```

## Thread Safety

All plugins must be `Send + Sync`:

- Plugin methods may be called from any thread
- Use `Arc<Mutex<T>>` for shared mutable state
- Consider using `parking_lot::Mutex` for better performance

```rust
use parking_lot::Mutex;
use std::sync::Arc;

pub struct MyPlugin {
    state: Arc<Mutex<PluginState>>,
}
```

## Async Context

All hook methods are async:

- Use `async/await` for I/O operations
- Don't block the executor
- Consider spawning background tasks for heavy work

```rust
async fn on_output(&mut self, line: &str, ctx: &PluginContext) -> Result<Action> {
    // Heavy processing in background
    let line = line.to_string();
    tokio::spawn(async move {
        // Process line without blocking
    });

    Ok(Action::Continue)
}
```

## Versioning

### Semantic Versioning

Plugins should follow semver:

- **Major**: Breaking API changes
- **Minor**: New features, backward compatible
- **Patch**: Bug fixes

### API Version Compatibility

API versions are compatible if:

- Major versions match
- Plugin minor version ≤ current minor version

```rust
// Plugin built for 0.1.0 works with Scarab 0.1.x and 0.2.x
// Plugin built for 0.2.0 works with Scarab 0.2.x only
// Plugin built for 1.0.0 works with Scarab 1.x.x only
```

## Examples

### Minimal Plugin

```rust
use scarab_plugin_api::*;
use async_trait::async_trait;

pub struct MinimalPlugin {
    metadata: PluginMetadata,
}

impl MinimalPlugin {
    pub fn new() -> Self {
        Self {
            metadata: PluginMetadata::new(
                "minimal",
                "1.0.0",
                "Minimal plugin example",
                "Author",
            ),
        }
    }
}

#[async_trait]
impl Plugin for MinimalPlugin {
    fn metadata(&self) -> &PluginMetadata {
        &self.metadata
    }
}
```

### Output Modifier

```rust
async fn on_output(&mut self, line: &str, ctx: &PluginContext) -> Result<Action> {
    if line.starts_with("ERROR") {
        // Add color
        let colored = format!("\x1b[31m{}\x1b[0m", line);
        Ok(Action::Modify(colored.into_bytes()))
    } else {
        Ok(Action::Continue)
    }
}
```

### Input Handler

```rust
async fn on_input(&mut self, input: &[u8], ctx: &PluginContext) -> Result<Action> {
    // Ctrl+X
    if input == b"\x18" {
        ctx.notify("Ctrl+X pressed");
        Ok(Action::Stop)  // Don't send to terminal
    } else {
        Ok(Action::Continue)
    }
}
```

### Stateful Plugin

```rust
pub struct StatefulPlugin {
    metadata: PluginMetadata,
    line_count: u64,
    error_count: u32,
}

async fn on_output(&mut self, line: &str, ctx: &PluginContext) -> Result<Action> {
    self.line_count += 1;

    if line.contains("ERROR") {
        self.error_count += 1;
    }

    if self.line_count % 100 == 0 {
        ctx.log(
            LogLevel::Info,
            &format!("Processed {} lines, {} errors", self.line_count, self.error_count),
        );
    }

    Ok(Action::Continue)
}
```

## See Also

- [Plugin Development Guide](plugin-development-guide.md)
- [Example Plugins](../examples/)
- [Scarab Documentation](README.md)
