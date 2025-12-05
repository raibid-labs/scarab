## What's New

### ECS-Safe UI/Navigation Bindings (#88)
- **New host bindings** for Fusabi plugins:
  - `spawn_overlay` / `remove_overlay` - Overlay management
  - `add_status_item` / `remove_status_item` - Status bar items
  - `prompt_jump` - Navigate between prompts (Up/Down/First/Last)
  - `register_focusable` / `unregister_focusable` - Navigation targets
- **Supporting types**: `OverlayConfig`, `StatusBarItem`, `JumpDirection`
- **Capability/quota enforcement** with rate limiting
- **14 integration tests** for binding validation

### Testing Infrastructure (#82, #87)
- **Comprehensive TESTING.md** with quick reference table
- **ratatui-testlib** with sixel features enabled
- **New tests**: graphics protocol detection, input latency measurement, escape sequence robustness
- **justfile targets**: `rtl-smoke`, `rtl-full`, `rtl-graphics`
- **CI workflow** for gated PTY tests (`SCARAB_TEST_RTL=1`)

### Documentation (#81)
- **docs/README.md** properly links to external docs, rustdoc, and canonical guides
- **Legacy docs deprecated** and moved to `docs/deprecated/`
- **No mdBook instructions** in main repo (handled externally)

## Breaking Changes
None

## Notes
- Fusabi pinned at v0.17.0 (bevy-fusabi constraint); will upgrade when compatible version releases
- ratatui-testlib Bevy/ECS integration pending upstream Phase 4

## Full Changelog
https://github.com/raibid-labs/scarab/compare/v0.1.0-alpha.15...v0.2.0-alpha.2
