# Navigation System

> **DEPRECATION NOTICE**: This document is part of the legacy mdBook documentation structure and may be outdated.
>
> For current navigation system documentation, see:
> - **Developer Guide**: [docs/navigation/developer-guide.md](../../navigation/developer-guide.md)
> - **Architecture Overview**: [docs/navigation.md](../../navigation.md)
> - **User Guide**: [docs/navigation/user-guide.md](../../navigation/user-guide.md)
>
> Last updated: 2025-12-03

---

Scarab's navigation system is built on Bevy's Entity Component System (ECS).

## Overview

The navigation system provides:
- Spatial and sequential pane navigation
- Focus management with history
- Tab management
- ECS-native design for performance and flexibility

## Design Documents

For comprehensive navigation system documentation, see:
- **[Navigation Developer Guide](../../navigation/developer-guide.md)** - Current developer documentation
- **[Navigation Architecture](../../navigation.md)** - Complete ECS-native architecture
- **[Navigation User Guide](../../navigation/user-guide.md)** - User-facing documentation

## Architecture

### Core Components

- **TabRoot**: Root entity for a tab
- **Pane**: Individual terminal pane entity
- **SplitNode**: Container for split layouts
- **FocusState**: Global resource tracking focused entities

### Systems

- **Focus Management**: Tracks and updates focused pane
- **Spatial Navigation**: Directional movement between panes
- **Layout Management**: Handles pane splits and resizing

## Implementation Phases

The navigation system was implemented in 7 phases:

0. **Scaffolding**: Basic ECS entities and components
1. **Focus Management**: Focus state and transitions
2. **Tab Management**: Create, delete, switch tabs
3. **Pane Operations**: Split, close, resize panes
4. **Spatial Navigation**: Up/down/left/right movement
5. **Integration**: Wire to IPC and input systems
6. **Testing**: Comprehensive test suite

Current status: **Phases 0-6 Complete**

## Code Locations

Navigation system code is in `crates/scarab-client/src/`:
- `navigation/` - Core navigation logic
- `ui/` - UI components and rendering
- `input/` - Input handling

## Development Workflow

### Running Tests

```bash
# Run all navigation tests
cargo test -p scarab-client --lib navigation

# Run specific test
cargo test -p scarab-client focus_management
```

### Adding New Features

1. Define ECS components/resources
2. Implement systems
3. Add tests
4. Update documentation

For detailed testing information, see the [Testing Guide](./testing.md).

## ECS Patterns

### Querying Entities

```rust
fn my_system(query: Query<&Pane, With<Focused>>) {
    for pane in query.iter() {
        // Process focused pane
    }
}
```

### Managing State

```rust
fn update_focus(mut focus: ResMut<FocusState>, new_pane: Entity) {
    focus.current = Some(new_pane);
    focus.history.push(new_pane);
}
```

## Performance Considerations

- ECS queries are cache-friendly
- Systems run in parallel when possible
- Minimal allocations in hot paths
- Event-driven updates

For architecture details, see the [Architecture Guide](./architecture.md).
