//! Safe abstraction layer for SharedState access
//!
//! This module provides a safe, validated interface for reading terminal state
//! from shared memory, eliminating unsafe raw pointer dereferences.
//!
//! # Safety Architecture
//!
//! The `TerminalStateReader` trait abstracts away direct SharedState access with:
//! - Bounds checking for all cell access
//! - Validation of magic numbers and memory integrity
//! - Lifetime tracking to prevent use-after-free
//! - Type-safe cursor and sequence number access
//!
//! # Usage Example
//!
//! ```no_run
//! use scarab_protocol::{SharedState, TerminalStateReader};
//!
//! fn render_terminal(state: &impl TerminalStateReader) {
//!     // Safe access with automatic bounds checking
//!     if let Some(cell) = state.cell(0, 0) {
//!         println!("Top-left cell: {:?}", cell.char_codepoint);
//!     }
//!
//!     let (cursor_x, cursor_y) = state.cursor_pos();
//!     println!("Cursor at: ({}, {})", cursor_x, cursor_y);
//! }
//! ```

use crate::{Cell, SharedState, GRID_HEIGHT, GRID_WIDTH};

/// Magic number for validating SharedState memory layout
///
/// This sentinel value helps detect:
/// - Uninitialized memory
/// - Memory corruption
/// - Process crashes that left invalid state
pub const SHARED_STATE_MAGIC: u64 = 0x5343_4152_4142_5348; // "SCARABSH" in hex

/// Safe, validated interface for reading terminal state
///
/// This trait provides the abstraction layer over SharedState that:
/// 1. Validates memory integrity before access
/// 2. Performs bounds checking on all cell access
/// 3. Provides ergonomic, type-safe getters
/// 4. Enables testing with mock implementations
pub trait TerminalStateReader {
    /// Get cell at position, returns None if out of bounds
    ///
    /// # Arguments
    /// * `row` - Zero-indexed row (0 to GRID_HEIGHT-1)
    /// * `col` - Zero-indexed column (0 to GRID_WIDTH-1)
    ///
    /// # Returns
    /// * `Some(&Cell)` if position is valid
    /// * `None` if out of bounds
    fn cell(&self, row: usize, col: usize) -> Option<&Cell>;

    /// Get all cells as a slice
    ///
    /// Returns the full grid buffer. Prefer using `cell()` with bounds
    /// checking when accessing individual cells.
    fn cells(&self) -> &[Cell];

    /// Get cursor position
    ///
    /// # Returns
    /// Tuple of (x, y) cursor coordinates in grid space
    fn cursor_pos(&self) -> (u16, u16);

    /// Get current sequence number
    ///
    /// The sequence number increments with each state update.
    /// Clients can poll this to detect changes.
    ///
    /// # Returns
    /// Monotonically increasing sequence counter
    fn sequence(&self) -> u64;

    /// Check if state is valid
    ///
    /// Validates:
    /// - Magic number matches expected value
    /// - Memory appears properly initialized
    /// - Cursor position is within bounds
    ///
    /// # Returns
    /// `true` if state passes validation checks
    fn is_valid(&self) -> bool;

    /// Get grid dimensions
    ///
    /// # Returns
    /// Tuple of (width, height) in cells
    fn dimensions(&self) -> (usize, usize);

    /// Check if dirty flag is set
    ///
    /// The dirty flag indicates pending updates that haven't been rendered.
    fn is_dirty(&self) -> bool;

    /// Get linear cell index from row/col coordinates
    ///
    /// # Arguments
    /// * `row` - Zero-indexed row
    /// * `col` - Zero-indexed column
    ///
    /// # Returns
    /// * `Some(index)` if coordinates are valid
    /// * `None` if out of bounds
    fn cell_index(&self, row: usize, col: usize) -> Option<usize> {
        let (width, height) = self.dimensions();
        if row >= height || col >= width {
            None
        } else {
            Some(row * width + col)
        }
    }

    /// Iterate over all cells with their coordinates
    ///
    /// Yields tuples of (row, col, &Cell) for convenient iteration.
    fn iter_cells(&self) -> CellIterator<'_, Self>
    where
        Self: Sized,
    {
        CellIterator {
            reader: self,
            index: 0,
        }
    }
}

/// Iterator over cells with coordinates
pub struct CellIterator<'a, R: TerminalStateReader> {
    reader: &'a R,
    index: usize,
}

impl<'a, R: TerminalStateReader> Iterator for CellIterator<'a, R> {
    type Item = (usize, usize, &'a Cell);

    fn next(&mut self) -> Option<Self::Item> {
        let (width, _height) = self.reader.dimensions();
        let cells = self.reader.cells();

        if self.index >= cells.len() {
            return None;
        }

        let row = self.index / width;
        let col = self.index % width;
        let cell = &cells[self.index];
        self.index += 1;

        Some((row, col, cell))
    }
}

/// Implementation note for SharedState
///
/// Due to `#[no_std]` constraint on scarab-protocol, we cannot directly
/// implement TerminalStateReader on SharedState here. The implementation
/// is provided in scarab-client as `SafeSharedState<'_>`.
///
/// This allows scarab-protocol to remain dependency-free while providing
/// the trait definition for both client and daemon.

#[cfg(test)]
mod tests {
    use super::*;

    // Mock implementation for testing
    struct MockState {
        cells: alloc::vec::Vec<Cell>,
        width: usize,
        height: usize,
        cursor: (u16, u16),
        sequence: u64,
    }

    impl TerminalStateReader for MockState {
        fn cell(&self, row: usize, col: usize) -> Option<&Cell> {
            self.cell_index(row, col)
                .and_then(|idx| self.cells.get(idx))
        }

        fn cells(&self) -> &[Cell] {
            &self.cells
        }

        fn cursor_pos(&self) -> (u16, u16) {
            self.cursor
        }

        fn sequence(&self) -> u64 {
            self.sequence
        }

        fn is_valid(&self) -> bool {
            self.cells.len() == self.width * self.height
                && (self.cursor.0 as usize) < self.width
                && (self.cursor.1 as usize) < self.height
        }

        fn dimensions(&self) -> (usize, usize) {
            (self.width, self.height)
        }

        fn is_dirty(&self) -> bool {
            false
        }
    }

    #[test]
    fn test_bounds_checking() {
        let mock = MockState {
            cells: alloc::vec![Cell::default(); 100],
            width: 10,
            height: 10,
            cursor: (5, 5),
            sequence: 42,
        };

        // Valid access
        assert!(mock.cell(0, 0).is_some());
        assert!(mock.cell(9, 9).is_some());

        // Out of bounds
        assert!(mock.cell(10, 0).is_none());
        assert!(mock.cell(0, 10).is_none());
        assert!(mock.cell(100, 100).is_none());
    }

    #[test]
    fn test_validation() {
        let valid = MockState {
            cells: alloc::vec![Cell::default(); 100],
            width: 10,
            height: 10,
            cursor: (5, 5),
            sequence: 42,
        };
        assert!(valid.is_valid());

        let invalid_cursor = MockState {
            cells: alloc::vec![Cell::default(); 100],
            width: 10,
            height: 10,
            cursor: (20, 5), // Out of bounds
            sequence: 42,
        };
        assert!(!invalid_cursor.is_valid());

        let invalid_size = MockState {
            cells: alloc::vec![Cell::default(); 50], // Wrong size
            width: 10,
            height: 10,
            cursor: (5, 5),
            sequence: 42,
        };
        assert!(!invalid_size.is_valid());
    }

    #[test]
    fn test_iterator() {
        let mut cells = alloc::vec![Cell::default(); 6];
        for i in 0..6 {
            cells[i].char_codepoint = (b'A' + i as u8) as u32;
        }

        let mock = MockState {
            cells,
            width: 3,
            height: 2,
            cursor: (0, 0),
            sequence: 1,
        };

        let collected: alloc::vec::Vec<_> = mock.iter_cells().collect();
        assert_eq!(collected.len(), 6);

        // Check first cell (0, 0)
        assert_eq!(collected[0].0, 0); // row
        assert_eq!(collected[0].1, 0); // col
        assert_eq!(collected[0].2.char_codepoint, b'A' as u32);

        // Check last cell (1, 2)
        assert_eq!(collected[5].0, 1); // row
        assert_eq!(collected[5].1, 2); // col
        assert_eq!(collected[5].2.char_codepoint, b'F' as u32);
    }
}

// Need alloc for tests with Vec
#[cfg(test)]
extern crate alloc;
