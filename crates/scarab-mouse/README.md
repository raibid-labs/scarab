# scarab-mouse

Comprehensive mouse support plugin for Scarab Terminal Emulator.

## Features

### Click Operations
- **Single Click**: Position cursor at click location
- **Double Click**: Select word under cursor
- **Triple Click**: Select entire line
- **Ctrl+Click**: Open URLs and file paths
- **Shift+Click**: Extend selection to clicked position

### Mouse Buttons
- **Left Button**: Selection and cursor positioning
- **Right Button**: Context menu with common actions
- **Middle Button**: Paste from X11 primary selection
- **Scroll Wheel**: Navigate scrollback buffer

### Mouse Modes
- **Normal Mode**: Scarab handles all mouse events
- **Application Mode**: Mouse events forwarded to running applications (vim, tmux, etc.)
- **Auto-Detection**: Automatically switches mode based on ANSI escape sequences

### Selection Types
- **Character Selection**: Click and drag for precise selection
- **Word Selection**: Double-click to select words
- **Line Selection**: Triple-click to select lines
- **Block Selection**: Rectangular selection (Alt+Drag)

### Context Menu
Right-click brings up a context menu with:
- Copy selection
- Paste from clipboard
- Select all
- Clear selection
- Search
- New tab
- Split pane (horizontal/vertical)

Smart context menus for:
- URLs: "Open URL", "Copy URL"
- File paths: "Open File", "Copy Path"

### Integration
- Works seamlessly with clipboard plugin for copy/paste
- Integrates with URL detection for Ctrl+Click
- Supports terminal application mouse protocols (SGR mode)

## Architecture

The plugin is split into two parts:

1. **Plugin Side** (`lib.rs`): Implements the `Plugin` trait for daemon-side logic
   - Mouse mode detection from ANSI sequences
   - Command handlers for mouse operations
   - State management

2. **Bevy Side** (`bevy_plugin.rs`): Client-side rendering and input handling
   - Mouse event capture from Bevy input system
   - Selection rendering
   - Context menu UI
   - IPC communication with daemon

## Usage

### Adding to Scarab Client

```rust
use scarab_mouse::MousePlugin;

// In your main function
let mouse_plugin = MousePlugin::new();
let bevy_plugin = mouse_plugin.bevy_plugin();

app.add_plugins(bevy_plugin);
```

### Configuration

The plugin can be configured via Scarab's configuration system:

```toml
[plugins.scarab-mouse]
enabled = true

# Default mouse mode
default_mode = "normal"  # or "application"

# Enable/disable specific features
enable_url_detection = true
enable_file_detection = true
enable_context_menu = true

# Click timing (milliseconds)
double_click_threshold = 500
triple_click_threshold = 500

# Selection behavior
select_word_on_double_click = true
select_line_on_triple_click = true
```

## Mouse Mode Detection

The plugin automatically detects when applications request mouse mode:

- `CSI ? 1000 h` - Enable X10 mouse reporting
- `CSI ? 1002 h` - Enable button-event tracking
- `CSI ? 1003 h` - Enable any-event tracking
- `CSI ? 1006 h` - Enable SGR extended mode

Common applications that use mouse mode:
- vim/neovim (with `:set mouse=a`)
- emacs
- tmux (with `set -g mouse on`)
- htop
- less
- ranger/mc

## Technical Details

### Mouse Event Flow

1. **Bevy Input System** captures raw mouse events
2. **Click Detector** determines click type (single/double/triple)
3. **Mode Handler** checks current mouse mode
4. **Event Processor** either:
   - Handles locally (Normal mode): selection, cursor positioning
   - Forwards to application (Application mode): generates ANSI sequences

### ANSI Mouse Sequences

The plugin generates SGR format mouse sequences for application mode:

```
CSI < button ; x ; y M    (press)
CSI < button ; x ; y m    (release)
```

Button codes:
- 0: Left button
- 1: Middle button
- 2: Right button
- 64: Scroll up
- 65: Scroll down

Modifiers add to button code:
- +4: Shift
- +8: Alt
- +16: Ctrl

### Selection Rendering

Selection is rendered as colored overlays on top of terminal cells:
- Uses Bevy's sprite system for efficient rendering
- Selection color is themeable
- Supports different selection types (linear, word, line, block)

## Commands

The plugin provides the following commands accessible via command palette:

- `mouse.copy` - Copy selected text to clipboard
- `mouse.paste` - Paste from clipboard
- `mouse.select_all` - Select all text
- `mouse.clear_selection` - Clear current selection
- `mouse.toggle_mode` - Switch between Normal and Application mode

## Testing

Run the test suite:

```bash
cargo test -p scarab-mouse
```

Run with logging:

```bash
RUST_LOG=scarab_mouse=debug cargo run -p scarab-client
```

## TODO

- [ ] Implement actual IPC communication with daemon
- [ ] Add font metrics integration for accurate grid positioning
- [ ] Implement context menu UI rendering
- [ ] Add URL/file path detection and highlighting
- [ ] Implement clipboard integration
- [ ] Add configuration file support
- [ ] Support for custom mouse cursors
- [ ] Mouse gesture support (e.g., right-drag for scrollback)
- [ ] Accessibility: keyboard navigation for context menu
- [ ] Visual feedback for mouse mode (indicator in status bar)

## License

MIT OR Apache-2.0
