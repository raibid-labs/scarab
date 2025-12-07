//! Headless Test Harness for Scarab Client
//!
//! This module provides a reusable test harness for writing headless Bevy UI tests.
//! It eliminates boilerplate and provides a clean API for testing UI components
//! without requiring a GPU or display server.
//!
//! ## Features
//!
//! - **Minimal Setup**: Automatically configures MinimalPlugins
//! - **Event Handling**: Send and process events with a single call
//! - **Component Queries**: Query entities by component type
//! - **Assertions**: Built-in assertions for common test patterns
//! - **Mock Support**: Includes mock SharedState for terminal grid simulation
//!
//! ## Example Usage
//!
//! ```rust,no_run
//! use crate::harness::HeadlessTestHarness;
//! use bevy::prelude::*;
//!
//! #[test]
//! fn test_my_ui_component() {
//!     let mut harness = HeadlessTestHarness::new();
//!
//!     // Spawn a component
//!     harness.world_mut().spawn(Node::default());
//!     harness.update();
//!
//!     // Assert it exists
//!     harness.assert_component_exists::<Node>();
//! }
//! ```

use bevy::ecs::query::{QueryData, QueryFilter};
use bevy::ecs::schedule::ScheduleLabel;
use bevy::prelude::*;

pub mod headless;
pub mod mocks;

// Re-export HeadlessHarness for convenience
pub use headless::HeadlessHarness;

/// A reusable test harness for headless Bevy UI testing.
///
/// This struct wraps a Bevy `App` instance configured with `MinimalPlugins`
/// and provides helper methods for common testing operations.
///
/// ## Lifecycle
///
/// 1. Create with `HeadlessTestHarness::new()`
/// 2. Set up test state using `world_mut()`
/// 3. Call `update()` to run one frame
/// 4. Query and assert using helper methods
///
/// ## Thread Safety
///
/// The harness is not `Send` or `Sync` because Bevy's `World` is not thread-safe.
/// Each test should create its own harness instance.
pub struct HeadlessTestHarness {
    pub app: App,
}

impl HeadlessTestHarness {
    /// Create a new headless test environment.
    ///
    /// This initializes a Bevy `App` with:
    /// - `MinimalPlugins` (no window, no GPU)
    /// - `Assets<Image>` resource for UI components
    /// - Mock `SharedMemoryReader` for terminal grid simulation
    ///
    /// ## Example
    ///
    /// ```rust,no_run
    /// let mut harness = HeadlessTestHarness::new();
    /// harness.update(); // Run one frame
    /// ```
    pub fn new() -> Self {
        let mut app = App::new();

        // Use MinimalPlugins for headless operation
        app.add_plugins(MinimalPlugins);

        // Initialize Assets<Image> for UI components that use images
        app.init_resource::<Assets<Image>>();

        // Add mock SharedMemoryReader for terminal grid simulation
        app.insert_resource(mocks::MockSharedMemoryReader::default());

        Self { app }
    }

    /// Create a new headless test environment with custom setup.
    ///
    /// This allows you to add custom plugins or resources before running tests.
    ///
    /// ## Example
    ///
    /// ```rust,no_run
    /// let mut harness = HeadlessTestHarness::with_setup(|app| {
    ///     app.add_plugins(MyCustomPlugin);
    ///     app.init_resource::<MyCustomResource>();
    /// });
    /// ```
    pub fn with_setup<F>(setup: F) -> Self
    where
        F: FnOnce(&mut App),
    {
        let mut harness = Self::new();
        setup(&mut harness.app);
        harness
    }

    /// Run one Bevy update cycle.
    ///
    /// This executes all scheduled systems for one frame. Call this after
    /// spawning entities or sending events to process them.
    ///
    /// ## Example
    ///
    /// ```rust,no_run
    /// harness.world_mut().spawn(Node::default());
    /// harness.update(); // Process spawned entity
    /// ```
    pub fn update(&mut self) {
        self.app.update();
    }

    /// Run multiple Bevy update cycles.
    ///
    /// Useful for testing systems that require multiple frames to complete.
    ///
    /// ## Example
    ///
    /// ```rust,no_run
    /// harness.update_n(10); // Run 10 frames
    /// ```
    pub fn update_n(&mut self, count: usize) {
        for _ in 0..count {
            self.app.update();
        }
    }

    /// Send an event and process it immediately.
    ///
    /// This is a convenience method that combines sending an event with
    /// running an update cycle.
    ///
    /// ## Example
    ///
    /// ```rust,no_run
    /// #[derive(Event)]
    /// struct MyEvent;
    ///
    /// harness.send_event(MyEvent);
    /// // Event has been sent and processed
    /// ```
    pub fn send_event<E: Event>(&mut self, event: E) {
        self.app.world_mut().send_event(event);
        self.app.update();
    }

    /// Query all entities with a specific component.
    ///
    /// Returns a vector of entity IDs that have the specified component.
    ///
    /// ## Example
    ///
    /// ```rust,no_run
    /// let entities = harness.query::<Node>();
    /// assert_eq!(entities.len(), 3);
    /// ```
    pub fn query<T: Component>(&mut self) -> Vec<Entity> {
        let mut query = self.app.world_mut().query_filtered::<Entity, With<T>>();
        query.iter(self.app.world()).collect()
    }

    /// Query entities with a component and return component references.
    ///
    /// Returns a vector of references to the specified component.
    ///
    /// ## Example
    ///
    /// ```rust,no_run
    /// let nodes = harness.query_components::<Node>();
    /// for node in nodes {
    ///     println!("Width: {:?}", node.width);
    /// }
    /// ```
    pub fn query_components<T: Component>(&mut self) -> Vec<&T> {
        let mut query = self.app.world_mut().query::<&T>();
        query.iter(self.app.world()).collect()
    }

    /// Query entities matching a complex filter.
    ///
    /// This provides access to Bevy's powerful query system for complex
    /// component filtering.
    ///
    /// ## Example
    ///
    /// ```rust,no_run
    /// let entities = harness.query_filtered::<Entity, (With<Node>, Without<Transform>)>();
    /// ```
    pub fn query_filtered<D: QueryData, F: QueryFilter>(
        &mut self,
    ) -> Vec<<<D as QueryData>::ReadOnly as bevy::ecs::query::WorldQuery>::Item<'_>> {
        let mut query = self.app.world_mut().query_filtered::<D, F>();
        query.iter(self.app.world()).collect()
    }

    /// Assert that at least one entity has the specified component.
    ///
    /// Panics if no entities with the component are found.
    ///
    /// ## Example
    ///
    /// ```rust,no_run
    /// harness.assert_component_exists::<Node>();
    /// ```
    pub fn assert_component_exists<T: Component>(&mut self) {
        let count = self.query::<T>().len();
        assert!(
            count > 0,
            "Expected at least one entity with component {}",
            std::any::type_name::<T>()
        );
    }

    /// Assert the exact count of entities with a component.
    ///
    /// Panics if the count doesn't match the expected value.
    ///
    /// ## Example
    ///
    /// ```rust,no_run
    /// harness.assert_component_count::<Node>(5);
    /// ```
    pub fn assert_component_count<T: Component>(&mut self, expected: usize) {
        let actual = self.query::<T>().len();
        assert_eq!(
            actual,
            expected,
            "Expected {} entities with component {}, found {}",
            expected,
            std::any::type_name::<T>(),
            actual
        );
    }

    /// Get the Node component for the first entity with a specific marker component.
    ///
    /// Returns `None` if no entity with the marker component has a Node.
    ///
    /// ## Example
    ///
    /// ```rust,no_run
    /// #[derive(Component)]
    /// struct CommandPalette;
    ///
    /// if let Some((width, height)) = harness.get_node_size::<CommandPalette>() {
    ///     assert_eq!(width, 600.0);
    /// }
    /// ```
    pub fn get_node_size<T: Component>(&mut self) -> Option<(f32, f32)> {
        let entities = self.query::<T>();
        if entities.is_empty() {
            return None;
        }

        let entity = entities[0];
        let node = self.app.world().get::<Node>(entity)?;

        let width = match node.width {
            Val::Px(px) => px,
            Val::Percent(pct) => pct, // In tests, percent values are returned as-is
            _ => 0.0,
        };

        let height = match node.height {
            Val::Px(px) => px,
            Val::Percent(pct) => pct,
            _ => 0.0,
        };

        Some((width, height))
    }

    /// Get mutable reference to the Bevy world.
    ///
    /// Use this for advanced operations like spawning entities,
    /// inserting resources, or querying complex component combinations.
    ///
    /// ## Example
    ///
    /// ```rust,no_run
    /// harness.world_mut().spawn((
    ///     Node::default(),
    ///     Transform::default(),
    /// ));
    /// ```
    pub fn world_mut(&mut self) -> &mut World {
        self.app.world_mut()
    }

    /// Get immutable reference to the Bevy world.
    ///
    /// Use this for read-only queries and assertions.
    ///
    /// ## Example
    ///
    /// ```rust,no_run
    /// let entity = harness.query::<Node>()[0];
    /// let node = harness.world().get::<Node>(entity).unwrap();
    /// ```
    pub fn world(&self) -> &World {
        self.app.world()
    }

    /// Get a reference to a resource.
    ///
    /// Panics if the resource doesn't exist.
    ///
    /// ## Example
    ///
    /// ```rust,no_run
    /// let images = harness.resource::<Assets<Image>>();
    /// ```
    pub fn resource<R: Resource>(&self) -> &R {
        self.app.world().resource::<R>()
    }

    /// Get a mutable reference to a resource.
    ///
    /// Panics if the resource doesn't exist.
    ///
    /// ## Example
    ///
    /// ```rust,no_run
    /// let mut images = harness.resource_mut::<Assets<Image>>();
    /// images.add(Image::default());
    /// ```
    pub fn resource_mut<R: Resource>(&mut self) -> Mut<'_, R> {
        self.app.world_mut().resource_mut::<R>()
    }

    /// Spawn an entity with components.
    ///
    /// This is a convenience wrapper around `world_mut().spawn()`.
    ///
    /// ## Example
    ///
    /// ```rust,no_run
    /// let entity = harness.spawn((Node::default(), Transform::default()));
    /// ```
    pub fn spawn<B: Bundle>(&mut self, bundle: B) -> Entity {
        self.app.world_mut().spawn(bundle).id()
    }

    /// Despawn an entity.
    ///
    /// Returns true if the entity was successfully despawned.
    ///
    /// ## Example
    ///
    /// ```rust,no_run
    /// let entity = harness.spawn(Node::default());
    /// harness.despawn(entity);
    /// ```
    pub fn despawn(&mut self, entity: Entity) -> bool {
        self.app.world_mut().despawn(entity)
    }

    /// Get a component from an entity.
    ///
    /// Returns `None` if the entity doesn't have the component.
    ///
    /// ## Example
    ///
    /// ```rust,no_run
    /// let entity = harness.spawn(Node::default());
    /// let node = harness.get::<Node>(entity).unwrap();
    /// ```
    pub fn get<T: Component>(&self, entity: Entity) -> Option<&T> {
        self.app.world().get::<T>(entity)
    }

    /// Add a system to run during update cycles.
    ///
    /// This is useful for testing system behavior.
    ///
    /// ## Example
    ///
    /// ```rust,no_run
    /// fn my_system(query: Query<&Node>) {
    ///     // System logic
    /// }
    ///
    /// harness.add_system(Update, my_system);
    /// harness.update(); // my_system runs
    /// ```
    pub fn add_system<M>(
        &mut self,
        schedule: impl ScheduleLabel,
        system: impl IntoSystemConfigs<M>,
    ) {
        self.app.add_systems(schedule, system);
    }

    /// Add a plugin to the test app.
    ///
    /// This allows testing with real plugins.
    ///
    /// ## Example
    ///
    /// ```rust,no_run
    /// harness.add_plugin(MyCustomPlugin);
    /// harness.update();
    /// ```
    pub fn add_plugin<P: Plugin>(&mut self, plugin: P) {
        self.app.add_plugins(plugin);
    }

    /// Assert that a resource exists.
    ///
    /// Panics if the resource is not present.
    ///
    /// ## Example
    ///
    /// ```rust,no_run
    /// harness.assert_resource_exists::<Assets<Image>>();
    /// ```
    pub fn assert_resource_exists<R: Resource>(&self) {
        assert!(
            self.app.world().contains_resource::<R>(),
            "Expected resource {} to exist",
            std::any::type_name::<R>()
        );
    }

    /// Get the number of entities in the world.
    ///
    /// Useful for sanity checks and memory leak detection.
    ///
    /// ## Example
    ///
    /// ```rust,no_run
    /// let initial_count = harness.entity_count();
    /// harness.spawn(Node::default());
    /// assert_eq!(harness.entity_count(), initial_count + 1);
    /// ```
    pub fn entity_count(&self) -> usize {
        self.app.world().entities().len() as usize
    }
}

impl Default for HeadlessTestHarness {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Test: Harness initializes correctly
    #[test]
    fn test_harness_initialization() {
        let harness = HeadlessTestHarness::new();

        // Assets<Image> should be initialized
        harness.assert_resource_exists::<Assets<Image>>();

        // MockSharedMemoryReader should be initialized
        harness.assert_resource_exists::<mocks::MockSharedMemoryReader>();
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
}
