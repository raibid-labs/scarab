# Navigation

> **DEPRECATION NOTICE**: This document is part of the legacy mdBook documentation structure and may be outdated.
>
> For current navigation documentation, see:
> - **User Guide**: [docs/navigation/user-guide.md](../../navigation/user-guide.md)
> - **Developer Guide**: [docs/navigation/developer-guide.md](../../navigation/developer-guide.md)
> - **Architecture Overview**: [docs/navigation.md](../../navigation.md)
>
> Last updated: 2025-12-03

---

Scarab features an ECS-native navigation system that provides intuitive pane and tab management.

## Navigation Modes

For detailed information about navigation modes and behavior, see:
- [Navigation User Guide](../../navigation/user-guide.md) - Current user-facing documentation
- [Navigation Developer Guide](../../navigation/developer-guide.md) - Current developer documentation
- [Navigation Architecture](../../navigation.md) - Complete technical overview

## Quick Reference

### Tab Navigation

- Create new tab
- Switch between tabs
- Close tabs

### Pane Navigation

- Split panes horizontally/vertically
- Navigate between panes
- Resize panes
- Close panes

### Focus Management

The navigation system uses a hybrid approach:
- **Spatial navigation**: Use directional keys to move between panes
- **Sequential navigation**: Tab through panes in order
- **Direct selection**: Click or use shortcuts to jump to specific panes

## Advanced Features

- **Focus history**: Return to previously focused panes
- **Smart wraparound**: Configurable boundary behavior
- **Visual feedback**: Clear focus indicators

For implementation details, see the [Navigation System Developer Guide](../developer-guide/navigation.md).
