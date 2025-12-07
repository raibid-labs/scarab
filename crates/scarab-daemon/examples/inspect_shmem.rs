// Quick diagnostic tool to inspect shared memory contents
use scarab_protocol::{SharedState, GRID_HEIGHT, GRID_WIDTH, SHMEM_PATH};
use shared_memory::ShmemConf;

fn main() {
    let shmem = match ShmemConf::new().os_id(SHMEM_PATH).open() {
        Ok(m) => m,
        Err(e) => {
            eprintln!("Failed to open shared memory: {}", e);
            eprintln!("Is the daemon running?");
            std::process::exit(1);
        }
    };

    let shared_ptr = shmem.as_ptr() as *const SharedState;

    unsafe {
        let state = &*shared_ptr;

        println!("=== Shared Memory State ===");
        println!("Sequence number: {}", state.sequence_number);
        println!("Cursor position: ({}, {})", state.cursor_x, state.cursor_y);
        println!("Grid size: {}x{}", GRID_WIDTH, GRID_HEIGHT);
        println!("\n=== First 10 rows ===");

        for row in 0..10.min(GRID_HEIGHT) {
            print!("Row {:2}: ", row);
            for col in 0..GRID_WIDTH {
                let idx = row * GRID_WIDTH + col;
                let cell = &state.cells[idx];

                if cell.char_codepoint == 0 || cell.char_codepoint == 32 {
                    print!(" ");
                } else if let Some(ch) = char::from_u32(cell.char_codepoint) {
                    print!("{}", ch);
                } else {
                    print!("?");
                }
            }
            println!();
        }

        // Count non-empty cells
        let non_empty = state
            .cells
            .iter()
            .filter(|c| c.char_codepoint != 0 && c.char_codepoint != 32)
            .count();
        println!(
            "\nNon-empty cells: {}/{}",
            non_empty,
            GRID_WIDTH * GRID_HEIGHT
        );
    }
}
