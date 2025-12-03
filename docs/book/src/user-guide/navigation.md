# Navigation

Scarab features an ECS-native navigation system that provides intuitive pane and tab management.

## Navigation Modes

For detailed information about navigation modes and behavior, see:
- [Navigation Design Document](../../navigation/README.md)
- [Navigation Specification](../../navigation/NAVIGATION_SPEC.md)

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
