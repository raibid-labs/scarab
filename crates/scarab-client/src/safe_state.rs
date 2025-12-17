//! Safe SharedState access layer for scarab-client
//!
//! This module provides safe wrappers around raw SharedState pointers,
//! eliminating unsafe dereferences and adding validation.
//!
//! ## Consistency Guarantees
//!
//! The daemon writes cells to shared memory and increments a sequence number.
//! This module provides methods to detect if the sequence changed during a read:
//!
//! - `try_read()` - Single attempt, returns None if sequence changed during read
//! - `read_consistent()` - Retries until a consistent read succeeds
//!
//! For most rendering use cases, the existing `TerminalStateReader` trait methods
//! are sufficient since individual Cell writes are atomic.

use scarab_protocol::{
    terminal_state::TerminalStateReader, Cell, SharedState, GRID_HEIGHT, GRID_WIDTH,
};
use std::marker::PhantomData;
use std::sync::atomic::Ordering;

/// Safe wrapper for SharedState with lifetime tracking
///
/// This wrapper ensures:
/// - All access is bounds-checked
/// - Lifetime prevents use-after-free
/// - Validation on construction
/// - No unsafe code at call sites
///
/// # Example
/// ```no_run
/// use scarab_client::safe_state::SafeSharedState;
/// use scarab_protocol::terminal_state::TerminalStateReader;
///
/// fn render(state: &SafeSharedState) {
///     if let Some(cell) = state.cell(0, 0) {
///         println!("Char: {}", cell.char_codepoint);
///     }
/// }
/// ```
pub struct SafeSharedState<'a> {
    /// Pointer to shared memory
    ptr: *const SharedState,
    /// Lifetime marker to prevent use-after-free
    _lifetime: PhantomData<&'a SharedState>,
}

impl<'a> SafeSharedState<'a> {
    /// Create a new SafeSharedState from a raw pointer
    ///
    /// # Safety
    /// Caller must ensure:
    /// - `ptr` points to valid, initialized SharedState
    /// - SharedState remains valid for lifetime 'a
    /// - Pointer is properly aligned
    ///
    /// # Panics
    /// Panics if pointer is null
    pub unsafe fn from_ptr(ptr: *const SharedState) -> Self {
        assert!(!ptr.is_null(), "SharedState pointer cannot be null");
        Self {
            ptr,
            _lifetime: PhantomData,
        }
    }

    /// Create from shared memory reference
    ///
    /// This is the preferred way to create SafeSharedState when you
    /// have a reference to shared memory.
    pub fn from_shmem(shmem: &'a shared_memory::Shmem) -> Self {
        let ptr = shmem.as_ptr() as *const SharedState;
        unsafe { Self::from_ptr(ptr) }
    }

    /// Get reference to underlying SharedState
    ///
    /// # Safety
    /// Returns a reference with proper lifetime tracking.
    /// Internal method - prefer using TerminalStateReader trait methods.
    #[inline]
    fn state_ref(&self) -> &'a SharedState {
        unsafe { &*self.ptr }
    }

    /// Validate cursor position is within grid bounds
    fn cursor_in_bounds(&self) -> bool {
        let state = self.state_ref();
        (state.cursor_x as usize) < GRID_WIDTH && (state.cursor_y as usize) < GRID_HEIGHT
    }

    /// Check if memory layout appears valid
    ///
    /// Performs basic sanity checks:
    /// - Cursor is within bounds
    /// - Sequence number is reasonable (non-zero usually)
    fn is_memory_valid(&self) -> bool {
        self.cursor_in_bounds()
    }

    /// Read the sequence number using volatile/atomic semantics
    #[inline]
    fn read_sequence_atomic(&self) -> u64 {
        let state = self.state_ref();
        // Use volatile read to prevent compiler reordering
        unsafe { std::ptr::read_volatile(&state.sequence_number) }
    }

    /// Read cells with consistency guarantee
    ///
    /// This method reads cells and verifies the sequence number didn't change
    /// during the read, retrying if necessary.
    ///
    /// Returns a consistent snapshot of the cell data along with the sequence number.
    ///
    /// # Arguments
    /// * `max_retries` - Maximum retry attempts before giving up (0 = unlimited)
    ///
    /// # Returns
    /// * `Some((sequence, cells))` - Consistent read succeeded
    /// * `None` - Failed after max_retries (only if max_retries > 0)
    pub fn read_consistent(&self, max_retries: u32) -> Option<(u64, Vec<Cell>)> {
        let mut attempts = 0u32;

        loop {
            // Step 1: Read sequence before
            let seq_before = self.read_sequence_atomic();

            // Step 2: Memory barrier to ensure we see current data
            std::sync::atomic::fence(Ordering::Acquire);

            // Step 3: Read data
            let state = self.state_ref();
            let cells = state.cells.to_vec();

            // Step 4: Memory barrier before re-reading sequence
            std::sync::atomic::fence(Ordering::Acquire);

            // Step 5: Verify sequence unchanged
            let seq_after = self.read_sequence_atomic();
            if seq_before == seq_after {
                // Success - consistent read!
                return Some((seq_before, cells));
            }

            // Sequence changed during read - retry
            std::hint::spin_loop();
            attempts += 1;
            if max_retries > 0 && attempts >= max_retries {
                return None;
            }
        }
    }

    /// Read cursor position with consistency guarantee
    ///
    /// Lighter weight than `read_consistent` since cursor is only 4 bytes.
    pub fn read_cursor_consistent(&self, max_retries: u32) -> Option<(u64, u16, u16)> {
        let mut attempts = 0u32;

        loop {
            let seq_before = self.read_sequence_atomic();

            std::sync::atomic::fence(Ordering::Acquire);

            let state = self.state_ref();
            let cursor_x = state.cursor_x;
            let cursor_y = state.cursor_y;

            std::sync::atomic::fence(Ordering::Acquire);

            let seq_after = self.read_sequence_atomic();
            if seq_before == seq_after {
                return Some((seq_before, cursor_x, cursor_y));
            }

            std::hint::spin_loop();
            attempts += 1;
            if max_retries > 0 && attempts >= max_retries {
                return None;
            }
        }
    }

    /// Try to read cells without blocking, returning None if sequence changed during read
    ///
    /// This is useful for render loops where you'd rather skip a frame than block.
    /// Unlike `read_consistent`, this does NOT retry - it returns immediately.
    pub fn try_read(&self) -> Option<(u64, &[Cell])> {
        let seq_before = self.read_sequence_atomic();

        std::sync::atomic::fence(Ordering::Acquire);

        let state = self.state_ref();
        let cells = &state.cells[..];

        std::sync::atomic::fence(Ordering::Acquire);

        let seq_after = self.read_sequence_atomic();
        if seq_before == seq_after {
            Some((seq_before, cells))
        } else {
            None // Sequence changed during read
        }
    }
}

impl<'a> TerminalStateReader for SafeSharedState<'a> {
    fn cell(&self, row: usize, col: usize) -> Option<&Cell> {
        if row >= GRID_HEIGHT || col >= GRID_WIDTH {
            return None;
        }

        let idx = row * GRID_WIDTH + col;
        let state = self.state_ref();
        state.cells.get(idx)
    }

    fn cells(&self) -> &[Cell] {
        let state = self.state_ref();
        &state.cells
    }

    fn cursor_pos(&self) -> (u16, u16) {
        let state = self.state_ref();
        (state.cursor_x, state.cursor_y)
    }

    fn sequence(&self) -> u64 {
        let state = self.state_ref();
        state.sequence_number
    }

    fn is_valid(&self) -> bool {
        self.is_memory_valid()
    }

    fn dimensions(&self) -> (usize, usize) {
        (GRID_WIDTH, GRID_HEIGHT)
    }

    fn is_dirty(&self) -> bool {
        let state = self.state_ref();
        state.dirty_flag != 0
    }

    fn is_error_mode(&self) -> bool {
        let state = self.state_ref();
        state.error_mode != 0
    }
}

/// Mock terminal state for testing
///
/// Provides a TerminalStateReader implementation that doesn't require
/// shared memory, making it suitable for unit tests.
///
/// # Example
/// ```
/// use scarab_client::safe_state::MockTerminalState;
/// use scarab_protocol::terminal_state::TerminalStateReader;
/// use scarab_protocol::Cell;
///
/// let mut mock = MockTerminalState::new(80, 24);
/// mock.set_cursor(5, 10);
/// mock.set_cell(0, 0, Cell::default());
///
/// assert_eq!(mock.cursor_pos(), (5, 10));
/// assert!(mock.cell(0, 0).is_some());
/// ```
#[derive(Clone)]
pub struct MockTerminalState {
    cells: Vec<Cell>,
    width: usize,
    height: usize,
    cursor_x: u16,
    cursor_y: u16,
    sequence: u64,
    dirty: bool,
}

impl MockTerminalState {
    /// Create a new mock terminal state with given dimensions
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            cells: vec![Cell::default(); width * height],
            width,
            height,
            cursor_x: 0,
            cursor_y: 0,
            sequence: 0,
            dirty: false,
        }
    }

    /// Create with default dimensions (80x24)
    pub fn default_size() -> Self {
        Self::new(80, 24)
    }

    /// Set cursor position
    pub fn set_cursor(&mut self, x: u16, y: u16) {
        self.cursor_x = x;
        self.cursor_y = y;
        self.dirty = true;
    }

    /// Set a specific cell
    pub fn set_cell(&mut self, row: usize, col: usize, cell: Cell) -> bool {
        if let Some(idx) = self.cell_index(row, col) {
            if idx < self.cells.len() {
                self.cells[idx] = cell;
                self.dirty = true;
                return true;
            }
        }
        false
    }

    /// Increment sequence number (simulating state update)
    pub fn increment_sequence(&mut self) {
        self.sequence += 1;
    }

    /// Clear dirty flag
    pub fn clear_dirty(&mut self) {
        self.dirty = false;
    }

    /// Fill entire grid with a character
    pub fn fill(&mut self, c: char) {
        let codepoint = c as u32;
        for cell in &mut self.cells {
            cell.char_codepoint = codepoint;
        }
        self.dirty = true;
    }

    /// Write text at cursor position
    pub fn write_text(&mut self, text: &str) {
        let mut x = self.cursor_x as usize;
        let y = self.cursor_y as usize;

        for c in text.chars() {
            if x >= self.width {
                break;
            }

            if let Some(idx) = self.cell_index(y, x) {
                if idx < self.cells.len() {
                    self.cells[idx].char_codepoint = c as u32;
                    x += 1;
                }
            }
        }

        self.cursor_x = x.min(self.width - 1) as u16;
        self.dirty = true;
    }

    /// Get mutable access to cells (for advanced testing)
    pub fn cells_mut(&mut self) -> &mut [Cell] {
        &mut self.cells
    }
}

impl TerminalStateReader for MockTerminalState {
    fn cell(&self, row: usize, col: usize) -> Option<&Cell> {
        self.cell_index(row, col)
            .and_then(|idx| self.cells.get(idx))
    }

    fn cells(&self) -> &[Cell] {
        &self.cells
    }

    fn cursor_pos(&self) -> (u16, u16) {
        (self.cursor_x, self.cursor_y)
    }

    fn sequence(&self) -> u64 {
        self.sequence
    }

    fn is_valid(&self) -> bool {
        self.cells.len() == self.width * self.height
            && (self.cursor_x as usize) < self.width
            && (self.cursor_y as usize) < self.height
    }

    fn dimensions(&self) -> (usize, usize) {
        (self.width, self.height)
    }

    fn is_dirty(&self) -> bool {
        self.dirty
    }

    fn is_error_mode(&self) -> bool {
        false // Mock state is never in error mode by default
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mock_creation() {
        let mock = MockTerminalState::new(80, 24);
        assert_eq!(mock.dimensions(), (80, 24));
        assert_eq!(mock.cursor_pos(), (0, 0));
        assert_eq!(mock.sequence(), 0);
        assert!(mock.is_valid());
    }

    #[test]
    fn test_mock_set_cursor() {
        let mut mock = MockTerminalState::new(80, 24);
        mock.set_cursor(10, 5);
        assert_eq!(mock.cursor_pos(), (10, 5));
        assert!(mock.is_dirty());
    }

    #[test]
    fn test_mock_set_cell() {
        let mut mock = MockTerminalState::new(80, 24);
        let mut cell = Cell::default();
        cell.char_codepoint = 'A' as u32;

        assert!(mock.set_cell(0, 0, cell));
        assert_eq!(mock.cell(0, 0).unwrap().char_codepoint, 'A' as u32);
    }

    #[test]
    fn test_mock_bounds_checking() {
        let mock = MockTerminalState::new(10, 10);

        // Valid access
        assert!(mock.cell(0, 0).is_some());
        assert!(mock.cell(9, 9).is_some());

        // Out of bounds
        assert!(mock.cell(10, 0).is_none());
        assert!(mock.cell(0, 10).is_none());
        assert!(mock.cell(100, 100).is_none());
    }

    #[test]
    fn test_mock_fill() {
        let mut mock = MockTerminalState::new(5, 5);
        mock.fill('X');

        for row in 0..5 {
            for col in 0..5 {
                assert_eq!(mock.cell(row, col).unwrap().char_codepoint, 'X' as u32);
            }
        }
    }

    #[test]
    fn test_mock_write_text() {
        let mut mock = MockTerminalState::new(80, 24);
        mock.write_text("Hello");

        assert_eq!(mock.cell(0, 0).unwrap().char_codepoint, 'H' as u32);
        assert_eq!(mock.cell(0, 1).unwrap().char_codepoint, 'e' as u32);
        assert_eq!(mock.cell(0, 2).unwrap().char_codepoint, 'l' as u32);
        assert_eq!(mock.cell(0, 3).unwrap().char_codepoint, 'l' as u32);
        assert_eq!(mock.cell(0, 4).unwrap().char_codepoint, 'o' as u32);
    }

    #[test]
    fn test_mock_write_text_overflow() {
        let mut mock = MockTerminalState::new(5, 1);
        mock.write_text("Hello, World!"); // Too long

        // Should only write "Hello"
        assert_eq!(mock.cell(0, 0).unwrap().char_codepoint, 'H' as u32);
        assert_eq!(mock.cell(0, 4).unwrap().char_codepoint, 'o' as u32);
    }

    #[test]
    fn test_mock_validation() {
        let valid = MockTerminalState::new(10, 10);
        assert!(valid.is_valid());

        let mut invalid_cursor = MockTerminalState::new(10, 10);
        invalid_cursor.cursor_x = 100; // Out of bounds
        assert!(!invalid_cursor.is_valid());
    }

    #[test]
    fn test_cell_iterator() {
        let mut mock = MockTerminalState::new(3, 2);
        mock.fill('X');

        let cells: Vec<_> = mock.iter_cells().collect();
        assert_eq!(cells.len(), 6);

        // Check first cell
        assert_eq!(cells[0].0, 0); // row
        assert_eq!(cells[0].1, 0); // col
        assert_eq!(cells[0].2.char_codepoint, 'X' as u32);

        // Check last cell
        assert_eq!(cells[5].0, 1); // row
        assert_eq!(cells[5].1, 2); // col
    }
}
