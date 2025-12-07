//! Copy Mode Bevy Integration
//!
//! This module integrates copy mode functionality with the Bevy game engine,
//! providing systems and resources for vim-like keyboard navigation and selection.

use bevy::prelude::*;
use scarab_plugin_api::copy_mode::{
    copy_mode_indicator, copy_mode_position_indicator, find_matches, get_selection_bounds,
    search_match_indicator, CopyModeState, SearchDirection, SearchState, SelectionMode,
};
use scarab_plugin_api::key_tables::CopyModeAction;
use scarab_plugin_api::status_bar::RenderItem;

/// Bevy plugin for copy mode functionality
pub struct CopyModePlugin;

impl Plugin for CopyModePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CopyModeStateResource>()
            .init_resource::<CopyModeSearchResource>()
            .init_resource::<TerminalDimensions>()
            .add_event::<CopyModeActionEvent>()
            .add_event::<CopyModeIndicatorEvent>()
            .add_systems(Startup, spawn_copy_mode_cursor)
            .add_systems(
                Update,
                (
                    handle_copy_mode_actions,
                    update_cursor_visibility,
                    update_cursor_position,
                    render_selection_highlights,
                    render_search_highlights,
                    update_mode_indicator,
                )
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

/// Terminal dimensions for coordinate conversion
#[derive(Resource, Clone, Copy, Debug)]
pub struct TerminalDimensions {
    /// Number of columns in the terminal grid
    pub cols: u16,
    /// Number of rows in the terminal grid
    pub rows: u16,
    /// Width of a single cell in pixels
    pub cell_width: f32,
    /// Height of a single cell in pixels
    pub cell_height: f32,
    /// Minimum Y coordinate (scrollback top, typically negative)
    pub min_y: i32,
}

impl Default for TerminalDimensions {
    fn default() -> Self {
        Self {
            cols: 80,
            rows: 24,
            cell_width: 10.0,
            cell_height: 20.0,
            min_y: -1000, // Default scrollback of 1000 lines
        }
    }
}

impl TerminalDimensions {
    /// Convert logical grid coordinates to screen space position
    pub fn logical_to_screen(&self, x: u16, y: i32) -> Vec3 {
        Vec3::new(
            x as f32 * self.cell_width,
            y as f32 * self.cell_height,
            10.0, // Z layer for cursor/selection
        )
    }

    /// Get the maximum Y coordinate (bottom of visible screen)
    pub fn max_y(&self) -> i32 {
        self.rows as i32 - 1
    }
}

/// Marker component for the copy mode cursor entity
#[derive(Component)]
pub struct CopyModeCursorMarker;

/// Marker component for selection highlight entities
#[derive(Component)]
pub struct SelectionHighlight {
    /// The Y coordinate of this highlight
    pub y: i32,
}

/// Marker component for search highlight entities
#[derive(Component)]
pub struct SearchHighlight {
    /// The Y coordinate of this highlight
    pub y: i32,
    /// Whether this is the currently selected match
    pub is_current: bool,
}

/// Event for copy mode actions
#[derive(Event, Clone, Copy, Debug)]
pub struct CopyModeActionEvent {
    /// The action to perform
    pub action: CopyModeAction,
}

/// Event for copy mode indicator updates
///
/// This event is emitted whenever the copy mode state changes and
/// the status bar indicators need to be updated.
#[derive(Event, Clone, Debug)]
pub struct CopyModeIndicatorEvent {
    /// Mode indicator items (e.g., "COPY", "VISUAL")
    pub mode_items: Vec<RenderItem>,
    /// Position indicator items (e.g., "L10,C5")
    pub position_items: Vec<RenderItem>,
    /// Search match indicator items (e.g., "2/5")
    pub match_items: Vec<RenderItem>,
}

/// Run condition that checks if copy mode is active
pub fn copy_mode_active(state: Res<CopyModeStateResource>) -> bool {
    state.is_active()
}

/// System that spawns the copy mode cursor entity at startup
pub fn spawn_copy_mode_cursor(mut commands: Commands) {
    commands.spawn((
        CopyModeCursorMarker,
        Sprite {
            color: Color::srgba(0.0, 1.0, 0.0, 0.5), // Semi-transparent green
            custom_size: Some(Vec2::new(10.0, 20.0)),
            ..default()
        },
        Transform::from_translation(Vec3::new(0.0, 0.0, 10.0)),
        Visibility::Hidden, // Hidden by default until copy mode activates
    ));
}

/// System that updates cursor visibility based on copy mode state
pub fn update_cursor_visibility(
    copy_mode_state: Res<CopyModeStateResource>,
    mut cursor_query: Query<&mut Visibility, With<CopyModeCursorMarker>>,
) {
    if let Ok(mut visibility) = cursor_query.get_single_mut() {
        *visibility = if copy_mode_state.is_active() {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
}

/// System that updates the visual cursor position
pub fn update_cursor_position(
    copy_mode_state: Res<CopyModeStateResource>,
    terminal_dims: Res<TerminalDimensions>,
    mut cursor_query: Query<(&mut Transform, &mut Sprite), With<CopyModeCursorMarker>>,
) {
    if let Ok((mut transform, mut sprite)) = cursor_query.get_single_mut() {
        let cursor = copy_mode_state.state.cursor;

        // Convert logical coordinates to screen space
        transform.translation = terminal_dims.logical_to_screen(cursor.x, cursor.y);

        // Update sprite size based on terminal dimensions
        if let Some(ref mut size) = sprite.custom_size {
            size.x = terminal_dims.cell_width;
            size.y = terminal_dims.cell_height;
        }
    }
}

/// System that renders selection highlights
pub fn render_selection_highlights(
    copy_mode_state: Res<CopyModeStateResource>,
    terminal_dims: Res<TerminalDimensions>,
    mut commands: Commands,
    existing_highlights: Query<Entity, With<SelectionHighlight>>,
) {
    // Despawn all existing highlights
    for entity in existing_highlights.iter() {
        commands.entity(entity).despawn();
    }

    // Spawn new highlights if there's an active selection
    if let Some(ref selection) = copy_mode_state.state.selection {
        let (min_x, min_y, max_x, max_y) = get_selection_bounds(selection);

        match copy_mode_state.state.selection_mode {
            SelectionMode::None => {}
            SelectionMode::Cell => {
                // Character-by-character selection
                if min_y == max_y {
                    // Single line
                    let width = (max_x - min_x + 1) as f32 * terminal_dims.cell_width;
                    let pos = terminal_dims.logical_to_screen(min_x, min_y);

                    commands.spawn((
                        SelectionHighlight { y: min_y },
                        Sprite {
                            color: Color::srgba(0.0, 0.5, 1.0, 0.3), // Semi-transparent blue
                            custom_size: Some(Vec2::new(width, terminal_dims.cell_height)),
                            ..default()
                        },
                        Transform::from_translation(pos + Vec3::new(width / 2.0, 0.0, 9.0)),
                    ));
                } else {
                    // Multi-line selection
                    for y in min_y..=max_y {
                        let (start_x, end_x) = if y == min_y {
                            (min_x, terminal_dims.cols - 1)
                        } else if y == max_y {
                            (0, max_x)
                        } else {
                            (0, terminal_dims.cols - 1)
                        };

                        let width = (end_x - start_x + 1) as f32 * terminal_dims.cell_width;
                        let pos = terminal_dims.logical_to_screen(start_x, y);

                        commands.spawn((
                            SelectionHighlight { y },
                            Sprite {
                                color: Color::srgba(0.0, 0.5, 1.0, 0.3),
                                custom_size: Some(Vec2::new(width, terminal_dims.cell_height)),
                                ..default()
                            },
                            Transform::from_translation(pos + Vec3::new(width / 2.0, 0.0, 9.0)),
                        ));
                    }
                }
            }
            SelectionMode::Line => {
                // Whole line selection
                for y in min_y..=max_y {
                    let width = terminal_dims.cols as f32 * terminal_dims.cell_width;
                    let pos = terminal_dims.logical_to_screen(0, y);

                    commands.spawn((
                        SelectionHighlight { y },
                        Sprite {
                            color: Color::srgba(0.0, 0.5, 1.0, 0.3),
                            custom_size: Some(Vec2::new(width, terminal_dims.cell_height)),
                            ..default()
                        },
                        Transform::from_translation(pos + Vec3::new(width / 2.0, 0.0, 9.0)),
                    ));
                }
            }
            SelectionMode::Block => {
                // Rectangular block selection
                let width = (max_x - min_x + 1) as f32 * terminal_dims.cell_width;
                for y in min_y..=max_y {
                    let pos = terminal_dims.logical_to_screen(min_x, y);

                    commands.spawn((
                        SelectionHighlight { y },
                        Sprite {
                            color: Color::srgba(0.0, 0.5, 1.0, 0.3),
                            custom_size: Some(Vec2::new(width, terminal_dims.cell_height)),
                            ..default()
                        },
                        Transform::from_translation(pos + Vec3::new(width / 2.0, 0.0, 9.0)),
                    ));
                }
            }
            SelectionMode::Word => {
                // Word selection (similar to cell for now)
                let width = (max_x - min_x + 1) as f32 * terminal_dims.cell_width;
                let pos = terminal_dims.logical_to_screen(min_x, min_y);

                commands.spawn((
                    SelectionHighlight { y: min_y },
                    Sprite {
                        color: Color::srgba(0.0, 0.5, 1.0, 0.3),
                        custom_size: Some(Vec2::new(width, terminal_dims.cell_height)),
                        ..default()
                    },
                    Transform::from_translation(pos + Vec3::new(width / 2.0, 0.0, 9.0)),
                ));
            }
        }
    }
}

/// System that renders search match highlights
pub fn render_search_highlights(
    search_state: Res<CopyModeSearchResource>,
    terminal_dims: Res<TerminalDimensions>,
    mut commands: Commands,
    existing_highlights: Query<Entity, With<SearchHighlight>>,
) {
    // Despawn all existing search highlights
    for entity in existing_highlights.iter() {
        commands.entity(entity).despawn();
    }

    // Render search matches if search is active and there are matches
    if search_state.is_active() && !search_state.state.matches.is_empty() {
        let current_match_idx = search_state.state.current_match;

        for (idx, search_match) in search_state.state.matches.iter().enumerate() {
            let is_current = Some(idx) == current_match_idx;
            let start = search_match.start;
            let end = search_match.end;

            // Calculate width and position
            let width = (end.x - start.x + 1) as f32 * terminal_dims.cell_width;
            let pos = terminal_dims.logical_to_screen(start.x, start.y);

            // Different colors for current match vs other matches
            let color = if is_current {
                Color::srgba(1.0, 0.5, 0.0, 0.5) // Orange for current match
            } else {
                Color::srgba(1.0, 1.0, 0.0, 0.3) // Yellow for other matches
            };

            commands.spawn((
                SearchHighlight {
                    y: start.y,
                    is_current,
                },
                Sprite {
                    color,
                    custom_size: Some(Vec2::new(width, terminal_dims.cell_height)),
                    ..default()
                },
                Transform::from_translation(pos + Vec3::new(width / 2.0, 0.0, 8.0)), // Z=8 to render under selection
            ));
        }
    }
}

/// System that updates the mode indicator for the status bar
///
/// This system generates indicator events whenever the copy mode state changes.
/// The events can be consumed by the status bar system to update the display.
pub fn update_mode_indicator(
    copy_mode_state: Res<CopyModeStateResource>,
    search_state: Res<CopyModeSearchResource>,
    mut indicator_events: EventWriter<CopyModeIndicatorEvent>,
) {
    // Only emit indicator events if copy mode is active
    if !copy_mode_state.is_active() {
        return;
    }

    // Generate indicator items using the plugin API functions
    let mode_items = copy_mode_indicator(&copy_mode_state.state, search_state.is_active());
    let position_items = copy_mode_position_indicator(&copy_mode_state.state);
    let match_items = search_match_indicator(&search_state.state);

    // Emit the indicator event
    indicator_events.send(CopyModeIndicatorEvent {
        mode_items,
        position_items,
        match_items,
    });
}

/// System that handles copy mode action events
pub fn handle_copy_mode_actions(
    mut events: EventReader<CopyModeActionEvent>,
    mut copy_mode_state: ResMut<CopyModeStateResource>,
    mut search_state: ResMut<CopyModeSearchResource>,
    terminal_dims: Res<TerminalDimensions>,
    // TODO: Add scrollback buffer resource for text extraction
    // TODO: Add clipboard context resource for copying
) {
    for event in events.read() {
        let state = &mut copy_mode_state.state;

        match event.action {
            // Movement actions
            CopyModeAction::MoveLeft => {
                state.move_left();
                state.update_selection();
            }
            CopyModeAction::MoveRight => {
                state.move_right(terminal_dims.cols);
                state.update_selection();
            }
            CopyModeAction::MoveUp => {
                state.move_up(terminal_dims.min_y);
                state.update_selection();
            }
            CopyModeAction::MoveDown => {
                state.move_down(terminal_dims.max_y());
                state.update_selection();
            }
            CopyModeAction::MoveWordForward => {
                // TODO: Implement word-forward movement
                // For now, just move right
                state.move_right(terminal_dims.cols);
                state.update_selection();
            }
            CopyModeAction::MoveWordBackward => {
                // TODO: Implement word-backward movement
                // For now, just move left
                state.move_left();
                state.update_selection();
            }
            CopyModeAction::MoveToLineStart => {
                state.move_to_line_start();
                state.update_selection();
            }
            CopyModeAction::MoveToLineEnd => {
                // TODO: Get actual line length from buffer
                // For now, use terminal width
                state.move_to_line_end(terminal_dims.cols);
                state.update_selection();
            }
            CopyModeAction::MoveToTop => {
                state.move_to_top(terminal_dims.min_y);
                state.update_selection();
            }
            CopyModeAction::MoveToBottom => {
                state.move_to_bottom(terminal_dims.max_y());
                state.update_selection();
            }

            // Selection actions
            CopyModeAction::ToggleSelection => {
                state.toggle_cell_selection();
            }
            CopyModeAction::ToggleLineSelection => {
                state.toggle_line_selection();
            }
            CopyModeAction::ToggleBlockSelection => {
                state.toggle_block_selection();
            }

            // Search actions
            CopyModeAction::SearchForward => {
                search_state.state.start_search(SearchDirection::Forward);
                // TODO: Open search input UI
                // For now, perform a demo search
                let get_line = |_y: i32| Some("Example text with match".to_string());
                let matches = find_matches(
                    "match",
                    get_line,
                    terminal_dims.min_y,
                    terminal_dims.max_y(),
                );
                search_state
                    .state
                    .update_query("match".to_string(), matches);
            }
            CopyModeAction::SearchBackward => {
                search_state.state.start_search(SearchDirection::Backward);
                // TODO: Open search input UI
                // For now, perform a demo search
                let get_line = |_y: i32| Some("Example text with match".to_string());
                let matches = find_matches(
                    "match",
                    get_line,
                    terminal_dims.min_y,
                    terminal_dims.max_y(),
                );
                search_state
                    .state
                    .update_query("match".to_string(), matches);
            }
            CopyModeAction::NextMatch => {
                search_state.state.next_match();
                // Move cursor to current match if it exists
                if let Some(current_match) = search_state.state.current() {
                    state.cursor = current_match.start;
                }
            }
            CopyModeAction::PrevMatch => {
                search_state.state.prev_match();
                // Move cursor to current match if it exists
                if let Some(current_match) = search_state.state.current() {
                    state.cursor = current_match.start;
                }
            }

            // Copy and exit actions
            CopyModeAction::CopyAndExit => {
                // TODO: Extract text from scrollback buffer
                // For now, just demonstrate the API usage
                let _text = state.get_selection_text(|_y| {
                    // TODO: Get actual line from scrollback buffer
                    Some("Example line".to_string())
                });

                // TODO: Copy to clipboard using clipboard context
                // if let Some(text) = text {
                //     clipboard.set_text(text);
                // }

                // Exit copy mode and clear search
                state.deactivate();
                search_state.state.deactivate();
            }
            CopyModeAction::Exit => {
                state.deactivate();
                search_state.state.deactivate();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use scarab_plugin_api::copy_mode::CopyModeCursor;

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

        resource.state.activate(CopyModeCursor::new(5, 10));
        assert!(resource.is_active());

        resource.state.deactivate();
        assert!(!resource.is_active());
    }

    #[test]
    fn test_terminal_dimensions_logical_to_screen() {
        let dims = TerminalDimensions {
            cols: 80,
            rows: 24,
            cell_width: 10.0,
            cell_height: 20.0,
            min_y: -100,
        };

        let pos = dims.logical_to_screen(5, 10);
        assert_eq!(pos.x, 50.0);
        assert_eq!(pos.y, 200.0);
        assert_eq!(pos.z, 10.0);
    }

    #[test]
    fn test_terminal_dimensions_max_y() {
        let dims = TerminalDimensions {
            cols: 80,
            rows: 24,
            cell_width: 10.0,
            cell_height: 20.0,
            min_y: -100,
        };

        assert_eq!(dims.max_y(), 23);
    }

    #[test]
    fn test_copy_mode_action_event() {
        let event = CopyModeActionEvent {
            action: CopyModeAction::MoveLeft,
        };

        assert!(matches!(event.action, CopyModeAction::MoveLeft));
    }

    #[test]
    fn test_copy_mode_indicator_event() {
        let event = CopyModeIndicatorEvent {
            mode_items: vec![RenderItem::Text("COPY".to_string())],
            position_items: vec![RenderItem::Text("L1,C1".to_string())],
            match_items: vec![],
        };

        assert_eq!(event.mode_items.len(), 1);
        assert_eq!(event.position_items.len(), 1);
        assert_eq!(event.match_items.len(), 0);
    }
}
