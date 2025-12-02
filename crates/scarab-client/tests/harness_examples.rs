//! Example tests using HeadlessTestHarness
//!
//! This file demonstrates how to use the HeadlessTestHarness for testing
//! Scarab UI components. These examples serve as both tests and documentation.

mod harness;

use bevy::prelude::*;
use harness::{mocks::*, HeadlessTestHarness};

/// Example 1: Testing basic UI component spawning
///
/// This test demonstrates how to spawn a simple UI component and verify
/// its existence using the harness.
#[test]
fn example_spawn_simple_ui_component() {
    let mut harness = HeadlessTestHarness::new();

    // Spawn a basic UI node
    harness.spawn((
        Node {
            width: Val::Px(600.0),
            height: Val::Px(400.0),
            position_type: PositionType::Absolute,
            ..default()
        },
        BackgroundColor(Color::srgba(0.1, 0.1, 0.1, 0.95)),
        Name::new("TestPanel"),
    ));

    harness.update();

    // Verify the component exists
    harness.assert_component_exists::<Node>();
    harness.assert_component_count::<Node>(1);
}

/// Example 2: Testing event-driven UI systems
///
/// This test shows how to test systems that respond to events.
#[test]
fn example_event_driven_system() {
    let mut harness = HeadlessTestHarness::new();

    // Define custom events and state
    #[derive(Event)]
    struct ToggleVisibilityEvent;

    #[derive(Component)]
    struct VisibilityToggle {
        visible: bool,
    }

    #[derive(Resource, Default)]
    struct ToggleState {
        toggle_count: u32,
    }

    // Add event and resource
    harness.app.add_event::<ToggleVisibilityEvent>();
    harness.app.init_resource::<ToggleState>();

    // Add system that handles the event
    fn handle_toggle(
        mut events: EventReader<ToggleVisibilityEvent>,
        mut query: Query<&mut VisibilityToggle>,
        mut state: ResMut<ToggleState>,
    ) {
        for _ in events.read() {
            for mut toggle in query.iter_mut() {
                toggle.visible = !toggle.visible;
                state.toggle_count += 1;
            }
        }
    }

    harness.add_system(Update, handle_toggle);

    // Spawn entity with toggle component
    harness.spawn(VisibilityToggle { visible: true });
    harness.update();

    // Send toggle event
    harness.send_event(ToggleVisibilityEvent);

    // Verify state changed
    let state = harness.resource::<ToggleState>();
    assert_eq!(state.toggle_count, 1);

    let entities = harness.query::<VisibilityToggle>();
    let toggle = harness.get::<VisibilityToggle>(entities[0]).unwrap();
    assert!(!toggle.visible); // Should be toggled off
}

/// Example 3: Testing hierarchical UI structures
///
/// This demonstrates testing parent-child relationships in the UI.
#[test]
fn example_hierarchical_ui() {
    let mut harness = HeadlessTestHarness::new();

    // Spawn parent with children
    let parent = harness.spawn((
        Node {
            width: Val::Px(600.0),
            height: Val::Px(400.0),
            flex_direction: FlexDirection::Column,
            ..default()
        },
        Name::new("Parent"),
    ));

    // Add children using world_mut
    harness.world_mut().entity_mut(parent).with_children(|parent| {
        parent.spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Px(50.0),
                ..default()
            },
            Name::new("Child1"),
        ));

        parent.spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Px(50.0),
                ..default()
            },
            Name::new("Child2"),
        ));
    });

    harness.update();

    // Verify hierarchy
    let children = harness.world().get::<Children>(parent).unwrap();
    assert_eq!(children.len(), 2);

    // Verify total node count
    harness.assert_component_count::<Node>(3); // 1 parent + 2 children
}

/// Example 4: Testing with mock terminal data
///
/// This shows how to use MockSharedMemoryReader to test UI that responds
/// to terminal output.
#[test]
fn example_mock_terminal_data() {
    let mut harness = HeadlessTestHarness::new();

    // Get the mock reader and populate it with test data
    let mut reader = harness.resource_mut::<MockSharedMemoryReader>();
    reader.set_text(0, 0, "ERROR: Connection failed", 0xFF0000FF, 0x000000FF);
    reader.set_text(0, 1, "Retrying...", 0xFFFF00FF, 0x000000FF);
    reader.tick(); // Increment sequence number

    // Verify the data is accessible
    let reader = harness.resource::<MockSharedMemoryReader>();
    assert_eq!(reader.get_row_text(0), "ERROR: Connection failed");
    assert_eq!(reader.get_row_text(1), "Retrying...");
    assert_eq!(reader.sequence_number(), 1);
}

/// Example 5: Testing node size calculations
///
/// This demonstrates how to verify UI layout calculations.
#[test]
fn example_node_size_calculation() {
    let mut harness = HeadlessTestHarness::new();

    #[derive(Component)]
    struct CommandPalette;

    harness.spawn((
        CommandPalette,
        Node {
            width: Val::Px(600.0),
            height: Val::Px(400.0),
            ..default()
        },
    ));

    harness.update();

    // Verify dimensions
    let size = harness.get_node_size::<CommandPalette>();
    assert_eq!(size, Some((600.0, 400.0)));
}

/// Example 6: Testing with sample terminal output
///
/// This shows how to use the pre-defined sample data helpers.
#[test]
fn example_sample_terminal_output() {
    let sample_state = sample_terminal_output();

    let mut harness = HeadlessTestHarness::new();
    harness.app.insert_resource(MockSharedMemoryReader::with_state(sample_state));

    let reader = harness.resource::<MockSharedMemoryReader>();
    let first_line = reader.get_row_text(0);

    // Verify sample data is present
    assert!(first_line.contains("user@scarab"));
    assert!(first_line.contains("ls -la"));
}

/// Example 7: Testing URL detection (for link hints)
///
/// This demonstrates testing components that search for URLs in terminal output.
#[test]
fn example_url_detection() {
    let url_state = sample_url_output();

    let mut harness = HeadlessTestHarness::new();
    harness.app.insert_resource(MockSharedMemoryReader::with_state(url_state));

    let reader = harness.resource::<MockSharedMemoryReader>();

    // Verify URLs are present in the grid
    assert!(reader.get_row_text(0).contains("https://github.com"));
    assert!(reader.get_row_text(1).contains("https://docs.rs"));
}

/// Example 8: Testing multi-frame animations
///
/// This shows how to test systems that update over multiple frames.
#[test]
fn example_multi_frame_animation() {
    let mut harness = HeadlessTestHarness::new();

    #[derive(Component)]
    struct AnimatedOpacity {
        value: f32,
        target: f32,
        speed: f32,
    }

    fn animate_opacity(mut query: Query<&mut AnimatedOpacity>) {
        for mut opacity in query.iter_mut() {
            if opacity.value < opacity.target {
                opacity.value = (opacity.value + opacity.speed).min(opacity.target);
            }
        }
    }

    harness.add_system(Update, animate_opacity);

    // Spawn animated component
    harness.spawn(AnimatedOpacity {
        value: 0.0,
        target: 1.0,
        speed: 0.1,
    });

    harness.update();

    // Run 10 frames
    harness.update_n(10);

    // Verify animation completed
    let entities = harness.query::<AnimatedOpacity>();
    let opacity = harness.get::<AnimatedOpacity>(entities[0]).unwrap();
    assert_eq!(opacity.value, 1.0);
}

/// Example 9: Testing resource modifications
///
/// This demonstrates testing systems that modify resources.
#[test]
fn example_resource_modification() {
    let mut harness = HeadlessTestHarness::new();

    #[derive(Resource)]
    struct Counter(u32);

    fn increment_counter(mut counter: ResMut<Counter>) {
        counter.0 += 1;
    }

    harness.app.insert_resource(Counter(0));
    harness.add_system(Update, increment_counter);

    // Run multiple frames
    for expected in 1..=5 {
        harness.update();
        let counter = harness.resource::<Counter>();
        assert_eq!(counter.0, expected);
    }
}

/// Example 10: Testing component cleanup
///
/// This shows how to verify entities are properly despawned.
#[test]
fn example_component_cleanup() {
    let mut harness = HeadlessTestHarness::new();

    #[derive(Component)]
    struct TemporaryMarker;

    // Spawn entities
    let entity1 = harness.spawn(TemporaryMarker);
    let entity2 = harness.spawn(TemporaryMarker);
    harness.update();

    harness.assert_component_count::<TemporaryMarker>(2);

    // Despawn one
    harness.despawn(entity1);
    harness.update();

    harness.assert_component_count::<TemporaryMarker>(1);

    // Despawn the other
    harness.despawn(entity2);
    harness.update();

    harness.assert_component_count::<TemporaryMarker>(0);
}

/// Example 11: Testing with custom setup
///
/// This demonstrates using the `with_setup` constructor for complex initialization.
#[test]
fn example_custom_setup() {
    #[derive(Resource)]
    struct CustomConfig {
        theme: String,
    }

    impl Default for CustomConfig {
        fn default() -> Self {
            Self {
                theme: "dark".to_string(),
            }
        }
    }

    let harness = HeadlessTestHarness::with_setup(|app| {
        app.init_resource::<CustomConfig>();
    });

    let config = harness.resource::<CustomConfig>();
    assert_eq!(config.theme, "dark");
}

/// Example 12: Testing colored terminal output
///
/// This demonstrates verifying ANSI color handling.
#[test]
fn example_colored_output() {
    let colored_state = sample_colored_output();

    let mut harness = HeadlessTestHarness::new();
    harness.app.insert_resource(MockSharedMemoryReader::with_state(colored_state));

    let reader = harness.resource::<MockSharedMemoryReader>();

    // Verify different colored lines
    assert!(reader.get_row_text(0).contains("ERROR"));
    assert!(reader.get_row_text(1).contains("SUCCESS"));
    assert!(reader.get_row_text(2).contains("WARNING"));
    assert!(reader.get_row_text(3).contains("INFO"));
}

/// Example 13: Testing entity queries with filters
///
/// This shows how to use complex queries with the harness.
#[test]
fn example_filtered_queries() {
    let mut harness = HeadlessTestHarness::new();

    #[derive(Component)]
    struct Active;

    #[derive(Component)]
    struct Inactive;

    // Spawn active and inactive entities
    harness.spawn((Node::default(), Active));
    harness.spawn((Node::default(), Active));
    harness.spawn((Node::default(), Inactive));
    harness.update();

    // Query only active nodes
    let active_nodes = harness.query_filtered::<Entity, (With<Node>, With<Active>)>();
    assert_eq!(active_nodes.len(), 2);

    // Query only inactive nodes
    let inactive_nodes = harness.query_filtered::<Entity, (With<Node>, With<Inactive>)>();
    assert_eq!(inactive_nodes.len(), 1);
}

/// Example 14: Testing mock terminal grid manipulation
///
/// This demonstrates advanced mock grid operations.
#[test]
fn example_mock_grid_manipulation() {
    let mut harness = HeadlessTestHarness::new();

    let mut reader = harness.resource_mut::<MockSharedMemoryReader>();

    // Fill a rectangular region
    reader.fill_rect(10, 5, 20, 3, '#', 0xFFFFFFFF, 0x000000FF);

    // Set cursor position
    reader.set_cursor(15, 6);

    // Simulate output
    reader.simulate_output(
        &[
            (0, 0, "Line 1"),
            (0, 1, "Line 2"),
            (0, 2, "Line 3"),
        ],
        0xFFFFFFFF,
        0x000000FF,
    );

    let reader = harness.resource::<MockSharedMemoryReader>();
    let state = reader.get_state();

    // Verify cursor position
    assert_eq!(state.cursor_x, 15);
    assert_eq!(state.cursor_y, 6);

    // Verify filled rectangle
    assert_eq!(reader.get_char(10, 5), Some('#'));
    assert_eq!(reader.get_char(29, 7), Some('#'));

    // Verify simulated output
    assert_eq!(reader.get_row_text(0), "Line 1");
}

/// Example 15: Testing asset handles
///
/// This shows how to work with assets in headless tests.
#[test]
fn example_asset_handles() {
    let mut harness = HeadlessTestHarness::new();

    // Create a mock image
    let mut images = harness.resource_mut::<Assets<Image>>();
    let handle = images.add(Image::default());

    // Spawn entity with image handle
    harness.spawn((
        ImageNode::new(handle.clone()),
        Node {
            width: Val::Px(100.0),
            height: Val::Px(100.0),
            ..default()
        },
    ));

    harness.update();

    // Verify the image node exists
    harness.assert_component_exists::<ImageNode>();

    // Verify the asset is in storage
    let images = harness.resource::<Assets<Image>>();
    assert!(images.get(&handle).is_some());
}
