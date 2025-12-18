# Release Documentation

This directory contains comprehensive documentation for managing releases of Scarab Terminal.

## Quick Links

- **[RELEASE_CHECKLIST.md](./RELEASE_CHECKLIST.md)** - Step-by-step checklist for every release
- **[RELEASE_PROCESS.md](./RELEASE_PROCESS.md)** - Complete release management process
- **[Templates](./templates/)** - Announcement and communication templates

## Overview

Scarab uses a structured release process with comprehensive documentation and automation to ensure consistent, high-quality releases.

### Key Documents

#### 1. Release Checklist (`RELEASE_CHECKLIST.md`)

A practical, step-by-step checklist covering:
- Pre-release testing and validation
- Documentation updates
- Version management
- Git tagging and releases
- Package manager updates
- Post-release verification
- Communication and announcements

**When to use**: Every release, from alpha to stable. Print or keep open while releasing.

#### 2. Release Process (`RELEASE_PROCESS.md`)

Comprehensive process documentation covering:
- Release types (alpha, beta, rc, stable, hotfix)
- Version numbering scheme (semantic versioning)
- Branch strategy (Git Flow)
- Release cadence and timelines
- Detailed workflows
- Rollback procedures
- Communication plans
- Troubleshooting guide

**When to use**: Reference for understanding the overall process, planning releases, or resolving issues.

#### 3. Templates Directory (`templates/`)

Ready-to-use templates for:
- **release-notes-template.md** - GitHub Release page content
- **release-announcement-template.md** - Social media, discussions, blog posts
- **hotfix-announcement-template.md** - Emergency release communications

**When to use**: When drafting release announcements or responding to critical issues.

## Quick Start

### First Time Releasing

1. **Read the Process Document**
   ```bash
   cat docs/RELEASE_PROCESS.md
   ```
   Understand release types, versioning, and branch strategy.

2. **Review the Checklist**
   ```bash
   cat docs/RELEASE_CHECKLIST.md
   ```
   Familiarize yourself with all steps.

3. **Install Required Tools**
   ```bash
   cargo install cargo-audit
   rustup update stable
   ```

4. **Review Templates**
   ```bash
   ls docs/templates/
   ```
   See what announcements you'll need to prepare.

### Preparing a Release

1. **Run Pre-Release Check**
   ```bash
   ./scripts/check-release.sh
   ```
   Validates code quality, tests, and documentation.

2. **Update CHANGELOG**
   ```bash
   $EDITOR CHANGELOG.md
   ```
   Move items from `[Unreleased]` to new version section.

3. **Bump Version**
   ```bash
   ./scripts/bump-version.sh X.Y.Z
   ```
   Updates all crate versions automatically.

4. **Follow the Checklist**
   Open `docs/RELEASE_CHECKLIST.md` and work through each item.

### Making the Release

1. **Create and Push Tag**
   ```bash
   git tag -a vX.Y.Z -m "Release vX.Y.Z"
   git push origin vX.Y.Z
   ```

2. **Monitor GitHub Actions**
   - Visit: https://github.com/raibid-labs/scarab/actions
   - Ensure all builds complete successfully

3. **Publish Release**
   - Edit the draft release on GitHub
   - Use template from `docs/templates/release-notes-template.md`
   - Publish when ready

4. **Announce Release**
   - Use templates from `docs/templates/release-announcement-template.md`
   - Post to GitHub Discussions, social media, etc.

### Emergency Hotfix

If a critical issue is found after release:

1. **Follow Hotfix Process**
   See "Rollback Procedure" in `docs/RELEASE_PROCESS.md`

2. **Use Hotfix Template**
   ```bash
   cat docs/templates/hotfix-announcement-template.md
   ```

3. **Execute Quickly**
   Hotfixes should be released within hours, not days.

## Automation Scripts

### `/scripts/bump-version.sh`

Automatically updates version numbers across all workspace crates.

**Usage**:
```bash
./scripts/bump-version.sh 0.2.0
./scripts/bump-version.sh 1.0.0-beta.1
```

**What it does**:
- Updates version in all `Cargo.toml` files
- Updates inter-crate dependencies
- Updates `Cargo.lock`
- Verifies the workspace still builds

**Next steps** (it tells you):
- Review changes with `git diff`
- Update CHANGELOG.md
- Commit and tag

### `/scripts/check-release.sh`

Comprehensive pre-release validation.

**Usage**:
```bash
./scripts/check-release.sh
```

**What it checks**:
- Git status (clean working directory)
- Code formatting (`cargo fmt`)
- Linter warnings (`cargo clippy`)
- Test suite (`cargo test`)
- Build success (`cargo build`)
- Security audit (`cargo audit`)
- Version synchronization
- CHANGELOG.md status
- Documentation completeness
- Dependency status

**Exit codes**:
- `0`: All checks passed, ready to release
- `1`: Critical checks failed, fix before releasing

## GitHub Actions Workflow

The release process is partially automated via `.github/workflows/release.yml`.

### Triggered By

- Pushing a tag matching `v*` (e.g., `v0.1.0`)
- Manual workflow dispatch with tag input

### What It Does

1. **Creates Release**
   - Generates GitHub Release draft
   - Uses template from workflow

2. **Builds Binaries**
   - Matrix build for all platforms:
     - macOS (Apple Silicon, Intel)
     - Linux (glibc, musl, ARM64)
     - Windows (MSVC)
   - Strips binaries for smaller size
   - Creates archives (`.tar.gz` or `.zip`)
   - Uploads to GitHub Release

3. **Publishes to Crates.io**
   - Publishes all workspace crates in dependency order
   - Uses `CARGO_REGISTRY_TOKEN` secret

4. **Updates Package Managers**
   - Homebrew formula update (macOS)
   - AUR PKGBUILD update (manual follow-up needed)

### What You Must Do Manually

- Write release notes (use template)
- Publish the release (convert from draft)
- Announce the release
- Monitor for issues

## Release Types and Cadence

| Type | Frequency | Purpose |
|------|-----------|---------|
| **Alpha** | As needed | Early testing, incomplete features |
| **Beta** | 2-4 weeks before stable | Feature-complete, wider testing |
| **RC** | 1 week before stable | Final validation |
| **Stable** | Every 6-12 weeks | Production-ready |
| **Hotfix** | As needed | Critical bug fixes |

See `docs/RELEASE_PROCESS.md` for detailed descriptions of each type.

## Version Numbering

Scarab follows [Semantic Versioning 2.0.0](https://semver.org/):

**Format**: `MAJOR.MINOR.PATCH[-PRERELEASE]`

**Examples**:
- `0.1.0-alpha.1` - First alpha of v0.1.0
- `0.1.0-beta.1` - First beta of v0.1.0
- `0.1.0` - Stable release
- `0.1.1` - Patch/hotfix release
- `0.2.0` - Minor release with new features
- `1.0.0` - Major release with breaking changes

**Rules**:
- MAJOR: Breaking changes (rare before 1.0)
- MINOR: New features, backwards-compatible
- PATCH: Bug fixes only, backwards-compatible
- PRERELEASE: alpha < beta < rc < stable

## Branch Strategy

**Main Branches**:
- `main` - Production-ready code, all releases tagged from here
- `develop` - Integration branch for next release

**Supporting Branches**:
- `feature/*` - New features
- `release/*` - Release preparation
- `hotfix/*` - Emergency fixes

**Example Flow**:
```bash
# Feature development
git checkout -b feature/my-feature develop
# ... work ...
git checkout develop
git merge --no-ff feature/my-feature

# Release preparation
git checkout -b release/v0.2.0 develop
# ... version bumps, changelog, testing ...
git checkout main
git merge --no-ff release/v0.2.0
git tag -a v0.2.0
git push origin main v0.2.0

# Hotfix
git checkout -b hotfix/v0.2.1 v0.2.0
# ... fix ...
git checkout main
git merge --no-ff hotfix/v0.2.1
git tag -a v0.2.1
```

## Communication Channels

### Pre-Release
- GitHub Discussions: Announce upcoming release
- Team chat: Coordinate preparation

### Release Day
- GitHub Release: Official release notes
- GitHub Discussions: Announcement post
- Social media: Twitter/X, Reddit, Mastodon
- Community: Discord, forums

### Post-Release
- Monitor GitHub Issues for bug reports
- Respond to community feedback
- Weekly summary of adoption metrics

## Troubleshooting

See the "Troubleshooting" section in `docs/RELEASE_PROCESS.md` for solutions to common issues:

- GitHub Actions build failures
- crates.io publish errors
- Package manager update problems
- Release tag/workflow issues
- Rollback procedures

## Best Practices

### Before Every Release

1. **Run the check script**: `./scripts/check-release.sh`
2. **Test manually**: Don't rely solely on automation
3. **Update docs**: Ensure all changes are documented
4. **Review CHANGELOG**: Make it user-friendly
5. **Test package installation**: Homebrew, cargo install, etc.

### During Release

1. **Follow the checklist**: Don't skip steps
2. **Monitor builds**: Watch GitHub Actions progress
3. **Test downloads**: Verify binaries work
4. **Be ready to rollback**: Have hotfix plan ready

### After Release

1. **Monitor issues**: First 24-48 hours are critical
2. **Respond quickly**: Acknowledge reports promptly
3. **Document problems**: Update checklist if issues found
4. **Thank contributors**: Recognition builds community

## For Team Members

### Release Manager Role

Each release should have a designated Release Manager who:
- Owns the end-to-end process
- Coordinates with contributors
- Makes go/no-go decisions
- Handles communication
- Ensures documentation is updated

### First-Time Release Manager

If this is your first time managing a release:

1. **Read everything**: Start with `RELEASE_PROCESS.md`
2. **Shadow a release**: Observe someone else first (if possible)
3. **Start small**: First release could be a patch or RC
4. **Ask questions**: Better to ask than assume
5. **Document issues**: Help improve the process

### Delegating Tasks

For major releases, consider delegating:
- QA: Run test suite and manual testing
- Docs: Review and update documentation
- Packaging: Update Homebrew/AUR
- Comms: Draft and post announcements
- Monitoring: Watch for issues post-release

## Continuous Improvement

This documentation should evolve with our process:

### After Each Release

1. **Conduct retrospective**: What went well? What didn't?
2. **Update docs**: Fix gaps or confusing sections
3. **Improve automation**: Automate repetitive manual steps
4. **Update templates**: Refine based on what worked
5. **Share learnings**: Document in post-mortem

### Contributing Improvements

If you find issues with this documentation:

1. **Create an issue**: Describe the problem
2. **Submit a PR**: Fix documentation directly
3. **Discuss changes**: Major process changes need team buy-in

## Resources

### Internal Links

- [ROADMAP.md](../ROADMAP.md) - Project roadmap
- [CHANGELOG.md](../CHANGELOG.md) - Version history
- [README.md](../README.md) - Project overview
- [CLAUDE.md](../CLAUDE.md) - Technical architecture

### External References

- [Semantic Versioning](https://semver.org/)
- [Keep a Changelog](https://keepachangelog.com/)
- [Cargo Publishing](https://doc.rust-lang.org/cargo/reference/publishing.html)
- [GitHub Releases](https://docs.github.com/en/repositories/releasing-projects-on-github)
- [Homebrew Formula](https://docs.brew.sh/Formula-Cookbook)
- [Arch AUR](https://wiki.archlinux.org/title/AUR_submission_guidelines)

## Questions?

If you have questions about the release process:

1. **Check this documentation first** - Most answers are here
2. **Search past releases** - See how previous releases were done
3. **Ask the team** - In chat or GitHub Discussions
4. **Update the docs** - If the answer wasn't documented, add it

---

**Last Updated**: 2025-11-24
**Next Review**: Before first stable release (v1.0.0)
