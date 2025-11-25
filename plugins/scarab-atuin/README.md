# Scarab Atuin Plugin

Integration plugin for [Atuin](https://github.com/atuinsh/atuin) shell history with Scarab Terminal.

## Features

- **Cross-session history**: Access command history from all terminal sessions
- **Advanced search**: Fuzzy search with real-time filtering
- **Statistics**: View command usage analytics
- **Cloud sync**: Optionally sync history across machines
- **Keyboard shortcuts**: Quick access via Ctrl+R

## Installation

### Prerequisites

1. **Install Atuin**:
   ```bash
   cargo install atuin
   ```

2. **Initialize Atuin**:
   ```bash
   atuin init bash  # or zsh, fish, etc.
   ```

3. **Import existing history** (optional):
   ```bash
   atuin import auto
   ```

### Plugin Installation

The plugin is included with Scarab Terminal. To enable it:

1. Ensure the plugin file exists:
   ```
   plugins/scarab-atuin/scarab-atuin.fsx
   ```

2. Configure the plugin (optional):
   ```bash
   mkdir -p ~/.config/scarab/plugins
   cp plugins/scarab-atuin/atuin.toml ~/.config/scarab/plugins/
   ```

3. Edit configuration:
   ```bash
   nano ~/.config/scarab/plugins/atuin.toml
   ```

## Usage

### Search History

Press `Ctrl+R` to open the Atuin history search overlay:

```
┌────────────────────────────────────────────┐
│ Atuin History Search: git (5 results)     │
│ ┌────────────────────────────────────────┐ │
│ │ > git commit -m "feat: Add feature"   │ │ ← Selected
│ │   git commit --amend                  │ │
│ │   git status                          │ │
│ │   git log --oneline                   │ │
│ │   git push origin main                │ │
│ └────────────────────────────────────────┘ │
│ ↑↓ navigate • Enter select • Esc close    │
└────────────────────────────────────────────┘
```

### Navigation

- **Type**: Filter results in real-time
- **↑/↓**: Navigate through results
- **Enter**: Select and insert command
- **Esc**: Close search overlay

### Command Palette

Access Atuin features from the Scarab command palette:

- **Search Atuin History**: Open history search
- **Sync Atuin History**: Sync with cloud
- **Show Command Statistics**: Display usage stats

## Configuration

Edit `~/.config/scarab/plugins/atuin.toml`:

```toml
[atuin]
enabled = true           # Enable/disable plugin
keybinding = "Ctrl+R"    # Keybinding for search
max_results = 20         # Max results to show
auto_sync = false        # Auto-sync after commands
show_stats = true        # Show command statistics
```

## Atuin Configuration

The plugin respects your Atuin configuration in `~/.config/atuin/config.toml`. Common settings:

```toml
# Sync configuration
sync_address = "https://api.atuin.sh"
sync_frequency = "10m"

# Search configuration
search_mode = "fuzzy"  # or "prefix", "fulltext"
filter_mode = "global" # or "session", "directory"

# UI configuration
show_preview = true
exit_mode = "return-query"
```

See the [Atuin documentation](https://docs.atuin.sh) for full configuration options.

## Troubleshooting

### Plugin not loading

Check that Atuin is installed:
```bash
which atuin
```

View plugin logs:
```bash
scarab --log-level debug
```

### Search not working

1. Verify Atuin is initialized:
   ```bash
   atuin status
   ```

2. Check history exists:
   ```bash
   atuin search ""
   ```

3. Try importing history:
   ```bash
   atuin import auto
   ```

### Sync issues

1. Login to Atuin:
   ```bash
   atuin login
   ```

2. Register an account:
   ```bash
   atuin register
   ```

3. Manually sync:
   ```bash
   atuin sync
   ```

## Development

### Building from source

The plugin is written in Fusabi (F# dialect) and runs in the Scarab daemon:

```bash
# Test the plugin
scarab-daemon --plugin plugins/scarab-atuin/scarab-atuin.fsx

# Compile to bytecode (future)
fusabi compile scarab-atuin.fsx -o scarab-atuin.fzb
```

### Testing

Manual test procedure:

1. Install Atuin and populate history
2. Launch Scarab terminal
3. Press Ctrl+R
4. Verify search overlay appears
5. Type search term
6. Verify filtering works
7. Select result with Enter
8. Verify command is inserted

### Contributing

Contributions welcome! Please:

1. Follow the existing code style
2. Add tests for new features
3. Update documentation
4. Submit a pull request

## Architecture

```
┌─────────────────────────────────────┐
│  Scarab Client (Bevy UI)            │
│  ┌───────────────────────────────┐  │
│  │ Modal Overlay                 │  │ ← Rendered by client
│  │ > git commit ___              │  │
│  └───────────────────────────────┘  │
└─────────────────────────────────────┘
              ↕ IPC (RemoteCommand)
┌─────────────────────────────────────┐
│  Scarab Daemon                      │
│  ┌───────────────────────────────┐  │
│  │ scarab-atuin.fsx              │  │ ← Plugin runs here
│  │ - on_input (Ctrl+R)           │  │
│  │ - query_atuin()               │  │
│  │ - send_modal_command()        │  │
│  └───────────────────────────────┘  │
└─────────────────────────────────────┘
              ↕ Shell execution
┌─────────────────────────────────────┐
│  Atuin CLI                           │
│  $ atuin search --format json       │
└─────────────────────────────────────┘
```

## License

This plugin is part of the Scarab Terminal project and follows the same license.

## Resources

- [Atuin Documentation](https://docs.atuin.sh)
- [Atuin GitHub](https://github.com/atuinsh/atuin)
- [Scarab Plugin API](../../docs/plugin-api.md)
- [Fusabi Language](https://github.com/fusabi-lang/fusabi)
