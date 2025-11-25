# Plugin Development Guide

Welcome to the Scarab Plugin Development Guide! This comprehensive documentation will teach you how to extend Scarab Terminal with powerful plugins using the Fusabi language.

## What Can You Build?

Scarab plugins can:

- **Enhance terminal output** - Detect patterns, highlight URLs, colorize logs
- **Add custom UI** - Create overlays, notifications, command palettes
- **Automate workflows** - Auto-CD, command timing, directory suggestions
- **Integrate external services** - Git status, clipboard history, note-taking
- **Customize behavior** - Keyboard shortcuts, theme switching, window management

## Two Plugin Types

Scarab supports two runtime environments, each optimized for different use cases:

### Frontend Plugins (.fsx)
- **Interpreted** F# scripts running in the Bevy client
- **Hot-reloadable** during development
- **UI-focused** with RemoteUI components
- Best for: keyboard shortcuts, overlays, notifications, visual widgets

### Backend Plugins (.fzb)
- **Compiled** bytecode running in the daemon
- **High-performance** for output processing
- **Direct PTY access** for terminal manipulation
- Best for: output scanning, git detection, log parsing, command hooks

## Quick Start

### 1. Setup Your Development Environment

Install recommended VSCode extensions:
```bash
code --install-extension rust-lang.rust-analyzer
code --install-extension ionide.ionide-fsharp
code --install-extension skellock.just
```

Install cargo-watch for hot reloading:
```bash
cargo install cargo-watch
```

### 2. Create Your First Plugin

```bash
just plugin-new my-first-plugin frontend
```

This creates:
```
plugins/my-first-plugin/
├── my-first-plugin.fsx    # Plugin source code
├── plugin.toml            # Plugin manifest
└── README.md              # Documentation
```

### 3. Start Development Mode

```bash
just dev-mode my-first-plugin
```

This starts a hot-reload server that automatically recompiles your plugin when you save changes.

### 4. Edit Your Plugin

Open `plugins/my-first-plugin/my-first-plugin.fsx` and start coding!

## Documentation Structure

### Tutorials (Learn by Doing)

1. **[Hello World (Frontend)](tutorials/01-hello-world-frontend.md)** - Your first .fsx plugin
2. **[Hello World (Backend)](tutorials/02-hello-world-backend.md)** - Your first .fzb plugin
3. **[Plugin API Deep Dive](tutorials/03-plugin-api-deep-dive.md)** - Understanding the API
4. **[Real Plugin: URL Shortener](tutorials/04-real-plugin-url-shortener.md)** - Complete example
5. **[Frontend UI with RemoteUI](tutorials/05-frontend-ui-remoteui.md)** - Building interfaces
6. **[Backend Processing with Hooks](tutorials/06-backend-hooks.md)** - Output scanning
7. **[Testing and Publishing](tutorials/07-testing-and-publishing.md)** - Production readiness

### Architecture Guides (Understand the System)

- **[Frontend vs Backend](architecture/frontend-vs-backend.md)** - When to use .fsx vs .fzb
- **[Plugin Lifecycle](architecture/plugin-lifecycle.md)** - Load, hooks, unload
- **[Performance Guide](architecture/performance.md)** - Optimization strategies

### API Reference (Look Up Details)

- **[PluginContext](api-reference/plugin-context.md)** - Core API methods
- **[Hooks](api-reference/hooks.md)** - All available hooks
- **[RemoteUI Components](api-reference/remote-ui.md)** - UI building blocks

### Example Plugins (See It In Action)

8 complete, working examples demonstrating common patterns:

- **[hello-notification](../plugins/examples/hello-notification/)** - Simple notification on startup
- **[url-detector](../plugins/examples/url-detector/)** - Detect and highlight URLs
- **[command-timer](../plugins/examples/command-timer/)** - Time command execution
- **[git-status](../plugins/examples/git-status/)** - Show git info in prompt
- **[clipboard-history](../plugins/examples/clipboard-history/)** - Clipboard manager
- **[quick-notes](../plugins/examples/quick-notes/)** - Scratchpad overlay
- **[auto-cd](../plugins/examples/auto-cd/)** - Smart directory suggestions
- **[theme-switcher](../plugins/examples/theme-switcher/)** - Theme picker with preview

## Development Commands

### Plugin Lifecycle

```bash
# Create new plugin
just plugin-new my-plugin frontend

# Start hot-reload development
just dev-mode my-plugin

# Build plugin to bytecode
just plugin-build my-plugin

# Test plugin
just plugin-test my-plugin

# Package for distribution
just plugin-package my-plugin
```

### VSCode Integration

Press `Ctrl+Shift+P` and search for:
- "Tasks: Run Task" → "Watch Plugin (Hot Reload)"
- "Tasks: Run Task" → "Create New Plugin"

## Getting Help

- **Questions?** Open a GitHub Discussion
- **Bug Reports?** Create an Issue
- **Need Examples?** Check the `plugins/examples/` directory
- **API Clarification?** Read the API Reference docs

## Philosophy

Scarab plugins should be:

1. **Fast** - No blocking operations, async by default
2. **Reliable** - Handle errors gracefully, never crash the terminal
3. **Focused** - Do one thing well
4. **Transparent** - Users should know what plugins are doing
5. **Delightful** - Add joy to the terminal experience

## Next Steps

Ready to build your first plugin?

→ **Start with [Tutorial 1: Hello World (Frontend)](tutorials/01-hello-world-frontend.md)**

Or jump straight to:
- **[Frontend vs Backend Guide](architecture/frontend-vs-backend.md)** - Choose the right runtime
- **[API Reference](api-reference/plugin-context.md)** - Browse available methods
- **[Example Plugins](../plugins/examples/)** - Learn from working code

Happy hacking!
