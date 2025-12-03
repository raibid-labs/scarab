//! ECS-Native Navigation Module for Scarab Terminal Emulator
//!
//! This module provides a comprehensive keyboard-first navigation system built
//! natively on Bevy's ECS architecture. Features:
//!
//! - **Multiple Navigation Modes**: Normal, Hints, Insert, CommandPalette
//! - **Focus Management**: Track and navigate between focusable elements
//! - **Hint Labels**: Vimium-style hints for quick keyboard navigation
//! - **Navigation Groups**: Logical grouping of related navigation targets
//! - **Action System**: Extensible actions for different navigation contexts
//! - **History Tracking**: Navigate back through focus history
//! - **Mode Stack**: Push/pop navigation modes for complex workflows
//! - **Focusable Detection**: Automatic scanning of terminal content for URLs, paths, emails
//!
//! ## Architecture
//!
//! The navigation system is designed to be:
//! - **ECS-native**: Uses components, resources, and events throughout
//! - **Composable**: Different systems can add focusable elements
//! - **Extensible**: Easy to add new navigation actions and modes
//! - **Type-safe**: Strong typing for navigation targets and actions
//!
//! ## Usage Example
//!
//! ```rust,ignore
//! use bevy::prelude::*;
//! use scarab_client::navigation::*;
//!
//! fn spawn_focusable_link(mut commands: Commands) {
//!     commands.spawn((
//!         NavFocus,
//!         NavHint {
//!             label: "aa".to_string(),
//!             position: Vec2::new(100.0, 200.0),
//!             action: NavAction::Open("https://example.com".to_string()),
//!         },
//!         NavGroup("links".to_string()),
//!     ));
//! }
//! ```

use bevy::prelude::*;

// ==================== Sub-modules ====================

pub mod focusable;
pub mod metrics;

// Re-export metrics types
pub use metrics::{NavMetrics, NavMetricsPlugin, NavMetricsReport};

// ==================== Navigation Modes ====================

/// Current navigation mode
///
/// The navigation mode determines how input is interpreted and which
/// navigation features are active.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum NavMode {
    /// Normal terminal mode - standard input handling
    #[default]
    Normal,

    /// Hint mode - display hint labels for quick navigation
    /// Similar to Vimium's link hints feature
    Hints,

    /// Insert mode - text input is passed to terminal
    Insert,

    /// Command palette mode - fuzzy search commands
    CommandPalette,
}

// ==================== Components ====================

/// Marks an entity as the current focus target
///
/// Only one entity should have this component at a time. Systems can query
/// for this component to find the currently focused navigation target.
///
/// # Example
/// ```rust,ignore
/// fn highlight_focused(query: Query<&Transform, With<NavFocus>>) {
///     if let Ok(transform) = query.get_single() {
///         // Draw focus indicator at transform position
///     }
/// }
/// ```
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct NavFocus;

/// Navigation hint component for focusable elements
///
/// Stores the hint label (e.g., "aa", "ab"), screen position, and the action
/// to perform when this hint is activated.
///
/// # Example
/// ```rust,ignore
/// commands.spawn((
///     NavHint {
///         label: "ab".to_string(),
///         position: Vec2::new(150.0, 300.0),
///         action: NavAction::JumpPrompt(42),
///     },
/// ));
/// ```
#[derive(Component, Debug, Clone, PartialEq)]
pub struct NavHint {
    /// Hint label to display (e.g., "aa", "ab", "ac")
    pub label: String,

    /// Screen position for hint label rendering
    pub position: Vec2,

    /// Action to perform when this hint is activated
    pub action: NavAction,
}

/// Groups related navigation targets together
///
/// Navigation groups allow logical organization of focusable elements.
/// For example, all links in a prompt block could share the same group,
/// or all panes in a tab could be grouped together.
///
/// # Example
/// ```rust,ignore
/// // Group all links from a specific command output
/// commands.spawn((
///     NavFocus,
///     NavHint { /* ... */ },
///     NavGroup("prompt_42_links".to_string()),
/// ));
/// ```
#[derive(Component, Debug, Clone, PartialEq, Eq, Hash)]
pub struct NavGroup(pub String);

// ==================== Navigation Actions ====================

/// Actions that can be triggered by navigation
///
/// Each action represents a specific navigation operation. Actions are
/// triggered when a hint is activated or when a keyboard shortcut is pressed.
#[derive(Debug, Clone, PartialEq)]
pub enum NavAction {
    /// Open a URL or file path
    Open(String),

    /// Click at a specific grid position (col, row)
    Click(u16, u16),

    /// Jump to a specific prompt marker by line number
    JumpPrompt(u32),

    /// Navigate to the next pane in the current tab
    NextPane,

    /// Navigate to the previous pane in the current tab
    PrevPane,

    /// Navigate to the next tab in the current window
    NextTab,

    /// Navigate to the previous tab in the current window
    PrevTab,

    /// Cancel the current navigation operation
    Cancel,
}

// ==================== Navigation State Resource ====================

/// Global navigation state resource
///
/// This resource tracks the current navigation mode, mode history stack,
/// focus history, and hint filtering input. It serves as the central
/// coordination point for the navigation system.
#[derive(Resource, Debug, Clone)]
pub struct NavState {
    /// Current active navigation mode
    pub current_mode: NavMode,

    /// Stack of previous modes (for push/pop navigation)
    /// Allows returning to previous modes after completing an operation
    pub mode_stack: Vec<NavMode>,

    /// History of previously focused entities
    /// Enables "go back" navigation similar to browser history
    pub focus_history: Vec<Entity>,

    /// Current hint filter input (user typing to filter hints)
    /// In hint mode, users can type characters to filter visible hints
    pub hint_filter: String,

    /// Maximum focus history size to prevent unbounded growth
    pub max_history_size: usize,
}

impl Default for NavState {
    fn default() -> Self {
        Self {
            current_mode: NavMode::Normal,
            mode_stack: Vec::new(),
            focus_history: Vec::new(),
            hint_filter: String::new(),
            max_history_size: 50,
        }
    }
}

impl NavState {
    /// Push the current mode onto the stack and enter a new mode
    ///
    /// # Example
    /// ```rust,ignore
    /// nav_state.push_mode(NavMode::Hints);
    /// // Do navigation in hint mode
    /// nav_state.pop_mode(); // Returns to previous mode
    /// ```
    pub fn push_mode(&mut self, new_mode: NavMode) {
        self.mode_stack.push(self.current_mode);
        self.current_mode = new_mode;
    }

    /// Pop the previous mode from the stack
    ///
    /// Returns `true` if a mode was popped, `false` if stack was empty.
    pub fn pop_mode(&mut self) -> bool {
        if let Some(previous_mode) = self.mode_stack.pop() {
            self.current_mode = previous_mode;
            true
        } else {
            false
        }
    }

    /// Record a focus change in history
    ///
    /// Maintains a circular buffer of focus history, automatically
    /// dropping the oldest entries when the max size is reached.
    pub fn record_focus(&mut self, entity: Entity) {
        self.focus_history.push(entity);

        // Trim history if it exceeds max size
        if self.focus_history.len() > self.max_history_size {
            self.focus_history.remove(0);
        }
    }

    /// Get the previous focus from history (if any)
    ///
    /// Returns the most recent focus entity before the current one.
    pub fn previous_focus(&self) -> Option<Entity> {
        if self.focus_history.len() >= 2 {
            // Return second-to-last (current focus is last)
            self.focus_history.get(self.focus_history.len() - 2).copied()
        } else {
            None
        }
    }

    /// Clear the hint filter input
    pub fn clear_hint_filter(&mut self) {
        self.hint_filter.clear();
    }

    /// Check if currently in hint mode
    pub fn is_hint_mode(&self) -> bool {
        self.current_mode == NavMode::Hints
    }

    /// Check if currently in insert mode
    pub fn is_insert_mode(&self) -> bool {
        self.current_mode == NavMode::Insert
    }

    /// Check if currently in normal mode
    pub fn is_normal_mode(&self) -> bool {
        self.current_mode == NavMode::Normal
    }

    /// Check if currently in command palette mode
    pub fn is_command_palette_mode(&self) -> bool {
        self.current_mode == NavMode::CommandPalette
    }
}

// ==================== Navigation Events ====================

/// Event fired when entering hint mode
///
/// Systems can listen for this event to prepare hint labels,
/// pause normal input handling, etc.
#[derive(Event, Debug, Clone, Copy)]
pub struct EnterHintModeEvent;

/// Event fired when exiting hint mode
///
/// Systems should clean up hint labels and resume normal
/// input handling when receiving this event.
#[derive(Event, Debug, Clone, Copy)]
pub struct ExitHintModeEvent;

/// Event fired when a navigation action is triggered
///
/// This is the primary event for navigation operations. Systems
/// should listen for this event and handle their respective actions.
///
/// # Example
/// ```rust,ignore
/// fn handle_nav_actions(mut events: EventReader<NavActionEvent>) {
///     for event in events.read() {
///         match &event.action {
///             NavAction::Open(url) => open_url(url),
///             NavAction::NextPane => switch_to_next_pane(),
///             _ => {}
///         }
///     }
/// }
/// ```
#[derive(Event, Debug, Clone)]
pub struct NavActionEvent {
    /// The navigation action to perform
    pub action: NavAction,

    /// Optional entity that triggered the action
    pub source: Option<Entity>,

    /// Timestamp of the action for debugging/analytics
    pub timestamp: std::time::Instant,
}

impl NavActionEvent {
    /// Create a new navigation action event
    pub fn new(action: NavAction) -> Self {
        Self {
            action,
            source: None,
            timestamp: std::time::Instant::now(),
        }
    }

    /// Create a navigation action event with a source entity
    pub fn with_source(action: NavAction, source: Entity) -> Self {
        Self {
            action,
            source: Some(source),
            timestamp: std::time::Instant::now(),
        }
    }
}

/// Event fired when focus changes between entities
///
/// Allows systems to respond to focus changes, such as scrolling
/// the focused element into view or updating visual indicators.
#[derive(Event, Debug, Clone, Copy)]
pub struct FocusChangedEvent {
    /// Previously focused entity (if any)
    pub old_focus: Option<Entity>,

    /// Newly focused entity
    pub new_focus: Entity,
}

// ==================== System Set for Ordering ====================

/// System set for navigation systems
///
/// Provides ordering guarantees for navigation systems. Systems can
/// be added to this set to ensure they run in the correct phase.
///
/// # Example
/// ```rust,ignore
/// app.add_systems(Update, (
///     update_hint_positions.in_set(NavSystemSet),
///     render_hint_labels.in_set(NavSystemSet).after(update_hint_positions),
/// ));
/// ```
#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NavSystemSet {
    /// Input processing phase - handle keyboard/mouse input
    Input,

    /// Update phase - update navigation state and components
    Update,

    /// Render phase - render hint labels and focus indicators
    Render,
}

// ==================== Navigation Plugin ====================

/// Plugin that adds the navigation system to a Bevy app
///
/// This plugin registers all navigation components, resources, events,
/// and system sets. Add this plugin to your app to enable the navigation
/// system.
///
/// # Example
/// ```rust,ignore
/// App::new()
///     .add_plugins(DefaultPlugins)
///     .add_plugins(NavigationPlugin)
///     .run();
/// ```
pub struct NavigationPlugin;

impl Plugin for NavigationPlugin {
    fn build(&self, app: &mut App) {
        app
            // Register navigation state resource
            .init_resource::<NavState>()

            // Register navigation events
            .add_event::<EnterHintModeEvent>()
            .add_event::<ExitHintModeEvent>()
            .add_event::<NavActionEvent>()
            .add_event::<FocusChangedEvent>()

            // Configure system sets for proper ordering
            .configure_sets(
                Update,
                (
                    NavSystemSet::Input,
                    NavSystemSet::Update,
                    NavSystemSet::Render,
                )
                    .chain(), // Run in order: Input -> Update -> Render
            );

        // Note: Actual navigation systems will be added by other plugins
        // that depend on this navigation plugin. This keeps the core
        // navigation module focused on types and infrastructure.
    }
}

// ==================== Re-exports ====================

// Re-export focusable types for convenience
pub use focusable::{
    FocusablePlugin,
    FocusableRegion,
    FocusableType,
    FocusableSource,
    FocusableScanConfig,
};

// Re-export for tests
#[cfg(test)]
pub(crate) use focusable::FocusableDetector;

#[cfg(test)]
mod basic_tests {
    use super::*;

    #[test]
    fn test_nav_state_default() {
        let state = NavState::default();
        assert_eq!(state.current_mode, NavMode::Normal);
        assert!(state.mode_stack.is_empty());
        assert!(state.focus_history.is_empty());
        assert!(state.hint_filter.is_empty());
    }

    #[test]
    fn test_nav_state_push_pop() {
        let mut state = NavState::default();

        // Push hint mode
        state.push_mode(NavMode::Hints);
        assert_eq!(state.current_mode, NavMode::Hints);
        assert_eq!(state.mode_stack.len(), 1);

        // Push command palette mode
        state.push_mode(NavMode::CommandPalette);
        assert_eq!(state.current_mode, NavMode::CommandPalette);
        assert_eq!(state.mode_stack.len(), 2);

        // Pop back to hints
        assert!(state.pop_mode());
        assert_eq!(state.current_mode, NavMode::Hints);
        assert_eq!(state.mode_stack.len(), 1);

        // Pop back to normal
        assert!(state.pop_mode());
        assert_eq!(state.current_mode, NavMode::Normal);
        assert_eq!(state.mode_stack.len(), 0);

        // Pop with empty stack
        assert!(!state.pop_mode());
        assert_eq!(state.current_mode, NavMode::Normal);
    }

    #[test]
    fn test_focus_history() {
        let mut state = NavState::default();
        let entity1 = Entity::from_raw(1);
        let entity2 = Entity::from_raw(2);
        let entity3 = Entity::from_raw(3);

        // Record first focus
        state.record_focus(entity1);
        assert_eq!(state.previous_focus(), None);

        // Record second focus
        state.record_focus(entity2);
        assert_eq!(state.previous_focus(), Some(entity1));

        // Record third focus
        state.record_focus(entity3);
        assert_eq!(state.previous_focus(), Some(entity2));
    }

    #[test]
    fn test_focus_history_limit() {
        let mut state = NavState::default();
        state.max_history_size = 3;

        // Fill history beyond max size
        for i in 0..10 {
            state.record_focus(Entity::from_raw(i));
        }

        // Should only keep last 3
        assert_eq!(state.focus_history.len(), 3);
        assert_eq!(state.focus_history[0], Entity::from_raw(7));
        assert_eq!(state.focus_history[1], Entity::from_raw(8));
        assert_eq!(state.focus_history[2], Entity::from_raw(9));
    }

    #[test]
    fn test_mode_checks() {
        let mut state = NavState::default();

        assert!(state.is_normal_mode());
        assert!(!state.is_hint_mode());
        assert!(!state.is_insert_mode());
        assert!(!state.is_command_palette_mode());

        state.current_mode = NavMode::Hints;
        assert!(!state.is_normal_mode());
        assert!(state.is_hint_mode());

        state.current_mode = NavMode::Insert;
        assert!(state.is_insert_mode());

        state.current_mode = NavMode::CommandPalette;
        assert!(state.is_command_palette_mode());
    }

    #[test]
    fn test_nav_action_event_creation() {
        let action = NavAction::NextPane;
        let event = NavActionEvent::new(action.clone());

        assert_eq!(event.action, action);
        assert!(event.source.is_none());

        let entity = Entity::from_raw(42);
        let event_with_source = NavActionEvent::with_source(action.clone(), entity);

        assert_eq!(event_with_source.action, action);
        assert_eq!(event_with_source.source, Some(entity));
    }
}

// Comprehensive navigation tests module
#[cfg(test)]
mod tests;
