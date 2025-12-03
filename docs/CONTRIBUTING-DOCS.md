# Contributing to Documentation

Thank you for helping improve Scarab's documentation!

## Prerequisites

### Install mdBook

```bash
cargo install mdbook
```

### Optional Tools

```bash
# Link checking
cargo install mdbook-linkcheck

# Mermaid diagrams
cargo install mdbook-mermaid
```

## Building Documentation

### Build the book

```bash
cd docs/book
mdbook build
```

Or use the justfile target:

```bash
just docs-build
```

Output is generated in `docs/book/build/`.

## Local Preview

### Serve with live reload

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
3. Open your browser
4. Watch for changes and auto-reload

## Documentation Structure

```
docs/
├── book/                          # mdBook source
│   ├── book.toml                 # mdBook configuration
│   ├── src/                      # Markdown source files
│   │   ├── SUMMARY.md           # Table of contents
│   │   ├── introduction.md      # Landing page
│   │   ├── user-guide/          # User documentation
│   │   ├── developer-guide/     # Developer documentation
│   │   └── reference/           # API and reference docs
│   └── build/                   # Generated HTML (gitignored)
├── navigation/                   # Navigation system docs
├── audits/                       # Code audit reports
├── configuration.md              # Configuration guide
└── CONTRIBUTING-DOCS.md         # This file
```

## Content Guidelines

### Writing Style

- **Clear and concise**: Use simple language
- **Active voice**: "The daemon handles..." not "Requests are handled..."
- **Code examples**: Include runnable examples where possible
- **Cross-references**: Link to related sections liberally

### Markdown Conventions

#### Code Blocks

Use language tags for syntax highlighting:

````markdown
```rust
fn main() {
    println!("Hello, world!");
}
```

```bash
cargo build --release
```

```toml
[package]
name = "scarab"
```
````

#### Links

- **Internal links**: Use relative paths
  ```markdown
  [Configuration](./user-guide/configuration.md)
  ```

- **External links**: Use full URLs
  ```markdown
  [Bevy](https://bevyengine.org)
  ```

- **Existing docs**: Link rather than duplicate
  ```markdown
  For details, see [Navigation Spec](../../navigation/NAVIGATION_SPEC.md)
  ```

#### Headings

Use hierarchical headings:

```markdown
# Top Level (Page Title)

## Major Section

### Subsection

#### Detail Section
```

### File Organization

- **One topic per page**: Keep pages focused
- **Descriptive filenames**: Use kebab-case (e.g., `getting-started.md`)
- **Update SUMMARY.md**: Add new pages to the table of contents

## Adding New Pages

1. **Create the markdown file**:
   ```bash
   touch docs/book/src/user-guide/new-feature.md
   ```

2. **Add to SUMMARY.md**:
   ```markdown
   # User Guide
   - [Getting Started](./user-guide/getting-started.md)
   - [New Feature](./user-guide/new-feature.md)  # Add this
   ```

3. **Write content**: Follow the guidelines above

4. **Test locally**:
   ```bash
   just docs-serve
   ```

## Linking to Existing Documentation

Scarab has extensive standalone documentation. **Don't duplicate content** - link to it:

### Example

Instead of copying navigation documentation:

```markdown
# Navigation

## Spatial Navigation

[Full navigation specification](../../navigation/NAVIGATION_SPEC.md)

## Quick Reference

- Use arrow keys to move between panes
- See [Navigation README](../../navigation/README.md) for details
```

### Existing Documentation

- `docs/navigation/` - Navigation system design and specs
- `docs/configuration.md` - Configuration guide
- `docs/audits/` - Code audit reports
- `CLAUDE.md` - Project architecture and constraints
- `README.md` - Project overview

## Style Consistency

### Terminology

Use consistent terms:

- **daemon** (not server, backend)
- **client** (not frontend, GUI)
- **pane** (not window, panel)
- **tab** (not workspace, session)
- **Fusabi** (not F#, FSharp)

### Formatting

- **Code**: Use backticks for inline code: `scarab-daemon`
- **Commands**: Use code blocks for shell commands
- **Paths**: Use code formatting: `/home/user/.config/scarab/`
- **Keyboard**: Use kbd notation: `Ctrl+Shift+T`

## Documentation Types

### User Guide

- Focus on **what** and **how**
- Provide step-by-step instructions
- Include screenshots where helpful
- Assume no Rust knowledge

### Developer Guide

- Focus on **why** and **how it works**
- Explain architecture and design decisions
- Link to code when relevant
- Assume Rust knowledge

### Reference

- Complete, authoritative information
- Structured for lookup, not learning
- Include all options and parameters
- Keep up-to-date with code

## Testing Documentation

### Check for broken links

```bash
# If mdbook-linkcheck is installed
mdbook build

# Manual check
grep -r "\.md" docs/book/src/ | grep -o "\[.*\](.*)"
```

### Verify code examples

Run any shell commands or code examples to ensure they work:

```bash
# Test a command from the docs
cargo build -p scarab-daemon
```

## Review Process

Before submitting documentation changes:

1. **Build locally**: Ensure no errors
   ```bash
   just docs-build
   ```

2. **Preview**: Check formatting and links
   ```bash
   just docs-serve
   ```

3. **Spell check**: Use a spell checker

4. **Cross-references**: Verify all links work

5. **Update SUMMARY.md**: If adding new pages

## Continuous Improvement

Documentation is never finished. Look for:

- Outdated information
- Confusing explanations
- Missing examples
- Broken links
- Typos and grammar issues

Submit improvements, no matter how small!

## Getting Help

Questions about documentation?

- Open an issue on GitHub
- Ask in project chat
- Check existing documentation for patterns

## Resources

- [mdBook Documentation](https://rust-lang.github.io/mdBook/)
- [Markdown Guide](https://www.markdownguide.org/)
- [Rust Documentation Guidelines](https://doc.rust-lang.org/book/ch14-02-publishing-to-crates-io.html#documentation-comments-as-tests)
