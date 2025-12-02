# Plugin Development Workflow

A practical guide to the complete plugin development lifecycle, from idea to published plugin.

## Overview

The Scarab plugin development workflow is designed for rapid iteration:

1. **Create** - Generate plugin skeleton from template
2. **Develop** - Write code with hot reload feedback
3. **Test** - Verify functionality with automated tests
4. **Build** - Compile to optimized bytecode
5. **Package** - Bundle for distribution
6. **Publish** - Share with the community

This guide walks through each step with practical examples and best practices.

## The Development Cycle

```
┌─────────────┐
│   Create    │  just plugin-new <name> <type>
│   Plugin    │
└──────┬──────┘
       │
       v
┌─────────────┐
│   Write     │  Edit .fsx file
│   Code      │  Implement hooks and logic
└──────┬──────┘
       │
       v
┌─────────────┐
│  Hot Reload │  just dev-mode <name>
│  Testing    │  Watch for changes, auto-reload
└──────┬──────┘
       │
       v
┌─────────────┐
│   Build     │  just plugin-build <name>
│  (Optional) │  Compile to .fzb bytecode
└──────┬──────┘
       │
       v
┌─────────────┐
│   Package   │  just plugin-package <name>
│   & Share   │  Create distributable archive
└─────────────┘
```

## Step 1: Creating a New Plugin

### Using the Template Generator

The fastest way to start is with the plugin template:

```bash
cd /home/beengud/tmp/scarab
just plugin-new my-awesome-plugin frontend
```

**Arguments**:
- `my-awesome-plugin` - Plugin name (lowercase, hyphens allowed)
- `frontend` - Plugin type (`frontend` or `backend`)

**What gets created**:
```
plugins/my-awesome-plugin/
├── my-awesome-plugin.fsx    # Plugin source code
├── plugin.toml               # Manifest and configuration
└── README.md                 # Documentation template
```

### Plugin Types Explained

#### Frontend Plugins (`frontend`)

**Runtime**: Fusabi Frontend (interprets .fsx source files)
**Location**: Client process (Bevy GUI)
**Use cases**:
- UI overlays and widgets
- Keyboard shortcuts and input handling
- Visual effects and themes
- Client-side features

**Common hooks**:
- `OnKeyPress` - Intercept keyboard input
- `OnLoad` - Initialize UI components
- `OnResize` - Respond to terminal size changes

**Example**:
```fsharp
[<OnKeyPress>]
let onKeyPress (ctx: PluginContext) (key: KeyEvent) =
    async {
        match key.Code with
        | Key.F1 when key.Modifiers.Contains(Ctrl) ->
            ctx.ShowNotification("Help", "Ctrl+F1 pressed!")
            return Handled
        | _ -> return Continue
    }
```

#### Backend Plugins (`backend`)

**Runtime**: Fusabi VM (runs compiled .fzb bytecode)
**Location**: Daemon process (headless server)
**Use cases**:
- Output scanning and parsing
- Command interception and modification
- PTY event handling
- High-performance data processing

**Common hooks**:
- `OnOutput` - Process terminal output line-by-line
- `OnPreCommand` - Intercept commands before execution
- `OnPostCommand` - React to command completion

**Example**:
```fsharp
[<OnOutput>]
let onOutput (ctx: PluginContext) (line: string) =
    async {
        if line.Contains("ERROR") then
            ctx.Log Error (sprintf "Detected error: %s" line)
            ctx.TriggerAlert("Error detected in output")
        return Continue
    }
```

## Step 2: Hot Reload Development

Hot reload is the fastest way to develop plugins. Changes are automatically detected and applied without restarting Scarab.

### Starting Dev Mode

```bash
just dev-mode my-awesome-plugin
```

**What happens**:
1. Builds the plugin initially
2. Starts `cargo-watch` to monitor file changes
3. Automatically rebuilds and reloads on every save
4. Shows compilation errors in terminal

**Output**:
```
🔄 Starting dev mode for my-awesome-plugin
   Watching: plugins/my-awesome-plugin
   Press Ctrl+C to stop

[Running 'just plugin-build my-awesome-plugin']
🔨 Building plugin: my-awesome-plugin
   Source: plugins/my-awesome-plugin/my-awesome-plugin.fsx
✅ Plugin built successfully

[Watching for changes...]
```

### Development Loop

1. **Edit** your `.fsx` file in VSCode
2. **Save** (Ctrl+S)
3. **Watch** terminal for build status
4. **Test** in running Scarab instance
5. **Iterate**

**Example session**:
```bash
# Terminal 1: Start dev mode
just dev-mode clipboard-monitor

# Terminal 2: Start Scarab
cargo run -p scarab-daemon &
cargo run -p scarab-client

# Terminal 3: Watch logs
tail -f ~/.local/share/scarab/plugins.log
```

### How Hot Reload Works

**Frontend plugins (.fsx)**:
- File watcher detects changes
- Scarab client reloads .fsx source
- Plugin re-initialized immediately
- No daemon restart needed

**Backend plugins (.fzb)**:
- File watcher triggers rebuild
- New bytecode compiled
- Daemon reloads plugin
- Current state preserved where possible

### Requirements

Hot reload requires `cargo-watch`:
```bash
cargo install cargo-watch
```

## Step 3: Testing Plugins

### Interactive Testing

The fastest way to test is through direct interaction:

1. **Start Scarab** with your plugin enabled
2. **Trigger hooks** by performing actions
3. **Check logs** for debug output:
   ```bash
   tail -f ~/.local/share/scarab/plugins.log
   ```

### Automated Testing

Run plugin tests with:
```bash
just plugin-test my-awesome-plugin
```

### Manual Testing Checklist

Before marking a plugin as complete:

- [ ] Plugin loads without errors
- [ ] All hooks execute as expected
- [ ] No crashes or panics
- [ ] Configuration options work
- [ ] Cleanup happens on unload
- [ ] No memory leaks (check with long-running test)
- [ ] Performance is acceptable
- [ ] Error messages are helpful

## Step 4: Building Plugins

### Development Builds

During development, you typically work with source files (.fsx) directly. But for production or distribution, compile to bytecode:

```bash
just plugin-build my-awesome-plugin
```

**What happens**:
- Reads `plugins/my-awesome-plugin/my-awesome-plugin.fsx`
- Compiles with Fusabi compiler
- Generates optimized bytecode
- Outputs `plugins/my-awesome-plugin/my-awesome-plugin.fzb`

### Source vs Bytecode

| Aspect | .fsx (Source) | .fzb (Bytecode) |
|--------|---------------|-----------------|
| **Speed** | Slower (interpreted) | Faster (VM execution) |
| **Size** | Larger | Smaller (compressed) |
| **Debugging** | Easy (readable source) | Harder (binary) |
| **Hot Reload** | Yes | Requires rebuild |
| **Distribution** | Can expose code | Protected/obfuscated |
| **Use Case** | Development | Production |

**Recommendation**: Use .fsx during development, compile to .fzb for release.

## Step 5: Packaging Plugins

### Creating a Distribution Package

Package your plugin for sharing:

```bash
just plugin-package my-awesome-plugin
```

**What gets packaged**:
```
dist/plugins/my-awesome-plugin.tar.gz
├── my-awesome-plugin.fsx    (or .fzb if compiled)
├── plugin.toml
└── README.md
```

### Installation from Package

Users can install your plugin with:

```bash
# Download package
wget https://example.com/my-awesome-plugin.tar.gz

# Extract to plugins directory
tar -xzf my-awesome-plugin.tar.gz -C ~/.config/scarab/plugins/

# Enable in config
echo '[[plugins]]
name = "my-awesome-plugin"
enabled = true' >> ~/.config/scarab/config.toml
```

## Complete Workflow Example

Let's build a "git status indicator" plugin from start to finish.

### 1. Create the Plugin

```bash
just plugin-new git-status-indicator backend
```

### 2. Implement the Logic

Edit `plugins/git-status-indicator/git-status-indicator.fsx`:

```fsharp
module GitStatusIndicator

open Scarab.PluginApi
open System.IO
open System.Diagnostics

[<Plugin>]
let metadata = {
    Name = "git-status-indicator"
    Version = "1.0.0"
    Description = "Shows git branch and status in terminal"
    Author = "Your Name"
}

let getGitStatus (directory: string) : string option =
    try
        if Directory.Exists(Path.Combine(directory, ".git")) then
            let psi = ProcessStartInfo()
            psi.FileName <- "git"
            psi.Arguments <- "branch --show-current"
            psi.WorkingDirectory <- directory
            psi.RedirectStandardOutput <- true
            psi.UseShellExecute <- false

            use proc = Process.Start(psi)
            let branch = proc.StandardOutput.ReadToEnd().Trim()
            proc.WaitForExit()

            Some (sprintf "git:%s" branch)
        else
            None
    with
    | _ -> None

[<OnLoad>]
let onLoad (ctx: PluginContext) =
    async {
        ctx.Log Info "Git status indicator loaded"
        return Ok ()
    }

[<OnPromptRender>]
let onPromptRender (ctx: PluginContext) (cwd: string) =
    async {
        match getGitStatus cwd with
        | Some status ->
            ctx.SetOverlay("git-branch", status, { Fg = 0x88FF88FFu; ZIndex = 10.0f })
            return ()
        | None ->
            ctx.ClearOverlay("git-branch")
            return ()
    }
```

### 3. Configure the Plugin

Edit `plugins/git-status-indicator/plugin.toml`:

```toml
[plugin]
name = "git-status-indicator"
version = "1.0.0"
runtime = "backend"

[plugin.metadata]
description = "Shows git branch and status in terminal"
author = "Your Name"
license = "MIT"
homepage = "https://github.com/yourname/git-status-indicator"

[hooks]
on_load = true
on_prompt_render = true

[config]
color = "green"
position = "right"
```

### 4. Start Development

```bash
# Terminal 1: Hot reload
just dev-mode git-status-indicator

# Terminal 2: Run Scarab
cargo run -p scarab-daemon &
cargo run -p scarab-client

# Terminal 3: Watch logs
tail -f ~/.local/share/scarab/plugins.log
```

### 5. Test

Navigate to a git repository in Scarab and verify the branch name appears.

### 6. Build for Production

```bash
just plugin-build git-status-indicator
```

### 7. Package for Distribution

```bash
just plugin-package git-status-indicator
```

### 8. Share

Upload `dist/plugins/git-status-indicator.tar.gz` to GitHub releases or plugin registry.

## Tips for Efficient Development

### Use VSCode Tasks

Instead of typing commands, use VSCode tasks (Ctrl+Shift+P > "Tasks: Run Task"):
- **Watch Plugin (Hot Reload)** - Start dev mode
- **Compile Plugin to .fzb** - Build plugin
- **Test Plugin** - Run tests
- **Package Plugin** - Create distribution

### Keep Logs Visible

Always have a terminal tailing the log file:
```bash
tail -f ~/.local/share/scarab/plugins.log | grep -i "my-plugin"
```

### Use Git Branches

Create a branch for each plugin feature:
```bash
git checkout -b plugin/git-status-indicator
# ... develop plugin ...
git add plugins/git-status-indicator
git commit -m "feat(plugin): add git status indicator"
```

### Leverage Examples

Study existing plugins for patterns:
```bash
ls plugins/examples/
# clipboard-history, command-timer, git-status, etc.
```

### Version Control

**What to commit**:
- `plugins/my-plugin/my-plugin.fsx` (source)
- `plugins/my-plugin/plugin.toml` (manifest)
- `plugins/my-plugin/README.md` (docs)

**What to ignore** (add to .gitignore):
- `plugins/my-plugin/*.fzb` (generated bytecode)
- `dist/plugins/*.tar.gz` (packages)
- `*.log` (logs)

## Troubleshooting Common Issues

### Issue: "Plugin not found"

**Cause**: Wrong plugin name or directory structure

**Solution**:
```bash
# Check plugin exists
ls plugins/my-plugin/my-plugin.fsx

# Ensure name matches directory
# plugins/my-plugin/my-plugin.fsx (correct)
# plugins/my-plugin/plugin.fsx (wrong)
```

### Issue: "cargo-watch not found"

**Solution**:
```bash
cargo install cargo-watch
```

### Issue: Hot reload not triggering

**Solution**:
1. Check cargo-watch is running
2. Verify you're saving files (Ctrl+S)
3. Try manual rebuild: `just plugin-build <name>`

### Issue: Plugin loads but doesn't work

**Solution**:
1. Check logs: `tail -f ~/.local/share/scarab/plugins.log`
2. Verify hooks are declared in plugin.toml
3. Add debug logging to each hook

## Quick Command Reference

```bash
# Create new plugin
just plugin-new <name> <frontend|backend>

# Hot reload development
just dev-mode <name>

# Build plugin
just plugin-build <name>

# Test plugin
just plugin-test <name>

# Package plugin
just plugin-package <name>

# Build all plugins
just plugin-build-all

# Show plugin status
just plugin-status

# Clean build artifacts
just plugin-clean
```

## Resources

- **Plugin Examples**: `/plugins/examples/`
- **API Reference**: `/docs/plugin-development/api-reference/`
- **VSCode Setup**: [vscode-setup.md](./vscode-setup.md)
- **Debugging Guide**: [debugging.md](./debugging.md)
- **Fusabi Language**: [https://github.com/fusabi-lang/fusabi](https://github.com/fusabi-lang/fusabi)
