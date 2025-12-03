# Scarab Terminal Emulator

Scarab is a high-performance, split-process terminal emulator built in Rust.

## Features

- **Split Architecture**: Daemon (headless server) + Client (Bevy-based GUI)
- **Zero-Copy IPC**: Shared memory for bulk data transfer between processes
- **GPU-Accelerated Rendering**: Bevy game engine with cosmic-text
- **ECS-Native Navigation**: Built on Bevy's Entity Component System
- **Hybrid Plugin System**: Fusabi-lang (F# dialect) with dual runtimes

## Architecture

Scarab consists of two main processes:

- **scarab-daemon**: Headless server that owns PTY processes and maintains terminal state
- **scarab-client**: Bevy-based GUI that renders via shared memory

The daemon persists across client crashes/disconnects, providing a robust multiplexing experience.

## Quick Links

- [Getting Started](./user-guide/getting-started.md)
- [Navigation System](./user-guide/navigation.md)
- [Developer Guide](./developer-guide/architecture.md)
- [Configuration Reference](./reference/config-schema.md)

## Project Status

Scarab is under active development. Current focus:

- Phase 0-6: Bevy/ECS navigation system (Complete)
- Phase 7: Fusabi plugin system integration (In Progress)
- Phase 8: Advanced multiplexing features (Planned)

For detailed roadmap, see [Full Documentation](../../README.md).
