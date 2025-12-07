//! Standalone tests for HeadlessTestHarness
//!
//! These tests verify the harness itself works correctly,
//! without depending on scarab-client library code.

mod harness;

use bevy::prelude::*;
use harness::{mocks::*, HeadlessTestHarness};

/// Test: Harness initializes correctly
#[test]
fn test_harness_initialization() {
    let harness = HeadlessTestHarness::new();

    // Assets<Image> should be initialized
    harness.assert_resource_exists::<Assets<Image>>();

    // MockSharedMemoryReader should be initialized
    harness.assert_resource_exists::<MockSharedMemoryReader>();
}

/// Test: Can spawn and query components
#[test]
fn test_spawn_and_query() {
    let mut harness = HeadlessTestHarness::new();

    #[derive(Component)]
    struct CustomMarker;

    // Spawn entities
    harness.spawn(Node::default());
    harness.spawn((Node::default(), CustomMarker));
    harness.update();

    // Query them back
    let nodes = harness.query::<Node>();
    assert_eq!(nodes.len(), 2);

    let markers = harness.query::<CustomMarker>();
    assert_eq!(markers.len(), 1);
}

/// Test: Component assertions work
#[test]
fn test_component_assertions() {
    let mut harness = HeadlessTestHarness::new();

    harness.spawn(Node::default());
    harness.spawn(Node::default());
    harness.update();

    harness.assert_component_exists::<Node>();
    harness.assert_component_count::<Node>(2);
}

/// Test: Event system works
#[test]
fn test_event_system() {
    let mut harness = HeadlessTestHarness::new();

    #[derive(Event)]
    struct TestEvent;

    #[derive(Resource, Default)]
    struct EventCounter(u32);

    fn count_events(mut events: EventReader<TestEvent>, mut counter: ResMut<EventCounter>) {
        counter.0 += events.read().count() as u32;
    }

    harness.app.add_event::<TestEvent>();
    harness.app.init_resource::<EventCounter>();
    harness.add_system(Update, count_events);

    // Send events
    harness.send_event(TestEvent);
    harness.send_event(TestEvent);
    harness.send_event(TestEvent);

    let counter = harness.resource::<EventCounter>();
    assert_eq!(counter.0, 3);
}

/// Test: Node size extraction works
#[test]
fn test_node_size() {
    let mut harness = HeadlessTestHarness::new();

    #[derive(Component)]
    struct TestMarker;

    harness.spawn((
        TestMarker,
        Node {
            width: Val::Px(600.0),
            height: Val::Px(400.0),
            ..default()
        },
    ));
    harness.update();

    let size = harness.get_node_size::<TestMarker>();
    assert_eq!(size, Some((600.0, 400.0)));
}

/// Test: Multiple update cycles work
#[test]
fn test_multiple_updates() {
    let mut harness = HeadlessTestHarness::new();

    #[derive(Resource)]
    struct Counter(u32);

    fn increment(mut counter: ResMut<Counter>) {
        counter.0 += 1;
    }

    harness.app.insert_resource(Counter(0));
    harness.add_system(Update, increment);

    harness.update_n(5);

    let counter = harness.resource::<Counter>();
    assert_eq!(counter.0, 5);
}

/// Test: Custom setup works
#[test]
fn test_custom_setup() {
    #[derive(Resource, Default)]
    struct CustomResource;

    let harness = HeadlessTestHarness::with_setup(|app| {
        app.init_resource::<CustomResource>();
    });

    harness.assert_resource_exists::<CustomResource>();
}

/// Test: MockSharedMemoryReader basic operations
#[test]
fn test_mock_reader_basic() {
    let mut harness = HeadlessTestHarness::new();

    let mut reader = harness.resource_mut::<MockSharedMemoryReader>();
    reader.set_cell(5, 10, 'X', 0xFF0000FF, 0x000000FF);

    let reader = harness.resource::<MockSharedMemoryReader>();
    assert_eq!(reader.get_char(5, 10), Some('X'));
}

/// Test: MockSharedMemoryReader text operations
#[test]
fn test_mock_reader_text() {
    let mut harness = HeadlessTestHarness::new();

    let mut reader = harness.resource_mut::<MockSharedMemoryReader>();
    reader.set_text(0, 0, "Hello, World!", 0xFFFFFFFF, 0x000000FF);

    let reader = harness.resource::<MockSharedMemoryReader>();
    assert_eq!(reader.get_row_text(0), "Hello, World!");
}

/// Test: MockSharedMemoryReader sequence number
#[test]
fn test_mock_reader_sequence() {
    let mut harness = HeadlessTestHarness::new();

    let mut reader = harness.resource_mut::<MockSharedMemoryReader>();
    assert_eq!(reader.sequence_number(), 0);

    reader.tick();
    assert_eq!(reader.sequence_number(), 1);
}

/// Test: Sample terminal output helper
#[test]
fn test_sample_terminal_output() {
    let sample_state = sample_terminal_output();
    let mut harness = HeadlessTestHarness::new();
    harness
        .app
        .insert_resource(MockSharedMemoryReader::with_state(sample_state));

    let reader = harness.resource::<MockSharedMemoryReader>();
    assert!(reader.get_row_text(0).contains("user@scarab"));
}

/// Test: Sample URL output helper
#[test]
fn test_sample_url_output() {
    let url_state = sample_url_output();
    let mut harness = HeadlessTestHarness::new();
    harness
        .app
        .insert_resource(MockSharedMemoryReader::with_state(url_state));

    let reader = harness.resource::<MockSharedMemoryReader>();
    assert!(reader.get_row_text(0).contains("https://github.com"));
}

/// Test: Sample colored output helper
#[test]
fn test_sample_colored_output() {
    let colored_state = sample_colored_output();
    let mut harness = HeadlessTestHarness::new();
    harness
        .app
        .insert_resource(MockSharedMemoryReader::with_state(colored_state));

    let reader = harness.resource::<MockSharedMemoryReader>();
    assert!(reader.get_row_text(0).contains("ERROR"));
    assert!(reader.get_row_text(1).contains("SUCCESS"));
}

/// Test: Hierarchical UI structures
#[test]
fn test_hierarchical_ui() {
    let mut harness = HeadlessTestHarness::new();

    let parent = harness.spawn((
        Node {
            width: Val::Px(600.0),
            height: Val::Px(400.0),
            flex_direction: FlexDirection::Column,
            ..default()
        },
        Name::new("Parent"),
    ));

    harness
        .world_mut()
        .entity_mut(parent)
        .with_children(|parent| {
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

    let children = harness.world().get::<Children>(parent).unwrap();
    assert_eq!(children.len(), 2);

    harness.assert_component_count::<Node>(3);
}

/// Test: Asset handles
#[test]
fn test_asset_handles() {
    let mut harness = HeadlessTestHarness::new();

    let mut images = harness.resource_mut::<Assets<Image>>();
    let handle = images.add(Image::default());

    harness.spawn((
        ImageNode::new(handle.clone()),
        Node {
            width: Val::Px(100.0),
            height: Val::Px(100.0),
            ..default()
        },
    ));

    harness.update();

    harness.assert_component_exists::<ImageNode>();

    let images = harness.resource::<Assets<Image>>();
    assert!(images.get(&handle).is_some());
}

/// Test: Despawn entities
#[test]
fn test_despawn() {
    let mut harness = HeadlessTestHarness::new();

    #[derive(Component)]
    struct TempMarker;

    let entity = harness.spawn(TempMarker);
    harness.update();
    harness.assert_component_count::<TempMarker>(1);

    harness.despawn(entity);
    harness.update();
    harness.assert_component_count::<TempMarker>(0);
}
