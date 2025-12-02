# Implementation Guide: VTE Decoupling
**Target**: `crates/scarab-daemon/src/vte.rs`

## Objective
Modify `TerminalState` to support off-screen rendering for multiple panes.

## Changes

### 1. Abstract the Grid
Define a `Grid` trait or simple struct that holds `Vec<Cell>`.

```rust
pub struct Grid {
    pub cells: Vec<Cell>,
    pub cols: u16,
    pub rows: u16,
}

impl Grid {
    pub fn new(cols: u16, rows: u16) -> Self { ... }
    pub fn resize(&mut self, cols: u16, rows: u16) { ... }
    pub fn clear(&mut self) { ... }
    // ...
}
```

### 2. Update TerminalState
Remove `shared_ptr`. Add `grid: Grid`.

```rust
pub struct TerminalState {
    pub grid: Grid, // Owns its data now!
    // ...
}
```

### 3. Implement "Blit" (Copy to Shared Memory)
Add a method to copy the local grid to the shared memory buffer.

```rust
impl TerminalState {
    pub fn blit_to_shm(&self, shm: *mut SharedState) {
        unsafe {
            let state = &mut *shm;
            // Copy cells
            // Update cursor pos
            // ...
        }
    }
}
```

### 4. Integration
Update `Pane` to own `TerminalState`.
Update `main.rs` to drive this logic.
