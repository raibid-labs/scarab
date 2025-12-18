# Release Checklist

This document provides a step-by-step checklist for releasing Scarab Terminal. Use this checklist every time you prepare a release to ensure consistency and completeness.

## Pre-Release Phase

### 1. Code Quality & Testing

- [ ] Run full test suite: `cargo test --workspace`
- [ ] Run clippy with strict mode: `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- [ ] Run formatter check: `cargo fmt --all -- --check`
- [ ] Run benchmarks: `cargo bench --workspace`
- [ ] Test all platform builds locally:
  - [ ] Linux (x86_64 glibc)
  - [ ] Linux (x86_64 musl)
  - [ ] macOS (Intel) - if available
  - [ ] macOS (Apple Silicon) - if available
- [ ] Run integration tests: `cargo test --workspace --test '*'`
- [ ] Run E2E tests in `crates/scarab-client/tests/e2e/`
- [ ] Check for memory leaks with Valgrind (Linux only)
- [ ] Verify PTY functionality with various shells (bash, zsh, fish)
- [ ] Test plugin system with example plugins
- [ ] Verify shared memory IPC under high load
- [ ] Test session persistence across daemon restarts

### 2. Documentation Updates

- [ ] Update CHANGELOG.md with all changes since last release
  - [ ] Added features
  - [ ] Changed behavior
  - [ ] Deprecated features
  - [ ] Removed features
  - [ ] Fixed bugs
  - [ ] Security fixes
  - [ ] Performance improvements
- [ ] Update version numbers in README.md if needed
- [ ] Review and update ROADMAP.md for completed items
- [ ] Update docs/ for any new features or changed behavior
- [ ] Verify all code examples in documentation work
- [ ] Update screenshots/demos if UI changed
- [ ] Check all links in documentation are valid

### 3. Version Management

- [ ] Ensure all workspace crates have synchronized versions
- [ ] Update version in root `Cargo.toml` workspace members
- [ ] Update version in individual crate `Cargo.toml` files:
  - [ ] `crates/scarab-daemon/Cargo.toml`
  - [ ] `crates/scarab-client/Cargo.toml`
  - [ ] `crates/scarab-protocol/Cargo.toml`
  - [ ] `crates/scarab-plugin-api/Cargo.toml`
  - [ ] `crates/scarab-plugin-compiler/Cargo.toml`
  - [ ] `crates/scarab-config/Cargo.toml`
  - [ ] `crates/scarab-platform/Cargo.toml`
  - [ ] `crates/scarab-nav/Cargo.toml`
  - [ ] `crates/scarab-session/Cargo.toml`
- [ ] Update inter-crate dependencies to match new version
- [ ] Run `cargo update` to update Cargo.lock
- [ ] Commit version bump: `git commit -am "chore: Bump version to vX.Y.Z"`

### 4. Security Review

- [ ] Run `cargo audit` to check for vulnerable dependencies
- [ ] Review any security-related issues in GitHub
- [ ] Check for exposed secrets or API keys in code
- [ ] Verify plugin sandboxing is working correctly
- [ ] Review shared memory permissions

### 5. Dependency Management

- [ ] Update dependencies if needed: `cargo update`
- [ ] Check for breaking changes in Fusabi crates
- [ ] Verify Bevy version compatibility
- [ ] Test with latest Rust stable: `rustup update stable`
- [ ] Document minimum supported Rust version (MSRV)

## Release Phase

### 6. Git Tagging

- [ ] Ensure working directory is clean: `git status`
- [ ] Ensure you're on the main branch: `git branch --show-current`
- [ ] Pull latest changes: `git pull origin main`
- [ ] Create annotated tag: `git tag -a vX.Y.Z -m "Release vX.Y.Z"`
- [ ] Verify tag was created: `git tag -l vX.Y.Z`
- [ ] Push tag to GitHub: `git push origin vX.Y.Z`

### 7. GitHub Release Workflow

- [ ] Verify GitHub Actions workflow triggered: Check https://github.com/raibid-labs/scarab/actions
- [ ] Monitor build progress for all platforms:
  - [ ] macOS (Apple Silicon)
  - [ ] macOS (Intel)
  - [ ] Linux (x86_64 glibc)
  - [ ] Linux (x86_64 musl)
  - [ ] Linux (ARM64)
  - [ ] Windows (x86_64 MSVC)
- [ ] Verify all build artifacts are uploaded
- [ ] Check that release draft is created
- [ ] Review auto-generated release notes
- [ ] Edit release notes using template from `docs/templates/release-notes-template.md`
- [ ] Mark as pre-release if alpha/beta/rc
- [ ] Publish the release (converts from draft)

### 8. Crates.io Publishing

Note: The GitHub Actions workflow handles this automatically. Manual steps only if workflow fails.

- [ ] Verify GitHub Actions published to crates.io
- [ ] If manual publish needed, follow dependency order:
  ```bash
  cargo publish -p scarab-protocol
  sleep 10
  cargo publish -p scarab-plugin-api
  sleep 10
  cargo publish -p scarab-config
  sleep 10
  cargo publish -p scarab-platform
  sleep 10
  cargo publish -p scarab-daemon
  sleep 10
  cargo publish -p scarab-client
  ```
- [ ] Verify crates are live on crates.io:
  - [ ] https://crates.io/crates/scarab-protocol
  - [ ] https://crates.io/crates/scarab-plugin-api
  - [ ] https://crates.io/crates/scarab-config
  - [ ] https://crates.io/crates/scarab-platform
  - [ ] https://crates.io/crates/scarab-daemon
  - [ ] https://crates.io/crates/scarab-client

### 9. Package Manager Updates

#### Homebrew

- [ ] Verify GitHub Actions updated Homebrew formula
- [ ] If manual update needed:
  ```bash
  cd packaging/homebrew
  # Update version and checksums in scarab.rb
  git commit -am "chore: Update Homebrew formula for vX.Y.Z"
  git push
  ```
- [ ] Test Homebrew installation:
  ```bash
  brew tap raibid-labs/scarab
  brew install scarab
  scarab-daemon --version
  ```

#### Arch User Repository (AUR)

- [ ] Clone AUR repository: `git clone ssh://aur@aur.archlinux.org/scarab-terminal.git`
- [ ] Update PKGBUILD:
  - [ ] Update `pkgver` to new version
  - [ ] Update `pkgrel` to 1
  - [ ] Update source URL
  - [ ] Update checksums: `updpkgsums`
- [ ] Test build: `makepkg -si`
- [ ] Update `.SRCINFO`: `makepkg --printsrcinfo > .SRCINFO`
- [ ] Commit and push:
  ```bash
  git add PKGBUILD .SRCINFO
  git commit -m "Update to vX.Y.Z"
  git push
  ```
- [ ] Verify AUR package page updates: https://aur.archlinux.org/packages/scarab-terminal

## Post-Release Phase

### 10. Verification

- [ ] Download release artifacts from GitHub
- [ ] Test each platform binary:
  - [ ] Extract archive
  - [ ] Run `scarab-daemon --version`
  - [ ] Run `scarab-client --version`
  - [ ] Basic smoke test (daemon + client connection)
- [ ] Verify Homebrew installation works (macOS)
- [ ] Verify AUR installation works (Arch Linux)
- [ ] Check crates.io documentation rendered correctly
- [ ] Test installation from crates.io: `cargo install scarab-client`

### 11. Communication

- [ ] Create announcement using template from `docs/templates/release-announcement-template.md`
- [ ] Post release announcement on GitHub Discussions
- [ ] Update project README.md "Current Status" section if needed
- [ ] Announce on social media channels (if applicable):
  - [ ] Twitter/X
  - [ ] Reddit (r/rust, r/commandline)
  - [ ] Discord/Slack communities
- [ ] Notify contributors and maintainers
- [ ] Update project website (if exists)

### 12. Monitoring

- [ ] Monitor GitHub issues for release-related bugs
- [ ] Check CI/CD pipelines remain green
- [ ] Watch for package manager reports (Homebrew, AUR)
- [ ] Monitor crates.io download stats
- [ ] Review initial user feedback
- [ ] Prepare hotfix plan if critical issues arise

### 13. Next Cycle Preparation

- [ ] Create milestone for next release
- [ ] Triage open issues for next release
- [ ] Update ROADMAP.md for next phase
- [ ] Plan major features for next release
- [ ] Create post-mortem document if needed

## Emergency Rollback

If critical issues are discovered immediately after release:

- [ ] Create hotfix branch from tag: `git checkout -b hotfix/vX.Y.Z vX.Y.Z`
- [ ] Apply fix and test thoroughly
- [ ] Follow process in `docs/RELEASE_PROCESS.md` "Rollback Procedure" section
- [ ] Consider yanking affected crates.io versions if severe security issue

## Notes

- For alpha/beta/rc releases, mark as "pre-release" on GitHub
- Always test on a clean system before releasing
- Keep CHANGELOG.md updated throughout development, not just at release time
- Use semantic versioning (MAJOR.MINOR.PATCH)
- Coordinate with team members before major releases
- Document any manual steps that were required for future automation

## Checklist Template

For quick copy-paste, save this minimal checklist:

```
## Release vX.Y.Z Checklist

Pre-Release:
- [ ] Tests pass
- [ ] CHANGELOG updated
- [ ] Versions bumped
- [ ] Dependencies audited

Release:
- [ ] Tag created and pushed
- [ ] GitHub Actions completed
- [ ] Release published
- [ ] Package managers updated

Post-Release:
- [ ] Downloads verified
- [ ] Announcement posted
- [ ] Monitoring active
```
