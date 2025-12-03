# Getting Started

## Prerequisites

- Rust 1.70 or later
- Linux (X11 or Wayland)

## Installation

### Building from Source

```bash
# Clone the repository
git clone https://github.com/raibid-labs/scarab.git
cd scarab

# Build the workspace
cargo build --release
```

### Running Scarab

1. **Start the daemon** (in one terminal):
   ```bash
   cargo run -p scarab-daemon --release
   ```

2. **Start the client** (in another terminal):
   ```bash
   cargo run -p scarab-client --release
   ```

## Basic Usage

Once the client is running, you can:

- Create new tabs/panes
- Navigate between panes using keyboard shortcuts
- Configure keybindings and appearance

For detailed configuration options, see the [Configuration Guide](./configuration.md).

## Next Steps

- Learn about [Navigation](./navigation.md)
- Customize [Keybindings](./keybindings.md)
- Explore the [Developer Guide](../developer-guide/architecture.md)
