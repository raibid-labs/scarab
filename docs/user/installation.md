# Installation Guide

Scarab is a high-performance terminal emulator built with Rust. This guide covers installation on all supported platforms.

## Quick Start

### From Cargo (Recommended)

```bash
cargo install scarab-terminal
```

### From Source

```bash
git clone https://github.com/yourusername/scarab.git
cd scarab
cargo build --release
sudo cp target/release/scarab /usr/local/bin/
```

## Platform-Specific Installation

### macOS

#### Homebrew (Recommended)

```bash
brew tap scarab-terminal/scarab
brew install scarab
```

#### Manual Installation

1. Download the latest `.dmg` from [Releases](https://github.com/yourusername/scarab/releases)
2. Open the `.dmg` file
3. Drag Scarab.app to your Applications folder

#### Build from Source

```bash
# Install dependencies
brew install rust

# Clone and build
git clone https://github.com/yourusername/scarab.git
cd scarab
cargo build --release

# Install
sudo cp target/release/scarab /usr/local/bin/
```

### Linux

#### Arch Linux (AUR)

```bash
yay -S scarab-terminal
# or
paru -S scarab-terminal
```

#### Ubuntu/Debian

```bash
# Download the .deb package
wget https://github.com/yourusername/scarab/releases/latest/download/scarab_amd64.deb

# Install
sudo dpkg -i scarab_amd64.deb

# Install dependencies if needed
sudo apt-get install -f
```

#### Fedora/RHEL

```bash
# Download the .rpm package
wget https://github.com/yourusername/scarab/releases/latest/download/scarab.x86_64.rpm

# Install
sudo dnf install scarab.x86_64.rpm
```

#### AppImage

```bash
# Download
wget https://github.com/yourusername/scarab/releases/latest/download/Scarab.AppImage

# Make executable
chmod +x Scarab.AppImage

# Run
./Scarab.AppImage
```

#### Build from Source

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install build dependencies
sudo apt-get update
sudo apt-get install -y \
    cmake pkg-config libfreetype6-dev \
    libfontconfig1-dev libxcb-xfixes0-dev \
    libxkbcommon-dev python3

# Clone and build
git clone https://github.com/yourusername/scarab.git
cd scarab
cargo build --release

# Install
sudo cp target/release/scarab /usr/local/bin/
```

### Windows

#### WinGet

```powershell
winget install Scarab.Terminal
```

#### Chocolatey

```powershell
choco install scarab-terminal
```

#### Manual Installation

1. Download the latest `.exe` installer from [Releases](https://github.com/yourusername/scarab/releases)
2. Run the installer
3. Follow the installation wizard

#### Build from Source

```powershell
# Install Rust
# Download and run rustup-init.exe from https://rustup.rs

# Clone and build
git clone https://github.com/yourusername/scarab.git
cd scarab
cargo build --release

# The binary will be at target\release\scarab.exe
```

## Verification

After installation, verify Scarab is installed correctly:

```bash
scarab --version
```

You should see output like:
```
scarab 0.1.0
```

## Configuration

Scarab will create a default configuration file on first run:

- **Linux/macOS**: `~/.config/scarab/config.toml`
- **Windows**: `%APPDATA%\scarab\config.toml`

See the [Configuration Guide](configuration.md) for customization options.

## Troubleshooting

### Command not found

If you get "command not found" after installation:

1. Ensure the installation directory is in your PATH
2. For cargo installations, add `~/.cargo/bin` to your PATH:
   ```bash
   echo 'export PATH="$HOME/.cargo/bin:$PATH"' >> ~/.bashrc
   source ~/.bashrc
   ```

### Permission denied

If you get permission errors:

```bash
# Linux/macOS
sudo chmod +x /usr/local/bin/scarab

# Cargo installation
chmod +x ~/.cargo/bin/scarab
```

### Missing dependencies (Linux)

If you encounter dependency errors:

```bash
# Ubuntu/Debian
sudo apt-get install -y libfontconfig1 libfreetype6 libxcb-xfixes0

# Fedora/RHEL
sudo dnf install fontconfig freetype libxcb

# Arch Linux
sudo pacman -S fontconfig freetype2 libxcb
```

### Graphics issues

For GPU-related issues:

1. Update your graphics drivers
2. Try software rendering:
   ```bash
   SCARAB_RENDERER=software scarab
   ```

## Next Steps

- [Quick Start Guide](quickstart.md)
- [Configuration Reference](configuration.md)
- [Keybindings](keybindings.md)
- [Plugin Guide](plugins.md)

## Getting Help

- [GitHub Issues](https://github.com/yourusername/scarab/issues)
- [Documentation](https://docs.scarab-terminal.org)
- [Discord Community](https://discord.gg/scarab)

## Uninstallation

### Cargo

```bash
cargo uninstall scarab-terminal
```

### Homebrew

```bash
brew uninstall scarab
```

### AUR

```bash
yay -R scarab-terminal
```

### apt/deb

```bash
sudo apt-get remove scarab
```

### Manual

```bash
sudo rm /usr/local/bin/scarab
rm -rf ~/.config/scarab  # Remove config
```
