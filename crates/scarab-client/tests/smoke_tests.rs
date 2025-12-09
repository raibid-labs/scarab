//! Smoke tests for scarab-client
//!
//! These tests exercise unit-testable functionality without requiring a running client:
//! - Shared memory reading via SafeSharedState and MockTerminalState
//! - Terminal display calculations (TerminalMetrics)
//! - Input handling (key conversion)
//! - Theming (ColorConfig)
//! - Resize calculations (STATUS_BAR_HEIGHT)

#[cfg(test)]
mod smoke_tests {
    use scarab_client::MockTerminalState;
    use scarab_protocol::terminal_state::TerminalStateReader;
    use scarab_protocol::{Cell, TerminalMetrics, GRID_HEIGHT, GRID_WIDTH};
    use scarab_config::{ColorConfig, ColorPalette};
    use scarab_client::input::key_tables::bevy_to_api_keycode;
    use bevy::input::keyboard::KeyCode as BevyKeyCode;
    use scarab_plugin_api::key_tables::KeyCode as ApiKeyCode;
    use scarab_client::ui::STATUS_BAR_HEIGHT;

    // ========================================
    // 1. Shared Memory Reading Tests
    // ========================================

    #[test]
    fn test_mock_terminal_state_creation() {
        let state = MockTerminalState::new(GRID_WIDTH, GRID_HEIGHT);

        assert_eq!(state.dimensions(), (GRID_WIDTH, GRID_HEIGHT));
        assert_eq!(state.sequence(), 0);
        assert_eq!(state.cursor_pos(), (0, 0));
        assert!(state.is_valid());
        assert!(!state.is_dirty());
        assert!(!state.is_error_mode());
    }

    #[test]
    fn test_mock_terminal_state_cell_access() {
        let mut state = MockTerminalState::new(80, 24);

        // Default cells should be accessible
        assert!(state.cell(0, 0).is_some());
        assert!(state.cell(23, 79).is_some());

        // Out of bounds should return None
        assert!(state.cell(24, 0).is_none());
        assert!(state.cell(0, 80).is_none());
        assert!(state.cell(100, 100).is_none());

        // Modify a cell
        let mut test_cell = Cell::default();
        test_cell.char_codepoint = 'H' as u32;
        test_cell.fg = 0xFF00FF00; // Green
        test_cell.bg = 0xFF000000; // Black

        assert!(state.set_cell(5, 10, test_cell));

        // Verify the cell was set
        let cell = state.cell(5, 10).unwrap();
        assert_eq!(cell.char_codepoint, 'H' as u32);
        assert_eq!(cell.fg, 0xFF00FF00);
        assert_eq!(cell.bg, 0xFF000000);

        // State should now be dirty
        assert!(state.is_dirty());
    }

    #[test]
    fn test_mock_terminal_state_cursor_operations() {
        let mut state = MockTerminalState::new(80, 24);

        // Set cursor to various positions
        state.set_cursor(10, 5);
        assert_eq!(state.cursor_pos(), (10, 5));

        state.set_cursor(79, 23);
        assert_eq!(state.cursor_pos(), (79, 23));

        state.set_cursor(0, 0);
        assert_eq!(state.cursor_pos(), (0, 0));
    }

    #[test]
    fn test_mock_terminal_state_text_writing() {
        let mut state = MockTerminalState::new(80, 24);

        state.set_cursor(0, 0);
        state.write_text("Hello, Scarab!");

        // Verify text was written
        assert_eq!(state.cell(0, 0).unwrap().char_codepoint, 'H' as u32);
        assert_eq!(state.cell(0, 1).unwrap().char_codepoint, 'e' as u32);
        assert_eq!(state.cell(0, 2).unwrap().char_codepoint, 'l' as u32);
        assert_eq!(state.cell(0, 3).unwrap().char_codepoint, 'l' as u32);
        assert_eq!(state.cell(0, 4).unwrap().char_codepoint, 'o' as u32);
        assert_eq!(state.cell(0, 5).unwrap().char_codepoint, ',' as u32);
        assert_eq!(state.cell(0, 6).unwrap().char_codepoint, ' ' as u32);
        assert_eq!(state.cell(0, 7).unwrap().char_codepoint, 'S' as u32);

        // Cursor should have moved
        assert!(state.cursor_pos().0 > 0);
    }

    #[test]
    fn test_mock_terminal_state_fill() {
        let mut state = MockTerminalState::new(10, 10);
        state.fill('X');

        // Verify all cells are filled
        for row in 0..10 {
            for col in 0..10 {
                assert_eq!(state.cell(row, col).unwrap().char_codepoint, 'X' as u32);
            }
        }

        assert!(state.is_dirty());
    }

    #[test]
    fn test_mock_terminal_state_sequence_increment() {
        let mut state = MockTerminalState::new(80, 24);

        assert_eq!(state.sequence(), 0);

        state.increment_sequence();
        assert_eq!(state.sequence(), 1);

        state.increment_sequence();
        assert_eq!(state.sequence(), 2);
    }

    #[test]
    fn test_mock_terminal_state_validation() {
        let valid_state = MockTerminalState::new(80, 24);
        assert!(valid_state.is_valid());

        // Create invalid state by manipulating cursor
        let mut invalid_state = MockTerminalState::new(10, 10);
        invalid_state.set_cursor(100, 5); // Out of bounds
        assert!(!invalid_state.is_valid());
    }

    // ========================================
    // 2. Terminal Display Calculations Tests
    // ========================================

    #[test]
    fn test_terminal_metrics_default() {
        let metrics = TerminalMetrics::default();

        assert_eq!(metrics.cell_width, 9.0);
        assert_eq!(metrics.cell_height, 18.0);
        assert_eq!(metrics.columns, 80);
        assert_eq!(metrics.rows, 24);
    }

    #[test]
    fn test_terminal_metrics_custom() {
        let font_size = 15.0;
        let line_height = 1.2;
        let metrics = TerminalMetrics::new(font_size, line_height, 100, 50);

        assert_eq!(metrics.cell_width, 9.0); // 15.0 * 0.6
        assert_eq!(metrics.cell_height, 18.0); // 15.0 * 1.2
        assert_eq!(metrics.columns, 100);
        assert_eq!(metrics.rows, 50);
    }

    #[test]
    fn test_terminal_metrics_screen_to_grid() {
        let metrics = TerminalMetrics {
            cell_width: 10.0,
            cell_height: 20.0,
            columns: 80,
            rows: 24,
        };

        // Test origin
        assert_eq!(metrics.screen_to_grid(0.0, 0.0), (0, 0));

        // Test first cell boundaries
        assert_eq!(metrics.screen_to_grid(5.0, 10.0), (0, 0));
        assert_eq!(metrics.screen_to_grid(9.9, 19.9), (0, 0));

        // Test second cell
        assert_eq!(metrics.screen_to_grid(10.0, 0.0), (1, 0));
        assert_eq!(metrics.screen_to_grid(0.0, 20.0), (0, 1));

        // Test middle of grid
        assert_eq!(metrics.screen_to_grid(50.0, 40.0), (5, 2));

        // Test clamping at boundaries
        assert_eq!(metrics.screen_to_grid(1000.0, 0.0), (79, 0)); // Clamps to max col
        assert_eq!(metrics.screen_to_grid(0.0, 1000.0), (0, 23)); // Clamps to max row
        assert_eq!(metrics.screen_to_grid(-10.0, -10.0), (0, 0)); // Clamps to min
    }

    #[test]
    fn test_terminal_metrics_grid_to_screen() {
        let metrics = TerminalMetrics {
            cell_width: 10.0,
            cell_height: 20.0,
            columns: 80,
            rows: 24,
        };

        // Test origin
        assert_eq!(metrics.grid_to_screen(0, 0), (0.0, 0.0));

        // Test various positions
        assert_eq!(metrics.grid_to_screen(1, 0), (10.0, 0.0));
        assert_eq!(metrics.grid_to_screen(0, 1), (0.0, 20.0));
        assert_eq!(metrics.grid_to_screen(5, 2), (50.0, 40.0));

        // Test far corner
        assert_eq!(metrics.grid_to_screen(79, 23), (790.0, 460.0));
    }

    #[test]
    fn test_terminal_metrics_screen_size() {
        let metrics = TerminalMetrics {
            cell_width: 10.0,
            cell_height: 20.0,
            columns: 80,
            rows: 24,
        };

        let (width, height) = metrics.screen_size();
        assert_eq!(width, 800.0); // 80 * 10.0
        assert_eq!(height, 480.0); // 24 * 20.0
    }

    #[test]
    fn test_terminal_metrics_round_trip() {
        let metrics = TerminalMetrics {
            cell_width: 10.0,
            cell_height: 20.0,
            columns: 80,
            rows: 24,
        };

        // Test that grid -> screen -> grid is consistent
        let (screen_x, screen_y) = metrics.grid_to_screen(10, 5);
        let (col, row) = metrics.screen_to_grid(screen_x, screen_y);
        assert_eq!((col, row), (10, 5));

        // Test multiple positions
        for test_col in [0, 1, 5, 10, 79] {
            for test_row in [0, 1, 5, 10, 23] {
                let (sx, sy) = metrics.grid_to_screen(test_col, test_row);
                let (c, r) = metrics.screen_to_grid(sx, sy);
                assert_eq!((c, r), (test_col, test_row));
            }
        }
    }

    // ========================================
    // 3. Input Handling Tests
    // ========================================

    #[test]
    fn test_keycode_conversion_letters() {
        assert_eq!(bevy_to_api_keycode(BevyKeyCode::KeyA), Some(ApiKeyCode::KeyA));
        assert_eq!(bevy_to_api_keycode(BevyKeyCode::KeyB), Some(ApiKeyCode::KeyB));
        assert_eq!(bevy_to_api_keycode(BevyKeyCode::KeyZ), Some(ApiKeyCode::KeyZ));
        assert_eq!(bevy_to_api_keycode(BevyKeyCode::KeyM), Some(ApiKeyCode::KeyM));
    }

    #[test]
    fn test_keycode_conversion_numbers() {
        assert_eq!(bevy_to_api_keycode(BevyKeyCode::Digit0), Some(ApiKeyCode::Digit0));
        assert_eq!(bevy_to_api_keycode(BevyKeyCode::Digit1), Some(ApiKeyCode::Digit1));
        assert_eq!(bevy_to_api_keycode(BevyKeyCode::Digit5), Some(ApiKeyCode::Digit5));
        assert_eq!(bevy_to_api_keycode(BevyKeyCode::Digit9), Some(ApiKeyCode::Digit9));
    }

    #[test]
    fn test_keycode_conversion_function_keys() {
        assert_eq!(bevy_to_api_keycode(BevyKeyCode::F1), Some(ApiKeyCode::F1));
        assert_eq!(bevy_to_api_keycode(BevyKeyCode::F5), Some(ApiKeyCode::F5));
        assert_eq!(bevy_to_api_keycode(BevyKeyCode::F12), Some(ApiKeyCode::F12));
    }

    #[test]
    fn test_keycode_conversion_special_keys() {
        assert_eq!(bevy_to_api_keycode(BevyKeyCode::Escape), Some(ApiKeyCode::Escape));
        assert_eq!(bevy_to_api_keycode(BevyKeyCode::Enter), Some(ApiKeyCode::Enter));
        assert_eq!(bevy_to_api_keycode(BevyKeyCode::Tab), Some(ApiKeyCode::Tab));
        assert_eq!(bevy_to_api_keycode(BevyKeyCode::Backspace), Some(ApiKeyCode::Backspace));
        assert_eq!(bevy_to_api_keycode(BevyKeyCode::Space), Some(ApiKeyCode::Space));
    }

    #[test]
    fn test_keycode_conversion_arrow_keys() {
        assert_eq!(bevy_to_api_keycode(BevyKeyCode::ArrowLeft), Some(ApiKeyCode::Left));
        assert_eq!(bevy_to_api_keycode(BevyKeyCode::ArrowRight), Some(ApiKeyCode::Right));
        assert_eq!(bevy_to_api_keycode(BevyKeyCode::ArrowUp), Some(ApiKeyCode::Up));
        assert_eq!(bevy_to_api_keycode(BevyKeyCode::ArrowDown), Some(ApiKeyCode::Down));
    }

    #[test]
    fn test_keycode_conversion_navigation_keys() {
        assert_eq!(bevy_to_api_keycode(BevyKeyCode::Home), Some(ApiKeyCode::Home));
        assert_eq!(bevy_to_api_keycode(BevyKeyCode::End), Some(ApiKeyCode::End));
        assert_eq!(bevy_to_api_keycode(BevyKeyCode::PageUp), Some(ApiKeyCode::PageUp));
        assert_eq!(bevy_to_api_keycode(BevyKeyCode::PageDown), Some(ApiKeyCode::PageDown));
        assert_eq!(bevy_to_api_keycode(BevyKeyCode::Insert), Some(ApiKeyCode::Insert));
        assert_eq!(bevy_to_api_keycode(BevyKeyCode::Delete), Some(ApiKeyCode::Delete));
    }

    #[test]
    fn test_keycode_conversion_modifiers() {
        assert_eq!(bevy_to_api_keycode(BevyKeyCode::ControlLeft), Some(ApiKeyCode::ControlLeft));
        assert_eq!(bevy_to_api_keycode(BevyKeyCode::ControlRight), Some(ApiKeyCode::ControlRight));
        assert_eq!(bevy_to_api_keycode(BevyKeyCode::AltLeft), Some(ApiKeyCode::AltLeft));
        assert_eq!(bevy_to_api_keycode(BevyKeyCode::AltRight), Some(ApiKeyCode::AltRight));
        assert_eq!(bevy_to_api_keycode(BevyKeyCode::ShiftLeft), Some(ApiKeyCode::ShiftLeft));
        assert_eq!(bevy_to_api_keycode(BevyKeyCode::ShiftRight), Some(ApiKeyCode::ShiftRight));
        assert_eq!(bevy_to_api_keycode(BevyKeyCode::SuperLeft), Some(ApiKeyCode::SuperLeft));
        assert_eq!(bevy_to_api_keycode(BevyKeyCode::SuperRight), Some(ApiKeyCode::SuperRight));
    }

    // ========================================
    // 4. Theming Tests
    // ========================================

    #[test]
    fn test_color_config_default() {
        let config = ColorConfig::default();

        // Verify default theme is set
        assert!(config.theme.is_some());
        assert_eq!(config.theme.as_deref(), Some("slime"));

        // Verify palette has required colors (all are Strings, not Option<String>)
        assert!(!config.palette.black.is_empty());
        assert!(!config.palette.white.is_empty());
        assert!(!config.palette.red.is_empty());
        assert!(!config.palette.green.is_empty());
        assert!(!config.palette.blue.is_empty());
        assert!(!config.palette.yellow.is_empty());

        // Verify foreground/background are set
        assert!(config.foreground.is_some());
        assert!(config.background.is_some());
    }

    #[test]
    fn test_color_palette_creation() {
        let palette = ColorPalette {
            black: "#000000".to_string(),
            red: "#FF0000".to_string(),
            green: "#00FF00".to_string(),
            yellow: "#FFFF00".to_string(),
            blue: "#0000FF".to_string(),
            magenta: "#FF00FF".to_string(),
            cyan: "#00FFFF".to_string(),
            white: "#FFFFFF".to_string(),
            bright_black: "#808080".to_string(),
            bright_red: "#FF8080".to_string(),
            bright_green: "#80FF80".to_string(),
            bright_yellow: "#FFFF80".to_string(),
            bright_blue: "#8080FF".to_string(),
            bright_magenta: "#FF80FF".to_string(),
            bright_cyan: "#80FFFF".to_string(),
            bright_white: "#FFFFFF".to_string(),
        };

        assert_eq!(palette.black.as_str(), "#000000");
        assert_eq!(palette.red.as_str(), "#FF0000");
        assert_eq!(palette.green.as_str(), "#00FF00");
        assert_eq!(palette.blue.as_str(), "#0000FF");
        assert_eq!(palette.white.as_str(), "#FFFFFF");
    }

    #[test]
    fn test_color_config_custom_theme() {
        let mut config = ColorConfig::default();
        config.theme = Some("custom-theme".to_string());

        assert_eq!(config.theme.as_deref(), Some("custom-theme"));
    }

    // ========================================
    // 5. Resize Calculations Tests
    // ========================================

    #[test]
    fn test_status_bar_height_constant() {
        // Verify the status bar height constant is reasonable
        assert!(STATUS_BAR_HEIGHT > 0.0);
        assert!(STATUS_BAR_HEIGHT < 100.0); // Should be less than 100 pixels

        // Document the expected value
        assert_eq!(STATUS_BAR_HEIGHT, 24.0);
    }

    #[test]
    fn test_resize_with_status_bar() {
        // Simulate window resize calculation accounting for status bar
        let window_width: f32 = 800.0;
        let window_height: f32 = 600.0;
        let cell_width: f32 = 10.0;
        let cell_height: f32 = 20.0;

        // Calculate available height after status bar
        let available_height = window_height - STATUS_BAR_HEIGHT;

        // Calculate terminal dimensions
        let cols = (window_width / cell_width).floor() as u16;
        let rows = (available_height / cell_height).floor() as u16;

        assert_eq!(cols, 80); // 800 / 10
        assert_eq!(rows, 28); // (600 - 24) / 20 = 576 / 20 = 28

        // Verify the terminal grid doesn't overlap status bar
        let terminal_height = rows as f32 * cell_height;
        assert!(terminal_height + STATUS_BAR_HEIGHT <= window_height);
    }

    #[test]
    fn test_resize_minimum_dimensions_with_status_bar() {
        // Test that status bar is accounted for even in small windows
        let window_width: f32 = 400.0;
        let window_height: f32 = 200.0;
        let cell_width: f32 = 10.0;
        let cell_height: f32 = 20.0;

        let available_height = window_height - STATUS_BAR_HEIGHT;

        let cols = ((window_width / cell_width).floor() as u16).max(10);
        let rows = ((available_height / cell_height).floor() as u16).max(5);

        // Should enforce minimum dimensions
        assert!(cols >= 10);
        assert!(rows >= 5);

        // Even with minimums, should not exceed window
        let terminal_height = rows as f32 * cell_height;
        assert!(terminal_height + STATUS_BAR_HEIGHT <= window_height + 1.0); // +1 for floating point
    }

    #[test]
    fn test_resize_grid_bounds() {
        // Test that resize calculations respect GRID_WIDTH and GRID_HEIGHT limits
        let cell_width: f32 = 10.0;
        let cell_height: f32 = 20.0;

        // Simulate a very large window
        let large_window_width: f32 = 5000.0;
        let large_window_height: f32 = 5000.0;

        let available_height = large_window_height - STATUS_BAR_HEIGHT;

        let cols = ((large_window_width / cell_width).floor() as u16)
            .min(GRID_WIDTH as u16)
            .max(80);
        let rows = ((available_height / cell_height).floor() as u16)
            .min(GRID_HEIGHT as u16)
            .max(24);

        // Should not exceed protocol limits
        assert!(cols <= GRID_WIDTH as u16);
        assert!(rows <= GRID_HEIGHT as u16);

        // For this specific case
        assert_eq!(cols, GRID_WIDTH as u16); // 200
        assert_eq!(rows, GRID_HEIGHT as u16); // 100
    }

    #[test]
    fn test_cell_dimensions_from_font_size() {
        // Test that cell dimensions are calculated correctly from font size
        let font_size = 15.0;
        let line_height_multiplier = 1.2;

        let cell_width = font_size * 0.6; // Standard monospace ratio
        let cell_height = font_size * line_height_multiplier;

        assert_eq!(cell_width, 9.0);
        assert_eq!(cell_height, 18.0);

        // Verify metrics match
        let metrics = TerminalMetrics::new(font_size, line_height_multiplier, 80, 24);
        assert_eq!(metrics.cell_width, cell_width);
        assert_eq!(metrics.cell_height, cell_height);
    }
}
