# Scarab Plugin Examples - Index

This directory contains comprehensive examples for developing Scarab plugins using Fusabi.

## Documentation

| File | Description |
|------|-------------|
| **[QUICKSTART.md](QUICKSTART.md)** | Get started in 5 minutes - basic patterns and templates |
| **[README.md](README.md)** | Complete API reference and plugin development guide |
| **[INDEX.md](INDEX.md)** | This file - overview of all examples |

## Configuration Examples

| File | Description |
|------|-------------|
| **[plugins.toml](plugins.toml)** | Complete plugin configuration example with all options |

## Example Plugins

### Beginner Examples

| Plugin | LOC | Description | Key Concepts |
|--------|-----|-------------|--------------|
| **[hello-plugin.fsx](hello-plugin.fsx)** | ~60 | Simplest possible plugin | Plugin structure, metadata, on_load hook, logging |

### Intermediate Examples

| Plugin | LOC | Description | Key Concepts |
|--------|-----|-------------|--------------|
| **[output-filter.fsx](output-filter.fsx)** | ~120 | Highlight errors in output | on_output hook, regex patterns, overlays, configuration |
| **[custom-keybind.fsx](custom-keybind.fsx)** | ~200 | Custom keyboard shortcuts | on_input hook, key detection, modals, command palette |
| **[git-status.fsx](git-status.fsx)** | ~180 | Display git status indicator | on_pre/post_command, external processes, persistent overlays |
| **[notification-monitor.fsx](notification-monitor.fsx)** | ~150 | Notify on long-running commands | Stateful tracking, time measurement, notifications |

### Advanced Examples

| Plugin | LOC | Description | Key Concepts |
|--------|-----|-------------|--------------|
| **[session-manager.fsx](session-manager.fsx)** | ~260 | Advanced session management | Client attach/detach, persistent state, complex UI, file I/O |
| **[scarab-atuin.fsx](scarab-atuin.fsx)** | ~450 | Atuin shell history integration | External tool integration, JSON parsing, modal search UI, real-time filtering |

## Plugin Complexity Matrix

```
Simple Output   │ hello-plugin.fsx
      ↓         │
Pattern Match   │ output-filter.fsx
      ↓         │
Input Handling  │ custom-keybind.fsx
      ↓         │
External Procs  │ git-status.fsx
      ↓         │
State + Time    │ notification-monitor.fsx
      ↓         │
Full Featured   │ session-manager.fsx, scarab-atuin.fsx
```

## Hooks Demonstrated

| Hook | Used In |
|------|---------|
| **on_load** | All plugins |
| **on_unload** | All plugins (cleanup) |
| **on_output** | output-filter.fsx |
| **on_input** | custom-keybind.fsx, session-manager.fsx, scarab-atuin.fsx |
| **on_pre_command** | git-status.fsx, notification-monitor.fsx |
| **on_post_command** | git-status.fsx, notification-monitor.fsx, scarab-atuin.fsx |
| **on_resize** | git-status.fsx, session-manager.fsx |
| **on_attach** | session-manager.fsx |
| **on_detach** | session-manager.fsx |
| **on_remote_command** | custom-keybind.fsx, git-status.fsx, notification-monitor.fsx, session-manager.fsx, scarab-atuin.fsx |

## Features Demonstrated

### Core Features
- [x] Plugin metadata and registration (all plugins)
- [x] Logging at different levels (all plugins)
- [x] Context API usage (all plugins)

### Terminal Interaction
- [x] Reading terminal size (hello-plugin.fsx)
- [x] Reading cursor position (output-filter.fsx)
- [x] Reading terminal lines (git-status.fsx)
- [x] Reading/writing cells (potential addition)

### Input/Output Handling
- [x] Output interception (output-filter.fsx)
- [x] Input interception (custom-keybind.fsx, scarab-atuin.fsx)
- [x] Input modification (custom-keybind.fsx, scarab-atuin.fsx)
- [x] Regex pattern matching (output-filter.fsx)

### UI Components
- [x] Drawing overlays (all visual plugins)
- [x] Clearing overlays (all visual plugins)
- [x] Showing modals (custom-keybind.fsx, session-manager.fsx, scarab-atuin.fsx)
- [x] Command palette integration (most plugins)
- [x] User notifications (notification-monitor.fsx, scarab-atuin.fsx)

### State Management
- [x] Simple mutable state (output-filter.fsx, scarab-atuin.fsx)
- [x] Dictionary tracking (notification-monitor.fsx)
- [x] Persistent state (session-manager.fsx)
- [x] Configuration reading (all plugins)

### External Integration
- [x] Running shell commands (git-status.fsx, scarab-atuin.fsx)
- [x] Environment variable access (hello-plugin.fsx)
- [x] File I/O (session-manager.fsx)
- [x] Time tracking (notification-monitor.fsx)
- [x] JSON parsing (scarab-atuin.fsx)
- [x] External tool integration (scarab-atuin.fsx)

### Advanced Patterns
- [x] Command duration tracking (notification-monitor.fsx)
- [x] Client lifecycle management (session-manager.fsx)
- [x] Color coding and themes (session-manager.fsx)
- [x] Error handling (all plugins)
- [x] Real-time search filtering (scarab-atuin.fsx)
- [x] Graceful degradation (scarab-atuin.fsx)

## Learning Path

### Level 1: Getting Started
1. Read **QUICKSTART.md**
2. Copy and modify **hello-plugin.fsx**
3. Experiment with logging and terminal state

### Level 2: Basic Functionality
1. Study **output-filter.fsx** for output hooks
2. Study **custom-keybind.fsx** for input hooks
3. Build a simple plugin combining both

### Level 3: Integration
1. Study **git-status.fsx** for external processes
2. Study **notification-monitor.fsx** for state tracking
3. Build a plugin that monitors something in your environment

### Level 4: Advanced Features
1. Study **session-manager.fsx** for complex state
2. Study **scarab-atuin.fsx** for external tool integration
3. Implement persistent storage
4. Build a full-featured plugin for your workflow

## Real-World Plugin Examples

### scarab-atuin.fsx - Shell History Integration

This production-quality plugin demonstrates:
- **Tool detection**: Check if external tools are installed
- **JSON parsing**: Parse structured output from external commands
- **Modal search UI**: Interactive search with real-time filtering
- **Keyboard navigation**: Arrow keys, Enter, Escape handling
- **Command insertion**: Insert selected text into terminal
- **Error handling**: Graceful degradation when tool unavailable
- **Configuration**: Multiple config options
- **Command palette**: Integration with Scarab's command system

Key features:
- Integrates Atuin shell history tool
- Ctrl+R opens search modal
- Real-time search filtering
- Arrow key navigation
- Enter to select and insert command
- Auto-sync support (optional)
- Statistics display

Perfect for learning:
- How to integrate external CLI tools
- Building interactive search interfaces
- Parsing and displaying external data
- User-friendly error messages
- Production plugin patterns

## Plugin Ideas to Try

### Easy
- [ ] **clock-plugin.fsx** - Display current time in corner
- [ ] **prompt-customizer.fsx** - Add custom prompt indicators
- [ ] **directory-tracker.fsx** - Show current directory
- [ ] **battery-indicator.fsx** - Show laptop battery status

### Medium
- [ ] **tmux-integration.fsx** - Integrate with tmux sessions
- [ ] **ssh-helper.fsx** - Detect SSH connections and show hostname
- [ ] **clipboard-manager.fsx** - Advanced clipboard history
- [ ] **auto-suggestions.fsx** - Command completion suggestions
- [x] **atuin-integration.fsx** - Integrate Atuin shell history (scarab-atuin.fsx)

### Advanced
- [ ] **multiplexer.fsx** - Full terminal multiplexer
- [ ] **recording-plugin.fsx** - Record and replay terminal sessions
- [ ] **collaboration.fsx** - Share terminal with others
- [ ] **ai-assistant.fsx** - Integrate with AI for command help
- [ ] **fzf-integration.fsx** - Integrate fzf fuzzy finder
- [ ] **zoxide-integration.fsx** - Smart directory jumping

## Testing Checklist

When developing plugins, test:
- [ ] Plugin loads without errors
- [ ] Hooks execute at expected times
- [ ] Overlays appear in correct positions
- [ ] Terminal resize doesn't break UI
- [ ] Configuration is read correctly
- [ ] Cleanup happens on unload
- [ ] No memory leaks with long-running sessions
- [ ] Error messages are helpful
- [ ] External tool failures handled gracefully
- [ ] Search and filtering work smoothly

## Contributing

To contribute new examples:
1. Follow the existing file naming convention
2. Include comprehensive metadata comments
3. Add inline comments explaining non-obvious logic
4. Update this INDEX.md with your example
5. Update README.md if you introduce new patterns
6. Submit a PR with description of what your example teaches

## Resources

- **Fusabi Language**: https://github.com/fusabi-lang/fusabi
- **Scarab Docs**: /docs/
- **Plugin API Source**: /crates/scarab-plugin-api/
- **Core Plugins**: /crates/scarab-{palette,nav,platform}/
- **Atuin**: https://github.com/atuinsh/atuin

## License

All examples are MIT/Apache-2.0 licensed (same as Scarab).
