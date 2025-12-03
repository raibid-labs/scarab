# Documentation Setup

Instructions for building and serving the Scarab documentation.

## Prerequisites

### Install mdBook

mdBook is required to build the documentation:

```bash
cargo install mdbook
```

### Optional: Install mdBook Plugins

For additional features:

```bash
# Link checking
cargo install mdbook-linkcheck

# PDF generation
cargo install mdbook-pdf

# Mermaid diagrams
cargo install mdbook-mermaid
```

## Building Documentation

### Build the Book

```bash
# Using just
just docs-build

# Or directly with mdbook
cd docs/book
mdbook build
```

The built documentation will be in `docs/book/build/`.

### Serve Locally

```bash
# Using just (opens in browser)
just docs-serve

# Or directly with mdbook
cd docs/book
mdbook serve --open
```

The documentation will be served at `http://localhost:3000`.

## Building Rust API Docs

Generate rustdoc documentation for all crates:

```bash
# Generate docs for entire workspace
cargo doc --workspace --no-deps --open

# Generate docs for specific crate
cargo doc -p scarab-daemon --no-deps --open
cargo doc -p scarab-client --no-deps --open
cargo doc -p scarab-protocol --no-deps --open
cargo doc -p scarab-plugin-api --no-deps --open
cargo doc -p scarab-config --no-deps --open
```

## CI/CD Integration

Documentation is automatically built and deployed on:

1. **Pull Requests** - Docs build is verified
2. **Main Branch** - Deployed to GitHub Pages
3. **Releases** - Versioned documentation

### GitHub Pages Deployment

The documentation is deployed to:
- mdBook: `https://raibid-labs.github.io/scarab/`
- rustdoc: `https://raibid-labs.github.io/scarab/rustdoc/`

### Local Preview

To preview the deployed structure locally:

```bash
# Build both mdBook and rustdoc
just docs-build
cargo doc --workspace --no-deps

# Serve from a simple HTTP server
python -m http.server 8000 -d docs/book/build/
```

Then visit:
- mdBook: `http://localhost:8000/`
- rustdoc: `http://localhost:8000/rustdoc/` (after copying rustdoc output)

## Directory Structure

```
docs/
├── book/
│   ├── book.toml           # mdBook configuration
│   ├── src/                # Markdown source files
│   │   ├── SUMMARY.md      # Table of contents
│   │   ├── introduction.md
│   │   ├── getting-started/
│   │   ├── user-guide/
│   │   ├── developer-guide/
│   │   ├── reference/
│   │   └── roadmap/
│   ├── build/              # Built output (gitignored)
│   └── SETUP.md            # This file
└── [other documentation files]
```

## Troubleshooting

### mdbook Command Not Found

Install mdbook:
```bash
cargo install mdbook
```

Add Cargo bin to PATH:
```bash
export PATH="$HOME/.cargo/bin:$PATH"
```

### Missing Files Errors

If mdbook reports missing files, they may need to be created as stubs. Check SUMMARY.md for the expected file structure.

### Link Errors

To check for broken links (requires mdbook-linkcheck):

```bash
# Uncomment linkcheck in book.toml
# Then build
mdbook build
```

## Contributing to Documentation

### Adding New Pages

1. Create the markdown file in the appropriate directory
2. Add an entry to `src/SUMMARY.md`
3. Build and verify: `mdbook build`
4. Test locally: `mdbook serve`

### Documentation Standards

- Use clear, concise language
- Include code examples where applicable
- Link to related pages
- Keep navigation structure logical
- Update SUMMARY.md when adding pages

### Style Guide

- Use H1 (`#`) for page title
- Use H2-H6 for subsections
- Use code blocks with language tags
- Use tables for structured data
- Include "See Also" sections for related content

## See Also

- [Contributing to Docs](src/contributing-docs.md) - Contribution guidelines
- [mdBook Documentation](https://rust-lang.github.io/mdBook/) - Official mdBook guide
- [GitHub Pages](https://pages.github.com/) - Hosting documentation
