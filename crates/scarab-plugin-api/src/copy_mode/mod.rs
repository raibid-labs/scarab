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
    fn test_selection_mode_default() {
        let mode = SelectionMode::default();
        assert_eq!(mode, SelectionMode::None);
    }

    #[test]
    fn test_search_direction_default() {
        let direction = SearchDirection::default();
        assert_eq!(direction, SearchDirection::Forward);
    }
}
