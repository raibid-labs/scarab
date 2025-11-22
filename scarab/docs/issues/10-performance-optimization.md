# Issue #10: Performance Optimization & Profiling

**Phase**: 4A - Production Hardening
**Priority**: 🔴 Critical
**Workstream**: Performance Engineering
**Estimated Effort**: 1-2 weeks
**Assignee**: Performance Engineer Agent

---

## 🎯 Objective

Profile and optimize the complete Scarab system for production workloads, achieving <1% CPU idle, <50ms P99 frame time, and <100MB baseline memory.

---

## 📋 Background

With all features implemented, we need to:
- Profile actual performance under load
- Identify and eliminate bottlenecks
- Optimize memory usage
- Reduce input latency
- Implement comprehensive benchmarks

---

## ✅ Acceptance Criteria

- [ ] Profiling infrastructure (Tracy, perf, flamegraph)
- [ ] CPU profiling with flame graphs
- [ ] Memory profiling and leak detection
- [ ] Benchmark suite for all components
- [ ] Optimization of hot paths (>5% CPU)
- [ ] Memory optimization (<100MB baseline)
- [ ] Input latency reduction (<10ms P99)
- [ ] GPU memory management
- [ ] Performance regression tests in CI
- [ ] Performance documentation

---

## 🔧 Technical Approach

### Step 1: Profiling Infrastructure
```rust
// Enable Tracy profiling
#[cfg(feature = "profiling")]
use tracy_client;

fn main() {
    #[cfg(feature = "profiling")]
    let _client = tracy_client::Client::start();

    // App initialization
}

// Annotate hot functions
#[profiling::function]
fn process_vte_output(data: &[u8]) {
    // VTE parsing
}
```

### Step 2: CPU Profiling
```bash
# Generate flame graphs
cargo build --release
perf record --call-graph=dwarf target/release/scarab-daemon
perf script | stackcollapse-perf.pl | flamegraph.pl > flame.svg

# Tracy profiling
cargo run --release --features profiling
```

### Step 3: Memory Profiling
```bash
# Valgrind massif
valgrind --tool=massif --massif-out-file=massif.out \
  target/release/scarab-daemon

# heaptrack
heaptrack target/release/scarab-daemon
heaptrack --analyze heaptrack.scarab-daemon.*.gz
```

### Step 4: Benchmark Suite
```rust
// benches/vte_bench.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_vte_parsing(c: &mut Criterion) {
    let data = generate_ansi_sequence(1000);

    c.bench_function("vte_parse_1k", |b| {
        b.iter(|| {
            parse_vte(black_box(&data))
        })
    });
}

criterion_group!(benches, bench_vte_parsing);
criterion_main!(benches);
```

### Step 5: Optimization Targets

**CPU Hot Paths**:
- VTE parsing (target: <2% CPU)
- Text rendering (target: <3% during scroll)
- Shared memory sync (target: <0.5%)

**Memory Optimizations**:
- Glyph atlas reuse
- Scrollback buffer ring allocation
- Plugin state management

**Latency Optimizations**:
- Reduce event loop overhead
- Optimize IPC message serialization
- Cache frequently accessed data

---

## 📦 Deliverables

1. **Profiling**: Tracy integration, perf scripts
2. **Benchmarks**: Criterion benchmark suite
3. **Optimizations**: Code changes for hot paths
4. **Documentation**: Performance guide
5. **CI**: Performance regression tests

---

## 🔗 Dependencies

- **Depends On**: All Phase 1-3 complete
- **Blocks**: None (final phase)

---

## 📚 Resources

- [Tracy Profiler](https://github.com/wolfpld/tracy)
- [Criterion.rs](https://bheisler.github.io/criterion.rs/book/)
- [Rust Profiling Guide](https://nnethercote.github.io/perf-book/)
- [perf Examples](https://www.brendangregg.com/perf.html)

---

## 🎯 Success Metrics

- ✅ CPU idle <1%
- ✅ CPU scroll <5%
- ✅ P99 frame time <50ms
- ✅ P99 input latency <10ms
- ✅ Memory baseline <100MB
- ✅ GPU memory <150MB
- ✅ No memory leaks
- ✅ Benchmark suite in CI

---

## 💡 Optimization Ideas

### Memory
- Use `SmallVec` for common cases
- Pool allocations for cells
- Compact internal representations
- Lazy initialization

### CPU
- SIMD for color conversion
- Batch operations
- Reduce allocations
- Cache computed values

### GPU
- Texture atlas packing
- Instanced rendering
- Reduce draw calls
- Shader optimization

---

**Created**: 2025-11-21
**Labels**: `phase-4`, `critical`, `performance`, `optimization`
