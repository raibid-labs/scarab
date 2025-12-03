# Migrating from iTerm2

A guide for iTerm2 users switching to Scarab.

## Quick Links

For the complete migration guide, see:
- [From iTerm2](../../../migration/from-iterm2.md) - Detailed migration guide

## Overview

iTerm2 is a feature-rich macOS terminal. While Scarab is currently Linux-only, macOS support is planned.

## Current Status

Scarab macOS support is in development. This guide prepares iTerm2 users for the eventual migration.

## Configuration Migration

### iTerm2 Configuration Location

iTerm2 stores configuration in:
```
~/Library/Preferences/com.googlecode.iterm2.plist
```

### Scarab Configuration Location

```
~/.config/scarab/config.toml
```

## Feature Comparison

### iTerm2 Features in Scarab

- **Split panes** - In development
- **Tabs** - In development
- **Profiles** - Sessions in Scarab
- **Color schemes** - Themes
- **Hot keys** - Keybindings

### Scarab Advantages

- **Plugin system** - F# scripting
- **Session persistence** - SQLite-backed
- **GPU acceleration** - Bevy engine
- **Command palette** - Fuzzy search
- **Link hints** - Keyboard-driven

### iTerm2 Features Not in Scarab

- Triggers - Use plugins instead
- Automatic profile switching - Use session templates
- Shell integration - Partial (OSC 133)
- Tmux integration - Direct terminal multiplexing

## Theme Import (Future)

When macOS support is available:

```bash
scarab-tools import-theme --from iterm2 ~/Downloads/theme.itermcolors
```

## Migration Checklist (When macOS Support Available)

- [ ] Install Scarab on macOS
- [ ] Export iTerm2 color scheme
- [ ] Convert to Scarab theme
- [ ] Set up shell profile
- [ ] Configure keybindings
- [ ] Test basic operations
- [ ] Migrate workflows

## macOS Support Timeline

macOS support is planned for Phase 7 of the roadmap. Track progress:
- GitHub Issues: macOS support
- [Roadmap](../roadmap/overview.md)

## Alternative: Use on Linux

While waiting for macOS support, try Scarab on a Linux VM or cloud instance.

## See Also

- [Platform Support](../../../PLATFORM_SUPPORT.md) - Platform status
- [Roadmap](../roadmap/overview.md) - Development roadmap
- [Configuration](../getting-started/configuration.md) - Configuration guide
