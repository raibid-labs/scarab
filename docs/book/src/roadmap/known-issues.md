# Known Issues & Limitations

Current known issues and limitations in Scarab.

## Current Limitations (Alpha v0.1.0)

### Platform Support

**Linux Only**
- Currently supports Linux (X11 and Wayland)
- macOS support planned for Phase 7 (Q2 2025)
- Windows support planned for Phase 7 (Q2 2025)

### Missing Core Features

**Tabs and Splits**
- Single pane only currently
- Tabs planned for Phase 6 (Q1 2025)
- Split panes planned for Phase 6 (Q1 2025)

**Ligatures**
- Programming ligatures not yet supported
- Planned for Phase 8 (Q2-Q3 2025)

**Image Protocols**
- No Sixel support yet
- No Kitty graphics protocol yet
- No iTerm2 inline images yet
- Planned for Phase 8 (Q2-Q3 2025)

### Performance

**Large Scrollback**
- Performance degrades with >100k lines scrollback
- Optimization planned for Phase 5

**Plugin Load Time**
- Frontend plugin (.fsx) parsing can be slow
- Caching planned for Phase 6

### Configuration

**Limited Hot-Reload**
- Some config changes require restart
- Backend plugins require daemon restart
- Full hot-reload planned for Phase 6

### Plugin System

**Limited Plugin API**
- Some WezTerm Lua APIs not yet available
- Plugin marketplace not yet available
- API stability not guaranteed (alpha)

## Known Bugs

Track bugs on GitHub:
- [Open Issues](https://github.com/raibid-labs/scarab/issues)

### High Priority

None currently (alpha quality expected)

### Medium Priority

Check GitHub Issues for latest bug reports.

### Low Priority

Cosmetic issues and minor inconsistencies.

## Workarounds

### Large Scrollback Performance

Reduce scrollback buffer size:

```toml
[terminal]
scrollback_lines = 10000  # Instead of 100000
```

### Plugin Load Time

Use compiled backend plugins (.fzb) for performance-critical plugins.

### Missing Tabs/Splits

Use tmux or screen as a temporary workaround:

```bash
# In Scarab terminal
tmux new-session -s my-session
```

## Reporting Issues

Found a bug? Please report it:

1. Check [existing issues](https://github.com/raibid-labs/scarab/issues)
2. If not found, [open a new issue](https://github.com/raibid-labs/scarab/issues/new)
3. Include:
   - Scarab version (`cargo --version`)
   - OS and version
   - Steps to reproduce
   - Expected vs actual behavior
   - Logs (with `RUST_LOG=debug`)

## See Also

- [Troubleshooting](../reference/troubleshooting.md) - Common problems and solutions
- [FAQ](../reference/faq.md) - Frequently asked questions
- [Roadmap](./overview.md) - Feature timeline
- [GitHub Issues](https://github.com/raibid-labs/scarab/issues) - Bug tracker
