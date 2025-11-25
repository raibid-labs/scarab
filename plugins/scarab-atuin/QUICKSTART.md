# Atuin Plugin Quick Start

Get started with the Scarab Atuin plugin in 5 minutes.

## Installation

### 1. Install Atuin

```bash
cargo install atuin
```

### 2. Initialize Atuin

```bash
atuin init bash  # or zsh, fish, etc.
source ~/.bashrc
```

### 3. Import History

```bash
atuin import auto
```

### 4. Start Scarab

```bash
scarab
```

That's it! The plugin is automatically loaded.

## Basic Usage

### Search History

1. Press `Ctrl+R`
2. Type search term
3. Use `↑`/`↓` to navigate
4. Press `Enter` to select
5. Press `Esc` to cancel

### Example

```
$ # Press Ctrl+R

┌────────────────────────────────────┐
│ Atuin History Search               │
│ ┌────────────────────────────────┐ │
│ │ Search: git___                 │ │
│ └────────────────────────────────┘ │
│                                    │
│ > git commit -m "message"          │
│   git push origin main             │
│   git status                       │
│                                    │
│ 3 results • ↑↓ navigate • Enter   │
└────────────────────────────────────┘

# Press Enter
$ git commit -m "message"█
```

## Command Palette

Open command palette and search:

- **Search Atuin History** - Open search
- **Sync Atuin History** - Sync with cloud
- **Show Command Statistics** - View stats

## Configuration (Optional)

```bash
# Copy default config
mkdir -p ~/.config/scarab/plugins
cp plugins/scarab-atuin/atuin.toml ~/.config/scarab/plugins/

# Edit
nano ~/.config/scarab/plugins/atuin.toml
```

Available options:

```toml
[atuin]
enabled = true        # Enable plugin
max_results = 20      # Results limit
auto_sync = false     # Auto-sync after commands
```

## Troubleshooting

### Plugin Not Loading

Check logs:
```bash
scarab --log-level debug
```

### No Results

Import history:
```bash
atuin import auto
```

### Atuin Not Found

Install Atuin:
```bash
cargo install atuin
```

## Next Steps

- Read full documentation: `README.md`
- See usage examples: `USAGE_EXAMPLES.md`
- Run tests: `TESTING.md`
- Configure Atuin: `~/.config/atuin/config.toml`

## Common Workflows

### Finding Recent Git Commands

```bash
# Ctrl+R → type "git" → navigate → Enter
```

### Finding Docker Commands

```bash
# Ctrl+R → type "docker" → navigate → Enter
```

### Finding Complex Commands

```bash
# Ctrl+R → type partial command → navigate → Enter
```

## Tips

1. **Fuzzy search**: Don't need exact matches
2. **Directory aware**: Filters by current directory (configurable)
3. **Quick exit**: Press `Esc` to cancel
4. **Edit before execute**: Select command, then modify
5. **Cloud sync**: Register account for backup

## Resources

- Atuin docs: https://docs.atuin.sh
- Full README: `plugins/scarab-atuin/README.md`
- Testing guide: `plugins/scarab-atuin/TESTING.md`
- Usage examples: `plugins/scarab-atuin/USAGE_EXAMPLES.md`

## Need Help?

1. Check `README.md` for detailed documentation
2. Check `TESTING.md` for troubleshooting
3. Check Scarab logs: `scarab --log-level debug`
4. Check Atuin status: `atuin status`
5. File an issue on GitHub
