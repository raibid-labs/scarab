use std::sync::atomic::AtomicU64;
use std::sync::Arc;
use ratatui_testlib::ScreenState; // ratatui-testlib types
use scarab_daemon::vte::TerminalState;
use scarab_protocol::{SharedState, GRID_WIDTH}; // Scarab types

#[test]
fn test_vte_basic_text_rendering() {
    // 1. Setup Scarab VTE
    // We need a mock SharedState backed by real memory
    let mut shared_data = [0u8; std::mem::size_of::<SharedState>()];
    let shared_ptr = shared_data.as_mut_ptr() as *mut SharedState;
    unsafe {
        // Initialize with defaults
        std::ptr::write_bytes(shared_ptr, 0, 1);
    }
    let seq = Arc::new(AtomicU64::new(0));
    let mut scarab_vte = TerminalState::new(shared_ptr, seq);

    // 2. Setup ratatui-testlib Reference
    // ratatui-testlib can process raw byte streams directly via feed()
    let mut testlib_screen = ScreenState::new(80, 24);

    // 3. Test input with ANSI color codes
    let input = "Hello \x1b[31mRed\x1b[0m World";

    // Process input in both Scarab and ratatui-testlib
    scarab_vte.process_output(input.as_bytes());
    testlib_screen.feed(input.as_bytes());

    // 4. Verify Scarab State matches ratatui-testlib
    unsafe {
        let state = &*shared_ptr;

        // Helper to get cell from flat array
        let get_cell = |row: usize, col: usize| -> &scarab_protocol::Cell {
            &state.cells[row * GRID_WIDTH + col]
        };

        // Check "Hello " (positions 0-5)
        for i in 0..5 {
            let scarab_cell = get_cell(0, i);
            let testlib_cell = testlib_screen.get_cell(0, i as u16)
                .expect("ratatui-testlib cell should exist");

            assert_eq!(
                char::from_u32(scarab_cell.char_codepoint).unwrap(),
                testlib_cell.c,
                "Character mismatch at position {}", i
            );

            // Both should have default colors for "Hello "
            assert_eq!(testlib_cell.fg, None, "Hello should have default foreground");
        }

        // Check "Red" (positions 6-8) - should be red (color 1)
        for i in 6..9 {
            let scarab_cell = get_cell(0, i);
            let testlib_cell = testlib_screen.get_cell(0, i as u16)
                .expect("ratatui-testlib cell should exist");

            assert_eq!(
                char::from_u32(scarab_cell.char_codepoint).unwrap(),
                testlib_cell.c,
                "Character mismatch at position {}", i
            );

            // ratatui-testlib should show red foreground (color 1 for ANSI red)
            assert_eq!(
                testlib_cell.fg,
                Some(1),
                "Red text should have foreground color 1 at position {}",
                i
            );

            // Verify Scarab also parsed the color correctly
            // (Scarab uses RGBA format, check that it's not white/default)
            assert_ne!(
                scarab_cell.fg,
                0xFFFFFFFF, // Default white
                "Scarab should have non-default foreground for red text at position {}",
                i
            );
        }

        // Check " World" (positions 10-14) - should be back to default
        let cell_w = get_cell(0, 10);
        let testlib_w = testlib_screen.get_cell(0, 10)
            .expect("ratatui-testlib cell should exist");

        assert_eq!(char::from_u32(cell_w.char_codepoint).unwrap(), 'W');
        assert_eq!(testlib_w.c, 'W');
        assert_eq!(testlib_w.fg, None, "World should have default foreground after reset");
    }
}

#[test]
fn test_vte_color_codes() {
    // Test various ANSI color codes
    let mut testlib_screen = ScreenState::new(80, 24);

    // Test basic colors
    testlib_screen.feed(b"\x1b[31mRed\x1b[0m");
    testlib_screen.feed(b"\x1b[32mGreen\x1b[0m");
    testlib_screen.feed(b"\x1b[33mYellow\x1b[0m");

    // Verify red
    let cell = testlib_screen.get_cell(0, 0).expect("Cell should exist");
    assert_eq!(cell.c, 'R');
    assert_eq!(cell.fg, Some(1), "Red should be color 1");

    // Verify green
    let cell = testlib_screen.get_cell(0, 3).expect("Cell should exist");
    assert_eq!(cell.c, 'G');
    assert_eq!(cell.fg, Some(2), "Green should be color 2");

    // Verify yellow
    let cell = testlib_screen.get_cell(0, 8).expect("Cell should exist");
    assert_eq!(cell.c, 'Y');
    assert_eq!(cell.fg, Some(3), "Yellow should be color 3");
}

#[test]
fn test_vte_text_attributes() {
    // Test bold, italic, underline
    let mut testlib_screen = ScreenState::new(80, 24);

    testlib_screen.feed(b"\x1b[1mBold\x1b[0m");
    testlib_screen.feed(b"\x1b[3mItalic\x1b[0m");
    testlib_screen.feed(b"\x1b[4mUnderline\x1b[0m");

    // Verify bold
    let cell = testlib_screen.get_cell(0, 0).expect("Cell should exist");
    assert_eq!(cell.c, 'B');
    assert!(cell.bold, "Text should be bold");
    assert!(!cell.italic && !cell.underline);

    // Verify italic
    let cell = testlib_screen.get_cell(0, 4).expect("Cell should exist");
    assert_eq!(cell.c, 'I');
    assert!(cell.italic, "Text should be italic");
    assert!(!cell.bold && !cell.underline);

    // Verify underline
    let cell = testlib_screen.get_cell(0, 10).expect("Cell should exist");
    assert_eq!(cell.c, 'U');
    assert!(cell.underline, "Text should be underlined");
    assert!(!cell.bold && !cell.italic);
}
