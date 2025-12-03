# API Documentation

Scarab provides APIs for plugin development and integration.

## Plugin API

Defined in `crates/scarab-plugin-api/src/lib.rs`.

For plugin development guide, see [Plugin Development](../developer-guide/plugins.md).

## Core Traits

### Plugin Trait

Base trait for all plugins:

```rust
pub trait Plugin: Send + Sync {
    /// Initialize the plugin
    fn init(&mut self) -> Result<()>;

    /// Plugin name
    fn name(&self) -> &str;

    /// Plugin version
    fn version(&self) -> &str;

    /// Clean up resources
    fn shutdown(&mut self) -> Result<()>;
}
```

### OutputFilter Trait

Process terminal output (daemon plugins):

```rust
pub trait OutputFilter: Plugin {
    /// Filter output before rendering
    /// Returns modified output or None to drop
    fn filter(&mut self, data: &[u8]) -> Result<Option<Vec<u8>>>;
}
```

### InputHandler Trait

Process user input (client plugins):

```rust
pub trait InputHandler: Plugin {
    /// Handle key press
    /// Returns true if event was consumed
    fn on_key(&mut self, key: Key, mods: Modifiers) -> Result<bool>;

    /// Handle mouse event
    fn on_mouse(&mut self, event: MouseEvent) -> Result<bool>;
}
```

### UIExtension Trait

Custom UI components (client plugins):

```rust
pub trait UIExtension: Plugin {
    /// Render custom UI
    fn render(&mut self, ctx: &mut UIContext) -> Result<()>;

    /// Update state
    fn update(&mut self, delta: f32) -> Result<()>;
}
```

## Data Types

### Key

```rust
pub enum Key {
    Char(char),
    Up, Down, Left, Right,
    Enter, Escape, Backspace, Delete, Tab,
    Home, End, PageUp, PageDown,
    F1, F2, F3, F4, F5, F6, F7, F8, F9, F10, F11, F12,
}
```

### Modifiers

```rust
pub struct Modifiers {
    pub ctrl: bool,
    pub shift: bool,
    pub alt: bool,
    pub super_key: bool,
}
```

### MouseEvent

```rust
pub struct MouseEvent {
    pub x: u32,
    pub y: u32,
    pub button: MouseButton,
    pub action: MouseAction,
}

pub enum MouseButton {
    Left, Right, Middle,
}

pub enum MouseAction {
    Press, Release, Move,
}
```

## Plugin Context

### UIContext

Provided to UI plugins:

```rust
pub struct UIContext {
    /// Get terminal dimensions
    pub fn dimensions(&self) -> (u32, u32);

    /// Draw text at position
    pub fn draw_text(&mut self, x: u32, y: u32, text: &str);

    /// Draw rectangle
    pub fn draw_rect(&mut self, x: u32, y: u32, w: u32, h: u32, color: Color);

    /// Get focused pane ID
    pub fn focused_pane(&self) -> Option<PaneId>;
}
```

## Fusabi FFI

Fusabi plugins can call Rust functions:

```fsharp
// Fusabi (.fsx) example
extern fn terminal_write: string -> unit

let onKeyPress key =
    if key = "Ctrl+L" then
        terminal_write "clear\n"
        true
    else
        false
```

Register Rust functions:

```rust
// Rust side
#[no_mangle]
pub extern "C" fn terminal_write(data: *const u8, len: usize) {
    let slice = unsafe { std::slice::from_raw_parts(data, len) };
    let text = std::str::from_utf8(slice).unwrap();
    // Write to terminal...
}
```

## Plugin Lifecycle

1. **Load**: Plugin library loaded by daemon/client
2. **Init**: `Plugin::init()` called
3. **Active**: Plugin receives events
4. **Shutdown**: `Plugin::shutdown()` called on exit

## Error Handling

All plugin methods return `Result<T>`:

```rust
pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
```

Plugins should:
- Return `Err` for recoverable errors
- Log errors appropriately
- Never panic (may crash daemon/client)

## Plugin Configuration

Plugins can access configuration:

```rust
pub trait Plugin {
    fn configure(&mut self, config: &PluginConfig) -> Result<()>;
}

pub struct PluginConfig {
    /// Get string value
    pub fn get_string(&self, key: &str) -> Option<&str>;

    /// Get integer value
    pub fn get_int(&self, key: &str) -> Option<i64>;

    /// Get boolean value
    pub fn get_bool(&self, key: &str) -> Option<bool>;
}
```

## Example Plugin

### Minimal Plugin (Rust)

```rust
use scarab_plugin_api::{Plugin, Result};

pub struct HelloPlugin;

impl Plugin for HelloPlugin {
    fn init(&mut self) -> Result<()> {
        println!("Hello plugin loaded!");
        Ok(())
    }

    fn name(&self) -> &str {
        "hello"
    }

    fn version(&self) -> &str {
        "1.0.0"
    }

    fn shutdown(&mut self) -> Result<()> {
        println!("Goodbye!");
        Ok(())
    }
}
```

### Output Filter (Fusabi)

```fsharp
// syntax-highlight.fsx
module SyntaxHighlight

let colorize text =
    // Simple syntax highlighting
    text
    |> String.replace "TODO" "\x1b[31mTODO\x1b[0m"
    |> String.replace "DONE" "\x1b[32mDONE\x1b[0m"

let filter data =
    let text = System.Text.Encoding.UTF8.GetString(data)
    let colored = colorize text
    System.Text.Encoding.UTF8.GetBytes(colored)
```

## API Stability

- **Stable**: Core traits, basic types
- **Unstable**: UIContext, advanced features
- **Experimental**: Fusabi FFI

Breaking changes will bump major version.

## Documentation Generation

Generate API docs:

```bash
cargo doc --workspace --no-deps --open
```

## Related Documentation

- [Plugin Development](../developer-guide/plugins.md)
- [Configuration Schema](./config-schema.md)
- [IPC Protocol](./ipc-protocol.md)
