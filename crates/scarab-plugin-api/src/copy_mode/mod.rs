//! Copy Mode & Advanced Selection
//!
//! This module provides vim-like keyboard navigation and selection in terminal scrollback.
//! Users can enter copy mode, navigate with hjkl keys, select text with visual mode,
//! and yank to clipboard.

use serde::{Deserialize, Serialize};

/// Copy mode cursor with support for scrollback navigation
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct CopyModeCursor {
    /// Horizontal position (column)
    pub x: u16,
    /// Vertical position (row), can be negative for scrollback
    pub y: i32,
}

impl CopyModeCursor {
    /// Create a new cursor at the specified position
    pub fn new(x: u16, y: i32) -> Self {
        Self { x, y }
    }

    /// Create a cursor at the origin (0, 0)
    pub fn origin() -> Self {
        Self { x: 0, y: 0 }
    }
}

/// Selection region with anchor and active cursor positions
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Selection {
    /// The fixed anchor point where selection started
    pub anchor: CopyModeCursor,
    /// The active cursor that moves with navigation
    pub active: CopyModeCursor,
}

impl Selection {
    /// Create a new selection with the given anchor and active positions
    pub fn new(anchor: CopyModeCursor, active: CopyModeCursor) -> Self {
        Self { anchor, active }
    }

    /// Create a selection anchored at a single point
    pub fn at_point(cursor: CopyModeCursor) -> Self {
        Self {
            anchor: cursor,
            active: cursor,
        }
    }
}

/// Selection mode for different types of text selection
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum SelectionMode {
    /// No active selection
    #[default]
    None,
    /// Character-by-character selection (vim visual mode)
    Cell,
    /// Whole line selection (vim V mode)
    Line,
    /// Rectangular block selection (vim Ctrl+V mode)
    Block,
    /// Word-by-word semantic selection
    Word,
}

/// Copy mode state tracking active mode, cursor, and selection
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct CopyModeState {
    /// Whether copy mode is currently active
    pub active: bool,
    /// Current cursor position
    pub cursor: CopyModeCursor,
    /// Active selection, if any
    pub selection: Option<Selection>,
    /// Current selection mode
    pub selection_mode: SelectionMode,
    /// Viewport offset (lines scrolled from bottom)
    pub viewport_offset: i32,
}

impl CopyModeState {
    /// Create a new inactive copy mode state
    pub fn new() -> Self {
        Self::default()
    }

    /// Activate copy mode at the current cursor position
    pub fn activate(&mut self, cursor: CopyModeCursor) {
        self.active = true;
        self.cursor = cursor;
        self.selection = None;
        self.selection_mode = SelectionMode::None;
    }

    /// Deactivate copy mode and clear selection
    pub fn deactivate(&mut self) {
        self.active = false;
        self.selection = None;
        self.selection_mode = SelectionMode::None;
    }

    /// Start a selection at the current cursor position
    pub fn start_selection(&mut self, mode: SelectionMode) {
        self.selection_mode = mode;
        self.selection = Some(Selection::at_point(self.cursor));
    }

    /// Clear the current selection
    pub fn clear_selection(&mut self) {
        self.selection = None;
        self.selection_mode = SelectionMode::None;
    }

    /// Update the active end of the selection to the current cursor
    pub fn update_selection(&mut self) {
        if let Some(ref mut selection) = self.selection {
            selection.active = self.cursor;
        }
    }

    /// Toggle selection mode on/off
    pub fn toggle_selection(&mut self, mode: SelectionMode) {
        if self.selection_mode == mode {
            self.clear_selection();
        } else {
            self.start_selection(mode);
        }
    }

    /// Swap the anchor and active ends of the selection
    pub fn swap_selection_ends(&mut self) {
        if let Some(ref mut selection) = self.selection {
            std::mem::swap(&mut selection.anchor, &mut selection.active);
            self.cursor = selection.active;
        }
    }

    // Cursor movement methods

    /// Move cursor left, clamping at column 0
    pub fn move_left(&mut self) {
        if self.cursor.x > 0 {
            self.cursor.x -= 1;
        }
    }

    /// Move cursor right, clamping at max_cols
    pub fn move_right(&mut self, max_cols: u16) {
        if self.cursor.x < max_cols.saturating_sub(1) {
            self.cursor.x += 1;
        }
    }

    /// Move cursor up, clamping at scrollback top
    pub fn move_up(&mut self, min_y: i32) {
        if self.cursor.y > min_y {
            self.cursor.y -= 1;
        }
    }

    /// Move cursor down, clamping at screen bottom
    pub fn move_down(&mut self, max_y: i32) {
        if self.cursor.y < max_y {
            self.cursor.y += 1;
        }
    }

    /// Move cursor to column 0 (start of line)
    pub fn move_to_line_start(&mut self) {
        self.cursor.x = 0;
    }

    /// Move cursor to end of line
    pub fn move_to_line_end(&mut self, line_length: u16) {
        self.cursor.x = line_length.saturating_sub(1);
    }

    /// Move cursor to top of scrollback
    pub fn move_to_top(&mut self, min_y: i32) {
        self.cursor.y = min_y;
    }

    /// Move cursor to bottom of screen
    pub fn move_to_bottom(&mut self, max_y: i32) {
        self.cursor.y = max_y;
    }

    // Selection methods

    /// Toggle cell selection mode
    pub fn toggle_cell_selection(&mut self) {
        self.toggle_selection(SelectionMode::Cell);
    }

    /// Toggle line selection mode
    pub fn toggle_line_selection(&mut self) {
        self.toggle_selection(SelectionMode::Line);
    }

    /// Toggle block selection mode
    pub fn toggle_block_selection(&mut self) {
        self.toggle_selection(SelectionMode::Block);
    }

    /// Toggle word selection mode
    pub fn toggle_word_selection(&mut self) {
        self.toggle_selection(SelectionMode::Word);
    }

    /// Get the selected text using a callback to retrieve line content
    ///
    /// The callback function should take a line number (y coordinate) and return
    /// the line content as a String, or None if the line doesn't exist.
    pub fn get_selection_text<F>(&self, get_line: F) -> Option<String>
    where
        F: Fn(i32) -> Option<String>,
    {
        let selection = self.selection.as_ref()?;
        let (start, end) = normalize_selection(selection);

        let mut result = String::new();

        match self.selection_mode {
            SelectionMode::None => return None,
            SelectionMode::Cell => {
                // Character-by-character selection
                if start.y == end.y {
                    // Single line selection
                    if let Some(line) = get_line(start.y) {
                        let start_x = start.x as usize;
                        let end_x = (end.x as usize + 1).min(line.len());
                        if start_x < line.len() {
                            result.push_str(&line[start_x..end_x]);
                        }
                    }
                } else {
                    // Multi-line selection
                    for y in start.y..=end.y {
                        if let Some(line) = get_line(y) {
                            if y == start.y {
                                // First line: from start.x to end
                                let start_x = start.x as usize;
                                if start_x < line.len() {
                                    result.push_str(&line[start_x..]);
                                }
                            } else if y == end.y {
                                // Last line: from beginning to end.x
                                let end_x = (end.x as usize + 1).min(line.len());
                                result.push_str(&line[..end_x]);
                            } else {
                                // Middle lines: entire line
                                result.push_str(&line);
                            }
                            // Add newline except for last line
                            if y < end.y {
                                result.push('\n');
                            }
                        }
                    }
                }
            }
            SelectionMode::Line => {
                // Whole line selection
                for y in start.y..=end.y {
                    if let Some(line) = get_line(y) {
                        result.push_str(&line);
                        if y < end.y {
                            result.push('\n');
                        }
                    }
                }
            }
            SelectionMode::Block => {
                // Rectangular block selection
                let min_x = start.x.min(end.x);
                let max_x = start.x.max(end.x);

                for y in start.y..=end.y {
                    if let Some(line) = get_line(y) {
                        let start_x = min_x as usize;
                        let end_x = (max_x as usize + 1).min(line.len());
                        if start_x < line.len() {
                            result.push_str(&line[start_x..end_x]);
                        }
                        if y < end.y {
                            result.push('\n');
                        }
                    }
                }
            }
            SelectionMode::Word => {
                // Word selection (basic implementation - just use cell mode for now)
                if start.y == end.y {
                    if let Some(line) = get_line(start.y) {
                        let start_x = start.x as usize;
                        let end_x = (end.x as usize + 1).min(line.len());
                        if start_x < line.len() {
                            result.push_str(&line[start_x..end_x]);
                        }
                    }
                }
            }
        }

        if result.is_empty() {
            None
        } else {
            Some(result)
        }
    }
}

/// Search state for copy mode search functionality
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct SearchState {
    /// Whether search is currently active
    pub active: bool,
    /// Current search query
    pub query: String,
    /// Search direction
    pub direction: SearchDirection,
    /// List of all matches found
    pub matches: Vec<SearchMatch>,
    /// Index of the currently selected match
    pub current_match: Option<usize>,
}

impl SearchState {
    /// Create a new inactive search state
    pub fn new() -> Self {
        Self::default()
    }

    /// Start a new search in the given direction
    pub fn start_search(&mut self, direction: SearchDirection) {
        self.active = true;
        self.direction = direction;
        self.query.clear();
        self.matches.clear();
        self.current_match = None;
    }

    /// Update the search query and matches
    pub fn update_query(&mut self, query: String, matches: Vec<SearchMatch>) {
        self.query = query;
        let is_empty = matches.is_empty();
        self.matches = matches;
        self.current_match = if is_empty { None } else { Some(0) };
    }

    /// Navigate to the next match
    pub fn next_match(&mut self) {
        if let Some(current) = self.current_match {
            if !self.matches.is_empty() {
                self.current_match = Some((current + 1) % self.matches.len());
            }
        }
    }

    /// Navigate to the previous match
    pub fn prev_match(&mut self) {
        if let Some(current) = self.current_match {
            if !self.matches.is_empty() {
                self.current_match = Some(if current == 0 {
                    self.matches.len() - 1
                } else {
                    current - 1
                });
            }
        }
    }

    /// Get the currently selected match
    pub fn current(&self) -> Option<&SearchMatch> {
        self.current_match.and_then(|idx| self.matches.get(idx))
    }

    /// Deactivate search
    pub fn deactivate(&mut self) {
        self.active = false;
        self.query.clear();
        self.matches.clear();
        self.current_match = None;
    }
}

/// Search direction
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum SearchDirection {
    /// Search forward from cursor
    #[default]
    Forward,
    /// Search backward from cursor
    Backward,
}

/// A match found during search
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SearchMatch {
    /// Starting cursor position of the match
    pub start: CopyModeCursor,
    /// Ending cursor position of the match
    pub end: CopyModeCursor,
}

impl SearchMatch {
    /// Create a new search match
    pub fn new(start: CopyModeCursor, end: CopyModeCursor) -> Self {
        Self { start, end }
    }
}

/// Find all matches of a query string in the scrollback buffer
///
/// This function searches through the entire scrollback buffer for occurrences
/// of the query string and returns a list of matches with their positions.
///
/// # Arguments
/// * `query` - The search string (case-insensitive)
/// * `get_line` - Callback to retrieve line content by y coordinate
/// * `min_y` - Minimum y coordinate (top of scrollback)
/// * `max_y` - Maximum y coordinate (bottom of screen)
///
/// # Returns
/// Vector of SearchMatch structs representing each occurrence found
pub fn find_matches<F>(query: &str, get_line: F, min_y: i32, max_y: i32) -> Vec<SearchMatch>
where
    F: Fn(i32) -> Option<String>,
{
    let mut matches = Vec::new();

    if query.is_empty() {
        return matches;
    }

    let query_lower = query.to_lowercase();

    // Search through all lines in the buffer
    for y in min_y..=max_y {
        if let Some(line) = get_line(y) {
            let line_lower = line.to_lowercase();

            // Find all occurrences in this line
            let mut start_pos = 0;
            while let Some(match_pos) = line_lower[start_pos..].find(&query_lower) {
                let absolute_pos = start_pos + match_pos;
                let start = CopyModeCursor::new(absolute_pos as u16, y);
                let end = CopyModeCursor::new((absolute_pos + query.len() - 1) as u16, y);
                matches.push(SearchMatch::new(start, end));

                // Move past this match to find the next one
                start_pos = absolute_pos + 1;
            }
        }
    }

    matches
}

/// Normalize a selection so that start comes before end
///
/// Returns (start, end) where start.y < end.y, or if start.y == end.y, then start.x <= end.x
pub fn normalize_selection(selection: &Selection) -> (CopyModeCursor, CopyModeCursor) {
    let a = selection.anchor;
    let b = selection.active;

    if a.y < b.y || (a.y == b.y && a.x <= b.x) {
        (a, b)
    } else {
        (b, a)
    }
}

/// Get the bounding box of a selection
///
/// Returns (min_x, min_y, max_x, max_y) representing the rectangular bounds
pub fn get_selection_bounds(selection: &Selection) -> (u16, i32, u16, i32) {
    let (start, end) = normalize_selection(selection);

    // For cell selection, the bounds are tight around the start and end
    let min_x = start.x.min(end.x);
    let max_x = start.x.max(end.x);
    let min_y = start.y.min(end.y);
    let max_y = start.y.max(end.y);

    (min_x, min_y, max_x, max_y)
}

/// Find word boundaries at the given position in a line
///
/// Returns (start_x, end_x) representing the word boundaries.
/// A word is defined as a sequence of alphanumeric characters or underscores.
///
/// # Arguments
/// * `x` - The column position to search from
/// * `line` - The line content
///
/// # Returns
/// Tuple of (start_x, end_x) representing the word boundaries
pub fn find_word_bounds(x: u16, line: &str) -> (u16, u16) {
    let x_pos = x as usize;

    if line.is_empty() || x_pos >= line.len() {
        return (x, x);
    }

    let chars: Vec<char> = line.chars().collect();

    // Check if current position is a word character
    let is_word_char = |c: char| c.is_alphanumeric() || c == '_';

    if x_pos >= chars.len() {
        return (x, x);
    }

    let current_char = chars[x_pos];

    // If not on a word character, just return the current position
    if !is_word_char(current_char) {
        return (x, x);
    }

    // Find start of word (move left)
    let mut start = x_pos;
    while start > 0 && is_word_char(chars[start - 1]) {
        start -= 1;
    }

    // Find end of word (move right)
    let mut end = x_pos;
    while end < chars.len() - 1 && is_word_char(chars[end + 1]) {
        end += 1;
    }

    (start as u16, end as u16)
}

// Re-export status bar types for use in indicator functions
use crate::status_bar::{Color, RenderItem};

/// Generate mode indicator render items for status bar integration
///
/// Creates a styled indicator showing the current copy mode state.
/// Returns an empty vector if copy mode is not active.
///
/// # Arguments
/// * `state` - The current copy mode state
/// * `search_active` - Whether search is currently active
///
/// # Returns
/// Vector of RenderItem elements for the status bar
///
/// # Example
/// ```rust
/// use scarab_plugin_api::copy_mode::{CopyModeState, copy_mode_indicator};
///
/// let mut state = CopyModeState::new();
/// state.active = true;
/// let items = copy_mode_indicator(&state, false);
/// // Returns: [Background(orange), Foreground(dark), Bold, Text(" COPY "), ResetAttributes]
/// ```
pub fn copy_mode_indicator(state: &CopyModeState, search_active: bool) -> Vec<RenderItem> {
    if !state.active {
        return vec![];
    }

    let mode_text = if search_active {
        "SEARCH"
    } else {
        match state.selection {
            None => "COPY",
            Some(_) => match state.selection_mode {
                SelectionMode::None => "COPY",
                SelectionMode::Cell => "VISUAL",
                SelectionMode::Line => "V-LINE",
                SelectionMode::Block => "V-BLOCK",
                SelectionMode::Word => "V-WORD",
            },
        }
    };

    vec![
        RenderItem::Background(Color::Rgb(255, 158, 100)), // Orange
        RenderItem::Foreground(Color::Rgb(26, 27, 38)),    // Dark text
        RenderItem::Bold,
        RenderItem::Text(format!(" {} ", mode_text)),
        RenderItem::ResetAttributes,
    ]
}

/// Generate position indicator render items for status bar
///
/// Shows the current cursor position in "L{line},C{column}" format.
/// Line and column numbers are 1-indexed for user display.
///
/// # Arguments
/// * `state` - The current copy mode state
///
/// # Returns
/// Vector of RenderItem elements for the status bar
///
/// # Example
/// ```rust
/// use scarab_plugin_api::copy_mode::{CopyModeState, CopyModeCursor, copy_mode_position_indicator};
///
/// let mut state = CopyModeState::new();
/// state.active = true;
/// state.cursor = CopyModeCursor::new(5, 10);
/// let items = copy_mode_position_indicator(&state);
/// // Returns: [Text(" L11,C6 ")]
/// ```
pub fn copy_mode_position_indicator(state: &CopyModeState) -> Vec<RenderItem> {
    if !state.active {
        return vec![];
    }

    vec![RenderItem::Text(format!(
        " L{},C{} ",
        state.cursor.y + 1,
        state.cursor.x + 1
    ))]
}

/// Generate search match count indicator for status bar
///
/// Shows the current match number and total count in "{current}/{total}" format.
/// Returns empty vector if search is not active or there are no matches.
///
/// # Arguments
/// * `search` - The current search state
///
/// # Returns
/// Vector of RenderItem elements for the status bar
///
/// # Example
/// ```rust
/// use scarab_plugin_api::copy_mode::{SearchState, SearchMatch, CopyModeCursor, search_match_indicator};
///
/// let mut search = SearchState::new();
/// search.active = true;
/// search.matches = vec![
///     SearchMatch::new(CopyModeCursor::new(0, 0), CopyModeCursor::new(5, 0)),
///     SearchMatch::new(CopyModeCursor::new(0, 1), CopyModeCursor::new(5, 1)),
/// ];
/// search.current_match = Some(0);
/// let items = search_match_indicator(&search);
/// // Returns: [Text(" 1/2 ")]
/// ```
pub fn search_match_indicator(search: &SearchState) -> Vec<RenderItem> {
    if !search.active || search.matches.is_empty() {
        return vec![];
    }

    let current = search.current_match.map(|i| i + 1).unwrap_or(0);
    vec![RenderItem::Text(format!(
        " {}/{} ",
        current,
        search.matches.len()
    ))]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cursor_creation() {
        let cursor = CopyModeCursor::new(10, 5);
        assert_eq!(cursor.x, 10);
        assert_eq!(cursor.y, 5);
    }

    #[test]
    fn test_cursor_origin() {
        let cursor = CopyModeCursor::origin();
        assert_eq!(cursor.x, 0);
        assert_eq!(cursor.y, 0);
    }

    #[test]
    fn test_cursor_negative_y() {
        // Test scrollback support
        let cursor = CopyModeCursor::new(5, -100);
        assert_eq!(cursor.y, -100);
    }

    #[test]
    fn test_selection_normalization() {
        // Test forward selection (already normalized)
        let sel = Selection {
            anchor: CopyModeCursor::new(0, 0),
            active: CopyModeCursor::new(10, 5),
        };
        let (start, end) = normalize_selection(&sel);
        assert_eq!(start.x, 0);
        assert_eq!(start.y, 0);
        assert_eq!(end.x, 10);
        assert_eq!(end.y, 5);

        // Test backward selection (needs normalization)
        let sel = Selection {
            anchor: CopyModeCursor::new(10, 5),
            active: CopyModeCursor::new(5, 3),
        };
        let (start, end) = normalize_selection(&sel);
        assert_eq!(start.x, 5);
        assert_eq!(start.y, 3);
        assert_eq!(end.x, 10);
        assert_eq!(end.y, 5);

        // Test same line, different columns
        let sel = Selection {
            anchor: CopyModeCursor::new(10, 5),
            active: CopyModeCursor::new(3, 5),
        };
        let (start, end) = normalize_selection(&sel);
        assert_eq!(start.x, 3);
        assert_eq!(start.y, 5);
        assert_eq!(end.x, 10);
        assert_eq!(end.y, 5);
    }

    #[test]
    fn test_selection_bounds() {
        let sel = Selection {
            anchor: CopyModeCursor::new(5, 3),
            active: CopyModeCursor::new(10, 7),
        };
        let (min_x, min_y, max_x, max_y) = get_selection_bounds(&sel);
        assert_eq!(min_x, 5);
        assert_eq!(min_y, 3);
        assert_eq!(max_x, 10);
        assert_eq!(max_y, 7);
    }

    #[test]
    fn test_selection_bounds_reversed() {
        let sel = Selection {
            anchor: CopyModeCursor::new(10, 7),
            active: CopyModeCursor::new(5, 3),
        };
        let (min_x, min_y, max_x, max_y) = get_selection_bounds(&sel);
        assert_eq!(min_x, 5);
        assert_eq!(min_y, 3);
        assert_eq!(max_x, 10);
        assert_eq!(max_y, 7);
    }

    #[test]
    fn test_selection_bounds_single_line() {
        let sel = Selection {
            anchor: CopyModeCursor::new(3, 5),
            active: CopyModeCursor::new(10, 5),
        };
        let (min_x, min_y, max_x, max_y) = get_selection_bounds(&sel);
        assert_eq!(min_x, 3);
        assert_eq!(min_y, 5);
        assert_eq!(max_x, 10);
        assert_eq!(max_y, 5);
    }

    #[test]
    fn test_copy_mode_state_activation() {
        let mut state = CopyModeState::new();
        assert!(!state.active);

        state.activate(CopyModeCursor::new(5, 10));
        assert!(state.active);
        assert_eq!(state.cursor.x, 5);
        assert_eq!(state.cursor.y, 10);

        state.deactivate();
        assert!(!state.active);
    }

    #[test]
    fn test_copy_mode_selection_toggle() {
        let mut state = CopyModeState::new();
        state.cursor = CopyModeCursor::new(5, 5);

        // Start cell selection
        state.toggle_selection(SelectionMode::Cell);
        assert_eq!(state.selection_mode, SelectionMode::Cell);
        assert!(state.selection.is_some());

        // Toggle off
        state.toggle_selection(SelectionMode::Cell);
        assert_eq!(state.selection_mode, SelectionMode::None);
        assert!(state.selection.is_none());
    }

    #[test]
    fn test_copy_mode_swap_selection_ends() {
        let mut state = CopyModeState::new();
        state.cursor = CopyModeCursor::new(5, 5);
        state.start_selection(SelectionMode::Cell);

        // Move cursor to create selection
        state.cursor = CopyModeCursor::new(10, 10);
        state.update_selection();

        let selection = state.selection.as_ref().unwrap();
        assert_eq!(selection.anchor, CopyModeCursor::new(5, 5));
        assert_eq!(selection.active, CopyModeCursor::new(10, 10));

        // Swap ends
        state.swap_selection_ends();
        let selection = state.selection.as_ref().unwrap();
        assert_eq!(selection.anchor, CopyModeCursor::new(10, 10));
        assert_eq!(selection.active, CopyModeCursor::new(5, 5));
        assert_eq!(state.cursor, CopyModeCursor::new(5, 5));
    }

    #[test]
    fn test_search_state_navigation() {
        let mut search = SearchState::new();
        let matches = vec![
            SearchMatch::new(CopyModeCursor::new(0, 0), CopyModeCursor::new(5, 0)),
            SearchMatch::new(CopyModeCursor::new(10, 0), CopyModeCursor::new(15, 0)),
            SearchMatch::new(CopyModeCursor::new(0, 1), CopyModeCursor::new(5, 1)),
        ];

        search.update_query("test".to_string(), matches);
        assert_eq!(search.current_match, Some(0));

        // Navigate forward
        search.next_match();
        assert_eq!(search.current_match, Some(1));

        search.next_match();
        assert_eq!(search.current_match, Some(2));

        // Wrap around
        search.next_match();
        assert_eq!(search.current_match, Some(0));

        // Navigate backward
        search.prev_match();
        assert_eq!(search.current_match, Some(2));
    }

    #[test]
    fn test_search_state_no_matches() {
        let mut search = SearchState::new();
        search.update_query("notfound".to_string(), vec![]);
        assert_eq!(search.current_match, None);

        // Navigation should do nothing
        search.next_match();
        assert_eq!(search.current_match, None);

        search.prev_match();
        assert_eq!(search.current_match, None);
    }

    #[test]
    fn test_find_matches() {
        let get_line = |y: i32| match y {
            0 => Some("Hello world, hello Rust!".to_string()),
            1 => Some("Another HELLO here".to_string()),
            2 => Some("No match on this line".to_string()),
            _ => None,
        };

        let matches = find_matches("hello", get_line, 0, 2);

        // Should find "Hello" (case-insensitive) and "hello" on line 0, and "HELLO" on line 1
        assert_eq!(matches.len(), 3);

        // First match: "Hello" at start of line 0
        assert_eq!(matches[0].start, CopyModeCursor::new(0, 0));
        assert_eq!(matches[0].end, CopyModeCursor::new(4, 0));

        // Second match: "hello" at position 13 on line 0
        assert_eq!(matches[1].start, CopyModeCursor::new(13, 0));
        assert_eq!(matches[1].end, CopyModeCursor::new(17, 0));

        // Third match: "HELLO" on line 1
        assert_eq!(matches[2].start, CopyModeCursor::new(8, 1));
        assert_eq!(matches[2].end, CopyModeCursor::new(12, 1));
    }

    #[test]
    fn test_find_matches_empty_query() {
        let get_line = |_y: i32| Some("Some text".to_string());
        let matches = find_matches("", get_line, 0, 5);
        assert_eq!(matches.len(), 0);
    }

    #[test]
    fn test_find_matches_no_matches() {
        let get_line = |_y: i32| Some("No matches here".to_string());
        let matches = find_matches("xyz", get_line, 0, 5);
        assert_eq!(matches.len(), 0);
    }

    #[test]
    fn test_selection_mode_default() {
        let mode = SelectionMode::default();
        assert_eq!(mode, SelectionMode::None);
    }

    #[test]
    fn test_search_direction_default() {
        let direction = SearchDirection::default();
        assert_eq!(direction, SearchDirection::Forward);
    }

    #[test]
    fn test_move_left() {
        let mut state = CopyModeState::new();
        state.cursor = CopyModeCursor::new(5, 0);

        state.move_left();
        assert_eq!(state.cursor.x, 4);

        state.move_left();
        state.move_left();
        state.move_left();
        state.move_left();
        assert_eq!(state.cursor.x, 0);

        // Should clamp at 0
        state.move_left();
        assert_eq!(state.cursor.x, 0);
    }

    #[test]
    fn test_move_right() {
        let mut state = CopyModeState::new();
        state.cursor = CopyModeCursor::new(5, 0);

        state.move_right(80);
        assert_eq!(state.cursor.x, 6);

        // Should clamp at max_cols - 1
        state.cursor.x = 79;
        state.move_right(80);
        assert_eq!(state.cursor.x, 79);
    }

    #[test]
    fn test_move_up() {
        let mut state = CopyModeState::new();
        state.cursor = CopyModeCursor::new(5, 10);

        state.move_up(0);
        assert_eq!(state.cursor.y, 9);

        // Move to min_y
        for _ in 0..10 {
            state.move_up(0);
        }
        assert_eq!(state.cursor.y, 0);

        // Should clamp at min_y
        state.move_up(0);
        assert_eq!(state.cursor.y, 0);
    }

    #[test]
    fn test_move_down() {
        let mut state = CopyModeState::new();
        state.cursor = CopyModeCursor::new(5, 0);

        state.move_down(24);
        assert_eq!(state.cursor.y, 1);

        // Move to max_y
        state.cursor.y = 23;
        state.move_down(24);
        assert_eq!(state.cursor.y, 24);

        // Should clamp at max_y
        state.move_down(24);
        assert_eq!(state.cursor.y, 24);
    }

    #[test]
    fn test_move_up_with_scrollback() {
        let mut state = CopyModeState::new();
        state.cursor = CopyModeCursor::new(5, 0);

        // Move into scrollback (negative y)
        state.move_up(-100);
        assert_eq!(state.cursor.y, -1);

        state.cursor.y = -50;
        state.move_up(-100);
        assert_eq!(state.cursor.y, -51);

        // Clamp at scrollback top
        state.cursor.y = -100;
        state.move_up(-100);
        assert_eq!(state.cursor.y, -100);
    }

    #[test]
    fn test_move_to_line_start() {
        let mut state = CopyModeState::new();
        state.cursor = CopyModeCursor::new(42, 5);

        state.move_to_line_start();
        assert_eq!(state.cursor.x, 0);
        assert_eq!(state.cursor.y, 5); // y unchanged
    }

    #[test]
    fn test_move_to_line_end() {
        let mut state = CopyModeState::new();
        state.cursor = CopyModeCursor::new(5, 3);

        state.move_to_line_end(80);
        assert_eq!(state.cursor.x, 79);
        assert_eq!(state.cursor.y, 3); // y unchanged
    }

    #[test]
    fn test_move_to_top() {
        let mut state = CopyModeState::new();
        state.cursor = CopyModeCursor::new(5, 10);

        state.move_to_top(-100);
        assert_eq!(state.cursor.x, 5); // x unchanged
        assert_eq!(state.cursor.y, -100);
    }

    #[test]
    fn test_move_to_bottom() {
        let mut state = CopyModeState::new();
        state.cursor = CopyModeCursor::new(5, -50);

        state.move_to_bottom(24);
        assert_eq!(state.cursor.x, 5); // x unchanged
        assert_eq!(state.cursor.y, 24);
    }

    #[test]
    fn test_toggle_cell_selection() {
        let mut state = CopyModeState::new();
        state.cursor = CopyModeCursor::new(5, 5);

        state.toggle_cell_selection();
        assert_eq!(state.selection_mode, SelectionMode::Cell);
        assert!(state.selection.is_some());

        state.toggle_cell_selection();
        assert_eq!(state.selection_mode, SelectionMode::None);
        assert!(state.selection.is_none());
    }

    #[test]
    fn test_toggle_line_selection() {
        let mut state = CopyModeState::new();
        state.cursor = CopyModeCursor::new(5, 5);

        state.toggle_line_selection();
        assert_eq!(state.selection_mode, SelectionMode::Line);
        assert!(state.selection.is_some());

        state.toggle_line_selection();
        assert_eq!(state.selection_mode, SelectionMode::None);
        assert!(state.selection.is_none());
    }

    #[test]
    fn test_toggle_block_selection() {
        let mut state = CopyModeState::new();
        state.cursor = CopyModeCursor::new(5, 5);

        state.toggle_block_selection();
        assert_eq!(state.selection_mode, SelectionMode::Block);
        assert!(state.selection.is_some());

        state.toggle_block_selection();
        assert_eq!(state.selection_mode, SelectionMode::None);
        assert!(state.selection.is_none());
    }

    #[test]
    fn test_toggle_word_selection() {
        let mut state = CopyModeState::new();
        state.cursor = CopyModeCursor::new(5, 5);

        state.toggle_word_selection();
        assert_eq!(state.selection_mode, SelectionMode::Word);
        assert!(state.selection.is_some());

        state.toggle_word_selection();
        assert_eq!(state.selection_mode, SelectionMode::None);
        assert!(state.selection.is_none());
    }

    #[test]
    fn test_get_selection_text_cell_single_line() {
        let mut state = CopyModeState::new();
        state.cursor = CopyModeCursor::new(5, 0);
        state.start_selection(SelectionMode::Cell);
        state.cursor = CopyModeCursor::new(9, 0);
        state.update_selection();

        let get_line = |y: i32| {
            if y == 0 {
                Some("Hello, World!".to_string())
            } else {
                None
            }
        };

        let text = state.get_selection_text(get_line);
        assert_eq!(text, Some(", Wor".to_string()));
    }

    #[test]
    fn test_get_selection_text_cell_multi_line() {
        let mut state = CopyModeState::new();
        state.cursor = CopyModeCursor::new(5, 0);
        state.start_selection(SelectionMode::Cell);
        state.cursor = CopyModeCursor::new(5, 2);
        state.update_selection();

        let get_line = |y: i32| match y {
            0 => Some("Line 0".to_string()),
            1 => Some("Line 1".to_string()),
            2 => Some("Line 2".to_string()),
            _ => None,
        };

        let text = state.get_selection_text(get_line);
        assert_eq!(text, Some("0\nLine 1\nLine 2".to_string()));
    }

    #[test]
    fn test_get_selection_text_line_mode() {
        let mut state = CopyModeState::new();
        state.cursor = CopyModeCursor::new(5, 0);
        state.start_selection(SelectionMode::Line);
        state.cursor = CopyModeCursor::new(10, 2);
        state.update_selection();

        let get_line = |y: i32| match y {
            0 => Some("First line".to_string()),
            1 => Some("Second line".to_string()),
            2 => Some("Third line".to_string()),
            _ => None,
        };

        let text = state.get_selection_text(get_line);
        assert_eq!(
            text,
            Some("First line\nSecond line\nThird line".to_string())
        );
    }

    #[test]
    fn test_get_selection_text_block_mode() {
        let mut state = CopyModeState::new();
        state.cursor = CopyModeCursor::new(2, 0);
        state.start_selection(SelectionMode::Block);
        state.cursor = CopyModeCursor::new(5, 2);
        state.update_selection();

        let get_line = |y: i32| match y {
            0 => Some("ABCDEFGH".to_string()),
            1 => Some("12345678".to_string()),
            2 => Some("abcdefgh".to_string()),
            _ => None,
        };

        let text = state.get_selection_text(get_line);
        assert_eq!(text, Some("CDEF\n3456\ncdef".to_string()));
    }

    #[test]
    fn test_get_selection_text_no_selection() {
        let state = CopyModeState::new();

        let get_line = |_y: i32| Some("Line".to_string());

        let text = state.get_selection_text(get_line);
        assert_eq!(text, None);
    }

    #[test]
    fn test_movement_with_selection_update() {
        let mut state = CopyModeState::new();
        state.cursor = CopyModeCursor::new(5, 5);
        state.start_selection(SelectionMode::Cell);

        // Move and update selection
        state.move_right(80);
        state.update_selection();

        let selection = state.selection.as_ref().unwrap();
        assert_eq!(selection.anchor, CopyModeCursor::new(5, 5));
        assert_eq!(selection.active, CopyModeCursor::new(6, 5));

        // Move down and update
        state.move_down(24);
        state.update_selection();

        let selection = state.selection.as_ref().unwrap();
        assert_eq!(selection.anchor, CopyModeCursor::new(5, 5));
        assert_eq!(selection.active, CopyModeCursor::new(6, 6));
    }

    #[test]
    fn test_find_word_bounds() {
        let line = "Hello world, this is a test";

        // Test word "Hello"
        let (start, end) = find_word_bounds(2, line);
        assert_eq!(start, 0);
        assert_eq!(end, 4);

        // Test word "world"
        let (start, end) = find_word_bounds(6, line);
        assert_eq!(start, 6);
        assert_eq!(end, 10);

        // Test word "test"
        let (start, end) = find_word_bounds(24, line);
        assert_eq!(start, 23);
        assert_eq!(end, 26);

        // Test on space (non-word character)
        let (start, end) = find_word_bounds(5, line);
        assert_eq!(start, 5);
        assert_eq!(end, 5);
    }

    #[test]
    fn test_find_word_bounds_empty_line() {
        let line = "";
        let (start, end) = find_word_bounds(0, line);
        assert_eq!(start, 0);
        assert_eq!(end, 0);
    }

    #[test]
    fn test_find_word_bounds_with_underscores() {
        let line = "hello_world test_case";

        // Test word with underscore
        let (start, end) = find_word_bounds(6, line);
        assert_eq!(start, 0);
        assert_eq!(end, 10); // "hello_world"
    }

    #[test]
    fn test_copy_mode_indicator() {
        let mut state = CopyModeState::new();

        // Inactive state
        let items = copy_mode_indicator(&state, false);
        assert_eq!(items.len(), 0);

        // Active copy mode
        state.active = true;
        let items = copy_mode_indicator(&state, false);
        assert!(items.len() > 0);
        match &items[3] {
            RenderItem::Text(s) => assert_eq!(s, " COPY "),
            _ => panic!("Expected text item"),
        }

        // Visual mode
        state.selection = Some(Selection::at_point(CopyModeCursor::new(0, 0)));
        state.selection_mode = SelectionMode::Cell;
        let items = copy_mode_indicator(&state, false);
        match &items[3] {
            RenderItem::Text(s) => assert_eq!(s, " VISUAL "),
            _ => panic!("Expected text item"),
        }

        // Line mode
        state.selection_mode = SelectionMode::Line;
        let items = copy_mode_indicator(&state, false);
        match &items[3] {
            RenderItem::Text(s) => assert_eq!(s, " V-LINE "),
            _ => panic!("Expected text item"),
        }

        // Search mode
        let items = copy_mode_indicator(&state, true);
        match &items[3] {
            RenderItem::Text(s) => assert_eq!(s, " SEARCH "),
            _ => panic!("Expected text item"),
        }
    }

    #[test]
    fn test_copy_mode_position_indicator() {
        let mut state = CopyModeState::new();

        // Inactive state
        let items = copy_mode_position_indicator(&state);
        assert_eq!(items.len(), 0);

        // Active with cursor position
        state.active = true;
        state.cursor = CopyModeCursor::new(5, 10);
        let items = copy_mode_position_indicator(&state);
        assert_eq!(items.len(), 1);
        match &items[0] {
            RenderItem::Text(s) => assert_eq!(s, " L11,C6 "),
            _ => panic!("Expected text item"),
        }
    }

    #[test]
    fn test_search_match_indicator() {
        let mut search = SearchState::new();

        // Inactive search
        let items = search_match_indicator(&search);
        assert_eq!(items.len(), 0);

        // Active search with no matches
        search.active = true;
        let items = search_match_indicator(&search);
        assert_eq!(items.len(), 0);

        // Active search with matches
        search.matches = vec![
            SearchMatch::new(CopyModeCursor::new(0, 0), CopyModeCursor::new(5, 0)),
            SearchMatch::new(CopyModeCursor::new(0, 1), CopyModeCursor::new(5, 1)),
            SearchMatch::new(CopyModeCursor::new(0, 2), CopyModeCursor::new(5, 2)),
        ];
        search.current_match = Some(1);

        let items = search_match_indicator(&search);
        assert_eq!(items.len(), 1);
        match &items[0] {
            RenderItem::Text(s) => assert_eq!(s, " 2/3 "),
            _ => panic!("Expected text item"),
        }
    }
}
