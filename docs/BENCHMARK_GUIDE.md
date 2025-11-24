# Plugin Performance Benchmark Guide

This guide explains how to run and interpret the plugin system performance benchmarks.

## Quick Start

```bash
# Run all plugin benchmarks
cargo bench -p scarab-daemon --bench plugin_benchmarks

# Open the HTML report
open target/criterion/report/index.html
```

## Benchmark Categories

### 1. Plugin Loading (`loading_benches`)

Measures the time to load and initialize plugins from disk.

**Tests**:
- `plugin_loading_bytecode/minimal_bytecode` - Load minimal .fzb plugin
- `plugin_loading_bytecode/complex_bytecode` - Load complex .fzb plugin
- `plugin_loading_script/minimal` - Load minimal .fsx script
- `plugin_loading_script/with_function` - Load .fsx with functions
- `plugin_loading_script/with_hooks` - Load .fsx with hook definitions
- `plugin_loading_script/complex` - Load complex .fsx script

**Expected Results**:
- Bytecode: 100-500μs
- Scripts: 2-20ms (depends on complexity)

**Run specific group**:
```bash
cargo bench --bench plugin_benchmarks -- loading
```

---

### 2. Hook Dispatch (`dispatch_benches`)

Measures the latency of dispatching hooks to plugins.

**Tests**:
- `hook_dispatch_output/*` - Output hook latency
- `hook_dispatch_input/*` - Input hook latency
- `hook_dispatch_resize/*` - Resize hook latency

Each category tests:
- `no_plugins` - Baseline manager overhead
- `single_noop` - Single no-op plugin overhead
- `single_processing` - Single processing plugin overhead

**Expected Results**:
- No plugins: 1-2μs
- Single plugin: 3-15μs
- Processing plugin: 10-20μs

**Run specific group**:
```bash
cargo bench --bench plugin_benchmarks -- dispatch
```

---

### 3. Plugin Chaining (`chaining_benches`)

Measures performance degradation with multiple plugins.

**Tests**:
- `plugin_chaining/noop_plugins/*` - 1, 2, 5, 10, 20 no-op plugins
- `plugin_chaining/processing_plugins/*` - 1, 2, 5, 10, 20 processing plugins

**Expected Results**:
- Linear scaling up to 10 plugins
- 3-4μs overhead per plugin
- Total < 100μs for 20 plugins

**Run specific group**:
```bash
cargo bench --bench plugin_benchmarks -- chaining
```

---

### 4. VM Execution (`vm_benches`)

Measures Fusabi VM performance and adapter overhead.

**Tests**:
- `vm_execution/direct_minimal` - Direct VM execution (minimal bytecode)
- `vm_execution/direct_complex` - Direct VM execution (complex bytecode)
- `vm_execution/adapter_minimal` - Through plugin adapter
- `script_compilation/*` - Compilation pipeline stages
- `vm_cache/cache_hit` - VM cache hit performance
- `vm_cache/no_cache` - VM cache miss performance

**Expected Results**:
- Direct VM: 200-500ns
- Adapter overhead: 2-5μs
- Compilation: 2-10ms
- Cache hit: 2-3μs
- Cache miss: 5-10ms

**Run specific group**:
```bash
cargo bench --bench plugin_benchmarks -- vm
```

---

### 5. Throughput (`throughput_benches`)

Measures bulk processing performance.

**Tests**:
- `output_throughput/{no_plugins,single_plugin,five_plugins}/{10,100,1000}` - Lines processed
- `input_throughput/{no_plugins,single_plugin}/{10,100,1000}` - Input events processed

**Expected Results**:
- No plugins: 100,000+ lines/sec
- Single plugin: 50,000+ lines/sec
- Five plugins: 10,000+ lines/sec

**Run specific group**:
```bash
cargo bench --bench plugin_benchmarks -- throughput
```

---

### 6. Realistic Workload (`workload_benches`)

Simulates real-world terminal usage patterns.

**Tests**:
- `realistic_workload/no_plugins` - Baseline terminal session
- `realistic_workload/three_plugins` - With 3 typical plugins

Simulates:
- 100 output lines
- 20 input events
- 2 resize events

**Expected Results**:
- No plugins: < 200μs total
- Three plugins: < 3ms total

**Run specific group**:
```bash
cargo bench --bench plugin_benchmarks -- realistic
```

---

## Advanced Usage

### Save Baseline for Comparison

```bash
# Run benchmarks and save as baseline
cargo bench --bench plugin_benchmarks -- --save-baseline main

# Make changes, then compare
cargo bench --bench plugin_benchmarks -- --baseline main
```

### Profile with Tracy

```bash
# Build with Tracy support
cargo bench --bench plugin_benchmarks --features tracy

# Connect Tracy profiler (separate tool)
tracy
```

### Profile with Puffin

```bash
# Build with Puffin support
cargo bench --bench plugin_benchmarks --features puffin-profiling
```

### Run Quick Benchmarks (fewer samples)

```bash
# Faster but less accurate
cargo bench --bench plugin_benchmarks -- --quick
```

### Run Only Specific Tests

```bash
# Only bytecode loading
cargo bench --bench plugin_benchmarks -- loading_bytecode

# Only output hooks
cargo bench --bench plugin_benchmarks -- hook_dispatch_output

# Only 5-plugin tests
cargo bench --bench plugin_benchmarks -- "5 plugins"
```

---

## Interpreting Results

### Criterion Output Format

```
plugin_loading_bytecode/minimal_bytecode
                        time:   [189.23 μs 191.45 μs 193.89 μs]
                        change: [-2.1234% +0.1245% +2.4531%] (p = 0.92 > 0.05)
                        No change in performance detected.
```

**Fields**:
- `time`: [lower bound, estimate, upper bound] with 95% confidence
- `change`: Performance difference from previous run
- `p value`: Statistical significance (< 0.05 = significant change)

### Performance Grades

- **Excellent (A+)**: < 1μs latency
- **Good (A)**: 1-10μs latency
- **Acceptable (B)**: 10-50μs latency
- **Needs Attention (C)**: 50-100μs latency
- **Problem (D/F)**: > 100μs latency

### Red Flags

Watch for:
- **Regression**: Performance worse than baseline
- **High variance**: Large confidence intervals
- **Non-linear scaling**: Plugin chaining overhead grows faster than expected
- **Cache misses**: VM cache performance degradation

---

## Continuous Integration

### Add to CI Pipeline

```yaml
# .github/workflows/benchmarks.yml
name: Performance Benchmarks

on:
  push:
    branches: [main]
  pull_request:

jobs:
  benchmark:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      - name: Run benchmarks
        run: cargo bench --bench plugin_benchmarks -- --output-format bencher
      - name: Store benchmark result
        uses: benchmark-action/github-action-benchmark@v1
        with:
          tool: 'cargo'
          output-file-path: target/criterion/output.txt
```

---

## Troubleshooting

### Benchmarks Take Too Long

```bash
# Reduce sample size
cargo bench --bench plugin_benchmarks -- --sample-size 10

# Or use quick mode
cargo bench --bench plugin_benchmarks -- --quick
```

### Inconsistent Results

```bash
# Increase measurement time for stability
cargo bench --bench plugin_benchmarks -- --measurement-time 10

# Or increase sample size
cargo bench --bench plugin_benchmarks -- --sample-size 200
```

### Compare Across Machines

Criterion generates relative measurements, so results vary by hardware. Use:
- Percentage change (not absolute times) for comparisons
- Same machine for before/after comparisons
- Standardized CI environment for consistency

---

## Performance Targets

### Plugin System Goals

| Operation | Target | Stretch Goal |
|-----------|--------|--------------|
| Load .fzb plugin | < 500μs | < 200μs |
| Load .fsx plugin | < 20ms | < 5ms |
| Output hook (single plugin) | < 50μs | < 10μs |
| Input hook (single plugin) | < 50μs | < 10μs |
| VM execution | < 1μs | < 500ns |
| 5-plugin chain | < 100μs | < 50μs |
| Throughput (5 plugins) | > 1K lines/sec | > 10K lines/sec |

### Frame Budget (60 FPS = 16.67ms)

Recommended per-frame budget allocation:
- VTE Parsing: 2% (333μs)
- Plugin Processing: 5% (833μs)
- Rendering: 30% (5ms)
- IPC/Sync: 3% (500μs)
- Slack: 60% (10ms)

---

## Related Documentation

- [PLUGIN_PERFORMANCE_REPORT.md](./PLUGIN_PERFORMANCE_REPORT.md) - Detailed performance analysis
- [CLAUDE.md](../CLAUDE.md) - Project overview and architecture
- [Plugin API Documentation](../crates/scarab-plugin-api/README.md)

---

**Last Updated**: 2025-11-24
**Benchmark Version**: 1.0.0
