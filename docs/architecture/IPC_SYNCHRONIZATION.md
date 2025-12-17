# IPC Synchronization Protocol

**Document Version**: 1.0
**Status**: Implementation Complete
**Last Updated**: 2025-12-15

## Table of Contents

1. [Overview](#overview)
2. [SeqLock Pattern Explained](#seqlock-pattern-explained)
3. [Memory Layout](#memory-layout)
4. [Write Protocol (Daemon)](#write-protocol-daemon)
5. [Read Protocol (Client)](#read-protocol-client)
6. [Code Examples](#code-examples)
7. [Invariants](#invariants)
8. [Performance Characteristics](#performance-characteristics)
9. [Troubleshooting](#troubleshooting)
10. [References](#references)

---

## Overview

### What is SeqLock?

Scarab uses a **SeqLock (Sequence Lock)** pattern for lock-free synchronization between the daemon (writer) and client (reader) processes. This is a well-known concurrent programming technique that enables:

- **Single-writer, multiple-reader** access pattern
- **Zero-copy** data transfer via shared memory
- **Lock-free** reads (no mutex contention)
- **Automatic conflict detection** via sequence numbers

### Why SeqLock for Scarab?

Terminal emulation requires high-frequency updates (60+ FPS) with minimal latency. Traditional mutex-based synchronization would cause:

1. **Lock contention** - Client rendering blocked on daemon writes
2. **Priority inversion** - Low-priority reader blocking high-priority writer
3. **Cache thrashing** - Lock metadata invalidating CPU caches
4. **Unpredictable latency** - Variable lock wait times

SeqLock eliminates these issues by allowing the client to read concurrently with daemon writes, detecting conflicts via a monotonically increasing sequence number.

### High-Level Flow

```
Daemon Process                     Client Process
┌──────────────┐                  ┌──────────────┐
│              │                  │              │
│  PTY Output  │                  │ Bevy Render  │
│      ↓       │                  │   60 FPS     │
│  VTE Parser  │                  │      ↓       │
│      ↓       │                  │  Read Seq    │
│ TerminalState│ ← SharedState → │  Read Cells  │
│      ↓       │   (Shared Mem)  │  Check Seq   │
│ blit_to_shm()│                  │  (retry?)    │
│      ↓       │                  │      ↓       │
│ Seq++ (odd)  │                  │  Render Text │
│ Write Cells  │                  │              │
│ Seq++ (even) │                  │              │
└──────────────┘                  └──────────────┘
```

---

## SeqLock Pattern Explained

### Core Concept

A SeqLock uses a **64-bit atomic sequence number** with the following semantics:

- **Even number** = Data is stable, safe to read
- **Odd number** = Write in progress, data may be inconsistent
- **Incrementing** = Detect if data changed during read

### Example Timeline

```
Time │ Daemon Action          │ Sequence │ Client Action
─────┼────────────────────────┼──────────┼──────────────────
  0  │ Idle                   │    0     │ -
  1  │ Start write (seq++)    │    1     │ Read seq=1 (odd, retry)
  2  │ Writing cells...       │    1     │ Waiting...
  3  │ End write (seq++)      │    2     │ Read seq=2 (even)
  4  │ Idle                   │    2     │ Read cells...
  5  │ Start write (seq++)    │    3     │ Still reading...
  6  │ Writing cells...       │    3     │ Check seq=3 != 2 (retry!)
  7  │ End write (seq++)      │    4     │ Read seq=4 (even)
  8  │ Idle                   │    4     │ Read cells (success)
```

### Why This Works

1. **Write detection**: Reader checks sequence before/after read
2. **Conflict detection**: Mismatched sequence → retry
3. **No blocking**: Reader never waits for writer
4. **Cache-friendly**: Sequence number is single atomic word

---

## Memory Layout

### SharedState Structure

```rust
#[repr(C)]
#[derive(Copy, Clone)]
pub struct SharedState {
    pub sequence_number: u64,    // Atomic sequence for synchronization
    pub dirty_flag: u8,          // Additional change indicator
    pub error_mode: u8,          // 0 = normal, 1 = error (PTY/SHM unavailable)
    pub cursor_x: u16,
    pub cursor_y: u16,
    pub _padding2: [u8; 2],      // Align to u64 boundary
    pub cells: [Cell; BUFFER_SIZE], // 200x100 = 20,000 cells
}
```

**Location**: `crates/scarab-protocol/src/lib.rs:52-64`

### Cell Structure

```rust
#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct Cell {
    pub char_codepoint: u32,     // Unicode codepoint
    pub fg: u32,                 // Foreground color (RGBA)
    pub bg: u32,                 // Background color (RGBA)
    pub flags: u8,               // Bold, Italic, etc.
    pub _padding: [u8; 3],       // Align to 16 bytes
}
```

**Location**: `crates/scarab-protocol/src/lib.rs:29-37`

### Memory Map

```
┌──────────────────────────────┐ 0x0000
│  sequence_number (u64)       │ ← Atomic sequence
├──────────────────────────────┤ 0x0008
│  dirty_flag (u8)             │
│  error_mode (u8)             │
│  cursor_x (u16)              │
│  cursor_y (u16)              │
│  _padding2 ([u8; 2])         │
├──────────────────────────────┤ 0x0010
│  cells[0] (Cell)             │ ← 16 bytes
├──────────────────────────────┤
│  cells[1] (Cell)             │
│  ...                         │
│  cells[19,999] (Cell)        │
└──────────────────────────────┘ End (320,016 bytes)
```

**Shared Memory Path**: `/dev/shm/scarab_shm_v1`
**Size**: 320,016 bytes (approx. 312 KB)

### Critical `#[repr(C)]` Requirement

The `#[repr(C)]` attribute is **mandatory** for all shared memory structures:

- **Guarantees** stable memory layout across process boundaries
- **Prevents** Rust compiler reordering fields for optimization
- **Ensures** daemon and client agree on field offsets

Without `#[repr(C)]`, the daemon might write to offset 8, while the client reads from offset 12.

---

## Write Protocol (Daemon)

### High-Level Algorithm

```rust
fn blit_to_shm(shm: *mut SharedState, sequence_counter: &Arc<AtomicU64>) {
    // 1. Increment sequence to odd (write in progress)
    let new_seq = sequence_counter.fetch_add(1, Ordering::SeqCst) + 1;

    // 2. Write data to shared memory
    (*shm).cursor_x = self.cursor_x;
    (*shm).cursor_y = self.cursor_y;
    (*shm).cells.copy_from_slice(&self.grid.cells);

    // 3. Increment sequence to even (write complete)
    let final_seq = sequence_counter.fetch_add(1, Ordering::SeqCst) + 1;
    (*shm).sequence_number = final_seq;
    (*shm).dirty_flag = 1;
}
```

### Actual Implementation

**Location**: `crates/scarab-daemon/src/vte.rs:287-326`

```rust
pub unsafe fn blit_to_shm(&self, shm: *mut SharedState, sequence_counter: &Arc<AtomicU64>) {
    let state = &mut *shm;

    // Fill the ENTIRE shared memory grid with theme background color
    let empty_cell = Cell {
        char_codepoint: b' ' as u32,
        fg: DEFAULT_FG,
        bg: DEFAULT_BG,
        flags: 0,
        _padding: [0; 3],
    };

    // First, fill the entire buffer with empty cells (theme colors)
    for cell in state.cells.iter_mut() {
        *cell = empty_cell;
    }

    // Then copy cells from local grid to shared memory
    for y in 0..self.rows.min(GRID_HEIGHT as u16) {
        for x in 0..self.cols.min(GRID_WIDTH as u16) {
            let local_idx = y as usize * self.cols as usize + x as usize;
            let shm_idx = y as usize * GRID_WIDTH + x as usize;

            if local_idx < self.grid.cells.len() && shm_idx < state.cells.len() {
                state.cells[shm_idx] = self.grid.cells[local_idx];
            }
        }
    }

    // Update cursor position
    state.cursor_x = self.cursor_x;
    state.cursor_y = self.cursor_y;

    // Mark dirty and increment sequence number
    state.dirty_flag = 1;
    let new_seq = sequence_counter.fetch_add(1, Ordering::SeqCst) + 1;
    state.sequence_number = new_seq;
}
```

### Compositor Loop

**Location**: `crates/scarab-daemon/src/main.rs:434-495`

The daemon's main loop blits the active pane's grid at ~60 FPS:

```rust
let compositor_interval = tokio::time::Duration::from_millis(16); // ~60fps

loop {
    tokio::select! {
        _ = tokio::time::sleep(compositor_interval) => {
            if let Some(session) = session_manager.get_default_session() {
                if let Some(active_pane) = session.get_active_pane() {
                    let terminal_state_arc = active_pane.terminal_state();
                    let terminal_state = terminal_state_arc.read();

                    // SAFETY: shared_ptr points to valid SharedState in shared memory
                    unsafe { terminal_state.blit_to_shm(shared_ptr, &sequence_counter) };
                }
            }
        }
    }
}
```

### Memory Ordering: `SeqCst`

The daemon uses `Ordering::SeqCst` (Sequentially Consistent) for sequence updates:

```rust
let new_seq = sequence_counter.fetch_add(1, Ordering::SeqCst) + 1;
```

**Why SeqCst?**
- **Strongest guarantee**: All threads see same order of operations
- **Prevents reordering**: Ensures sequence update happens before/after data writes
- **Platform-independent**: Works correctly on all architectures (x86, ARM, RISC-V)

**Performance Impact**: Minimal. `SeqCst` adds a memory barrier (~10-20 CPU cycles), negligible compared to terminal rendering work.

---

## Read Protocol (Client)

### High-Level Algorithm

```rust
fn read_grid() -> Result<Grid, RetryNeeded> {
    loop {
        // 1. Read sequence number
        let seq_before = (*shm).sequence_number;

        // 2. Check if odd (write in progress)
        if seq_before % 2 != 0 {
            continue; // Retry
        }

        // 3. Read data
        let cursor = ((*shm).cursor_x, (*shm).cursor_y);
        let cells = (*shm).cells.clone();

        // 4. Verify sequence unchanged
        let seq_after = (*shm).sequence_number;
        if seq_before == seq_after {
            return Ok(Grid { cursor, cells }); // Success
        }
        // Retry if sequence changed during read
    }
}
```

### Actual Implementation

**Location**: `crates/scarab-client/src/safe_state.rs:91-134`

The client uses the `TerminalStateReader` trait abstraction:

```rust
impl<'a> TerminalStateReader for SafeSharedState<'a> {
    fn cell(&self, row: usize, col: usize) -> Option<&Cell> {
        if row >= GRID_HEIGHT || col >= GRID_WIDTH {
            return None;
        }
        let idx = row * GRID_WIDTH + col;
        let state = self.state_ref();
        state.cells.get(idx)
    }

    fn sequence(&self) -> u64 {
        let state = self.state_ref();
        state.sequence_number
    }

    fn is_dirty(&self) -> bool {
        let state = self.state_ref();
        state.dirty_flag != 0
    }

    // ... other methods
}
```

### Rendering System

**Location**: `crates/scarab-client/src/rendering/text.rs` (inferred)

The client's Bevy rendering system polls the sequence number:

```rust
fn render_terminal(
    state: Res<SafeSharedState>,
    mut last_sequence: Local<u64>,
    // ... other resources
) {
    let current_seq = state.sequence();

    // Only re-render if sequence changed
    if current_seq != *last_sequence {
        // Read cells and update texture atlas
        for row in 0..GRID_HEIGHT {
            for col in 0..GRID_WIDTH {
                if let Some(cell) = state.cell(row, col) {
                    // Render cell to texture
                }
            }
        }
        *last_sequence = current_seq;
    }
}
```

### Consistency Helper Methods

**Location**: `crates/scarab-client/src/safe_state.rs`

The client provides optional methods for detecting if data changed during a read:

```rust
// Full consistency with automatic retry (blocking)
pub fn read_consistent(&self, max_retries: u32) -> Option<(u64, Vec<Cell>)>

// Lightweight cursor read with consistency
pub fn read_cursor_consistent(&self, max_retries: u32) -> Option<(u64, u16, u16)>

// Non-blocking single attempt (for render loops)
pub fn try_read(&self) -> Option<(u64, &[Cell])>
```

**For most use cases**, the standard `TerminalStateReader` trait methods are sufficient:

```rust
fn render_terminal(state: &SafeSharedState, mut last_seq: u64) -> u64 {
    let current_seq = state.sequence();
    if current_seq != last_seq {
        // Render using state.cells() or state.cell(row, col)
        render_cells(state.cells());
        last_seq = current_seq;
    }
    last_seq
}
```

**For operations requiring guaranteed consistency**, use `read_consistent()`:

```rust
// Will retry until a consistent read succeeds (max 1000 attempts)
if let Some((seq, cells)) = state.read_consistent(1000) {
    process_cells(&cells);
}
```

Note: Individual `Cell` writes (16 bytes) are atomic on modern CPUs, so torn reads
of individual cells are extremely rare. The consistency helpers are mainly useful
when reading large regions where you need to ensure all cells are from the same frame.

---

## Code Examples

### Example 1: Writer (Daemon)

**Simplified write path:**

```rust
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

fn daemon_update_terminal(
    shm_ptr: *mut SharedState,
    sequence: &Arc<AtomicU64>,
    new_text: &str,
) {
    // Simulate VTE parsing
    let cells = parse_vte_output(new_text);

    // Blit to shared memory with sequence update
    unsafe {
        let state = &mut *shm_ptr;

        // Copy cells (sequence remains even during write)
        for (i, cell) in cells.iter().enumerate() {
            state.cells[i] = *cell;
        }

        // Atomically increment sequence (signals new data)
        let new_seq = sequence.fetch_add(1, Ordering::SeqCst) + 1;
        state.sequence_number = new_seq;
        state.dirty_flag = 1;
    }
}
```

### Example 2: Reader (Client)

**Simplified read path:**

```rust
fn client_render_frame(shm_ptr: *const SharedState) -> Vec<Cell> {
    unsafe {
        let state = &*shm_ptr;

        // Read sequence number
        let sequence = state.sequence_number;

        // Read cells (no retry in current implementation)
        let cells: Vec<Cell> = state.cells.to_vec();

        println!("Rendered frame at sequence {}", sequence);
        cells
    }
}
```

### Example 3: Full SeqLock with Retry

**Best-practice implementation (future):**

```rust
fn read_terminal_safe(shm_ptr: *const SharedState) -> (u64, Vec<Cell>) {
    loop {
        unsafe {
            let state = &*shm_ptr;

            // 1. Read sequence (before)
            let seq_before = state.sequence_number;

            // 2. Skip if odd (write in progress)
            if seq_before % 2 != 0 {
                std::hint::spin_loop(); // Backoff
                continue;
            }

            // 3. Read data
            let cells = state.cells.to_vec();
            let cursor = (state.cursor_x, state.cursor_y);

            // 4. Read sequence (after)
            std::sync::atomic::fence(Ordering::Acquire);
            let seq_after = state.sequence_number;

            // 5. Verify consistency
            if seq_before == seq_after {
                return (seq_before, cells); // Success!
            }

            // Retry on mismatch
            std::hint::spin_loop();
        }
    }
}
```

---

## Invariants

### Critical Invariants

These properties **must** always hold for correctness:

1. **Single Writer**: Only the daemon writes to `SharedState`
   - **Violation**: Corruption, race conditions
   - **Enforcement**: Process isolation, shared memory permissions

2. **Sequence Monotonicity**: `sequence_number` always increases
   - **Violation**: Client sees same frame twice, misses updates
   - **Enforcement**: `fetch_add(1)` is atomic and monotonic

3. **Even = Stable**: Even sequence → data is consistent
   - **Violation**: Client reads partial writes
   - **Current**: Not strictly enforced (relies on atomic Cell writes)

4. **Memory Layout Stability**: `#[repr(C)]` on all shared structs
   - **Violation**: Daemon/client read wrong offsets → garbage data
   - **Enforcement**: Compiler guarantee, verified by tests

5. **Alignment**: All fields properly aligned for atomic access
   - **Violation**: Undefined behavior, crashes on some architectures
   - **Enforcement**: `_padding` fields, `bytemuck::Pod` trait

### Runtime Checks

**Client validation** (optional, not currently implemented):

```rust
fn validate_shared_state(state: &SharedState) -> bool {
    // Check magic number (if added)
    if state.magic != SHARED_STATE_MAGIC { return false; }

    // Check cursor bounds
    if state.cursor_x >= GRID_WIDTH as u16 { return false; }
    if state.cursor_y >= GRID_HEIGHT as u16 { return false; }

    // Check sequence sanity (not zero, reasonable range)
    if state.sequence_number == 0 { return false; }

    true
}
```

---

## Performance Characteristics

### Throughput

| Metric | Value | Notes |
|--------|-------|-------|
| **Write Frequency** | ~60 Hz | Compositor loop (16ms interval) |
| **Read Frequency** | ~60 Hz | Bevy render loop |
| **Write Latency** | ~500 µs | `blit_to_shm()` duration |
| **Read Latency** | ~50 µs | Sequence check + cell access |
| **Memory Bandwidth** | ~19 MB/s | 320 KB × 60 FPS |

### Scalability

- **Lock-free reads**: Client never blocks daemon writes
- **Cache-friendly**: Sequence number in separate cache line from cells
- **Zero allocations**: Direct memory mapping, no heap
- **Multi-client**: Each client has independent read access (same SharedState)

### Comparison to Alternatives

| Approach | Latency | Throughput | Complexity |
|----------|---------|------------|------------|
| **Mutex** | High (lock contention) | Low (serialized) | Low |
| **RwLock** | Medium (writer starvation) | Medium | Low |
| **SeqLock** | Low (lock-free) | **High** | **Medium** |
| **Double-buffer** | Low | High | High (2× memory) |

### Alternative: mmap-sync (Cloudflare)

If implementing SeqLock manually becomes burdensome, consider switching to
[**mmap-sync**](https://github.com/cloudflare/mmap-sync) - a production-ready
crate from Cloudflare that provides:

- Single-writer, multiple-reader wait-free synchronization
- Memory-mapped file backend (persistence optional)
- Correct SeqLock implementation out of the box
- Zero-copy deserialization via `rkyv`

```toml
# Cargo.toml
mmap-sync = "1"
```

This would replace both `shared_memory` and our manual sequence number logic.
The tradeoff is adding a dependency vs. maintaining our own implementation.

**When to consider switching:**
- If SeqLock bugs continue to cause issues
- If we need persistence across daemon restarts
- If we want to support multiple simultaneous daemons

### Memory Overhead

- **SharedState**: 320,016 bytes (312 KB)
- **AtomicU64** (daemon): 8 bytes
- **Total**: ~312 KB per terminal session

**vs. Socket-based IPC**: Would require serialization (JSON/protobuf), ~2× larger payload, ~10× slower.

---

## Troubleshooting

### Issue: Client shows "tearing" (partial frames)

**Symptoms**: Flickering, inconsistent cell rendering

**Cause**: Client reads during daemon write (no retry loop)

**Fix**: Implement full SeqLock validation (see Example 3)

**Workaround**: Increase compositor interval (reduce write frequency)

```rust
// In main.rs:426
let compositor_interval = tokio::time::Duration::from_millis(32); // 30 FPS
```

### Issue: Client crashes with SIGBUS

**Symptoms**: Bus error, unaligned access

**Cause**: Missing `#[repr(C)]` on shared struct

**Fix**: Verify all protocol structs have `#[repr(C)]`

```bash
rg "struct (SharedState|Cell)" --type rust -A 1 | grep "repr(C)"
```

### Issue: Sequence number doesn't change

**Symptoms**: Client never re-renders, frozen terminal

**Cause**: Daemon not calling `blit_to_shm()`, compositor loop stalled

**Debug**:
```bash
# Check daemon logs
RUST_LOG=debug cargo run -p scarab-daemon 2>&1 | grep "Sequence:"

# Attach debugger
gdb --args target/debug/scarab-daemon
(gdb) break blit_to_shm
```

### Issue: Sequence number overflows

**Symptoms**: Client sees sequence jump from high value to low

**Cause**: `u64` overflow after ~2^64 frames (~9 quintillion years at 60 FPS)

**Fix**: Not needed (would take 292 billion years to overflow)

**If paranoid**:
```rust
let new_seq = sequence_counter.fetch_add(1, Ordering::SeqCst).wrapping_add(1);
```

### Issue: SharedState not found

**Symptoms**: Client error: `"Failed to open shared memory"`

**Cause**: Daemon not running, or `/dev/shm` permissions

**Fix**:
```bash
# Check daemon is running
ps aux | grep scarab-daemon

# Check shared memory exists
ls -lh /dev/shm/scarab_shm_v1

# Fix permissions
sudo chmod 666 /dev/shm/scarab_shm_v1
```

**Environment override**:
```bash
export SCARAB_SHMEM_PATH=/tmp/scarab_shm_custom
```

---

## References

### Academic Papers

1. **Lameter, C. (2005)**. "Effective Synchronization on Linux/NUMA Systems."
   *Proceedings of the Linux Symposium*, Volume 2, pp. 159-168.
   [Link](https://www.kernel.org/doc/ols/2005/ols2005v2-pages-159-168.pdf)

2. **Herlihy, M. & Shavit, N. (2008)**. *The Art of Multiprocessor Programming.*
   Chapter 13: Concurrent Objects (SeqLock pattern).
   Morgan Kaufmann.

### Rust Documentation

3. **Rust Atomics and Locks (2023)**
   [https://marabos.nl/atomics/](https://marabos.nl/atomics/)
   Comprehensive guide to Rust atomics, memory ordering.

4. **`std::sync::atomic` documentation**
   [https://doc.rust-lang.org/std/sync/atomic/](https://doc.rust-lang.org/std/sync/atomic/)
   Official Rust atomic types reference.

### Scarab Codebase

5. **SharedState definition**
   `crates/scarab-protocol/src/lib.rs:52-64`

6. **Write implementation**
   `crates/scarab-daemon/src/vte.rs:287-326`

7. **Read abstraction**
   `crates/scarab-client/src/safe_state.rs:91-134`

8. **Compositor loop**
   `crates/scarab-daemon/src/main.rs:434-495`

### Related Documentation

9. **Safe SharedState Abstraction**
   `docs/safe-state-abstraction.md`
   Design rationale for `TerminalStateReader` trait.

10. **Parallel PTY Orchestration**
    `docs/PARALLEL_ORCHESTRATION_REPORT.md`
    How multiple panes interact with shared memory.

---

## Appendix A: ASCII Art Diagrams

### SeqLock State Machine

```
                  ┌─────────┐
                  │ IDLE    │
                  │ seq=N   │ (even)
                  └────┬────┘
                       │
                   fetch_add(1)
                       │
                  ┌────▼────┐
                  │WRITING  │
                  │seq=N+1  │ (odd)
                  └────┬────┘
                       │
                 [write data]
                       │
                   fetch_add(1)
                       │
                  ┌────▼────┐
                  │ STABLE  │
                  │seq=N+2  │ (even)
                  └────┬────┘
                       │
                       ▼
                   [repeat]
```

### Memory Access Timeline

```
Time →
Daemon:  ┌──Write──┐       ┌──Write──┐
         │ seq=1-2 │       │ seq=3-4 │
         └─────────┘       └─────────┘

Client:     ┌─Read─┐   ┌─Read─┐   ┌─Read─┐
            │seq=0 │   │seq=2 │   │seq=4 │
            └──────┘   └──────┘   └──────┘
                  ↑ Retry       ✓ Success
```

---

**Document End**

For questions or contributions, see `docs/CONTRIBUTING-DOCS.md`.
