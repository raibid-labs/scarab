# Executive Summary: Scarab Plugin Integration

**Date**: 2025-12-11
**Project**: Scryforge & Sigilforge as Scarab Terminal Plugins

---

## TL;DR

Integrating Scryforge (YouTube/RSS browser) and Sigilforge (credential manager) into Scarab terminal is **feasible** with **moderate effort**. The main blocker is Fusabi ecosystem version fragmentation which requires upstream coordination before implementation can begin.

**Estimated Timeline**: 8-11 weeks
**Critical Path Blocker**: Fusabi package version alignment (~2 weeks)

---

## Goals Achieved

| Goal | Status | Notes |
|------|--------|-------|
| Plugin management testing | Ready | Plugin API mature, tests via ratatui-testlib |
| Minimal working navigation | Clear path | scarab-nav protocol established |
| Full Scarab-style navigation | Designed | Hint mode + vim keys documented |
| YouTube plugin | Needs auth | YouTubeProvider implemented, needs Sigilforge |
| Status bar tabs | Designed | RenderItem API supports tabs |

---

## Architecture Compatibility

All projects share compatible patterns:

```
                    ┌─────────────────┐
                    │  Scarab Client  │
                    │   (Bevy GPU)    │
                    └────────┬────────┘
                             │ shared memory
                    ┌────────┴────────┐
                    │  Scarab Daemon  │
                    │   (PTY + VTE)   │
                    └────────┬────────┘
                             │ Unix socket
        ┌────────────────────┼────────────────────┐
        │                    │                    │
┌───────┴───────┐   ┌────────┴────────┐   ┌───────┴───────┐
│   Scryforge   │   │    Sigilforge   │   │   scarab-nav  │
│  (TUI Plugin) │   │  (Auth Daemon)  │   │  (Navigation) │
└───────────────┘   └─────────────────┘   └───────────────┘
```

**Key Compatibility Points**:
- All use Tokio async runtime
- All use Unix socket IPC
- All use trait-based plugin/provider patterns
- Navigation protocol is protobuf (forward-compatible)

---

## Critical Issues

### 1. Fusabi Version Fragmentation (BLOCKING)

| Package | Has | Needs |
|---------|-----|-------|
| bevy-fusabi | 0.17.0 | 0.21.0 |
| fusabi-tui | 0.16.0 | 0.21.0 |
| fusabi-host | 0.18-0.19 | 0.21.0 |

**Action Required**: Submit upstream PRs to fusabi-lang repos before starting implementation.

**Impact**: Cannot compile Scryforge as Scarab plugin until versions align.

### 2. Ratatui → Bevy Rendering Bridge

Scryforge uses Ratatui's `Buffer` for rendering. Scarab uses Bevy's GPU mesh pipeline.

**Solution**: Create `TuiPluginBridge` adapter crate:
```rust
Ratatui Buffer → Cell Grid → Scarab Mesh Vertices
```

**Effort**: 5-7 days implementation

### 3. Sigilforge Security

Socket lacks authentication - any process can request OAuth tokens.

**Solution**: Add peer credential verification (Linux `SO_PEERCRED`).

**Impact**: Low risk for local development, should fix before production use.

---

## Recommended Order of Work

```
Phase 0: Upstream Dependencies (BLOCKING)
    │
    ├── bevy-fusabi PR (2-3 days)
    ├── fusabi-tui PR (2-3 days)
    └── Documentation PRs (2 days)
    │
    ▼
Phase 1: Plugin Infrastructure
    │
    ├── TuiPluginBridge crate (5-7 days)
    ├── ScryforgePlugin wrapper (3-4 days)
    └── Installation tests (2-3 days)
    │
    ▼
Phase 2-3: Navigation
    │
    ├── Focusable registration (2-3 days)
    ├── Hint mode integration (2 days)
    └── Full vim navigation (3-4 days)
    │
    ▼
Phase 4: YouTube Integration
    │
    ├── Sigilforge auth flow (2-3 days)
    ├── YouTube provider wiring (3-4 days)
    └── Thumbnails (optional, 3-5 days)
    │
    ▼
Phase 5-6: Polish
    │
    ├── Status bar tabs (4 days)
    ├── Integration tests (3-4 days)
    └── Documentation (2-3 days)
```

---

## Risk Assessment

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|------------|
| Fusabi PRs rejected/delayed | Low | Critical | Fork temporarily if needed |
| TuiPluginBridge complexity | Medium | Medium | Start text-only, add styling incrementally |
| OAuth UX friction | Medium | Low | Device code flow works without browser popup |
| Performance with 100+ items | Low | Low | Virtual scrolling, already tested in Scryforge |

---

## Success Metrics

### MVP (Week 6)
- [ ] Scryforge loads as Scarab plugin
- [ ] Navigate streams/items with keyboard
- [ ] Hint mode shows labels
- [ ] Select items via hint keys

### Full Release (Week 11)
- [ ] YouTube OAuth working via Sigilforge
- [ ] Browse subscriptions, playlists, watch later
- [ ] Status bar tabs for view switching
- [ ] 80%+ test coverage
- [ ] User documentation complete

---

## Resource Requirements

**Development**:
- 1 developer full-time for 8-11 weeks
- Or 2 developers for 4-6 weeks (parallel phases)

**Dependencies**:
- Fusabi maintainer availability for PR reviews
- Access to YouTube API quota for testing

**Testing**:
- ratatui-testlib provides all needed test infrastructure
- CI can run headless with `--features headless`

---

## Recommendations

1. **Start with Fusabi PRs immediately** - This is the critical path blocker. Even a fork with local patches unblocks the rest of the work.

2. **Build TuiPluginBridge incrementally** - Start with text-only rendering, add colors, then styling. Don't try to handle all Ratatui features at once.

3. **Use device code flow for OAuth** - Simpler UX than browser popup, works in pure terminal environment.

4. **Test navigation early** - The hint mode system is the key differentiator. Get it working early and iterate on UX.

5. **Defer thumbnails** - Video thumbnails are nice-to-have. Focus on text-based browsing first.

---

## Documents in This Audit

| Document | Description |
|----------|-------------|
| [01-technical-audit.md](./01-technical-audit.md) | Full technical analysis of all 6 projects |
| [02-integration-roadmap.md](./02-integration-roadmap.md) | Phase-by-phase implementation plan |
| [03-upstream-work.md](./03-upstream-work.md) | Fusabi-lang upstream PRs needed |
| 00-executive-summary.md | This document |

---

## Next Steps

1. **This Week**: Submit Fusabi upstream PRs
2. **Week 2**: Begin TuiPluginBridge while PRs in review
3. **Week 3**: Integrate ScryforgePlugin wrapper
4. **Week 4+**: Follow roadmap phases

---

## Conclusion

The integration is architecturally sound and all components are designed with compatible patterns. The main challenge is coordinating upstream Fusabi releases. Once that's resolved, the implementation path is well-defined with comprehensive testing infrastructure already available.

The end result will be a terminal emulator that doubles as a content browser - browse YouTube, read RSS, check email, all with Vimium-style keyboard navigation, without leaving the terminal.
