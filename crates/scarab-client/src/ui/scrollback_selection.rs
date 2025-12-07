// Extended selection system for scrollback buffer
// Allows text selection in both live view and scrollback history

use crate::terminal::scrollback::{ScrollbackBuffer, ScrollbackState};
use crate::ui::visual_selection::{SelectionMode, SelectionRegion};
use arboard::Clipboard;
use bevy::prelude::*;

/// Extended selection state that works with scrollback
#[derive(Resource)]
pub struct ScrollbackSelectionState {
    pub active: bool,
    pub mode: SelectionMode,
    pub region: SelectionRegion,
    /// Whether selection is in scrollback (vs live view)
    pub in_scrollback: bool,
    /// Start line in scrollback buffer (if in_scrollback is true)
    pub scrollback_start_line: usize,
    /// End line in scrollback buffer (if in_scrollback is true)
    pub scrollback_end_line: usize,
}

impl Default for ScrollbackSelectionState {
    fn default() -> Self {
        Self {
            active: false,
            mode: SelectionMode::Character,
            region: SelectionRegion::default(),
            in_scrollback: false,
            scrollback_start_line: 0,
            scrollback_end_line: 0,
        }
    }
}

impl ScrollbackSelectionState {
    /// Start selection in scrollback
    pub fn start_scrollback_selection(&mut self, line: usize, col: u16, mode: SelectionMode) {
        self.active = true;
        self.in_scrollback = true;
        self.mode = mode;
        self.scrollback_start_line = line;
        self.scrollback_end_line = line;
        self.region = SelectionRegion::new(col, 0, col, 0);
    }

    /// Update selection endpoint
    pub fn update_scrollback_selection(&mut self, line: usize, col: u16) {
        if !self.active || !self.in_scrollback {
            return;
        }

        self.scrollback_end_line = line;
        self.region.end_x = col;
    }

    /// Clear selection
    pub fn clear(&mut self) {
        self.active = false;
        self.in_scrollback = false;
        self.region = SelectionRegion::default();
    }
}

/// Event for mouse-based selection in scrollback
#[derive(Event)]
pub struct ScrollbackMouseSelection {
    pub start_line: usize,
    pub start_col: u16,
    pub end_line: usize,
    pub end_col: u16,
}

/// System to handle mouse selection in scrollback
fn handle_mouse_selection(
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    mut selection: ResMut<ScrollbackSelectionState>,
    scrollback: Res<ScrollbackBuffer>,
    scrollback_state: Res<ScrollbackState>,
    renderer: Option<Res<crate::rendering::text::TextRenderer>>,
) {
    if !scrollback_state.is_scrolled {
        return; // Only handle mouse selection when in scrollback view
    }

    // Need renderer for coordinate conversion
    let Some(renderer) = renderer else {
        return;
    };

    let window = windows.single();

    // Start selection on mouse press
    if mouse_buttons.just_pressed(MouseButton::Left) {
        if let Some(cursor_pos) = window.cursor_position() {
            // Convert cursor position to scrollback line/column
            if let Some((grid_col, grid_row)) = cursor_to_scrollback_coords(
                cursor_pos,
                &scrollback,
                renderer.cell_width,
                renderer.cell_height,
                window.width(),
                window.height(),
            ) {
                let line = grid_row;
                let col = grid_col as u16;

                selection.start_scrollback_selection(line, col, SelectionMode::Character);
                debug!("Started scrollback selection at line {}, col {}", line, col);
            }
        }
    }

    // Update selection while dragging
    if mouse_buttons.pressed(MouseButton::Left) && selection.active {
        if let Some(cursor_pos) = window.cursor_position() {
            if let Some((grid_col, grid_row)) = cursor_to_scrollback_coords(
                cursor_pos,
                &scrollback,
                renderer.cell_width,
                renderer.cell_height,
                window.width(),
                window.height(),
            ) {
                let line = grid_row;
                let col = grid_col as u16;

                selection.update_scrollback_selection(line, col);
            }
        }
    }

    // End selection on release
    if mouse_buttons.just_released(MouseButton::Left) && selection.active {
        debug!("Ended scrollback selection");
    }
}

/// Convert cursor position to scrollback buffer coordinates
///
/// # Arguments
/// * `cursor_pos` - Window cursor position (origin bottom-left, Y up)
/// * `scrollback` - Scrollback buffer containing scroll offset
/// * `cell_width` - Width of a single cell in pixels
/// * `cell_height` - Height of a single cell in pixels
/// * `window_width` - Window width in pixels
/// * `window_height` - Window height in pixels
///
/// # Returns
/// Option<(col, line)> where line is absolute scrollback buffer index
fn cursor_to_scrollback_coords(
    cursor_pos: Vec2,
    scrollback: &ScrollbackBuffer,
    cell_width: f32,
    cell_height: f32,
    window_width: f32,
    window_height: f32,
) -> Option<(usize, usize)> {
    use scarab_protocol::{GRID_HEIGHT, GRID_WIDTH};

    // Convert window coordinates to Bevy screen space
    // Bevy cursor position is relative to bottom-left, Y up
    // We need to convert to centered coordinate system used by grid rendering
    let grid_width = GRID_WIDTH as f32;
    let grid_height = GRID_HEIGHT as f32;

    // Calculate grid pixel dimensions
    let grid_pixel_width = grid_width * cell_width;
    let grid_pixel_height = grid_height * cell_height;

    // Grid is centered in window, calculate start position
    let grid_start_x = (window_width - grid_pixel_width) / 2.0;
    let grid_start_y = (window_height - grid_pixel_height) / 2.0;

    // Convert cursor to grid-relative coordinates
    let grid_rel_x = cursor_pos.x - grid_start_x;
    let grid_rel_y = cursor_pos.y - grid_start_y;

    // Check if cursor is within grid bounds
    if grid_rel_x < 0.0
        || grid_rel_x > grid_pixel_width
        || grid_rel_y < 0.0
        || grid_rel_y > grid_pixel_height
    {
        return None;
    }

    // Convert to grid coordinates (column, visible row)
    let col = (grid_rel_x / cell_width).floor() as usize;
    let visible_row = (grid_rel_y / cell_height).floor() as usize;

    // Clamp to grid bounds
    let col = col.min(GRID_WIDTH - 1);
    let visible_row = visible_row.min(GRID_HEIGHT - 1);

    // Convert visible row to scrollback buffer line index
    // scroll_offset = 0 means at bottom (live view)
    // scroll_offset > 0 means scrolled up by that many lines
    let scroll_offset = scrollback.scroll_offset();
    let total_lines = scrollback.line_count();

    // Calculate absolute line in scrollback buffer
    // When scrolled, visible_row 0 corresponds to (total_lines - scroll_offset)
    let scrollback_line = if scroll_offset > 0 {
        // In scrollback: map visible row to buffer line
        total_lines
            .saturating_sub(scroll_offset)
            .saturating_add(visible_row)
    } else {
        // At bottom (live view): should not happen since we only call this when scrolled
        // but handle it anyway by mapping to most recent lines
        total_lines
            .saturating_sub(GRID_HEIGHT)
            .saturating_add(visible_row)
    };

    // Ensure line is within scrollback buffer bounds
    if scrollback_line >= total_lines {
        return None;
    }

    Some((col, scrollback_line))
}

/// System to copy scrollback selection to clipboard
fn copy_scrollback_selection(
    keys: Res<ButtonInput<KeyCode>>,
    selection: Res<ScrollbackSelectionState>,
    scrollback: Res<ScrollbackBuffer>,
) {
    if !selection.active || !selection.in_scrollback {
        return;
    }

    // Ctrl+C or 'y' to copy
    let should_copy = (keys.pressed(KeyCode::ControlLeft) || keys.pressed(KeyCode::ControlRight))
        && keys.just_pressed(KeyCode::KeyC)
        || keys.just_pressed(KeyCode::KeyY);

    if !should_copy {
        return;
    }

    // Extract text from scrollback buffer
    let mut text = String::new();

    let start_line = selection
        .scrollback_start_line
        .min(selection.scrollback_end_line);
    let end_line = selection
        .scrollback_start_line
        .max(selection.scrollback_end_line);

    for line_idx in start_line..=end_line {
        if let Some(line) = scrollback.get_line(line_idx) {
            let line_text = line.to_string();

            // Apply column selection for first and last lines
            if line_idx == start_line && line_idx == end_line {
                // Single line selection
                let start_col = selection.region.start_x.min(selection.region.end_x) as usize;
                let end_col = (selection.region.start_x.max(selection.region.end_x) as usize + 1)
                    .min(line_text.len());

                if start_col < line_text.len() {
                    text.push_str(&line_text[start_col..end_col]);
                }
            } else if line_idx == start_line {
                // First line of multi-line selection
                let start_col = selection.region.start_x as usize;
                if start_col < line_text.len() {
                    text.push_str(&line_text[start_col..]);
                }
                text.push('\n');
            } else if line_idx == end_line {
                // Last line of multi-line selection
                let end_col = (selection.region.end_x as usize + 1).min(line_text.len());
                text.push_str(&line_text[..end_col]);
            } else {
                // Middle lines - full line
                text.push_str(&line_text);
                text.push('\n');
            }
        }
    }

    // Copy to clipboard
    match Clipboard::new() {
        Ok(mut clipboard) => {
            if let Err(e) = clipboard.set_text(&text) {
                error!("Failed to copy scrollback selection to clipboard: {}", e);
            } else {
                info!(
                    "Copied {} characters from scrollback to clipboard",
                    text.len()
                );
            }
        }
        Err(e) => {
            error!("Failed to initialize clipboard: {}", e);
        }
    }
}

/// Plugin for scrollback selection
pub struct ScrollbackSelectionPlugin;

impl Plugin for ScrollbackSelectionPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ScrollbackSelectionState::default())
            .add_event::<ScrollbackMouseSelection>()
            .add_systems(
                Update,
                (handle_mouse_selection, copy_scrollback_selection).chain(),
            );

        info!("Scrollback selection plugin initialized");
    }
}
