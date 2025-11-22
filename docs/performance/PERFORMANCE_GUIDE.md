# Scarab Performance Optimization Guide

## Table of Contents

1. [Performance Targets](#performance-targets)
2. [Profiling Tools](#profiling-tools)
3. [Running Benchmarks](#running-benchmarks)
4. [Optimization Strategies](#optimization-strategies)
5. [Performance Monitoring](#performance-monitoring)
6. [Troubleshooting](#troubleshooting)

---

## Performance Targets

Scarab is designed to meet the following performance targets:

| Metric | Target | Current | Status |
|--------|--------|---------|--------|
| CPU (idle) | < 1% | TBD | 🔄 |
| CPU (scrolling) | < 5% | TBD | 🔄 |
| P99 Frame Time | < 50ms | TBD | 🔄 |
| P99 Input Latency | < 10ms | TBD | 🔄 |
| Memory (baseline) | < 100MB | TBD | 🔄 |
| GPU Memory | < 150MB | TBD | 🔄 |
| VTE Parsing | < 2% CPU | TBD | 🔄 |
| Text Rendering | < 3% CPU | TBD | 🔄 |
| Shared Memory Sync | < 0.5% CPU | TBD | 🔄 |

---

## Profiling Tools

### 1. Built-in Profiling Infrastructure

Scarab includes comprehensive profiling support through multiple backends:

#### Tracy Integration

Tracy provides real-time profiling with minimal overhead:

```bash
# Build with Tracy support
cargo build --release --features=tracy

# Run the daemon
./target/release/scarab-daemon

# Connect with Tracy Profiler GUI to capture traces
```

#### Puffin Integration

For in-application profiling visualization:

```bash
# Build with Puffin support
cargo build --release --features=puffin-profiling

# Profiling data will be available through the debug UI
```

### 2. CPU Profiling

#### Using perf (Linux)

```bash
# Record CPU profile
sudo perf record -F 99 -a -g target/release/scarab-daemon

# Generate flame graph
perf script | flamegraph > flamegraph.svg

# Or use the provided script
./scripts/profile.sh cpu 30 ./profiling-results
```

#### Using Instruments (macOS)

```bash
# Build with debug symbols
cargo build --profile=profiling

# Profile with Instruments
instruments -t "Time Profiler" target/profiling/scarab-daemon
```

### 3. Memory Profiling

#### Valgrind Massif

```bash
# Heap profiling
valgrind --tool=massif --massif-out-file=massif.out target/release/scarab-daemon

# Generate report
ms_print massif.out > massif-report.txt
```

#### Memory Leak Detection

```bash
# Check for leaks
valgrind --leak-check=full --show-leak-kinds=all \
         --track-origins=yes --log-file=memcheck.log \
         target/release/scarab-daemon

# Or use the script
./scripts/profile.sh memory 30 ./profiling-results
```

---

## Running Benchmarks

### Quick Start

```bash
# Run all benchmarks
cargo bench

# Run specific benchmark suite
cargo bench --bench vte_parsing
cargo bench --bench shared_memory
cargo bench --bench ipc_throughput
cargo bench --bench text_rendering
cargo bench --bench gpu_operations
```

### Benchmark Suites

#### VTE Parsing Benchmarks

Tests terminal output parsing performance:
- Plain text processing
- ANSI color sequences
- Cursor movements
- Mixed sequences
- Scrollback buffer
- Batch processing

```bash
cargo bench --bench vte_parsing
```

#### Shared Memory Benchmarks

Tests shared memory operations:
- Creation/destruction
- Read/write throughput
- Atomic operations
- Concurrent access
- Memory barriers
- Ring buffer operations

```bash
cargo bench --bench shared_memory
```

#### IPC Throughput Benchmarks

Tests inter-process communication:
- Channel throughput
- Message latency
- Multi-producer scenarios
- Burst handling
- Serialization overhead

```bash
cargo bench --bench ipc_throughput
```

#### Text Rendering Benchmarks

Tests text rendering performance:
- Text shaping
- Line wrapping
- Unicode processing
- Glyph caching
- Scrolling
- Syntax highlighting

```bash
cargo bench --bench text_rendering
```

#### GPU Operations Benchmarks

Tests GPU-related operations:
- Buffer uploads
- Texture management
- Mesh generation
- Vertex transformation
- Draw call batching
- Instanced rendering

```bash
cargo bench --bench gpu_operations
```

### Interpreting Results

Criterion generates HTML reports in `target/criterion/`:

```bash
# Open the report
open target/criterion/report/index.html
```

Key metrics to watch:
- **Throughput**: Bytes/Elements processed per second
- **Latency**: Time per operation (lower is better)
- **Variance**: Consistency of performance

---

## Optimization Strategies

### 1. VTE Parsing Optimization

**Current bottlenecks:**
- Character-by-character processing
- State machine overhead

**Optimization approaches:**
```rust
// Batch processing
fn process_vte_batch(data: &[u8]) {
    // Process in chunks for better cache locality
    for chunk in data.chunks(4096) {
        parser.advance_batch(chunk);
    }
}

// SIMD acceleration for plain text detection
#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::*;

fn is_plain_text_simd(data: &[u8]) -> bool {
    // Use SIMD to quickly detect escape sequences
}
```

### 2. Text Rendering Optimization

**Current bottlenecks:**
- Glyph rasterization
- Layout recalculation

**Optimization approaches:**
```rust
// Glyph atlas caching
struct GlyphAtlas {
    cache: HashMap<GlyphKey, AtlasEntry>,
    texture: Texture2D,
}

// Incremental layout updates
fn update_layout_incremental(buffer: &mut TextBuffer, change: TextChange) {
    // Only recalculate affected lines
    let affected_lines = change.get_affected_lines();
    buffer.reshape_lines(affected_lines);
}
```

### 3. Shared Memory Optimization

**Current bottlenecks:**
- Synchronization overhead
- Cache coherency

**Optimization approaches:**
```rust
// Lock-free ring buffer
struct LockFreeRingBuffer {
    head: AtomicUsize,
    tail: AtomicUsize,
    data: Box<[u8]>,
}

// Batch updates to reduce synchronization
fn batch_update_shared_state(updates: Vec<StateUpdate>) {
    let mut guard = shared_state.lock();
    for update in updates {
        guard.apply(update);
    }
}
```

### 4. Memory Management

**Strategies:**
```rust
// Object pooling
struct CellPool {
    free_list: Vec<Cell>,
}

// Arena allocation for temporary data
struct FrameArena {
    buffer: Vec<u8>,
    offset: usize,
}

// Small string optimization
enum CompactString {
    Inline([u8; 23], u8),
    Heap(String),
}
```

---

## Performance Monitoring

### Runtime Metrics Collection

The daemon includes built-in metrics collection:

```rust
use scarab_daemon::profiling::MetricsCollector;

let metrics = MetricsCollector::new();

// Record metrics during operation
metrics.record_frame_time(frame_duration);
metrics.record_vte_parse(parse_duration, bytes_processed);

// Get performance report
let report = metrics.report();
report.print_summary();
```

### Continuous Monitoring

Enable metrics logging:

```bash
RUST_LOG=debug cargo run --release --features=profiling
```

Monitor key metrics:
```bash
# Watch CPU usage
watch -n 1 'ps aux | grep scarab-daemon'

# Monitor memory growth
while true; do
    ps -o pid,vsz,rss,comm -p $(pgrep scarab-daemon)
    sleep 1
done
```

### CI Performance Tracking

GitHub Actions workflow automatically:
1. Runs benchmarks on every commit
2. Detects performance regressions
3. Checks for memory leaks
4. Validates performance targets

---

## Troubleshooting

### High CPU Usage

1. **Check VTE parsing:**
```bash
cargo bench --bench vte_parsing -- --profile-time 30
```

2. **Profile with flamegraph:**
```bash
cargo flamegraph --bench vte_parsing
```

3. **Common fixes:**
   - Batch VTE processing
   - Cache parsed sequences
   - Use lookup tables for state transitions

### Memory Leaks

1. **Run Valgrind:**
```bash
valgrind --leak-check=full target/release/scarab-daemon
```

2. **Check for growing allocations:**
```rust
// Add allocation tracking
#[global_allocator]
static ALLOC: TracingAllocator = TracingAllocator;
```

3. **Common sources:**
   - Unbounded channels
   - Cached data without eviction
   - Circular references in async code

### Frame Drops

1. **Check render timing:**
```bash
cargo bench --bench gpu_operations
```

2. **Profile GPU usage:**
   - Use RenderDoc or NSight
   - Check draw call count
   - Monitor GPU memory

3. **Optimizations:**
   - Batch draw calls
   - Use instanced rendering
   - Implement frustum culling

### Input Latency

1. **Measure end-to-end latency:**
```rust
let start = Instant::now();
// Process input
let latency = start.elapsed();
metrics.record_input_latency(latency);
```

2. **Common causes:**
   - Synchronous IPC
   - Lock contention
   - Event queue backlog

3. **Solutions:**
   - Use async message passing
   - Reduce critical sections
   - Priority queue for input events

---

## Best Practices

### 1. Profile First

Always measure before optimizing:
```bash
# Complete profiling session
./scripts/profile.sh all 60 ./baseline-profile

# Make changes

# Profile again
./scripts/profile.sh all 60 ./optimized-profile

# Compare results
diff baseline-profile/summary.txt optimized-profile/summary.txt
```

### 2. Incremental Optimization

Focus on the biggest bottlenecks:
1. Identify top 3 CPU consumers
2. Optimize the worst offender
3. Re-profile and validate improvement
4. Repeat

### 3. Regression Prevention

Add benchmarks for optimized code:
```rust
#[bench]
fn bench_optimized_vte_parser(b: &mut Bencher) {
    let data = generate_test_data();
    b.iter(|| {
        optimized_parse(&data)
    });
}
```

### 4. Document Optimizations

For each optimization:
- Document the problem
- Explain the solution
- Show before/after metrics
- Note any trade-offs

---

## Advanced Topics

### SIMD Optimization

Example of SIMD-accelerated color conversion:
```rust
#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::*;

unsafe fn convert_rgba_simd(src: &[u8], dst: &mut [f32]) {
    let scale = _mm_set1_ps(1.0 / 255.0);

    for (chunk_in, chunk_out) in src.chunks_exact(16)
        .zip(dst.chunks_exact_mut(16)) {
        // Load 16 bytes (4 RGBA pixels)
        let bytes = _mm_loadu_si128(chunk_in.as_ptr() as *const __m128i);

        // Convert to floats and scale
        let floats = _mm_cvtepi32_ps(bytes);
        let scaled = _mm_mul_ps(floats, scale);

        // Store results
        _mm_storeu_ps(chunk_out.as_mut_ptr(), scaled);
    }
}
```

### Custom Allocators

Using specialized allocators for hot paths:
```rust
use mimalloc::MiMalloc;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

// Or use arena allocation for frame data
struct FrameAllocator {
    arena: bumpalo::Bump,
}

impl FrameAllocator {
    fn alloc<T>(&self, value: T) -> &T {
        self.arena.alloc(value)
    }
}
```

### Profile-Guided Optimization

Enable PGO for release builds:
```toml
[profile.release]
lto = "fat"
pgo = true
```

Build process:
```bash
# Generate profile data
cargo pgo build
cargo pgo run -- --bench

# Build with profile data
cargo pgo optimize
```

---

## Resources

### Tools
- [Tracy Profiler](https://github.com/wolfpld/tracy)
- [Criterion.rs](https://bheisler.github.io/criterion.rs/book/)
- [Flamegraph](https://github.com/flamegraph-rs/flamegraph)
- [Valgrind](https://valgrind.org/)

### Documentation
- [Rust Performance Book](https://nnethercote.github.io/perf-book/)
- [The Flame Graph](https://www.brendangregg.com/flamegraphs.html)
- [Linux Perf Examples](https://www.brendangregg.com/perf.html)

### Papers
- "Efficient Terminal Emulation" - Thomas Dickey
- "GPU Text Rendering" - Valve's Improved Alpha-Tested Magnification
- "Lock-Free Ring Buffers" - Mechanical Sympathy

---

*Last updated: 2025-11-21*