# Scarab Terminal - Installation Quick Start

## One-Line Install (Recommended)

```bash
curl -sSf https://raw.githubusercontent.com/raibid-labs/scarab/main/scripts/install.sh | bash
```

Or download and inspect first:

```bash
curl -sSf https://raw.githubusercontent.com/raibid-labs/scarab/main/scripts/install.sh -o install.sh
bash install.sh
```

---

## Platform-Specific Installation

### macOS

**Homebrew (Coming Soon)**:
```bash
brew tap raibid-labs/scarab
brew install scarab
```

**Manual**:
```bash
# Download latest release
VERSION="v0.1.0-alpha.7"
curl -LO "https://github.com/raibid-labs/scarab/releases/download/$VERSION/scarab-$VERSION-$(uname -m)-apple-darwin.tar.gz"

# Extract and install
tar -xzf scarab-*.tar.gz
sudo mv scarab-* /usr/local/bin/
```

### Linux

**Ubuntu/Debian**:
```bash
# Coming soon: .deb packages
wget https://github.com/raibid-labs/scarab/releases/latest/download/scarab_amd64.deb
sudo dpkg -i scarab_amd64.deb
```

**Arch Linux (AUR)**:
```bash
# Coming soon
yay -S scarab-terminal-bin
```

**Universal (Any Linux)**:
```bash
curl -sSf https://raw.githubusercontent.com/raibid-labs/scarab/main/scripts/install.sh | bash
```

### From Source

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Clone and build
git clone https://github.com/raibid-labs/scarab
cd scarab
cargo build --release

# Install
sudo cp target/release/scarab-* /usr/local/bin/
```

---

## Quick Start

1. **Start the daemon**:
   ```bash
   scarab-daemon &
   ```

2. **Launch terminal**:
   ```bash
   scarab
   ```

3. **Configuration** (optional):
   ```bash
   # macOS/Linux
   vi ~/.config/scarab/config.toml
   ```

---

## What Gets Installed

- `scarab-daemon` - Background server (handles PTY, plugins)
- `scarab-client` - GUI client (Bevy-based renderer)
- `scarab` - Symlink to client for convenience
- `scarab-plugin-compiler` - Compile Fusabi plugins (.fsx → .fzb)

---

## System Requirements

- **macOS**: 10.15+ (Catalina or newer)
- **Linux**: Any modern distribution with X11 or Wayland
- **GPU**: Any GPU with Vulkan/Metal support
- **RAM**: 256MB minimum

---

## Uninstall

**Installer-based**:
```bash
rm -rf ~/.local/bin/scarab*
rm -rf ~/.config/scarab
```

**Homebrew**:
```bash
brew uninstall scarab
```

---

## Troubleshooting

### "Command not found"
Add install directory to PATH:
```bash
export PATH="$HOME/.local/bin:$PATH"
```

### Permission denied
```bash
chmod +x ~/.local/bin/scarab*
```

### GPU issues
Use software renderer:
```bash
SCARAB_RENDERER=software scarab
```

---

## Next Steps

- [Configuration Guide](docs/user/configuration.md)
- [Plugin Development](docs/PLUGIN_LOGGING_AND_NOTIFICATIONS.md)
- [GitHub Issues](https://github.com/raibid-labs/scarab/issues)

---

**Version**: v0.1.0-alpha.7
**Last Updated**: 2025-11-24
