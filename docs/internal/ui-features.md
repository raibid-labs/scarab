# Advanced UI/UX Features

Scarab terminal emulator includes power-user features inspired by Vimium, Spacemacs, and modern terminal emulators.

## 🎯 Features Overview

### 1. Link Hints (Vimium-style)

Keyboard-driven link activation for URLs, file paths, and email addresses.

**Activation**: `Ctrl+K`

**Features**:
- Automatic detection of URLs (http://, www.)
- File path detection (/path/to/file, ./relative)
- Email address detection
- Two-character hint keys (aa, ab, ac, ...)
- Type hint keys to activate links
- Customizable via Fusabi scripts

**Example**:
```
# Terminal output with links
Check out https://github.com/user/repo or edit /src/main.rs

# Press Ctrl+K to show hints
[aa] https://github.com/user/repo
[ab] /src/main.rs

# Type 'aa' to open GitHub, 'ab' to open file in editor
```

**Customization** (`~/.config/scarab/ui/link_hints.fusabi`):
```javascript
fn detect_custom_links(text: String) -> Array {
    // Add custom link patterns (GitHub issues, Jira tickets, etc.)
}

fn generate_hint_keys(count: Int) -> Array {
    // Use custom hint key sequence (home row, etc.)
}
```

### 2. Command Palette

Fuzzy search for all terminal commands with <50ms search time for 1000+ commands.

**Activation**: `Ctrl+P`

**Features**:
- Fuzzy search algorithm
- Category grouping
- Keyboard navigation (↑↓ arrows)
- Custom command registration
- Keybinding display
- Score-based ranking

**Default Commands**:
- Edit: Copy, Paste, Cut, Undo, Redo
- Terminal: Clear, New Tab, Close Tab
- Window: Split Horizontal/Vertical, Focus
- Search: Find, Replace

**Customization** (`~/.config/scarab/ui/command_palette.fusabi`):
```javascript
fn register_custom_commands() -> Array {
    return [
        {
            id: "git.status",
            name: "Git Status",
            description: "Show git status",
            category: "Git",
            keybind: "Ctrl+G S",
            action: fn() { send_keys("git status\n"); }
        }
    ];
}
```

### 3. Leader Key Menu (Spacemacs-style)

Hierarchical menu system with 1-second timeout and visual feedback.

**Activation**: `Space` (configurable)

**Features**:
- Cascading menus (root → submenu → command)
- Visual timeout indicator
- Key sequence display
- Escape to cancel
- Customizable timeout

**Default Menu Structure**:
```
Space → f (Files) → f (Find), r (Recent), s (Save)
     → b (Buffer) → c (Clear), s (Save), y (Yank)
     → w (Window) → s (Split H), v (Split V), c (Close)
     → g (Git)    → s (Status), c (Commit), p (Push)
```

**Example Flow**:
```
1. Press Space        → Root menu appears
2. Press 'g'          → Git submenu appears
3. Press 's'          → Executes "git status"
```

**Customization** (`~/.config/scarab/ui/leader_key.fusabi`):
```javascript
fn create_menu_structure() -> Object {
    return {
        root: {
            title: "🚀 Custom Menu",
            items: [
                {key: "d", label: "Docker", submenu: "docker"}
            ]
        },
        docker: {
            title: "🐳 Docker",
            items: [
                {key: "p", label: "PS", command: "docker.ps"}
            ]
        }
    };
}
```

### 4. Visual Selection Mode

Vim-style text selection with keyboard.

**Activation**:
- `v` - Character-wise selection
- `V` - Line-wise selection (Shift+V)
- `Ctrl+V` - Block selection

**Features**:
- Arrow key navigation
- Visual highlight overlay
- Copy with 'y' (yank)
- Multiple selection modes
- Escape to cancel

**Usage**:
```
1. Press 'v'          → Enter visual mode
2. Use arrow keys     → Expand selection
3. Press 'y'          → Copy to clipboard
4. Press Escape       → Exit visual mode
```

### 5. Configurable Key Bindings

Fully customizable keyboard shortcuts with modifier support.

**Features**:
- Ctrl, Alt, Shift, Super modifiers
- Save/load from config file
- Conflict detection
- Default bindings

**Default Bindings**:
| Binding | Action |
|---------|--------|
| Ctrl+C | Copy |
| Ctrl+V | Paste |
| Ctrl+L | Clear Terminal |
| Ctrl+P | Command Palette |
| Ctrl+K | Link Hints |

**Configuration** (`~/.config/scarab/keybindings.conf`):
```
Ctrl+KeyC=edit.copy
Ctrl+KeyV=edit.paste
Ctrl+KeyP=palette.open
```

### 6. Smooth Animations

60 FPS animations with easing functions.

**Features**:
- Fade in/out transitions
- Slide animations
- Cubic easing (in, out, in-out)
- <200ms menu open time
- Configurable duration

**Easing Functions**:
- `ease_in_cubic` - Slow start, fast end
- `ease_out_cubic` - Fast start, slow end
- `ease_in_out_cubic` - Smooth both ends
- `ease_in_quad`, `ease_out_quad`
- `ease_in_sine`, `ease_out_sine`

## 🎨 Theming

All UI elements support theming via Fusabi scripts:

```javascript
fn get_palette_style() -> Object {
    return {
        background_color: {r: 0.05, g: 0.05, b: 0.05, a: 0.98},
        border_color: {r: 0.3, g: 0.3, b: 0.5, a: 1.0},
        // ... more properties
    };
}
```

## 📊 Performance Metrics

- **Link Detection**: >95% accuracy
- **Menu Open Time**: <200ms
- **Fuzzy Search**: <50ms for 1000 commands
- **Animation Frame Rate**: 60 FPS
- **Keyboard-only**: 100% workflow coverage

## 🔧 Configuration Files

| File | Purpose |
|------|---------|
| `~/.config/scarab/ui/link_hints.fusabi` | Link detection customization |
| `~/.config/scarab/ui/command_palette.fusabi` | Command palette customization |
| `~/.config/scarab/ui/leader_key.fusabi` | Leader key menu structure |
| `~/.config/scarab/keybindings.conf` | Key binding configuration |

## 🚀 Getting Started

1. **Enable UI features** (enabled by default):
   ```toml
   # ~/.config/scarab/config.toml
   [ui]
   link_hints_enabled = true
   command_palette_enabled = true
   leader_key_enabled = true
   animations_enabled = true
   ```

2. **Customize with Fusabi scripts**:
   ```bash
   # Copy example scripts
   cp examples/ui-scripts/*.fusabi ~/.config/scarab/ui/

   # Edit to customize
   $EDITOR ~/.config/scarab/ui/leader_key.fusabi
   ```

3. **Try the features**:
   ```bash
   # Start Scarab
   scarab

   # Press Ctrl+K for link hints
   # Press Ctrl+P for command palette
   # Press Space for leader menu
   ```

## 🧪 Testing

Run comprehensive UI tests:

```bash
cd crates/scarab-client
cargo test ui_tests
```

**Test Coverage**:
- Link detection accuracy
- Fuzzy search performance
- Animation smoothness
- Key binding conflicts
- Selection region calculations
- 60 FPS animation verification

## 📚 Examples

See `examples/ui-scripts/` for complete customization examples:

- `link_hints.fusabi` - Custom link patterns (GitHub issues, Jira)
- `command_palette.fusabi` - Git/Docker/NPM commands
- `leader_key.fusabi` - Full menu hierarchy

## 🎓 Learn More

- [Vimium Documentation](https://github.com/philc/vimium)
- [Spacemacs Leader Key](https://www.spacemacs.org/)
- [Fusabi Scripting Guide](./fusabi-scripting.md)
- [Bevy UI Examples](https://bevyengine.org/examples/UI/)

---

**Next Steps**: Explore the [Plugin System](./plugins.md) to extend Scarab further.
