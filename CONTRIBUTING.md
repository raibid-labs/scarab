# Contributing to Scarab

Thank you for your interest in contributing to Scarab! This document outlines our development workflow, git practices, and contribution guidelines.

## Table of Contents

- [Development Workflow](#development-workflow)
- [Git Practices](#git-practices)
- [Branch Naming](#branch-naming)
- [Commit Messages](#commit-messages)
- [Pull Request Process](#pull-request-process)
- [Code Review Guidelines](#code-review-guidelines)
- [Release Integration](#release-integration)

## Development Workflow

### Overview

Scarab uses a **trunk-based development** model with short-lived feature branches:

```
main (protected) ◄── Feature PRs (squash merged)
     │
     └── Releases tagged from main (vX.Y.Z)
```

**Key principles:**
- All changes go through Pull Requests
- PRs are squash merged to keep history clean
- `main` is always releasable
- Feature branches are short-lived (< 1 week ideally)

### Getting Started

```bash
# Clone the repository
git clone https://github.com/raibid-labs/scarab.git
cd scarab

# Create a feature branch from main
git checkout main
git pull origin main
git checkout -b feature/your-feature-name

# Make your changes, commit frequently
git add .
git commit -m "feat: add initial implementation"

# Push and create PR
git push -u origin feature/your-feature-name
```

## Git Practices

### Branch Protection

The `main` branch is protected with the following rules:
- Require pull request reviews before merging
- Require status checks to pass before merging
- Require linear history (squash merges only)
- No direct pushes to main

### Squash Merging

**All PRs are squash merged.** This means:
- Multiple commits in a PR become a single commit on main
- The squash commit message becomes the PR title + description
- Write meaningful PR titles (they become the commit message!)

**Why squash merging?**
1. **Clean history**: Each commit on main represents one complete feature/fix
2. **Bisect-friendly**: Easy to find which change introduced a bug
3. **Revert-friendly**: Easy to revert a complete feature
4. **CHANGELOG-friendly**: One commit = one changelog entry

### Keeping Your Branch Updated

```bash
# Option 1: Rebase (preferred for clean history)
git fetch origin
git rebase origin/main

# Option 2: Merge (if rebase is complex)
git fetch origin
git merge origin/main
```

## Branch Naming

Use descriptive branch names with prefixes:

| Prefix | Purpose | Example |
|--------|---------|---------|
| `feature/` | New features | `feature/status-bar-api` |
| `fix/` | Bug fixes | `fix/cursor-positioning` |
| `refactor/` | Code refactoring | `refactor/ipc-protocol` |
| `docs/` | Documentation only | `docs/plugin-guide` |
| `test/` | Test additions/fixes | `test/e2e-coverage` |
| `perf/` | Performance improvements | `perf/vte-caching` |
| `chore/` | Maintenance tasks | `chore/update-deps` |

**Format:** `prefix/short-description` or `prefix/issue-number-short-description`

**Examples:**
```
feature/ws1-object-model
fix/123-mouse-coordinates
docs/wezterm-parity-plan
refactor/event-system
```

## Commit Messages

We follow the [Conventional Commits](https://www.conventionalcommits.org/) specification.

### Format

```
<type>(<scope>): <subject>

[optional body]

[optional footer(s)]
```

### Types

| Type | Description |
|------|-------------|
| `feat` | New feature |
| `fix` | Bug fix |
| `docs` | Documentation changes |
| `style` | Formatting, missing semicolons, etc. |
| `refactor` | Code change that neither fixes a bug nor adds a feature |
| `perf` | Performance improvement |
| `test` | Adding or fixing tests |
| `chore` | Maintenance tasks, dependency updates |
| `ci` | CI/CD changes |
| `build` | Build system changes |

### Scopes (Optional)

Scopes indicate which part of the codebase is affected:

- `daemon` - scarab-daemon crate
- `client` - scarab-client crate
- `protocol` - scarab-protocol crate
- `plugin-api` - scarab-plugin-api crate
- `config` - scarab-config crate
- `mouse` - scarab-mouse crate
- `clipboard` - scarab-clipboard crate
- `ipc` - IPC-related changes
- `vte` - VTE parser changes
- `ui` - UI-related changes
- `fusabi` - Fusabi integration

### Examples

```bash
# Feature
git commit -m "feat(client): add status bar rendering API"

# Bug fix
git commit -m "fix(daemon): correct cursor position after resize"

# Documentation
git commit -m "docs: add WezTerm parity workstream documentation"

# Breaking change (add ! after type)
git commit -m "feat(protocol)!: redesign SharedState for image support"

# With body
git commit -m "feat(plugin-api): add object model infrastructure

Implements handle-based proxies for Window, Pane, and Tab objects.
Includes ObjectRegistry trait with client/daemon implementations.

Closes #42"
```

### Commit Message in PRs

Since we squash merge, **the PR title becomes the commit message**. Write your PR title as a proper conventional commit:

**Good PR titles:**
- `feat(client): implement key table stack for modal editing`
- `fix(daemon): prevent memory leak in image cache`
- `docs: add release roadmap for WezTerm parity`

**Bad PR titles:**
- `Update files`
- `WIP changes`
- `Fix stuff`

## Pull Request Process

### Creating a PR

1. **Push your branch:**
   ```bash
   git push -u origin feature/your-feature
   ```

2. **Create PR via GitHub CLI or web:**
   ```bash
   gh pr create --title "feat(scope): description" --body "..."
   ```

3. **Fill out the PR template:**
   - Summary of changes
   - Related issues (use `Closes #123`)
   - Testing performed
   - Screenshots (for UI changes)
   - Checklist items

### PR Template

```markdown
## Summary
<!-- Brief description of changes -->

## Related Issues
<!-- Link to related issues: Closes #123, Fixes #456 -->

## Changes
<!-- Bullet list of specific changes -->
- Added X
- Modified Y
- Removed Z

## Testing
<!-- How was this tested? -->
- [ ] Unit tests added/updated
- [ ] Integration tests added/updated
- [ ] Manual testing performed

## Screenshots
<!-- For UI changes, include before/after screenshots -->

## Checklist
- [ ] Code follows project style guidelines
- [ ] Self-review completed
- [ ] Documentation updated (if needed)
- [ ] CHANGELOG.md updated (for user-facing changes)
- [ ] No new warnings introduced
```

### PR Lifecycle

```
Draft PR ──► Ready for Review ──► Approved ──► Squash Merged
                   │                  │
                   │                  └── Request changes
                   │                           │
                   └───────────────────────────┘
```

1. **Draft PR**: For work-in-progress, early feedback
2. **Ready for Review**: All checks passing, ready for review
3. **Changes Requested**: Address feedback, re-request review
4. **Approved**: Maintainer approves
5. **Squash Merged**: Merged to main with squash

### PR Checklist Before Requesting Review

- [ ] Branch is up to date with main
- [ ] All CI checks pass
- [ ] Code compiles without warnings
- [ ] Tests added for new functionality
- [ ] Documentation updated if needed
- [ ] CHANGELOG.md updated for user-facing changes
- [ ] PR title follows conventional commit format
- [ ] No unrelated changes included

## Code Review Guidelines

### For Authors

- Keep PRs focused and small (< 400 lines ideal)
- Respond to feedback promptly
- Don't take feedback personally
- Explain your reasoning when disagreeing
- Mark resolved comments as resolved

### For Reviewers

- Be respectful and constructive
- Explain the "why" behind suggestions
- Distinguish between blocking and non-blocking feedback
- Approve when satisfied, don't block on nitpicks
- Use suggestion syntax for simple fixes:

  ```suggestion
  // Use this format for simple inline suggestions
  let result = calculate_value();
  ```

### Review Focus Areas

1. **Correctness**: Does the code do what it's supposed to?
2. **Design**: Is the approach sound? Are there better alternatives?
3. **Performance**: Any obvious performance issues?
4. **Security**: Any security concerns?
5. **Maintainability**: Is the code readable and maintainable?
6. **Tests**: Are changes adequately tested?

## Release Integration

### How PRs Relate to Releases

1. **Each squash-merged PR = one CHANGELOG entry**
2. **Breaking changes** (`feat!`, `fix!`) increment MAJOR version
3. **Features** (`feat`) increment MINOR version
4. **Fixes** (`fix`) increment PATCH version

### Updating CHANGELOG

For user-facing changes, update `CHANGELOG.md` in your PR:

```markdown
## [Unreleased]

### Added
- Status bar rendering API for custom status content (#123)

### Fixed
- Cursor positioning after window resize (#456)
```

The release process will move `[Unreleased]` entries to the new version section.

### Milestone Tagging

PRs should be associated with milestones when applicable:

```bash
# Via GitHub CLI
gh pr edit --add-milestone "v0.2.0-alpha.1"
```

Current milestones for WezTerm parity work:
- `v0.2.0-alpha.1` - Object Model + Event System foundations
- `v0.2.0-alpha.2` - Status Bar API + Key Tables
- `v0.2.0-alpha.3` - Image Protocols + Copy Mode
- `v0.2.0-beta.1` - Feature complete, stabilization
- `v0.2.0` - Stable release

## Quick Reference

### Common Git Commands

```bash
# Start new feature
git checkout main && git pull && git checkout -b feature/name

# Update branch from main
git fetch origin && git rebase origin/main

# Amend last commit (before pushing)
git commit --amend

# Interactive rebase (clean up commits before PR)
git rebase -i HEAD~3

# Force push after rebase (to your branch only!)
git push --force-with-lease

# Create PR
gh pr create --title "feat(scope): description"

# Check PR status
gh pr status

# View PR diff
gh pr diff
```

### Useful Aliases

Add to your `~/.gitconfig`:

```ini
[alias]
    co = checkout
    br = branch
    ci = commit
    st = status
    unstage = reset HEAD --
    last = log -1 HEAD
    lg = log --oneline --graph --decorate
    pr = !gh pr create
    prs = !gh pr status
```

## Questions?

- Open a [Discussion](https://github.com/raibid-labs/scarab/discussions) for general questions
- Open an [Issue](https://github.com/raibid-labs/scarab/issues) for bugs or feature requests
- Check existing documentation in `docs/`

Thank you for contributing to Scarab!
