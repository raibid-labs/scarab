# Project Roadmap

Scarab's development roadmap and feature timeline.

## Quick Links

For complete roadmap documentation, see:
- [ROADMAP.md](../../../../ROADMAP.md) - Strategic roadmap
- [ROADMAP-AI.md](../../../ROADMAP-AI.md) - Detailed AI-readable roadmap
- [WezTerm Parity Analysis](../../../wezterm-parity/README.md) - Feature parity tracking

## Current Status

**Phase**: Phase 5 - Integration & Polish (~80% complete)

**Version**: v0.1.0-alpha

## Completed Phases (1-4)

### Phase 1: Core Terminal Emulation
- VTE parser integration
- Basic text rendering
- PTY process management
- Shared memory IPC

### Phase 2: Plugin System
- Fusabi language integration
- Dual runtime (VM + interpreter)
- Plugin API and lifecycle
- Example plugins

### Phase 3: Session Management
- SQLite-backed persistence
- Session save/restore
- Client attachment/detachment

### Phase 4: Configuration & UI
- TOML configuration with hot-reload
- Command palette
- Link hints navigation
- Theme system

## Current Phase 5: Integration & Polish

**In Progress:**
- Interactive tutorial
- E2E testing infrastructure
- UI polish and animations
- Documentation completion
- Alpha release preparation

**Target**: Alpha release v0.1.0

## Upcoming Phases

### Phase 6: Multiplexing (Q1 2025)
- Tabs and split panes
- Window management
- Layout persistence
- Pane navigation

### Phase 7: Platform Support (Q2 2025)
- macOS support
- Windows support (WSL2 first)
- Cross-platform testing

### Phase 8: Advanced Rendering (Q2-Q3 2025)
- Ligature support
- Image protocols (Sixel, Kitty)
- GPU shader effects
- Custom rendering plugins

### Phase 9: Remote & Multiplexing (Q3-Q4 2025)
- Remote session support
- SSH integration
- Multiplexing protocol
- Cloud terminal support

### Phase 10: Beta & Ecosystem (Q4 2025)
- Beta release v0.2.0
- Plugin marketplace
- Documentation portal
- Community growth

## Feature Timeline

| Feature | Status | Target |
|---------|--------|--------|
| Core terminal | ✅ Complete | Phase 1 |
| Plugin system | ✅ Complete | Phase 2 |
| Sessions | ✅ Complete | Phase 3 |
| Configuration | ✅ Complete | Phase 4 |
| Tutorial | 🔄 In Progress | Phase 5 |
| Testing | 🔄 In Progress | Phase 5 |
| Tabs/Splits | 📋 Planned | Phase 6 |
| macOS | 📋 Planned | Phase 7 |
| Windows | 📋 Planned | Phase 7 |
| Ligatures | 📋 Planned | Phase 8 |
| Images | 📋 Planned | Phase 8 |
| Remote | 📋 Planned | Phase 9 |

## Contributing to Roadmap

Have ideas or want to contribute? Check:
- [GitHub Issues](https://github.com/raibid-labs/scarab/issues)
- [GitHub Discussions](https://github.com/raibid-labs/scarab/discussions)
- [Contributing Guide](../developer-guide/contributing.md)

## See Also

- [Phase Status](./phases.md) - Detailed phase breakdown
- [WezTerm Parity](./wezterm-parity.md) - Feature parity tracking
- [Known Issues](./known-issues.md) - Current limitations
