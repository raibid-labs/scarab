# Scarab Terminal vX.Y.Z

> [One-line summary of the release - what's the big news?]

**Release Date**: YYYY-MM-DD
**Release Type**: [Stable / Beta / Alpha / Release Candidate / Hotfix]
**Download**: [Binaries](#installation) | [Source](https://github.com/raibid-labs/scarab/archive/refs/tags/vX.Y.Z.tar.gz)

---

## Highlights

[2-4 bullet points of the most important changes/features in this release]

- **[Feature/Fix Name]**: Brief description of impact
- **[Feature/Fix Name]**: Brief description of impact
- **[Feature/Fix Name]**: Brief description of impact

---

## What's Changed

### Added

New features and capabilities:

- [Feature description with context] ([#123](link-to-pr))
- [Feature description with context] ([#456](link-to-pr))

### Changed

Behavior changes and improvements:

- [Change description] ([#789](link-to-pr))
- [Change description] ([#012](link-to-pr))

### Deprecated

Features marked for future removal (still work, but show warnings):

- [Deprecated feature] - Use [replacement] instead. Will be removed in vX.Y.Z
- [Deprecated feature] - Use [replacement] instead. Will be removed in vX.Y.Z

### Removed

Features removed in this release:

- [Removed feature] - Deprecated since vX.Y.Z
- [Removed feature] - Deprecated since vX.Y.Z

### Fixed

Bug fixes:

- Fix [description of bug] ([#345](link-to-issue))
- Fix [description of bug] ([#678](link-to-issue))

### Security

Security fixes (if any):

- **[CVE-YYYY-XXXXX]**: [Description] - Severity: [High/Medium/Low]
- [Security improvement description]

### Performance

Performance improvements:

- [Improvement description] - [X%] faster/[Xms] reduction
- [Improvement description] - [X%] less memory usage

---

## Breaking Changes

[If MAJOR version or contains breaking changes]

This release contains breaking changes. Please review before upgrading:

### 1. [Breaking Change Title]

**What changed**: [Description of the change]

**Why**: [Reason for the change]

**Migration**:
```rust
// Before
old_api_usage()

// After
new_api_usage()
```

**Affected**: [Who/what is affected]

### 2. [Breaking Change Title]

[Continue for each breaking change...]

---

## Installation

### Quick Install

#### macOS (Homebrew)
```bash
brew tap raibid-labs/scarab
brew install scarab

# Or upgrade if already installed
brew upgrade scarab
```

#### Arch Linux (AUR)
```bash
yay -S scarab-terminal
# or
paru -S scarab-terminal
```

#### Cargo (All Platforms)
```bash
cargo install scarab-client scarab-daemon
```

### Binary Downloads

Download pre-built binaries for your platform:

| Platform | Architecture | Download |
|----------|-------------|----------|
| macOS | Apple Silicon (M1/M2/M3) | [scarab-vX.Y.Z-aarch64-apple-darwin.tar.gz](link) |
| macOS | Intel (x86_64) | [scarab-vX.Y.Z-x86_64-apple-darwin.tar.gz](link) |
| Linux | x86_64 (glibc) | [scarab-vX.Y.Z-x86_64-unknown-linux-gnu.tar.gz](link) |
| Linux | x86_64 (musl) | [scarab-vX.Y.Z-x86_64-unknown-linux-musl.tar.gz](link) |
| Linux | ARM64 | [scarab-vX.Y.Z-aarch64-unknown-linux-gnu.tar.gz](link) |
| Windows | x86_64 (MSVC) | [scarab-vX.Y.Z-x86_64-pc-windows-msvc.zip](link) |

#### Verification

Verify downloads with checksums:
```bash
# Download checksums file
curl -LO https://github.com/raibid-labs/scarab/releases/download/vX.Y.Z/checksums.txt

# Verify (Linux/macOS)
sha256sum -c checksums.txt

# Verify specific file
sha256sum scarab-vX.Y.Z-<platform>.tar.gz
```

---

## Upgrade Guide

### From vA.B.C to vX.Y.Z

[For MINOR or MAJOR upgrades with significant changes]

#### Prerequisites

- [Any required dependency updates]
- [Backup recommendations]

#### Steps

1. **Backup your configuration**
   ```bash
   cp ~/.config/scarab/config.toml ~/.config/scarab/config.toml.backup
   ```

2. **Update Scarab**
   ```bash
   # Via Homebrew
   brew upgrade scarab

   # Via Cargo
   cargo install --force scarab-client scarab-daemon
   ```

3. **Migrate configuration** (if needed)
   ```bash
   # Run migration tool (if applicable)
   scarab-daemon --migrate-config
   ```

4. **Restart daemon**
   ```bash
   # Stop old daemon
   killall scarab-daemon

   # Start new daemon
   scarab-daemon &
   ```

5. **Verify**
   ```bash
   scarab-daemon --version
   scarab-client --version
   ```

#### Configuration Changes

[If configuration format changed]

Old format (`~/.config/scarab/config.toml`):
```toml
[old]
key = "value"
```

New format:
```toml
[new]
key = "value"
```

---

## Known Issues

[List any known issues or limitations]

- **[Issue title]**: [Description and workaround if available] - [#issue-number](link)
- **[Issue title]**: [Description and workaround if available] - [#issue-number](link)

For other issues, see the [issue tracker](https://github.com/raibid-labs/scarab/issues).

---

## Platform Support

| Platform | Status | Notes |
|----------|--------|-------|
| Linux (X11) | ✅ Fully Supported | Primary development platform |
| Linux (Wayland) | ✅ Fully Supported | Bevy 0.15 with full Wayland support |
| macOS (Apple Silicon) | ✅ Fully Supported | M1/M2/M3 native |
| macOS (Intel) | ✅ Fully Supported | x86_64 binaries |
| Windows | 🚧 Experimental | [Known limitations](link) |

**Minimum Requirements**:
- Rust 1.75+ (for building from source)
- 4GB RAM recommended
- GPU with OpenGL 3.3+ / Vulkan / Metal

---

## Acknowledgments

### Contributors

This release was made possible by contributions from:

[Auto-generated contributor list or manual list]

- [@username](https://github.com/username) - [Contribution description]
- [@username](https://github.com/username) - [Contribution description]

**First-time contributors** 🎉:
- [@username](https://github.com/username)

### Special Thanks

- [Thank specific contributors for major features]
- [Acknowledge bug reporters]
- [Thank community for feedback]

---

## Resources

- **Documentation**: https://github.com/raibid-labs/scarab/tree/main/docs
- **CHANGELOG**: [CHANGELOG.md](https://github.com/raibid-labs/scarab/blob/main/CHANGELOG.md)
- **Issue Tracker**: https://github.com/raibid-labs/scarab/issues
- **Discussions**: https://github.com/raibid-labs/scarab/discussions

---

## Support

If you encounter issues:

1. Check [Known Issues](#known-issues) above
2. Search [existing issues](https://github.com/raibid-labs/scarab/issues)
3. If not found, [create a new issue](https://github.com/raibid-labs/scarab/issues/new)

For questions and discussions:
- [GitHub Discussions](https://github.com/raibid-labs/scarab/discussions)
- [Community Discord](link-if-exists)

---

## What's Next

Looking ahead to v[NEXT]:

- [Planned feature]
- [Planned feature]
- [Planned feature]

See the full [ROADMAP.md](https://github.com/raibid-labs/scarab/blob/main/ROADMAP.md) for details.

---

**Full Changelog**: [v[PREVIOUS]...v[CURRENT]](https://github.com/raibid-labs/scarab/compare/v[PREVIOUS]...v[CURRENT])
