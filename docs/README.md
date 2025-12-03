# Scarab Documentation

Welcome to the Scarab terminal emulator documentation. This is the central index for all project documentation.

## Quick Navigation

| Category | Description | Key Documents |
|----------|-------------|---------------|
| **Getting Started** | Installation and first steps | [Installation](./user/installation.md), [Quick Start](./user/quickstart.md) |
| **User Guide** | Using Scarab features | [Configuration](./configuration.md), [Navigation](./navigation/user-guide.md) |
| **Developer Guide** | Architecture and development | [Architecture](./developer/architecture.md), [Plugin Development](./plugin-development/README.md) |
| **Reference** | Complete technical reference | [Config Schema](./reference/configuration.md), [Keybindings](./reference/keybindings.md) |
| **Release** | Release management | [Release Process](./RELEASE_PROCESS.md), [Versioning](./VERSIONING.md) |

---

## Getting Started

New to Scarab? Start here.

### Installation

- **[Installation Guide](./user/installation.md)** - Platform-specific installation instructions
  - macOS (Homebrew, manual)
  - Linux (AUR, apt, rpm, AppImage)
  - Windows (WinGet, Chocolatey, manual)
  - Build from source

### First Steps

- **[Quick Start Guide](./user/quickstart.md)** - Get up and running in 5 minutes
  - Basic usage and keyboard shortcuts
  - Essential configuration
  - Common tasks
  - Tips and tricks

### Migration

Coming from another terminal emulator?

- [From Alacritty](./migration/from-alacritty.md)
- [From iTerm2](./migration/from-iterm2.md)
- [From GNOME Terminal](./migration/from-gnome-terminal.md)

---

## User Guide

Complete guides for using Scarab's features.

### Configuration

- **[Configuration Guide](./configuration.md)** - Complete configuration reference
  - Terminal settings
  - Appearance and themes
  - Keybindings
  - Session management
  - Plugin configuration

- **[Customization Guide](./CUSTOMIZATION.md)** - Themes, colors, and appearance

### Navigation System

Scarab features a powerful keyboard-driven navigation system (Vimium-style).

**User Documentation:**
- **[Navigation User Guide](./navigation/user-guide.md)** - Keyboard navigation for users
  - Hint mode basics
  - Keybindings and shortcuts
  - Tips and troubleshooting

**Architecture Documentation:**
- **[Navigation Architecture Overview](./navigation.md)** - Complete technical documentation
  - ECS-native design
  - Navigation modes
  - Focusable types
  - Keymaps and configuration
  - Per-pane behavior

### Features

- **[Keybindings Reference](./reference/keybindings.md)** - All keyboard shortcuts
- **[Troubleshooting](./reference/troubleshooting.md)** - Common issues and solutions
- **[FAQ](./reference/faq.md)** - Frequently asked questions
- **[Telemetry Guide](./TELEMETRY.md)** - Usage analytics and metrics
- **[Telemetry Quick Reference](./TELEMETRY_QUICK_REFERENCE.md)** - Common telemetry patterns

### Tutorials

Step-by-step guides for common workflows:

- [Getting Started Tutorial](./tutorials/01-getting-started.md) - First steps with Scarab
- [Customization Tutorial](./tutorials/02-customization.md) - Personalize your terminal
- [Workflow Tutorial](./tutorials/03-workflows.md) - Advanced productivity tips

---

## Developer Guide

Documentation for Scarab developers and plugin authors.

### Architecture

- **[Architecture Overview](./developer/architecture.md)** - System design and components
  - Multi-process architecture
  - Core components (client, daemon, protocol)
  - Data flow
  - Design decisions
  - Performance considerations

- **[IPC Protocol](./architecture/ipc-protocol.md)** - Inter-process communication
- **[IPC Implementation](./architecture/IPC_IMPLEMENTATION.md)** - Shared memory details
- **[Session Management](./session-management.md)** - Session persistence and lifecycle
- **[Tab/Pane System](./TAB_PANE_DESIGN.md)** - Multiplexing architecture
- **[Safe State Abstraction](./safe-state-abstraction.md)** - Thread-safe state management

### Navigation System Development

**Developer Documentation:**
- **[Navigation Developer Guide](./navigation/developer-guide.md)** - Navigation system internals
  - ECS components and resources
  - Events and system sets
  - Per-pane behavior
  - Plugin bridge usage
  - Ratatui overlay integration
  - Testing and best practices

**Related Documents:**
- [Event Migration Guide](./event-migration-guide.md) - EventRegistry deprecation
- [Context Menu Implementation](./context-menu-implementation.md) - Right-click menus

### Plugin Development

Comprehensive guides for building Scarab plugins:

- **[Plugin Development Guide](./plugin-development/README.md)** - Main development guide
  - Frontend (.fsx) and Backend (.fzb) plugins
  - Getting started tutorials
  - Architecture patterns
  - API reference

**Tutorials:**
- [Hello World Frontend](./plugin-development/tutorials/01-hello-world-frontend.md)
- [Hello World Backend](./plugin-development/tutorials/02-hello-world-backend.md)
- [Plugin API Deep Dive](./plugin-development/tutorials/03-plugin-api-deep-dive.md)
- [Real Plugin: URL Shortener](./plugin-development/tutorials/04-real-plugin-url-shortener.md)
- [Frontend UI with RemoteUI](./plugin-development/tutorials/05-frontend-ui-remoteui.md)
- [Backend Hooks](./plugin-development/tutorials/06-backend-hooks.md)
- [Testing and Publishing](./plugin-development/tutorials/07-testing-and-publishing.md)

**API Reference:**
- [Plugin Context](./plugin-development/api-reference/plugin-context.md)
- [Hooks](./plugin-development/api-reference/hooks.md)
- [RemoteUI Components](./plugin-development/api-reference/remote-ui.md)
- [Utilities](./plugin-development/api-reference/utilities.md)

**Architecture:**
- [Frontend vs Backend](./plugin-development/architecture/frontend-vs-backend.md)
- [Plugin Lifecycle](./plugin-development/architecture/plugin-lifecycle.md)
- [Performance](./plugin-development/architecture/performance.md)
- [Security](./plugin-development/architecture/security.md)

**Additional Resources:**
- [Plugin Registry](./plugin-registry.md) - Plugin distribution system
- [Plugin Registry Quickstart](./plugin-registry-quickstart.md)
- [Plugin GPG Verification](./plugin-gpg-verification.md)
- [Plugin Logging and Notifications](./PLUGIN_LOGGING_AND_NOTIFICATIONS.md)

### Fusabi Scripting Language

Scarab uses Fusabi (F# dialect for Rust) for plugin development:

- **[Fusabi Language Guide](./FUSABI_LANGUAGE.md)** - F# dialect syntax and features
- **[Fusabi Development Guide](./FUSABI_GUIDE.md)** - Writing Fusabi code
- **[Fusabi VM Specification](./fusabi-vm-spec.md)** - Bytecode VM details

### Implementation Guides

- [Text Rendering](./text-rendering-implementation.md) - cosmic-text integration
- [Context Menu](./context-menu-implementation.md) - Right-click context menus
- [Config System](./config-system-implementation.md) - Configuration management
- [Session Implementation](./session-implementation-summary.md) - Session state tracking

---

## Reference

Complete technical references.

### Configuration

- **[Configuration Reference](./reference/configuration.md)** - Complete configuration options
- **[Keybindings Reference](./reference/keybindings.md)** - All keyboard shortcuts
- **[Performance Guide](./reference/performance.md)** - Performance tuning
- **[Troubleshooting](./reference/troubleshooting.md)** - Common issues
- **[FAQ](./reference/faq.md)** - Frequently asked questions

### Platform Support

- **[Platform Support Matrix](./PLATFORM_SUPPORT.md)** - OS compatibility
- **[Platform Support Status](./issues/12-platform-support-status.md)** - Current implementation status

### Advanced Features

- **[Sixel Graphics](./SIXEL_QUICK_REFERENCE.md)** - Sixel image protocol support
- **[SSH Domains](./ssh-domains.md)** - Remote terminal connections
- **[Deep Shell Integration](./implementation/DEEP_SHELL_INTEGRATION.md)** - OSC 133 and shell features

---

## Testing

Comprehensive testing guides and infrastructure.

- **[TESTING.md](../TESTING.md)** - **Comprehensive testing guide (START HERE)**
  - Unit tests
  - Integration tests
  - E2E tests
  - Performance tests

- **[Testing Guide](./testing-guide.md)** - How to run tests
- **[Headless Testing Quickstart](./testing/HEADLESS_TESTING_QUICKSTART.md)** - Automated testing
- **[Headless POC Results](./testing/HEADLESS_POC_RESULTS.md)** - Test infrastructure validation
- **[Benchmark Guide](./BENCHMARK_GUIDE.md)** - Performance benchmarking

---

## Performance

Performance optimization guides and benchmarks.

- **[Performance Guide](./performance/PERFORMANCE_GUIDE.md)** - Optimization strategies
- **[Plugin Performance Report](./PLUGIN_PERFORMANCE_REPORT.md)** - Plugin optimization
- **[Benchmark Guide](./BENCHMARK_GUIDE.md)** - Running benchmarks

---

## Tooling and Infrastructure

Development tools and infrastructure documentation.

- **[Homebrew Setup](./HOMEBREW_SETUP.md)** - macOS package distribution
- **[Tooling Quickstart](./TOOLING_QUICKSTART.md)** - Development tools
- **[Version Management](./VERSION_MANAGEMENT_QUICKSTART.md)** - Versioning workflow
- **[Contributing to Documentation](./CONTRIBUTING-DOCS.md)** - Documentation standards

---

## Release Management

Guides for creating and managing releases.

### Process

- **[Release Process](./RELEASE_PROCESS.md)** - How to create releases
- **[Release Checklist](./RELEASE_CHECKLIST.md)** - Pre-release validation
- **[Release README](./RELEASE_README.md)** - Release documentation
- **[Versioning](./VERSIONING.md)** - Semantic versioning guidelines

### Templates

- [Release Announcement Template](./templates/release-announcement-template.md)
- [Release Notes Template](./templates/release-notes-template.md)
- [Hotfix Announcement Template](./templates/hotfix-announcement-template.md)

### Current Releases

- [v0.2.0-alpha.0 Checklist](./releases/v0.2.0-alpha.0-checklist.md)

---

## Roadmap

Project roadmap and feature planning.

- **[Roadmap](./ROADMAP.md)** - Feature roadmap overview
- **[AI-Readable Roadmap](./ROADMAP-AI.md)** - Comprehensive Bevy/ECS implementation roadmap
- **[WezTerm Parity Analysis](./analysis/wezterm-gap-analysis.md)** - Feature comparison with WezTerm
- **[WezTerm Parity Tracking](./wezterm-parity/README.md)** - Detailed parity implementation guides
  - [Object Model](./wezterm-parity/01-object-model.md)
  - [Event System](./wezterm-parity/02-event-system.md)
  - [Status Bar API](./wezterm-parity/03-status-bar-api.md)
  - [Key Tables](./wezterm-parity/04-key-tables.md)
  - [Image Protocols](./wezterm-parity/05-image-protocols.md)
  - [Copy Mode](./wezterm-parity/06-copy-mode.md)
  - [Workstreams](./wezterm-parity/07-workstreams.md)
  - [Release Roadmap](./wezterm-parity/08-release-roadmap.md)

---

## Audits and Reviews

Technical audits and architectural reviews documenting Scarab's evolution.

### Recent Audits

- **[Claude Audit 2025-12-01](./audits/claude-2025-12-01/README.md)** - Comprehensive architecture audit
  - [Executive Summary](./audits/claude-2025-12-01/00-EXECUTIVE-SUMMARY.md)
  - [Rendering Architecture](./audits/claude-2025-12-01/01-RENDERING-ARCHITECTURE.md)
  - [Week 1 Complete](./audits/claude-2025-12-01/11-WEEK-1-COMPLETE.md)
  - [Week 2 Complete](./audits/claude-2025-12-01/12-WEEK-2-COMPLETE.md)
  - [Week 3 Progress](./audits/claude-2025-12-01/13-WEEK-3-PROGRESS.md)

- **[Gemini Roadmap Audit](./audits/gemini-roadmap-2025-12-02/README.md)** - Multiplexing and VTE gaps
  - [Multiplexing Gap](./audits/gemini-roadmap-2025-12-02/01-multiplexing-gap.md)
  - [VTE Refactor Guide](./audits/gemini-roadmap-2025-12-02/02-vte-refactor-guide.md)

- **[Gemini WezTerm Parity](./audits/gemini-wezterm-parity-2025-12-02/README.md)** - Feature parity analysis
  - [Parity Gap Analysis](./audits/gemini-wezterm-parity-2025-12-02/01-parity-gap-analysis.md)
  - [Claude Work Verification](./audits/gemini-wezterm-parity-2025-12-02/02-claude-work-verification.md)
  - [Implementation Plan](./audits/gemini-wezterm-parity-2025-12-02/03-implementation-plan.md)

### Navigation/ECS Audits

- [nav-ecs-001](./audits/codex-2025-12-02-nav-ecs-001/summary.md) - Initial ECS navigation review
- [nav-ecs-002](./audits/codex-2025-12-02-nav-ecs-002/summary.md)
- [nav-ecs-003](./audits/codex-2025-12-02-nav-ecs-003/summary.md)
- [nav-ecs-004](./audits/codex-2025-12-02-nav-ecs-004/summary.md)
- [nav-focusable-001](./audits/codex-2025-12-03-nav-focusable-001/AUDIT_REPORT.md) - Focusable system review

### Documentation & Testing Audits

- [docs-testlib-006](./audits/codex-2025-12-02-docs-testlib-006/summary.md)
- [docs-testlib-007](./audits/codex-2025-12-02-docs-testlib-007/summary.md)

---

## Research

Research documents exploring technical approaches and opportunities.

- **[Bevy Terminal Opportunities](./research/codex-2025-12-02-bevy-terminal-research/unique-opportunities.md)** - Unique advantages of Bevy for terminals
- **[Orchestration Planning](./research/codex-2025-12-02-orchestration-plan.md)** - Multi-agent development workflow
- **[Scarab/Bevy/ECS Audit](./research/codex-2025-12-02-scarab-ecs-audit/scarab-ecs-and-ratatui.md)** - ECS integration analysis
- **[WezTerm Lua Capabilities](./research/wezterm-lua-capabilities.md)** - Lua scripting comparison

---

## Video Resources

- **[Video Demos](./videos/README.md)** - Video demonstrations and tutorials
- **[Demo Assets](./assets/demos/README.md)** - How to record demo videos

---

## Document Status and History

### Current (Canonical) Documents

These documents are actively maintained and represent the current state of Scarab:

#### Navigation System
- `navigation/user-guide.md` - User-facing navigation guide
- `navigation/developer-guide.md` - Developer reference for navigation internals
- `navigation.md` - Complete ECS-native navigation architecture overview (updated 2025-12-03)

#### Plugin Development
- `plugin-development/README.md` - Main plugin development guide
- `plugin-development/tutorials/` - Step-by-step tutorials
- `plugin-development/api-reference/` - API documentation
- `plugin-development/architecture/` - Architectural guides

#### Architecture & Implementation
- `developer/architecture.md` - System architecture
- `architecture/ipc-protocol.md` - IPC design
- All files in `implementation-status/` - Current implementation tracking

#### User Documentation
- All files in `user/`, `tutorials/`, `reference/`, `migration/`

### Legacy and Historical Documents

The following documents contain outdated information or have been superseded. They are kept for historical reference.

#### Superseded by plugin-development/

These documents have been replaced by the comprehensive plugin development guide:

- `plugin-api.md` → See `plugin-development/api-reference/` instead
- `plugin-development-guide.md` → See `plugin-development/README.md` instead
- `PLUGIN_DEVELOPMENT.md` → See `plugin-development/README.md` instead
- `guides/plugin-development.md` → See `plugin-development/README.md` instead

#### Completion Reports (Historical)

Point-in-time completion reports from development phases:

- `AUDIT_REPORT_PASS_2.md`
- `AUDIT_REPORT_PASS_3.md`
- `PHASE4_COMPLETION_REPORT.md`
- `phase4-final-report.md`
- `phase4-summary.md`
- `reference/COMPLETION_REPORT.md`
- `memory/phase1-vte-completion-report.md`
- `TUTORIAL_IMPLEMENTATION_SUMMARY.md`
- `REGISTRY_IMPLEMENTATION_SUMMARY.md`
- `TEST_PLAN_MIMIC.md`

#### Implementation Summaries (Point-in-Time)

These were accurate when written but may be outdated:

- `implementation-summary-fusabi-vm.md`
- `integration-status.md`
- `ui-implementation-status.md`
- `implementation-status/task-c9-plugin-port-completion.md`

#### Analysis Documents (Historical)

Legacy analysis from earlier development phases:

- `analysis/gemini-deep-audit-2025-11-25.md`
- `analysis/gemini-audit-2025-12-01/`
- `analysis/todo-audit-report.md`
- `reviews/gemini-2025-11-24.md`

### Deprecation Notices

Documents with deprecation notices pointing to current documentation:

- `deprecations/eventregistry-2025-12-02.md` - EventRegistry deprecation

---

## Contributing to Documentation

When adding or updating documentation:

1. **Placement**: Use appropriate subdirectories
   - User-facing docs → `user/`, `tutorials/`, `reference/`
   - Developer docs → `developer/`, `architecture/`, `plugin-development/`
   - Implementation notes → `implementation/`, `implementation-status/`
   - Historical records → `audits/`, `analysis/`, `research/`

2. **Update this index**: Add links to new documents in the appropriate section

3. **Mark superseded documents**: Add deprecation notices to old documents when they're replaced

4. **Use clear titles**: Descriptive filenames and headers

5. **Maintain consistency**: Follow existing formatting patterns

6. **Include dates**: Add last-updated dates to long-form documents

7. **Link properly**: Ensure documents are reachable within 2 clicks from this index

For detailed documentation standards, see [CONTRIBUTING-DOCS.md](./CONTRIBUTING-DOCS.md).

---

## Quick Reference by Role

### I'm a User
1. Start: [Installation](./user/installation.md) → [Quick Start](./user/quickstart.md)
2. Configure: [Configuration Guide](./configuration.md)
3. Navigate: [Navigation User Guide](./navigation/user-guide.md)
4. Troubleshoot: [Troubleshooting](./reference/troubleshooting.md)

### I'm a Plugin Developer
1. Start: [Plugin Development Guide](./plugin-development/README.md)
2. Tutorial: [Hello World Frontend](./plugin-development/tutorials/01-hello-world-frontend.md)
3. API: [Plugin API Reference](./plugin-development/api-reference/)
4. Publish: [Testing and Publishing](./plugin-development/tutorials/07-testing-and-publishing.md)

### I'm a Scarab Contributor
1. Architecture: [Architecture Overview](./developer/architecture.md)
2. Navigation: [Navigation Developer Guide](./navigation/developer-guide.md)
3. Testing: [TESTING.md](../TESTING.md)
4. Contributing: [CONTRIBUTING-DOCS.md](./CONTRIBUTING-DOCS.md)

---

**Documentation index last updated: 2025-12-03**

*This is the canonical documentation index for Scarab. All documentation should be reachable from this page within 2 clicks.*
