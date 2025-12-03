# Phase 1 Agent Work Briefs

These are detailed instructions for the 4 parallel agents launching in Phase 1.

---

## Agent 1: Doc-Index (Issue #61)

**Mission:** Create central documentation index and consolidate navigation docs

### Files to Create:
- `/home/beengud/raibid-labs/scarab/docs/README.md`

### Files to Review:
- `/home/beengud/raibid-labs/scarab/README.md` (main project README)
- `/home/beengud/raibid-labs/scarab/TESTING.md`
- `/home/beengud/raibid-labs/scarab/ROADMAP.md`
- `/home/beengud/raibid-labs/scarab/IMPLEMENTATION_SUMMARY.md`
- `/home/beengud/raibid-labs/scarab/docs/navigation/` (entire directory)
- `/home/beengud/raibid-labs/scarab/docs/PLUGIN_DOCK_IMPLEMENTATION.md`

### Task Details:

1. **Create Central Index** at `docs/README.md` with sections:
   - Getting Started (link to interactive tutorial)
   - Architecture & Design
   - Developer Guides
   - API Documentation
   - Testing
   - Contributing

2. **Consolidate Navigation Docs:**
   - Review all docs in `docs/navigation/`
   - Identify duplicate or conflicting information
   - Create a single navigation documentation structure
   - Ensure navigation docs are linked from central index

3. **Link Existing Documentation:**
   - Root-level docs (TESTING.md, ROADMAP.md, etc.)
   - Examples directories
   - Plugin documentation
   - Configuration guides

4. **Add Quick Links Section:**
   - One-line commands for common tasks
   - Links to issue tracker, discussions
   - CI/CD status badges

### Success Criteria:
- Single source of truth for documentation navigation
- No broken links
- Clear hierarchy (Getting Started → Developer → Advanced)
- Navigation docs are consolidated and non-redundant

---

## Agent 2: Testing-Guide (Issue #62)

**Mission:** Enhance TESTING.md with one-line command reference

### Files to Modify:
- `/home/beengud/raibid-labs/scarab/TESTING.md` (already exists, enhance it)

### Current State:
TESTING.md exists and is comprehensive but needs:
- Quickstart section at the very top
- One-line command reference table
- Better organization for quick reference

### Task Details:

1. **Add Quickstart Section at Top:**
```markdown
## Quickstart

Run all tests:
```bash
cargo test --workspace
```

Run specific test suites:
```bash
cargo test -p scarab-client --lib          # Unit tests
cargo test -p scarab-client --test e2e     # E2E tests
just nav-smoke                              # Navigation tests
```
```

2. **Create Command Reference Table:**
```markdown
## Command Reference

| What to Test | Command | Time |
|--------------|---------|------|
| All tests | `cargo test --workspace` | 2m |
| Unit tests only | `cargo test --workspace --lib` | 30s |
| Navigation tests | `just nav-smoke` | 45s |
| E2E tests | `cargo test -p scarab-client --test e2e` | 1m |
| Golden tests | `cargo test -p scarab-client --test golden_tests` | 20s |
| Headless harness | `cargo test -p scarab-client --test headless_harness` | 30s |
| Full CI suite | `just ci` | 3m |
```

3. **Add "When to Use Which Test" Section:**
- Unit tests: Testing individual functions/modules
- Integration tests: Testing component interactions
- E2E tests: Testing full daemon-client workflows
- Golden tests: Regression testing for rendering
- Headless tests: UI testing without GPU

4. **Add Troubleshooting Quickstart:**
- Common issues with one-line fixes
- Debug commands
- Cleanup commands

### Success Criteria:
- New developers can find test commands in <30 seconds
- Clear command reference table
- Maintains existing comprehensive documentation
- No information loss from current version

---

## Agent 3: Doc-Portal (Issue #71)

**Mission:** Set up mdBook documentation portal with rustdoc integration

### Files to Create:
- `/home/beengud/raibid-labs/scarab/docs/book/book.toml`
- `/home/beengud/raibid-labs/scarab/docs/book/src/SUMMARY.md`
- `/home/beengud/raibid-labs/scarab/docs/book/src/*.md` (chapters)

### Files to Modify:
- `/home/beengud/raibid-labs/scarab/justfile` (add docs-build, docs-serve targets - already exist, verify they work)

### Task Details:

1. **Initialize mdBook Structure:**
```toml
# book.toml
[book]
title = "Scarab Terminal Emulator"
authors = ["raibid-labs"]
language = "en"
multilingual = false
src = "src"

[output.html]
default-theme = "ayu"
preferred-dark-theme = "ayu"
git-repository-url = "https://github.com/raibid-labs/scarab"
edit-url-template = "https://github.com/raibid-labs/scarab/edit/main/docs/book/{path}"

[output.html.fold]
enable = true
level = 0
```

2. **Create SUMMARY.md Structure:**
```markdown
# Summary

[Introduction](./intro.md)

# Getting Started
- [Installation](./getting-started/installation.md)
- [Quick Start](./getting-started/quick-start.md)
- [Interactive Tutorial](./getting-started/tutorial.md)

# User Guide
- [Configuration](./user-guide/configuration.md)
- [Keyboard Shortcuts](./user-guide/keybindings.md)
- [Plugins](./user-guide/plugins.md)
- [Themes](./user-guide/themes.md)

# Developer Guide
- [Architecture](./dev-guide/architecture.md)
- [Building from Source](./dev-guide/building.md)
- [Testing](./dev-guide/testing.md)
- [Navigation System](./dev-guide/navigation.md)
- [Plugin Development](./dev-guide/plugin-dev.md)

# API Reference
- [Rust API Docs](./api/rustdoc.md)
- [Plugin API](./api/plugin-api.md)
- [Configuration API](./api/config-api.md)

# Roadmap
- [Project Roadmap](./roadmap.md)
- [Release History](./releases.md)
```

3. **Link Existing Documentation:**
- Copy relevant content from root docs into book structure
- Create symlinks or includes for content that changes frequently
- Add rustdoc link page that opens API docs

4. **Add rustdoc Integration:**
- Create API reference page that links to `cargo doc` output
- Add build script to generate rustdoc before mdBook build
- Consider hosting both together

5. **Update Justfile:**
Verify these targets work (they already exist):
```justfile
docs-build:
    cd docs/book && mdbook build

docs-serve:
    cd docs/book && mdbook serve --open
```

### Dependencies:
- mdBook must be installed: `cargo install mdbook`
- Check if installed, provide installation instructions if not

### Success Criteria:
- `just docs-build` successfully builds documentation
- `just docs-serve` opens local preview
- Navigation between sections works
- Rustdoc is linked and accessible
- All internal links work

---

## Agent 4: Justfile-Targets (Issue #65)

**Mission:** Add justfile targets for testing workflows

### Files to Modify:
- `/home/beengud/raibid-labs/scarab/justfile` (already has 666 lines, add new targets)

### Current Justfile Status:
- Already has `test`, `test-verbose`, `nav-smoke`
- Already has plugin development targets
- Needs additional test-specific targets

### Task Details:

1. **Add Test Suite Targets:**
```justfile
# Run golden tests (snapshot validation)
test-golden:
    cargo test -p scarab-client --test golden_tests

# Run ratatui-testlib tests
test-ratatui:
    cargo test -p scarab-client --lib ratatui
    cargo test -p scarab-client --test ratatui_tests

# Run headless harness tests
test-headless:
    cargo test -p scarab-client --test headless_harness
    cargo test -p scarab-client --test headless_poc
    cargo test -p scarab-client --test harness_examples

# Run all visual/rendering tests
test-visual: test-golden test-ratatui test-headless
    @echo "All visual tests complete"

# Run navigation tests (alias for nav-smoke)
test-nav: nav-smoke

# Run E2E tests (excluding stress tests)
test-e2e:
    cargo test -p scarab-client --test e2e

# Run only unit tests
test-unit:
    cargo test --workspace --lib

# Run only integration tests
test-integration:
    cargo test --workspace --test '*'

# Run all tests with timing
test-timed:
    cargo test --workspace -- --show-output --test-threads=1 --nocapture
```

2. **Add Test Filtering Targets:**
```justfile
# Run tests matching a pattern
test-match pattern:
    cargo test --workspace {{pattern}}

# Run tests for specific crate
test-crate crate:
    cargo test -p {{crate}}

# Run single test by name
test-one test_name:
    cargo test --workspace {{test_name}} -- --exact --nocapture
```

3. **Add Test Cleanup Targets:**
```justfile
# Clean test artifacts
test-clean:
    rm -rf target/debug/deps/test_*
    rm -rf /tmp/scarab-test-*

# Reset test environment
test-reset: kill clean-shm test-clean
    @echo "Test environment reset"
```

4. **Add Test Watch Targets:**
```justfile
# Watch and run tests on file changes
test-watch:
    cargo watch -x 'test --workspace'

# Watch specific test suite
test-watch-suite suite:
    cargo watch -x 'test -p scarab-client --test {{suite}}'
```

5. **Add Documentation:**
Add comments explaining each target's purpose and when to use it.

### Integration with Existing Targets:
- Preserve all existing targets
- Maintain consistency with existing naming conventions
- Update `default` target list if needed

### Success Criteria:
- All new targets execute successfully
- `just --list` shows new targets with descriptions
- Targets follow existing justfile patterns
- No conflicts with existing targets
- Commands are correct and tested

---

## Launch Checklist for Phase 1

- [ ] Agent 1 (Doc-Index) - Create `/home/beengud/raibid-labs/scarab/docs/README.md`
- [ ] Agent 2 (Testing-Guide) - Enhance `/home/beengud/raibid-labs/scarab/TESTING.md`
- [ ] Agent 3 (Doc-Portal) - Set up `/home/beengud/raibid-labs/scarab/docs/book/`
- [ ] Agent 4 (Justfile-Targets) - Add targets to `/home/beengud/raibid-labs/scarab/justfile`

All agents can work in parallel - no dependencies between them.

---

**Created:** 2025-12-03
**Phase:** 1 - Foundation
