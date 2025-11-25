# Getting Started with Scarab Terminal

**Learn the basics and get productive in 5 minutes**

---

## Table of Contents

1. [Installation](#installation)
2. [First Launch](#first-launch)
3. [Basic Usage](#basic-usage)
4. [Configuration Basics](#configuration-basics)
5. [Next Steps](#next-steps)

---

## Installation

Scarab currently supports Linux with X11 or Wayland. macOS and Windows support are planned.

### Prerequisites

Before installing Scarab, ensure you have:

- **Rust 1.75+** with Cargo ([Install Rust](https://rustup.rs/))
- **Git** (for cloning the repository)
- **Build tools** (gcc, pkg-config, etc.)

### Platform-Specific Dependencies

**Ubuntu/Debian:**
```bash
sudo apt install build-essential pkg-config libfontconfig-dev
```

**Fedora/RHEL:**
```bash
sudo dnf install gcc pkg-config fontconfig-devel
```

**Arch Linux:**
```bash
sudo pacman -S base-devel fontconfig
```

### Building from Source

Since Scarab is in alpha, building from source is the recommended installation method:

```bash
# Clone the repository
git clone https://github.com/raibid-labs/scarab.git
cd scarab

# Build with optimizations (takes 3-5 minutes)
cargo build --release

# Binaries will be in target/release/
ls -lh target/release/scarab-*
```

You should see:
- `scarab-daemon` - The headless server (owns PTY processes)
- `scarab-client` - The GPU-accelerated GUI

### Optional: Add to PATH

For convenience, add the binaries to your PATH:

```bash
# Add to ~/.bashrc or ~/.zshrc
export PATH="$HOME/path/to/scarab/target/release:$PATH"

# Reload your shell
source ~/.bashrc  # or source ~/.zshrc
```

---

## First Launch

Scarab uses a split-process architecture: a daemon (server) and client (GUI). You need to start both.

### Step 1: Start the Daemon

The daemon is the "brain" of Scarab - it owns terminal processes and persists state.

```bash
# Start the daemon (keeps running in background)
scarab-daemon

# Or with debug logging
RUST_LOG=debug scarab-daemon
```

You should see:
```
Scarab Daemon v0.1.0-alpha
Shared memory initialized at: /scarab_shm_v1
IPC socket listening at: /tmp/scarab.sock
Session database: ~/.local/share/scarab/sessions.db
Loading plugins from: ~/.config/scarab/plugins/
Ready for client connections.
```

**Tip:** The daemon can run in the background. Consider using `tmux` or `systemd` for auto-start.

### Step 2: Launch the Client

In a **separate terminal**, start the GUI client:

```bash
# Launch the client
scarab-client

# Or with debug logging
RUST_LOG=debug scarab-client
```

A Bevy window will open with your terminal. The interactive tutorial will launch automatically on first run.

### Step 3: Complete the Tutorial

Follow the on-screen instructions to learn Scarab's key features:

1. **Welcome** - Introduction to Scarab
2. **Navigation** - Running commands
3. **Scrollback** - Mouse wheel scrolling
4. **Link Hints** - Keyboard-driven URL opening (Ctrl+Shift+O)
5. **Command Palette** - Quick command access (Ctrl+Shift+P)
6. **Plugins** - Plugin system overview
7. **Configuration** - Customization basics
8. **Completion** - Next steps

**Skip the tutorial:** Press `ESC` at any time
**Replay later:** Run with `scarab-client --tutorial`

---

## Basic Usage

### Running Commands

Scarab works just like any terminal emulator:

```bash
# Navigate your filesystem
cd ~/projects
ls -la

# Run commands
git status
cargo build

# Use interactive tools
vim file.rs
htop
```

### Scrollback Navigation

Scarab maintains a scrollback buffer (default: 10,000 lines):

- **Mouse Wheel** - Scroll up/down through history
- **Shift+PageUp/PageDown** - Jump by page
- **Ctrl+Home/End** - Jump to top/bottom

### Link Detection

Scarab automatically detects URLs, file paths, and more:

1. Output a URL: `echo "https://github.com/raibid-labs/scarab"`
2. Press **Ctrl+Shift+O** to highlight all links
3. Press the shown key (e.g., `a`, `b`, `c`) to open

**Supported patterns:**
- URLs: `https://example.com`, `http://localhost:8080`
- File paths: `/home/user/file.txt`, `./src/main.rs`
- Git commits: `abc123f` (in git repositories)

### Command Palette

Access all Scarab features with **Ctrl+Shift+P**:

```
> session new          # Create new session
> theme monokai       # Switch theme
> plugin reload       # Reload plugins
> config edit         # Open config file
```

Use fuzzy search: type `sess new` to find "session new"

### Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| `Ctrl+Shift+O` | Open link hints |
| `Ctrl+Shift+P` | Open command palette |
| `Ctrl+Shift+C` | Copy selection |
| `Ctrl+Shift+V` | Paste |
| `Ctrl+Shift+F` | Search in scrollback |
| `Ctrl+Shift+N` | New window |
| `Ctrl+Shift+T` | New tab (upcoming) |

---

## Configuration Basics

### Config File Location

Scarab looks for configuration in:
```
~/.config/scarab/config.toml
```

If it doesn't exist, Scarab uses sensible defaults.

### Create Your First Config

Create the config directory and file:

```bash
mkdir -p ~/.config/scarab
touch ~/.config/scarab/config.toml
```

Add basic configuration:

```toml
# ~/.config/scarab/config.toml

[terminal]
columns = 120        # Terminal width in characters
rows = 40            # Terminal height in characters
scrollback = 10000   # Lines of scrollback history

[font]
family = "JetBrains Mono"  # Font name
size = 14.0                # Font size in points
bold = true                # Use bold for bold text

[theme]
name = "dracula"  # Built-in themes: dracula, monokai, solarized-dark

[performance]
fps_limit = 60           # Max FPS (0 = unlimited)
vsync = true             # Enable VSync
texture_cache_size = 256 # Texture atlas size in MB

[plugins]
auto_load = true         # Auto-load plugins on startup
search_paths = [
    "~/.config/scarab/plugins",
    "/usr/share/scarab/plugins"
]
```

### Apply Configuration

Scarab supports **hot-reload** - just save the file:

```bash
# Edit config
vim ~/.config/scarab/config.toml

# Save the file - changes apply within 100ms!
# No need to restart Scarab
```

### Verify Configuration

Check if your config is valid:

```bash
# Daemon will log any config errors
tail -f /tmp/scarab-daemon.log
```

Or use the command palette:
```
Ctrl+Shift+P > config validate
```

---

## Next Steps

### 1. Customize Your Setup

Read the [Customization Guide](./02-customization.md) to learn about:
- Themes and color schemes
- Font configuration and ligatures
- Custom keybindings
- GPU rendering options

### 2. Create Your First Plugin

Follow the [Plugin Development Tutorial](./03-plugin-development.md):
- Write a simple "hello world" plugin in F#
- Understand the plugin API
- Learn about hot-reloading

### 3. Explore Example Plugins

Check out `examples/plugins/` for inspiration:
- `git-status.fsx` - Git repository status overlay
- `notification-monitor.fsx` - Desktop notifications for long commands
- `output-filter.fsx` - Filter and highlight terminal output
- `session-manager.fsx` - Advanced session management

### 4. Join the Community

- **GitHub Discussions:** Ask questions and share ideas
- **Report Issues:** Help us improve by reporting bugs
- **Contribute:** Submit PRs for features or documentation

### 5. Advanced Workflows

Read [Workflow Integration](./03-workflows.md) for:
- Git workflow integration
- Docker development
- SSH session management
- Remote development with VS Code

---

## Troubleshooting

### Daemon Won't Start

**Error:** `Failed to create shared memory`

**Solution:**
```bash
# Check if old shared memory exists
ls -la /dev/shm/scarab*

# Remove old shared memory
rm /dev/shm/scarab*

# Restart daemon
scarab-daemon
```

### Client Can't Connect

**Error:** `Failed to open shared memory. Is the daemon running?`

**Solution:**
```bash
# Check if daemon is running
ps aux | grep scarab-daemon

# Check IPC socket
ls -la /tmp/scarab.sock

# Restart daemon
pkill scarab-daemon
scarab-daemon
```

### High CPU Usage

**Solution:**
```toml
# In ~/.config/scarab/config.toml
[performance]
fps_limit = 60  # Limit frame rate
vsync = true    # Enable VSync
```

### Font Not Found

**Error:** `Failed to load font: JetBrains Mono`

**Solution:**
```bash
# Install JetBrains Mono
# Ubuntu/Debian:
sudo apt install fonts-jetbrains-mono

# Or use a system font
# In config.toml:
[font]
family = "Monospace"  # Generic monospace
```

---

## Quick Reference

### File Locations

| Path | Purpose |
|------|---------|
| `~/.config/scarab/config.toml` | Main configuration |
| `~/.config/scarab/plugins/` | User plugins |
| `~/.local/share/scarab/sessions.db` | Session database |
| `/tmp/scarab.sock` | IPC socket |
| `/scarab_shm_v1` | Shared memory |

### Common Commands

```bash
# Start daemon
scarab-daemon

# Start client
scarab-client

# Start with tutorial
scarab-client --tutorial

# Build from source
cargo build --release --workspace

# Run tests
cargo test --workspace

# Check config validity
cargo run -p scarab-daemon -- --validate-config
```

---

**Next:** [Customization Guide](./02-customization.md) - Learn to personalize Scarab

**Back to:** [README](../../README.md)
