# ECS-Safe Host Bindings for Fusabi Plugins

This document describes the ECS-safe host bindings API that allows Fusabi plugins to interact with Scarab's Bevy ECS without direct World mutation.

## Overview

Fusabi plugins run in a sandboxed environment and cannot directly access Bevy's World or ECS components. Instead, they communicate through the **Host Bindings API**, which provides:

- **Safety**: All operations go through validated message passing
- **Sandboxing**: Per-plugin capability flags and resource quotas
- **Rate Limiting**: Protection against runaway plugins
- **Validation**: Bounds checking and URL/path sanitization

## Quick Start

```fsharp
module MyPlugin

open Scarab.Host

[<OnLoad>]
let onLoad (ctx: PluginContext) =
    // Get host bindings with default limits
    let host = Host.create ctx
    
    // Register a focusable region
    let focusableId = host.registerFocusable {
        X = 10us
        Y = 5us
        Width = 20us
        Height = 1us
        Label = "GitHub"
        Action = OpenUrl "https://github.com"
    }
    
    // Enter hint mode
    host.enterHintMode ()
```

## API Reference

### HostBindings

The main entry point for ECS-safe operations.

#### Creation

```rust
// With default limits and capabilities
let bindings = HostBindings::with_defaults();

// With custom configuration
let bindings = HostBindings::new(
    HostBindingLimits {
        max_focusables: 100,
        rate_limit: 20,
        ..Default::default()
    },
    PluginNavCapabilities::default(),
);
```

#### Navigation Methods

| Method | Description | Capability Required |
|--------|-------------|---------------------|
| `enter_hint_mode()` | Trigger hint mode UI | `can_enter_hint_mode` |
| `exit_nav_mode()` | Exit navigation mode | (none) |
| `register_focusable()` | Register a navigation target | `can_register_focusables` |
| `unregister_focusable()` | Remove a navigation target | (none) |
| `prompt_jump()` | Navigate to previous/next prompt | (none) |

#### UI Methods

| Method | Description | Quota |
|--------|-------------|-------|
| `spawn_overlay()` | Create floating overlay at position | `max_overlays` (default: 10) |
| `remove_overlay()` | Remove overlay by ID | (none) |
| `add_status_item()` | Add status bar item | `max_status_items` (default: 5) |
| `remove_status_item()` | Remove status item by ID | (none) |

#### Configuration Methods

| Method | Description |
|--------|-------------|
| `set_nav_style()` | Set hint character style (Vimium, Numeric, etc.) |
| `set_nav_keymap()` | Set navigation keybindings |
| `resource_usage()` | Get current resource usage snapshot |

### PluginFocusable

Represents a navigable region in the terminal.

```rust
PluginFocusable {
    x: u16,           // Column position (0-based)
    y: u16,           // Row position (0-based)
    width: u16,       // Width in cells
    height: u16,      // Height in cells
    label: String,    // Hint label text
    action: PluginFocusableAction,
}
```

### PluginFocusableAction

Action to perform when a focusable is activated:

| Variant | Description | Validation |
|---------|-------------|------------|
| `OpenUrl(url)` | Open URL in browser | Allowed: `http://`, `https://`, `file://` |
| `OpenFile(path)` | Open file in editor | No `..` traversal, no sensitive paths |
| `Custom(name)` | Plugin callback | Plugin receives event |

## Safety Constraints

### Resource Quotas

Each plugin has limits on resources it can create:

| Resource | Default Limit | Description |
|----------|---------------|-------------|
| Focusables | 50 | Navigation targets |
| Overlays | 10 | UI overlay surfaces |
| Status Items | 5 | Status bar entries |

When a quota is exceeded, the API returns `PluginError::QuotaExceeded`.

### Rate Limiting

All API calls are rate-limited to prevent abuse:

- **Default**: 10 actions per second
- Exceeding the limit returns `PluginError::RateLimitExceeded`
- The window resets every second

### Capability Flags

Plugins must have appropriate capabilities enabled:

```rust
PluginNavCapabilities {
    can_enter_hint_mode: bool,     // Allow triggering hint mode
    can_register_focusables: bool, // Allow creating focusables
    max_focusables: usize,         // Per-plugin focusable limit
    can_trigger_actions: bool,     // Allow programmatic actions
}
```

### Bounds Validation

All coordinates are validated:

- Maximum coordinate: 1000
- Zero dimensions are rejected
- Labels must be 1-256 characters

### URL/Path Sanitization

| Check | Blocked |
|-------|---------|
| Dangerous protocols | `javascript:`, `data:`, `vbscript:` |
| Path traversal | `../` patterns |
| Sensitive paths | `/etc/passwd`, `.ssh/`, `/proc/` |

## Navigation Styles

Configure hint appearance via `NavStyle`:

```rust
NavStyle::Vimium          // "sadfjklewcmpgh" (default)
NavStyle::VimiumUppercase // "SADFJKLEWCMPGH"
NavStyle::Numeric         // "1234567890"
NavStyle::HomeRow         // "asdfghjkl"
NavStyle::Custom("abc")   // Custom characters
```

## Navigation Keymaps

Configure keybindings via `NavKeymap`:

```rust
NavKeymap::Default  // f=hints, Esc=cancel, Enter=confirm
NavKeymap::Vim      // Vim-style bindings
NavKeymap::Emacs    // Emacs-style bindings
NavKeymap::Custom(vec![
    ("f".into(), "enter_hints".into()),
    ("q".into(), "cancel".into()),
])
```

## Error Handling

All methods return `Result<T, PluginError>`. Common errors:

| Error | Cause | Solution |
|-------|-------|----------|
| `CapabilityDenied` | Missing capability | Request capability in manifest |
| `QuotaExceeded` | Too many resources | Unregister unused resources |
| `RateLimitExceeded` | Too many API calls | Add delays, batch operations |
| `ValidationError` | Invalid input | Check coordinates, URLs |

## Best Practices

1. **Unregister focusables** when no longer needed to stay under quota
2. **Batch operations** where possible to avoid rate limits
3. **Use appropriate capabilities** - request only what you need
4. **Handle errors gracefully** - log and degrade gracefully
5. **Prefer safe URLs** - use `https://` over `http://`

## Integration with Fusabi Config

Plugins can specify navigation preferences in their Fusabi config:

```fsharp
// config.fsx
module Config

let navConfig = {
    Style = NavStyle.HomeRow
    Keymap = NavKeymap.Vim
    MaxFocusables = 25
}
```

## Fusabi Version Note

**Current Status (Issue #88):**

| Component | Pinned | Latest on crates.io |
|-----------|--------|---------------------|
| fusabi-vm | 0.17.0 | 0.21.0 |
| fusabi-frontend | 0.17.0 | 0.21.0 |
| bevy-fusabi | 0.1.4 | 0.1.4 |

Scarab uses Fusabi v0.17.0 due to `bevy-fusabi 0.1.4` requiring `^0.17.0`. Issue #88 requested v0.32.x which **does not exist** on crates.io (latest is v0.21.0).

**Upgrade Path Options:**
1. ✅ **Wait for bevy-fusabi update** - Recommended, no maintenance burden
2. ⚠️ **Fork bevy-fusabi** - Patch dependency, adds maintenance burden
3. ❌ **Remove bevy-fusabi** - Loses hot-reloadable config asset integration

**What Works Today:**
- All ECS-safe UI/nav bindings in `host_bindings.rs` are version-independent
- Fusabi script plugins (.fsx) work correctly
- Fusabi bytecode plugins (.fzb) work correctly
- Hot-reloading works via bevy-fusabi integration

**When to Upgrade:**
The version will auto-upgrade when bevy-fusabi releases a compatible update with fusabi-vm 0.21+.

## See Also

- [Navigation API Reference](./api-reference/navigation.md)
- [Plugin Development Guide](./README.md)
- [Security Considerations](./architecture/security.md)
