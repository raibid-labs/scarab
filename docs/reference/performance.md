# Performance Tuning Guide

Optimize Scarab Terminal for maximum performance.

## Table of Contents

- [Performance Goals](#performance-goals)
- [GPU Backend Selection](#gpu-backend-selection)
- [Font Rendering Optimization](#font-rendering-optimization)
- [Scrollback Management](#scrollback-management)
- [IPC & Shared Memory Tuning](#ipc--shared-memory-tuning)
- [Plugin Performance](#plugin-performance)
- [Benchmarking Tools](#benchmarking-tools)
- [Platform-Specific Optimizations](#platform-specific-optimizations)

---

## Performance Goals

Scarab targets these performance metrics:

| Metric | Target | Typical | Notes |
|--------|--------|---------|-------|
| **Startup Time** | <100ms | 50-80ms | Daemon only (client ~200ms with GPU init) |
| **Input Latency** | <5ms | 2-4ms | Keyboard to screen (excludes shell processing) |
| **Frame Time** | <16ms | 8-12ms | For 60 FPS rendering |
| **Memory Usage** | <100MB | 60-80MB | Daemon + Client combined |
| **CPU Usage (idle)** | <1% | 0.3-0.8% | Both processes |
| **Scrollback Speed** | >10K lines/sec | 15-25K | Rendering speed |

**Reference Hardware**:
- CPU: Intel Core i5-8250U (4 cores, 1.6GHz base)
- GPU: Intel UHD Graphics 620
- RAM: 8GB DDR4
- OS: Ubuntu 22.04 LTS

---

## GPU Backend Selection

Scarab uses `wgpu` for GPU-accelerated rendering. Backend selection impacts performance significantly.

### Automatic Selection

By default, Scarab auto-detects the best backend:

```toml
[ui]
backend = "auto"  # Default
```

**Selection priority**:
1. **Vulkan** - Best performance on modern hardware (Linux/Windows)
2. **Metal** - Best on macOS
3. **DirectX 12** - Windows alternative
4. **OpenGL** - Fallback for older systems

### Manual Backend Selection

Force a specific backend for testing or optimization:

```bash
# Environment variable (temporary)
WGPU_BACKEND=vulkan scarab-client

# Config file (persistent)
```

```toml
[ui]
# Options: "vulkan", "metal", "dx12", "gl", "auto"
backend = "vulkan"
```

### Backend Comparison

| Backend | Linux | macOS | Windows | Performance | Compatibility |
|---------|-------|-------|---------|-------------|---------------|
| **Vulkan** | ✅ Best | ❌ | ✅ Good | Excellent | Modern GPUs only |
| **Metal** | ❌ | ✅ Best | ❌ | Excellent | macOS 10.14+ |
| **DirectX 12** | ❌ | ❌ | ✅ Best | Excellent | Windows 10+ |
| **OpenGL** | ✅ Fallback | ✅ Fallback | ✅ Fallback | Good | Universal |

### Backend Diagnostics

Check which backend is active:

```bash
# Enable wgpu logging
RUST_LOG=wgpu=info scarab-client 2>&1 | grep -i backend

# Expected output:
# [wgpu] Backend: Vulkan
# [wgpu] Adapter: Intel UHD Graphics 620
# [wgpu] Device: Intel UHD Graphics 620 (Vulkan)
```

### Troubleshooting Backends

**Vulkan not working (Linux)**:
```bash
# Install Vulkan runtime
sudo apt install vulkan-tools libvulkan1

# Verify Vulkan support
vulkaninfo | grep deviceName

# Install GPU-specific drivers
sudo apt install mesa-vulkan-drivers      # Intel/AMD
sudo apt install nvidia-vulkan-driver     # NVIDIA
```

**Metal not working (macOS)**:
```bash
# Requires macOS 10.14+
sw_vers

# Check Metal support
system_profiler SPDisplaysDataType | grep Metal
```

**Force software rendering** (debugging only):
```bash
WGPU_BACKEND=gl scarab-client
# Or use llvmpipe (CPU-only):
LIBGL_ALWAYS_SOFTWARE=1 scarab-client
```

---

## Font Rendering Optimization

Font rendering is the most performance-critical operation in a terminal.

### Texture Atlas Sizing

Scarab uses a texture atlas to cache rendered glyphs. Proper sizing balances memory and performance:

```toml
[font]
# Smaller fonts = smaller atlas = faster
size = 12.0  # vs 16.0 (saves ~40% atlas size)

# Reduce fallback fonts
fallback = [
    "DejaVu Sans Mono",  # Only essential fallbacks
    "Noto Color Emoji"   # For emoji
]
# Don't include 10+ fallback fonts
```

**Atlas Size Calculation**:
- 12pt font: ~2048x2048 texture (~16MB VRAM)
- 14pt font: ~2048x2048 texture (~16MB VRAM)
- 16pt font: ~4096x4096 texture (~64MB VRAM)
- 18pt font: ~4096x4096 texture (~64MB VRAM)

**Optimization**: Keep `size` ≤ 14.0 to stay in 2K atlas tier.

### Font Feature Reduction

Disable expensive font features:

```toml
[font]
# Disable ligatures for ~15% render speedup
enable_ligatures = false

# Use thin strokes (macOS only, minor impact)
use_thin_strokes = true

# Reduce line height for tighter packing
line_height = 1.0  # vs 1.2 (saves ~16% vertical space)
```

**Ligature Performance Impact**:
- Disabled: ~8ms frame time
- Enabled: ~12ms frame time (+50% cost)
- Trade-off: Aesthetics vs performance

### Glyph Cache Tuning

Advanced tuning for glyph rasterization cache:

```toml
[font.cache]
# Max cached glyphs (default: 2048)
max_glyphs = 1024  # Reduce for low memory systems

# Cache eviction strategy
eviction = "lru"  # Options: "lru", "lfu", "fifo"

# Pre-cache common glyphs on startup
warm_cache = true
```

**Memory vs Speed Trade-off**:
- Larger cache = more memory, fewer re-rasterizations
- Smaller cache = less memory, potential stutter on new chars

---

## Scrollback Management

Scrollback buffer size directly impacts memory usage and scrolling performance.

### Buffer Size Configuration

```toml
[terminal]
# Recommended settings by use case:

# Minimal (low memory, fast)
scrollback_lines = 1000

# Standard (balanced)
scrollback_lines = 10000  # Default

# Power user (high memory, feature-rich)
scrollback_lines = 50000

# Unlimited (not recommended)
scrollback_lines = 100000  # Max allowed
```

**Memory Impact**:
- 1,000 lines: ~1 MB RAM
- 10,000 lines: ~10 MB RAM
- 50,000 lines: ~50 MB RAM
- 100,000 lines: ~100 MB RAM

**Performance Impact**:
- Scrolling speed: O(n) where n = visible lines (constant time)
- Search speed: O(m) where m = scrollback size
- Memory allocations: Linear with buffer size

### Scrollback Optimization

```toml
[terminal]
# Disable scrollback entirely (tmux-style)
scrollback_lines = 0

# Enable circular buffer (overwrites old lines)
scrollback_mode = "circular"

# Compress old lines (saves ~60% memory)
scrollback_compression = true

# Don't save scrollback in sessions
[sessions]
save_scrollback = false
```

### Search Performance

Large scrollback buffers slow down search. Optimize with:

```toml
[search]
# Limit search scope
max_search_lines = 10000  # Search last 10K lines only

# Enable incremental search
incremental = true

# Use faster search algorithm
algorithm = "boyer-moore"  # vs "naive"
```

**Search Speed** (50,000 line buffer):
- Naive: ~500ms
- Boyer-Moore: ~50ms (10x faster)
- Regex: ~200ms (depends on pattern)

---

## IPC & Shared Memory Tuning

Scarab's zero-copy IPC is already highly optimized, but can be tuned for edge cases.

### Shared Memory Size

```toml
[ipc]
# Default: 16 MB (supports up to 200x100 grid)
shm_size = 16

# Reduce for smaller terminals
shm_size = 8   # 80x24 grid

# Increase for huge grids
shm_size = 32  # 300x150 grid
```

**Formula**:
```
Required SHM = (cols × rows × cell_size) + metadata
cell_size = 16 bytes (character + attributes)
metadata = ~1 MB (ring buffer overhead)

Example (200x100):
200 × 100 × 16 = 320,000 bytes = 0.32 MB
+ 1 MB overhead = ~1.32 MB minimum
Recommended: 16 MB (allows growth)
```

### Socket Configuration

```toml
[ipc]
# Unix domain socket (default, fastest)
mode = "unix"
socket_path = "/tmp/scarab.sock"

# TCP socket (slower, but works over network)
mode = "tcp"
address = "127.0.0.1:7800"

# Named pipe (Windows only)
mode = "pipe"
pipe_name = "\\\\.\\pipe\\scarab"
```

**Performance Comparison**:
- Unix socket: <1μs latency
- TCP localhost: ~5μs latency
- Named pipe: ~2μs latency (Windows)

### Lock-Free Synchronization

Scarab uses `AtomicU64` sequence numbers for lock-free updates. Tuning:

```toml
[ipc]
# Polling interval for client (microseconds)
poll_interval = 100  # Default: 100μs (10,000 Hz)

# Spin vs sleep strategy
poll_strategy = "hybrid"  # Options: "spin", "sleep", "hybrid"

# Spin count before sleep (hybrid mode)
spin_count = 10
```

**Strategy Trade-offs**:
- **Spin**: Lowest latency (<1μs), high CPU usage (~20%)
- **Sleep**: Low CPU (<1%), higher latency (~5ms)
- **Hybrid**: Balanced (spin 10 times, then sleep)

---

## Plugin Performance

Plugins can significantly impact performance. Monitor and optimize:

### Plugin Profiling

```bash
# Enable plugin timing
RUST_LOG=scarab_plugin_api=debug scarab-daemon

# Look for slow plugins in logs
tail -f ~/.local/share/scarab/scarab.log | grep "Plugin.*took"

# Example output:
# [DEBUG] Plugin 'git-status' hook OnOutput took 2.3ms
# [WARN]  Plugin 'slow-plugin' hook OnOutput took 45ms (slow!)
```

### Plugin Optimization Tips

1. **Use compiled bytecode** (`.fzb`) instead of scripts (`.fsx`):
   ```bash
   # Compile plugin
   fusabi-compile my-plugin.fsx -o my-plugin.fzb

   # 10-100x faster execution
   ```

2. **Optimize hook usage**:
   ```fsharp
   // BAD: OnOutput called for EVERY line
   let on_output (ctx: PluginContext) (line: string) =
       if line.Contains("git") then
           // Heavy processing
       else
           ()

   // GOOD: Use OnPostCommand instead (called once per command)
   let on_post_command (ctx: PluginContext) (cmd: string) =
       if cmd.StartsWith("git") then
           // Heavy processing
   ```

3. **Batch operations**:
   ```fsharp
   // BAD: Queue 100 individual overlay updates
   for i in 1..100 do
       ctx.QueueCommand(RemoteCommand.DrawOverlay { ... })

   // GOOD: Batch into single update
   let overlays = [for i in 1..100 -> { ... }]
   ctx.QueueCommand(RemoteCommand.BatchOverlays overlays)
   ```

4. **Cache expensive operations**:
   ```fsharp
   // Cache git status for 1 second
   let mutable last_check = DateTime.MinValue
   let mutable cached_status = None

   let get_git_status () =
       let now = DateTime.Now
       if (now - last_check).TotalSeconds < 1.0 then
           cached_status
       else
           last_check <- now
           cached_status <- Some (check_git_repo())
           cached_status
   ```

### Plugin Limits

```toml
[plugins]
# Maximum plugins to load
max_plugins = 10  # Default: unlimited

# Per-plugin timeout (milliseconds)
hook_timeout = 100  # Kill slow hooks

# Disable plugins in performance mode
performance_mode = true  # Disables all plugins
```

---

## Benchmarking Tools

Measure Scarab performance objectively.

### Built-in Profiling

```bash
# Enable frame timing
SCARAB_PROFILE=1 scarab-client

# Output: Frame times to ~/.local/share/scarab/profile.json

# Analyze with:
scarab-analyze-profile ~/.local/share/scarab/profile.json
```

**Metrics Tracked**:
- Frame time (ms)
- Render time (ms)
- Input latency (ms)
- GPU time (ms)
- Memory usage (MB)

### Tracy Profiler

For deep profiling, use Tracy:

```bash
# Build with Tracy support
cargo build --release --features tracy

# Run Tracy server
tracy-profiler

# Run Scarab client
./target/release/scarab-client

# Profiler GUI shows detailed frame breakdown
```

**Tracy Features**:
- Per-frame CPU timeline
- GPU command buffer visualization
- Memory allocations
- Lock contention
- Call stack sampling

### Criterion Benchmarks

Run Rust benchmarks:

```bash
# Run all benchmarks
cargo bench --workspace

# Run specific benchmark
cargo bench --bench rendering

# Results in target/criterion/report/index.html
```

**Benchmark Suites**:
- `rendering`: Font atlas, glyph caching
- `ipc`: Shared memory, socket throughput
- `vte`: Terminal parsing speed
- `plugins`: Plugin load time, hook overhead

### Input Latency Test

Measure keyboard-to-screen latency:

```bash
# Install latency tester
cargo install --git https://github.com/raibid-labs/scarab --example latency-test

# Run test
latency-test

# Instructions:
# 1. Press space bar rapidly
# 2. Tool measures time until character appears on screen
# 3. Average over 100 samples shown
```

**Expected Latencies**:
- Scarab: 2-4ms
- Alacritty: 2-3ms (reference)
- Kitty: 3-5ms
- GNOME Terminal: 10-15ms

### Scrolling Benchmark

```bash
# Test scrolling performance
scarab-bench scroll --lines 10000

# Output:
# Lines: 10000
# Time: 0.42s
# Speed: 23,809 lines/sec
# Frame drops: 0
```

### Memory Profiler

```bash
# Use heaptrack
heaptrack scarab-client
# Use for 5 minutes
heaptrack_gui heaptrack.scarab-client.*.gz

# Or valgrind (slower)
valgrind --tool=massif scarab-client
ms_print massif.out.*
```

---

## Platform-Specific Optimizations

### Linux

**X11 Optimizations**:
```bash
# Disable compositing for lower latency
xfwm4 --compositor=off  # XFCE
kwriteconfig5 --file kwinrc --group Compositing --key Enabled false  # KDE

# Or use picom with minimal effects
picom --backend glx --vsync --no-fading-openclose
```

**Wayland Optimizations**:
```bash
# Force Wayland native
GDK_BACKEND=wayland scarab-client

# Enable GPU acceleration
export MOZ_ENABLE_WAYLAND=1
```

**Kernel Tuning**:
```bash
# Increase inotify limits (for hot reload)
sudo sysctl fs.inotify.max_user_watches=524288

# Increase shared memory
sudo sysctl kernel.shmmax=536870912  # 512 MB
```

### macOS

**Metal Optimizations**:
```toml
[ui]
backend = "metal"  # Force Metal
vsync = true       # Use display link
```

**Retina Display**:
```toml
[font]
use_thin_strokes = true  # Better for Retina
size = 13.0              # Optimal for Retina
```

**Power Saving**:
```bash
# Reduce frame rate on battery
defaults write com.raibid-labs.scarab battery_fps -int 30
```

### Windows

**DirectX 12 Optimizations**:
```toml
[ui]
backend = "dx12"
vsync = true
```

**Windows Terminal Integration**:
```json
// settings.json
{
  "profiles": {
    "defaults": {
      "useAcrylic": false,  // Disable transparency for performance
      "experimental.retroTerminalEffect": false
    }
  }
}
```

---

## Performance Monitoring

Monitor Scarab in production:

### System Monitoring

```bash
# CPU and memory usage
htop -p $(pgrep scarab)

# GPU usage (NVIDIA)
watch -n 1 nvidia-smi

# GPU usage (AMD)
watch -n 1 radeontop

# Network (if using remote daemon)
iftop -f "port 7800"
```

### Application Metrics

```bash
# Enable Prometheus exporter
SCARAB_METRICS=1 scarab-daemon --metrics-port 9090

# Metrics available at http://localhost:9090/metrics

# Example metrics:
# scarab_frame_time_seconds
# scarab_input_latency_seconds
# scarab_memory_bytes
# scarab_gpu_utilization_percent
# scarab_plugin_hook_duration_seconds
```

### Grafana Dashboard

```bash
# Import dashboard template
grafana-cli plugins install grafana-piechart-panel
# Import: docs/monitoring/scarab-dashboard.json
```

**Key Metrics to Monitor**:
- Frame time (should be <16ms for 60 FPS)
- Input latency (should be <5ms)
- Memory usage (should be stable, not growing)
- GPU utilization (should spike during scrolling)
- Plugin overhead (should be <10% of frame time)

---

## Quick Wins Checklist

Apply these for immediate performance gains:

- [ ] Use Vulkan/Metal backend explicitly
- [ ] Reduce scrollback to 10,000 lines or less
- [ ] Disable font ligatures if not needed
- [ ] Use font size ≤ 14.0
- [ ] Disable cursor blinking
- [ ] Disable UI animations
- [ ] Compile plugins to `.fzb` bytecode
- [ ] Limit to essential plugins only
- [ ] Disable compositor (X11)
- [ ] Use shared memory IPC (not TCP)

**Expected Improvement**: 30-50% faster frame times, 40% lower memory usage.

---

## See Also

- [Configuration Reference](./configuration.md) - All tuning options
- [Troubleshooting](./troubleshooting.md) - Performance issues
- [FAQ](./faq.md) - Common questions
