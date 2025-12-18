# Scarab Documentation

Welcome to Scarab - a high-performance, split-process terminal emulator built with Rust and Bevy.

## Documentation Structure

```
docs/
├── guides/           # User-facing guides
├── development/      # Developer and contributor guides
├── architecture/     # Technical architecture docs
├── plugins/          # Plugin development with Fusabi
├── reference/        # Technical reference docs
├── releases/         # Release notes and changelogs
├── adr/              # Architecture Decision Records
└── internal/         # Internal dev notes (audits, research)
```

## Quick Start

| I want to... | Go to |
|--------------|-------|
| Learn how to use Scarab | [guides/](./guides/) |
| Configure Scarab | [guides/configuration.md](./guides/configuration.md) |
| Build a plugin | [plugins/](./plugins/) |
| Understand the architecture | [architecture/](./architecture/) |
| Contribute to Scarab | [development/](./development/) |
| Check platform support | [reference/PLATFORM_SUPPORT.md](./reference/PLATFORM_SUPPORT.md) |

## User Guides

- **[Configuration](./guides/configuration.md)** - Terminal settings and options
- **[Customization](./guides/CUSTOMIZATION.md)** - Themes, colors, appearance
- **[Navigation](./guides/navigation.md)** - Keyboard navigation (Vimium-style hints)
- **[Session Management](./guides/session-management.md)** - Tabs, panes, sessions
- **[SSH Domains](./guides/ssh-domains.md)** - Remote connections
- **[Homebrew Setup](./guides/HOMEBREW_SETUP.md)** - macOS installation

## Developer Guides

- **[Testing Summary](./development/TESTING_SUMMARY.md)** - How to run tests
- **[Benchmark Guide](./development/BENCHMARK_GUIDE.md)** - Performance testing
- **[Migration Guide](./development/MIGRATION_GUIDE.md)** - Breaking changes
- **[Tooling Quickstart](./development/TOOLING_QUICKSTART.md)** - Dev tools setup

## Architecture

- **[Architecture Report](./architecture/architecture_report.md)** - System overview
- **[Tab/Pane Design](./architecture/TAB_PANE_DESIGN.md)** - Multiplexing design
- **[Structure](./architecture/STRUCTURE.md)** - Codebase organization

## Plugin Development

Scarab uses [Fusabi](https://github.com/fusabi-lang/fusabi) (an F# dialect for Rust) for plugins.

- **[Fusabi Guide](./plugins/FUSABI_GUIDE.md)** - Getting started with Fusabi
- **[Fusabi Language](./plugins/FUSABI_LANGUAGE.md)** - Language reference
- **[Plugin Manifest](./plugins/PLUGIN_MANIFEST.md)** - Plugin configuration
- **[Plugin Dock](./plugins/PLUGIN_DOCK_IMPLEMENTATION.md)** - Status bar plugins

## Reference

- **[Platform Support](./reference/PLATFORM_SUPPORT.md)** - OS compatibility
- **[Sixel Reference](./reference/SIXEL_QUICK_REFERENCE.md)** - Image protocol
- **[Versioning](./reference/VERSIONING.md)** - Semantic versioning policy

## Architecture Decision Records

- **[001-historical-decisions.md](./adr/001-historical-decisions.md)** - Key architectural decisions

## Releases

See [releases/](./releases/) for version-specific release notes and checklists.

## Implementation Plans

- **[UI Implementation Plan](./UI_IMPLEMENTATION_PLAN.md)** - Upcoming UI features (omnibar, breadcrumbs, Bevy showcase)
- **[Roadmap](./ROADMAP.md)** - Project roadmap

## API Documentation

Generate Rust API docs:

```bash
cargo doc --workspace --open
```

## Contributing

See [CONTRIBUTING.md](../CONTRIBUTING.md) in the repository root.

---

*Last updated: 2025-12-18*
