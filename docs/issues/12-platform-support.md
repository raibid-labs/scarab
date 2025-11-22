# Issue #12: Cross-Platform Support & Packaging

**Phase**: 4C - Production Hardening
**Priority**: 🟡 High
**Workstream**: Platform Engineering
**Estimated Effort**: 2 weeks
**Assignee**: Platform Engineer Agent

---

## 🎯 Objective

Enable full Linux (X11/Wayland), macOS (Metal), and Windows (DirectX) support with automated packaging for major package managers.

---

## 📋 Background

Current state:
- macOS development (primary)
- Basic Linux support
- Windows untested

We need:
- Platform-specific optimizations
- Windows Named Pipes (vs Unix sockets)
- Graphics backend selection
- Package automation
- Release workflow

---

## ✅ Acceptance Criteria

- [ ] Linux support (X11 + Wayland)
- [ ] macOS support (Metal backend)
- [ ] Windows support (DirectX + Named Pipes)
- [ ] Platform detection and auto-config
- [ ] Cross-compilation scripts
- [ ] Homebrew formula (macOS)
- [ ] AUR package (Arch Linux)
- [ ] Cargo install support
- [ ] GitHub Releases automation
- [ ] Single binary per platform (<10MB)

---

## 🔧 Technical Approach

### Step 1: Platform Abstraction
```rust
// src/platform/mod.rs

#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "windows")]
mod windows;

pub trait Platform {
    fn socket_path() -> PathBuf;
    fn config_dir() -> PathBuf;
    fn data_dir() -> PathBuf;
}

#[cfg(target_os = "macos")]
pub use macos::MacPlatform as CurrentPlatform;

#[cfg(target_os = "linux")]
pub use linux::LinuxPlatform as CurrentPlatform;

#[cfg(target_os = "windows")]
pub use windows::WindowsPlatform as CurrentPlatform;
```

### Step 2: Windows Named Pipes
```rust
// src/platform/windows.rs

use winapi::um::namedpipeapi::*;

pub struct WindowsIpc {
    pipe: HANDLE,
}

impl WindowsIpc {
    pub fn create_server(name: &str) -> Result<Self> {
        let pipe_name = format!(r"\\.\pipe\{}", name);

        let pipe = unsafe {
            CreateNamedPipeA(
                pipe_name.as_ptr() as *const i8,
                PIPE_ACCESS_DUPLEX,
                PIPE_TYPE_BYTE | PIPE_READMODE_BYTE,
                PIPE_UNLIMITED_INSTANCES,
                4096, 4096,
                0,
                null_mut(),
            )
        };

        Ok(Self { pipe })
    }
}
```

### Step 3: Graphics Backend Selection
```rust
// Cargo.toml

[dependencies.bevy]
version = "0.15"
default-features = false
features = [
    "bevy_winit",
    "bevy_core_pipeline",
    "bevy_ui",
    "bevy_render",
]

[target.'cfg(target_os = "macos")'.dependencies.bevy]
features = ["metal"]

[target.'cfg(target_os = "linux")'.dependencies.bevy]
features = ["x11", "wayland"]

[target.'cfg(target_os = "windows")'.dependencies.bevy]
features = ["dx12"]
```

### Step 4: Cross-Compilation
```bash
# scripts/build-all-platforms.sh

#!/bin/bash
set -e

# macOS (Apple Silicon)
cargo build --release --target aarch64-apple-darwin
strip target/aarch64-apple-darwin/release/scarab-*

# macOS (Intel)
cargo build --release --target x86_64-apple-darwin
strip target/x86_64-apple-darwin/release/scarab-*

# Linux (x86_64)
cargo build --release --target x86_64-unknown-linux-gnu
strip target/x86_64-unknown-linux-gnu/release/scarab-*

# Windows (x86_64)
cargo build --release --target x86_64-pc-windows-msvc
```

### Step 5: Homebrew Formula
```ruby
# Formula/scarab.rb

class Scarab < Formula
  desc "GPU-accelerated terminal emulator with plugin system"
  homepage "https://github.com/raibid-labs/scarab"
  url "https://github.com/raibid-labs/scarab/archive/v0.1.0.tar.gz"
  sha256 "..."
  license "MIT"

  depends_on "rust" => :build

  def install
    system "cargo", "install", *std_cargo_args(path: ".")
  end

  test do
    system "#{bin}/scarab-daemon", "--version"
  end
end
```

### Step 6: AUR Package
```bash
# PKGBUILD

pkgname=scarab-terminal
pkgver=0.1.0
pkgrel=1
pkgdesc="GPU-accelerated terminal emulator with plugin system"
arch=('x86_64')
url="https://github.com/raibid-labs/scarab"
license=('MIT')
depends=('gcc-libs')
makedepends=('rust' 'cargo')
source=("$pkgname-$pkgver.tar.gz::$url/archive/v$pkgver.tar.gz")
sha256sums=('...')

build() {
  cd "$srcdir/$pkgname-$pkgver"
  cargo build --release --locked
}

package() {
  cd "$srcdir/$pkgname-$pkgver"
  install -Dm755 target/release/scarab-daemon "$pkgdir/usr/bin/scarab-daemon"
  install -Dm755 target/release/scarab-client "$pkgdir/usr/bin/scarab"
}
```

### Step 7: Release Automation
```yaml
# .github/workflows/release.yml

name: Release

on:
  push:
    tags:
      - 'v*'

jobs:
  build:
    strategy:
      matrix:
        include:
          - os: macos-latest
            target: aarch64-apple-darwin
          - os: macos-latest
            target: x86_64-apple-darwin
          - os: ubuntu-latest
            target: x86_64-unknown-linux-gnu
          - os: windows-latest
            target: x86_64-pc-windows-msvc

    runs-on: ${{ matrix.os }}

    steps:
      - uses: actions/checkout@v3

      - name: Build
        run: |
          cargo build --release --target ${{ matrix.target }}
          strip target/${{ matrix.target }}/release/scarab-*

      - name: Package
        run: |
          cd target/${{ matrix.target }}/release
          tar czf scarab-${{ github.ref_name }}-${{ matrix.target }}.tar.gz scarab-*

      - name: Upload Release Asset
        uses: actions/upload-release-asset@v1
        with:
          upload_url: ${{ github.event.release.upload_url }}
          asset_path: ./scarab-${{ github.ref_name }}-${{ matrix.target }}.tar.gz
```

---

## 📦 Deliverables

1. **Platform Code**: Linux, macOS, Windows support
2. **Packaging**: Homebrew, AUR, Cargo
3. **CI/CD**: Cross-platform builds
4. **Documentation**: Installation per platform
5. **Release**: Automated GitHub Releases

---

## 🔗 Dependencies

- **Depends On**: All Phase 1-3 complete
- **Blocks**: None

---

## 📚 Resources

- [Cross Platform Rust](https://rust-lang.github.io/rustup/cross-compilation.html)
- [Homebrew Formula](https://docs.brew.sh/Formula-Cookbook)
- [AUR Guidelines](https://wiki.archlinux.org/title/AUR_submission_guidelines)
- [GitHub Releases](https://docs.github.com/en/repositories/releasing-projects-on-github)

---

## 🎯 Success Metrics

- ✅ All platforms supported
- ✅ Binary size <10MB compressed
- ✅ Single-command install
- ✅ Auto-updates working
- ✅ Platform-specific optimizations

---

## 🖥️ Platform-Specific Features

### macOS
- Metal graphics backend
- Unix domain sockets
- macOS-specific keybindings (Cmd vs Ctrl)
- Touchbar support (optional)

### Linux
- X11 + Wayland support
- Proper desktop integration (.desktop file)
- System tray icon
- Font config integration

### Windows
- DirectX 12 backend
- Named Pipes for IPC
- Windows Terminal integration
- PowerShell support

---

**Created**: 2025-11-21
**Labels**: `phase-4`, `high-priority`, `platform`, `packaging`, `ci-cd`
