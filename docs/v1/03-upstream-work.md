# Upstream Work Required: Fusabi-Lang Ecosystem

**Date**: 2025-12-11
**Repository**: https://github.com/fusabi-lang/fusabi

---

## Tracked Issues

| Package | Issue | Priority |
|---------|-------|----------|
| bevy-fusabi | [#6](https://github.com/fusabi-lang/bevy-fusabi/issues/6) | Critical |
| fusabi-tui | [#6](https://github.com/fusabi-lang/fusabi-tui/issues/6) | Critical |
| fusabi-host | [#8](https://github.com/fusabi-lang/fusabi-host/issues/8) | High |
| fusabi-plugin-runtime | [#8](https://github.com/fusabi-lang/fusabi-plugin-runtime/issues/8) | High |
| fusabi-stdlib-ext | [#8](https://github.com/fusabi-lang/fusabi-stdlib-ext/issues/8) | Medium |

---

## Overview

The Fusabi-lang ecosystem has version fragmentation across its packages. Several key integration libraries are pinned to older Fusabi versions (0.16.0 - 0.19.x) while the core language has advanced to v0.21.0.

This document outlines the required upstream work before Scarab can fully integrate Scryforge/Sigilforge plugins.

---

## Issue 1: bevy-fusabi Version Mismatch

**GitHub Issue**: https://github.com/fusabi-lang/bevy-fusabi/issues/6

### Current State
- **Package**: `bevy-fusabi`
- **Version**: 0.1.4
- **Fusabi Dependency**: 0.17.0
- **Required**: 0.21.0

### Problem
Scarab uses Bevy 0.15 and needs the latest Fusabi 0.21.0 features. The current `bevy-fusabi` crate is pinned to Fusabi 0.17.0, which lacks:
- Recent VM optimizations
- New stdlib functions
- Bug fixes since 0.17.0

### Proposed Changes

```toml
# bevy-fusabi/Cargo.toml
[dependencies]
fusabi-vm = "0.21.0"       # was: 0.17.0
fusabi-frontend = "0.21.0"  # was: 0.17.0
bevy = { version = "0.15", default-features = false, features = ["bevy_asset"] }
```

### API Changes Required
1. Update asset loading to new `fusabi-frontend` compile API
2. Adjust for any `Value` enum changes in VM
3. Update error handling if `Result` types changed

### GitHub Issue Template

**Title**: Update Fusabi dependencies to 0.21.0

**Body**:
```markdown
## Summary
Update fusabi-vm and fusabi-frontend dependencies from 0.17.0 to 0.21.0 for compatibility with latest Fusabi features and Scarab terminal integration.

## Motivation
- Scarab terminal requires Fusabi 0.21.0 for its plugin system
- 0.17.0 lacks recent VM optimizations and stdlib additions
- Version fragmentation causes integration issues

## Changes Required
- [ ] Update Cargo.toml dependencies
- [ ] Adjust compile API calls if changed
- [ ] Update Value enum handling if needed
- [ ] Run existing tests
- [ ] Update version to 0.2.0

## Related
- Scarab integration: https://github.com/raibid-labs/scarab
- Fusabi changelog: [link to 0.21.0 release notes]
```

---

## Issue 2: fusabi-tui Version Mismatch

**GitHub Issue**: https://github.com/fusabi-lang/fusabi-tui/issues/6

### Current State
- **Package**: `fusabi-tui`
- **Version**: 0.2.0
- **Fusabi Dependency**: 0.16.0
- **Required**: 0.21.0

### Problem
`fusabi-tui` provides Ratatui bindings for Fusabi scripts. It's essential for Scryforge's TUI widgets but is pinned to 0.16.0.

### Proposed Changes

```toml
# fusabi-tui/Cargo.toml
[dependencies]
fusabi-vm = { version = "0.21.0", optional = true }
ratatui = "0.29"  # also update if needed
```

### API Changes Required
1. Update host function registration API
2. Adjust for Value type changes
3. Review any breaking changes in stdlib bindings

### GitHub Issue Template

**Title**: Update Fusabi dependency to 0.21.0

**Body**:
```markdown
## Summary
Update fusabi-vm dependency from 0.16.0 to 0.21.0 for Scryforge integration.

## Motivation
- Scryforge uses fusabi-tui for TUI widgets
- Scarab requires Fusabi 0.21.0 for plugin system
- Version mismatch prevents clean integration

## Changes Required
- [ ] Update Cargo.toml fusabi-vm to 0.21.0
- [ ] Review host function registration API
- [ ] Update any Value type handling
- [ ] Run widget tests
- [ ] Update version to 0.3.0

## Testing
All existing examples should continue to work:
- [ ] table.fsx
- [ ] list.fsx
- [ ] gauge.fsx
- [ ] chart.fsx
```

---

## Issue 3: fusabi-host Documentation

**GitHub Issue**: https://github.com/fusabi-lang/fusabi-host/issues/8

### Current State
- **Package**: `fusabi-host`
- **Version**: 0.1.0
- **Documented Compatibility**: 0.18.x - 0.19.x
- **Required**: 0.21.0 (verification)

### Problem
Documentation claims compatibility with 0.18-0.19 but doesn't mention 0.21.0. Need to verify and update.

### Proposed Changes

1. Test with Fusabi 0.21.0
2. Update documentation to specify 0.21.0 compatibility
3. Bump version if any changes needed

### GitHub Issue Template

**Title**: Verify and document Fusabi 0.21.0 compatibility

**Body**:
```markdown
## Summary
Verify fusabi-host works with Fusabi 0.21.0 and update compatibility documentation.

## Current Documentation
> Compatible with Fusabi 0.18.x - 0.19.x

## Requested
- [ ] Test EnginePool with Fusabi 0.21.0
- [ ] Test capability system
- [ ] Test host function registration
- [ ] Update README compatibility section
- [ ] Update Cargo.toml version constraints if needed
```

---

## Issue 4: fusabi-plugin-runtime API Version

**GitHub Issue**: https://github.com/fusabi-lang/fusabi-plugin-runtime/issues/8

### Current State
- **Package**: `fusabi-plugin-runtime`
- **Version**: 0.1.1
- **API Version in Manifests**: `{ major = 0, minor = 18, patch = 0 }`

### Problem
Plugin manifests specify API version 0.18.0 but the runtime should support 0.21.0 plugins.

### Proposed Changes

Update default API version validation:

```rust
// fusabi-plugin-runtime/src/manifest.rs
pub const CURRENT_API_VERSION: ApiVersion = ApiVersion {
    major: 0,
    minor: 21,  // was: 18
    patch: 0,
};
```

### GitHub Issue Template

**Title**: Update API version to 0.21.0

**Body**:
```markdown
## Summary
Update the plugin runtime's API version to 0.21.0 to match current Fusabi release.

## Changes
- [ ] Update CURRENT_API_VERSION constant
- [ ] Update documentation examples
- [ ] Ensure backward compatibility with 0.18.x plugins
- [ ] Add migration guide if breaking changes
```

---

## Issue 5: fusabi-stdlib-ext Compatibility

**GitHub Issue**: https://github.com/fusabi-lang/fusabi-stdlib-ext/issues/8

### Current State
- **Package**: `fusabi-stdlib-ext`
- **Version**: 0.1.0
- **Fusabi Dependency**: Not explicitly versioned in docs

### Problem
Need to verify compatibility with 0.21.0 and update if necessary.

### Proposed Changes

```toml
# fusabi-stdlib-ext/Cargo.toml
[dependencies]
fusabi-vm = "0.21.0"
```

### GitHub Issue Template

**Title**: Verify Fusabi 0.21.0 compatibility

**Body**:
```markdown
## Summary
Verify all stdlib-ext modules work with Fusabi 0.21.0.

## Modules to Test
- [ ] process
- [ ] fs
- [ ] path
- [ ] env
- [ ] format
- [ ] net
- [ ] time
- [ ] metrics
```

---

## Issue 6: Community Package Compatibility

### Current State
Community packages in `fusabi-community/packages/` may have outdated dependencies.

### Affected Packages
- `json` - Pure library, likely compatible
- `commander` - TUI app, may need updates

### Proposed Changes

1. Add CI job to test packages against latest Fusabi
2. Update any incompatible packages
3. Add version matrix to package metadata

### GitHub Issue Template

**Title**: Add Fusabi version compatibility CI

**Body**:
```markdown
## Summary
Add CI workflow to test community packages against multiple Fusabi versions.

## Proposed Matrix
- Fusabi 0.19.x (legacy)
- Fusabi 0.20.x (stable)
- Fusabi 0.21.x (latest)

## Benefits
- Catch compatibility issues early
- Clear version support documentation
- Easier upgrades for users
```

---

## Priority Order

1. **Critical (Blocking Scarab Integration)**
   - bevy-fusabi → 0.21.0
   - fusabi-tui → 0.21.0

2. **High (Required for Full Integration)**
   - fusabi-plugin-runtime API version update
   - fusabi-host documentation update

3. **Medium (Nice to Have)**
   - fusabi-stdlib-ext verification
   - Community package CI

---

## Coordination Plan

### Step 1: Core Updates
1. Fork `bevy-fusabi` and `fusabi-tui`
2. Update dependencies in forks
3. Run tests, fix any issues
4. Submit PRs upstream

### Step 2: Documentation PRs
1. Update `fusabi-host` README
2. Update `fusabi-plugin-runtime` docs
3. Add migration guide if breaking changes

### Step 3: CI/Infrastructure
1. Add version matrix to CI
2. Set up automated compatibility testing
3. Add deprecation warnings for old versions

---

## Timeline Estimate

| Task | Estimate | Blocked By |
|------|----------|------------|
| bevy-fusabi PR | 2-3 days | None |
| fusabi-tui PR | 2-3 days | None |
| fusabi-host docs | 1 day | Testing |
| fusabi-plugin-runtime | 1 day | None |
| fusabi-stdlib-ext | 1 day | None |
| Community package CI | 2 days | None |
| **Total** | **9-11 days** | - |

---

## Appendix: Version Compatibility Table

| Package | Current | Fusabi Dep | Target | Notes |
|---------|---------|------------|--------|-------|
| fusabi (core) | 0.21.0 | - | - | Reference |
| fusabi-vm | 0.21.0 | - | - | Reference |
| fusabi-frontend | 0.21.0 | - | - | Reference |
| bevy-fusabi | 0.1.4 | 0.17.0 | 0.21.0 | **Update needed** |
| fusabi-tui | 0.2.0 | 0.16.0 | 0.21.0 | **Update needed** |
| fusabi-host | 0.1.0 | 0.18-0.19 | 0.21.0 | Verify |
| fusabi-plugin-runtime | 0.1.1 | 0.18.x | 0.21.0 | API version |
| fusabi-stdlib-ext | 0.1.0 | Unknown | 0.21.0 | Verify |

---

## Contact

For questions about these upstream changes, coordinate with:
- Fusabi-lang maintainer
- Scarab integration lead
- Scryforge maintainer
