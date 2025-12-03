# Scarab Documentation

Welcome to the Scarab terminal emulator documentation.

## Getting Started

- [README](../README.md) - Project overview and quick start
- [Installation Guide](./user/installation.md) - How to install Scarab
- [Quick Start Guide](./user/quickstart.md) - Get up and running quickly
- [CLAUDE.md](../CLAUDE.md) - AI assistant context and project overview

## User Guides

- [Customization Guide](./CUSTOMIZATION.md) - Themes, colors, and appearance
- [Keybindings Reference](./reference/keybindings.md) - All keyboard shortcuts
- [Configuration Reference](./reference/configuration.md) - Complete configuration options
- [Troubleshooting](./reference/troubleshooting.md) - Common issues and solutions
- [FAQ](./reference/faq.md) - Frequently asked questions

### Tutorials

- [Getting Started Tutorial](./tutorials/01-getting-started.md) - First steps with Scarab
- [Customization Tutorial](./tutorials/02-customization.md) - Personalize your terminal
- [Workflow Tutorial](./tutorials/03-workflows.md) - Advanced productivity tips

### Migration Guides

- [From Alacritty](./migration/from-alacritty.md)
- [From iTerm2](./migration/from-iterm2.md)
- [From GNOME Terminal](./migration/from-gnome-terminal.md)

## Developer Guides

### Navigation System

- [Navigation User Guide](./navigation/user-guide.md) - Keyboard navigation for users
- [Navigation Developer Guide](./navigation/developer-guide.md) - Navigation system internals
- [Navigation Architecture Overview](./navigation.md) - Complete ECS-native navigation architecture
  - Vimium-style keyboard navigation
  - Hint mode for URLs, paths, emails
  - Prompt marker integration (OSC 133)
  - Per-pane navigation state
  - Plugin integration API

### Plugin Development

- [Plugin Development Guide](./plugin-development/README.md) - Comprehensive plugin development guide
  - Frontend (.fsx) and Backend (.fzb) plugins
  - Tutorials, architecture guides, API reference
  - Example plugins and best practices
- [Plugin API Reference](./plugin-development/api-reference/) - Detailed API documentation
  - [Plugin Context](./plugin-development/api-reference/plugin-context.md)
  - [Hooks](./plugin-development/api-reference/hooks.md)
  - [RemoteUI Components](./plugin-development/api-reference/remote-ui.md)
  - [Utilities](./plugin-development/api-reference/utilities.md)

### Architecture

- [Architecture Overview](./developer/architecture.md) - System design and components
- [IPC Protocol](./architecture/ipc-protocol.md) - Inter-process communication
- [IPC Implementation](./architecture/IPC_IMPLEMENTATION.md) - Shared memory details
- [Session Management](./session-management.md) - Session persistence and lifecycle
- [Tab/Pane System](./TAB_PANE_DESIGN.md) - Multiplexing architecture
- [Safe State Abstraction](./safe-state-abstraction.md) - Thread-safe state management

### Fusabi Scripting Language

- [Fusabi Language Guide](./FUSABI_LANGUAGE.md) - F# dialect for Rust
- [Fusabi Development Guide](./FUSABI_GUIDE.md) - Writing Fusabi code
- [Fusabi VM Specification](./fusabi-vm-spec.md) - Bytecode VM details

### Implementation Summaries

- [Text Rendering](./text-rendering-implementation.md) - cosmic-text integration
- [Context Menu](./context-menu-implementation.md) - Right-click context menus
- [Config System](./config-system-implementation.md) - Configuration management
- [Session Management](./session-implementation-summary.md) - Session state tracking

## Testing

- [Testing Guide](./testing-guide.md) - How to run tests
- [Headless Testing Quickstart](./testing/HEADLESS_TESTING_QUICKSTART.md) - Automated testing
- [Headless POC Results](./testing/HEADLESS_POC_RESULTS.md) - Test infrastructure validation
- [Benchmark Guide](./BENCHMARK_GUIDE.md) - Performance benchmarking

## Performance

- [Performance Guide](./performance/PERFORMANCE_GUIDE.md) - Optimization strategies
- [Plugin Performance Report](./PLUGIN_PERFORMANCE_REPORT.md) - Plugin optimization

## Tooling and Infrastructure

- [Homebrew Setup](./HOMEBREW_SETUP.md) - macOS package distribution
- [Tooling Quickstart](./TOOLING_QUICKSTART.md) - Development tools
- [Version Management](./VERSION_MANAGEMENT_QUICKSTART.md) - Versioning workflow
- [Telemetry Guide](./TELEMETRY.md) - Usage analytics and metrics
- [Telemetry Quick Reference](./TELEMETRY_QUICK_REFERENCE.md) - Common telemetry patterns

## Release Management

- [Release Process](./RELEASE_PROCESS.md) - How to create releases
- [Release Checklist](./RELEASE_CHECKLIST.md) - Pre-release validation
- [Release README](./RELEASE_README.md) - Release documentation
- [Versioning](./VERSIONING.md) - Semantic versioning guidelines

### Release Templates

- [Release Announcement Template](./templates/release-announcement-template.md)
- [Release Notes Template](./templates/release-notes-template.md)
- [Hotfix Announcement Template](./templates/hotfix-announcement-template.md)

## Roadmap

- [Roadmap](./ROADMAP.md) - Feature roadmap overview
- [AI-Readable Roadmap](./ROADMAP-AI.md) - Comprehensive Bevy/ECS implementation roadmap
- [WezTerm Parity Analysis](./analysis/wezterm-gap-analysis.md) - Feature comparison with WezTerm
- [WezTerm Parity Tracking](./wezterm-parity/README.md) - Detailed parity implementation guides

## Audits

Technical audits and architectural reviews:

- [Claude Audit 2025-12-01](./audits/claude-2025-12-01/README.md) - Comprehensive architecture audit
- [Gemini Roadmap Audit](./audits/gemini-roadmap-2025-12-02/README.md) - Multiplexing and VTE gaps
- [Gemini WezTerm Parity](./audits/gemini-wezterm-parity-2025-12-02/README.md) - Feature parity analysis
- [Codex Navigation/ECS Audits](./audits/) - Navigation system implementation reviews
  - [nav-ecs-001](./audits/codex-2025-12-02-nav-ecs-001/summary.md)
  - [nav-ecs-002](./audits/codex-2025-12-02-nav-ecs-002/summary.md)
  - [nav-ecs-003](./audits/codex-2025-12-02-nav-ecs-003/summary.md)
  - [nav-ecs-004](./audits/codex-2025-12-02-nav-ecs-004/summary.md)
  - [nav-focusable-001](./audits/codex-2025-12-03-nav-focusable-001/AUDIT_REPORT.md)

## Research

- [Bevy Terminal Opportunities](./research/codex-2025-12-02-bevy-terminal-research/unique-opportunities.md)
- [Orchestration Planning](./research/codex-2025-12-02-orchestration-plan.md)
- [Scarab/Bevy/ECS Audit](./research/codex-2025-12-02-scarab-ecs-audit/scarab-ecs-and-ratatui.md)
- [WezTerm Lua Capabilities](./research/wezterm-lua-capabilities.md)

## Platform Support

- [Platform Support Matrix](./PLATFORM_SUPPORT.md) - OS compatibility
- [Platform Support Status](./issues/12-platform-support-status.md) - Current implementation status

## Video Resources

- [Video Demos](./videos/README.md) - Video demonstrations and tutorials
- [Demo Assets](./assets/demos/README.md) - How to record demo videos

---

## Document Status

### Current (Canonical)

The following documents are actively maintained and represent the current state of Scarab:

#### Navigation
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

### Legacy/Superseded

The following documents contain outdated information or have been superseded by newer documentation. They are kept for historical reference but should not be used as authoritative sources.

#### Replaced by `plugin-development/`
- `plugin-api.md` - See `plugin-development/api-reference/` instead
- `plugin-development-guide.md` - See `plugin-development/README.md` instead
- `PLUGIN_DEVELOPMENT.md` - See `plugin-development/README.md` instead
- `guides/plugin-development.md` - See `plugin-development/README.md` instead

#### Completion Reports (Historical)
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
These were accurate at time of writing but may be outdated:
- `implementation-summary-fusabi-vm.md`
- `integration-status.md`
- `ui-implementation-status.md`
- `implementation-status/task-c9-plugin-port-completion.md`

#### Analysis Documents (Historical)
Legacy analysis from earlier phases:
- `analysis/gemini-deep-audit-2025-11-25.md`
- `analysis/gemini-audit-2025-12-01/`
- `analysis/todo-audit-report.md`
- `reviews/gemini-2025-11-24.md`

---

## Contributing to Documentation

When adding or updating documentation:

1. Place user-facing docs in appropriate subdirectories (`user/`, `tutorials/`, `reference/`)
2. Place developer docs in `developer/`, `architecture/`, or `plugin-development/`
3. Update this index with links to new documents
4. Mark superseded documents in the "Legacy/Superseded" section
5. Use clear, descriptive titles and maintain consistent formatting
6. Include last-updated dates in long-form documents

---

*Documentation index last updated: 2025-12-03*
