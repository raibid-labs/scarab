# Release Process

This document describes the release process for Scarab.

## Overview

Scarab follows semantic versioning (SemVer) and uses GitHub Actions for automated releases.

## Release Workflow

### 1. Pre-Release Checklist

Before creating a release:

- [ ] All tests pass (`cargo test --workspace`)
- [ ] CI is green on main branch
- [ ] CHANGELOG.md is updated
- [ ] VERSION file is updated
- [ ] Documentation is up to date
- [ ] No outstanding critical bugs
- [ ] All planned features for release are merged

### 2. Version Bump

Update version numbers in:

1. **`VERSION`** file (single source of truth)
2. **`Cargo.toml`** (workspace version)
3. **`CHANGELOG.md`** (add release section)
4. **`README.md`** (update badges if needed)

Example:
```bash
# Update VERSION file
echo "0.2.0" > VERSION

# Cargo.toml will use workspace.package.version
# CHANGELOG.md needs manual update
```

### 3. Create Release Branch (for major/minor releases)

```bash
git checkout -b release/v0.2.0
git push origin release/v0.2.0
```

### 4. Tag the Release

```bash
# Create annotated tag
git tag -a v0.2.0 -m "Release v0.2.0"

# Push tag to trigger release workflow
git push origin v0.2.0
```

### 5. GitHub Release

The release workflow (`.github/workflows/release.yml`) will:

1. Build binaries for Linux, macOS, Windows
2. Run full test suite
3. Create GitHub release with binaries
4. Generate release notes from CHANGELOG
5. Publish crates to crates.io (if configured)

### 6. Post-Release

After release is published:

1. Verify binaries are available
2. Test installation from GitHub releases
3. Update documentation website (if applicable)
4. Announce release on social media/channels
5. Close milestone in GitHub

## Release Types

### Patch Releases (0.1.x)

- Bug fixes only
- No new features
- Backward compatible
- Fast-tracked process

### Minor Releases (0.x.0)

- New features
- Backward compatible
- Deprecations allowed
- Full testing required

### Major Releases (x.0.0)

- Breaking changes
- API changes
- Migration guide required
- Extended testing period

## Release Workflow File

The release workflow is defined in `.github/workflows/release.yml`:

### Triggers

- Push of version tags (`v*.*.*`)
- Manual workflow dispatch (for testing)

### Jobs

1. **Build**: Compile binaries for all platforms
2. **Test**: Run full test suite
3. **Package**: Create distribution packages
4. **Release**: Create GitHub release with artifacts

### Artifacts

- Linux binary (x86_64, aarch64)
- macOS binary (x86_64, aarch64)
- Windows binary (x86_64)
- Source tarball
- Checksums (SHA256)

## Branch Protection

The `main` branch has the following protections:

### Required Checks

- CI must pass (`.github/workflows/ci.yml`)
- All tests must pass
- No merge conflicts

### Required Reviews

- At least 1 approving review from CODEOWNERS
- Changes requested must be resolved
- Stale reviews are dismissed on new commits

### Merge Requirements

- Squash merging preferred for feature branches
- Linear history maintained
- No force pushes allowed

## CODEOWNERS

The `.github/CODEOWNERS` file defines review requirements:

```
# Codeowners for scarab repository

# Default reviewers for everything
* @scarab-team @raibid-labs/core

# Core crates require core team review
/crates/scarab-daemon/ @raibid-labs/core
/crates/scarab-client/ @raibid-labs/core
/crates/scarab-protocol/ @raibid-labs/core

# Plugin API requires plugin team review
/crates/scarab-plugin-api/ @raibid-labs/plugins @raibid-labs/core

# Documentation can be reviewed by docs team
/docs/ @raibid-labs/docs
*.md @raibid-labs/docs

# CI/CD requires ops review
/.github/ @raibid-labs/ops @raibid-labs/core
```

## Emergency Hotfixes

For critical security or bug fixes:

1. Create hotfix branch from latest release tag
2. Apply minimal fix
3. Fast-track review process
4. Create patch release immediately
5. Backport to main

Example:
```bash
git checkout v0.1.9
git checkout -b hotfix/security-fix
# Apply fix
git commit -m "fix: security vulnerability"
git tag v0.1.10
git push origin v0.1.10
git checkout main
git cherry-pick <hotfix-commit>
```

## Release Checklist Template

Copy this for each release:

```markdown
## Release v0.x.x Checklist

### Pre-Release
- [ ] All tests passing
- [ ] CI green
- [ ] CHANGELOG.md updated
- [ ] VERSION file updated
- [ ] Documentation reviewed
- [ ] No critical bugs

### Release Process
- [ ] Version bumped in all files
- [ ] Release branch created (if major/minor)
- [ ] Tag created and pushed
- [ ] GitHub release created
- [ ] Binaries verified

### Post-Release
- [ ] Installation tested
- [ ] Documentation updated
- [ ] Announcement prepared
- [ ] Milestone closed
```

## Troubleshooting

### Release Workflow Fails

1. Check CI logs for errors
2. Verify all tests pass locally
3. Check for dependency issues
4. Retry workflow if transient failure

### Binary Build Fails

1. Check cross-compilation setup
2. Verify dependencies available on target
3. Check for platform-specific code issues

### Tag Already Exists

If you need to recreate a tag:

```bash
# Delete local tag
git tag -d v0.x.x

# Delete remote tag
git push origin :refs/tags/v0.x.x

# Create new tag
git tag -a v0.x.x -m "Release v0.x.x"
git push origin v0.x.x
```

## Related Documentation

- [CHANGELOG.md](../CHANGELOG.md) - Release history
- [CONTRIBUTING.md](../CONTRIBUTING.md) - Contribution guidelines
- [.github/workflows/release.yml](../.github/workflows/release.yml) - Release automation
- [VERSION](../VERSION) - Current version
