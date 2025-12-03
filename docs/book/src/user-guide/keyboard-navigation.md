# Keyboard Navigation

Master keyboard-driven navigation in Scarab for maximum efficiency.

## Quick Links

For complete navigation documentation, see:
- [Navigation User Guide](../../../navigation/user-guide.md) - Complete navigation guide
- [Navigation Documentation](../../../navigation.md) - Navigation architecture

## Overview

Scarab provides Vimium-style keyboard navigation throughout the terminal UI, allowing you to navigate without touching the mouse.

## Focus Navigation

### Moving Between Panes

- **Tab** - Move focus to next pane
- **Shift+Tab** - Move focus to previous pane
- **Ctrl+h/j/k/l** - Move focus in direction (Vim-style)
- **Ctrl+Arrow Keys** - Move focus in direction

### Focus Indicators

The currently focused pane is highlighted with:
- Colored border (configurable)
- Visual feedback on focus change

## Scrollback Navigation

Navigate scrollback history with keyboard:

- **Shift+PgUp** - Scroll up one page
- **Shift+PgDn** - Scroll down one page
- **Shift+Home** - Scroll to top of scrollback
- **Shift+End** - Scroll to bottom (live output)
- **Ctrl+Shift+Up** - Scroll up one line
- **Ctrl+Shift+Down** - Scroll down one line

## Command Navigation

### Command Palette

Press **Ctrl+Shift+P** to open the command palette, then:

- Type to filter commands
- **Up/Down** arrows to navigate
- **Enter** to execute
- **Escape** to close

### Link Hints

Press **Ctrl+Shift+O** to activate link hints:

1. All clickable elements show keyboard hints
2. Type the hint characters to select
3. Press **Enter** to open the link
4. Press **Escape** to cancel

Supported elements:
- URLs (http://, https://)
- File paths
- Email addresses
- Custom patterns (via plugins)

## Prompt Navigation

Scarab supports OSC 133 prompt markers for semantic navigation:

- **Ctrl+Shift+Z** - Jump to previous prompt
- **Ctrl+Shift+X** - Jump to next prompt
- **Ctrl+Shift+C** - Copy command at current prompt

## Configuration

Customize navigation keybindings in `~/.config/scarab/config.toml`:

```toml
[keybindings.navigation]
focus_next = "Tab"
focus_prev = "Shift+Tab"
focus_up = "Ctrl+k"
focus_down = "Ctrl+j"
focus_left = "Ctrl+h"
focus_right = "Ctrl+l"
```

## See Also

- [Link Hints](./link-hints.md) - Detailed link hints guide
- [Keybindings Reference](../reference/keybindings.md) - All keybindings
- [Navigation Developer Guide](../developer-guide/navigation.md) - Navigation internals
