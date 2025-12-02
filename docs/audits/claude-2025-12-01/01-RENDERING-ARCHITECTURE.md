# Rendering Architecture Analysis
**Date:** December 1, 2025
**Focus:** Scarab Client Rendering Pipeline

---

## Executive Summary

The Scarab client rendering pipeline is **well-architected at the feature level** (modular plugins, clean separation) but **tightly coupled to Bevy at the implementation level**, making it **difficult to test without GPU context**.

**Grade: C for Testability**

---

## Architecture Overview

### Rendering Pipeline Flow

```
┌─────────────────────────────────────────────────────────────┐
│                     DAEMON (PTY)                             │
│                                                              │
│  ┌────────────────────────────────────────────────────┐    │
│  │  VTE Parser → SharedState.cells[]                  │    │
│  │  (200x100 grid, 20,000 cells)                      │    │
│  │  AtomicU64::sequence_number++                      │    │
│  └────────────────────────────────────────────────────┘    │
└─────────────────────────────────────────────────────────────┘
                           │
                           │ Shared Memory (Lock-Free)
                           │
                           ▼
┌─────────────────────────────────────────────────────────────┐
│                  CLIENT (Bevy ECS)                           │
│                                                              │
│  ┌────────────────────────────────────────────────────┐    │
│  │  STAGE 1: Sync System                              │    │
│  │  - Poll SharedState.sequence_number                │    │
│  │  - Detect changes (atomic load)                    │    │
│  │  - Mark TerminalMesh dirty                         │    │
│  └────────────────────────────────────────────────────┘    │
│                           │
│                           ▼
│  ┌────────────────────────────────────────────────────┐    │
│  │  STAGE 2: Mesh Generation                          │    │
│  │  - generate_terminal_mesh(&SharedState)            │    │
│  │  - For each cell:                                  │    │
│  │    • Rasterize glyph via cosmic-text               │    │
│  │    • Cache in 4096x4096 texture atlas              │    │
│  │    • Generate quad (background + glyph + decorations) │
│  │  - Returns: Bevy Mesh (positions, UVs, colors, indices) │
│  └────────────────────────────────────────────────────┘    │
│                           │
│                           ▼
│  ┌────────────────────────────────────────────────────┐    │
│  │  STAGE 3: GPU Rendering                            │    │
│  │  - Bevy 2D Renderer                                │    │
│  │  - Mesh2d + ColorMaterial                          │    │
│  │  - Orthographic camera                             │    │
│  └────────────────────────────────────────────────────┘    │
└─────────────────────────────────────────────────────────────┘
```

---

## Module Breakdown

### Core Rendering (crates/scarab-client/src/rendering/)

**rendering/text.rs** (576 lines)
- `TextRenderer` resource (fonts, atlas, metrics)
- `generate_terminal_mesh()` - Main mesh builder
- `DirtyRegion` tracking (currently unused - always full redraw)
- `update_terminal_mesh_system()` - Bevy system

**rendering/atlas.rs** (337 lines)
- `GlyphAtlas` - 4096x4096 RGBA8 texture atlas
- Simple row-based packing
- `GlyphKey → AtlasRect` HashMap cache
- **Issue:** No dynamic expansion (fixed size)

**rendering/config.rs** (158 lines)
- `FontConfig` - Font family, size, metrics
- Color utilities (ANSI → RGBA conversion)
- `TextAttributes` - Bold, italic, underline

**rendering/scrollback_render.rs** (294 lines)
- Separate mesh generation for scrollback view
- Similar to main grid but reads from VecDeque history

---

### UI Components (crates/scarab-client/src/ui/)

**Total:** 12 UI plugins, ~4,500 lines

**Key Components:**
- `command_palette.rs` - Fuzzy search, command registry
- `link_hints.rs` - Vimium-style link detection
- `overlays.rs` - Remote overlays from daemon
- `visual_selection.rs` - Text selection (character/line/block)
- `scroll_indicator.rs` - Scrollback position indicator
- `search_overlay.rs` - In-terminal search
- `dock.rs` - Plugin sidebar
- `animations.rs` - Fade in/out with easing
- `keybindings.rs` - Custom key mapping
- `leader_key.rs` - Vim-style leader sequences

**Rendering Pattern:**
```rust
// Most UI uses direct Bevy spawn:
commands.spawn((
    Text2d::new("Hello"),
    Transform::from_xyz(x, y, 0.0),
    // ...
));
```

**Issue:** No intermediate representation for testing

---

## Critical Coupling Points

### 1. Mesh Generation Requires Bevy Assets

**Problem:**
```rust
pub fn generate_terminal_mesh(
    state: &SharedState,
    renderer: &mut TextRenderer,
    dirty_region: &DirtyRegion,
    images: &mut ResMut<Assets<Image>>, // ← Bevy ECS resource!
) -> Mesh {
    // ...
    renderer.atlas.get_or_cache(glyph_key, images)
    // ...
}
```

**Cannot call without:**
- Bevy `World` (ECS)
- `Assets<Image>` (asset system)
- GPU texture allocation

**Impact:** Cannot unit test mesh generation

---

### 2. TextRenderer Couples Font + Atlas + GPU

**Problem:**
```rust
pub struct TextRenderer {
    pub font_system: FontSystem,   // cosmic-text
    pub swash_cache: SwashCache,   // glyph rasterizer
    pub atlas: GlyphAtlas,          // GPU texture + HashMap
    pub config: FontConfig,
    pub cell_width: f32,
    pub cell_height: f32,
}
```

**Issues:**
- Atlas stores `Handle<Image>` (requires Bevy)
- Cannot test font metrics without GPU context
- Mixed concerns (fonts + caching + GPU)

---

### 3. Unsafe SharedState Access

**Pattern appears in multiple systems:**
```rust
unsafe {
    let state = &*(shmem.as_ptr() as *const SharedState);
    // Use state...
}
```

**Issues:**
- No bounds checking
- Potential UB if daemon unmaps memory
- No abstraction for safe access
- Difficult to mock for testing

---

### 4. UI Rendering No Abstraction

**Pattern:**
```rust
// Command Palette
commands.spawn((
    Node {
        position_type: PositionType::Absolute,
        width: Val::Px(600.0),
        // ...
    },
    BackgroundColor(Color::srgba(0.1, 0.1, 0.1, 0.95)),
));

// Link Hints
commands.spawn((
    Text2d::new(hint_text),
    Transform::from_xyz(x, y, 100.0),
));
```

**Issue:** Direct Bevy types = hard to test layout logic

---

## Testability Analysis

### ✅ **What CAN Be Tested (Good)**

1. **Business Logic** (~487 tests exist)
   - Link detection regex
   - Fuzzy search algorithms
   - Keybinding parsing
   - Animation math
   - Selection geometry
   - Grid coordinate conversion

2. **Integration (E2E with harness)**
   - Daemon-client IPC
   - Input forwarding
   - Session persistence

---

### ❌ **What CANNOT Be Tested (Critical Gap)**

1. **Mesh Generation**
   - Requires Bevy `Assets<Image>`
   - Requires GPU context
   - Cannot verify quad generation
   - Cannot verify atlas packing

2. **UI Layout**
   - Command palette positioning
   - Overlay bounds checking
   - Link hint placement
   - Scroll indicator visibility

3. **Rendering Correctness**
   - Glyph atlas caching
   - Color conversion
   - Underline/strikethrough placement

4. **Integration**
   - SharedState → Mesh flow
   - Dirty region optimization
   - Atlas overflow handling

---

## Architectural Issues

### 🔴 **Critical**

1. **No Headless Rendering Mode**
   - Cannot run rendering without window
   - Cannot capture mesh data for assertions
   - Cannot snapshot test UI layouts

2. **No Rendering Trait Abstraction**
   - Vendor lock-in to Bevy
   - Cannot swap backends
   - Cannot mock for tests

3. **Unsafe Shared Memory Pattern**
   - Multiple systems use raw pointers
   - No safety guarantees
   - Hard to test safely

### 🟡 **High Priority**

4. **Dirty Region Tracking Disabled**
   ```rust
   // Always does full redraw!
   terminal_mesh.dirty_region.mark_full_redraw();
   ```
   - Infrastructure exists but unused
   - Performance cost
   - Should be enabled

5. **Fixed Atlas Size**
   ```rust
   warn!("Atlas full! Consider implementing dynamic atlas expansion");
   return None; // Glyph dropped!
   ```
   - 4096x4096 can overflow
   - No fallback behavior
   - Emojis/CJK may not render

6. **UI Rendering Couples Logic + Presentation**
   - No separation of "what to render" vs "how to render"
   - Cannot test layout without spawning entities
   - Difficult to add new backends

---

## Recommendations

### 🎯 **Option 1: Headless Bevy Test Harness** (FAST)

**Create:**
```rust
// tests/headless_harness.rs
pub struct HeadlessTestHarness {
    app: App,
    mock_assets: MockAssets,
}

impl HeadlessTestHarness {
    pub fn new() -> Self {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins); // No window!

        // Mock Assets<Image>
        let mock_assets = MockAssets::new();
        app.insert_resource(mock_assets.clone());

        Self { app, mock_assets }
    }

    pub fn update(&mut self) {
        self.app.update();
    }

    pub fn query<T: Component>(&self) -> Vec<&T> {
        // Query ECS for components
    }

    pub fn assert_mesh_data(&self, expected: &MeshData) {
        // Extract and compare mesh
    }
}
```

**Usage:**
```rust
#[test]
fn test_command_palette_renders() {
    let mut harness = HeadlessTestHarness::new();

    // Trigger command palette
    harness.send_event(OpenCommandPalette);
    harness.update();

    // Assert: Modal exists with correct size
    let nodes = harness.query::<Node>();
    assert!(nodes.iter().any(|n| n.width == Val::Px(600.0)));
}
```

**Pros:**
- Minimal architecture changes
- Uses real Bevy systems
- Can test actual logic

**Cons:**
- Requires mocking Assets
- May hit Bevy limitations

**Effort:** 2 weeks

---

### 🎯 **Option 2: Rendering Abstraction Layer** (ARCHITECTURAL)

**Extract pure data:**
```rust
// Pure data structure
pub struct MeshData {
    pub positions: Vec<[f32; 3]>,
    pub uvs: Vec<[f32; 2]>,
    pub colors: Vec<[f32; 4]>,
    pub indices: Vec<u32>,
}

// Pure function (testable!)
pub fn generate_mesh_data(
    state: &SharedState,
    renderer: &TextRenderer,
) -> MeshData {
    let mut data = MeshData::default();

    for (idx, cell) in state.cells.iter().enumerate() {
        // Generate quad data
        data.positions.extend_from_slice(&quad_positions);
        data.uvs.extend_from_slice(&quad_uvs);
        // ...
    }

    data
}

// Bevy adapter
impl From<MeshData> for Mesh {
    fn from(data: MeshData) -> Mesh {
        Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default())
            .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, data.positions)
            .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, data.uvs)
            // ...
    }
}
```

**Testing:**
```rust
#[test]
fn test_mesh_generation() {
    let state = create_test_state();
    let renderer = create_test_renderer();

    let mesh = generate_mesh_data(&state, &renderer);

    assert_eq!(mesh.positions.len(), expected_vertices);
    assert!(mesh.uvs.iter().all(|uv| uv[0] >= 0.0 && uv[0] <= 1.0));
}
```

**Pros:**
- Clean architecture
- Easy to test
- Future-proof for other backends

**Cons:**
- Large refactoring
- Needs careful design
- May impact performance

**Effort:** 3-4 weeks

---

### 🎯 **Option 3: Snapshot Testing** (HYBRID)

**Serialize mesh to JSON:**
```rust
#[derive(Serialize)]
struct MeshSnapshot {
    vertex_count: usize,
    positions: Vec<[f32; 3]>,
    colors_sample: Vec<[f32; 4]>, // First 10
    bounding_box: BoundingBox,
}

#[test]
fn test_render_hello_world() {
    let harness = setup_harness();
    harness.render_text("Hello, World!");

    let snapshot = harness.capture_mesh_snapshot();
    insta::assert_json_snapshot!(snapshot);
}
```

**Pros:**
- Catches regressions
- Visual verification
- Easy to review changes

**Cons:**
- Doesn't test logic
- Still needs harness
- Brittle if mesh format changes

**Effort:** 1 week (after harness exists)

---

## Immediate Action Items

### Week 1-2: POC Headless Testing

1. **Create `tests/headless_poc.rs`**
   - Prove Bevy can run without window
   - Prove `Assets<Image>` can be mocked
   - Prove mesh data can be extracted

2. **Test Simple Mesh Generation**
   - Generate mesh for "Hello"
   - Capture vertex data
   - Assert vertex count/positions

3. **Test UI Component**
   - Spawn command palette
   - Query Node components
   - Assert width/height/visibility

**Success Criteria:**
- Tests run in CI without display
- Can assert on mesh data
- Can assert on UI layout

---

### Week 3-4: Build Harness

1. **Create `TestHarness` struct**
   - Wraps Bevy `App` with `MinimalPlugins`
   - Provides helper methods for common assertions
   - Handles mocking

2. **Add Utilities**
   - `query()` - Query ECS components
   - `assert_text_at()` - Verify grid contents
   - `assert_component_exists()` - UI presence
   - `capture_snapshot()` - For regression tests

3. **Write Tests for Features**
   - Command palette visibility
   - Link hints positioning
   - Overlay bounds
   - Visual selection rendering

**Success Criteria:**
- 10+ rendering tests passing
- UI components testable
- Tests run in < 5 seconds

---

## Long-Term Vision

### Ideal Architecture

```rust
// Domain: Pure business logic
mod domain {
    pub struct TerminalGrid { /* ... */ }
    pub struct RenderCommand { /* ... */ }

    pub fn compute_render_commands(grid: &TerminalGrid) -> Vec<RenderCommand> {
        // Pure function!
    }
}

// Adapters: Framework-specific
mod adapters {
    pub mod bevy {
        pub fn execute_render_commands(cmds: &[RenderCommand], world: &mut World) {
            // Bevy-specific
        }
    }

    pub mod test {
        pub fn execute_render_commands(cmds: &[RenderCommand]) -> TestOutput {
            // Testing mock
        }
    }
}

// Tests
#[test]
fn test_rendering() {
    let grid = create_test_grid();
    let commands = domain::compute_render_commands(&grid);
    let output = adapters::test::execute_render_commands(&commands);

    assert_eq!(output.text_at(0, 0), "H");
}
```

**Benefits:**
- Domain logic is pure (easily testable)
- Bevy is an implementation detail
- Can add new backends (WASM, headless, etc.)

---

## Conclusion

The rendering pipeline is **well-structured at the feature level** but needs **architectural improvements for testability**.

**Priority actions:**
1. ✅ Prove headless testing is viable (Week 1-2)
2. ✅ Build test harness (Week 3-4)
3. ⏸️ Consider abstraction layer (Long-term)

**Expected outcome:** Frontend testing loop closed in 4-6 weeks

---

**Document:** 01-RENDERING-ARCHITECTURE.md
**Next:** 02-TESTING-INFRASTRUCTURE.md (Current test coverage analysis)
