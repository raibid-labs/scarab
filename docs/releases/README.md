# Release Documentation

This directory contains release planning, checklists, and tracking documents for Scarab Terminal releases.

## v0.2.0-alpha.0 Release

The next release transitioning from perpetual alpha (v0.1.0-alpha.X) to structured versioning.

### Quick Access

- **[Quick Summary](./v0.2.0-alpha-SUMMARY.md)** - At-a-glance status and commands (4.7 KB, 167 lines)
- **[Release Plan](./v0.2.0-alpha-plan.md)** - Comprehensive planning document (13 KB, 361 lines)
- **[Detailed Checklist](./v0.2.0-alpha.0-checklist.md)** - Step-by-step verification (6.0 KB, 163 lines)

### Document Purpose

| Document | Purpose | Audience | When to Use |
|----------|---------|----------|-------------|
| SUMMARY.md | Quick reference, key commands | Developers, Release Managers | Daily standup, quick checks |
| Plan.md | Comprehensive planning, timeline | Project Managers, Stakeholders | Sprint planning, status reviews |
| Checklist.md | Detailed verification steps | Release Managers, QA | During release process |

### Related Documentation

- **[Release Process](../RELEASE_PROCESS.md)** - General release workflow and procedures
- **[Release Checklist Template](../RELEASE_CHECKLIST.md)** - Template for future releases
- **[Audit 007 Summary](../audits/codex-2025-12-02-docs-testlib-007/summary.md)** - Background context

## Release Status

**Current Version**: v0.1.0-alpha.15
**Target Version**: v0.2.0-alpha.0
**Status**: Ready for stabilization sprint
**Blocking Issues**: 1 (navigation IPC handler)
**GitHub Issue**: [#72](https://github.com/raibid-labs/scarab/issues/72)

### Progress Overview

#### Completed Work (10/11 from Audit 007)
- ✅ Documentation portal (mdBook)
- ✅ Central documentation index
- ✅ TESTING.md guide (757 lines)
- ✅ ratatui-testlib integration
- ✅ BevyTuiTestHarness stubs
- ✅ justfile test targets
- ✅ Telemetry HUD plugin
- ✅ Diagnostics recorder
- ✅ Graphics inspector
- ✅ Bevy UI inspector

#### In Progress
- ⏳ Accessibility support (#68)

#### Remaining Tasks
- 🔴 Fix navigation IPC handler (blocking)
- 🟡 Verify full test suite passes
- 🟡 Run all CI gates
- 🟡 Version bump and CHANGELOG update

## Quick Commands

```bash
# Test everything
just test-all              # Comprehensive test suite
just ci                    # Format, clippy, test

# Build documentation
just docs-build            # Build mdBook
just docs-serve            # Serve locally

# Release preparation
cargo check --workspace    # Verify compilation
cargo audit                # Security audit
```

## Release Timeline

**Week 1**: Stabilization (Dec 4-10)
- Fix blocking issues
- Verify test suite
- Ensure CI gates pass

**Week 2**: Release (Dec 11-17)
- Version bump
- Create release branch
- Tag and publish

## Key Decisions

### Scope
- **Included**: All audit 007 work (documentation, testing, developer tools)
- **Deferred**: Accessibility completion (#68), compiler warnings cleanup
- **Excluded**: Context menus (#31), plugin marketplace UI (#32), shader effects (#35)

### Breaking Changes
- Navigation system now ECS-native
- EventRegistry deprecated
- IPC protocol extended

### Success Criteria
- All blocking issues resolved
- All CI gates pass
- Documentation complete
- CHANGELOG updated
- No P0 bugs open

## Document Maintenance

### When to Update

- **SUMMARY.md**: Daily during sprint, after status changes
- **Plan.md**: Weekly, or when scope/timeline changes
- **Checklist.md**: During release execution, mark items complete

### Version History

| Date | Version | Status | Notes |
|------|---------|--------|-------|
| 2025-12-03 | v0.2.0-alpha.0 | Planning | Initial release plan created |

## References

- **Process Documentation**: `../RELEASE_PROCESS.md`, `../RELEASE_CHECKLIST.md`
- **Audit Reports**: `../audits/codex-2025-12-02-docs-testlib-007/`
- **GitHub Issues**: [#72 (stabilization sprint)](https://github.com/raibid-labs/scarab/issues/72)
- **Closed Issues**: #61, #62, #63, #64, #65, #66, #67, #69, #70, #71
- **Open Issues**: #68 (accessibility)

## Contact

For questions about this release:
- Review the planning documents in this directory
- Check GitHub Issue #72 for latest status
- Refer to `docs/RELEASE_PROCESS.md` for general process questions

---

**Last Updated**: 2025-12-03
**Maintained By**: Release Team
**Next Review**: Weekly during stabilization sprint
