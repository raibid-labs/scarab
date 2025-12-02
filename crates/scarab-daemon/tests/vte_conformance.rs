use std::sync::atomic::AtomicU64;
use std::sync::Arc;
use scarab_daemon::vte::TerminalState;
use scarab_protocol::{SharedState, GRID_WIDTH, GRID_HEIGHT};

#[test]
fn test_vte_basic_text_rendering() {
    // 1. Setup Scarab VTE with local grid
    let mut scarab_vte = TerminalState::new(GRID_WIDTH as u16, GRID_HEIGHT as u16);

    // Setup shared memory for blitting
    let mut shared_data = [0u8; std::mem::size_of::<SharedState>()];
    let shared_ptr = shared_data.as_mut_ptr() as *mut SharedState;
    unsafe {
        std::ptr::write_bytes(shared_ptr, 0, 1);
    }
    let seq = Arc::new(AtomicU64::new(0));

    let input = "Hello \x1b[31mRed\x1b[0m World";

    // Process input in Scarab (writes to local grid)
    scarab_vte.process_output(input.as_bytes());

    // Blit local grid to shared memory
    scarab_vte.blit_to_shm(shared_ptr, &seq);

    // 3. Verify Scarab State via shared memory
    // cells is a flat array: cells[row * GRID_WIDTH + col]
    unsafe {
        let state = &*shared_ptr;

        // Check "Hello " - row 0, col 0
        let cell_h = state.cells[0 * GRID_WIDTH + 0];
        assert_eq!(char::from_u32(cell_h.char_codepoint).unwrap(), 'H');

        // Check "Red" (should be red) - row 0, col 6
        let cell_r = state.cells[0 * GRID_WIDTH + 6];
        assert_eq!(char::from_u32(cell_r.char_codepoint).unwrap(), 'R');
        assert_ne!(cell_r.fg, 0, "Red text should have non-zero foreground color");

        // Check " World" (should be default color) - row 0, col 10
        let cell_w = state.cells[0 * GRID_WIDTH + 10];
        assert_eq!(char::from_u32(cell_w.char_codepoint).unwrap(), 'W');
        // Note: Default FG is 0xFFCCCCCC (light gray), not 0
        // After SGR reset, the FG should be back to default
    }
}
