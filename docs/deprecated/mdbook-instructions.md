# mdBook Instructions (Deprecated)

> **DEPRECATED**: This document contains legacy mdBook instructions. The primary documentation site is now built from the external `~/raibid-labs/docs` repository. The `docs/book/` folder remains for historical reference but is not actively maintained.

## Why Deprecated

The in-repo mdBook setup (`docs/book/`) has been superseded by:

1. **External docs site** - Built from `~/raibid-labs/docs` repository
2. **Rustdoc** - `cargo doc --workspace --open` for API reference
3. **In-repo canonical docs** - `docs/README.md` as the documentation index

## Legacy mdBook Setup

If you need to work with the legacy mdBook content in `docs/book/`:

### Install mdBook

```bash
cargo install mdbook
```

### Optional Tools

```bash
cargo install mdbook-linkcheck
cargo install mdbook-mermaid
```

### Build

```bash
cd docs/book
mdbook build
```

### Serve Locally

```bash
cd docs/book
mdbook serve --open
```

## Current Approach

For contributing to documentation, see [CONTRIBUTING-DOCS.md](../CONTRIBUTING-DOCS.md).

---

**Deprecated:** 2025-12-04
