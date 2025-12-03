# Link Hints

Open URLs and files with keyboard shortcuts using Scarab's link hints feature.

## Quick Links

For complete navigation documentation, see:
- [Navigation User Guide](../../../navigation/user-guide.md) - Complete navigation guide

## Overview

Link hints provide Vimium-style keyboard navigation for clickable elements in your terminal. No mouse required!

## Basic Usage

1. Press **Ctrl+Shift+O** to activate link hints
2. Hint labels appear over all clickable elements
3. Type the hint characters (e.g., "ab")
4. Press **Enter** to open the selected link
5. Press **Escape** to cancel

## Supported Elements

Link hints work with:

### URLs
- `http://` and `https://` URLs
- Example: `https://github.com/raibid-labs/scarab`

### File Paths
- Absolute paths: `/home/user/file.txt`
- Relative paths: `./src/main.rs`
- Paths with line numbers: `file.rs:42`

### Email Addresses
- Standard email format: `user@example.com`

### Custom Patterns
Plugins can register custom patterns:
- IP addresses
- Git commit hashes
- Custom URL schemes

## Configuration

### Customizing Hint Characters

Edit `~/.config/scarab/config.toml`:

```toml
[navigation.link_hints]
# Characters used for hint labels (avoid ambiguous chars like 0/O, 1/l)
hint_chars = "asdfjkl"

# Highlight color for hints
highlight_color = "#ff0000"

# Background color for hint labels
label_bg = "#000000"
label_fg = "#ffffff"
```

### Custom Patterns

Add custom patterns via plugins:

```fsharp
// ~/.config/scarab/plugins/custom-hints.fsx
open Scarab.PluginApi

let metadata = {
    Name = "custom-hints"
    Version = "1.0.0"
    Description = "Custom link hint patterns"
    // ...
}

let registerPatterns (ctx: PluginContext) =
    // Add pattern for Docker container IDs
    ctx.RegisterHintPattern(@"[0-9a-f]{12}", fun id ->
        sprintf "docker logs %s" id
    )

    // Add pattern for Jira tickets
    ctx.RegisterHintPattern(@"PROJ-\d+", fun ticket ->
        sprintf "https://jira.company.com/browse/%s" ticket
    )
```

## Keyboard Shortcuts

| Key | Action |
|-----|--------|
| **Ctrl+Shift+O** | Activate link hints |
| **a-z** | Type hint characters |
| **Enter** | Open selected link |
| **Escape** | Cancel hint mode |
| **Backspace** | Remove last hint character |

## Actions

When a hint is activated:

- **URLs**: Open in default browser
- **Files**: Open in `$EDITOR` or default application
- **Emails**: Open in default email client
- **Custom**: Execute plugin-defined action

## Performance

Link hints are optimized for large terminal buffers:
- Hints generated on-demand
- Viewport culling for off-screen elements
- Efficient regex matching

Typical performance:
- < 50ms hint generation for 10,000 lines
- < 10ms per keystroke in hint mode

## See Also

- [Keyboard Navigation](./keyboard-navigation.md) - General keyboard navigation
- [Navigation Developer Guide](../developer-guide/navigation.md) - Navigation internals
- [Plugin Development](../developer-guide/plugins.md) - Create custom hint patterns
