# Performance Regression Guard

This document describes Scarab's performance regression testing strategy.

## Overview

Performance regression guards prevent performance degradation in critical code paths:

- VTE parsing throughput
- Rendering FPS
- IPC latency
- Plugin load time
- Memory usage

## Benchmark Baselines

### Critical Performance Metrics

| Metric | Baseline | Threshold | Test |
|--------|----------|-----------|------|
| VTE Parsing | 50 MB/s | -10% | `cargo bench vte_parsing` |
| Render FPS | 60 FPS | -5% | `cargo bench text_rendering` |
| IPC Latency | <5ms | +20% | `cargo bench ipc_throughput` |
| Plugin Load | <10ms | +50% | `cargo bench plugin_benchmarks` |
| Startup Time | <200ms | +30% | Manual test |

## Running Benchmarks

### All Benchmarks

```bash
cargo bench --workspace
```

### Specific Benchmarks

```bash
# VTE parsing
cargo bench -p scarab-daemon vte_parsing

# Text rendering
cargo bench -p scarab-client text_rendering

# IPC throughput
cargo bench -p scarab-daemon ipc_throughput

# Plugin loading
cargo bench -p scarab-daemon plugin_benchmarks
```

## CI Integration

### Benchmark Job

The CI workflow includes a benchmark job that:

1. Runs all benchmarks
2. Compares against baseline (stored in git)
3. Fails if thresholds exceeded
4. Comments on PR with results

Example `.github/workflows/performance.yml`:

```yaml
name: Performance Tests

on:
  pull_request:
    branches: [main]
  schedule:
    - cron: '0 0 * * 0'  # Weekly on Sunday

jobs:
  benchmark:
    name: Performance Benchmarks
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable

      - name: Run benchmarks
        run: cargo bench --workspace -- --save-baseline pr-bench

      - name: Compare with main
        run: |
          git fetch origin main
          git checkout origin/main
          cargo bench --workspace -- --save-baseline main-bench
          cargo bench --workspace -- --baseline main-bench --load-baseline pr-bench

      - name: Check thresholds
        run: |
          # Parse benchmark results and check thresholds
          # Fail if any metric exceeds threshold
```

## Benchmark Structure

### Criterion Benchmarks

All benchmarks use Criterion for statistical analysis:

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};

fn vte_parsing_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("vte_parsing");

    // Configure group
    group.sample_size(100);
    group.measurement_time(Duration::from_secs(10));

    // Benchmark different input sizes
    for size in [1024, 4096, 16384, 65536] {
        let input = generate_vte_sequence(size);

        group.bench_with_input(
            BenchmarkId::from_parameter(size),
            &size,
            |b, _| {
                b.iter(|| {
                    parse_vte(black_box(&input))
                });
            },
        );
    }

    group.finish();
}

criterion_group!(benches, vte_parsing_benchmark);
criterion_main!(benches);
```

## Performance Baselines

### Storing Baselines

Baselines are stored in `benchmarks/baselines/`:

```
benchmarks/
├── baselines/
│   ├── v0.1.0.json     # Baseline for v0.1.0
│   ├── v0.2.0.json     # Baseline for v0.2.0
│   └── main.json       # Rolling main branch baseline
└── results/
    └── latest.json     # Latest benchmark run
```

### Updating Baselines

When intentional performance changes are made:

```bash
# Run benchmarks and save as new baseline
cargo bench --workspace -- --save-baseline v0.2.0

# Commit baseline
git add benchmarks/baselines/v0.2.0.json
git commit -m "perf: Update baseline for v0.2.0"
```

## Regression Detection

### Automated Detection

The performance CI job automatically detects regressions:

```bash
# Compare PR against main
cargo bench --workspace -- --baseline main

# Parse output for regressions
if grep -q "regressed" bench-output.txt; then
    echo "Performance regression detected!"
    exit 1
fi
```

### Manual Verification

For investigating regressions:

```bash
# Run comparison
cargo bench --workspace -- --baseline v0.1.0

# Generate detailed report
cargo bench --workspace -- --verbose

# Profile with perf
perf record --call-graph dwarf cargo bench --no-run
perf report
```

## Performance Targets

### VTE Parsing

**Target**: Process 50+ MB/s of ANSI escape sequences

Benchmark:
```rust
fn bench_vte_throughput(c: &mut Criterion) {
    let input = generate_mixed_ansi(1_000_000); // 1MB

    c.bench_function("vte_1mb", |b| {
        b.iter(|| {
            for byte in black_box(&input) {
                parser.advance(*byte);
            }
        });
    });
}
```

### Rendering Performance

**Target**: Maintain 60 FPS with 200x100 terminal

Benchmark:
```rust
fn bench_render_frame(c: &mut Criterion) {
    let terminal_state = setup_200x100_terminal();

    c.bench_function("render_frame_200x100", |b| {
        b.iter(|| {
            render_terminal_frame(black_box(&terminal_state))
        });
    });
}
```

### IPC Latency

**Target**: <5ms round-trip for IPC messages

Benchmark:
```rust
fn bench_ipc_roundtrip(c: &mut Criterion) {
    let (tx, rx) = setup_ipc_channel();

    c.bench_function("ipc_roundtrip", |b| {
        b.iter(|| {
            let msg = IPCMessage::Resize(80, 24);
            tx.send(black_box(msg)).unwrap();
            rx.recv().unwrap()
        });
    });
}
```

## Profiling Integration

### CPU Profiling

Use cargo-flamegraph:

```bash
cargo flamegraph --bench vte_parsing
```

### Memory Profiling

Use valgrind/massif:

```bash
valgrind --tool=massif cargo bench --no-run
ms_print massif.out.*
```

### GPU Profiling

Use tracy or renderdoc:

```bash
cargo bench --features tracy
```

## Continuous Monitoring

### Benchmark Dashboard

Track performance over time:

- Historical trends
- Regression alerts
- Baseline comparisons

### Alerts

Notify team when:

- Any benchmark regresses >10%
- Critical path regresses >5%
- Memory usage increases >20%

## Best Practices

1. **Consistent Environment**: Run benchmarks on dedicated hardware
2. **Warm-up**: Use adequate warm-up iterations
3. **Statistical Significance**: Use Criterion's statistical analysis
4. **Baseline Management**: Update baselines deliberately
5. **Document Changes**: Explain perf improvements/regressions in commits

## Related Documentation

- [Testing Guide](../TESTING.md)
- [Benchmarks](../benchmarks/README.md)
- [Performance Optimization](../PERFORMANCE_OPTIMIZATION_SUMMARY.md)
