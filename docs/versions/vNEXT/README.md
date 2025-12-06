# vNEXT - Upcoming Release

This directory contains documentation for features and changes planned for the next release.

## Planned Features

### Core Terminal
- [ ] Improved GPU rendering performance
- [ ] Enhanced sixel image support
- [ ] Better Unicode handling

### Plugin System
- [x] Plugin manifest schema with capability enforcement (#96)
- [x] fusabi-plugin-runtime integration (#95)
- [x] fusabi-stdlib-ext modules (#95, #101)
- [ ] Plugin marketplace integration
- [ ] Hot-reload improvements

### UI/UX
- [x] fusabi-tui integration (#97)
- [x] ratatui-testlib smoke tests (#98)
- [ ] Improved command palette fuzzy search
- [ ] Enhanced link hints rendering
- [ ] Better theme system

### Testing
- [x] ratatui-testlib smoke test framework (#98)
- [ ] Golden snapshot tests with ratatui-testlib
- [ ] Performance regression tests

### Documentation
- [x] Plugin manifest documentation (#96)
- [x] fusabi-tui integration guide (#97)
- [x] Documentation structure guide (this release)
- [ ] Tutorial improvements
- [ ] API reference updates

## Breaking Changes

### Plugin API
- Plugin manifests are now recommended (not required yet for backward compatibility)
- Capabilities must be declared in manifests for new plugins
- Old plugins without manifests get minimal permissions

### Configuration
- No breaking changes planned

## Migration Guide

### For Plugin Developers
1. Add a `plugin-name.manifest.toml` file (see `docs/PLUGIN_MANIFEST.md`)
2. Declare required capabilities and modules
3. Update documentation to reference new manifest schema

### For Users
- No migration needed
- Existing configurations continue to work
- New features available immediately upon upgrade

## Timeline

Target release: TBD

## Notes

- All completed items from issues #95-#104 will be included
- Focus on stability and test coverage
- Documentation improvements are ongoing
