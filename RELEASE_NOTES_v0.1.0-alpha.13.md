# Scarab Terminal v0.1.0-alpha.13

**Release Date**: November 25, 2025

This release represents a major architectural milestone - completing the migration from TOML-based configuration to Fusabi-based configuration, fixing critical installation issues, and adding several major features requested by the community.

---

## 🚀 Major Features

### Fusabi Configuration System (Breaking Change)

**The whole point of Scarab is to use Fusabi for configuration, not TOML.** This release fixes that fundamental architectural inconsistency.

- ✅ **Fusabi Config Loader** - Configuration now uses `.fsx` files (Fusabi F# syntax)
- ✅ **Backwards Compatible** - Legacy `.toml` configs still work with deprecation warning
- ✅ **Config Location**: `~/.config/scarab/config.fsx`
- ✅ **Fallback Chain**: `.fsx` → `.toml` → defaults
- ⚠️ **Value Extraction WIP** - Config compiles but uses defaults (pending Fusabi language features)

**What This Means:**
```fsharp
// ~/.config/scarab/config.fsx
let scrollback_lines = 10000
let terminal_columns = 80
let terminal_rows = 24
```

Once Fusabi adds float/record support, full configuration will look like:
```fsharp
[<FontConfig>]
let font = {
    Family = "JetBrains Mono"
    Size = 14.0
    LineHeight = 1.2
}
```

See: [Fusabi Feature Request #109](https://github.com/fusabi-lang/fusabi/issues/109)

---

### Interactive Tutorial System

Complete 8-step guided onboarding for new users:

1. **Welcome** - Introduce Scarab
2. **Navigation** - Basic command usage
3. **Scrollback** - Mouse wheel scrolling
4. **Link Hints** - URL detection and opening
5. **Command Palette** - Quick command access
6. **Plugins** - Plugin system overview
7. **Configuration** - Config file location
8. **Completion** - Summary and next steps

**Features:**
- ✅ Automatic first-run detection
- ✅ Beautiful ASCII art overlay
- ✅ Progress bar with step indicators
- ✅ Keyboard navigation (Space/Enter, Backspace, Escape)
- ✅ Manual trigger with `--tutorial` flag
- ✅ Persistent progress tracking

---

### Atuin Shell History Integration

Enhanced shell history with cross-session sync and powerful search.

**Features:**
- ✅ Ctrl+R integration with Atuin search
- ✅ Cross-session history sync
- ✅ Advanced search with filters
- ✅ Auto-detect Atuin installation
- ✅ Graceful fallback to standard history

**Requirements:**
- Atuin CLI installed ([atuin.sh](https://atuin.sh))
- Enable plugin: Add `"scarab-atuin"` to `plugins.enabled`

---

### Scrollback UI Improvements

Complete scrollback buffer implementation:

- ✅ **10,000 line buffer** with LRU eviction
- ✅ **Mouse wheel scrolling** (60 FPS requirement met)
- ✅ **Keyboard navigation** (Shift+PageUp/Down/Home/End)
- ✅ **Text selection** in scrollback (Ctrl+C to copy)
- ✅ **Scroll position indicator**
- ✅ **Search overlay** (Ctrl+F) with regex support

---

### Installation Improvements

#### Just Install Commands

Simple, reliable installation:

```bash
# Install to ~/.local/bin (default)
just install

# Install to custom location
just install PREFIX=/usr/local

# Uninstall
just uninstall
```

**Features:**
- ✅ Automatic target directory detection
- ✅ Tilde expansion working correctly
- ✅ Creates symlink: `scarab` → `scarab-client`
- ✅ Installs daemon, client, and plugin compiler
- ✅ Clear PATH instructions

---

## 🐛 Bug Fixes

### Shared Memory Handling

**Issue**: Daemon crashed with "Shared memory OS specific ID already exists"

**Fix**: Graceful handling of existing shared memory segments
- Try to create new shared memory first
- If exists, open existing segment
- Provide helpful cleanup instructions on failure
- Prevents crashes during development/testing

### Justfile Syntax Errors

**Issues**:
- Plugin-new recipe failed with "Unknown start of token '.'"
- Install command used wrong directory

**Fixes**:
- Fixed heredoc syntax for F# code generation
- Fixed just variable expansion in install recipes
- Renamed duplicate recipes for clarity

---

## 📚 Documentation

### Complete Reference Documentation

- ✅ **Configuration Reference** - All TOML/Fusabi options with defaults
- ✅ **Keybindings Reference** - 50+ shortcuts with platform variations
- ✅ **Troubleshooting Guide** - 25+ common issues with solutions
- ✅ **Performance Tuning** - Optimization strategies
- ✅ **Migration Guides** - From Alacritty, iTerm2, GNOME Terminal

### Plugin Development Documentation

- ✅ **Comprehensive Guide** - 7-part tutorial series
- ✅ **VSCode Extensions** - F# support setup
- ✅ **Hot-Reload Workflow** - `just dev-mode`
- ✅ **Architecture Guide** - When to use .fsx vs .fzb
- ✅ **API Reference** - PluginContext, Hooks, RemoteUI
- ✅ **8 Working Examples** - Production-ready plugin templates

---

## 🔧 Technical Details

### Dependencies

- Fusabi VM: 0.5.0
- Fusabi Frontend: 0.5.0
- Bevy: 0.15
- Rust: 1.75+

### Platforms

Binaries available for:
- Linux (x86_64, ARM64)
- macOS (x86_64, ARM64)
- Windows (x86_64, ARM64)

---

## ⚠️ Known Limitations

### Fusabi Config Extraction (WIP)

**Current Behavior**: Config files compile successfully, but values aren't extracted yet. Scarab uses sensible defaults.

**Status**: ⚠️ Waiting on Fusabi language features
- Float literals → [Issue #109](https://github.com/fusabi-lang/fusabi/issues/109)
- Record types → [Issue #109](https://github.com/fusabi-lang/fusabi/issues/109)

**Workaround**: Use legacy TOML config for full customization

**User Experience**:
```
Loading Fusabi config from: ~/.config/scarab/config.fsx
⚠️  Fusabi config loader is WIP - using defaults
📝 Config file compiled successfully, but extraction not yet implemented
```

### Tutorial Validation (Incomplete)

**Current Behavior**: Tutorial steps advance manually, but don't detect user actions yet.

**Status**: ⚠️ Validation stubs in place, full implementation pending

**User Experience**: Tutorial works perfectly as guided walkthrough, just doesn't auto-advance on completion.

---

## 🎯 What's Next

### Short Term (Blocked on Fusabi)

1. **Config Value Extraction** - Once Fusabi adds float/record support
2. **Full Config DSL** - Rich configuration language with functions
3. **Hot-Reload** - Edit .fsx and reload instantly

### Medium Term

1. **Tutorial Validation** - Auto-detect user actions
2. **Advanced Scrollback** - Fuzzy search, history replay
3. **Plugin Marketplace** - Browse and install plugins

### Long Term

1. **Multiplexer** - Built-in tmux-like functionality
2. **SSH Integration** - Remote terminal sessions
3. **Collaboration** - Shared terminal sessions

---

## 📦 Installation

### Quick Start

```bash
# Install from source
git clone https://github.com/raibid-labs/scarab
cd scarab
just install

# Run Scarab
scarab-daemon &
scarab
```

### Homebrew (Coming Soon)

```bash
brew tap raibid-labs/scarab
brew install scarab
```

---

## 🙏 Acknowledgments

Special thanks to:
- **Fusabi Team** - For building an amazing F# scripting engine
- **Bevy Community** - For the powerful game engine
- **Early Adopters** - For testing and feedback

---

## 🔗 Links

- **Homepage**: https://github.com/raibid-labs/scarab
- **Documentation**: https://github.com/raibid-labs/scarab/tree/main/docs
- **Issues**: https://github.com/raibid-labs/scarab/issues
- **Fusabi Feature Request**: https://github.com/fusabi-lang/fusabi/issues/109

---

## 🆚 Breaking Changes

### Configuration Files

**Changed**: Config file extension and format
- **Old**: `~/.config/scarab/config.toml` (TOML)
- **New**: `~/.config/scarab/config.fsx` (Fusabi)

**Migration**:
- Keep your existing `.toml` file - it still works!
- You'll see: `⚠️ Loading legacy TOML config`
- Migrate at your own pace

**Why**: Aligns with Scarab's design philosophy of using Fusabi for everything.

---

**Full Changelog**: https://github.com/raibid-labs/scarab/compare/v0.1.0-alpha.7...v0.1.0-alpha.13
