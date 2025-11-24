//! VTE parsing performance benchmarks

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use vte::{Parser, Perform};

/// Test performer that processes VTE sequences
struct BenchPerformer {
    operations: usize,
}

impl BenchPerformer {
    fn new() -> Self {
        Self { operations: 0 }
    }

    fn reset(&mut self) {
        self.operations = 0;
    }
}

impl Perform for BenchPerformer {
    fn print(&mut self, _c: char) {
        self.operations += 1;
    }

    fn execute(&mut self, _byte: u8) {
        self.operations += 1;
    }

    fn csi_dispatch(&mut self, _params: &[i64], _intermediates: &[u8], _ignore: bool, _c: u8) {
        self.operations += 1;
    }

    fn osc_dispatch(&mut self, _params: &[&[u8]], _bell_terminated: bool) {
        self.operations += 1;
    }

    fn hook(&mut self, _params: &[i64], _intermediates: &[u8], _ignore: bool, _c: u8) {
        self.operations += 1;
    }

    fn put(&mut self, _byte: u8) {
        self.operations += 1;
    }

    fn unhook(&mut self) {
        self.operations += 1;
    }
}

fn generate_text_data(size: usize) -> Vec<u8> {
    // Generate plain text data
    let text = "The quick brown fox jumps over the lazy dog. ";
    text.bytes().cycle().take(size).collect()
}

fn generate_ansi_colors(count: usize) -> Vec<u8> {
    // Generate ANSI color escape sequences
    let mut data = Vec::new();
    for i in 0..count {
        let color = 30 + (i % 8);
        data.extend_from_slice(format!("\x1b[{}mColor{}\x1b[0m ", color, i).as_bytes());
    }
    data
}

fn generate_cursor_movements(count: usize) -> Vec<u8> {
    // Generate cursor movement sequences
    let mut data = Vec::new();
    for i in 0..count {
        match i % 4 {
            0 => data.extend_from_slice(b"\x1b[A"), // Up
            1 => data.extend_from_slice(b"\x1b[B"), // Down
            2 => data.extend_from_slice(b"\x1b[C"), // Right
            _ => data.extend_from_slice(b"\x1b[D"), // Left
        }
        data.push(b'X'); // Character after movement
    }
    data
}

fn generate_mixed_sequences(size: usize) -> Vec<u8> {
    // Generate a realistic mix of text and escape sequences
    let mut data = Vec::new();
    let mut remaining = size;
    let mut i = 0;

    while remaining > 0 {
        match i % 5 {
            0 => {
                // Plain text
                let text = "Hello, World! ";
                let bytes = text.as_bytes();
                if remaining >= bytes.len() {
                    data.extend_from_slice(bytes);
                    remaining -= bytes.len();
                } else {
                    data.extend_from_slice(&bytes[..remaining]);
                    remaining = 0;
                }
            }
            1 => {
                // Color change
                let seq = format!("\x1b[{}m", 30 + (i % 8)).into_bytes();
                if remaining >= seq.len() {
                    data.extend_from_slice(&seq);
                    remaining -= seq.len();
                } else {
                    break;
                }
            }
            2 => {
                // Cursor position
                let seq = format!("\x1b[{};{}H", (i % 24) + 1, (i % 80) + 1).into_bytes();
                if remaining >= seq.len() {
                    data.extend_from_slice(&seq);
                    remaining -= seq.len();
                } else {
                    break;
                }
            }
            3 => {
                // Clear line
                let seq = b"\x1b[2K";
                if remaining >= seq.len() {
                    data.extend_from_slice(seq);
                    remaining -= seq.len();
                } else {
                    break;
                }
            }
            _ => {
                // Tab and newline
                let seq = b"\t\n";
                if remaining >= seq.len() {
                    data.extend_from_slice(seq);
                    remaining -= seq.len();
                } else {
                    break;
                }
            }
        }
        i += 1;
    }

    data
}

fn generate_scrollback_buffer(lines: usize) -> Vec<u8> {
    // Generate data simulating scrollback buffer content
    let mut data = Vec::new();
    for i in 0..lines {
        data.extend_from_slice(format!("[{:06}] Log line with some content here\n", i).as_bytes());
        if i % 10 == 0 {
            // Add some color every 10 lines
            data.extend_from_slice(
                format!("\x1b[31m[ERROR]\x1b[0m Error at line {}\n", i).as_bytes(),
            );
        }
    }
    data
}

fn bench_plain_text(c: &mut Criterion) {
    let mut group = c.benchmark_group("vte_plain_text");

    for size in [100, 1000, 10000, 100000].iter() {
        let data = generate_text_data(*size);
        group.throughput(Throughput::Bytes(*size as u64));
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, _| {
            let mut parser = Parser::new();
            let mut performer = BenchPerformer::new();

            b.iter(|| {
                performer.reset();
                for byte in &data {
                    parser.advance(&mut performer, *byte);
                }
                black_box(&performer.operations);
            });
        });
    }

    group.finish();
}

fn bench_ansi_colors(c: &mut Criterion) {
    let mut group = c.benchmark_group("vte_ansi_colors");

    for count in [10, 100, 1000, 10000].iter() {
        let data = generate_ansi_colors(*count);
        group.throughput(Throughput::Bytes(data.len() as u64));
        group.bench_with_input(BenchmarkId::from_parameter(count), count, |b, _| {
            let mut parser = Parser::new();
            let mut performer = BenchPerformer::new();

            b.iter(|| {
                performer.reset();
                for byte in &data {
                    parser.advance(&mut performer, *byte);
                }
                black_box(&performer.operations);
            });
        });
    }

    group.finish();
}

fn bench_cursor_movements(c: &mut Criterion) {
    let mut group = c.benchmark_group("vte_cursor");

    for count in [100, 1000, 10000].iter() {
        let data = generate_cursor_movements(*count);
        group.throughput(Throughput::Bytes(data.len() as u64));
        group.bench_with_input(BenchmarkId::from_parameter(count), count, |b, _| {
            let mut parser = Parser::new();
            let mut performer = BenchPerformer::new();

            b.iter(|| {
                performer.reset();
                for byte in &data {
                    parser.advance(&mut performer, *byte);
                }
                black_box(&performer.operations);
            });
        });
    }

    group.finish();
}

fn bench_mixed_sequences(c: &mut Criterion) {
    let mut group = c.benchmark_group("vte_mixed");

    for size in [1000, 10000, 100000].iter() {
        let data = generate_mixed_sequences(*size);
        group.throughput(Throughput::Bytes(data.len() as u64));
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, _| {
            let mut parser = Parser::new();
            let mut performer = BenchPerformer::new();

            b.iter(|| {
                performer.reset();
                for byte in &data {
                    parser.advance(&mut performer, *byte);
                }
                black_box(&performer.operations);
            });
        });
    }

    group.finish();
}

fn bench_scrollback(c: &mut Criterion) {
    let mut group = c.benchmark_group("vte_scrollback");

    for lines in [100, 1000, 10000].iter() {
        let data = generate_scrollback_buffer(*lines);
        group.throughput(Throughput::Bytes(data.len() as u64));
        group.bench_with_input(BenchmarkId::from_parameter(lines), lines, |b, _| {
            let mut parser = Parser::new();
            let mut performer = BenchPerformer::new();

            b.iter(|| {
                performer.reset();
                for byte in &data {
                    parser.advance(&mut performer, *byte);
                }
                black_box(&performer.operations);
            });
        });
    }

    group.finish();
}

fn bench_batch_processing(c: &mut Criterion) {
    let mut group = c.benchmark_group("vte_batch");

    for chunk_size in [1, 10, 100, 1000, 4096].iter() {
        let data = generate_mixed_sequences(100000);
        group.bench_with_input(
            BenchmarkId::from_parameter(chunk_size),
            chunk_size,
            |b, &chunk_size| {
                let mut parser = Parser::new();
                let mut performer = BenchPerformer::new();

                b.iter(|| {
                    performer.reset();
                    for chunk in data.chunks(chunk_size) {
                        for byte in chunk {
                            parser.advance(&mut performer, *byte);
                        }
                    }
                    black_box(&performer.operations);
                });
            },
        );
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_plain_text,
    bench_ansi_colors,
    bench_cursor_movements,
    bench_mixed_sequences,
    bench_scrollback,
    bench_batch_processing
);
criterion_main!(benches);
