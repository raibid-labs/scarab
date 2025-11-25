# Release Process

This document describes the release management process for Scarab Terminal, including release types, versioning strategy, branch management, and detailed procedures.

## Table of Contents

- [Release Types](#release-types)
- [Version Numbering](#version-numbering)
- [Branch Strategy](#branch-strategy)
- [Release Cadence](#release-cadence)
- [Release Workflow](#release-workflow)
- [Rollback Procedure](#rollback-procedure)
- [Communication Plan](#communication-plan)
- [Roles and Responsibilities](#roles-and-responsibilities)

## Release Types

Scarab follows a structured release process with multiple release types:

### 1. Alpha Releases (vX.Y.Z-alpha.N)

**Purpose**: Early testing of new features with known limitations.

**Characteristics**:
- Incomplete feature set
- May have known bugs
- API may change without notice
- No stability guarantees
- Not recommended for production use

**Audience**: Early adopters, testers, contributors

**Cadence**: As needed during active development

**Example**: `v0.1.0-alpha.1`, `v0.2.0-alpha.3`

**Release Criteria**:
- Core functionality works
- No critical crashes
- Basic documentation exists
- Tests pass for implemented features

### 2. Beta Releases (vX.Y.Z-beta.N)

**Purpose**: Feature-complete releases for wider testing before stable.

**Characteristics**:
- All planned features implemented
- API mostly stable (minor changes possible)
- Known bugs documented
- Performance not yet optimized
- Suitable for testing environments

**Audience**: Advanced users, testers, plugin developers

**Cadence**: 2-4 weeks before stable release

**Example**: `v0.1.0-beta.1`, `v1.0.0-beta.2`

**Release Criteria**:
- Feature complete for milestone
- All critical bugs fixed
- Documentation complete
- Full test coverage
- Performance acceptable

### 3. Release Candidates (vX.Y.Z-rc.N)

**Purpose**: Final validation before stable release.

**Characteristics**:
- API frozen
- No new features
- Only critical bugfixes
- Production-ready quality
- Final documentation review

**Audience**: All users willing to test

**Cadence**: 1-2 weeks before stable, only if needed

**Example**: `v0.1.0-rc.1`, `v1.0.0-rc.2`

**Release Criteria**:
- No known critical bugs
- Performance targets met
- Security audit passed
- Documentation finalized
- Package managers ready

### 4. Stable Releases (vX.Y.Z)

**Purpose**: Production-ready releases with full support.

**Characteristics**:
- Fully tested and validated
- API stable per semver
- Complete documentation
- Performance optimized
- Security reviewed
- Full support commitment

**Audience**: All users

**Cadence**: Every 6-12 weeks for minor, as needed for patch

**Example**: `v0.1.0`, `v1.0.0`, `v1.1.0`

**Release Criteria**:
- All release criteria met
- Release candidate tested successfully
- Zero known critical bugs
- Package managers updated
- Announcement prepared

### 5. Hotfix Releases (vX.Y.Z+1)

**Purpose**: Critical bugfixes for stable releases.

**Characteristics**:
- Patch version bump only
- Minimal changes
- Security fixes
- Critical bug fixes only
- Fast-tracked process

**Audience**: All users on affected versions

**Cadence**: As needed for critical issues

**Example**: `v1.0.1`, `v1.0.2`

**Release Criteria**:
- Fixes critical issue
- No feature changes
- Minimal risk of regression
- Emergency approval if needed

## Version Numbering

Scarab follows [Semantic Versioning 2.0.0](https://semver.org/):

### Format: MAJOR.MINOR.PATCH[-PRERELEASE][+BUILD]

**MAJOR** (X.0.0): Breaking changes
- Incompatible API changes
- Major architectural changes
- Plugin API breaking changes
- Configuration format changes
- Shared memory protocol changes

**MINOR** (0.X.0): New features, backwards-compatible
- New features added
- Deprecations (with migration path)
- Performance improvements
- New plugin capabilities
- Enhanced UI features

**PATCH** (0.0.X): Bugfixes, backwards-compatible
- Bug fixes only
- Security patches
- Documentation fixes
- Performance fixes (no behavior change)

**PRERELEASE**: Alpha, beta, rc suffixes
- Format: `-alpha.N`, `-beta.N`, `-rc.N`
- N starts at 1 and increments for each prerelease
- Order: alpha < beta < rc < stable

**BUILD**: Build metadata (rarely used)
- Format: `+20250124.commit-hash`
- Does not affect version precedence

### Version Progression Examples

**Pre-1.0 Development**:
```
0.1.0-alpha.1 → 0.1.0-alpha.2 → 0.1.0-beta.1 → 0.1.0-rc.1 → 0.1.0
0.1.1 (hotfix)
0.2.0-alpha.1 → ... → 0.2.0
...
0.9.0 → 0.9.1 → 0.10.0 (last before 1.0)
1.0.0-rc.1 → 1.0.0 (stable API commitment)
```

**Post-1.0 Development**:
```
1.0.0 → 1.0.1 (hotfix) → 1.1.0-beta.1 → 1.1.0
1.1.0 → 1.1.1 → 1.2.0
2.0.0-alpha.1 → ... → 2.0.0-rc.1 → 2.0.0 (breaking changes)
```

### Workspace Version Synchronization

All workspace crates MUST have synchronized versions:
- `scarab-daemon` = `scarab-client` = `scarab-protocol` = etc.
- Use `cargo-workspaces` or manual updates
- Version bumps affect all crates simultaneously
- Inter-crate dependencies use exact version matching

## Branch Strategy

Scarab uses a simplified Git Flow model:

### Main Branches

**`main`** (protected)
- Always deployable
- Contains latest stable or RC code
- All releases tagged from this branch
- Requires PR review for changes
- CI must pass before merge

**`develop`** (protected)
- Integration branch for features
- Next release staging area
- May be unstable
- Requires PR review
- CI must pass before merge

### Supporting Branches

**Feature Branches** (`feature/*`)
- Created from: `develop`
- Merged into: `develop`
- Naming: `feature/issue-number-short-description`
- Example: `feature/123-plugin-hot-reload`
- Delete after merge

**Release Branches** (`release/*`)
- Created from: `develop`
- Merged into: `main` and `develop`
- Naming: `release/vX.Y.Z`
- Example: `release/v0.1.0`
- Only bugfixes and polish
- Delete after release

**Hotfix Branches** (`hotfix/*`)
- Created from: `main` (specific tag)
- Merged into: `main` and `develop`
- Naming: `hotfix/vX.Y.Z`
- Example: `hotfix/v1.0.1`
- Critical fixes only
- Delete after release

### Branch Workflow

```
main     ─────v0.1.0────────────v0.2.0─────────
              │                 │
release       └──────v0.2.0────┘
                     │
develop  ─────────────┴──────────────────────
         │      │           │
feature  ├─────┘            └────────
```

## Release Cadence

### Target Schedule

**Minor Releases**: Every 6-12 weeks
- Planned features complete
- Regular cadence for predictability
- Announced in roadmap

**Patch Releases**: As needed
- Critical bugs: within 24-48 hours
- Non-critical bugs: batched weekly/monthly
- Security issues: immediate

**Pre-releases**:
- Alpha: Weekly during development sprints
- Beta: 2-4 weeks before stable
- RC: 1 week before stable (if needed)

### Release Windows

**Preferred Days**: Tuesday or Wednesday
- Allows time for monitoring
- Avoids Monday rush and Friday risks
- Team available for support

**Avoid**: Holidays, weekends, major conferences

## Release Workflow

### 1. Planning Phase

**Timeline**: 2-4 weeks before release

1. Review ROADMAP.md and milestone issues
2. Triage remaining issues:
   - Must-have (blocks release)
   - Should-have (delays if not ready)
   - Nice-to-have (punts to next release)
3. Update CHANGELOG.md draft
4. Announce planned release date to team
5. Freeze new features (minor releases)

### 2. Preparation Phase

**Timeline**: 1 week before release

1. Create release branch: `git checkout -b release/vX.Y.Z develop`
2. Bump versions in all crates:
   ```bash
   # Option 1: Manual update
   ./scripts/bump-version.sh X.Y.Z

   # Option 2: Using cargo-workspaces
   cargo workspaces version --no-git-commit X.Y.Z
   ```
3. Update CHANGELOG.md with final changes
4. Update documentation for new features
5. Run full test suite: `cargo test --workspace`
6. Run benchmarks and compare: `cargo bench --workspace`
7. Security audit: `cargo audit`
8. Build all platform targets locally
9. Plugin compatibility testing
10. Documentation review

### 3. Release Candidate Phase (if needed)

**Timeline**: 1 week for testing

1. Tag RC: `git tag -a vX.Y.Z-rc.1 -m "Release candidate 1"`
2. Push tag: `git push origin vX.Y.Z-rc.1`
3. Build and distribute RC binaries
4. Announce RC to community
5. Collect feedback (1 week)
6. If issues found:
   - Fix in release branch
   - Create new RC: `vX.Y.Z-rc.2`
   - Repeat testing
7. If no issues: proceed to release

### 4. Release Execution

**Timeline**: Release day

1. Merge release branch to main:
   ```bash
   git checkout main
   git merge --no-ff release/vX.Y.Z
   ```
2. Create annotated tag:
   ```bash
   git tag -a vX.Y.Z -m "Release vX.Y.Z

   Summary of changes:
   - Feature A
   - Feature B
   - Bug fixes

   See CHANGELOG.md for full details."
   ```
3. Push to GitHub:
   ```bash
   git push origin main
   git push origin vX.Y.Z
   ```
4. Monitor GitHub Actions workflow
5. Review and publish GitHub Release
6. Verify crates.io publication
7. Update package managers (Homebrew, AUR)
8. Merge release branch back to develop:
   ```bash
   git checkout develop
   git merge --no-ff release/vX.Y.Z
   git push origin develop
   ```
9. Delete release branch:
   ```bash
   git branch -d release/vX.Y.Z
   git push origin --delete release/vX.Y.Z
   ```

### 5. Post-Release Phase

**Timeline**: 1-2 weeks after release

1. Announce release (see Communication Plan)
2. Monitor for critical issues
3. Update project status
4. Create next milestone
5. Post-release retrospective
6. Document lessons learned

## Rollback Procedure

If critical issues are discovered after release, follow this rollback procedure:

### Severity Assessment

**Critical** (immediate rollback):
- Data loss or corruption
- Security vulnerability (CVE)
- Complete functionality breakdown
- Shared memory corruption

**Major** (hotfix within 24h):
- Feature completely broken
- Performance regression >50%
- Plugin system failure

**Minor** (hotfix within 1 week):
- UI glitches
- Non-critical feature issues
- Performance regression <50%

### Rollback Steps

#### 1. Immediate Mitigation (Critical Issues)

```bash
# 1. Create incident issue
gh issue create --title "CRITICAL: [Brief description]" \
  --label "critical,bug" --assignee "@me"

# 2. Yank crates from crates.io (if published)
cargo yank --vers X.Y.Z scarab-client
cargo yank --vers X.Y.Z scarab-daemon

# 3. Mark GitHub Release as "This release has known issues"
# (Manual step in GitHub UI)

# 4. Post urgent notice in README.md
echo "⚠️ WARNING: v$VERSION has critical issues. Do not use. Fix incoming." >> README.md
git commit -am "docs: Add critical warning for v$VERSION"
git push
```

#### 2. Hotfix Development

```bash
# 1. Create hotfix branch from previous stable tag
git checkout -b hotfix/vX.Y.Z vX.Y.(Z-1)

# 2. Cherry-pick fix or develop new fix
git cherry-pick <commit-hash>
# OR
# ... make changes ...

# 3. Test thoroughly
cargo test --workspace
cargo build --release --workspace

# 4. Bump patch version
./scripts/bump-version.sh X.Y.(Z+1)

# 5. Update CHANGELOG.md
echo "## vX.Y.(Z+1) - $(date +%Y-%m-%d)

### Fixed
- Critical issue: [description]
" >> CHANGELOG.md

# 6. Commit and tag
git commit -am "fix: Critical issue [description]"
git tag -a vX.Y.(Z+1) -m "Hotfix vX.Y.(Z+1)"
```

#### 3. Emergency Release

```bash
# 1. Push hotfix
git push origin hotfix/vX.Y.(Z+1)
git push origin vX.Y.(Z+1)

# 2. Monitor GitHub Actions
# 3. Publish release immediately

# 4. Un-yank previous version if needed
# (Manual decision based on severity)

# 5. Merge back to main and develop
git checkout main
git merge --no-ff hotfix/vX.Y.(Z+1)
git push origin main

git checkout develop
git merge --no-ff hotfix/vX.Y.(Z+1)
git push origin develop

# 6. Clean up
git branch -d hotfix/vX.Y.(Z+1)
```

#### 4. Communication

```bash
# Post announcement
gh issue comment <incident-issue> --body "
## Hotfix Released: vX.Y.(Z+1)

The critical issue has been fixed in vX.Y.(Z+1).

**What was wrong**: [Brief description]
**Impact**: [Who was affected]
**Fix**: [What we did]

Please upgrade immediately:
\`\`\`bash
# Homebrew
brew upgrade scarab

# Cargo
cargo install --force scarab-client

# Manual
# Download from https://github.com/raibid-labs/scarab/releases/tag/vX.Y.(Z+1)
\`\`\`

We apologize for the inconvenience.
"
```

### Prevention

To minimize the need for rollbacks:

1. **Thorough testing**: Never skip RC phase for major releases
2. **Gradual rollout**: Consider phased releases (10% → 50% → 100%)
3. **Monitoring**: Set up error tracking and monitoring
4. **Fast feedback**: Encourage early reports from community
5. **Automated testing**: Expand CI/CD coverage
6. **Beta program**: Maintain active beta tester group

## Communication Plan

Effective communication is critical for successful releases.

### Internal Communication

**Team Notification**:
- Announce planned release 2 weeks in advance
- Daily standup updates during release week
- Immediate notification of any issues

**Channels**:
- GitHub Discussions (internal)
- Team chat (Discord/Slack)
- Email for critical issues

### External Communication

**Pre-Release**:
- GitHub Discussions: Announce upcoming release
- Social media: Teaser for major features
- Blog post: For major releases (1.0, 2.0, etc.)

**Release Day**:
1. GitHub Release notes (detailed)
2. GitHub Discussions announcement
3. Social media posts:
   - Twitter/X
   - Reddit (r/rust, r/commandline, r/terminal_porn)
   - Mastodon
4. Rust community Discord/Zulip
5. Update project website

**Post-Release**:
- Thank contributors
- Gather feedback
- Address issues promptly
- Weekly summary of adoption/issues

### Communication Templates

See the following template files:
- `docs/templates/release-announcement-template.md` - For GitHub/social media
- `docs/templates/release-notes-template.md` - For GitHub Release page
- `docs/templates/hotfix-announcement-template.md` - For urgent fixes

### Crisis Communication

For critical issues:
1. **Acknowledge immediately** (within 1 hour)
2. **Provide timeline** for fix (within 4 hours)
3. **Regular updates** every 2-4 hours
4. **Root cause analysis** after resolution
5. **Prevention plan** to avoid recurrence

## Roles and Responsibilities

### Release Manager

**Responsibilities**:
- Owns the release process end-to-end
- Coordinates with contributors
- Makes go/no-go decisions
- Handles communication
- Updates documentation

**Tasks**:
- Create release branch
- Coordinate testing
- Review CHANGELOG
- Execute release steps
- Monitor post-release

**Authority**:
- Can delay release if critical issues found
- Can approve hotfix releases
- Final say on version numbering

### Quality Assurance

**Responsibilities**:
- Verify all tests pass
- Manual testing on target platforms
- Performance testing
- Documentation review

**Tasks**:
- Run test suite
- Test on multiple platforms
- Verify package installations
- Check plugin compatibility

### Package Maintainers

**Homebrew Maintainer**:
- Update formula
- Test installation
- Respond to formula issues

**AUR Maintainer**:
- Update PKGBUILD
- Test build process
- Monitor AUR comments

### Communication Lead

**Responsibilities**:
- Draft announcements
- Post to social media
- Monitor community feedback
- Coordinate with release manager

## Automation and Tools

### Required Tools

```bash
# Install release tools
cargo install cargo-workspaces  # Version management
cargo install cargo-audit       # Security auditing
cargo install cargo-release     # Release automation (optional)
gh                              # GitHub CLI
```

### Automation Scripts

The `scripts/` directory contains automation:

- `scripts/bump-version.sh` - Update all crate versions
- `scripts/check-release.sh` - Pre-release validation
- `scripts/build-all-targets.sh` - Cross-platform builds
- `scripts/update-changelog.sh` - CHANGELOG helpers

### GitHub Actions

The release process is partially automated via `.github/workflows/release.yml`:

- Triggered by pushing a version tag
- Builds all platform binaries
- Creates GitHub Release draft
- Publishes to crates.io
- Updates package managers

Manual intervention required:
- Publishing the GitHub Release
- Writing release notes
- Announcement posting

## Troubleshooting

### Common Issues

**Problem**: GitHub Actions build fails for specific platform

**Solution**:
1. Check build logs for specific error
2. Common causes:
   - Missing system dependencies
   - Cross-compilation toolchain issues
   - Platform-specific code bugs
3. Fix locally and re-tag or create patch release

---

**Problem**: crates.io publish fails with "version already exists"

**Solution**:
- Versions cannot be republished on crates.io
- Must bump version and create new tag
- Use `cargo yank` if critical issue

---

**Problem**: Homebrew formula update fails

**Solution**:
1. Check SHA256 checksums match
2. Verify release assets are available
3. Test formula locally: `brew install --build-from-source ./packaging/homebrew/scarab.rb`
4. Check Homebrew CI: https://formulae.brew.sh/

---

**Problem**: AUR package out of sync

**Solution**:
1. Clone AUR repo: `git clone ssh://aur@aur.archlinux.org/scarab-terminal.git`
2. Update PKGBUILD and .SRCINFO
3. Test: `makepkg -si`
4. Push update

---

**Problem**: Release tag pushed but GitHub Actions didn't trigger

**Solution**:
1. Check tag format matches `v*` pattern
2. Verify workflows file exists and is valid
3. Manually trigger: Go to Actions → Release → Run workflow

---

**Problem**: Need to cancel/delete a release

**Solution**:
```bash
# Delete tag locally and remotely
git tag -d vX.Y.Z
git push origin :refs/tags/vX.Y.Z

# Delete GitHub Release (via UI or CLI)
gh release delete vX.Y.Z

# Yank from crates.io if published
cargo yank --vers X.Y.Z scarab-client
cargo yank --vers X.Y.Z scarab-daemon
```

## References

- [Semantic Versioning 2.0.0](https://semver.org/)
- [Cargo Publishing Guide](https://doc.rust-lang.org/cargo/reference/publishing.html)
- [GitHub Actions Documentation](https://docs.github.com/en/actions)
- [Homebrew Formula Cookbook](https://docs.brew.sh/Formula-Cookbook)
- [Arch User Repository Guidelines](https://wiki.archlinux.org/title/AUR_submission_guidelines)

## Changelog

- **2025-11-24**: Initial release process documentation
