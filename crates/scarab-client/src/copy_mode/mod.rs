//! Copy Mode Bevy Integration
//!
//! This module integrates copy mode functionality with the Bevy game engine,
//! providing systems and resources for vim-like keyboard navigation and selection.

use bevy::prelude::*;
use scarab_plugin_api::copy_mode::{CopyModeState, SearchState};

/// Bevy plugin for copy mode functionality
pub struct CopyModePlugin;

impl Plugin for CopyModePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CopyModeStateResource>()
            .init_resource::<CopyModeSearchResource>()
            .add_systems(
                Update,
                (handle_copy_mode_actions, update_copy_mode_cursor)
                    .chain()
                    .run_if(copy_mode_active),
            );
    }
}

/// Bevy resource wrapping copy mode state
#[derive(Resource, Default)]
pub struct CopyModeStateResource {
    /// The underlying copy mode state
    pub state: CopyModeState,
}

impl CopyModeStateResource {
    /// Create a new copy mode state resource
    pub fn new() -> Self {
        Self {
            state: CopyModeState::new(),
        }
    }

    /// Check if copy mode is active
    pub fn is_active(&self) -> bool {
        self.state.active
    }
}

/// Bevy resource wrapping search state
#[derive(Resource, Default)]
pub struct CopyModeSearchResource {
    /// The underlying search state
    pub state: SearchState,
}

impl CopyModeSearchResource {
    /// Create a new search state resource
    pub fn new() -> Self {
        Self {
            state: SearchState::new(),
        }
    }

    /// Check if search is active
    pub fn is_active(&self) -> bool {
        self.state.active
    }
}

/// Marker component for the copy mode cursor entity
#[derive(Component)]
pub struct CopyModeCursorMarker;

/// Marker component for selection highlight entities
#[derive(Component)]
pub struct SelectionHighlight;

/// Run condition that checks if copy mode is active
pub fn copy_mode_active(state: Res<CopyModeStateResource>) -> bool {
    state.is_active()
}

/// System that handles copy mode action events
///
/// This system processes CopyModeAction events from the key table system
/// and updates the copy mode state accordingly. It handles navigation,
/// selection, clipboard operations, and mode transitions.
pub fn handle_copy_mode_actions(
    // TODO: Add event reader for KeyActionEvent
    // TODO: Add scrollback buffer resource
    // TODO: Add terminal metrics resource
    // TODO: Add clipboard context resource
    mut _copy_mode_state: ResMut<CopyModeStateResource>,
    mut _search_state: ResMut<CopyModeSearchResource>,
) {
    // Stub implementation - will be completed in later phases
    // This system will:
    // 1. Read KeyActionEvent events
    // 2. Filter for CopyModeAction variants
    // 3. Update cursor position based on navigation actions
    // 4. Handle selection toggle/update
    // 5. Extract and copy text to clipboard
    // 6. Handle search actions
}

/// System that updates the visual cursor position
///
/// This system syncs the visual cursor entity with the logical cursor position
/// from the copy mode state, accounting for viewport offset and terminal metrics.
pub fn update_copy_mode_cursor(
    // TODO: Add terminal metrics resource
    // TODO: Add query for cursor transform
    _copy_mode_state: Res<CopyModeStateResource>,
) {
    // Stub implementation - will be completed in later phases
    // This system will:
    // 1. Read the current cursor position from copy mode state
    // 2. Convert logical coordinates to screen space
    // 3. Update the transform of the cursor entity
    // 4. Account for viewport offset (scrolling)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_copy_mode_state_resource_creation() {
        let resource = CopyModeStateResource::new();
        assert!(!resource.is_active());
    }

    #[test]
    fn test_search_state_resource_creation() {
        let resource = CopyModeSearchResource::new();
        assert!(!resource.is_active());
    }

    #[test]
    fn test_copy_mode_state_activation() {
        let mut resource = CopyModeStateResource::new();
        assert!(!resource.is_active());

        resource
            .state
            .activate(scarab_plugin_api::copy_mode::CopyModeCursor::new(5, 10));
        assert!(resource.is_active());

        resource.state.deactivate();
        assert!(!resource.is_active());
    }

    #[test]
    fn test_copy_mode_active_run_condition() {
        let resource = CopyModeStateResource::new();
        assert!(!resource.is_active());

        // Test that the run condition returns false when inactive
        // Note: Testing the actual Bevy run condition requires a proper Bevy app
        // which is better suited for integration tests
    }
}
