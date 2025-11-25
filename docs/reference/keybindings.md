# Keybindings Reference

Complete keyboard shortcut reference for Scarab Terminal.

## Platform Conventions

Scarab follows platform-native keyboard conventions:

| Modifier | macOS | Linux/Windows |
|----------|-------|---------------|
| Primary | `Cmd` | `Ctrl` |
| Secondary | `Ctrl` | `Alt` |
| Tertiary | `Option` | `Alt` |
| System | `Cmd` | `Super`/`Win` |

**Note**: This document uses Linux/Windows notation by default. macOS users should substitute `Cmd` for `Ctrl` in most shortcuts.

---

## Default Keybindings

### Core Editing

| Action | macOS | Linux/Windows | Customizable | Description |
|--------|-------|---------------|--------------|-------------|
| Copy | `Cmd+C` | `Ctrl+Shift+C` | ✅ | Copy selected text to clipboard |
| Paste | `Cmd+V` | `Ctrl+Shift+V` | ✅ | Paste from clipboard |
| Select All | `Cmd+A` | `Ctrl+Shift+A` | ✅ | Select all terminal content |
| Clear Selection | `Escape` | `Escape` | ✅ | Clear current selection |

**Note**: `Ctrl+C` (without Shift) sends SIGINT to running process, not copy.

---

### Navigation & Scrolling

| Action | macOS | Linux/Windows | Customizable | Description |
|--------|-------|---------------|--------------|-------------|
| Scroll Line Up | `Shift+Up` | `Shift+Up` | ✅ | Scroll up one line |
| Scroll Line Down | `Shift+Down` | `Shift+Down` | ✅ | Scroll down one line |
| Scroll Page Up | `Shift+PageUp` | `Shift+PageUp` | ✅ | Scroll up one page |
| Scroll Page Down | `Shift+PageDown` | `Shift+PageDown` | ✅ | Scroll down one page |
| Scroll to Top | `Cmd+Home` | `Shift+Home` | ✅ | Jump to scrollback start |
| Scroll to Bottom | `Cmd+End` | `Shift+End` | ✅ | Jump to current line |
| Scroll Half Page Up | `Ctrl+U` | `Ctrl+U` | ✅ | Scroll up half page (vim-style) |
| Scroll Half Page Down | `Ctrl+D` | `Ctrl+D` | ✅ | Scroll down half page (vim-style) |

---

### Search

| Action | macOS | Linux/Windows | Customizable | Description |
|--------|-------|---------------|--------------|-------------|
| Open Search | `Cmd+F` | `Ctrl+F` | ✅ | Open search overlay |
| Find Next | `Enter` | `Enter` | ✅ | Jump to next match |
| Find Previous | `Shift+Enter` | `Shift+Enter` | ✅ | Jump to previous match |
| Find Next (Alt) | `Cmd+G` | `Ctrl+G` | ✅ | Alternative next match |
| Find Previous (Alt) | `Cmd+Shift+G` | `Ctrl+Shift+G` | ✅ | Alternative previous match |
| Close Search | `Escape` | `Escape` | ✅ | Close search overlay |
| Toggle Regex | `Cmd+R` | `Ctrl+R` | ✅ | Toggle regex mode |
| Toggle Case Sensitive | `Cmd+Shift+C` | `Ctrl+Shift+C` | ✅ | Toggle case sensitivity |

**Search Features**:
- Incremental search (updates as you type)
- Regex support with capture groups
- Case-sensitive/insensitive modes
- Wraps around at buffer boundaries
- Highlights all matches

---

### View & Display

| Action | macOS | Linux/Windows | Customizable | Description |
|--------|-------|---------------|--------------|-------------|
| Increase Font Size | `Cmd++` | `Ctrl++` | ✅ | Make text larger |
| Decrease Font Size | `Cmd+-` | `Ctrl+-` | ✅ | Make text smaller |
| Reset Font Size | `Cmd+0` | `Ctrl+0` | ✅ | Restore default size |
| Toggle Fullscreen | `Cmd+Enter` | `F11` | ✅ | Enter/exit fullscreen |
| Toggle Tab Bar | `Cmd+Shift+T` | `Ctrl+Shift+T` | ✅ | Show/hide tab bar |
| Zoom In | `Cmd+=` | `Ctrl+=` | ✅ | Zoom interface |
| Zoom Out | `Cmd+_` | `Ctrl+_` | ✅ | Zoom out interface |

---

### Tabs & Windows

| Action | macOS | Linux/Windows | Customizable | Description |
|--------|-------|---------------|--------------|-------------|
| New Tab | `Cmd+T` | `Ctrl+Shift+T` | ✅ | Create new tab |
| Close Tab | `Cmd+W` | `Ctrl+Shift+W` | ✅ | Close current tab |
| Next Tab | `Cmd+]` | `Ctrl+Tab` | ✅ | Switch to next tab |
| Previous Tab | `Cmd+[` | `Ctrl+Shift+Tab` | ✅ | Switch to previous tab |
| Tab 1-9 | `Cmd+1-9` | `Ctrl+1-9` | ✅ | Jump to tab by number |
| Last Tab | `Cmd+0` | `Ctrl+0` | ✅ | Jump to last tab |
| Move Tab Left | `Cmd+Shift+Left` | `Ctrl+Shift+Left` | ✅ | Reorder tab left |
| Move Tab Right | `Cmd+Shift+Right` | `Ctrl+Shift+Right` | ✅ | Reorder tab right |
| New Window | `Cmd+N` | `Ctrl+Shift+N` | ✅ | Create new window |
| Close Window | `Cmd+Shift+W` | `Alt+F4` | ✅ | Close current window |

---

### Features & Commands

| Action | macOS | Linux/Windows | Customizable | Description |
|--------|-------|---------------|--------------|-------------|
| Command Palette | `Cmd+Shift+P` | `Ctrl+Shift+P` | ✅ | Open command palette |
| Link Hints | `Cmd+Shift+O` | `Ctrl+Shift+O` | ✅ | Show link hints for URLs |
| Copy Mode | `Cmd+Shift+C` | `Ctrl+Shift+C` | ✅ | Enter vim-style copy mode |
| Show Settings | `Cmd+,` | `Ctrl+,` | ✅ | Open settings UI |
| Reload Config | `Cmd+Shift+R` | `Ctrl+Shift+R` | ✅ | Reload configuration |
| Show Plugins | `Cmd+Shift+L` | `Ctrl+Shift+L` | ✅ | List loaded plugins |
| Plugin Manager | `Cmd+Shift+M` | `Ctrl+Shift+M` | ✅ | Open plugin manager |

---

### Session Management

| Action | macOS | Linux/Windows | Customizable | Description |
|--------|-------|---------------|--------------|-------------|
| Save Session | `Cmd+S` | `Ctrl+S` | ✅ | Save current session |
| Load Session | `Cmd+O` | `Ctrl+O` | ✅ | Open session picker |
| New Session | `Cmd+Shift+N` | `Ctrl+Shift+N` | ✅ | Create new session |
| Rename Session | `Cmd+Shift+R` | `Ctrl+Shift+R` | ✅ | Rename current session |
| Delete Session | `Cmd+Shift+D` | `Ctrl+Shift+D` | ✅ | Delete session |
| Session List | `Cmd+Shift+S` | `Ctrl+Shift+S` | ✅ | Show all sessions |

---

### Copy Mode (Vim-style)

Enter copy mode with `Ctrl+Shift+C` to use vim-style navigation:

| Action | Key | Description |
|--------|-----|-------------|
| Move Left | `h` | Move cursor left |
| Move Down | `j` | Move cursor down |
| Move Up | `k` | Move cursor up |
| Move Right | `l` | Move cursor right |
| Word Forward | `w` | Jump to next word |
| Word Backward | `b` | Jump to previous word |
| Line Start | `0` or `Home` | Jump to line start |
| Line End | `$` or `End` | Jump to line end |
| Page Down | `Ctrl+F` | Scroll page down |
| Page Up | `Ctrl+B` | Scroll page up |
| Start Selection | `v` | Begin visual selection |
| Line Selection | `V` | Select entire line |
| Copy Selection | `y` | Copy and exit |
| Cancel | `Escape` or `q` | Exit copy mode |

**Copy Mode Features**:
- Vim keybindings for navigation
- Visual selection modes
- Search with `/` (forward) and `?` (backward)
- Jump to line with `:123`
- Marks with `m{a-z}` and jump with `'{a-z}`

---

### Link Hints

Press `Ctrl+Shift+O` to show hints for URLs and file paths:

| Action | Key | Description |
|--------|-----|-------------|
| Type Hint | `a-z` | Type hint characters to follow |
| Follow Link | `Enter` | Open currently highlighted link |
| Copy Link | `c` | Copy link URL to clipboard |
| Cancel | `Escape` | Exit link hints mode |

**Link Detection**:
- HTTP/HTTPS URLs
- File paths (absolute and relative)
- SSH connections (`ssh://user@host`)
- Git remotes (`git@github.com:user/repo`)
- IP addresses

---

## Customization

### Configuration File

Customize keybindings in `~/.config/scarab/config.toml`:

```toml
[keybindings]
# Standard bindings
copy_mode = "Ctrl+Shift+C"
paste = "Ctrl+Shift+V"
search = "Ctrl+Shift+F"
command_palette = "Ctrl+Shift+P"
new_window = "Ctrl+Shift+N"
close_window = "Ctrl+Shift+W"
next_tab = "Ctrl+Tab"
prev_tab = "Ctrl+Shift+Tab"

# Custom bindings
[keybindings.custom]
"scroll_page_up" = "Shift+PageUp"
"scroll_page_down" = "Shift+PageDown"
"scroll_to_top" = "Shift+Home"
"scroll_to_bottom" = "Shift+End"
"increase_font_size" = "Ctrl+Plus"
"decrease_font_size" = "Ctrl+Minus"
"reset_font_size" = "Ctrl+0"
"toggle_fullscreen" = "F11"
"link_hints" = "Ctrl+Shift+O"
"split_horizontal" = "Ctrl+Shift+H"
"split_vertical" = "Ctrl+Shift+V"
"focus_next_pane" = "Ctrl+Shift+N"
"focus_prev_pane" = "Ctrl+Shift+P"
```

### Key Format

**Modifiers** (combine with `+`):
- `Ctrl` - Control key
- `Shift` - Shift key
- `Alt` - Alt/Option key
- `Super` - Windows/Command key

**Special Keys**:
- `Space`, `Enter`, `Tab`, `Escape`, `Backspace`, `Delete`
- `Home`, `End`, `PageUp`, `PageDown`
- `F1`-`F12`
- `Left`, `Right`, `Up`, `Down`
- `Insert`, `PrintScreen`, `Pause`

**Examples**:
```toml
"my_action" = "Ctrl+Shift+X"      # Ctrl+Shift+X
"another" = "Alt+F4"               # Alt+F4
"special" = "Super+Space"          # Win/Cmd+Space
"function" = "F12"                 # F12 alone
"arrow" = "Ctrl+Left"              # Ctrl+Left Arrow
```

### Available Actions

Complete list of customizable actions:

**Navigation**:
- `scroll_line_up`, `scroll_line_down`
- `scroll_page_up`, `scroll_page_down`
- `scroll_half_page_up`, `scroll_half_page_down`
- `scroll_to_top`, `scroll_to_bottom`

**Editing**:
- `copy`, `paste`, `select_all`, `clear_selection`
- `copy_mode_enter`, `copy_mode_exit`

**Search**:
- `search_open`, `search_close`
- `search_next`, `search_previous`
- `search_toggle_regex`, `search_toggle_case`

**View**:
- `increase_font_size`, `decrease_font_size`, `reset_font_size`
- `toggle_fullscreen`, `toggle_tab_bar`
- `zoom_in`, `zoom_out`, `zoom_reset`

**Tabs**:
- `new_tab`, `close_tab`
- `next_tab`, `prev_tab`
- `tab_1` through `tab_9`, `tab_last`
- `move_tab_left`, `move_tab_right`

**Windows**:
- `new_window`, `close_window`

**Features**:
- `command_palette`, `link_hints`
- `show_settings`, `reload_config`
- `show_plugins`, `plugin_manager`

**Sessions**:
- `save_session`, `load_session`, `new_session`
- `rename_session`, `delete_session`, `session_list`

**Splits** (upcoming):
- `split_horizontal`, `split_vertical`
- `focus_next_pane`, `focus_prev_pane`
- `resize_pane_up`, `resize_pane_down`
- `resize_pane_left`, `resize_pane_right`

### Example Configurations

**Vim-style Navigation**:
```toml
[keybindings.custom]
"scroll_line_up" = "k"
"scroll_line_down" = "j"
"scroll_page_up" = "Ctrl+B"
"scroll_page_down" = "Ctrl+F"
"scroll_to_top" = "g g"  # Press g twice
"scroll_to_bottom" = "Shift+G"
```

**tmux-style Splits**:
```toml
[keybindings]
leader_key = "Ctrl+B"  # tmux-style leader

[keybindings.custom]
"split_horizontal" = "Ctrl+B %"  # Leader + %
"split_vertical" = "Ctrl+B \""   # Leader + "
"focus_next_pane" = "Ctrl+B o"   # Leader + o
"close_pane" = "Ctrl+B x"        # Leader + x
```

**macOS-style**:
```toml
[keybindings.custom]
"copy" = "Cmd+C"
"paste" = "Cmd+V"
"search" = "Cmd+F"
"new_tab" = "Cmd+T"
"close_tab" = "Cmd+W"
"command_palette" = "Cmd+Shift+P"
```

---

## Non-Customizable Bindings

Some keybindings are handled by the terminal protocol and cannot be customized:

| Key | Action | Reason |
|-----|--------|--------|
| `Ctrl+C` | Send SIGINT | Terminal protocol standard |
| `Ctrl+Z` | Send SIGTSTP | Terminal protocol standard |
| `Ctrl+D` | Send EOF | Terminal protocol standard |
| `Ctrl+\` | Send SIGQUIT | Terminal protocol standard |
| `Ctrl+S` | XOFF (pause output) | Flow control (legacy) |
| `Ctrl+Q` | XON (resume output) | Flow control (legacy) |

**Note**: Flow control (`Ctrl+S`/`Ctrl+Q`) can be disabled in shell with `stty -ixon`.

---

## Platform-Specific Notes

### macOS

**Default differences**:
- `Cmd` is primary modifier (not `Ctrl`)
- `Option` used for alt characters
- Some shortcuts reserved by system (e.g., `Cmd+H` for hide)

**Recommended mappings**:
```toml
# macOS-friendly bindings
[keybindings.custom]
"copy" = "Cmd+C"
"paste" = "Cmd+V"
"new_tab" = "Cmd+T"
"close_tab" = "Cmd+W"
"search" = "Cmd+F"
"command_palette" = "Cmd+Shift+P"
"toggle_fullscreen" = "Cmd+Ctrl+F"  # System fullscreen
```

### Linux (X11 vs Wayland)

**X11**:
- Full modifier support
- System shortcuts can conflict (configure DE to avoid)

**Wayland**:
- Some key combinations may be intercepted by compositor
- Use compositor-specific config to pass through

**GNOME conflicts**:
- `Super+Space` - Activities overview
- `Ctrl+Alt+T` - Launch terminal
- `Alt+F1`/`F2` - Launch applications

Solution: Remap in GNOME Settings > Keyboard or use alternative modifiers.

### Windows

**Default differences**:
- `Alt+F4` closes window (system-level)
- `Win` key is `Super` modifier
- Some antivirus software may intercept keys

**Recommended**:
Avoid `Alt+F4` and `Win+key` combinations that conflict with Windows.

---

## Troubleshooting

### Keybinding not working

**Check conflicts**:
1. Desktop environment shortcuts (GNOME/KDE/Windows)
2. Window manager keybindings (i3/sway/dwm)
3. Other running applications
4. Terminal protocol restrictions (see Non-Customizable)

**Test binding**:
```bash
# Enable debug logging
SCARAB_LOG_LEVEL=debug scarab-client

# Press your key combination
# Check logs: ~/.local/share/scarab/scarab.log
# Look for "KeyEvent" lines
```

**Common conflicts**:
- `Ctrl+Shift+C/V` - Some terminals use these
- `Ctrl+Alt+T` - GNOME terminal launcher
- `Super+keys` - System shortcuts on most DEs
- `F11` - Browser fullscreen

### Key not recognized

**Symptom**: Key combination doesn't trigger action

**Solutions**:
1. Check key name spelling (case-sensitive)
2. Test with simpler binding (e.g., `F12` alone)
3. Check keyboard layout (some keys vary by layout)
4. Use `scarab-daemon --list-keys` to see recognized keys

### Modifier keys swapped

**Symptom**: Ctrl acts like Alt, etc.

**Solutions**:
1. Check `~/.Xmodmap` or `~/.config/xkb/` configuration
2. Verify keyboard layout settings
3. Test in different terminal to isolate issue
4. Use `xev` (Linux) or Karabiner (macOS) to debug

---

## See Also

- [Configuration Reference](./configuration.md) - Full config options
- [Command Palette](./features.md#command-palette) - Available commands
- [Copy Mode](./features.md#copy-mode) - Detailed vim-mode guide
- [Troubleshooting](./troubleshooting.md) - Common issues
