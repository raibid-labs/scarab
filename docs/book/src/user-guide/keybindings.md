# Keybindings

Scarab provides customizable keybindings for all operations.

## Default Keybindings

### Tab Management

| Action | Default Keybinding |
|--------|-------------------|
| New Tab | `Ctrl+Shift+T` |
| Close Tab | `Ctrl+Shift+W` |
| Next Tab | `Ctrl+Tab` |
| Previous Tab | `Ctrl+Shift+Tab` |

### Pane Management

| Action | Default Keybinding |
|--------|-------------------|
| Split Horizontal | `Ctrl+Shift+H` |
| Split Vertical | `Ctrl+Shift+V` |
| Close Pane | `Ctrl+Shift+Q` |
| Focus Up | `Ctrl+Shift+Up` |
| Focus Down | `Ctrl+Shift+Down` |
| Focus Left | `Ctrl+Shift+Left` |
| Focus Right | `Ctrl+Shift+Right` |

### General

| Action | Default Keybinding |
|--------|-------------------|
| Copy | `Ctrl+Shift+C` |
| Paste | `Ctrl+Shift+V` |
| Search | `Ctrl+Shift+F` |

## Customizing Keybindings

Edit your `~/.config/scarab/config.toml`:

```toml
[keybindings]
new_tab = "Ctrl+T"
split_horizontal = "Alt+Shift+H"
focus_up = "Alt+Up"
```

For complete keybinding configuration options, see the [Configuration Guide](./configuration.md).

## Keybinding Syntax

Keybindings use the following modifiers:
- `Ctrl` - Control key
- `Shift` - Shift key
- `Alt` - Alt key
- `Super` - Super/Windows/Command key

Multiple modifiers can be combined: `Ctrl+Shift+T`
