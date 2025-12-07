use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput};
use scarab_client::safe_state::MockTerminalState;
use scarab_protocol::{Cell, TerminalStateReader};
use std::time::Duration;

fn bench_state_iteration(c: &mut Criterion) {
    let mut group = c.benchmark_group("state_iteration");
    group.measurement_time(Duration::from_secs(5));

    for (cols, rows) in [(80, 24), (150, 40), (200, 100)] {
        let cell_count = cols * rows;
        group.throughput(Throughput::Elements(cell_count as u64));

        // Setup mock state
        let mut state = MockTerminalState::new(cols, rows);

        // Fill with data
        {
            let cells = state.cells_mut();
            for i in 0..cells.len() {
                cells[i].char_codepoint = 'a' as u32 + (i % 26) as u32;
                cells[i].fg = 0xFFFFFFFF;
                cells[i].bg = 0x000000FF;
            }
        }

        group.bench_function(format!("iter_{}x{}", cols, rows), |b| {
            b.iter(|| {
                // Simulate the access pattern of the renderer
                let dimensions = state.dimensions();
                let buffer = state.cells();

                let mut checksum = 0u64;
                for cell in buffer {
                    // Simulate some "work" per cell (e.g. checking dirty flags)
                    if cell.char_codepoint != 0 {
                        checksum = checksum.wrapping_add(cell.char_codepoint as u64);
                    }
                }
                black_box(checksum);
                black_box(dimensions);
            });
        });
    }
    group.finish();
}

criterion_group!(benches, bench_state_iteration);
criterion_main!(benches);
