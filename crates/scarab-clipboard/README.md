# scarab-clipboard

Comprehensive clipboard integration and text selection plugin for Scarab Terminal.

## Features

### Copy Operations

- **Copy Selection** (`Ctrl+Shift+C`): Copy selected text to system clipboard
- **Copy Line** (`Ctrl+Shift+L`): Copy entire current line
- **Copy Last Command Output**: Copy output of the last executed command (TODO)
- **X11 Primary Selection**: Automatic copy-on-select for Linux/X11 environments

### Paste Operations

- **Paste** (`Ctrl+Shift+V`): Paste from system clipboard
- **Paste Primary** (Middle-click): Paste from X11 primary selection
- **Bracket Paste Mode**: Automatically wraps multiline pastes with escape sequences for shell safety
- **Smart Paste Confirmation**: Prompts before pasting large or multiline content

### Selection Modes

The plugin supports Vim-style visual selection modes:

- **Character Mode** (`v`): Select characters with precise control
- **Word Mode** (Double-click): Select entire words at cursor or click position
- **Line Mode** (`V`): Select entire lines
- **Block Mode** (`Ctrl+V`): Rectangular/column selection

### Safety Features

- **Paste Confirmation**: Automatic prompts for:
  - Multiline content (>5 lines)
  - Large pastes (>1KB)
- **Bracket Paste Mode**: Wraps pasted content with `\x1b[200~` and `\x1b[201~` to prevent accidental command execution
- **Selection Indicators**: Visual overlays show active selection mode

## Architecture

### Plugin Structure

```
scarab-clipboard/
├── src/
│   ├── lib.rs           # Main plugin implementation
│   ├── clipboard.rs     # Cross-platform clipboard manager
│   └── selection.rs     # Selection state and region management
├── Cargo.toml
└── README.md
```

### Key Components

#### ClipboardManager (`clipboard.rs`)

- Wraps `arboard` for cross-platform clipboard access
- Supports standard clipboard and X11 primary selection
- Configurable paste confirmation modes
- Error handling and logging

#### SelectionState (`selection.rs`)

- Manages text selection regions
- Supports multiple selection modes
- Normalization and boundary detection
- Grid coordinate tracking

#### ClipboardPlugin (`lib.rs`)

- Implements `scarab_plugin_api::Plugin` trait
- Handles keybindings and command execution
- Integrates with RemoteCommand protocol for UI overlays
- Text extraction from terminal grid

## Usage

### Basic Copy/Paste

```rust
// In your plugin configuration
use scarab_clipboard::ClipboardPlugin;

let clipboard_plugin = ClipboardPlugin::new();
```

### Keybindings

| Key | Action |
|-----|--------|
| `Ctrl+Shift+C` | Copy selection to clipboard |
| `Ctrl+Shift+V` | Paste from clipboard |
| `Ctrl+Shift+L` | Copy current line |
| `v` | Enter character-wise selection mode |
| `V` | Enter line-wise selection mode |
| `Ctrl+V` | Enter block selection mode |
| `y` | Yank (copy) selection and exit visual mode |
| `Esc` | Cancel selection |

### Command Palette

The plugin registers the following commands:

- `clipboard.copy` - Copy selected text
- `clipboard.copy_line` - Copy current line
- `clipboard.paste` - Paste from clipboard
- `clipboard.paste_primary` - Paste from X11 primary selection
- `clipboard.visual_character` - Start character selection
- `clipboard.visual_line` - Start line selection
- `clipboard.visual_block` - Start block selection
- `clipboard.toggle_bracket_mode` - Toggle bracket paste mode

## Configuration

```toml
[plugins.clipboard]
enabled = true

# Paste confirmation mode: "always", "smart", or "never"
confirmation_mode = "smart"

# Enable bracket paste mode by default
bracket_mode = true

# Maximum safe paste size (bytes) before confirmation
max_safe_size = 1024

# Maximum safe line count before confirmation
max_safe_lines = 5
```

## Implementation Details

### Text Extraction

The plugin extracts text from the terminal grid using the `PluginContext` API:

```rust
fn extract_selection_text(
    ctx: &PluginContext,
    region: &SelectionRegion,
    mode: SelectionMode,
) -> String
```

### Selection Modes

1. **Character Mode**: Character-by-character selection, preserving newlines
2. **Word Mode**: Expands to word boundaries using alphanumeric + underscore detection
3. **Line Mode**: Selects entire lines from column 0 to end
4. **Block Mode**: Rectangular selection maintaining column positions

### Bracket Paste

When enabled, multiline pastes are wrapped:

```
\x1b[200~<pasted content>\x1b[201~
```

This tells the shell to treat the content as a single paste operation, preventing line-by-line execution.

## Testing

```bash
# Run all tests
cargo test -p scarab-clipboard

# Run with clipboard access (requires display server)
cargo test -p scarab-clipboard -- --ignored --test-threads=1
```

### Test Coverage

- Selection region geometry and normalization
- Word boundary detection
- Paste confirmation thresholds
- Clipboard manager initialization
- Copy/paste round-trip (requires display)

## TODO

### High Priority

- [ ] Mouse-based selection (drag to select)
- [ ] Double/triple-click word/line selection
- [ ] X11 primary selection proper implementation
- [ ] Wayland clipboard support verification
- [ ] Copy last command output functionality

### Medium Priority

- [ ] Selection highlighting in UI
- [ ] Configurable paste confirmation thresholds
- [ ] Clipboard history (ring buffer)
- [ ] Rich text format support (strip ANSI codes option)
- [ ] Regex-based smart selection (URLs, paths, IPs)

### Low Priority

- [ ] OSC 52 remote clipboard support (for SSH/tmux)
- [ ] Clipboard synchronization across sessions
- [ ] Custom selection markers/indicators
- [ ] Selection undo/redo

## Platform Support

| Platform | Standard Clipboard | Primary Selection | Bracket Paste |
|----------|-------------------|-------------------|---------------|
| Linux (X11) | ✅ | 🚧 (partial) | ✅ |
| Linux (Wayland) | ✅ | ⚠️ (needs testing) | ✅ |
| macOS | ✅ | N/A | ✅ |
| Windows | ✅ | N/A | ✅ |

## Dependencies

- `arboard` 3.3 - Cross-platform clipboard access
- `scarab-plugin-api` - Plugin trait and context
- `scarab-protocol` - IPC types and remote commands
- `parking_lot` - Efficient synchronization primitives
- `regex` - Word boundary detection

## Integration with Scarab

This plugin is designed to work as a client-side plugin (running in `scarab-client`). It:

1. Responds to keyboard input via `on_input()`
2. Extracts text using `PluginContext::get_line()`
3. Sends visual feedback via `RemoteCommand::DrawOverlay`
4. Uses `arboard` for system clipboard integration

The plugin does NOT require daemon-side integration, making it lightweight and responsive.

## License

MIT OR Apache-2.0 (following Scarab workspace license)

## Contributing

When contributing to this plugin:

1. Maintain cross-platform compatibility
2. Add tests for new selection modes
3. Document keybindings in this README
4. Follow the existing error handling patterns
5. Log important operations for debugging

## References

- [arboard documentation](https://docs.rs/arboard/)
- [VTE Bracket Paste Mode](https://invisible-island.net/xterm/ctlseqs/ctlseqs.html#h2-Bracketed-Paste-Mode)
- [X11 Clipboard Specification](https://www.x.org/releases/X11R7.7/doc/libX11/libX11/libX11.html#Selections)
