# WezTerm Feature Parity

Tracking feature parity with WezTerm terminal emulator.

## Quick Links

For complete parity analysis, see:
- [WezTerm Parity Documentation](../../../wezterm-parity/README.md) - Detailed analysis
- [Gap Analysis](../../../analysis/wezterm-gap-analysis.md) - Feature comparison

## Overview

WezTerm is a mature, feature-rich terminal emulator. Scarab aims for parity with WezTerm's core features while offering unique advantages.

## Feature Comparison

### ✅ Implemented

| Feature | Scarab | WezTerm | Notes |
|---------|--------|---------|-------|
| GPU Acceleration | ✅ Bevy | ✅ Custom | Both GPU-accelerated |
| Configuration Files | ✅ TOML | ✅ Lua | Different languages |
| Scripting | ✅ Fusabi (F#) | ✅ Lua | Scarab uses F# |
| Color Schemes | ✅ Yes | ✅ Yes | Compatible |
| Scrollback | ✅ Yes | ✅ Yes | Full support |
| Mouse Support | ✅ Yes | ✅ Yes | Complete |
| True Color | ✅ Yes | ✅ Yes | 24-bit color |

### 🔄 In Progress

| Feature | Scarab Status | Target |
|---------|---------------|--------|
| Tabs | 🔄 Phase 6 | Q1 2025 |
| Splits | 🔄 Phase 6 | Q1 2025 |
| Ligatures | 🔄 Phase 8 | Q2-Q3 2025 |

### 📋 Planned

| Feature | Scarab Status | Target |
|---------|---------------|--------|
| Image Protocols | 📋 Phase 8 | Q2-Q3 2025 |
| Multiplexing | 📋 Phase 9 | Q3-Q4 2025 |
| SSH Integration | 📋 Phase 9 | Q3-Q4 2025 |
| macOS Support | 📋 Phase 7 | Q2 2025 |
| Windows Support | 📋 Phase 7 | Q2 2025 |

### ❌ Different Approach

| Feature | WezTerm | Scarab Alternative |
|---------|---------|-------------------|
| Lua Scripting | ✅ | Fusabi (F#) scripting |
| Built-in Multiplexing | ✅ | Split-process + sessions |
| Domains | ✅ | Future: Remote sessions |
| Font Fallback | ✅ | In development |

## Scarab Unique Features

Features that Scarab has but WezTerm doesn't:

- **Split-Process Architecture** - Daemon + Client for resilience
- **F# Plugin System** - Type-safe Fusabi plugins
- **Zero-Copy IPC** - Lock-free shared memory
- **ECS-Native Navigation** - Bevy ECS for UI
- **Command Palette** - Fuzzy-searchable commands
- **Link Hints** - Vimium-style keyboard navigation
- **Session Persistence** - SQLite-backed sessions

## WezTerm Unique Features

Features that WezTerm has but Scarab doesn't (yet):

- **SSH Domains** - Direct SSH integration (Scarab: planned Phase 9)
- **Multiplexing** - Built-in (Scarab: planned Phase 6)
- **Font Shaping** - Advanced text shaping (Scarab: in development)
- **Ligatures** - Programming ligatures (Scarab: planned Phase 8)
- **Image Protocols** - Sixel, iTerm2, Kitty (Scarab: planned Phase 8)
- **Platform Support** - macOS, Windows (Scarab: planned Phase 7)

## Migration Path

For users migrating from WezTerm:

1. **Core Features** - Available now (GPU, colors, scrollback)
2. **Configuration** - Convert Lua → TOML/F#
3. **Tabs/Splits** - Wait for Phase 6 (Q1 2025)
4. **Advanced Features** - Roadmap aligned for 2025

See [Migration Guides](../user-guide/migration.md) for assistance.

## Contributing to Parity

Want to help achieve parity?

1. Check [GitHub Issues](https://github.com/raibid-labs/scarab/issues)
2. Review [WezTerm Parity Tracking](../../../wezterm-parity/README.md)
3. Implement missing features
4. Submit pull requests

## See Also

- [Roadmap Overview](./overview.md) - Development roadmap
- [Phase Status](./phases.md) - Current phase details
- [Known Issues](./known-issues.md) - Current limitations
