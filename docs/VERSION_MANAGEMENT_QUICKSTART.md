# Version Management Quick Start

This guide provides quick commands for managing versions in the Scarab workspace.

## Current Version

```bash
cat VERSION
# 0.1.0-alpha.1
```

## Checking Versions

```bash
# Check all workspace crate versions
cargo metadata --format-version=1 --no-deps | jq -r '.packages[] | select(.name | startswith("scarab-")) | "\(.name): \(.version)"' | sort

# Expected output (all should match):
# scarab-client: 0.1.0-alpha.1
# scarab-config: 0.1.0-alpha.1
# scarab-daemon: 0.1.0-alpha.1
# scarab-nav: 0.1.0-alpha.1
# scarab-palette: 0.1.0-alpha.1
# scarab-platform: 0.1.0-alpha.1
# scarab-plugin-api: 0.1.0-alpha.1
# scarab-plugin-compiler: 0.1.0-alpha.1
# scarab-protocol: 0.1.0-alpha.1
# scarab-session: 0.1.0-alpha.1
```

## Bumping Versions

### Using cargo-release (Recommended)

```bash
# Install cargo-release
cargo install cargo-release

# Always do a dry run first
cargo release --workspace --dry-run

# Patch release (0.1.0 -> 0.1.1)
cargo release patch --workspace --execute

# Minor release (0.1.0 -> 0.2.0)
cargo release minor --workspace --execute

# Major release (0.1.0 -> 1.0.0)
cargo release major --workspace --execute

# Pre-release (specify exact version)
cargo release --version 0.2.0-alpha.1 --workspace --execute
cargo release --version 0.2.0-beta.1 --workspace --execute
cargo release --version 0.2.0-rc.1 --workspace --execute
```

### Manual Version Update

If cargo-release is not available:

1. Edit `Cargo.toml` in workspace root:
   ```toml
   [workspace.package]
   version = "0.2.0-alpha.1"
   ```

2. Update VERSION file:
   ```bash
   echo "0.2.0-alpha.1" > VERSION
   ```

3. Verify changes:
   ```bash
   cargo check --workspace
   ```

4. Commit and tag:
   ```bash
   git add Cargo.toml VERSION
   git commit -m "chore: Bump version to 0.2.0-alpha.1"
   git tag -a v0.2.0-alpha.1 -m "Release v0.2.0-alpha.1"
   git push origin main --tags
   ```

## Version Consistency Checks

```bash
# Ensure all crates use workspace version
grep -r "^version.workspace = true" crates/*/Cargo.toml | wc -l
# Should output: 10 (number of workspace crates)

# Check for any hardcoded versions in workspace crates
grep -r "^version = " crates/*/Cargo.toml
# Should output nothing (all should use workspace = true)
```

## Testing Before Release

```bash
# Run all tests
cargo test --workspace --all-features

# Run benchmarks
cargo bench --workspace

# Build release binaries
cargo build --workspace --release

# Check for warnings
cargo clippy --workspace --all-features -- -D warnings

# Format check
cargo fmt --all -- --check
```

## Release Workflow

1. **Ensure clean state**:
   ```bash
   git status
   # Should show no uncommitted changes
   ```

2. **Run pre-release checks**:
   ```bash
   cargo test --workspace --all-features
   cargo clippy --workspace --all-features
   cargo fmt --all --check
   ```

3. **Update CHANGELOG.md**:
   - Document all changes since last release
   - Follow [Keep a Changelog](https://keepachangelog.com/) format

4. **Bump version**:
   ```bash
   cargo release --version 0.2.0-alpha.1 --workspace --dry-run
   cargo release --version 0.2.0-alpha.1 --workspace --execute
   ```

5. **Verify release**:
   ```bash
   git log -1
   git tag -l "v*" | tail -1
   ```

## Troubleshooting

### Version Mismatch

If you see version mismatches:

```bash
# Reset to workspace version
cd crates/scarab-client
cargo update --workspace
```

### cargo-release Issues

If cargo-release fails:

1. Check `.cargo/release.toml` configuration
2. Ensure you're on the allowed branch (main)
3. Verify no uncommitted changes
4. Check that all tests pass

### Git Tag Issues

If git tags are out of sync:

```bash
# List all tags
git tag -l

# Delete a local tag
git tag -d v0.1.0-alpha.1

# Delete a remote tag
git push origin :refs/tags/v0.1.0-alpha.1

# Create new tag
git tag -a v0.1.0-alpha.1 -m "Release v0.1.0-alpha.1"
git push origin v0.1.0-alpha.1
```

## Files Involved

- `Cargo.toml` - Workspace root with [workspace.package]
- `VERSION` - Current version file
- `.cargo/release.toml` - cargo-release configuration
- `CHANGELOG.md` - Version history and changes
- `docs/VERSIONING.md` - Complete versioning strategy
- `crates/*/Cargo.toml` - Individual crate manifests (inherit version)

## References

- [Semantic Versioning](https://semver.org/)
- [cargo-release](https://github.com/crate-ci/cargo-release)
- [Cargo Workspace](https://doc.rust-lang.org/cargo/reference/workspaces.html)
- [docs/VERSIONING.md](./VERSIONING.md) - Complete versioning documentation
