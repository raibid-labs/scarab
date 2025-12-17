/// Comprehensive unit tests for VTE parser integration
///
/// Tests cover:
/// - Basic text printing
/// - ANSI color codes (8, 16, 256 colors)
/// - Cursor movement and positioning
/// - Screen clearing and line clearing
/// - Text attributes (bold, italic, underline, inverse)
/// - Scrollback buffer
/// - UTF-8 multibyte characters
/// - Edge cases and error handling

#[cfg(test)]
mod tests {
    use scarab_daemon::vte::*;
    use scarab_protocol::{Cell, SharedState, GRID_HEIGHT, GRID_WIDTH};
    use std::sync::atomic::AtomicU64;
    use std::sync::Arc;

    /// Helper to create a test terminal state with shared memory for verification
    fn create_test_terminal() -> (Box<SharedState>, TerminalState, Arc<AtomicU64>) {
        let state = Box::new(unsafe { std::mem::zeroed::<SharedState>() });
        let sequence_counter = Arc::new(AtomicU64::new(0));
        let terminal = TerminalState::new(GRID_WIDTH as u16, GRID_HEIGHT as u16);
        (state, terminal, sequence_counter)
    }

    /// Helper to blit terminal to shared state and get cell content at position
    fn blit_and_get_cell(
        terminal: &mut TerminalState,
        state: &mut SharedState,
        seq: &Arc<AtomicU64>,
        x: usize,
        y: usize,
    ) -> Cell {
        let ptr = state as *mut SharedState;
        // SAFETY: ptr points to valid SharedState on the stack
        unsafe { terminal.blit_to_shm(ptr, seq) };
        let idx = y * GRID_WIDTH + x;
        state.cells[idx]
    }

    /// Helper to get cell directly from terminal's grid
    fn get_grid_cell(terminal: &TerminalState, x: u16, y: u16) -> Option<&Cell> {
        terminal.grid.get(x, y)
    }

    #[test]
    fn test_basic_text_printing() {
        let (mut state, mut terminal, seq) = create_test_terminal();

        // Print "Hello"
        terminal.process_output(b"Hello");

        // Verify characters in local grid
        assert_eq!(
            get_grid_cell(&terminal, 0, 0).unwrap().char_codepoint,
            'H' as u32
        );
        assert_eq!(
            get_grid_cell(&terminal, 1, 0).unwrap().char_codepoint,
            'e' as u32
        );
        assert_eq!(
            get_grid_cell(&terminal, 2, 0).unwrap().char_codepoint,
            'l' as u32
        );
        assert_eq!(
            get_grid_cell(&terminal, 3, 0).unwrap().char_codepoint,
            'l' as u32
        );
        assert_eq!(
            get_grid_cell(&terminal, 4, 0).unwrap().char_codepoint,
            'o' as u32
        );

        // Cursor should be at position 5
        assert_eq!(terminal.cursor_x, 5);
        assert_eq!(terminal.cursor_y, 0);

        // Also verify blit works
        // SAFETY: state is a valid SharedState on the stack
        unsafe { terminal.blit_to_shm(&mut *state as *mut SharedState, &seq) };
        assert_eq!(state.cursor_x, 5);
        assert_eq!(state.cursor_y, 0);
    }

    #[test]
    fn test_newline_and_carriage_return() {
        let (_state, mut terminal, _seq) = create_test_terminal();

        // Print "Line1\nLine2\n"
        terminal.process_output(b"Line1\nLine2\n");

        // Verify first line
        assert_eq!(
            get_grid_cell(&terminal, 0, 0).unwrap().char_codepoint,
            'L' as u32
        );
        assert_eq!(
            get_grid_cell(&terminal, 4, 0).unwrap().char_codepoint,
            '1' as u32
        );

        // Verify second line
        assert_eq!(
            get_grid_cell(&terminal, 0, 1).unwrap().char_codepoint,
            'L' as u32
        );
        assert_eq!(
            get_grid_cell(&terminal, 4, 1).unwrap().char_codepoint,
            '2' as u32
        );

        // Cursor should be on line 3
        assert_eq!(terminal.cursor_y, 2);
    }

    #[test]
    fn test_carriage_return() {
        let (_state, mut terminal, _seq) = create_test_terminal();

        // Print "Hello\rWorld"
        terminal.process_output(b"Hello\rWorld");

        // "World" should overwrite "Hello" at the beginning
        assert_eq!(
            get_grid_cell(&terminal, 0, 0).unwrap().char_codepoint,
            'W' as u32
        );
        assert_eq!(
            get_grid_cell(&terminal, 1, 0).unwrap().char_codepoint,
            'o' as u32
        );
        assert_eq!(
            get_grid_cell(&terminal, 2, 0).unwrap().char_codepoint,
            'r' as u32
        );
        assert_eq!(
            get_grid_cell(&terminal, 3, 0).unwrap().char_codepoint,
            'l' as u32
        );
        assert_eq!(
            get_grid_cell(&terminal, 4, 0).unwrap().char_codepoint,
            'd' as u32
        );

        // Cursor should be at position 5
        assert_eq!(terminal.cursor_x, 5);
    }

    #[test]
    fn test_ansi_colors() {
        let (_state, mut terminal, _seq) = create_test_terminal();

        // Red text
        terminal.process_output(b"\x1b[31mRed\x1b[0m");

        // Check that "Red" has red foreground color
        let red_cell = get_grid_cell(&terminal, 0, 0).unwrap();
        assert_eq!(red_cell.char_codepoint, 'R' as u32);
        assert_eq!(red_cell.fg, 0xFFCD0000); // Red color

        // After reset, should use default color
        terminal.process_output(b"Normal");
        let normal_cell = get_grid_cell(&terminal, 3, 0).unwrap();
        assert_eq!(normal_cell.fg, 0xFFCCCCCC); // Default foreground
    }

    #[test]
    fn test_bright_colors() {
        let (_state, mut terminal, _seq) = create_test_terminal();

        // Bright red text
        terminal.process_output(b"\x1b[91mBrightRed\x1b[0m");

        // Check bright red color
        let cell = get_grid_cell(&terminal, 0, 0).unwrap();
        assert_eq!(cell.fg, 0xFFFF0000); // Bright red
    }

    #[test]
    fn test_256_colors() {
        let (_state, mut terminal, _seq) = create_test_terminal();

        // 256-color mode (color index 196 = bright red)
        terminal.process_output(b"\x1b[38;5;196mColor\x1b[0m");

        // Verify the character was written
        let cell = get_grid_cell(&terminal, 0, 0).unwrap();
        assert_eq!(cell.char_codepoint, 'C' as u32);
        // Color should be non-default
        assert_ne!(cell.fg, 0xFFCCCCCC);
    }

    #[test]
    fn test_true_color() {
        let (_state, mut terminal, _seq) = create_test_terminal();

        // 24-bit true color foreground: RGB(255, 128, 64) = orange
        terminal.process_output(b"\x1b[38;2;255;128;64mTrueColor\x1b[0m");

        let cell = get_grid_cell(&terminal, 0, 0).unwrap();
        assert_eq!(cell.char_codepoint, 'T' as u32);
        // 0xFF000000 | (255 << 16) | (128 << 8) | 64 = 0xFFFF8040
        assert_eq!(cell.fg, 0xFFFF8040);

        // Reset and test background true color
        terminal.process_output(b"\x1b[2J\x1b[H"); // Clear screen, home cursor
        terminal.process_output(b"\x1b[48;2;0;100;200mBlueBG\x1b[0m");

        let cell = get_grid_cell(&terminal, 0, 0).unwrap();
        // 0xFF000000 | (0 << 16) | (100 << 8) | 200 = 0xFF0064C8
        assert_eq!(cell.bg, 0xFF0064C8);
    }

    #[test]
    fn test_background_colors() {
        let (_state, mut terminal, _seq) = create_test_terminal();

        // Blue background
        terminal.process_output(b"\x1b[44mBlue BG\x1b[0m");

        // Check background color
        let cell = get_grid_cell(&terminal, 0, 0).unwrap();
        assert_eq!(cell.bg, 0xFF0000EE); // Blue background
    }

    #[test]
    fn test_text_attributes() {
        let (_state, mut terminal, _seq) = create_test_terminal();

        // Bold text
        terminal.process_output(b"\x1b[1mBold\x1b[0m");
        let bold_cell = get_grid_cell(&terminal, 0, 0).unwrap();
        assert_eq!(bold_cell.flags & FLAG_BOLD, FLAG_BOLD);

        // Italic text
        terminal.process_output(b" \x1b[3mItalic\x1b[0m");
        let italic_cell = get_grid_cell(&terminal, 5, 0).unwrap();
        assert_eq!(italic_cell.flags & FLAG_ITALIC, FLAG_ITALIC);

        // Underline text
        terminal.process_output(b" \x1b[4mUnder\x1b[0m");
        let under_cell = get_grid_cell(&terminal, 12, 0).unwrap();
        assert_eq!(under_cell.flags & FLAG_UNDERLINE, FLAG_UNDERLINE);
    }

    #[test]
    fn test_cursor_movement() {
        let (mut state, mut terminal, seq) = create_test_terminal();

        // Move cursor to position (5, 3)
        terminal.process_output(b"\x1b[4;6H");

        assert_eq!(terminal.cursor_x, 5);
        assert_eq!(terminal.cursor_y, 3);

        // Write at that position
        terminal.process_output(b"X");
        let cell = get_grid_cell(&terminal, 5, 3).unwrap();
        assert_eq!(cell.char_codepoint, 'X' as u32);

        // Verify via blit
        // SAFETY: state is a valid SharedState on the stack
        unsafe { terminal.blit_to_shm(&mut *state as *mut SharedState, &seq) };
        assert_eq!(state.cursor_x, 6); // Moved forward after writing
        assert_eq!(state.cursor_y, 3);
    }

    #[test]
    fn test_cursor_up_down_left_right() {
        let (_state, mut terminal, _seq) = create_test_terminal();

        // Start at (0, 0), move down 5 times
        terminal.process_output(b"\x1b[5B");
        assert_eq!(terminal.cursor_y, 5);

        // Move right 10 times
        terminal.process_output(b"\x1b[10C");
        assert_eq!(terminal.cursor_x, 10);

        // Move up 2 times
        terminal.process_output(b"\x1b[2A");
        assert_eq!(terminal.cursor_y, 3);

        // Move left 5 times
        terminal.process_output(b"\x1b[5D");
        assert_eq!(terminal.cursor_x, 5);
    }

    #[test]
    fn test_clear_screen() {
        let (_state, mut terminal, _seq) = create_test_terminal();

        // Fill screen with text
        terminal.process_output(b"Hello World");

        // Clear screen
        terminal.process_output(b"\x1b[2J");

        // All cells should be empty
        let cols = GRID_WIDTH.min(terminal.grid.cols as usize);
        let rows = GRID_HEIGHT.min(terminal.grid.rows as usize);
        for y in 0..rows {
            for x in 0..cols {
                let cell = get_grid_cell(&terminal, x as u16, y as u16).unwrap();
                assert_eq!(cell.char_codepoint, 0);
            }
        }

        // Cursor should be at origin
        assert_eq!(terminal.cursor_x, 0);
        assert_eq!(terminal.cursor_y, 0);
    }

    #[test]
    fn test_clear_line() {
        let (_state, mut terminal, _seq) = create_test_terminal();

        // Write a line
        terminal.process_output(b"Hello World");

        // Move cursor back
        terminal.process_output(b"\x1b[6D");

        // Clear to end of line
        terminal.process_output(b"\x1b[K");

        // "Hello " should remain, "World" should be cleared
        assert_eq!(
            get_grid_cell(&terminal, 0, 0).unwrap().char_codepoint,
            'H' as u32
        );
        assert_eq!(get_grid_cell(&terminal, 5, 0).unwrap().char_codepoint, 0);
        assert_eq!(get_grid_cell(&terminal, 6, 0).unwrap().char_codepoint, 0);
    }

    #[test]
    fn test_save_restore_cursor() {
        let (_state, mut terminal, _seq) = create_test_terminal();

        // Move to (10, 5)
        terminal.process_output(b"\x1b[6;11H");

        // Save cursor position
        terminal.process_output(b"\x1b[s");

        // Move elsewhere
        terminal.process_output(b"\x1b[1;1H");
        assert_eq!(terminal.cursor_x, 0);
        assert_eq!(terminal.cursor_y, 0);

        // Restore cursor
        terminal.process_output(b"\x1b[u");
        assert_eq!(terminal.cursor_x, 10);
        assert_eq!(terminal.cursor_y, 5);
    }

    #[test]
    fn test_line_wrapping() {
        let (_state, mut terminal, _seq) = create_test_terminal();

        // Create a string longer than screen width
        let cols = terminal.grid.cols as usize;
        let long_text = "A".repeat(cols + 10);
        terminal.process_output(long_text.as_bytes());

        // First line should be full
        for x in 0..cols {
            assert_eq!(
                get_grid_cell(&terminal, x as u16, 0)
                    .unwrap()
                    .char_codepoint,
                'A' as u32
            );
        }

        // Text should wrap to second line
        for x in 0..10 {
            assert_eq!(
                get_grid_cell(&terminal, x as u16, 1)
                    .unwrap()
                    .char_codepoint,
                'A' as u32
            );
        }
    }

    #[test]
    fn test_scrolling() {
        let (_state, mut terminal, _seq) = create_test_terminal();

        let rows = terminal.grid.rows as usize;

        // Fill screen with numbered lines
        for i in 0..rows + 5 {
            terminal.process_output(format!("Line {}\n", i).as_bytes());
        }

        // First line should contain "Line 5" (lines 0-4 scrolled off)
        assert_eq!(
            get_grid_cell(&terminal, 0, 0).unwrap().char_codepoint,
            'L' as u32
        );

        // Cursor should be near bottom
        assert!(terminal.cursor_y >= rows as u16 - 10);
    }

    #[test]
    fn test_utf8_characters() {
        let (_state, mut terminal, _seq) = create_test_terminal();

        // Test various UTF-8 characters
        terminal.process_output("Hello 世界! 🚀".as_bytes());

        // Verify ASCII characters
        assert_eq!(
            get_grid_cell(&terminal, 0, 0).unwrap().char_codepoint,
            'H' as u32
        );

        // Verify UTF-8 characters were stored
        let mut has_chinese = false;
        for x in 0..20 {
            let codepoint = get_grid_cell(&terminal, x, 0).unwrap().char_codepoint;
            if codepoint == '世' as u32 || codepoint == '界' as u32 {
                has_chinese = true;
                break;
            }
        }
        assert!(has_chinese, "Should contain Chinese characters");
    }

    #[test]
    fn test_tab_character() {
        let (_state, mut terminal, _seq) = create_test_terminal();

        // Tab should advance to next tab stop (every 8 columns)
        terminal.process_output(b"A\tB");

        assert_eq!(
            get_grid_cell(&terminal, 0, 0).unwrap().char_codepoint,
            'A' as u32
        );
        assert_eq!(
            get_grid_cell(&terminal, 8, 0).unwrap().char_codepoint,
            'B' as u32
        );
    }

    #[test]
    fn test_backspace() {
        let (_state, mut terminal, _seq) = create_test_terminal();

        terminal.process_output(b"Hello\x08\x08World");

        // "World" should overwrite last 2 chars of "Hello"
        assert_eq!(
            get_grid_cell(&terminal, 0, 0).unwrap().char_codepoint,
            'H' as u32
        );
        assert_eq!(
            get_grid_cell(&terminal, 1, 0).unwrap().char_codepoint,
            'e' as u32
        );
        assert_eq!(
            get_grid_cell(&terminal, 2, 0).unwrap().char_codepoint,
            'l' as u32
        );
        assert_eq!(
            get_grid_cell(&terminal, 3, 0).unwrap().char_codepoint,
            'W' as u32
        );
    }

    #[test]
    fn test_combined_attributes() {
        let (_state, mut terminal, _seq) = create_test_terminal();

        // Bold + Red + Underline
        terminal.process_output(b"\x1b[1;31;4mBoldRedUnder\x1b[0m");

        let cell = get_grid_cell(&terminal, 0, 0).unwrap();
        assert_eq!(cell.flags & FLAG_BOLD, FLAG_BOLD);
        assert_eq!(cell.flags & FLAG_UNDERLINE, FLAG_UNDERLINE);
        assert_eq!(cell.fg, 0xFFCD0000); // Red
    }

    #[test]
    fn test_inverse_attribute() {
        let (_state, mut terminal, _seq) = create_test_terminal();

        // Inverse text (swap fg/bg)
        terminal.process_output(b"\x1b[7mInverse\x1b[0m");

        let cell = get_grid_cell(&terminal, 0, 0).unwrap();
        assert_eq!(cell.flags & FLAG_INVERSE, FLAG_INVERSE);
    }

    #[test]
    fn test_partial_escape_sequences() {
        let (_state, mut terminal, _seq) = create_test_terminal();

        // Send escape sequence in chunks
        terminal.process_output(b"\x1b");
        terminal.process_output(b"[");
        terminal.process_output(b"31");
        terminal.process_output(b"m");
        terminal.process_output(b"Red");

        let cell = get_grid_cell(&terminal, 0, 0).unwrap();
        assert_eq!(cell.char_codepoint, 'R' as u32);
        assert_eq!(cell.fg, 0xFFCD0000); // Red
    }

    #[test]
    fn test_blit_updates_shared_state() {
        let (mut state, mut terminal, seq) = create_test_terminal();

        let initial_seq = state.sequence_number;

        // Process some output
        terminal.process_output(b"Test");

        // Blit to shared memory
        // SAFETY: state is a valid SharedState on the stack
        unsafe { terminal.blit_to_shm(&mut *state as *mut SharedState, &seq) };

        // Dirty flag should be set
        assert_eq!(state.dirty_flag, 1);

        // Sequence number should have incremented
        assert!(state.sequence_number > initial_seq);

        // Cursor should be updated
        assert_eq!(state.cursor_x, 4);
        assert_eq!(state.cursor_y, 0);
    }

    #[test]
    fn test_cursor_bounds_checking() {
        let (_state, mut terminal, _seq) = create_test_terminal();

        let cols = terminal.grid.cols;
        let rows = terminal.grid.rows;

        // Try to move cursor beyond screen bounds
        terminal.process_output(format!("\x1b[1000;1000H").as_bytes());

        // Cursor should be clamped to screen size
        assert!(terminal.cursor_x < cols);
        assert!(terminal.cursor_y < rows);
    }

    #[test]
    fn test_large_output_performance() {
        let (_state, mut terminal, _seq) = create_test_terminal();

        // Generate large output (simulate scrolling)
        let large_text = "A".repeat(100_000);

        // This should not panic or cause performance issues
        terminal.process_output(large_text.as_bytes());

        // Test completed successfully if we reach here
    }
}
