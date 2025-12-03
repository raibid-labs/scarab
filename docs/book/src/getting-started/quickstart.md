# Quick Start

Get up and running with Scarab in 5 minutes.

## Quick Links

For detailed quick start information, see:
- [Quick Start Guide](../../../user/quickstart.md) - Complete quick start documentation
- [Main README](../../../../README.md#quick-start) - Quick start in the main README

## Starting Scarab

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

## Interactive Tutorial

On first launch, Scarab will guide you through an 8-step interactive tutorial covering:

1. Welcome and introduction
2. Navigation basics
3. Scrollback usage
4. Link hints (Ctrl+Shift+O)
5. Command palette (Ctrl+Shift+P)
6. Plugin system overview
7. Configuration basics
8. Next steps

**Replay anytime:** `scarab-client --tutorial`

## Basic Usage

Once running:
- Type commands as in any terminal
- Use mouse wheel for scrollback
- Press **Ctrl+Shift+O** for link hints
- Press **Ctrl+Shift+P** for command palette

## Next Steps

- Learn about [Navigation](../user-guide/navigation.md)
- Customize [Configuration](./configuration.md)
- Explore [Plugins](../user-guide/plugins.md)
