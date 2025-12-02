# VSCode Setup for Scarab Plugin Development

A comprehensive guide to configuring Visual Studio Code for optimal Scarab plugin development experience.

## Introduction

VSCode is the recommended editor for Scarab plugin development because:
- Excellent Rust support via rust-analyzer
- Native F#/Fusabi syntax highlighting through Ionide
- Integrated debugging with LLDB
- Built-in terminal for running `just` commands
- Rich extension ecosystem

This guide will help you set up a professional development environment optimized for building Scarab plugins.

## Required Extensions

### 1. rust-analyzer

**Purpose**: Essential Rust language server for intelligent code completion, type hints, and inline errors.

**Installation**:
```bash
code --install-extension rust-lang.rust-analyzer
```

**What it provides**:
- Real-time type checking and error detection
- Intelligent code completion for Rust APIs
- Inline documentation on hover
- Go-to-definition for Scarab API types
- Automatic imports and refactoring tools

**Configuration** (already in workspace):
```json
{
  "rust-analyzer.cargo.features": "all",
  "rust-analyzer.check.command": "clippy",
  "rust-analyzer.check.extraArgs": ["--", "-D", "warnings"]
}
```

### 2. Ionide-fsharp

**Purpose**: F# language support for writing Fusabi plugin scripts (.fsx files).

**Installation**:
```bash
code --install-extension ionide.ionide-fsharp
```

**What it provides**:
- F# syntax highlighting for .fsx files
- Type inference and inline type annotations
- Autocomplete for Fusabi API functions
- Tree view for F# project structure
- Integrated tooltips for function signatures

**Configuration** (already in workspace):
```json
{
  "FSharp.enableTreeView": true,
  "FSharp.inlayHints.enabled": true,
  "FSharp.inlayHints.typeAnnotations": true
}
```

### 3. Even Better TOML

**Purpose**: Syntax highlighting and validation for `plugin.toml` manifest files.

**Installation**:
```bash
code --install-extension tamasfe.even-better-toml
```

**What it provides**:
- Schema validation for plugin.toml
- Autocomplete for TOML keys
- Format-on-save for TOML files
- Error detection for malformed TOML

### 4. Error Lens

**Purpose**: Display inline errors and warnings directly in the editor.

**Installation**:
```bash
code --install-extension usernamehw.errorlens
```

**What it provides**:
- Inline error messages in your code (no need to hover)
- Color-coded error severity (red = error, yellow = warning)
- Instant feedback on compilation errors
- Better visibility of Rust clippy warnings

**Why it's essential**: When developing plugins, you'll be switching between Rust (daemon) and F# (plugins). Error Lens gives you instant visual feedback in both languages without context switching.

## Recommended Extensions

These extensions enhance the development experience but aren't strictly required:

### CodeLLDB (Debugging)

**Installation**:
```bash
code --install-extension vadimcn.vscode-lldb
```

**Purpose**: Native debugger for Rust binaries. Lets you set breakpoints in scarab-daemon and scarab-client.

### GitLens

**Installation**:
```bash
code --install-extension eamodio.gitlens
```

**Purpose**: Enhanced Git integration - see blame annotations, commit history, and more.

### Better Comments

**Installation**:
```bash
code --install-extension aaron-bond.better-comments
```

**Purpose**: Highlighted comments with different styles:
```rust
// TODO: Implement this hook
// ! CRITICAL: Memory safety issue here
// ? Question: Should this be async?
// * Important note about plugin lifecycle
```

### Just (Justfile Support)

**Installation**:
```bash
code --install-extension skellock.just
```

**Purpose**: Syntax highlighting for `justfile` commands. Helpful when reading the Scarab justfile.

### Markdown All in One

**Installation**:
```bash
code --install-extension yzhang.markdown-all-in-one
```

**Purpose**: Enhanced markdown editing for plugin README files.

## Workspace Settings Explained

The Scarab repository includes a `.vscode/settings.json` file with optimized defaults. Here's what each section does:

### Rust Settings
```json
{
  "[rust]": {
    "editor.defaultFormatter": "rust-lang.rust-analyzer",
    "editor.formatOnSave": true
  }
}
```
**Effect**: Automatically formats Rust code with `rustfmt` when you save files.

### F# Settings
```json
{
  "[fsharp]": {
    "editor.defaultFormatter": "ionide.ionide-fsharp",
    "editor.formatOnSave": true,
    "editor.tabSize": 4
  }
}
```
**Effect**: Formats F# plugin code with 4-space indentation (F# convention).

### File Associations
```json
{
  "files.associations": {
    "*.fsx": "fsharp",
    "*.fzb": "binary",
    "justfile": "makefile"
  }
}
```
**Effect**:
- `.fsx` files open with F# syntax highlighting
- `.fzb` bytecode files open in hex editor (not text editor)
- `justfile` gets Makefile-like syntax highlighting

### File Exclusions
```json
{
  "files.exclude": {
    "**/.git": true,
    "**/target": true,
    "**/*.fzb": false
  }
}
```
**Effect**: Hides `target/` directory from file tree (it's huge) but shows `.fzb` compiled plugins.

### Terminal Settings
```json
{
  "terminal.integrated.env.linux": {
    "RUST_BACKTRACE": "1"
  }
}
```
**Effect**: Enables full stack traces in integrated terminal for better Rust debugging.

## Keyboard Shortcuts for Plugin Development

### Default Shortcuts

These work out of the box:

| Shortcut | Action | Use Case |
|----------|--------|----------|
| `Ctrl+Shift+B` | Run default build task | Build entire workspace |
| `Ctrl+Shift+P` | Command palette | Run any task or command |
| `F5` | Start debugging | Debug daemon or client |
| `Ctrl+` | Toggle terminal | Quick access to just commands |
| `Ctrl+P` | Quick open file | Navigate to plugin.fsx files |
| `Ctrl+Shift+F` | Global search | Find API usage across codebase |

### Custom Shortcuts (Recommended)

Add these to your `keybindings.json` (Ctrl+Shift+P > "Preferences: Open Keyboard Shortcuts (JSON)"):

```json
[
  {
    "key": "ctrl+shift+r",
    "command": "workbench.action.tasks.runTask",
    "args": "Watch Plugin (Hot Reload)"
  },
  {
    "key": "ctrl+shift+t",
    "command": "workbench.action.tasks.runTask",
    "args": "Test Plugin"
  },
  {
    "key": "ctrl+shift+c",
    "command": "workbench.action.tasks.runTask",
    "args": "Compile Plugin to .fzb"
  }
]
```

**Effect**:
- `Ctrl+Shift+R`: Start hot reload watcher for current plugin
- `Ctrl+Shift+T`: Run plugin tests
- `Ctrl+Shift+C`: Compile plugin to bytecode

## Snippets for Common Patterns

### Installing Snippets

1. Open Command Palette (Ctrl+Shift+P)
2. Type "Configure User Snippets"
3. Select "fsharp.json" (for F# plugins)

### Fusabi Plugin Snippets

Add these to your `fsharp.json` snippets file:

```json
{
  "Scarab Plugin Boilerplate": {
    "prefix": "scarab-plugin",
    "body": [
      "module ${1:PluginName}",
      "",
      "open Scarab.PluginApi",
      "",
      "[<Plugin>]",
      "let metadata = {",
      "    Name = \"${1:PluginName}\"",
      "    Version = \"${2:0.1.0}\"",
      "    Description = \"${3:Plugin description}\"",
      "    Author = \"${4:Your Name}\"",
      "}",
      "",
      "[<OnLoad>]",
      "let onLoad (ctx: PluginContext) =",
      "    ctx.Log Info \"${1:PluginName} loaded!\"",
      "    async { return Ok () }",
      "",
      "$0"
    ],
    "description": "Create a new Scarab plugin boilerplate"
  },

  "OnOutput Hook": {
    "prefix": "on-output",
    "body": [
      "[<OnOutput>]",
      "let onOutput (ctx: PluginContext) (line: string) =",
      "    async {",
      "        ${1:// Process output line}",
      "        return Continue",
      "    }"
    ],
    "description": "Add OnOutput hook"
  },

  "OnKeyPress Hook": {
    "prefix": "on-key",
    "body": [
      "[<OnKeyPress>]",
      "let onKeyPress (ctx: PluginContext) (key: KeyEvent) =",
      "    async {",
      "        match key.Modifiers, key.Code with",
      "        | Ctrl, Key.${1:C} -> ",
      "            ${2:// Handle Ctrl+C}",
      "            return Handled",
      "        | _ -> return Continue",
      "    }"
    ],
    "description": "Add OnKeyPress hook"
  },

  "Plugin Command": {
    "prefix": "plugin-command",
    "body": [
      "[<Command(\"${1:command-id}\", \"${2:Command description}\")>]",
      "let ${3:commandName} (ctx: PluginContext) =",
      "    async {",
      "        ${4:// Command implementation}",
      "        return Ok ()",
      "    }"
    ],
    "description": "Add a plugin command"
  }
}
```

### Usage

Type the prefix and press Tab:
- `scarab-plugin` + Tab = Full plugin template
- `on-output` + Tab = OnOutput hook skeleton
- `on-key` + Tab = OnKeyPress hook skeleton
- `plugin-command` + Tab = Command registration

## Troubleshooting Common VSCode Issues

### Issue 1: rust-analyzer Not Working

**Symptoms**: No autocomplete, no type hints, "rust-analyzer failed to load" error

**Solutions**:
1. Check Rust is installed: `rustc --version`
2. Ensure you're in the workspace root (where Cargo.toml is)
3. Reload VSCode: Ctrl+Shift+P > "Developer: Reload Window"
4. Check rust-analyzer output: View > Output > Select "rust-analyzer" from dropdown
5. Update rust-analyzer: `rustup update`

### Issue 2: Ionide Not Highlighting .fsx Files

**Symptoms**: .fsx files look like plain text, no syntax highlighting

**Solutions**:
1. Check file association: .fsx should map to "fsharp" language
2. Install .NET SDK if missing: `sudo apt install dotnet-sdk-8.0` (or equivalent)
3. Restart VSCode after installing Ionide
4. Check language mode in bottom-right corner - click and select "F#"

### Issue 3: Tasks Not Appearing

**Symptoms**: Can't find "Watch Plugin" or other tasks in task runner

**Solutions**:
1. Ensure you opened VSCode in the scarab/ root directory
2. Check `.vscode/tasks.json` exists
3. Reload window: Ctrl+Shift+P > "Developer: Reload Window"
4. Try running manually: Ctrl+Shift+P > "Tasks: Run Task"

### Issue 4: Format-on-Save Not Working

**Symptoms**: Rust or F# files don't auto-format when saving

**Solutions**:
1. Check formatters are installed (rust-analyzer, Ionide)
2. Verify settings.json has `"editor.formatOnSave": true`
3. Check file-specific settings: `"[rust]": { "editor.formatOnSave": true }`
4. Ensure no conflicting formatter is installed

### Issue 5: Debugger Won't Start

**Symptoms**: F5 does nothing or shows "Could not find debug adapter"

**Solutions**:
1. Install CodeLLDB extension: `code --install-extension vadimcn.vscode-lldb`
2. Check `.vscode/launch.json` exists
3. Ensure Rust binaries are built: `cargo build`
4. Try "Debug Scarab Daemon" from Run and Debug panel (Ctrl+Shift+D)

### Issue 6: Terminal Doesn't Show Colors

**Symptoms**: `just` commands work but output is monochrome

**Solutions**:
1. Enable color support: Add to settings.json:
   ```json
   {
     "terminal.integrated.env.linux": {
       "TERM": "xterm-256color"
     }
   }
   ```
2. Restart terminal instance

## Recommended Workflow Setup

### Terminal Layout

For plugin development, use a split terminal layout:

1. Open integrated terminal (Ctrl+`)
2. Click "+" dropdown > Select "Split Terminal"
3. In left pane: `just dev-mode <plugin-name>` (hot reload watcher)
4. In right pane: `tail -f ~/.local/share/scarab/plugins.log` (live logs)

### Editor Layout

Recommended window arrangement:

```
┌─────────────────────┬─────────────────────┐
│                     │                     │
│  plugin.fsx         │  plugin.toml        │
│  (main code)        │  (config)           │
│                     │                     │
├─────────────────────┴─────────────────────┤
│                                           │
│  Terminal (split view)                    │
│  [dev-mode]  |  [logs]                    │
│                                           │
└───────────────────────────────────────────┘
```

To set up:
1. Open plugin.fsx
2. Ctrl+\ (split editor right)
3. Open plugin.toml in new pane
4. Ctrl+` (open terminal)
5. Click split button in terminal

### Multi-Root Workspace (Advanced)

If you develop multiple plugins, create a multi-root workspace:

1. File > Add Folder to Workspace
2. Add: scarab/ (main repo)
3. Add: ~/.config/scarab/plugins/ (your plugins)
4. File > Save Workspace As... > scarab-dev.code-workspace

**Benefits**:
- Quick navigation between repo and plugins
- Unified search across all plugins
- Consistent settings across projects

## Performance Tips

### Exclude Large Directories

Add to workspace settings.json:
```json
{
  "search.exclude": {
    "**/target": true,
    "**/node_modules": true,
    "**/.git": true
  }
}
```

**Effect**: Faster file search and reduced indexing time.

### Limit rust-analyzer Scope

If rust-analyzer is slow:
```json
{
  "rust-analyzer.workspace.symbol.search.scope": "workspace",
  "rust-analyzer.cargo.buildScripts.enable": false
}
```

### Disable Unused Extensions

Extensions run in the background and use resources. Disable extensions you don't need:
1. Extensions panel (Ctrl+Shift+X)
2. Right-click unused extensions
3. Select "Disable (Workspace)"

## Next Steps

Now that your editor is configured:

1. **Test the setup**: Try creating a plugin with `just plugin-new test-plugin frontend`
2. **Explore tasks**: Open Command Palette > "Tasks: Run Task" to see available commands
3. **Read the workflow guide**: [dev-workflow.md](./dev-workflow.md)
4. **Learn debugging**: [debugging.md](./debugging.md)

## Quick Reference

### Essential Commands
- Build workspace: `Ctrl+Shift+B`
- Run task: `Ctrl+Shift+P` > "Tasks: Run Task"
- Debug: `F5`
- Terminal: `Ctrl+`
- Command palette: `Ctrl+Shift+P`

### Files to Know
- `.vscode/settings.json` - Workspace settings
- `.vscode/tasks.json` - Build/plugin tasks
- `.vscode/launch.json` - Debug configurations
- `.vscode/extensions.json` - Recommended extensions

### Getting Help
- rust-analyzer docs: [rust-analyzer.github.io](https://rust-analyzer.github.io)
- Ionide docs: [ionide.io](https://ionide.io)
- VSCode docs: [code.visualstudio.com/docs](https://code.visualstudio.com/docs)
- Scarab Discord: [Ask in #plugin-dev channel]
