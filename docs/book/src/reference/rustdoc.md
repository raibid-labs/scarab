# Rust API Documentation (rustdoc)

Complete API documentation for Scarab's Rust crates.

## Generating API Documentation

Generate comprehensive API documentation for all Scarab crates:

```bash
# Generate documentation for the entire workspace
cargo doc --workspace --no-deps --open

# Generate documentation for a specific crate
cargo doc -p scarab-daemon --no-deps --open
cargo doc -p scarab-client --no-deps --open
cargo doc -p scarab-protocol --no-deps --open
cargo doc -p scarab-plugin-api --no-deps --open
cargo doc -p scarab-config --no-deps --open
```

## Online Documentation

When deployed, API documentation will be available at:
```
https://raibid-labs.github.io/scarab/api/
```

## Crate Documentation

### scarab-daemon

Headless terminal server that owns PTY processes and maintains terminal state.

**Key modules:**
- `pty` - PTY process management
- `vte` - VTE parser integration
- `ipc` - IPC server and shared memory writer
- `session` - Session management
- `plugin` - Plugin loader and Fusabi VM integration

**Generate docs:**
```bash
cargo doc -p scarab-daemon --no-deps --open
```

### scarab-client

Bevy-based GUI client that renders terminal via shared memory.

**Key modules:**
- `render` - Text rendering with cosmic-text
- `input` - Keyboard and mouse input handling
- `ui` - Bevy UI components
- `ipc` - IPC client and shared memory reader
- `plugin` - Plugin loader and Fusabi script interpreter
- `navigation` - ECS-based navigation system

**Generate docs:**
```bash
cargo doc -p scarab-client --no-deps --open
```

### scarab-protocol

IPC protocol definitions and shared memory layout.

**Key types:**
- `SharedState` - Main shared memory structure
- `Cell` - Terminal cell representation
- `ControlMessage` - IPC control messages
- `ResponseMessage` - IPC response messages

**Critical:**
- All shared memory structs are `#[repr(C)]`
- Uses `bytemuck::{Pod, Zeroable}` for safe zero-copy

**Generate docs:**
```bash
cargo doc -p scarab-protocol --no-deps --open
```

### scarab-plugin-api

Shared plugin traits and types for both daemon and client plugins.

**Key traits:**
- `Plugin` - Base plugin trait
- `OutputFilter` - Daemon output filtering
- `InputHandler` - Client input handling
- `UIExtension` - Client UI extensions

**Generate docs:**
```bash
cargo doc -p scarab-plugin-api --no-deps --open
```

### scarab-config

Configuration management with TOML and Fusabi support.

**Key types:**
- `Config` - Main configuration structure
- `Theme` - Color theme configuration
- `Keybindings` - Keyboard shortcut configuration
- `PluginConfig` - Plugin-specific configuration

**Generate docs:**
```bash
cargo doc -p scarab-config --no-deps --open
```

## Documentation Standards

### Writing Documentation

When contributing to Scarab, follow these documentation standards:

1. **Module-level docs** - Describe the module's purpose and main types
2. **Type docs** - Document all public types with examples
3. **Function docs** - Document parameters, return values, and errors
4. **Examples** - Include runnable examples where possible
5. **Safety** - Document all `unsafe` code with safety requirements

### Example Documentation

```rust
/// Manages PTY processes for terminal emulation.
///
/// The `PtyManager` creates and maintains PTY processes, handling
/// input/output and resize events.
///
/// # Examples
///
/// ```
/// use scarab_daemon::pty::PtyManager;
///
/// let mut manager = PtyManager::new();
/// let pty = manager.spawn("/bin/bash", &[]).unwrap();
/// ```
///
/// # Safety
///
/// PTY file descriptors must be properly closed to avoid resource leaks.
pub struct PtyManager {
    // ...
}
```

## CI/CD Integration

API documentation is automatically generated and deployed on:

1. **Pull requests** - Docs are generated to verify no warnings
2. **Main branch** - Docs are built and deployed to GitHub Pages
3. **Releases** - Tagged documentation for each release

### Checking Documentation

Before submitting a PR, verify documentation builds without warnings:

```bash
# Check for documentation warnings
cargo doc --workspace --no-deps 2>&1 | grep warning

# Generate documentation with all lints
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps
```

## Documentation Lints

Scarab enforces documentation quality:

```rust
#![warn(missing_docs)]
#![warn(rustdoc::broken_intra_doc_links)]
#![warn(rustdoc::invalid_codeblock_attributes)]
```

## See Also

- [API Documentation (manual)](./api.md) - Hand-written API guide
- [Plugin Traits](./plugin-traits.md) - Plugin trait reference
- [Fusabi FFI](./fusabi-ffi.md) - Fusabi interop documentation
- [Contributing](../developer-guide/contributing.md) - Contribution guidelines
