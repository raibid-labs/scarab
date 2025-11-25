<div align="center">

```
 ____                      _
/ ___|  ___ __ _ _ __ __ _| |__
\___ \ / __/ _` | '__/ _` | '_ \
 ___) | (_| (_| | | | (_| | |_) |
|____/ \___\__,_|_|  \__,_|_.__/

```

# Scarab Terminal Emulator

**Next-Generation GPU-Accelerated Terminal with F# Plugins**

[![Build Status](https://github.com/raibid-labs/scarab/workflows/CI/badge.svg)](https://github.com/raibid-labs/scarab/actions)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.75%2B-orange.svg)](https://www.rust-lang.org/)
[![Fusabi](https://img.shields.io/badge/Fusabi-0.5.0-purple.svg)](https://github.com/fusabi-lang/fusabi)
[![Bevy](https://img.shields.io/badge/Bevy-0.15-blue.svg)](https://bevyengine.org/)

[Features](#-features) | [Installation](#-installation) | [Quick Start](#-quick-start) | [Plugins](#-plugin-system) | [Documentation](#-documentation) | [Contributing](#-contributing)

</div>

---

## Visual Demos

<div align="center">

### Link Hints - Open URLs with Keyboard

![Link Hints Demo](docs/assets/demos/link-hints-demo.gif)

*Press Ctrl+Shift+O to highlight all links, then press the shown key to open*

---

### Command Palette - Quick Access to Everything

![Command Palette](docs/assets/demos/command-palette.gif)

*Press Ctrl+Shift+P for fuzzy searchable command palette*

---

### Plugin System - Extend with F#

![Plugin Installation](docs/assets/demos/plugin-install.gif)

*Write powerful plugins in Fusabi (F# for Rust) with hot-reload support*

---

### Real-Time Theme Switching

![Theme Switch](docs/assets/demos/theme-switch.gif)

*Switch themes instantly - no restart required*

---

### Watch Full Demos

**Scarab in 2 Minutes** - Quick feature overview
[▶ Watch on YouTube](#) | [📥 Download MP4](docs/videos/scarab-2min-demo.mp4)

**Your First Plugin** - Step-by-step plugin creation
[▶ Watch on YouTube](#) | [📥 Download MP4](docs/videos/first-plugin-tutorial.mp4)

**Advanced Workflows** - Power user tips
[▶ Watch on YouTube](#) | [📥 Download MP4](docs/videos/advanced-workflows.mp4)

</div>

---

## ⚠️ Alpha Software Warning

**Scarab is currently in alpha (v0.1.0-alpha)**. While the core architecture is stable and functional, expect:
- Incomplete features and rough edges
- Potential breaking changes between releases
- Limited platform support (Linux only, macOS/Windows planned)
- Active development with frequent updates

**Current Status**: ~80% of MVP features complete | Phase 5: Integration & Polish

---

## 🚀 What is Scarab?

Scarab is a **high-performance, split-process terminal emulator** built in Rust that reimagines terminal extensibility through:

- **Split Architecture**: Daemon (headless server) + Client (GPU-accelerated GUI) for resilience and flexibility
- **Zero-Copy IPC**: Shared memory ring buffer with lock-free synchronization
- **Hybrid F# Plugin System**: Powered by [Fusabi](https://github.com/fusabi-lang/fusabi) - an official F# dialect for Rust
- **GPU-Accelerated Rendering**: Built on Bevy game engine with cosmic-text
- **Hot-Reloadable UI**: Write UI scripts in F# that reload instantly without recompilation

Built for developers who want a **modern, hackable terminal** with native scripting support.

---

## ✨ Features

### Core Terminal
- ✅ **Full VTE Compatibility**: ANSI escape sequences, colors (256 + true color), scrollback
- ✅ **GPU Rendering**: 60+ FPS at 200x100 cells with cosmic-text texture atlas caching
- ✅ **Session Persistence**: SQLite-backed session management, survives client crashes
- ✅ **Split-Process Design**: Daemon persists terminal state, clients attach/detach freely

### Plugin System (Fusabi-powered)
- ✅ **Dual Runtime Support**:
  - **Client-side** (`.fsx` scripts): Hot-reloadable UI plugins via `fusabi-frontend`
  - **Daemon-side** (`.fzb` bytecode): High-performance compiled plugins via `fusabi-vm`
- ✅ **Rich Plugin API**: 10+ hooks (output, input, resize, pre/post command, remote commands)
- ✅ **Remote UI Protocol**: Daemon plugins can control client UI (overlays, modals, commands)
- ✅ **Example Plugins**:
  - `scarab-nav`: Link hints and keyboard navigation
  - `scarab-palette`: Command palette with fuzzy search
  - `scarab-session`: Session management commands
  - Git status indicators, notification monitors, custom keybindings

### Performance
- 🚀 **Zero-Copy Rendering**: Shared memory with lock-free `AtomicU64` sequence numbers
- 🚀 **Optimized Builds**: LTO, single codegen unit, opt-level 3
- 🚀 **Profiling Support**: Tracy, puffin, criterion benchmarks

### Configuration
- 🔧 **TOML-based** with hot-reload
- 🔧 **F# DSL** for advanced configuration (Fusabi-lang syntax)
- 🔧 **Per-session overrides** with inheritance

---

## 📦 Installation

### Prerequisites
- **Rust 1.75+** with Cargo ([Install Rust](https://rustup.rs/))
- **Linux** with X11 or Wayland (macOS/Windows support planned)
- **Git** (for cloning repository)

### From Source (Recommended for Alpha)

```bash
# Clone the repository
git clone https://github.com/raibid-labs/scarab.git
cd scarab

# Build with optimizations
cargo build --release

# Binaries will be in target/release/
# - scarab-daemon
# - scarab-client
```

### Platform-Specific Notes

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

---

## 🏁 Quick Start

### 1. Start the Daemon (Terminal Server)

In a terminal, run:
```bash
cargo run --release -p scarab-daemon
```

The daemon will:
- Create shared memory at `/scarab_shm_v1`
- Start IPC socket at `/tmp/scarab.sock`
- Initialize session manager with SQLite database
- Load daemon plugins from `~/.config/scarab/plugins/`

### 2. Launch the Client (GUI)

In a **separate terminal**, run:
```bash
cargo run --release -p scarab-client
```

The client will:
- Connect to daemon via IPC socket
- Map shared memory for zero-copy rendering
- Open Bevy window with terminal UI
- Load client UI scripts from `~/.config/scarab/ui-scripts/`
- **Launch interactive tutorial on first run** (press ESC to skip)

### 3. Interactive Tutorial

On first launch, Scarab will guide you through an 8-step interactive tutorial:

1. **Welcome** - Introduction to Scarab
2. **Navigation** - Running commands
3. **Scrollback** - Mouse wheel scrolling
4. **Link Hints** - Keyboard-driven URL opening (Ctrl+Shift+O)
5. **Command Palette** - Quick command access (Ctrl+Shift+P)
6. **Plugins** - Plugin system overview
7. **Configuration** - Customization basics
8. **Completion** - Next steps

**Replay anytime:** `scarab-client --tutorial`

### 4. Create Your First Plugin (5 minutes!)

Create `~/.config/scarab/plugins/hello.fsx`:

```fsharp
(*
 * hello.fsx - My First Scarab Plugin
 * A simple plugin that greets you when loaded
 *)

open Scarab.PluginApi

// Plugin metadata
let metadata = {
    Name = "hello-plugin"
    Version = "1.0.0"
    Description = "My first Scarab plugin"
    Author = "Your Name"
    Homepage = None
    ApiVersion = "0.1.0"
    MinScarabVersion = "0.1.0"
}

// Called when plugin loads
let on_load (ctx: PluginContext) : Async<Result<unit, string>> =
    async {
        ctx.Log(LogLevel.Info, "Hello from my first plugin!")

        // Get terminal size
        let (cols, rows) = ctx.GetSize()
        ctx.Log(LogLevel.Info, sprintf "Terminal size: %dx%d" cols rows)

        return Ok ()
    }

// Export the plugin
Plugin.Register {
    Metadata = metadata
    OnLoad = on_load
    OnUnload = None
    OnOutput = None
    OnInput = None
    OnResize = None
    OnAttach = None
    OnDetach = None
    OnPreCommand = None
    OnPostCommand = None
    OnRemoteCommand = None
    GetCommands = fun () -> []
}
```

Restart the daemon to load your plugin:
```bash
# Ctrl+C the daemon, then:
cargo run --release -p scarab-daemon
```

See your plugin's log messages in the daemon output!

---

## 🔌 Plugin System

### Why Fusabi?

[Fusabi](https://github.com/fusabi-lang/fusabi) is an **official F# dialect scripting language designed for Rust**. It provides:

- **Familiar F# Syntax**: If you know F#, you know Fusabi
- **Type Safety**: Compile-time type checking prevents runtime errors
- **Async/Await**: First-class support for asynchronous operations
- **Zero-Cost FFI**: Direct interop with Rust types
- **Dual Runtimes**: Compiled bytecode (`.fzb`) and interpreted scripts (`.fsx`)

### Plugin Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                      SCARAB CLIENT (GUI)                     │
│                                                              │
│  ┌────────────────────────────────────────────────────┐    │
│  │  UI Scripts (.fsx) - fusabi-frontend               │    │
│  │  • Hot-reloadable (no Rust recompilation)          │    │
│  │  • UI overlays, custom keybindings                 │    │
│  │  • Fast iteration for UI development               │    │
│  └────────────────────────────────────────────────────┘    │
│                                                              │
│  ┌────────────────────────────────────────────────────┐    │
│  │  Bevy Rendering Engine + Shared Memory Reader      │    │
│  └────────────────────────────────────────────────────┘    │
└─────────────────────────────────────────────────────────────┘
                           ▲
                           │ Zero-Copy IPC (Shared Memory)
                           ▼
┌─────────────────────────────────────────────────────────────┐
│                   SCARAB DAEMON (Headless)                   │
│                                                              │
│  ┌────────────────────────────────────────────────────┐    │
│  │  Compiled Plugins (.fzb) - fusabi-vm               │    │
│  │  • High performance (pre-compiled bytecode)        │    │
│  │  • Output filtering, mux logic                     │    │
│  │  • Process monitoring, shell integration           │    │
│  └────────────────────────────────────────────────────┘    │
│                                                              │
│  ┌────────────────────────────────────────────────────┐    │
│  │  PTY Manager + VTE Parser + Session State          │    │
│  └────────────────────────────────────────────────────┘    │
└─────────────────────────────────────────────────────────────┘
```

### Plugin Hooks

Plugins can hook into various terminal lifecycle events:

| Hook | Location | Description |
|------|----------|-------------|
| `OnLoad` | Daemon/Client | Called when plugin is loaded |
| `OnUnload` | Daemon/Client | Called before plugin is unloaded |
| `OnOutput` | Daemon | Intercept terminal output before rendering |
| `OnInput` | Daemon | Intercept keyboard/mouse input before PTY |
| `OnResize` | Daemon | Called when terminal is resized |
| `OnAttach` | Daemon | Called when a client connects |
| `OnDetach` | Daemon | Called when a client disconnects |
| `OnPreCommand` | Daemon | Called before shell command execution |
| `OnPostCommand` | Daemon | Called after shell command completes |
| `OnRemoteCommand` | Daemon | Handle commands from client UI (command palette) |
| `GetCommands` | Daemon | Provide commands for client command palette |

### Example: Git Status Plugin

See a real-world plugin that shows git status in your terminal:

```fsharp
// examples/plugins/git-status.fsx
open Scarab.PluginApi
open System.Diagnostics

let metadata = {
    Name = "git-status"
    Version = "1.0.0"
    Description = "Display git repository status in terminal"
    Author = "Scarab Examples"
    Homepage = Some "https://github.com/raibid-labs/scarab"
    ApiVersion = "0.1.0"
    MinScarabVersion = "0.1.0"
}

// Check if in git repository
let checkGitRepo () : bool =
    try
        let psi = ProcessStartInfo("git", "rev-parse --git-dir")
        psi.RedirectStandardOutput <- true
        psi.UseShellExecute <- false
        use proc = Process.Start(psi)
        proc.WaitForExit()
        proc.ExitCode = 0
    with _ -> false

// Update and draw git status overlay
let updateGitStatus (ctx: PluginContext) =
    if checkGitRepo() then
        // Get current branch
        let branch = (* ... git command ... *)
        let isDirty = (* ... git status ... *)

        // Draw overlay in top-right corner
        ctx.QueueCommand(RemoteCommand.DrawOverlay {
            Id = 10000UL
            X = (* top-right position *)
            Y = 0us
            Text = sprintf " git:%s%s " branch (if isDirty then "*" else "")
            Style = { Fg = 0xFFFFFFFFu; Bg = 0x00AA00FFu; ZIndex = 100.0f }
        })

// Register hooks
Plugin.Register {
    Metadata = metadata
    OnLoad = (* initial status check *)
    OnPostCommand = Some (* update after git/cd commands *)
    OnResize = Some (* reposition overlay *)
    (* ... *)
}
```

**More Examples**: See `examples/plugins/` for:
- `hello-plugin.fsx` - Minimal hello world
- `output-filter.fsx` - Filter and highlight terminal output
- `custom-keybind.fsx` - Custom keyboard shortcuts
- `notification-monitor.fsx` - Desktop notifications for long commands
- `session-manager.fsx` - Session switching and management

---

## 📖 Documentation

### Getting Started
- **[Interactive Tutorial](docs/tutorials/01-getting-started.md)** - Get productive in 5 minutes
- **[Customization Guide](docs/tutorials/02-customization.md)** - Themes, fonts, keybindings
- **[Workflow Integration](docs/tutorials/03-workflows.md)** - Git, Docker, SSH workflows

### Project Documentation
- **[ROADMAP.md](./ROADMAP.md)** - Strategic development roadmap (Phases 1-10)
- **[CLAUDE.md](./CLAUDE.md)** - Architecture overview and build commands
- **[IMPLEMENTATION_SUMMARY.md](./IMPLEMENTATION_SUMMARY.md)** - Technical implementation details
- **[MIGRATION_GUIDE.md](./MIGRATION_GUIDE.md)** - Bevy 0.15 migration guide

### Plugin Development
- **[examples/plugins/](./examples/plugins/)** - Example plugins with detailed comments
- **[examples/plugin-template/](./examples/plugin-template/)** - Starter template for new plugins
- **API Reference**: Run `cargo doc --open --workspace` for full API documentation

### Configuration
- **[examples/fusabi-config/](./examples/fusabi-config/)** - Configuration examples
  - `minimal.fsx` - Minimal configuration
  - `standard.fsx` - Standard configuration with comments
  - `advanced.fsx` - Advanced features
  - `custom-theme.fsx` - Custom color themes

---

## 🏗️ Building from Source

### Full Workspace Build

```bash
# Debug build (faster compilation)
cargo build --workspace

# Release build (optimized, recommended for usage)
cargo build --release --workspace

# Build specific crate
cargo build --release -p scarab-daemon
cargo build --release -p scarab-client
```

### Development Workflow

```bash
# Check all crates for errors (faster than build)
cargo check --workspace

# Run all tests
cargo test --workspace

# Run specific test suite
cargo test -p scarab-client ui_tests

# Run with debug logging
RUST_LOG=debug cargo run -p scarab-daemon
RUST_LOG=debug cargo run -p scarab-client

# Watch mode (requires cargo-watch)
cargo watch -x 'check --workspace'
```

### Performance Profiling

```bash
# Build with profiling symbols
cargo build --profile profiling

# Run benchmarks
cargo bench --workspace

# Generate flamegraph (requires cargo-flamegraph)
cargo flamegraph -p scarab-daemon
```

---

## ⚡ Performance Highlights

- **Zero-Copy IPC**: Shared memory ring buffer eliminates data copying between daemon and client
- **Lock-Free Synchronization**: `AtomicU64` sequence numbers avoid mutex contention
- **GPU-Accelerated**: Bevy engine with cosmic-text texture atlas caching
- **Optimized Builds**: Thin LTO, single codegen unit, aggressive optimizations
- **60+ FPS Rendering**: Smooth scrolling even at 200x100 cell grids

**Benchmark Results** (on reference hardware):
- Startup time: ~50ms (daemon) + ~200ms (client)
- Input latency: <5ms (keyboard to screen)
- Memory usage: ~15MB (daemon) + ~80MB (client with GPU textures)
- Plugin load time: <10ms for typical `.fsx` script

---

## 🗺️ Roadmap

**Current Phase**: Phase 5 - Integration & Polish (~80% complete)

### Completed (Phases 1-4)
- ✅ Core terminal emulation (VTE parser, rendering)
- ✅ Zero-copy IPC with shared memory
- ✅ Plugin system with Fusabi integration
- ✅ Session management and persistence
- ✅ Configuration system with hot-reload
- ✅ Remote UI protocol (daemon controls client UI)
- ✅ Core plugins: nav, palette, session

### In Progress (Phase 5)
- 🔄 Interactive tutorial and onboarding
- 🔄 E2E testing with real daemon-client interaction
- 🔄 UI polish and animations
- 🔄 Documentation and examples
- 🔄 Alpha release preparation

### Upcoming (Phases 6-10)
- 📋 **Phase 6**: Tabs, splits, window management
- 📋 **Phase 7**: macOS and Windows support
- 📋 **Phase 8**: Advanced rendering (ligatures, images, sixel)
- 📋 **Phase 9**: Multiplexing and remote sessions
- 📋 **Phase 10**: Beta release and ecosystem growth

See [ROADMAP.md](./ROADMAP.md) for detailed phase breakdown.

---

## 🤝 Contributing

Scarab is in active development and welcomes contributions! Here's how to get started:

### Ways to Contribute
- 🐛 **Report Bugs**: Open an issue with reproduction steps
- 💡 **Suggest Features**: Discuss new ideas in GitHub Discussions
- 📝 **Improve Documentation**: Fix typos, add examples, clarify guides
- 🔌 **Write Plugins**: Share your plugins in `examples/plugins/`
- 🧪 **Add Tests**: Improve test coverage for any crate
- 🎨 **UI/UX**: Design improvements for Bevy client

### Development Setup

1. **Fork and Clone**:
   ```bash
   git clone https://github.com/YOUR_USERNAME/scarab.git
   cd scarab
   ```

2. **Create a Branch**:
   ```bash
   git checkout -b feature/my-awesome-feature
   ```

3. **Make Changes**:
   ```bash
   # Make your changes
   cargo check --workspace  # Ensure no errors
   cargo test --workspace   # Run tests
   cargo fmt               # Format code
   cargo clippy --workspace # Lint code
   ```

4. **Commit and Push**:
   ```bash
   git add .
   git commit -m "feat: Add my awesome feature"
   git push origin feature/my-awesome-feature
   ```

5. **Open Pull Request**: Create a PR on GitHub with description of changes

### Code Guidelines
- Follow Rust standard formatting (`cargo fmt`)
- Pass clippy lints (`cargo clippy`)
- Add tests for new functionality
- Update documentation for API changes
- Keep commits atomic and well-described

### Questions?
- Join discussions on [GitHub Discussions](https://github.com/raibid-labs/scarab/discussions)
- Read the [CLAUDE.md](./CLAUDE.md) architecture guide
- Check existing [Issues](https://github.com/raibid-labs/scarab/issues) and [PRs](https://github.com/raibid-labs/scarab/pulls)

---

## 📄 License

Scarab is licensed under the **MIT License**. See [LICENSE](LICENSE) for details.

---

## 🙏 Acknowledgments

Scarab stands on the shoulders of giants:

### Core Technologies
- **[Fusabi](https://github.com/fusabi-lang/fusabi)** - F# scripting language for Rust (plugin system foundation)
- **[Bevy](https://bevyengine.org/)** - Data-driven game engine (GPU rendering)
- **[cosmic-text](https://github.com/pop-os/cosmic-text)** - Advanced text shaping and rendering
- **[Alacritty](https://github.com/alacritty/alacritty)** - VTE parser (`alacritty_terminal` crate)

### Inspirations
- **[tmux](https://github.com/tmux/tmux)** - Session persistence and multiplexing concepts
- **[kitty](https://github.com/kovidgoyal/kitty)** - GPU acceleration and protocol innovation
- **[Warp](https://www.warp.dev/)** - Modern terminal UX and command palette
- **[Zellij](https://github.com/zellij-org/zellij)** - Rust terminal multiplexer with plugins

### Community
- **Rust Community** - For the amazing ecosystem
- **Bevy Community** - For support and plugins
- **F# Community** - For inspiration behind Fusabi

---

<div align="center">

**Built with ❤️ by [raibid-labs](https://github.com/raibid-labs)**

**[⬆ Back to Top](#scarab-terminal-emulator)**

</div>
