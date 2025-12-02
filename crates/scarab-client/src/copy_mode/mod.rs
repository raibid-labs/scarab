//! Copy Mode Bevy Integration
//!
//! This module integrates copy mode functionality with the Bevy game engine,
//! providing systems and resources for vim-like keyboard navigation and selection.

use bevy::prelude::*;
use scarab_plugin_api::copy_mode::{get_selection_bounds, CopyModeState, SearchState, SelectionMode};
use scarab_plugin_api::key_tables::CopyModeAction;

/// Bevy plugin for copy mode functionality
pub struct CopyModePlugin;

impl Plugin for CopyModePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CopyModeStateResource>()
            .init_resource::<CopyModeSearchResource>()
            .init_resource::<TerminalDimensions>()
            .add_event::<CopyModeActionEvent>()
            .add_systems(Startup, spawn_copy_mode_cursor)
            .add_systems(
                Update,
                (
                    handle_copy_mode_actions,
                    update_cursor_visibility,
                    update_cursor_position,
                    render_selection_highlights,
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

/// Event for copy mode actions
#[derive(Event, Clone, Copy, Debug)]
pub struct CopyModeActionEvent {
    /// The action to perform
    pub action: CopyModeAction,
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

/// System that handles copy mode action events
pub fn handle_copy_mode_actions(
    mut events: EventReader<CopyModeActionEvent>,
    mut copy_mode_state: ResMut<CopyModeStateResource>,
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

                // Exit copy mode
                state.deactivate();
            }
            CopyModeAction::Exit => {
                state.deactivate();
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
}
