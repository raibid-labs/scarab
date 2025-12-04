# Release Documentation

This directory contains release planning, checklists, and tracking documents for Scarab Terminal releases.

## v0.2.0-alpha.0 Release

The next release transitioning from perpetual alpha (v0.1.0-alpha.X) to structured versioning.

### Quick Access

- **[Quick Summary](./v0.2.0-alpha-SUMMARY.md)** - At-a-glance status and commands (4.7 KB, 167 lines)
- **[Release Plan](./v0.2.0-alpha-plan.md)** - Comprehensive planning document (13 KB, 379 lines)
- **[Detailed Checklist](./v0.2.0-alpha.0-checklist.md)** - Step-by-step verification (6.0 KB, 163 lines)
- **[Beta Criteria](./v0.2.0-beta-criteria.md)** - Requirements for beta transition (Issue #79)

### Document Purpose

| Document | Purpose | Audience | When to Use |
|----------|---------|----------|-------------|
| SUMMARY.md | Quick reference, key commands | Developers, Release Managers | Daily standup, quick checks |
| Plan.md | Comprehensive planning, timeline | Project Managers, Stakeholders | Sprint planning, status reviews |
| Checklist.md | Detailed verification steps | Release Managers, QA | During release process |
| Beta Criteria | Beta readiness requirements | Release Team, Leadership | Evaluating beta transition |

### Related Documentation

- **[Release Process](../RELEASE_PROCESS.md)** - General release workflow and procedures
- **[Release Checklist Template](../RELEASE_CHECKLIST.md)** - Template for future releases
- **[Audit 007 Summary](../audits/codex-2025-12-02-docs-testlib-007/summary.md)** - Background context

## Release Status

**Current Version**: v0.1.0-alpha.15
**Target Version**: v0.2.0-alpha.0
**Status**: Ready for stabilization sprint
**Blocking Issues**: 1 (navigation IPC handler)
**GitHub Issues**: [#72](https://github.com/raibid-labs/scarab/issues/72) (alpha release), [#79](https://github.com/raibid-labs/scarab/issues/79) (beta criteria)

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

## Release Roadmap

### Alpha Phase (Current)
**Goal**: Stabilize core features and infrastructure

- **v0.2.0-alpha.0**: Initial structured release (audit 007 consolidation)
- **v0.2.0-alpha.1+**: Iterative improvements, bug fixes, feature completion

**Duration**: 4-6 weeks of alpha releases

### Beta Phase (Next)
**Goal**: Production-ready code suitable for community testing

See [v0.2.0-beta-criteria.md](./v0.2.0-beta-criteria.md) for comprehensive requirements.

**Key Beta Requirements**:
- All CI jobs green (test, lint, docs, security)
- Plugin ABI frozen (no breaking changes)
- Navigation system stable
- Configuration schema finalized
- IPC protocol versioned
- Full documentation deployed
- Migration guide complete
- Zero P0/P1 bugs

**Duration**: 2-week minimum stability period

### Stable Phase (Future)
**Goal**: Public release suitable for production use

- No breaking changes from beta
- Community feedback incorporated
- Security audit passed
- Performance baselines met
- Package manager submissions

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

**Weeks 3-8**: Alpha iterations
- Bug fixes and improvements
- Feature completion
- Community feedback

**Weeks 9-10**: Beta stabilization
- Beta release
- Community testing
- Final bug fixes

**Week 11+**: Stable release
- Production release
- Package manager submissions
- Public announcement

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

#### Alpha Success Criteria
- All blocking issues resolved
- All CI gates pass
- Documentation complete
- CHANGELOG updated
- No P0 bugs open

#### Beta Success Criteria
See [v0.2.0-beta-criteria.md](./v0.2.0-beta-criteria.md) for comprehensive list:
- All quality gates pass
- Feature freeze (ABI/API stable)
- Security audit complete
- Documentation deployed
- 100+ community testers

#### Stable Success Criteria
- 2-week beta stability period
- Zero P0/P1 bugs
- Community feedback positive (>75%)
- Performance baselines met
- Package manager ready

## Document Maintenance

### When to Update

- **SUMMARY.md**: Daily during sprint, after status changes
- **Plan.md**: Weekly, or when scope/timeline changes
- **Checklist.md**: During release execution, mark items complete
- **Beta Criteria**: When evaluating beta readiness
- **README.md**: When release status/roadmap changes

### Version History

| Date | Version | Status | Notes |
|------|---------|--------|-------|
| 2025-12-03 | v0.2.0-alpha.0 | Planning | Initial release plan created |
| 2025-12-03 | v0.2.0-beta.0 | Planning | Beta criteria documented (Issue #79) |

## References

- **Process Documentation**: `../RELEASE_PROCESS.md`, `../RELEASE_CHECKLIST.md`
- **Audit Reports**: `../audits/codex-2025-12-02-docs-testlib-007/`
- **GitHub Issues**:
  - [#72 (stabilization sprint)](https://github.com/raibid-labs/scarab/issues/72)
  - [#79 (beta criteria)](https://github.com/raibid-labs/scarab/issues/79)
- **Closed Issues**: #61, #62, #63, #64, #65, #66, #67, #69, #70, #71
- **Open Issues**: #68 (accessibility)

## Contact

For questions about this release:
- Review the planning documents in this directory
- Check GitHub Issues #72 (alpha) and #79 (beta) for latest status
- Refer to `docs/RELEASE_PROCESS.md` for general process questions

---

**Last Updated**: 2025-12-03
**Maintained By**: Release Team
**Next Review**: Weekly during stabilization sprint
