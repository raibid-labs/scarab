# Installation

This page provides instructions for installing Scarab on your system.

## Quick Links

For detailed installation instructions, see:
- [Installation Guide](../../../user/installation.md) - Complete installation documentation

## Prerequisites

- **Rust 1.75+** with Cargo ([Install Rust](https://rustup.rs/))
- **Linux** with X11 or Wayland (macOS/Windows support planned)
- **Git** (for cloning repository)

## Platform-Specific Dependencies

### Ubuntu/Debian
```bash
sudo apt install build-essential pkg-config libfontconfig-dev
```

### Fedora/RHEL
```bash
sudo dnf install gcc pkg-config fontconfig-devel
```

### Arch Linux
```bash
sudo pacman -S base-devel fontconfig
```

## From Source (Recommended for Alpha)

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

## Next Steps

Once installed, proceed to the [Quick Start](./quickstart.md) guide to launch Scarab.
