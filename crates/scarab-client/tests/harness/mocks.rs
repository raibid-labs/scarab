//! Mock types for headless testing
//!
//! This module provides mock implementations of Scarab's runtime dependencies
//! for use in headless tests. These mocks allow testing UI logic without
//! requiring a running daemon or shared memory.

use bevy::prelude::*;
use scarab_protocol::{Cell, SharedState, BUFFER_SIZE, GRID_HEIGHT, GRID_WIDTH};
use std::sync::{Arc, Mutex};

/// Mock implementation of SharedMemoryReader for headless tests.
///
/// This provides a simulated terminal grid that can be manipulated in tests
/// to verify UI behavior without requiring actual shared memory.
///
/// ## Example
///
/// ```rust,no_run
/// use crate::harness::mocks::MockSharedMemoryReader;
///
/// let mut reader = MockSharedMemoryReader::default();
/// reader.set_cell(0, 0, 'H', 0xFFFFFFFF, 0x000000FF);
/// reader.set_cell(1, 0, 'i', 0xFFFFFFFF, 0x000000FF);
/// ```
#[derive(Resource, Clone)]
pub struct MockSharedMemoryReader {
    state: Arc<Mutex<MockSharedState>>,
}

impl Default for MockSharedMemoryReader {
    fn default() -> Self {
        Self {
            state: Arc::new(Mutex::new(MockSharedState::default())),
        }
    }
}

impl MockSharedMemoryReader {
    /// Create a new mock reader with default (empty) state.
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a mock reader with custom initial state.
    pub fn with_state(state: MockSharedState) -> Self {
        Self {
            state: Arc::new(Mutex::new(state)),
        }
    }

    /// Set a single cell in the grid.
    ///
    /// ## Example
    ///
    /// ```rust,no_run
    /// reader.set_cell(5, 10, 'A', 0xFFFFFFFF, 0x000000FF);
    /// ```
    pub fn set_cell(&mut self, x: u16, y: u16, c: char, fg: u32, bg: u32) {
        let mut state = self.state.lock().unwrap();
        state.set_cell(x, y, c, fg, bg);
    }

    /// Set a string of text starting at a position.
    ///
    /// ## Example
    ///
    /// ```rust,no_run
    /// reader.set_text(0, 0, "Hello, World!", 0xFFFFFFFF, 0x000000FF);
    /// ```
    pub fn set_text(&mut self, x: u16, y: u16, text: &str, fg: u32, bg: u32) {
        let mut state = self.state.lock().unwrap();
        state.set_text(x, y, text, fg, bg);
    }

    /// Set the cursor position.
    pub fn set_cursor(&mut self, x: u16, y: u16) {
        let mut state = self.state.lock().unwrap();
        state.cursor_x = x;
        state.cursor_y = y;
    }

    /// Get the current sequence number.
    pub fn sequence_number(&self) -> u64 {
        let state = self.state.lock().unwrap();
        state.sequence_number
    }

    /// Increment the sequence number (simulates a daemon update).
    pub fn tick(&mut self) {
        let mut state = self.state.lock().unwrap();
        state.sequence_number += 1;
    }

    /// Get a copy of the current grid state.
    pub fn get_state(&self) -> MockSharedState {
        let state = self.state.lock().unwrap();
        state.clone()
    }

    /// Clear the entire grid.
    pub fn clear(&mut self) {
        let mut state = self.state.lock().unwrap();
        state.clear();
    }

    /// Get the character at a specific position.
    pub fn get_char(&self, x: u16, y: u16) -> Option<char> {
        let state = self.state.lock().unwrap();
        state.get_char(x, y)
    }

    /// Get the entire text content of a row.
    pub fn get_row_text(&self, y: u16) -> String {
        let state = self.state.lock().unwrap();
        state.get_row_text(y)
    }

    /// Fill a rectangular region with a character.
    ///
    /// ## Example
    ///
    /// ```rust,no_run
    /// // Fill a 10x5 box with '#'
    /// reader.fill_rect(0, 0, 10, 5, '#', 0xFFFFFFFF, 0xFF0000FF);
    /// ```
    pub fn fill_rect(
        &mut self,
        x: u16,
        y: u16,
        width: u16,
        height: u16,
        c: char,
        fg: u32,
        bg: u32,
    ) {
        let mut state = self.state.lock().unwrap();
        for dy in 0..height {
            for dx in 0..width {
                state.set_cell(x + dx, y + dy, c, fg, bg);
            }
        }
    }

    /// Simulate terminal output by setting text at multiple positions.
    ///
    /// ## Example
    ///
    /// ```rust,no_run
    /// reader.simulate_output(&[
    ///     (0, 0, "user@host:~$"),
    ///     (0, 1, "ls -la"),
    ///     (0, 2, "total 42"),
    /// ], 0xFFFFFFFF, 0x000000FF);
    /// ```
    pub fn simulate_output(&mut self, lines: &[(u16, u16, &str)], fg: u32, bg: u32) {
        let mut state = self.state.lock().unwrap();
        for (x, y, text) in lines {
            state.set_text(*x, *y, text, fg, bg);
        }
        state.sequence_number += 1;
    }
}

/// Mock terminal grid state.
///
/// This mirrors the `SharedState` structure from `scarab-protocol` but uses
/// standard Rust types for easier manipulation in tests.
#[derive(Clone)]
pub struct MockSharedState {
    pub sequence_number: u64,
    pub cursor_x: u16,
    pub cursor_y: u16,
    pub cells: Vec<Cell>,
}

impl Default for MockSharedState {
    fn default() -> Self {
        Self {
            sequence_number: 0,
            cursor_x: 0,
            cursor_y: 0,
            cells: vec![Cell::default(); BUFFER_SIZE],
        }
    }
}

impl MockSharedState {
    /// Create a new mock state with default values.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set a single cell in the grid.
    pub fn set_cell(&mut self, x: u16, y: u16, c: char, fg: u32, bg: u32) {
        if x >= GRID_WIDTH as u16 || y >= GRID_HEIGHT as u16 {
            return; // Silently ignore out-of-bounds
        }

        let index = (y as usize * GRID_WIDTH) + x as usize;
        if index < BUFFER_SIZE {
            self.cells[index] = Cell {
                char_codepoint: c as u32,
                fg,
                bg,
                flags: 0,
                _padding: [0; 3],
            };
        }
    }

    /// Set a string of text starting at a position.
    pub fn set_text(&mut self, x: u16, y: u16, text: &str, fg: u32, bg: u32) {
        for (i, c) in text.chars().enumerate() {
            let pos_x = x + i as u16;
            if pos_x >= GRID_WIDTH as u16 {
                break; // Stop at end of line
            }
            self.set_cell(pos_x, y, c, fg, bg);
        }
    }

    /// Get the character at a specific position.
    pub fn get_char(&self, x: u16, y: u16) -> Option<char> {
        if x >= GRID_WIDTH as u16 || y >= GRID_HEIGHT as u16 {
            return None;
        }

        let index = (y as usize * GRID_WIDTH) + x as usize;
        if index < BUFFER_SIZE {
            let codepoint = self.cells[index].char_codepoint;
            char::from_u32(codepoint)
        } else {
            None
        }
    }

    /// Get the entire text content of a row.
    pub fn get_row_text(&self, y: u16) -> String {
        if y >= GRID_HEIGHT as u16 {
            return String::new();
        }

        let mut text = String::new();
        for x in 0..GRID_WIDTH {
            if let Some(c) = self.get_char(x as u16, y) {
                text.push(c);
            }
        }
        text.trim_end().to_string() // Remove trailing spaces
    }

    /// Clear the entire grid to default (spaces).
    pub fn clear(&mut self) {
        for cell in &mut self.cells {
            *cell = Cell::default();
        }
        self.cursor_x = 0;
        self.cursor_y = 0;
        self.sequence_number += 1;
    }

    /// Convert to a real SharedState (for integration tests).
    pub fn to_shared_state(&self) -> SharedState {
        let mut cells = [Cell::default(); BUFFER_SIZE];
        cells.copy_from_slice(&self.cells);

        SharedState {
            sequence_number: self.sequence_number,
            dirty_flag: 1,
            error_mode: 0,
            cursor_x: self.cursor_x,
            cursor_y: self.cursor_y,
            _padding2: [0; 2],
            cells,
        }
    }
}

/// Helper to create a grid with sample terminal output.
///
/// ## Example
///
/// ```rust,no_run
/// let state = sample_terminal_output();
/// let reader = MockSharedMemoryReader::with_state(state);
/// ```
pub fn sample_terminal_output() -> MockSharedState {
    let mut state = MockSharedState::new();

    // Simulate a typical terminal session
    state.set_text(0, 0, "user@scarab:~$ ls -la", 0x00FF00FF, 0x000000FF);
    state.set_text(0, 1, "total 128", 0xFFFFFFFF, 0x000000FF);
    state.set_text(
        0,
        2,
        "drwxr-xr-x  5 user user  4096 Dec  1 12:00 .",
        0xFFFFFFFF,
        0x000000FF,
    );
    state.set_text(
        0,
        3,
        "drwxr-xr-x 20 user user  4096 Nov 30 18:30 ..",
        0xFFFFFFFF,
        0x000000FF,
    );
    state.set_text(
        0,
        4,
        "-rw-r--r--  1 user user  1234 Dec  1 10:45 README.md",
        0xFFFFFFFF,
        0x000000FF,
    );
    state.set_text(
        0,
        5,
        "drwxr-xr-x  8 user user  4096 Dec  1 11:20 src",
        0x00FFFFFF,
        0x000000FF,
    );
    state.set_text(0, 6, "user@scarab:~$ _", 0x00FF00FF, 0x000000FF);

    state.cursor_x = 15;
    state.cursor_y = 6;
    state.sequence_number = 1;

    state
}

/// Helper to create a grid with a URL for link hint testing.
pub fn sample_url_output() -> MockSharedState {
    let mut state = MockSharedState::new();

    state.set_text(
        0,
        0,
        "Check out https://github.com/raibid-labs/scarab",
        0xFFFFFFFF,
        0x000000FF,
    );
    state.set_text(
        0,
        1,
        "Documentation at https://docs.rs/scarab",
        0xFFFFFFFF,
        0x000000FF,
    );
    state.set_text(0, 2, "Email: support@scarab.dev", 0xFFFFFFFF, 0x000000FF);

    state.sequence_number = 1;

    state
}

/// Helper to create a grid with ANSI color output.
pub fn sample_colored_output() -> MockSharedState {
    let mut state = MockSharedState::new();

    // Red error message
    state.set_text(0, 0, "ERROR: ", 0xFF0000FF, 0x000000FF);
    state.set_text(7, 0, "File not found", 0xFFFFFFFF, 0x000000FF);

    // Green success message
    state.set_text(0, 1, "SUCCESS: ", 0x00FF00FF, 0x000000FF);
    state.set_text(9, 1, "Build completed", 0xFFFFFFFF, 0x000000FF);

    // Yellow warning
    state.set_text(0, 2, "WARNING: ", 0xFFFF00FF, 0x000000FF);
    state.set_text(9, 2, "Deprecated API", 0xFFFFFFFF, 0x000000FF);

    // Blue info
    state.set_text(0, 3, "INFO: ", 0x0000FFFF, 0x000000FF);
    state.set_text(6, 3, "Starting server...", 0xFFFFFFFF, 0x000000FF);

    state.sequence_number = 1;

    state
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mock_reader_creation() {
        let reader = MockSharedMemoryReader::new();
        assert_eq!(reader.sequence_number(), 0);
    }

    #[test]
    fn test_set_and_get_cell() {
        let mut reader = MockSharedMemoryReader::new();
        reader.set_cell(5, 10, 'X', 0xFF0000FF, 0x000000FF);

        let c = reader.get_char(5, 10);
        assert_eq!(c, Some('X'));
    }

    #[test]
    fn test_set_text() {
        let mut reader = MockSharedMemoryReader::new();
        reader.set_text(0, 0, "Hello", 0xFFFFFFFF, 0x000000FF);

        assert_eq!(reader.get_char(0, 0), Some('H'));
        assert_eq!(reader.get_char(1, 0), Some('e'));
        assert_eq!(reader.get_char(2, 0), Some('l'));
        assert_eq!(reader.get_char(3, 0), Some('l'));
        assert_eq!(reader.get_char(4, 0), Some('o'));
    }

    #[test]
    fn test_get_row_text() {
        let mut reader = MockSharedMemoryReader::new();
        reader.set_text(0, 0, "Hello, World!", 0xFFFFFFFF, 0x000000FF);

        let text = reader.get_row_text(0);
        assert_eq!(text, "Hello, World!");
    }

    #[test]
    fn test_cursor_position() {
        let mut reader = MockSharedMemoryReader::new();
        reader.set_cursor(42, 13);

        let state = reader.get_state();
        assert_eq!(state.cursor_x, 42);
        assert_eq!(state.cursor_y, 13);
    }

    #[test]
    fn test_tick_increments_sequence() {
        let mut reader = MockSharedMemoryReader::new();
        assert_eq!(reader.sequence_number(), 0);

        reader.tick();
        assert_eq!(reader.sequence_number(), 1);

        reader.tick();
        assert_eq!(reader.sequence_number(), 2);
    }

    #[test]
    fn test_clear() {
        let mut reader = MockSharedMemoryReader::new();
        reader.set_text(0, 0, "Test", 0xFFFFFFFF, 0x000000FF);
        reader.clear();

        // All cells should be spaces
        assert_eq!(reader.get_char(0, 0), Some(' '));
        assert_eq!(reader.get_row_text(0), "");
    }

    #[test]
    fn test_fill_rect() {
        let mut reader = MockSharedMemoryReader::new();
        reader.fill_rect(5, 5, 3, 2, '#', 0xFFFFFFFF, 0x000000FF);

        // Check the filled region
        assert_eq!(reader.get_char(5, 5), Some('#'));
        assert_eq!(reader.get_char(6, 5), Some('#'));
        assert_eq!(reader.get_char(7, 5), Some('#'));
        assert_eq!(reader.get_char(5, 6), Some('#'));
        assert_eq!(reader.get_char(6, 6), Some('#'));
        assert_eq!(reader.get_char(7, 6), Some('#'));

        // Check outside the region
        assert_eq!(reader.get_char(4, 5), Some(' '));
        assert_eq!(reader.get_char(8, 5), Some(' '));
    }

    #[test]
    fn test_sample_terminal_output() {
        let state = sample_terminal_output();
        assert!(state.get_row_text(0).contains("user@scarab"));
        assert!(state.get_row_text(1).contains("total"));
        assert_eq!(state.cursor_x, 15);
        assert_eq!(state.cursor_y, 6);
    }

    #[test]
    fn test_sample_url_output() {
        let state = sample_url_output();
        assert!(state.get_row_text(0).contains("https://github.com"));
        assert!(state.get_row_text(1).contains("https://docs.rs"));
    }

    #[test]
    fn test_sample_colored_output() {
        let state = sample_colored_output();
        assert!(state.get_row_text(0).contains("ERROR"));
        assert!(state.get_row_text(1).contains("SUCCESS"));
        assert!(state.get_row_text(2).contains("WARNING"));
        assert!(state.get_row_text(3).contains("INFO"));
    }

    #[test]
    fn test_out_of_bounds_handling() {
        let mut reader = MockSharedMemoryReader::new();

        // Should not panic
        reader.set_cell(1000, 1000, 'X', 0xFFFFFFFF, 0x000000FF);
        assert_eq!(reader.get_char(1000, 1000), None);
    }

    #[test]
    fn test_to_shared_state_conversion() {
        let mut mock = MockSharedState::new();
        mock.set_text(0, 0, "Test", 0xFFFFFFFF, 0x000000FF);
        mock.cursor_x = 10;
        mock.cursor_y = 5;
        mock.sequence_number = 42;

        let shared = mock.to_shared_state();
        assert_eq!(shared.cursor_x, 10);
        assert_eq!(shared.cursor_y, 5);
        assert_eq!(shared.sequence_number, 42);
    }
}
