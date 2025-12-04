# Contributing to Scarab

Welcome to the Scarab contributor community! This guide will help you get started contributing to the project.

## Quick Links

- [Development Workflow](#development-workflow)
- [Getting Started](#getting-started)
- [Areas to Contribute](#areas-to-contribute)
- [Development Guidelines](#development-guidelines)
- [Pull Request Process](#pull-request-process)
- [Community Guidelines](#community-guidelines)

---

## Development Workflow

Scarab follows a **trunk-based development** model with short-lived feature branches. All changes go through Pull Requests that are squash-merged to keep history clean.

For complete details, see the [Contributing Guide](../../../../CONTRIBUTING.md) in the repository root.

### Quick Start

```bash
# Clone the repository
git clone https://github.com/raibid-labs/scarab.git
cd scarab

# Create a feature branch
git checkout -b feature/your-feature-name

# Make changes and commit
git add .
git commit -m "feat(scope): description"

# Push and create PR
git push -u origin feature/your-feature-name
gh pr create --title "feat(scope): description"
```

---

## Getting Started

### Prerequisites

**Required:**
- Rust 1.75+ with Cargo ([Install Rust](https://rustup.rs/))
- Git for version control
- Linux with X11 or Wayland (macOS/Windows support planned)

**Optional:**
- `just` command runner ([Install just](https://github.com/casey/just))
- `mdbook` for documentation ([Install mdbook](https://rust-lang.github.io/mdBook/))
- GitHub CLI `gh` for PR workflow ([Install gh](https://cli.github.com/))

### Build the Project

```bash
# Check code compiles
cargo check --workspace

# Build all crates
cargo build --workspace

# Build with optimizations
cargo build --release --workspace

# Run tests
cargo test --workspace
```

### Run the Application

```bash
# Terminal 1: Start daemon
cargo run --release -p scarab-daemon

# Terminal 2: Start client
cargo run --release -p scarab-client
```

---

## Areas to Contribute

### 1. Bug Fixes

Found a bug? We'd love your help fixing it!

**Process:**
1. Check if an issue already exists
2. Create a new issue with reproduction steps if needed
3. Create a branch: `fix/issue-number-description`
4. Fix the bug and add a test
5. Submit a PR referencing the issue

**Example:**
```bash
git checkout -b fix/123-cursor-positioning
# Make your fix
cargo test --workspace
git commit -m "fix(client): correct cursor position after resize

Closes #123"
```

### 2. New Features

Have an idea for a new feature?

**Process:**
1. Open a GitHub Discussion to discuss the feature
2. Wait for maintainer feedback and approval
3. Create a branch: `feature/feature-name`
4. Implement with tests and documentation
5. Submit a PR with detailed description

**Guidelines:**
- Start small - incremental PRs are easier to review
- Add tests for all new functionality
- Update relevant documentation
- Follow existing code patterns

### 3. Documentation

Documentation improvements are always welcome!

**Types of documentation:**
- User guides and tutorials
- API documentation (rustdoc comments)
- Examples and code snippets
- Architecture explanations
- Troubleshooting guides

See [Contributing to Documentation](../contributing-docs.md) for mdBook-specific guidelines.

### 4. Testing

Improve test coverage and quality:

- Add unit tests for edge cases
- Write integration tests for component interactions
- Create E2E tests for user workflows
- Add benchmarks for performance-critical code

See [Testing Guide](./testing.md) for comprehensive testing documentation.

### 5. Plugin Development

Write plugins to extend Scarab's functionality:

- Create example plugins demonstrating features
- Build utility plugins for common workflows
- Share plugins with the community

See [Plugin Development Guide](./plugins.md) for details.

### 6. Performance Optimization

Help make Scarab faster:

- Profile and identify bottlenecks
- Optimize hot code paths
- Reduce memory allocations
- Improve rendering performance

See [Performance Guide](../reference/performance.md) for profiling tools.

### 7. Platform Support

Expand platform compatibility:

- Test on different Linux distributions
- Contribute macOS support (planned Phase 7)
- Contribute Windows support (planned Phase 7)
- Improve X11/Wayland compatibility

---

## Development Guidelines

### Code Style

Scarab follows standard Rust conventions:

```bash
# Format code
cargo fmt --all

# Check for lints
cargo clippy --workspace -- -D warnings

# Run both (plus tests)
just ci
```

See [Code Style Guide](./code-style.md) for detailed conventions.

### Commit Messages

Use [Conventional Commits](https://www.conventionalcommits.org/):

```
<type>(<scope>): <subject>

[optional body]

[optional footer]
```

**Types:**
- `feat` - New feature
- `fix` - Bug fix
- `docs` - Documentation changes
- `refactor` - Code refactoring
- `test` - Test additions/fixes
- `perf` - Performance improvements
- `chore` - Maintenance tasks

**Examples:**
```bash
feat(client): add status bar rendering API
fix(daemon): prevent memory leak in image cache
docs: update plugin development guide
refactor(ipc): simplify shared memory layout
```

See [CONTRIBUTING.md](../../../../CONTRIBUTING.md#commit-messages) for complete details.

### Testing Requirements

All code changes should include tests:

- **New features**: Add unit tests and integration tests
- **Bug fixes**: Add regression test that would have caught the bug
- **Refactoring**: Ensure existing tests still pass

```bash
# Run all tests
cargo test --workspace

# Run specific test suite
cargo test -p scarab-client navigation

# Run with output visible
cargo test --workspace -- --nocapture
```

### Documentation Requirements

Update documentation for:

- Public APIs (rustdoc comments)
- New features (user guide)
- Configuration changes (config schema)
- Breaking changes (CHANGELOG.md and migration guide)

---

## Pull Request Process

### 1. Before Creating a PR

Checklist:

- [ ] Code compiles without warnings
- [ ] All tests pass (`cargo test --workspace`)
- [ ] Code is formatted (`cargo fmt --all`)
- [ ] No clippy warnings (`cargo clippy --workspace`)
- [ ] Documentation is updated
- [ ] CHANGELOG.md is updated (for user-facing changes)
- [ ] Branch is up to date with main

### 2. Creating the PR

Use descriptive titles following conventional commit format:

**Good titles:**
- `feat(client): implement key table stack for modal editing`
- `fix(daemon): prevent memory leak in image cache`
- `docs: add WezTerm parity roadmap`

**Bad titles:**
- `Update files`
- `WIP changes`
- `Fix stuff`

### 3. PR Template

Fill out all sections of the PR template:

```markdown
## Summary
Brief description of changes

## Related Issues
Closes #123, Fixes #456

## Changes
- Added X
- Modified Y
- Removed Z

## Testing
- [ ] Unit tests added/updated
- [ ] Integration tests added/updated
- [ ] Manual testing performed

## Checklist
- [ ] Code follows project style guidelines
- [ ] Self-review completed
- [ ] Documentation updated
- [ ] CHANGELOG.md updated
```

### 4. Review Process

After submitting:

1. **Automated checks** run (CI builds and tests)
2. **Maintainer review** provides feedback
3. **Address feedback** and push updates
4. **Approval** from maintainer
5. **Squash merge** to main

**Tips:**
- Respond to feedback promptly
- Ask questions if feedback is unclear
- Keep PRs focused and small (< 400 lines ideal)
- Don't take feedback personally

---

## Community Guidelines

### Code of Conduct

Be respectful, constructive, and professional:

- **Respectful**: Treat others with kindness and respect
- **Constructive**: Provide helpful feedback and suggestions
- **Inclusive**: Welcome contributors of all backgrounds
- **Collaborative**: Work together toward common goals

### Communication Channels

- **GitHub Issues**: Bug reports and feature requests
- **GitHub Discussions**: Questions, ideas, and general discussion
- **Pull Requests**: Code review and technical discussion

### Getting Help

Stuck or have questions?

1. Check existing documentation:
   - [README.md](../../../../README.md)
   - [CLAUDE.md](../../../../CLAUDE.md)
   - [Developer Guide](./architecture.md)
   - [Documentation Sources](./documentation-sources.md)

2. Search GitHub Issues for similar questions

3. Open a Discussion for general questions

4. Ask in PR comments for review-specific questions

### Recognition

Contributors are recognized through:

- **CHANGELOG.md**: All contributors credited in release notes
- **GitHub Contributors**: Visible on repository page
- **Community Shoutouts**: Regular contributor highlights

---

## First-Time Contributors

New to open source? Welcome! Here's how to get started:

### Good First Issues

Look for issues labeled `good first issue` - these are:

- Well-defined with clear acceptance criteria
- Relatively small in scope
- Good learning opportunities
- Mentor-supported

Browse: [Good First Issues](https://github.com/raibid-labs/scarab/labels/good%20first%20issue)

### Beginner-Friendly Areas

Easy places to start contributing:

1. **Documentation**: Fix typos, improve clarity, add examples
2. **Examples**: Create new plugin examples
3. **Tests**: Add test coverage for existing code
4. **Error Messages**: Improve error message clarity
5. **Code Comments**: Add explanatory comments to complex code

### Mentorship

Need help getting started? Maintainers are happy to:

- Answer questions about the codebase
- Provide guidance on your first PR
- Review and provide constructive feedback
- Help you learn Rust/Bevy/Git

---

## Advanced Topics

### Working with Submodules

Scarab uses git submodules for some dependencies:

```bash
# Initialize submodules
git submodule update --init --recursive

# Update submodules
git submodule update --remote
```

### Release Process

Maintainers handle releases. Contributors should:

- Update CHANGELOG.md for user-facing changes
- Mark breaking changes with `!` (e.g., `feat!`)
- Follow semantic versioning conventions

### Architecture Decisions

Major architectural changes require:

1. GitHub Discussion proposing the change
2. Maintainer consensus
3. Architecture Decision Record (ADR)
4. Implementation plan with milestones

---

## Resources

### Project Documentation

- [Architecture Overview](./architecture.md)
- [Plugin Development](./plugins.md)
- [Testing Guide](./testing.md)
- [Documentation Sources](./documentation-sources.md)

### External Resources

- [Rust Book](https://doc.rust-lang.org/book/)
- [Bevy Book](https://bevyengine.org/learn/book/)
- [Fusabi Documentation](https://github.com/fusabi-lang/fusabi)
- [Conventional Commits](https://www.conventionalcommits.org/)

### Development Tools

- [cargo-watch](https://github.com/watchexec/cargo-watch) - Auto-rebuild on changes
- [cargo-edit](https://github.com/killercup/cargo-edit) - Manage dependencies
- [cargo-flamegraph](https://github.com/flamegraph-rs/flamegraph) - Performance profiling
- [just](https://github.com/casey/just) - Command runner

---

## Questions?

Can't find what you need?

- Open a [Discussion](https://github.com/raibid-labs/scarab/discussions)
- Check the [FAQ](../reference/faq.md)
- Ask in an [Issue](https://github.com/raibid-labs/scarab/issues)

**Thank you for contributing to Scarab!**

---

## See Also

- [Contributing Guide (Root)](../../../../CONTRIBUTING.md) - Detailed git workflow
- [Contributing to Documentation](../contributing-docs.md) - mdBook guide
- [Code Style Guide](./code-style.md) - Rust conventions
- [Pull Request Process](./pr-process.md) - PR workflow details

---

**Last Updated**: 2025-12-03
