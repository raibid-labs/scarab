# Platform Support Documentation

## Overview

Scarab Terminal provides native support for macOS, Linux, and Windows with platform-specific optimizations and packaging.

## Supported Platforms

| Platform | Architecture | Graphics Backend | Package Manager | Status |
|----------|-------------|------------------|-----------------|--------|
| macOS 12+ | Apple Silicon (M1/M2) | Metal | Homebrew | ✅ Stable |
| macOS 11+ | Intel x86_64 | Metal | Homebrew | ✅ Stable |
| Linux | x86_64 | Vulkan/X11/Wayland | AUR, Cargo | ✅ Stable |
| Linux | ARM64/aarch64 | Vulkan/X11/Wayland | Cargo | 🔄 Beta |
| Windows 10/11 | x86_64 | DirectX 12 | Cargo, Installer | ✅ Stable |
| Windows 10/11 | ARM64 | DirectX 12 | Cargo | 🔄 Beta |

## Installation

### macOS

#### Homebrew (Recommended)

```bash
# Add the Scarab tap
brew tap raibid-labs/scarab

# Install Scarab
brew install scarab

# Start the daemon service
brew services start scarab

# Launch the terminal
scarab-terminal
```

#### From Source

```bash
# Requires Rust 1.75+
git clone https://github.com/raibid-labs/scarab
cd scarab
cargo install --path crates/scarab-client
cargo install --path crates/scarab-daemon
```

### Linux

#### Arch Linux (AUR)

```bash
# Using yay
yay -S scarab-terminal

# Using paru
paru -S scarab-terminal

# Manual installation
git clone https://aur.archlinux.org/scarab-terminal.git
cd scarab-terminal
makepkg -si
```

#### Debian/Ubuntu

```bash
# Download the latest release
wget https://github.com/raibid-labs/scarab/releases/latest/download/scarab-linux-x86_64.tar.gz

# Extract
tar xzf scarab-linux-x86_64.tar.gz

# Install to /usr/local
sudo mv scarab-daemon scarab-client /usr/local/bin/
```

#### Fedora/RHEL

```bash
# Coming soon - RPM package
# For now, use cargo install
cargo install scarab-client scarab-daemon
```

#### From Source

```bash
# Install dependencies
# Ubuntu/Debian
sudo apt install build-essential pkg-config libx11-dev libxi-dev \
    libxcursor-dev libxrandr-dev libvulkan-dev

# Fedora
sudo dnf install gcc pkg-config libX11-devel libXi-devel \
    libXcursor-devel libXrandr-devel vulkan-loader-devel

# Arch
sudo pacman -S base-devel pkgconf libx11 libxi libxcursor \
    libxrandr vulkan-icd-loader

# Build and install
cargo install --path crates/scarab-client
cargo install --path crates/scarab-daemon
```

### Windows

#### Using Cargo

```powershell
# Requires Visual Studio 2019+ with C++ tools
cargo install scarab-client scarab-daemon
```

#### Windows Installer

Download the latest `.msi` installer from the [releases page](https://github.com/raibid-labs/scarab/releases).

#### From Source

```powershell
# Clone the repository
git clone https://github.com/raibid-labs/scarab
cd scarab

# Build (requires Visual Studio)
cargo build --release

# Binaries will be in target/release/
```

## Platform-Specific Features

### macOS

- **Graphics**: Native Metal backend for optimal performance
- **IPC**: Unix domain sockets at `/tmp/scarab.sock`
- **Keybindings**: macOS-specific (Cmd instead of Ctrl)
- **Integration**: Touchbar support (optional)
- **Configuration**: `~/Library/Application Support/Scarab/`

### Linux

- **Graphics**: Vulkan (preferred), OpenGL fallback
- **Display Servers**: Both X11 and Wayland supported
- **IPC**: Unix domain sockets at `$XDG_RUNTIME_DIR/scarab.sock`
- **Desktop Integration**: `.desktop` file for application menus
- **System Tray**: StatusNotifierItem protocol support
- **Configuration**: `$XDG_CONFIG_HOME/scarab/` or `~/.config/scarab/`

### Windows

- **Graphics**: DirectX 12 (preferred), Vulkan fallback
- **IPC**: Named Pipes at `\\.\pipe\scarab`
- **Shell Integration**: PowerShell and CMD support
- **Windows Terminal**: Can be used as a profile
- **Configuration**: `%APPDATA%\Scarab\`

## Graphics Backend Selection

Scarab automatically selects the optimal graphics backend for your platform:

| Platform | Primary | Fallback |
|----------|---------|----------|
| macOS | Metal | - |
| Linux | Vulkan | OpenGL |
| Windows | DirectX 12 | Vulkan |

To override the automatic selection:

```toml
# ~/.config/scarab/config.toml
[gpu]
backend = "vulkan"  # Options: "metal", "vulkan", "dx12", "opengl", "auto"
```

## Cross-Compilation

### Building for Multiple Targets

```bash
# Install cross-compilation targets
rustup target add aarch64-apple-darwin
rustup target add x86_64-unknown-linux-gnu
rustup target add x86_64-pc-windows-msvc

# Build for all platforms
./scripts/build-all-platforms.sh

# Build for specific target
cargo build --release --target aarch64-apple-darwin
```

### Docker Build Environment

```dockerfile
# Dockerfile for cross-compilation
FROM rust:1.75

RUN apt-get update && apt-get install -y \
    mingw-w64 \
    gcc-aarch64-linux-gnu \
    musl-tools

RUN rustup target add \
    x86_64-pc-windows-gnu \
    aarch64-unknown-linux-gnu \
    x86_64-unknown-linux-musl

WORKDIR /build
```

## Binary Size Optimization

Scarab uses several techniques to keep binary size under 10MB:

1. **Link-time Optimization (LTO)**: `lto = "thin"`
2. **Single Codegen Unit**: `codegen-units = 1`
3. **Strip Debug Symbols**: `strip = true`
4. **Size-optimized Profile**: `opt-level = "z"`

### Size Comparison

| Platform | Debug | Release | Optimized |
|----------|-------|---------|-----------|
| macOS ARM64 | 45MB | 12MB | 8.2MB |
| macOS x86_64 | 48MB | 13MB | 8.8MB |
| Linux x86_64 | 52MB | 14MB | 9.1MB |
| Windows x86_64 | 55MB | 15MB | 9.5MB |

## Troubleshooting

### macOS

**Issue**: "scarab-daemon" cannot be opened because the developer cannot be verified.

**Solution**:
```bash
# Remove quarantine attribute
xattr -d com.apple.quarantine /usr/local/bin/scarab-daemon
```

### Linux

**Issue**: Vulkan not found

**Solution**:
```bash
# Install Vulkan drivers
# Intel
sudo apt install mesa-vulkan-drivers

# AMD
sudo apt install mesa-vulkan-drivers

# NVIDIA
sudo apt install nvidia-driver-XXX
```

**Issue**: Wayland permission denied

**Solution**:
```bash
# Ensure XDG_RUNTIME_DIR is set
export XDG_RUNTIME_DIR="/run/user/$(id -u)"
```

### Windows

**Issue**: Named pipe access denied

**Solution**:
Run as Administrator or adjust Windows Defender settings.

**Issue**: Missing VCRUNTIME140.dll

**Solution**:
Install [Visual C++ Redistributable](https://support.microsoft.com/en-us/help/2977003/).

## Performance Tuning

### GPU Selection

For systems with multiple GPUs:

```toml
[gpu]
# Prefer discrete GPU
prefer_discrete = true

# Or specify by index
device_index = 0
```

### Memory Usage

```toml
[performance]
# Limit GPU memory usage (MB)
max_gpu_memory = 512

# Terminal buffer size
scrollback_lines = 10000
```

### Power Management

```toml
[power]
# Laptop mode - reduces GPU usage
low_power_mode = true

# Frame rate limiting
max_fps = 60
```

## Development

### Platform Detection

```rust
use scarab_platform::{current_platform, Platform};

let platform = current_platform();
println!("Running on: {}", platform.platform_name());

// Platform-specific code
#[cfg(target_os = "macos")]
{
    // macOS-specific implementation
}
```

### IPC Implementation

```rust
use scarab_platform::ipc;

// Create server (platform-agnostic)
let server = ipc::create_server("scarab", &Default::default())?;

// Accept connections
let stream = server.accept()?;
```

## Contributing

Platform-specific contributions are welcome! Please test on your target platform:

1. Run the test suite: `cargo test --all`
2. Run platform tests: `cargo test -p scarab-platform`
3. Test IPC functionality
4. Verify graphics backend selection
5. Check package installation

## License

MIT - See LICENSE file for details.