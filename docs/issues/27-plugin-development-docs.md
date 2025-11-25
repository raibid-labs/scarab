# Issue #27: Comprehensive Plugin Development Documentation

## 🎯 Goal
Create the **definitive guide** for Scarab plugin development, covering tools, workflows, and best practices for both `.fsx` (frontend) and `.fzb` (backend) plugins.

## 🐛 Problem

Current plugin documentation is fragmented and incomplete:

- ❌ No VSCode extension recommendations or IDE setup
- ❌ No development workflow tooling (`just dev-mode` or equivalent)
- ❌ Unclear when to use `.fsx` vs `.fzb`
- ❌ No tutorial series (beginner to advanced)
- ❌ Limited real-world examples
- ❌ No debugging guide
- ❌ No performance profiling guide
- ❌ No plugin testing framework

**This documentation must serve TWO audiences:**
1. **External users** - Learning to extend Scarab
2. **Internal developers (us)** - Reference for building core plugins

## 💡 Proposed Solution

Create a comprehensive, multi-layered plugin development ecosystem:

### 1. Development Environment Setup
- VSCode extensions configuration
- `just dev-mode` command for hot-reload workflow
- Debugging tools integration
- Linting and formatting

### 2. Tutorial Series (7 Parts)
- Part 1: Hello World (.fsx frontend plugin)
- Part 2: Hello World (.fzb backend plugin)
- Part 3: Understanding the Plugin API
- Part 4: Building a Real Plugin (URL shortener)
- Part 5: Frontend UI with RemoteUI
- Part 6: Backend Processing with Hooks
- Part 7: Testing and Publishing

### 3. Architecture Deep Dive
- When/why/how to use `.fsx` vs `.fzb`
- Plugin lifecycle
- Performance considerations
- Security model

### 4. API Reference
- Complete plugin API documentation
- Hook reference with examples
- RemoteUI component library
- PluginContext methods

### 5. Development Tools
- `just dev-mode` - Hot reload dev server
- `just plugin-new <name>` - Scaffold generator
- `just plugin-test <name>` - Run plugin tests
- `just plugin-package <name>` - Build `.fzb` + manifest

## 📋 Implementation Tasks

### Phase 1: Development Tools (1 day)

#### 1.1 VSCode Extensions Configuration

**File**: `.vscode/extensions.json` (update)

```json
{
  "recommendations": [
    // Rust development
    "rust-lang.rust-analyzer",
    "tamasfe.even-better-toml",
    "serayuzgur.crates",

    // F#/Fusabi development
    "ionide.ionide-fsharp",
    "ms-vscode.vscode-json",

    // Plugin development
    "redhat.vscode-yaml",
    "ms-vscode.hexeditor",

    // Quality of life
    "aaron-bond.better-comments",
    "usernamehw.errorlens",
    "streetsidesoftware.code-spell-checker"
  ]
}
```

**File**: `.vscode/settings.json` (new)

```json
{
  "rust-analyzer.cargo.features": "all",
  "rust-analyzer.checkOnSave.command": "clippy",

  // Fusabi file associations
  "files.associations": {
    "*.fsx": "fsharp",
    "*.fzb": "binary"
  },

  // Auto-format on save
  "editor.formatOnSave": true,
  "[rust]": {
    "editor.defaultFormatter": "rust-lang.rust-analyzer"
  },
  "[fsharp]": {
    "editor.defaultFormatter": "ionide.ionide-fsharp"
  }
}
```

**File**: `.vscode/tasks.json` (new)

```json
{
  "version": "2.0.0",
  "tasks": [
    {
      "label": "Watch Plugin (Hot Reload)",
      "type": "shell",
      "command": "just dev-mode ${input:pluginName}",
      "problemMatcher": [],
      "isBackground": true
    },
    {
      "label": "Compile Plugin to .fzb",
      "type": "shell",
      "command": "just plugin-build ${input:pluginName}",
      "problemMatcher": []
    },
    {
      "label": "Test Plugin",
      "type": "shell",
      "command": "just plugin-test ${input:pluginName}",
      "problemMatcher": []
    }
  ],
  "inputs": [
    {
      "id": "pluginName",
      "type": "promptString",
      "description": "Plugin name (e.g., my-plugin)"
    }
  ]
}
```

#### 1.2 Justfile Commands

**File**: `justfile` (add new recipes)

```makefile
# Plugin development commands

# Hot-reload development mode for a plugin
dev-mode plugin_name:
    #!/usr/bin/env bash
    echo "🔄 Starting dev mode for {{plugin_name}}"
    echo "   Watching: plugins/{{plugin_name}}"
    echo "   Press Ctrl+C to stop"

    # Build plugin initially
    just plugin-build {{plugin_name}}

    # Watch for changes and rebuild
    cargo watch -w plugins/{{plugin_name}} \
        -x "run -p scarab-plugin-compiler -- plugins/{{plugin_name}}/{{plugin_name}}.fsx" \
        -s "just reload-plugin {{plugin_name}}"

# Create new plugin from template
plugin-new plugin_name type="frontend":
    #!/usr/bin/env bash
    mkdir -p plugins/{{plugin_name}}

    if [ "{{type}}" = "frontend" ]; then
        cat > plugins/{{plugin_name}}/{{plugin_name}}.fsx << 'EOF'
    module {{plugin_name}}

    open Scarab.PluginApi

    [<Plugin>]
    let metadata = {
        Name = "{{plugin_name}}"
        Version = "0.1.0"
        Description = "TODO: Add description"
        Author = "Your Name"
    }

    [<OnLoad>]
    let onLoad (ctx: PluginContext) =
        ctx.Log Info "{{plugin_name}} loaded!"

    [<OnKeyPress>]
    let onKeyPress (ctx: PluginContext) (key: KeyEvent) =
        // TODO: Handle key presses
        ()
    EOF
    else
        cat > plugins/{{plugin_name}}/{{plugin_name}}.fsx << 'EOF'
    module {{plugin_name}}

    open Scarab.PluginApi

    [<Plugin>]
    let metadata = {
        Name = "{{plugin_name}}"
        Version = "0.1.0"
        Description = "TODO: Add description"
        Author = "Your Name"
    }

    [<OnOutput>]
    let onOutput (ctx: PluginContext) (text: string) =
        // TODO: Process terminal output
        ()
    EOF
    fi

    # Create manifest
    cat > plugins/{{plugin_name}}/plugin.toml << 'EOF'
    [plugin]
    name = "{{plugin_name}}"
    version = "0.1.0"
    runtime = "{{type}}"

    [plugin.metadata]
    description = "TODO: Add description"
    author = "Your Name"
    license = "MIT"

    [hooks]
    # List enabled hooks (uncomment as needed)
    # on_load = true
    # on_output = true
    # on_input = true
    # on_resize = true
    # on_key_press = true
    EOF

    echo "✅ Created new {{type}} plugin: {{plugin_name}}"
    echo "   Location: plugins/{{plugin_name}}"
    echo "   Next steps:"
    echo "     1. Edit plugins/{{plugin_name}}/{{plugin_name}}.fsx"
    echo "     2. Run: just dev-mode {{plugin_name}}"

# Build plugin to .fzb bytecode
plugin-build plugin_name:
    cargo run -p scarab-plugin-compiler -- plugins/{{plugin_name}}/{{plugin_name}}.fsx -o plugins/{{plugin_name}}/{{plugin_name}}.fzb

# Test plugin
plugin-test plugin_name:
    cargo test -p scarab-plugin-api -- {{plugin_name}}

# Reload plugin in running daemon
reload-plugin plugin_name:
    #!/usr/bin/env bash
    if pgrep scarab-daemon > /dev/null; then
        echo "🔄 Reloading plugin: {{plugin_name}}"
        # Send SIGUSR1 to daemon to reload plugins
        pkill -SIGUSR1 scarab-daemon
        echo "✅ Plugin reloaded"
    else
        echo "⚠️  Daemon not running"
    fi

# Package plugin for distribution
plugin-package plugin_name:
    #!/usr/bin/env bash
    just plugin-build {{plugin_name}}
    mkdir -p dist/plugins
    tar -czf dist/plugins/{{plugin_name}}.tar.gz \
        -C plugins/{{plugin_name}} \
        {{plugin_name}}.fzb \
        plugin.toml \
        README.md
    echo "📦 Package created: dist/plugins/{{plugin_name}}.tar.gz"
```

### Phase 2: Tutorial Series (3 days)

#### Tutorial 1: Hello World (Frontend)
**File**: `docs/plugin-development/01-hello-world-frontend.md`

Complete walkthrough:
1. Create plugin directory
2. Write simple `.fsx` script
3. Use `just dev-mode` for hot reload
4. Test in Scarab
5. View logs

**Example**: A plugin that shows a notification when Scarab starts

#### Tutorial 2: Hello World (Backend)
**File**: `docs/plugin-development/02-hello-world-backend.md`

Complete walkthrough:
1. Create plugin directory
2. Write `.fsx` script with output hook
3. Compile to `.fzb` bytecode
4. Load in daemon
5. Debug with logs

**Example**: A plugin that detects URLs in terminal output

#### Tutorial 3: Understanding the Plugin API
**File**: `docs/plugin-development/03-plugin-api-deep-dive.md`

Comprehensive guide:
- Plugin lifecycle (load → hooks → unload)
- Available hooks and when they trigger
- PluginContext methods
- RemoteUI for frontend plugins
- Error handling
- Performance considerations

#### Tutorial 4: Real Plugin - URL Shortener
**File**: `docs/plugin-development/04-real-plugin-url-shortener.md`

Build a complete plugin that:
1. Detects long URLs in terminal output
2. Sends to URL shortening service
3. Shows notification with shortened URL
4. Copies to clipboard

**Demonstrates**:
- Output parsing
- External API calls
- UI notifications
- Clipboard integration

#### Tutorial 5: Frontend UI with RemoteUI
**File**: `docs/plugin-development/05-frontend-ui-remoteui.md`

Deep dive into UI:
- RemoteUI component system
- Creating overlays
- Handling user input
- Styling components
- Animation and transitions

**Example**: Command palette clone

#### Tutorial 6: Backend Processing with Hooks
**File**: `docs/plugin-development/06-backend-hooks.md`

Deep dive into backend:
- Output scanning patterns
- State management
- Terminal manipulation
- IPC communication
- Performance optimization

**Example**: Git status parser

#### Tutorial 7: Testing and Publishing
**File**: `docs/plugin-development/07-testing-and-publishing.md`

Production readiness:
- Unit testing plugins
- Integration testing
- Performance profiling
- Packaging for distribution
- Publishing to plugin registry

### Phase 3: Architecture Guide (1 day)

**File**: `docs/plugin-development/architecture/frontend-vs-backend.md`

#### When to Use .fsx (Frontend Plugins)

**Characteristics:**
- Interpreted at runtime (no compilation step)
- Hot-reloadable during development
- Runs in client process (Bevy UI)
- Access to RemoteUI components
- Can render custom UI elements

**Best For:**
- UI enhancements (overlays, panels, widgets)
- Keyboard shortcut handlers
- Theme extensions
- Command palette commands
- Visual indicators
- Notifications

**Limitations:**
- Cannot process terminal output directly
- No access to PTY
- Higher latency for compute tasks
- Single-threaded (runs on Bevy main thread)

**Performance:**
- Startup: Fast (interpreted, no bytecode loading)
- Runtime: Adequate for UI logic (<16ms per frame)
- Memory: Low overhead

**Example Use Cases:**
```
✅ Custom command palette commands
✅ Keyboard macro system
✅ Window manager integration
✅ Notification system
✅ Status bar widgets
✅ Context menus
❌ Log file parsing
❌ Git status detection
❌ Performance-critical output scanning
```

#### When to Use .fzb (Backend Plugins)

**Characteristics:**
- Compiled to bytecode (one-time compilation)
- Runs in daemon process
- Access to terminal output/input streams
- Can manipulate PTY directly
- Lower latency for processing

**Best For:**
- Output scanning and parsing
- Terminal state manipulation
- Performance-critical hooks
- Background processing
- External API integration
- File watching

**Limitations:**
- Requires compilation step
- No direct UI rendering
- Must use IPC for UI commands
- Restart required for updates (no hot reload)

**Performance:**
- Startup: Slower (bytecode loading + JIT)
- Runtime: Fast (compiled, optimized)
- Memory: Higher (VM overhead)

**Example Use Cases:**
```
✅ URL detection in output
✅ Git status parsing
✅ Log file colorization
✅ Command timing
✅ Auto-completion
✅ Directory change detection
❌ Custom UI overlays
❌ Keyboard shortcuts
❌ UI theming
```

#### Hybrid Approach

Many plugins benefit from BOTH:

**Example: Atuin Integration**
- **Backend (.fzb)**: Detect `Ctrl+R`, query Atuin, parse results
- **Frontend (.fsx)**: Render search overlay, handle user selection

**Example: Git Integration**
- **Backend (.fzb)**: Parse git commands, detect repo changes
- **Frontend (.fsx)**: Show branch indicator in status bar

**Communication:**
```
Backend (.fzb in daemon)
    ↓ (IPC via RemoteCommand)
Frontend (.fsx in client)
    ↓ (User interaction)
Backend (.fzb in daemon)
```

### Phase 4: API Reference (1 day)

**File**: `docs/plugin-development/api-reference/plugin-context.md`

Complete API documentation with examples for every method.

**File**: `docs/plugin-development/api-reference/hooks.md`

Every hook with:
- When it triggers
- Parameters
- Return values
- Performance impact
- Examples

**File**: `docs/plugin-development/api-reference/remote-ui.md`

UI component library:
- Overlay
- Modal
- Notification
- Input
- List
- Button
- Custom components

### Phase 5: Examples Gallery (1 day)

**Directory**: `plugins/examples/`

Create 8 example plugins:

1. **hello-notification** (.fsx) - Show notification on startup
2. **url-detector** (.fzb) - Highlight URLs in output
3. **command-timer** (.fzb) - Time command execution
4. **git-status** (.fzb) - Show git info in prompt
5. **clipboard-history** (.fsx) - Clipboard manager overlay
6. **quick-notes** (.fsx) - Scratchpad overlay
7. **auto-cd** (.fzb) - Smart directory suggestions
8. **theme-switcher** (.fsx) - Theme picker with preview

Each example includes:
- Complete source code
- README with explanation
- Configuration examples
- Test cases

## 🎨 Documentation Structure

```
docs/
└── plugin-development/
    ├── README.md                                    ← Overview
    ├── setup/
    │   ├── vscode-setup.md                         ← IDE configuration
    │   ├── dev-workflow.md                         ← just dev-mode, etc.
    │   └── debugging.md                            ← Debugging guide
    ├── tutorials/
    │   ├── 01-hello-world-frontend.md             ← .fsx tutorial
    │   ├── 02-hello-world-backend.md              ← .fzb tutorial
    │   ├── 03-plugin-api-deep-dive.md             ← API overview
    │   ├── 04-real-plugin-url-shortener.md        ← Complete example
    │   ├── 05-frontend-ui-remoteui.md             ← UI guide
    │   ├── 06-backend-hooks.md                    ← Backend guide
    │   └── 07-testing-and-publishing.md           ← Production
    ├── architecture/
    │   ├── frontend-vs-backend.md                  ← When to use each
    │   ├── plugin-lifecycle.md                     ← Lifecycle diagram
    │   ├── performance.md                          ← Optimization
    │   └── security.md                             ← Security model
    ├── api-reference/
    │   ├── plugin-context.md                       ← PluginContext API
    │   ├── hooks.md                                ← All hooks
    │   ├── remote-ui.md                            ← UI components
    │   └── utilities.md                            ← Helper functions
    └── examples/
        ├── hello-notification/                     ← 8 complete examples
        ├── url-detector/
        ├── command-timer/
        ├── git-status/
        ├── clipboard-history/
        ├── quick-notes/
        ├── auto-cd/
        └── theme-switcher/
```

## 🧪 Testing

### Automated Checks
- [ ] All code examples compile
- [ ] All plugins in examples/ load successfully
- [ ] All API methods documented
- [ ] All hooks documented

### Manual Review
- [ ] Complete tutorial series (7 parts)
- [ ] Follow setup guide on fresh machine
- [ ] Test `just dev-mode` workflow
- [ ] Verify VSCode extensions work

## 📊 Success Criteria

- [ ] `just dev-mode` provides hot-reload workflow
- [ ] VSCode fully configured for plugin development
- [ ] 7-part tutorial series complete
- [ ] Architecture guide clearly explains .fsx vs .fzb
- [ ] Complete API reference with examples
- [ ] 8 example plugins demonstrating patterns
- [ ] Developers (us) can reference this for core plugins
- [ ] External users can create plugins from scratch

## 🔗 Related Issues

- Issue #24: Atuin Plugin (example of hybrid plugin)
- Issue #25: Interactive Tutorial (references plugin docs)
- Issue #26: Reference Documentation (complementary)

---

**Priority**: 🔴 CRITICAL
**Effort**: 4-5 days
**Assignee**: Technical Writer + AI Engineer
