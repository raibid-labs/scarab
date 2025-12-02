# HeadlessTestHarness for Scarab Client

A comprehensive testing utility for writing headless Bevy UI tests without requiring a GPU or display server.

## Features

- **Zero Dependencies on Display**: Runs using Bevy's `MinimalPlugins`
- **Mock Terminal Grid**: Full `MockSharedMemoryReader` for simulating terminal output
- **Rich API**: 20+ helper methods for spawning, querying, and asserting on components
- **Event Testing**: Full support for testing event-driven systems
- **Asset Mocking**: Pre-configured `Assets<Image>` for UI components
- **Sample Data Helpers**: Pre-built terminal output patterns for common test scenarios

## Quick Start

```rust
use crate::harness::HeadlessTestHarness;
use bevy::prelude::*;

#[test]
fn test_my_ui_component() {
    let mut harness = HeadlessTestHarness::new();

    // Spawn a UI component
    harness.spawn((
        Node {
            width: Val::Px(600.0),
            height: Val::Px(400.0),
            ..default()
        },
        Name::new("MyComponent"),
    ));

    harness.update();

    // Assert it exists
    harness.assert_component_exists::<Node>();
}
```

## API Overview

### Initialization

- `HeadlessTestHarness::new()` - Create new harness with default setup
- `HeadlessTestHarness::with_setup(|app| {...})` - Custom initialization

### Frame Management

- `update()` - Run one Bevy frame
- `update_n(count)` - Run multiple frames
- `send_event(event)` - Send event and process it

### Entity Management

- `spawn(bundle)` - Spawn entity with components
- `despawn(entity)` - Remove entity
- `query::<T>()` - Get all entities with component T
- `query_components::<T>()` - Get all T components
- `query_filtered::<D, F>()` - Complex filtered queries

### Assertions

- `assert_component_exists::<T>()` - At least one T exists
- `assert_component_count::<T>(n)` - Exactly n entities with T
- `assert_resource_exists::<R>()` - Resource R exists

### Resources

- `resource::<R>()` - Get resource
- `resource_mut::<R>()` - Get mutable resource

### World Access

- `world()` - Immutable world reference
- `world_mut()` - Mutable world reference
- `app` - Direct access to Bevy App (public field)

### UI Helpers

- `get_node_size::<T>()` - Get (width, height) for first entity with marker T
- `add_system(schedule, system)` - Add system to schedule
- `add_plugin(plugin)` - Add plugin
- `entity_count()` - Total entity count

## Mock Terminal Grid

The `MockSharedMemoryReader` simulates terminal output for testing UI that responds to terminal state.

### Basic Operations

```rust
let mut harness = HeadlessTestHarness::new();
let mut reader = harness.resource_mut::<MockSharedMemoryReader>();

// Set single cell
reader.set_cell(5, 10, 'X', 0xFFFFFFFF, 0x000000FF);

// Set text string
reader.set_text(0, 0, "Hello, World!", 0xFFFFFFFF, 0x000000FF);

// Set cursor position
reader.set_cursor(15, 6);

// Increment sequence number (simulates daemon update)
reader.tick();

// Read back
assert_eq!(reader.get_char(5, 10), Some('X'));
assert_eq!(reader.get_row_text(0), "Hello, World!");
```

### Advanced Operations

```rust
// Fill rectangle
reader.fill_rect(10, 5, 20, 3, '#', 0xFFFFFFFF, 0x000000FF);

// Simulate multi-line output
reader.simulate_output(&[
    (0, 0, "user@host:~$"),
    (0, 1, "ls -la"),
    (0, 2, "total 42"),
], 0xFFFFFFFF, 0x000000FF);

// Clear grid
reader.clear();
```

### Sample Data Helpers

Pre-built terminal states for common scenarios:

```rust
use crate::harness::mocks::*;

// Sample shell session
let state = sample_terminal_output();
harness.app.insert_resource(MockSharedMemoryReader::with_state(state));

// URLs for link hint testing
let state = sample_url_output();

// Colored output (errors, warnings, etc.)
let state = sample_colored_output();
```

## Testing Patterns

### Testing Event-Driven Systems

```rust
#[derive(Event)]
struct ToggleEvent;

#[test]
fn test_toggle_system() {
    let mut harness = HeadlessTestHarness::new();

    harness.app.add_event::<ToggleEvent>();
    harness.add_system(Update, my_toggle_system);

    // Send event and verify behavior
    harness.send_event(ToggleEvent);

    // Assert state changed
    let state = harness.resource::<MyState>();
    assert!(state.toggled);
}
```

### Testing Hierarchical UI

```rust
#[test]
fn test_parent_child_ui() {
    let mut harness = HeadlessTestHarness::new();

    let parent = harness.spawn(Node::default());

    harness.world_mut().entity_mut(parent).with_children(|parent| {
        parent.spawn(Node::default());
        parent.spawn(Node::default());
    });

    harness.update();

    let children = harness.world().get::<Children>(parent).unwrap();
    assert_eq!(children.len(), 2);
}
```

### Testing with Terminal Output

```rust
#[test]
fn test_url_detection() {
    let mut harness = HeadlessTestHarness::new();

    let mut reader = harness.resource_mut::<MockSharedMemoryReader>();
    reader.set_text(0, 0, "Visit https://github.com", 0xFFFFFFFF, 0x000000FF);
    reader.tick();

    // Test your URL detection system here
    harness.add_system(Update, detect_urls_system);
    harness.update();

    // Assert URLs were detected
    let urls = harness.resource::<DetectedUrls>();
    assert_eq!(urls.0.len(), 1);
}
```

### Testing Animations

```rust
#[test]
fn test_fade_in_animation() {
    let mut harness = HeadlessTestHarness::new();

    #[derive(Component)]
    struct Opacity(f32);

    harness.spawn(Opacity(0.0));
    harness.add_system(Update, fade_in_system);

    // Run 10 frames
    harness.update_n(10);

    let entities = harness.query::<Opacity>();
    let opacity = harness.get::<Opacity>(entities[0]).unwrap();
    assert!(opacity.0 > 0.5);
}
```

## Test Coverage

The harness itself is thoroughly tested with:

- **7 harness tests** in `mod.rs`
- **10 mock tests** in `mocks.rs`
- **15 standalone tests** in `harness_standalone.rs`
- **15 example tests** in `harness_examples.rs`

**Total: 36 tests** covering all harness functionality

## File Structure

```
crates/scarab-client/tests/harness/
├── mod.rs              # Main HeadlessTestHarness implementation
├── mocks.rs            # MockSharedMemoryReader and sample data
└── README.md           # This file
```

## Success Metrics

- All 36 harness tests pass
- All 6 original POC tests pass
- Tests run in < 2 seconds
- Zero GPU/display dependencies
- No unsafe code

## Next Steps

1. Write tests for existing UI components (command palette, link hints, etc.)
2. Add snapshot testing for ECS state
3. Create visual regression test framework (future work)
4. Document testing best practices in CONTRIBUTING.md

## Notes

- The harness uses `MinimalPlugins` which excludes windowing and rendering
- UI layout calculations work, but actual text rendering requires GPU
- Event simulation works, but windowing events (mouse, keyboard) must be programmatically created
- Asset loading from files is not supported (use programmatic asset creation)

## Related Files

- `/home/beengud/raibid-labs/scarab/crates/scarab-client/tests/headless_poc.rs` - Original POC
- `/home/beengud/raibid-labs/scarab/crates/scarab-client/tests/harness_standalone.rs` - Standalone tests
- `/home/beengud/raibid-labs/scarab/crates/scarab-client/tests/harness_examples.rs` - Usage examples
