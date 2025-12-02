//! Terminal grid access interface for mouse operations
//!
//! This module defines traits for accessing terminal grid content
//! from mouse handlers without creating circular dependencies.

use crate::types::Position;

/// Trait for reading character data from the terminal grid
///
/// This allows mouse handlers to:
/// - Detect word boundaries for double-click selection
/// - Scan for URLs and file paths
/// - Implement smart selection features
///
/// The client should implement this trait by wrapping the SharedMemoryReader
/// and exposing safe character access.
pub trait TerminalGridReader: Send + Sync {
    /// Get character at grid position
    ///
    /// # Arguments
    /// * `pos` - Grid position (col, row)
    ///
    /// # Returns
    /// * `Some(char)` if position is valid and contains a character
    /// * `None` if position is out of bounds or cell is empty
    fn char_at(&self, pos: Position) -> Option<char>;

    /// Get a line of text from the grid
    ///
    /// # Arguments
    /// * `row` - Row number (0-indexed)
    ///
    /// # Returns
    /// String containing the entire row, with trailing spaces trimmed
    fn line_at(&self, row: u16) -> String;

    /// Get grid dimensions
    ///
    /// # Returns
    /// Tuple of (columns, rows)
    fn dimensions(&self) -> (u16, u16);

    /// Check if position is within bounds
    fn is_valid_position(&self, pos: Position) -> bool {
        let (cols, rows) = self.dimensions();
        pos.x < cols && pos.y < rows
    }
}

/// Default implementation that returns None for all queries
///
/// This is used as a fallback when no grid reader is available.
pub struct NoOpGridReader;

impl TerminalGridReader for NoOpGridReader {
    fn char_at(&self, _pos: Position) -> Option<char> {
        None
    }

    fn line_at(&self, _row: u16) -> String {
        String::new()
    }

    fn dimensions(&self) -> (u16, u16) {
        (80, 24)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockGridReader {
        data: Vec<Vec<char>>,
    }

    impl TerminalGridReader for MockGridReader {
        fn char_at(&self, pos: Position) -> Option<char> {
            self.data
                .get(pos.y as usize)
                .and_then(|row| row.get(pos.x as usize))
                .copied()
        }

        fn line_at(&self, row: u16) -> String {
            self.data
                .get(row as usize)
                .map(|chars| chars.iter().collect())
                .unwrap_or_default()
        }

        fn dimensions(&self) -> (u16, u16) {
            let rows = self.data.len() as u16;
            let cols = self.data.first().map_or(0, |row| row.len() as u16);
            (cols, rows)
        }
    }

    #[test]
    fn test_char_at() {
        let reader = MockGridReader {
            data: vec![vec!['H', 'e', 'l', 'l', 'o']],
        };

        assert_eq!(reader.char_at(Position::new(0, 0)), Some('H'));
        assert_eq!(reader.char_at(Position::new(4, 0)), Some('o'));
        assert_eq!(reader.char_at(Position::new(5, 0)), None);
        assert_eq!(reader.char_at(Position::new(0, 1)), None);
    }

    #[test]
    fn test_line_at() {
        let reader = MockGridReader {
            data: vec![
                vec!['H', 'e', 'l', 'l', 'o'],
                vec!['W', 'o', 'r', 'l', 'd'],
            ],
        };

        assert_eq!(reader.line_at(0), "Hello");
        assert_eq!(reader.line_at(1), "World");
        assert_eq!(reader.line_at(2), "");
    }

    #[test]
    fn test_is_valid_position() {
        let reader = MockGridReader {
            data: vec![vec!['A'; 80]; 24],
        };

        assert!(reader.is_valid_position(Position::new(0, 0)));
        assert!(reader.is_valid_position(Position::new(79, 23)));
        assert!(!reader.is_valid_position(Position::new(80, 0)));
        assert!(!reader.is_valid_position(Position::new(0, 24)));
    }
}
