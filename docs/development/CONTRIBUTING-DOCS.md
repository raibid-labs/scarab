# Contributing to Documentation

Thank you for helping improve Scarab's documentation!

## Documentation Architecture

Scarab uses a **multi-source documentation model**:

| Source | Purpose | Location |
|--------|---------|----------|
| **External Docs Site** | User guides, tutorials, browsable docs | `~/raibid-labs/docs` (separate repo) |
| **Rustdoc** | API reference, inline code docs | `cargo doc --workspace --open` |
| **In-Repo Docs** | Canonical references, implementation notes | `docs/` folder |

> **Note:** The in-repo `docs/book/` folder contains legacy mdBook scaffolding. For mdBook-specific instructions, see [deprecated/mdbook-instructions.md](./deprecated/mdbook-instructions.md).

## In-Repo Documentation

This `docs/` folder contains:

- **Canonical references** - Navigation system, plugin development, architecture
- **Implementation notes** - Design decisions, status tracking
- **Audits & research** - Code reviews, gap analyses
- **Deprecation notices** - Superseded documentation

### Structure

```
docs/
├── README.md                     # Documentation index (start here)
├── navigation/                   # Navigation system (current)
├── plugin-development/           # Plugin development (current)
├── developer/                    # Architecture, implementation
├── deprecated/                   # Superseded documentation
├── audits/                       # Code audits and reviews
└── ...                           # Other reference docs
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
````

#### Links

- **Internal links**: Use relative paths
  ```markdown
  [Configuration](./configuration.md)
  ```

- **External links**: Use full URLs
  ```markdown
  [Bevy](https://bevyengine.org)
  ```

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

## Adding Documentation

### Adding to In-Repo Docs

1. **Choose the right location**:
   - User-facing → `user/`, `tutorials/`, `reference/`
   - Developer → `developer/`, `architecture/`, `plugin-development/`
   - Historical → `audits/`, `deprecated/`

2. **Update docs/README.md**: Add links to new documents

3. **Use descriptive filenames**: kebab-case (e.g., `getting-started.md`)

### Marking Documentation as Deprecated

When documentation is superseded:

1. **Move to `deprecated/`** folder
2. **Add deprecation notice** at the top:
   ```markdown
   > **DEPRECATED**: This document has been superseded by [Current Doc](../path/to/current.md).
   ```
3. **Update deprecated/README.md** to list the file

## Rustdoc Guidelines

For API documentation in Rust code:

```rust
/// Creates a new terminal session with the given configuration.
/// 
/// # Arguments
/// 
/// * `config` - Session configuration options
/// 
/// # Examples
/// 
/// ```rust
/// let session = Session::new(SessionConfig::default());
/// ```
pub fn new(config: SessionConfig) -> Self { ... }
```

Generate and view:

```bash
cargo doc --workspace --open
```

## Review Process

Before submitting documentation changes:

1. **Verify links work**: Check relative paths
2. **Spell check**: Use a spell checker
3. **Update index**: Add to docs/README.md if needed
4. **Test code examples**: Ensure they run

## Getting Help

- Open an issue on GitHub
- Check the [documentation index](./README.md)
- Review existing documentation patterns

## Resources

- [Markdown Guide](https://www.markdownguide.org/)
- [Rust Documentation Guidelines](https://doc.rust-lang.org/book/ch14-02-publishing-to-crates-io.html#documentation-comments-as-tests)
