//! Smoke tests for scarab-daemon
//!
//! These tests verify daemon functionality including plugin loading,
//! terminal processing, and IPC communication.

#[cfg(test)]
mod smoke_tests {
    use scarab_daemon::profiling::MetricsCollector;
    use scarab_daemon::vte::{TerminalState, FLAG_BOLD, FLAG_UNDERLINE};
    use scarab_protocol::{SharedState, GRID_HEIGHT, GRID_WIDTH};
    use std::sync::atomic::AtomicU64;
    use std::sync::Arc;
    use std::time::Duration;

    /// Test that SharedState structure is properly initialized with correct header fields
    #[test]
    fn test_shared_memory_initialization() {
        // Create a SharedState on the stack (simulating shared memory)
        let mut state = SharedState {
            sequence_number: 0,
            dirty_flag: 0,
            error_mode: 0,
            cursor_x: 0,
            cursor_y: 0,
            _padding2: [0; 2],
            cells: [scarab_protocol::Cell::default(); scarab_protocol::BUFFER_SIZE],
        };

        // Verify initial state
        assert_eq!(
            state.sequence_number, 0,
            "Sequence number should start at 0"
        );
        assert_eq!(state.dirty_flag, 0, "Dirty flag should start at 0");
        assert_eq!(state.error_mode, 0, "Error mode should start at 0 (normal)");
        assert_eq!(state.cursor_x, 0, "Cursor X should start at 0");
        assert_eq!(state.cursor_y, 0, "Cursor Y should start at 0");

        // Verify grid dimensions
        assert_eq!(
            state.cells.len(),
            GRID_WIDTH * GRID_HEIGHT,
            "Grid should have correct total size"
        );

        // Verify default cell values
        let first_cell = state.cells[0];
        assert_eq!(
            first_cell.char_codepoint, b' ' as u32,
            "Default cell should be space"
        );
        assert_eq!(
            first_cell.fg, 0xFFFFFFFF,
            "Default foreground should be white"
        );
        assert_eq!(
            first_cell.bg, 0xFF000000,
            "Default background should be black"
        );
        assert_eq!(first_cell.flags, 0, "Default flags should be 0");

        // Simulate daemon writing to shared memory
        state.sequence_number = 1;
        state.dirty_flag = 1;
        state.cursor_x = 10;
        state.cursor_y = 5;

        // Write a character
        let idx = state.cursor_y as usize * GRID_WIDTH + state.cursor_x as usize;
        state.cells[idx] = scarab_protocol::Cell {
            char_codepoint: 'H' as u32,
            fg: 0xFFFF0000, // Red
            bg: 0xFF000000, // Black
            flags: FLAG_BOLD,
            _padding: [0; 3],
        };

        // Verify updates
        assert_eq!(state.sequence_number, 1);
        assert_eq!(state.dirty_flag, 1);
        assert_eq!(state.cells[idx].char_codepoint, 'H' as u32);
        assert_eq!(state.cells[idx].fg, 0xFFFF0000);
        assert_eq!(state.cells[idx].flags, FLAG_BOLD);
    }

    /// Test VTE parser handles basic ANSI escape sequences
    #[test]
    fn test_vte_parser_basic_sequences() {
        let mut state = TerminalState::new(80, 24);

        // Test simple text output
        state.process_output(b"Hello, World!");
        assert_eq!(state.cursor_x, 13, "Cursor should advance after text");
        assert_eq!(state.cursor_y, 0, "Cursor should stay on first line");

        // Verify text was written
        assert_eq!(state.grid.get(0, 0).unwrap().char_codepoint, 'H' as u32);
        assert_eq!(state.grid.get(1, 0).unwrap().char_codepoint, 'e' as u32);
        assert_eq!(state.grid.get(2, 0).unwrap().char_codepoint, 'l' as u32);

        // Test newline
        state.process_output(b"\n");
        assert_eq!(state.cursor_x, 0, "Newline should reset cursor X to 0");
        assert_eq!(state.cursor_y, 1, "Newline should advance cursor Y");

        // Test cursor positioning (CSI H)
        state.process_output(b"\x1b[5;10H"); // Move to row 5, col 10
        assert_eq!(
            state.cursor_x, 9,
            "Cursor X should be at column 9 (0-indexed)"
        );
        assert_eq!(state.cursor_y, 4, "Cursor Y should be at row 4 (0-indexed)");

        // Test SGR (Select Graphic Rendition) - bold
        state.process_output(b"\x1b[1mBold");
        let bold_cell = state.grid.get(9, 4).unwrap();
        assert_eq!(bold_cell.char_codepoint, 'B' as u32);
        assert!(
            (bold_cell.flags & FLAG_BOLD) != 0,
            "Character should have bold flag"
        );

        // Test SGR - reset
        state.process_output(b"\x1b[0mNormal");
        let normal_cell = state.grid.get(13, 4).unwrap();
        assert_eq!(normal_cell.flags, 0, "Flags should be reset after SGR 0");
    }

    /// Test VTE parser handles color sequences
    #[test]
    fn test_vte_parser_colors() {
        let mut state = TerminalState::new(80, 24);

        // Test basic ANSI color (red foreground)
        state.process_output(b"\x1b[31mRed");
        let red_cell = state.grid.get(0, 0).unwrap();
        assert_eq!(red_cell.char_codepoint, 'R' as u32);
        assert_eq!(red_cell.fg, 0xFFCD0000, "Should have ANSI red foreground");

        // Test background color (blue background)
        state.process_output(b"\x1b[44mBlue");
        let blue_cell = state.grid.get(3, 0).unwrap();
        assert_eq!(blue_cell.bg, 0xFF0000EE, "Should have ANSI blue background");

        // Test reset to default colors
        state.process_output(b"\x1b[39;49mDefault");
        let default_cell = state.grid.get(7, 0).unwrap();
        assert_eq!(default_cell.fg, 0xFFCCCCCC, "Should reset to default FG");
        assert_eq!(default_cell.bg, 0xFF000000, "Should reset to default BG");
    }

    /// Test VTE parser scrolling behavior
    #[test]
    fn test_vte_parser_scrolling() {
        let mut state = TerminalState::new(80, 5); // Small 5-row terminal

        // Fill the terminal with 5 lines
        state.process_output(b"Line0");
        state.process_output(b"\nLine1");
        state.process_output(b"\nLine2");
        state.process_output(b"\nLine3");
        state.process_output(b"\nLine4");

        // Verify all 5 lines are present before scroll
        assert_eq!(state.grid.get(0, 0).unwrap().char_codepoint, 'L' as u32);
        assert_eq!(state.grid.get(4, 0).unwrap().char_codepoint, '0' as u32);
        assert_eq!(state.grid.get(4, 1).unwrap().char_codepoint, '1' as u32);
        assert_eq!(state.grid.get(4, 2).unwrap().char_codepoint, '2' as u32);
        assert_eq!(state.grid.get(4, 3).unwrap().char_codepoint, '3' as u32);
        assert_eq!(state.grid.get(4, 4).unwrap().char_codepoint, '4' as u32);

        // Cursor should be at end of last line
        assert_eq!(state.cursor_y, 4, "Cursor should be on last row");

        // Add a newline - this will trigger scrolling
        state.process_output(b"\n");

        // Cursor should remain on row 4 after scroll
        assert_eq!(
            state.cursor_y, 4,
            "Cursor should stay on last row after scroll"
        );

        // First line (Line0) should have scrolled off, Line1 should now be at top
        assert_eq!(
            state.grid.get(4, 0).unwrap().char_codepoint,
            '1' as u32,
            "Line0 should have scrolled off, Line1 at top"
        );

        // Verify remaining lines
        assert_eq!(state.grid.get(4, 1).unwrap().char_codepoint, '2' as u32);
        assert_eq!(state.grid.get(4, 2).unwrap().char_codepoint, '3' as u32);
        assert_eq!(state.grid.get(4, 3).unwrap().char_codepoint, '4' as u32);

        // Last line should be empty (just scrolled in)
        assert_eq!(
            state.grid.get(0, 4).unwrap().char_codepoint,
            0,
            "New bottom line should be empty"
        );
    }

    /// Test VTE parser cursor movement commands
    #[test]
    fn test_vte_parser_cursor_movement() {
        let mut state = TerminalState::new(80, 24);

        // Position cursor
        state.process_output(b"\x1b[10;20H"); // Row 10, Col 20
        assert_eq!(state.cursor_x, 19);
        assert_eq!(state.cursor_y, 9);

        // Cursor up
        state.process_output(b"\x1b[3A"); // Up 3 rows
        assert_eq!(state.cursor_y, 6, "Cursor should move up 3 rows");

        // Cursor down
        state.process_output(b"\x1b[2B"); // Down 2 rows
        assert_eq!(state.cursor_y, 8, "Cursor should move down 2 rows");

        // Cursor forward
        state.process_output(b"\x1b[5C"); // Forward 5 cols
        assert_eq!(state.cursor_x, 24, "Cursor should move forward 5 columns");

        // Cursor back
        state.process_output(b"\x1b[4D"); // Back 4 cols
        assert_eq!(state.cursor_x, 20, "Cursor should move back 4 columns");
    }

    /// Test VTE parser erase commands
    #[test]
    fn test_vte_parser_erase() {
        let mut state = TerminalState::new(80, 24);

        // Write some text
        state.process_output(b"Hello World");
        assert_eq!(state.grid.get(0, 0).unwrap().char_codepoint, 'H' as u32);

        // Move cursor back to start
        state.process_output(b"\x1b[1;1H");

        // Erase from cursor to end of line (CSI K)
        state.process_output(b"\x1b[K");

        // Characters should be cleared
        assert_eq!(
            state.grid.get(0, 0).unwrap().char_codepoint,
            0,
            "Line should be erased"
        );

        // Write more text
        state.process_output(b"Test\n");
        state.process_output(b"Line 2");

        // Clear entire screen (CSI 2J)
        state.process_output(b"\x1b[2J");
        assert_eq!(state.cursor_x, 0, "Cursor should be at origin after clear");
        assert_eq!(state.cursor_y, 0);
        assert_eq!(
            state.grid.get(0, 0).unwrap().char_codepoint,
            0,
            "Screen should be cleared"
        );
    }

    /// Test OSC 133 shell integration markers
    #[test]
    fn test_vte_parser_osc_133() {
        let mut state = TerminalState::new(80, 24);

        // Simulate shell prompt markers
        state.process_output(b"\x1b]133;A\x07"); // Prompt start
        assert_eq!(state.prompt_markers.len(), 1);
        assert!(matches!(
            state.prompt_markers[0].marker_type,
            scarab_daemon::vte::PromptMarkerType::PromptStart
        ));

        state.process_output(b"\x1b]133;B\x07"); // Command start
        assert_eq!(state.prompt_markers.len(), 2);

        state.process_output(b"\x1b]133;C\x07"); // Command executed
        assert_eq!(state.prompt_markers.len(), 3);

        state.process_output(b"\x1b]133;D;0\x07"); // Command finished with exit code 0
        assert_eq!(state.prompt_markers.len(), 4);
        if let scarab_daemon::vte::PromptMarkerType::CommandFinished { exit_code } =
            state.prompt_markers[3].marker_type
        {
            assert_eq!(exit_code, 0);
        } else {
            panic!("Expected CommandFinished marker");
        }
    }

    /// Test plugin manager can be created and initialized
    #[test]
    fn test_plugin_manager_initialization() {
        use parking_lot::Mutex;
        use scarab_daemon::ipc::ClientRegistry;
        use scarab_daemon::plugin_manager::PluginManager;
        use scarab_plugin_api::context::{PluginConfigData, PluginContext, PluginSharedState};
        use std::sync::Arc;

        // Create minimal plugin context for testing
        let config = PluginConfigData::default();
        let state = Arc::new(Mutex::new(PluginSharedState::new(80, 24)));
        let context = Arc::new(PluginContext::new(config, state, "test-plugin"));
        let client_registry = ClientRegistry::new();

        let manager = PluginManager::new(context, client_registry);

        // Verify initial state
        assert_eq!(manager.enabled_count(), 0, "Should start with no plugins");
        assert_eq!(manager.list_plugins().len(), 0);

        // Test timeout configuration
        let manager_with_timeout = manager.with_timeout(500);
        assert_eq!(
            manager_with_timeout.hook_timeout,
            Duration::from_millis(500)
        );
    }

    /// Test session creation and basic operations
    #[test]
    fn test_session_management() {
        use scarab_daemon::session::Session;

        // Create a new session
        let session =
            Session::new("test-session".to_string(), 80, 24).expect("Failed to create session");

        // Verify session properties
        assert_eq!(session.name, "test-session");
        assert!(!session.id.is_empty(), "Session should have an ID");

        // Session should have a default tab
        let tabs = session.list_tabs();
        assert_eq!(tabs.len(), 1, "New session should have 1 tab");
        // list_tabs returns (TabId, title, is_active, pane_count)
        assert_eq!(tabs[0].1, "Tab 1", "Default tab should be named 'Tab 1'");
        assert!(tabs[0].2, "First tab should be active");
        assert_eq!(tabs[0].3, 1, "Default tab should have 1 pane");

        // Test getting active pane
        let active_pane = session.get_active_pane();
        assert!(active_pane.is_some(), "Session should have an active pane");
    }

    /// Test profiling metrics collector
    #[test]
    fn test_profiling_metrics() {
        let collector = MetricsCollector::new();

        // Verify collector is enabled by default
        assert!(
            collector.is_enabled(),
            "Metrics should be enabled by default"
        );

        // Record some VTE parsing metrics
        collector.record_vte_parse(Duration::from_micros(100), 1024);
        collector.record_vte_parse(Duration::from_micros(150), 2048);

        // Record rendering metrics
        collector.record_render(Duration::from_millis(16), 10);

        // Record plugin metrics
        collector.record_plugin_load(Duration::from_millis(50));
        collector.record_plugin_output(Duration::from_micros(25));

        // Get performance report
        let report = collector.report();

        // Verify averages are calculated correctly
        assert!(
            report.avg_vte_parse_time_us > 0.0,
            "VTE parse time should be recorded"
        );
        assert!(
            report.avg_render_time_ms > 0.0,
            "Render time should be recorded"
        );
        assert!(
            report.avg_plugin_load_time_ms > 0.0,
            "Plugin load time should be recorded"
        );
        assert!(
            report.avg_plugin_output_time_us > 0.0,
            "Plugin output time should be recorded"
        );

        // Test disabling metrics
        collector.disable();
        assert!(!collector.is_enabled(), "Metrics should be disabled");

        // Recording should be no-op when disabled
        collector.record_vte_parse(Duration::from_millis(1000), 1024);
        let report_after_disable = collector.report();

        // Count should not have changed
        assert_eq!(
            report_after_disable.avg_vte_parse_time_us, report.avg_vte_parse_time_us,
            "Metrics should not update when disabled"
        );

        // Test reset
        collector.enable();
        collector.reset();
        let report_after_reset = collector.report();
        assert_eq!(
            report_after_reset.avg_vte_parse_time_us, 0.0,
            "Metrics should be reset"
        );
    }

    /// Test profiling macros work (when profiling feature is enabled)
    #[test]
    fn test_profiling_macros() {
        #[cfg(feature = "profiling")]
        {
            use scarab_daemon::profiling::{function, scope};

            // These should compile without errors
            function!();
            scope!("test_scope");
            scope!("test_scope_with_data", "data");
        }

        #[cfg(not(feature = "profiling"))]
        {
            use scarab_daemon::{function, scope};

            // No-op macros should also compile without errors
            function!();
            scope!("test_scope");
            scope!("test_scope_with_data", "data");
        }
    }

    /// Test terminal grid dimensions and resizing
    #[test]
    fn test_terminal_grid_operations() {
        use scarab_daemon::vte::Grid;

        let mut grid = Grid::new(80, 24);
        assert_eq!(grid.cols, 80);
        assert_eq!(grid.rows, 24);
        assert_eq!(grid.cells.len(), 80 * 24);

        // Test writing to grid
        if let Some(cell) = grid.get_mut(10, 5) {
            cell.char_codepoint = 'X' as u32;
            cell.fg = 0xFFFF0000; // Red
        }

        // Verify read
        assert_eq!(grid.get(10, 5).unwrap().char_codepoint, 'X' as u32);
        assert_eq!(grid.get(10, 5).unwrap().fg, 0xFFFF0000);

        // Test resize
        grid.resize(100, 30);
        assert_eq!(grid.cols, 100);
        assert_eq!(grid.rows, 30);
        assert_eq!(grid.cells.len(), 100 * 30);

        // Content should be preserved where possible
        assert_eq!(
            grid.get(10, 5).unwrap().char_codepoint,
            'X' as u32,
            "Content should be preserved after resize"
        );

        // Test clear
        grid.clear();
        assert_eq!(
            grid.get(10, 5).unwrap().char_codepoint,
            0,
            "Grid should be cleared"
        );
    }

    /// Test blit operation from local grid to shared memory
    #[test]
    fn test_terminal_blit_to_shm() {
        let mut state = TerminalState::new(80, 24);

        // Write some content
        state.process_output(b"Test Content\x1b[1;1H");

        // Create a SharedState to blit to
        let mut shared_state = SharedState {
            sequence_number: 0,
            dirty_flag: 0,
            error_mode: 0,
            cursor_x: 0,
            cursor_y: 0,
            _padding2: [0; 2],
            cells: [scarab_protocol::Cell::default(); scarab_protocol::BUFFER_SIZE],
        };

        let sequence_counter = Arc::new(AtomicU64::new(0));

        // Blit to shared memory
        // SAFETY: shared_state is a valid SharedState on the stack
        unsafe { state.blit_to_shm(&mut shared_state as *mut SharedState, &sequence_counter) };

        // Verify shared memory was updated
        assert_eq!(shared_state.sequence_number, 1, "Sequence should increment");
        assert_eq!(shared_state.dirty_flag, 1, "Dirty flag should be set");
        assert_eq!(shared_state.cursor_x, 0);
        assert_eq!(shared_state.cursor_y, 0);

        // Verify content was copied
        assert_eq!(shared_state.cells[0].char_codepoint, 'T' as u32);
        assert_eq!(shared_state.cells[1].char_codepoint, 'e' as u32);
    }

    /// Test terminal state handles text attributes correctly
    #[test]
    fn test_terminal_text_attributes() {
        let mut state = TerminalState::new(80, 24);

        // Test bold
        state.process_output(b"\x1b[1mBold\x1b[0m");
        let bold_cell = state.grid.get(0, 0).unwrap();
        assert!((bold_cell.flags & FLAG_BOLD) != 0);

        // Test underline
        state.process_output(b"\x1b[4mUnderline\x1b[0m");
        let underline_cell = state.grid.get(4, 0).unwrap();
        assert!((underline_cell.flags & FLAG_UNDERLINE) != 0);

        // Test combined attributes
        state.process_output(b"\x1b[1;4mBoldUnderline\x1b[0m");
        let combined_cell = state.grid.get(13, 0).unwrap();
        assert!((combined_cell.flags & FLAG_BOLD) != 0);
        assert!((combined_cell.flags & FLAG_UNDERLINE) != 0);
    }
}
