# Documentation Sources

Scarab's documentation ecosystem consists of multiple systems, each serving a specific purpose. This page explains where to find documentation, how to build it, and how to contribute.

## Quick Navigation

- [mdBook Portal](#mdbook-portal) - User and developer guides
- [Rustdoc](#rustdoc) - API reference
- [Top-level Docs](#top-level-docs) - Quick reference
- [Legacy/Deprecated](#legacydeprecated) - Historical reference

---

## mdBook Portal

**Location:** `docs/book/`

The mdBook portal is the primary documentation hub for Scarab, providing comprehensive guides for users, developers, and plugin authors.

### Purpose

- User guides, tutorials, and walkthroughs
- Developer guides and architecture documentation
- Plugin development tutorials and API guides
- Reference documentation and troubleshooting

### Build

Build the documentation locally:

```bash
cd docs/book
mdbook build
```

Or use the justfile target:

```bash
just docs-build
```

Output is generated in `docs/book/build/`.

### Preview

Serve with live reload for local development:

```bash
cd docs/book
mdbook serve --open
```

Or use the justfile target:

```bash
just docs-serve
```

This will:
1. Build the documentation
2. Start a local web server (default: http://localhost:3000)
3. Open your browser automatically
4. Watch for changes and auto-reload

### Contributing

See [Contributing to Documentation](../contributing-docs.md) for detailed guidelines on writing and organizing mdBook content

---

## Rustdoc

**Location:** Generated from source code in `crates/`

Rustdoc provides comprehensive API documentation for all Scarab crates, generated directly from source code comments.

### Purpose

- API reference for public types, traits, and functions
- Module documentation with examples
- Trait documentation and type definitions
- Safety requirements for unsafe code

### Build

Generate documentation for the entire workspace:

```bash
cargo doc --workspace --no-deps --open
```

Generate documentation for a specific crate:

```bash
cargo doc -p scarab-daemon --no-deps --open
cargo doc -p scarab-client --no-deps --open
cargo doc -p scarab-protocol --no-deps --open
```

### Standards

See [Rustdoc Standards](../reference/rustdoc.md) for detailed guidelines on writing API documentation, including:

- Module-level documentation
- Type and trait documentation
- Function documentation with examples
- Safety requirements for unsafe code
- CI integration and lint checking

---

## Top-level Docs

**Location:** Repository root and `docs/`

Top-level documentation provides quick reference and project-wide information that doesn't fit in the mdBook structure.

### Key Files

**Repository Root:**
- `docs/README.md` - Central index linking to all documentation sources
- `TESTING.md` - Test commands and patterns (one-line quick reference)
- `CHANGELOG.md` - Release history and version changes
- `CLAUDE.md` - AI assistant context (architecture, constraints, build commands)

**Standalone Docs:**
- `docs/configuration.md` - Configuration guide
- `docs/navigation/` - Navigation system documentation
- `docs/plugin-development/` - Plugin development guides
- `docs/audits/` - Technical audits and architectural reviews

### When to Use

Use top-level docs for:
- Quick reference guides
- Release management documentation
- Historical records (audits, reports)
- AI assistant context
- Integration guides that span multiple systems

### Maintenance

Keep these files up-to-date:
- Update `docs/README.md` when adding major documentation
- Update `TESTING.md` when adding new test patterns
- Update `CHANGELOG.md` for every release
- Update `CLAUDE.md` when architecture changes

---

## Legacy/Deprecated

**Location:** `docs/deprecated/`

Historical documentation that has been superseded by newer, more comprehensive documentation.

### Purpose

Files in this directory are preserved for historical reference only:

1. **Superseded Documentation** - Replaced by more comprehensive guides
2. **Point-in-Time Reports** - Completion reports from specific development phases
3. **Historical Analysis** - Legacy architectural analysis from earlier development stages

### What's Deprecated

- Old plugin development guides (replaced by `docs/plugin-development/`)
- Phase completion reports (historical snapshots)
- Implementation summaries (point-in-time documentation)
- Audit pass reports (superseded by newer audits)

See `docs/deprecated/README.md` for:
- Full list of deprecated files
- Reasons for deprecation
- Pointers to current documentation

### Important

**Never use deprecated documentation for current development.** Always check `docs/README.md` for the latest documentation.

## See Also

- [Contributing to Documentation](../contributing-docs.md) - Detailed contribution guidelines
- [Rustdoc Standards](../reference/rustdoc.md) - API documentation standards
- [Documentation Index](../../../README.md) - Central documentation hub
- [Testing Guide](../../../../TESTING.md) - Test commands and patterns
