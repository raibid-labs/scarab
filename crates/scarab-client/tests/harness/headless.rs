//! HeadlessHarness with grid snapshot capabilities
//!
//! This module provides a wrapper around HeadlessTestHarness with additional
//! terminal-specific functionality for grid testing and snapshot capture.

use super::mocks::MockSharedMemoryReader;
use super::HeadlessTestHarness;
use bevy::prelude::*;
use scarab_protocol::{GRID_HEIGHT, GRID_WIDTH};

/// Headless test harness with grid snapshot capabilities.
///
/// This extends the basic `HeadlessTestHarness` with terminal-specific
/// functionality for testing grid rendering and snapshot capture.
///
/// ## Example
///
/// ```rust,no_run
/// let mut harness = HeadlessHarness::new();
/// harness.set_grid_text(0, 0, "test");
/// let snapshot = harness.capture_grid_snapshot();
/// assert!(snapshot.contains("test"));
/// ```
pub struct HeadlessHarness {
    /// The underlying Bevy app with headless configuration
    inner: HeadlessTestHarness,
}

impl HeadlessHarness {
    /// Create a new headless test harness.
    ///
    /// Initializes a Bevy app with:
    /// - MinimalPlugins (no window/GPU)
    /// - ScheduleRunnerPlugin for fast test execution
    /// - MockSharedMemoryReader for terminal grid simulation
    ///
    /// ## Example
    ///
    /// ```rust,no_run
    /// let harness = HeadlessHarness::new();
    /// ```
    pub fn new() -> Self {
        let inner = HeadlessTestHarness::new();

        Self { inner }
    }

    /// Create a headless harness with custom Bevy app setup.
    ///
    /// Allows adding custom plugins, systems, or resources before testing.
    ///
    /// ## Example
    ///
    /// ```rust,no_run
    /// let harness = HeadlessHarness::with_setup(|app| {
    ///     app.add_plugins(MyRenderingPlugin);
    /// });
    /// ```
    pub fn with_setup<F>(setup: F) -> Self
    where
        F: FnOnce(&mut App),
    {
        let inner = HeadlessTestHarness::with_setup(setup);

        Self { inner }
    }

    /// Run one Bevy update cycle.
    ///
    /// Executes all scheduled systems for one frame.
    pub fn update(&mut self) {
        self.inner.update();
    }

    /// Run multiple Bevy update cycles.
    ///
    /// Useful for testing multi-frame behaviors.
    pub fn update_n(&mut self, count: usize) {
        self.inner.update_n(count);
    }

    /// Set a single cell in the terminal grid.
    ///
    /// ## Arguments
    /// * `x` - Column position (0-indexed)
    /// * `y` - Row position (0-indexed)
    /// * `c` - Character to place
    /// * `fg` - Foreground color (RGBA as u32)
    /// * `bg` - Background color (RGBA as u32)
    ///
    /// ## Example
    ///
    /// ```rust,no_run
    /// harness.set_grid_cell(0, 0, 'A', 0xFFFFFFFF, 0x000000FF);
    /// ```
    pub fn set_grid_cell(&mut self, x: u16, y: u16, c: char, fg: u32, bg: u32) {
        let mut reader = self.inner.resource_mut::<MockSharedMemoryReader>();
        reader.set_cell(x, y, c, fg, bg);
    }

    /// Set a string of text in the terminal grid starting at position.
    ///
    /// Uses default colors (white on black).
    ///
    /// ## Example
    ///
    /// ```rust,no_run
    /// harness.set_grid_text(0, 0, "Hello, World!");
    /// ```
    pub fn set_grid_text(&mut self, x: u16, y: u16, text: &str) {
        self.set_grid_text_colored(x, y, text, 0xFFFFFFFF, 0x000000FF);
    }

    /// Set a string of text with custom colors.
    ///
    /// ## Example
    ///
    /// ```rust,no_run
    /// // Red text on black background
    /// harness.set_grid_text_colored(0, 0, "ERROR", 0xFF0000FF, 0x000000FF);
    /// ```
    pub fn set_grid_text_colored(&mut self, x: u16, y: u16, text: &str, fg: u32, bg: u32) {
        let mut reader = self.inner.resource_mut::<MockSharedMemoryReader>();
        reader.set_text(x, y, text, fg, bg);
    }

    /// Set the cursor position in the terminal grid.
    ///
    /// ## Example
    ///
    /// ```rust,no_run
    /// harness.set_cursor(10, 5);
    /// ```
    pub fn set_cursor(&mut self, x: u16, y: u16) {
        let mut reader = self.inner.resource_mut::<MockSharedMemoryReader>();
        reader.set_cursor(x, y);
    }

    /// Increment the grid sequence number.
    ///
    /// Simulates a daemon update to the shared memory state.
    /// This triggers client-side synchronization logic.
    ///
    /// ## Example
    ///
    /// ```rust,no_run
    /// harness.set_grid_text(0, 0, "test");
    /// harness.tick_grid(); // Signal update
    /// ```
    pub fn tick_grid(&mut self) {
        let mut reader = self.inner.resource_mut::<MockSharedMemoryReader>();
        reader.tick();
    }

    /// Clear the entire terminal grid.
    ///
    /// Resets all cells to spaces and moves cursor to (0, 0).
    pub fn clear_grid(&mut self) {
        let mut reader = self.inner.resource_mut::<MockSharedMemoryReader>();
        reader.clear();
    }

    /// Get the character at a specific grid position.
    ///
    /// Returns `None` if position is out of bounds.
    pub fn get_grid_char(&self, x: u16, y: u16) -> Option<char> {
        let reader = self.inner.resource::<MockSharedMemoryReader>();
        reader.get_char(x, y)
    }

    /// Get the text content of an entire row.
    ///
    /// Returns empty string if row is out of bounds.
    /// Trailing spaces are trimmed.
    pub fn get_grid_row(&self, y: u16) -> String {
        let reader = self.inner.resource::<MockSharedMemoryReader>();
        reader.get_row_text(y)
    }

    /// Fill a rectangular region with a character.
    ///
    /// ## Example
    ///
    /// ```rust,no_run
    /// // Draw a 10x5 box of '#' characters
    /// harness.fill_grid_rect(0, 0, 10, 5, '#', 0xFFFFFFFF, 0x000000FF);
    /// ```
    pub fn fill_grid_rect(
        &mut self,
        x: u16,
        y: u16,
        width: u16,
        height: u16,
        c: char,
        fg: u32,
        bg: u32,
    ) {
        let mut reader = self.inner.resource_mut::<MockSharedMemoryReader>();
        reader.fill_rect(x, y, width, height, c, fg, bg);
    }

    /// Simulate multi-line terminal output.
    ///
    /// Convenience method for setting multiple lines at once.
    ///
    /// ## Example
    ///
    /// ```rust,no_run
    /// harness.simulate_output(&[
    ///     (0, 0, "user@host:~$"),
    ///     (0, 1, "ls -la"),
    ///     (0, 2, "total 42"),
    /// ], 0xFFFFFFFF, 0x000000FF);
    /// ```
    pub fn simulate_output(&mut self, lines: &[(u16, u16, &str)], fg: u32, bg: u32) {
        let mut reader = self.inner.resource_mut::<MockSharedMemoryReader>();
        reader.simulate_output(lines, fg, bg);
    }

    /// Capture the entire terminal grid as a text snapshot.
    ///
    /// This converts the grid state into a human-readable string format
    /// suitable for snapshot testing with `insta::assert_snapshot!`.
    ///
    /// ## Format
    ///
    /// The snapshot includes:
    /// - Grid dimensions header
    /// - Cursor position
    /// - All grid rows with visible content
    /// - Row numbers for debugging
    ///
    /// ## Example
    ///
    /// ```rust,no_run
    /// harness.set_grid_text(0, 0, "Hello");
    /// let snapshot = harness.capture_grid_snapshot();
    /// insta::assert_snapshot!(snapshot);
    /// ```
    pub fn capture_grid_snapshot(&self) -> String {
        let reader = self.inner.resource::<MockSharedMemoryReader>();
        let state = reader.get_state();

        let mut output = String::new();

        // Header with dimensions
        output.push_str(&format!(
            "=== Terminal Grid Snapshot ({} cols × {} rows) ===\n",
            GRID_WIDTH, GRID_HEIGHT
        ));
        output.push_str(&format!(
            "Cursor: ({}, {})\n",
            state.cursor_x, state.cursor_y
        ));
        output.push_str(&format!("Sequence: {}\n", state.sequence_number));
        output.push_str("---\n");

        // Capture all rows
        for y in 0..GRID_HEIGHT {
            let row_text = state.get_row_text(y as u16);

            // Skip empty trailing rows for cleaner snapshots
            if y > 0 && row_text.is_empty() {
                let mut all_empty = true;
                for future_y in y..GRID_HEIGHT {
                    if !state.get_row_text(future_y as u16).is_empty() {
                        all_empty = false;
                        break;
                    }
                }
                if all_empty {
                    output.push_str(&format!("... ({} empty rows omitted)\n", GRID_HEIGHT - y));
                    break;
                }
            }

            // Include row number for debugging
            if !row_text.is_empty() {
                output.push_str(&format!("{:3} | {}\n", y, row_text));
            }
        }

        output
    }

    /// Capture a specific region of the grid as a snapshot.
    ///
    /// Useful for testing specific areas without capturing the entire grid.
    ///
    /// ## Example
    ///
    /// ```rust,no_run
    /// // Capture top-left 20x10 region
    /// let snapshot = harness.capture_grid_region(0, 0, 20, 10);
    /// ```
    pub fn capture_grid_region(&self, x: u16, y: u16, width: u16, height: u16) -> String {
        let reader = self.inner.resource::<MockSharedMemoryReader>();
        let state = reader.get_state();

        let mut output = String::new();
        output.push_str(&format!(
            "=== Grid Region ({}, {}) {}×{} ===\n",
            x, y, width, height
        ));

        let max_y = (y + height).min(GRID_HEIGHT as u16);
        for row in y..max_y {
            let mut row_text = String::new();
            let max_x = (x + width).min(GRID_WIDTH as u16);

            for col in x..max_x {
                if let Some(c) = state.get_char(col, row) {
                    row_text.push(c);
                } else {
                    row_text.push(' ');
                }
            }

            let row_text = row_text.trim_end();
            if !row_text.is_empty() {
                output.push_str(&format!("{:3} | {}\n", row, row_text));
            }
        }

        output
    }

    /// Get access to the underlying HeadlessTestHarness.
    ///
    /// Allows using all standard harness features for Bevy testing.
    pub fn inner_mut(&mut self) -> &mut HeadlessTestHarness {
        &mut self.inner
    }

    /// Get immutable access to the underlying HeadlessTestHarness.
    pub fn inner(&self) -> &HeadlessTestHarness {
        &self.inner
    }

    /// Spawn a Bevy entity with components.
    ///
    /// Convenience wrapper around inner harness.
    pub fn spawn<B: Bundle>(&mut self, bundle: B) -> Entity {
        self.inner.spawn(bundle)
    }

    /// Get mutable access to the Bevy world.
    ///
    /// For advanced ECS operations.
    pub fn world_mut(&mut self) -> &mut World {
        self.inner.world_mut()
    }

    /// Get immutable access to the Bevy world.
    pub fn world(&self) -> &World {
        self.inner.world()
    }

    /// Get a reference to a Bevy resource.
    pub fn resource<R: Resource>(&self) -> &R {
        self.inner.resource::<R>()
    }

    /// Get a mutable reference to a Bevy resource.
    pub fn resource_mut<R: Resource>(&mut self) -> Mut<'_, R> {
        self.inner.resource_mut::<R>()
    }
}

impl Default for HeadlessHarness {
    fn default() -> Self {
        Self::new()
    }
}
