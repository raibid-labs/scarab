# Headless Bevy Testing - Quick Start Guide

**For Scarab UI Developers**

This guide shows you how to write headless tests for Bevy UI components in Scarab.

---

## Why Headless Testing?

**Before:**
- Manual testing: `cargo run -p scarab-client` → visual inspection
- No automation
- Slow feedback loop
- Regression bugs slip through

**Now:**
- Automated: `cargo test` verifies UI
- Fast (< 1 second)
- CI-friendly (no GPU/display needed)
- Catch bugs before commit

---

## Running Tests

```bash
# Run all headless tests
cargo test -p scarab-client --test headless_poc

# Run with output
cargo test -p scarab-client --test headless_poc -- --nocapture

# Run specific test
cargo test -p scarab-client --test headless_poc test_query_basic_components
```

**Expected output:**
```
running 6 tests
test test_bevy_runs_headless ........... ok
test test_query_basic_components ....... ok
test test_mock_assets .................. ok
test test_scarab_ui_component .......... ok
test test_event_system_headless ........ ok
test test_multiple_update_cycles ....... ok

test result: ok. 6 passed; 0 failed; 0 ignored
finished in 0.01s
```

---

## Basic Test Structure

```rust
use bevy::prelude::*;

#[test]
fn test_my_ui_component() {
    // 1. Setup: Create headless Bevy app
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);

    // 2. Spawn: Create your UI component
    let entity = app.world_mut().spawn((
        MyComponent,
        Node {
            width: Val::Px(600.0),
            height: Val::Px(400.0),
            ..default()
        },
        Name::new("MyComponent"),
    )).id();

    // 3. Update: Run one frame
    app.update();

    // 4. Assert: Verify component state
    let world = app.world();
    let node = world.get::<Node>(entity).unwrap();
    assert_eq!(node.width, Val::Px(600.0));
}
```

---

## Common Patterns

### Pattern 1: Query Components

```rust
#[test]
fn test_query_by_marker() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);

    #[derive(Component)]
    struct MyMarker;

    // Spawn multiple entities
    app.world_mut().spawn((MyMarker, Node::default()));
    app.world_mut().spawn((MyMarker, Node::default()));
    app.update();

    // Query all entities with marker
    let mut query = app.world_mut().query_filtered::<Entity, With<MyMarker>>();
    let entities: Vec<Entity> = query.iter(app.world()).collect();

    assert_eq!(entities.len(), 2);
}
```

### Pattern 2: Send Events

```rust
#[test]
fn test_event_response() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);

    #[derive(Event)]
    struct ShowUIEvent;

    #[derive(Resource, Default)]
    struct UIState {
        visible: bool,
    }

    // Register event and system
    app.add_event::<ShowUIEvent>();
    app.init_resource::<UIState>();
    app.add_systems(Update, |
        mut events: EventReader<ShowUIEvent>,
        mut state: ResMut<UIState>
    | {
        for _ in events.read() {
            state.visible = true;
        }
    });

    // Send event
    app.world_mut().send_event(ShowUIEvent);
    app.update();

    // Verify state changed
    let state = app.world().resource::<UIState>();
    assert!(state.visible);
}
```

### Pattern 3: Parent-Child Entities

```rust
#[test]
fn test_child_entities() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);

    // Spawn parent with children
    let parent = app.world_mut().spawn(Node::default())
        .with_children(|parent| {
            parent.spawn((Text::new("Child 1"), Name::new("Child1")));
            parent.spawn((Text::new("Child 2"), Name::new("Child2")));
        }).id();

    app.update();

    // Query children
    let children = app.world().get::<Children>(parent).unwrap();
    assert_eq!(children.len(), 2);

    // Query by name
    let mut name_query = app.world_mut().query::<&Name>();
    let names: Vec<&str> = name_query.iter(app.world())
        .map(|n| n.as_str())
        .collect();
    assert!(names.contains(&"Child1"));
    assert!(names.contains(&"Child2"));
}
```

### Pattern 4: Mock Assets

```rust
#[test]
fn test_image_assets() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.init_resource::<Assets<Image>>();

    // Create mock image
    let mut images = app.world_mut().resource_mut::<Assets<Image>>();
    let handle = images.add(Image::default());

    // Spawn entity with image
    app.world_mut().spawn((
        ImageNode::new(handle.clone()),
        Node::default(),
    ));

    app.update();

    // Verify asset exists
    let images = app.world().resource::<Assets<Image>>();
    assert!(images.get(&handle).is_some());
}
```

---

## What You Can Test

✅ **Component spawning** - Does my UI create the right entities?
✅ **Layout properties** - Is the width/height correct?
✅ **State management** - Does state update on events?
✅ **Event handling** - Do systems respond to events?
✅ **Parent-child relationships** - Are children spawned correctly?
✅ **Component queries** - Can I find entities by marker?
✅ **Resource updates** - Do resources change as expected?

---

## What You Cannot Test

❌ **Visual output** - Cannot verify pixel rendering
❌ **Font rendering** - No cosmic-text layout
❌ **Asset loading from files** - No file I/O
❌ **GPU shaders** - No graphics context
❌ **Window events** - No window manager

**For these:** Use integration/E2E tests or manual testing.

---

## Tips

1. **Use MinimalPlugins** - Never use `DefaultPlugins` in headless tests
2. **One frame = one update()** - Call `app.update()` to process systems
3. **Query after update** - Systems run during `update()`, not during spawn
4. **Mock resources** - Create test resources manually
5. **Use marker components** - Easy to query for specific UI elements

---

## Example: Testing Command Palette

```rust
#[test]
fn test_command_palette_spawns() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);

    #[derive(Component)]
    struct CommandPaletteMarker;

    // Simulate opening palette
    let palette = app.world_mut().spawn((
        CommandPaletteMarker,
        Node {
            width: Val::Px(600.0),
            height: Val::Px(400.0),
            position_type: PositionType::Absolute,
            left: Val::Px(200.0),
            top: Val::Px(100.0),
            ..default()
        },
        BackgroundColor(Color::srgba(0.1, 0.1, 0.1, 0.95)),
    )).id();

    app.update();

    // Verify palette exists
    let mut query = app.world_mut().query_filtered::<Entity, With<CommandPaletteMarker>>();
    assert_eq!(query.iter(app.world()).count(), 1);

    // Verify properties
    let node = app.world().get::<Node>(palette).unwrap();
    assert_eq!(node.width, Val::Px(600.0));
    assert_eq!(node.position_type, PositionType::Absolute);
}
```

---

## Next Steps

1. **Read the POC:** `crates/scarab-client/tests/headless_poc.rs`
2. **Study the examples** in this guide
3. **Write tests** for your UI components
4. **Run tests** before committing: `cargo test -p scarab-client`

---

## Getting Help

- **POC Test File:** `/home/beengud/raibid-labs/scarab/crates/scarab-client/tests/headless_poc.rs`
- **Results Report:** `/home/beengud/raibid-labs/scarab/docs/testing/HEADLESS_POC_RESULTS.md`
- **Solutions Doc:** `/home/beengud/raibid-labs/scarab/docs/audits/claude-2025-12-01/05-FRONTEND-TESTING-SOLUTIONS.md`

---

**Happy Testing! 🧪**
