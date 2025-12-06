# Scarab Documentation Structure

This document describes the organization of Scarab's documentation.

## Directory Structure

```
docs/
├── STRUCTURE.md                  # This file - documentation organization
├── PLUGIN_MANIFEST.md            # Plugin manifest schema reference
├── FUSABI_TUI_INTEGRATION.md     # fusabi-tui migration guide
├── PLUGIN_DOCK_IMPLEMENTATION.md # Plugin dock system docs
├── versions/                     # Version-specific documentation
│   └── vNEXT/                   # Upcoming release docs
│       └── README.md            # vNEXT feature list
├── audits/                      # Code audit records
│   ├── codex-*/                 # Codex audit sessions
│   ├── gemini-*/                # Gemini audit sessions
│   └── claude-*/                # Claude audit sessions
├── reviews/                     # Code review documentation
├── assets/                      # Documentation assets
│   └── demos/                   # Demo recordings
└── tutorials/                   # User tutorials (planned)
    ├── 01-getting-started.md
    ├── 02-customization.md
    └── 03-workflows.md
```

## Top-Level Documentation Files

Located in the repository root:

- **`README.md`** - Main project README with overview, installation, quick start
- **`TESTING.md`** - Comprehensive testing guide
- **`CONTRIBUTING.md`** - Contribution guidelines
- **`ROADMAP.md`** - Strategic development roadmap
- **`CHANGELOG.md`** - Release history and changes
- **`MIGRATION_GUIDE.md`** - Version migration instructions
- **`CLAUDE.md`** - Architecture overview for AI assistants

## Documentation Categories

### 1. User Documentation

**Purpose**: Help users install, configure, and use Scarab

- `README.md` - Quick start and overview
- `docs/tutorials/` - Step-by-step guides
- `TESTING.md` - How to run tests
- `examples/` - Example configurations and plugins

**Target Audience**: End users, terminal enthusiasts

### 2. Developer Documentation

**Purpose**: Help contributors understand and extend Scarab

- `CONTRIBUTING.md` - How to contribute
- `CLAUDE.md` - Architecture overview
- `docs/audits/` - Code quality reviews
- `docs/reviews/` - Detailed implementation reviews
- Crate-level `README.md` files in `crates/*/`

**Target Audience**: Contributors, plugin developers

### 3. Plugin Developer Documentation

**Purpose**: Help developers create Scarab plugins

- `docs/PLUGIN_MANIFEST.md` - Manifest schema
- `examples/plugins/` - Example plugins
- Plugin API reference (via `cargo doc`)
- `docs/FUSABI_TUI_INTEGRATION.md` - UI integration

**Target Audience**: Plugin authors

### 4. Integration Documentation

**Purpose**: Technical guides for integrating shared libraries

- `docs/FUSABI_TUI_INTEGRATION.md` - fusabi-tui adoption
- `IMPLEMENTATION_SUMMARY.md` - Technical details
- `DEPENDENCY_DIAGRAM.txt` - Dependency graph

**Target Audience**: Core developers, integrators

### 5. Release Documentation

**Purpose**: Track releases and version-specific information

- `CHANGELOG.md` - All releases
- `docs/versions/vNEXT/` - Upcoming features
- `RELEASE_NOTES_*.md` - Specific release notes
- `VERSION` - Current version

**Target Audience**: Users, release managers

## Documentation Best Practices

### Writing Guidelines

1. **Clear Structure**: Use headers, lists, and code blocks
2. **Examples**: Provide working code examples
3. **Cross-References**: Link to related documentation
4. **Keep Updated**: Update docs with code changes
5. **Audience-Specific**: Write for the intended reader

### Code Examples

- Use triple backticks with language tags
- Test examples to ensure they work
- Provide context and explanation
- Show both input and output

### Markdown Style

- Use ATX headers (`#`, `##`, `###`)
- One sentence per line for readability
- Prefer lists over long paragraphs
- Use tables for structured data

## Maintenance

### Adding New Documentation

1. Identify the appropriate category
2. Follow the established structure
3. Update `STRUCTURE.md` if adding new directories
4. Cross-reference from relevant docs
5. Add to CI doc check (if applicable)

### Deprecating Documentation

1. Move to `docs/archive/` (create if needed)
2. Add deprecation notice with redirect
3. Update cross-references
4. Note in CHANGELOG

### Version-Specific Docs

- Use `docs/versions/vX.Y.Z/` for release-specific docs
- `docs/versions/vNEXT/` for upcoming changes
- Archive old versions after major releases

## CI Integration

Documentation checks (planned):

- Markdown linting
- Link checking
- Code example validation
- Spelling and grammar

## Related Files

- **`TESTING.md`** - Test documentation
- **`CONTRIBUTING.md`** - Contribution guidelines
- **`ROADMAP.md`** - Future planning
- **`CHANGELOG.md`** - Release history

## Questions?

If you're unsure where documentation belongs:

1. Check this structure guide
2. Look for similar existing documentation
3. Ask in GitHub Discussions
4. Create an issue for clarification
