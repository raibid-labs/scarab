# Deprecated Documentation

This folder contains outdated documentation files that have been **superseded by ECS-native implementations** and current documentation. These files are kept for historical reference only and should not be used for current development or usage guidance.

## Why These Are Deprecated

Scarab has migrated to a fully **ECS-native architecture** using Bevy. Legacy documentation referenced:
- Event-driven patterns (replaced by Bevy ECS systems and events)
- Non-Bevy plugin APIs (replaced by ECS-native plugin host)
- Pre-integration navigation systems (replaced by ECS components/resources)

## Finding Current Documentation

For current, maintained documentation:

| Topic | Current Location |
|-------|------------------|
| **Documentation Index** | [`docs/README.md`](../README.md) |
| **Navigation System** | [`docs/navigation.md`](../navigation.md) (ECS-native) |
| **Navigation User Guide** | [`docs/navigation/user-guide.md`](../navigation/user-guide.md) |
| **Navigation Developer Guide** | [`docs/navigation/developer-guide.md`](../navigation/developer-guide.md) |
| **Plugin Development** | [`docs/plugin-development/README.md`](../plugin-development/README.md) |
| **Plugin API Reference** | [`docs/plugin-development/api-reference/`](../plugin-development/api-reference/) |
| **Architecture** | [`docs/developer/architecture.md`](../developer/architecture.md) |
| **Rustdoc API** | Run `cargo doc --workspace --open` |
| **External Docs Site** | Built from `~/raibid-labs/docs` |

## Contents

### Superseded Plugin Development Guides

These have been replaced by the ECS-native plugin development guide:

| Deprecated File | Replacement |
|----------------|-------------|
| `plugin-api.md` | [`plugin-development/api-reference/`](../plugin-development/api-reference/) |
| `plugin-development-guide.md` | [`plugin-development/README.md`](../plugin-development/README.md) |
| `PLUGIN_DEVELOPMENT.md` | [`plugin-development/README.md`](../plugin-development/README.md) |
| `guides-plugin-development.md` | [`plugin-development/README.md`](../plugin-development/README.md) |

### Completion Reports (Historical)

Point-in-time completion reports from development phases (accurate when written, now historical):

- `AUDIT_REPORT_PASS_2.md` - Phase 2 audit completion
- `AUDIT_REPORT_PASS_3.md` - Phase 3 audit completion
- `PHASE4_COMPLETION_REPORT.md` - Phase 4 completion
- `phase4-final-report.md` - Phase 4 final summary
- `phase4-summary.md` - Phase 4 overview
- `reference-COMPLETION_REPORT.md` - Reference implementation completion
- `memory-phase1-vte-completion-report.md` - Phase 1 VTE completion
- `TUTORIAL_IMPLEMENTATION_SUMMARY.md` - Tutorial implementation summary
- `REGISTRY_IMPLEMENTATION_SUMMARY.md` - Registry implementation summary
- `TEST_PLAN_MIMIC.md` - Test plan documentation

### Point-in-Time Implementation Summaries

These were accurate when written but may be outdated:

- `implementation-summary-fusabi-vm.md` - Fusabi VM integration summary
- `integration-status.md` - Integration status snapshot
- `ui-implementation-status.md` - UI implementation snapshot
- `task-c9-plugin-port-completion.md` - Plugin port completion report

### Superseded Build Instructions

- `mdbook-instructions.md` - Legacy mdBook build instructions (docs now built externally)

## Questions?

If you're looking for specific information and can't find it in the current documentation:

1. Check the [Documentation Index](../README.md)
2. Run `cargo doc --workspace --open` for API reference
3. Check recent [Audits](../audits/) for architectural updates
4. Open an issue if documentation is missing

---

**Last Updated:** 2025-12-04

*These files are preserved for historical reference. For current documentation, always refer to the main [documentation index](../README.md).*
