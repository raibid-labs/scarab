# Versioning Strategy

This document describes the versioning strategy for Scarab terminal emulator.

## Semantic Versioning

Scarab follows [Semantic Versioning 2.0.0](https://semver.org/):

```
MAJOR.MINOR.PATCH[-PRERELEASE][+BUILD]
```

- **MAJOR**: Incompatible API changes
- **MINOR**: Backwards-compatible functionality additions
- **PATCH**: Backwards-compatible bug fixes
- **PRERELEASE**: Alpha, beta, or release candidate versions
- **BUILD**: Build metadata (not used for version precedence)

### Examples

- `0.1.0-alpha.1` - First alpha release of v0.1.0
- `0.1.0-beta.1` - First beta release of v0.1.0
- `0.1.0-rc.1` - First release candidate of v0.1.0
- `0.1.0` - First stable release
- `0.1.1` - Patch release with bug fixes
- `0.2.0` - Minor release with new features
- `1.0.0` - Major release (stable API)

## Pre-1.0 Development

During pre-1.0 development (current phase):

- **MINOR** versions may include breaking changes
- **PATCH** versions should be backwards-compatible
- Breaking changes should be clearly documented in CHANGELOG.md
- Pre-release versions (alpha, beta, rc) are used for testing

## Pre-release Conventions

### Alpha Releases

- **Purpose**: Early testing, incomplete features, unstable API
- **Frequency**: As needed during active development
- **Format**: `0.MINOR.0-alpha.N`
- **Audience**: Core developers and early testers
- **Stability**: Breaking changes expected

### Beta Releases

- **Purpose**: Feature-complete for the release, API stabilizing
- **Frequency**: After alpha testing completes
- **Format**: `0.MINOR.0-beta.N`
- **Audience**: Wider testing community
- **Stability**: Minimal breaking changes, focus on bug fixes

### Release Candidates

- **Purpose**: Final testing before stable release
- **Frequency**: When no major bugs remain
- **Format**: `0.MINOR.0-rc.N`
- **Audience**: All users for production testing
- **Stability**: No new features, only critical bug fixes

## Version Bumping

### Automated Version Bumping

Use `cargo-release` to manage versions:

```bash
# Install cargo-release
cargo install cargo-release

# Preview what will happen (dry run)
cargo release --workspace --dry-run

# Create a patch release (0.1.0 -> 0.1.1)
cargo release patch --workspace --execute

# Create a minor release (0.1.0 -> 0.2.0)
cargo release minor --workspace --execute

# Create a major release (0.1.0 -> 1.0.0)
cargo release major --workspace --execute

# Create a pre-release
cargo release --version 0.2.0-alpha.1 --workspace --execute
```

### Manual Version Bumping

If you need to manually update versions:

1. **Update workspace version** in root `Cargo.toml`:
   ```toml
   [workspace.package]
   version = "0.2.0-alpha.1"
   ```

2. **Update VERSION file** in repository root:
   ```bash
   echo "0.2.0-alpha.1" > VERSION
   ```

3. **Verify all crates inherit the version**:
   ```bash
   cargo metadata --format-version=1 | jq '.packages[] | select(.name | startswith("scarab-")) | {name, version}'
   ```

4. **Update CHANGELOG.md** with changes for this version

5. **Test the build**:
   ```bash
   cargo check --workspace
   cargo test --workspace
   cargo build --workspace --release
   ```

6. **Commit and tag**:
   ```bash
   git add Cargo.toml VERSION CHANGELOG.md
   git commit -m "chore: Bump version to 0.2.0-alpha.1"
   git tag -a v0.2.0-alpha.1 -m "Release v0.2.0-alpha.1"
   git push origin main --tags
   ```

## When to Release

### Alpha Releases

Create an alpha release when:
- New core functionality is implemented but not fully tested
- Breaking API changes are being introduced
- You want feedback from core developers
- Major architectural changes are in progress

### Beta Releases

Create a beta release when:
- All planned features for the minor version are implemented
- API is mostly stable (only minor changes expected)
- You want broader community testing
- Integration testing is complete

### Release Candidates

Create a release candidate when:
- No known critical bugs remain
- Documentation is complete
- All tests are passing
- Ready for production testing

### Stable Releases

Create a stable release when:
- At least one RC has been tested in production-like environments
- No critical issues reported for at least 1 week
- All documentation is updated
- CHANGELOG.md is complete

## Release Checklist

Before any release:

1. [ ] All tests passing (`cargo test --workspace`)
2. [ ] All benchmarks run successfully (`cargo bench --workspace`)
3. [ ] Documentation updated
4. [ ] CHANGELOG.md updated with all changes
5. [ ] VERSION file updated
6. [ ] No uncommitted changes
7. [ ] Code review completed (for stable releases)

For stable releases only:

8. [ ] Release notes written
9. [ ] Migration guide prepared (if breaking changes)
10. [ ] Binary artifacts built and tested for all platforms
11. [ ] Announcement prepared

## Workspace Version Synchronization

All crates in the Scarab workspace share the same version number. This ensures:

- Consistent versioning across all components
- Simplified dependency management
- Clear compatibility guarantees
- Easier release process

### Internal Dependencies

When referencing other Scarab crates, always use path dependencies without version constraints:

```toml
[dependencies]
scarab-protocol = { path = "../scarab-protocol" }
scarab-config = { path = "../scarab-config" }
```

The workspace version inheritance ensures all crates stay synchronized.

### Version Inheritance

All crates inherit metadata from the workspace:

```toml
[package]
name = "scarab-client"
version.workspace = true
edition.workspace = true
authors.workspace = true
license.workspace = true
repository.workspace = true
homepage.workspace = true
```

This is defined in the root `Cargo.toml`:

```toml
[workspace.package]
version = "0.1.0-alpha.1"
authors = ["Scarab Team <team@raibid-labs.com>"]
license = "MIT OR Apache-2.0"
repository = "https://github.com/raibid-labs/scarab"
homepage = "https://github.com/raibid-labs/scarab"
edition = "2021"
rust-version = "1.75"
```

## CI/CD Integration

Automated releases are triggered by:

1. **Git tags**: Pushing a tag like `v0.1.0-alpha.1` triggers release workflow
2. **cargo-release**: Automates version bumping, changelog, tagging, and publishing

See `.cargo/release.toml` for cargo-release configuration.

## Version History

| Version | Date | Type | Description |
|---------|------|------|-------------|
| 0.1.0-alpha.1 | TBD | Alpha | Initial alpha release with core IPC and plugin system |

## References

- [Semantic Versioning 2.0.0](https://semver.org/)
- [Cargo Book - SemVer Compatibility](https://doc.rust-lang.org/cargo/reference/semver.html)
- [cargo-release Documentation](https://github.com/crate-ci/cargo-release)
- [Keep a Changelog](https://keepachangelog.com/)
