# Implementation Plan: Bridging the Gap
**Date:** December 2, 2025
**Target**: Architecture Refactor for Multiplexing & Images

## Phase 1: Multiplexing Architecture (The "Tab" Model)

**Objective**: Transform `Scarab` from a single-PTY session model to a Tree-of-Panes model.

### 1. Data Model Changes (`scarab-daemon`)
Modify `crates/scarab-daemon/src/session/manager.rs`:

```rust
// Current
pub struct Session {
    pub pty_master: Arc<Mutex<Option<Box<dyn MasterPty>>>>,
    pub grid_state: Arc<RwLock<GridState>>,
    // ...
}

// Proposed
pub struct Pane {
    pub id: PaneId,
    pub pty_master: Arc<Mutex<Option<Box<dyn MasterPty>>>>,
    pub grid_state: Arc<RwLock<GridState>>,
    pub viewport: Rect, // Position in the layout
}

pub struct Tab {
    pub id: TabId,
    pub title: String,
    pub root_pane: PaneId, // Or a Tree structure defining splits
    pub active_pane: PaneId,
}

pub struct Session {
    pub id: SessionId,
    pub tabs: HashMap<TabId, Tab>,
    pub active_tab: TabId,
    pub panes: HashMap<PaneId, Arc<Pane>>, // Flattened ownership
    // ...
}
```

### 2. IPC / Protocol Changes (`scarab-protocol`)
The Shared Memory layout currently assumes one grid.
*   **Challenge**: How to render multiple panes efficiently?
*   **Option A (Compositing in Daemon)**: Daemon renders all panes into a single "Virtual Grid" that represents the window. Client just renders that grid.
    *   *Pros*: Simple for Client.
    *   *Cons*: Resizing is complex; Mouse coordinate translation must happen in Daemon.
*   **Option B (Compositing in Client)**: Daemon exposes state for *all* visible panes.
    *   *Pros*: Client handles UI logic (tabs, resizing borders).
    *   *Cons*: Complex IPC.

**Recommendation**: **Option A (Daemon Compositing)** for MVP.
Let the Daemon manage the logical "Screen". When a split happens, the Daemon manages two PTYs but renders them onto the single 2D grid array in Shared Memory (using `copy_from` operations). This keeps the Client dumb and fast.

### 3. Implementation Tasks
1.  Define `Pane` struct.
2.  Refactor `Session` to hold `Vec<Pane>` (start with simple tiling or tab list).
3.  Implement "Active Pane" logic. Input is routed only to the Active Pane's PTY.
4.  Implement `render_to_grid()`: A function that takes all visible panes and composites them into the master `GridState`.

## Phase 2: Image Rendering Pipeline

**Objective**: End-to-end display of images.

### 1. Shared Memory Extension
We need a "Blob Store" in shared memory for image data.
1.  Reserve a `ConstSize` region (e.g., 16MB) in SHM for "Asset Data".
2.  Or create a secondary SHM segment for heavy assets.

### 2. Grid Protocol Extension
Add a new `Cell` flag or a separate "Overlay Layer".
*   **Overlay Approach**: The IPC struct should include a list of `ImagePlacement` structs.
    ```rust
    #[repr(C)]
    pub struct ImagePlacement {
        pub image_id: u64, // Hash or Offset in Blob Store
        pub x: u16,
        pub y: u16,
        pub width: u16,
        pub height: u16,
    }
    ```

### 3. Client Rendering
1.  Read `ImagePlacement` list.
2.  Retrieve image data from Blob Store.
3.  Load into `bevy::render::texture::Image`.
4.  Spawn a Bevy `SpriteBundle` at the correct coordinates over the terminal text.

## Phase 3: Instructions for Claude CLI

To continue development, execute the following work packages:

### Work Package A: Session Refactor
1.  Create `crates/scarab-daemon/src/session/pane.rs`.
2.  Move `pty_master` and `grid_state` from `Session` to `Pane`.
3.  Update `Session` to manage a list of `Pane`s.
4.  Fix compilation errors in `main.rs` (input routing).

### Work Package B: Image Transport
1.  In `scarab-protocol`, define `SharedImageBuffer` struct.
2.  In `scarab-daemon`, when `parse_iterm2_image` succeeds, write data to `SharedImageBuffer`.
3.  In `scarab-client`, add system to poll `SharedImageBuffer` and update Bevy assets.
