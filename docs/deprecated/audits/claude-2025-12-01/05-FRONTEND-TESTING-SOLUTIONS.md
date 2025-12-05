# Frontend Testing Solutions - Detailed Proposals
**Date:** December 1, 2025
**Goal:** Enable automated verification of Scarab client UI without manual testing

---

## The Problem Statement

**Current Reality:**
```
Developer adds tab bar to bottom of window
    ↓
Must manually run: cargo run -p scarab-client
    ↓
Visual inspection: "Does it look right?"
    ↓
No automated verification possible
```

**Desired State:**
```
Developer adds tab bar
    ↓
Writes test:
    test.assert_component_visible::<TabBar>();
    test.assert_position(TabBar, bottom, height=2);
    ↓
cargo test --all
    ↓
PASS (or FAIL with clear error)
```

---

## Solution 1: Headless Bevy Test Harness ⭐ RECOMMENDED

### Overview
Run Bevy in headless mode (no window) and extract ECS state for assertions.

### Implementation

**Phase 1: Proof of Concept (Week 1)**

```rust
// crates/scarab-client/tests/headless_poc.rs

use bevy::prelude::*;
use bevy::MinimalPlugins;

#[test]
fn poc_bevy_runs_headless() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);

    // Add minimal systems
    app.update();

    // Success if we get here without panic
    assert!(true);
}

#[test]
fn poc_query_components() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);

    // Spawn test entity
    app.world_mut().spawn((
        Node {
            width: Val::Px(100.0),
            height: Val::Px(50.0),
            ..default()
        },
        Name::new("TestNode"),
    ));

    app.update();

    // Query it back
    let mut query = app.world_mut().query::<(&Node, &Name)>();
    let (node, name) = query.single(app.world());

    assert_eq!(name.as_str(), "TestNode");
    assert_eq!(node.width, Val::Px(100.0));
}

#[test]
fn poc_mock_assets() {
    // Can we mock Assets<Image>?
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);

    // Register mock asset
    app.init_resource::<Assets<Image>>();

    let mut images = app.world_mut().resource_mut::<Assets<Image>>();
    let handle = images.add(Image::default());

    assert!(images.get(&handle).is_some());
}
```

**Success Criteria:**
- All tests pass in CI without X11/Wayland
- Can spawn components and query them
- Can mock asset system

---

**Phase 2: Test Harness (Week 2-3)**

```rust
// crates/scarab-client/tests/harness/mod.rs

use bevy::prelude::*;
use scarab_client::*;

pub struct HeadlessTestHarness {
    app: App,
}

impl HeadlessTestHarness {
    pub fn new() -> Self {
        let mut app = App::new();

        // Minimal plugins (no window)
        app.add_plugins(MinimalPlugins);

        // Add scarab plugins (except rendering)
        app.add_plugins((
            IntegrationPlugin,
            AdvancedUIPlugin, // UI systems
            TutorialPlugin,
        ));

        // Mock shared memory
        let mock_state = MockSharedState::new(200, 100);
        app.insert_resource(mock_state);

        // Mock assets
        app.init_resource::<Assets<Image>>();

        Self { app }
    }

    pub fn update(&mut self) {
        self.app.update();
    }

    pub fn send_event<E: Event>(&mut self, event: E) {
        self.app.world_mut().send_event(event);
        self.update();
    }

    pub fn query<T: Component>(&self) -> Vec<Entity> {
        self.app
            .world()
            .query_filtered::<Entity, With<T>>()
            .iter(self.app.world())
            .collect()
    }

    pub fn assert_component_exists<T: Component>(&self) {
        assert!(
            !self.query::<T>().is_empty(),
            "Component {} not found",
            std::any::type_name::<T>()
        );
    }

    pub fn assert_component_count<T: Component>(&self, expected: usize) {
        let count = self.query::<T>().len();
        assert_eq!(
            count, expected,
            "Expected {} components of type {}, found {}",
            expected,
            std::any::type_name::<T>(),
            count
        );
    }

    pub fn get_node_size<T: Component>(&self) -> Option<(f32, f32)> {
        let mut query = self.app.world().query::<(&Node, &T)>();
        query.iter(self.app.world()).next().map(|(node, _)| {
            let width = match node.width {
                Val::Px(px) => px,
                _ => 0.0,
            };
            let height = match node.height {
                Val::Px(px) => px,
                _ => 0.0,
            };
            (width, height)
        })
    }
}
```

**Phase 3: Real Tests (Week 3-4)**

```rust
// crates/scarab-client/tests/ui_rendering_tests.rs

mod harness;
use harness::HeadlessTestHarness;
use scarab_client::ui::*;

#[test]
fn test_command_palette_spawns_on_event() {
    let mut test = HeadlessTestHarness::new();

    // Trigger command palette
    test.send_event(ShowCommandPaletteEvent);

    // Assert: Palette UI exists
    test.assert_component_exists::<CommandPaletteMarker>();

    // Assert: Has expected size
    let (width, height) = test
        .get_node_size::<CommandPaletteMarker>()
        .expect("Palette should have Node");

    assert_eq!(width, 600.0);
    assert!(height > 0.0);
}

#[test]
fn test_link_hints_render_for_urls() {
    let mut test = HeadlessTestHarness::new();

    // Populate grid with URL
    test.set_grid_text(0, "Visit https://example.com");
    test.update();

    // Trigger link hints
    test.send_event(ActivateLinkHintsEvent);

    // Assert: Link hint entities exist
    test.assert_component_exists::<LinkHintMarker>();

    // Count hints
    test.assert_component_count::<LinkHintMarker>(1);
}

#[test]
fn test_overlay_renders_within_bounds() {
    let mut test = HeadlessTestHarness::new();

    // Trigger overlay
    test.send_event(RemoteMessageEvent::DrawOverlay {
        id: 1,
        x: 10,
        y: 5,
        text: "TEST",
        style: OverlayStyle::default(),
    });

    // Assert: Overlay exists
    test.assert_component_exists::<OverlayMarker>();

    // Query position
    let transform = test.get_transform::<OverlayMarker>().unwrap();
    assert!(transform.translation.x >= 0.0);
    assert!(transform.translation.y <= 0.0); // Y inverted
}

#[test]
fn test_tab_bar_at_bottom() {
    let mut test = HeadlessTestHarness::new();

    // Enable tabs
    test.spawn_tabs(&["Tab 1", "Tab 2", "Tab 3"]);
    test.update();

    // Assert: Tab bar component exists
    test.assert_component_exists::<TabBarMarker>();

    // Assert: Positioned at bottom
    let node = test.get_node::<TabBarMarker>().unwrap();
    assert_eq!(node.position_type, PositionType::Absolute);
    assert_eq!(node.bottom, Val::Px(0.0));
}
```

**Effort:** 3 weeks total
**Risk:** LOW (Bevy supports headless mode)
**ROI:** HIGH (enables all UI testing)

---

## Solution 2: Rendering Abstraction Layer

### Overview
Extract pure data structures from Bevy-specific rendering.

### Architecture

```rust
// Domain layer (pure, testable)
pub mod domain {
    pub struct RenderableGrid {
        pub cells: Vec<RenderableCell>,
        pub width: usize,
        pub height: usize,
    }

    pub struct RenderableCell {
        pub char: char,
        pub fg_color: [f32; 4],
        pub bg_color: [f32; 4],
        pub attrs: CellAttributes,
    }

    pub struct MeshData {
        pub quads: Vec<Quad>,
        pub glyphs: Vec<GlyphInstance>,
    }

    pub fn compute_mesh_data(grid: &RenderableGrid) -> MeshData {
        // Pure function!
        let mut data = MeshData::default();
        for (idx, cell) in grid.cells.iter().enumerate() {
            data.quads.push(compute_quad(idx, cell));
            if cell.char != ' ' {
                data.glyphs.push(compute_glyph(idx, cell));
            }
        }
        data
    }
}

// Bevy adapter
pub mod bevy_adapter {
    use super::domain::*;

    pub fn mesh_from_data(data: &MeshData) -> Mesh {
        let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, ...);
        for quad in &data.quads {
            mesh.insert_attribute(POSITION, quad.positions);
            mesh.insert_attribute(UV, quad.uvs);
        }
        mesh
    }
}

// Tests
#[test]
fn test_mesh_generation() {
    let grid = RenderableGrid::from_text("Hello");
    let mesh_data = domain::compute_mesh_data(&grid);

    assert_eq!(mesh_data.glyphs.len(), 5); // H, e, l, l, o
    assert!(mesh_data.quads.len() >= 5); // At least one per char
}
```

**Benefits:**
- Pure functions easy to test
- No Bevy dependency for logic
- Future-proof for other backends

**Drawbacks:**
- Large refactor
- Need to migrate existing code
- Performance implications?

**Effort:** 4-5 weeks
**Risk:** MEDIUM (architectural changes)
**ROI:** HIGH (long-term maintainability)

---

## Solution 3: Extend Ratatui-testlib

### Overview
Add Bevy integration to ratatui-testlib for Scarab's use case.

### Proposed API

```rust
// In ratatui-testlib
pub struct BevyTuiTestHarness {
    pty: PtySession,
    bevy_app: App,
    screen: Screen,
}

impl BevyTuiTestHarness {
    pub fn with_bevy_ratatui() -> Result<Self> {
        let pty = PtySession::new(80, 24)?;
        let mut bevy_app = App::new();
        bevy_app.add_plugins(MinimalPlugins);
        bevy_app.add_plugins(BevyRatatuiPlugin);

        Ok(Self { pty, bevy_app, screen: Screen::new() })
    }

    pub fn query<T: Component>(&self) -> Vec<&T> {
        // Query Bevy ECS
    }

    pub fn update(&mut self) -> Result<()> {
        self.bevy_app.update();
        self.screen.sync_from_pty(&mut self.pty)?;
        Ok(())
    }

    pub fn assert_component_visible<T>(&self) -> Result<()> {
        // Check Bevy component + screen output
    }
}
```

**Usage in Scarab:**

```rust
#[test]
fn test_scarab_command_palette() {
    let mut test = BevyTuiTestHarness::with_bevy_ratatui()?;

    // Spawn scarab-client
    test.spawn_app(Command::new("./scarab-client"))?;

    // Send Ctrl+Shift+P
    test.press_key(KeyCode::Char('p'), Modifiers::CTRL | Modifiers::SHIFT)?;
    test.update()?;

    // Assert: Command palette visible in both Bevy and terminal
    test.assert_component_visible::<CommandPaletteMarker>()?;
    test.assert_screen_contains("Commands")?;
}
```

**Benefits:**
- Leverages existing tool
- Reusable for other Bevy+Ratatui apps
- PTY + ECS integration

**Drawbacks:**
- Ratatui-testlib still in development
- Adds external dependency risk
- May not fit its mission

**Effort:** 5-6 weeks (including upstream contribution)
**Risk:** HIGH (external dependency)
**ROI:** MEDIUM (depends on ratatui-testlib adoption)

---

## Solution Comparison

| Criteria | Headless Harness | Abstraction Layer | Ratatui-testlib |
|----------|------------------|-------------------|-----------------|
| **Speed to MVP** | ✅ 3 weeks | ❌ 5 weeks | ❌ 6 weeks |
| **Architecture Impact** | ✅ Minimal | ❌ Large refactor | ✅ Minimal |
| **Long-term Maintenance** | ⚠️ Medium | ✅ Low (clean arch) | ⚠️ Depends on upstream |
| **Testing Capability** | ✅ Full UI testing | ✅ Full + unit tests | ✅ Full (PTY + UI) |
| **Risk** | ✅ LOW | ⚠️ MEDIUM | ❌ HIGH |
| **Effort** | ✅ 3 weeks | ❌ 5 weeks | ❌ 6 weeks |
| **ROI** | ✅ HIGH | ✅ HIGH | ⚠️ MEDIUM |

**Winner:** Headless Bevy Test Harness (Solution 1)

---

## Implementation Roadmap

### Week 1: POC
- [ ] Create `tests/headless_poc.rs`
- [ ] Prove Bevy runs without window
- [ ] Prove components can be queried
- [ ] Prove Assets can be mocked
- [ ] Document findings

**Deliverable:** POC passing in CI

### Week 2: Harness Foundation
- [ ] Create `HeadlessTestHarness` struct
- [ ] Implement `new()`, `update()`, `query()`
- [ ] Add mock SharedState
- [ ] Add mock Assets<Image>
- [ ] Write 5 example tests

**Deliverable:** Reusable test harness

### Week 3: UI Component Tests
- [ ] Test command palette
- [ ] Test link hints
- [ ] Test overlays
- [ ] Test visual selection
- [ ] Test scroll indicator

**Deliverable:** 15+ UI tests

### Week 4: Integration + Documentation
- [ ] Add snapshot testing support
- [ ] Performance benchmarks
- [ ] Write testing guide
- [ ] Update CI configuration

**Deliverable:** Complete testing infrastructure

---

## Success Metrics

✅ **Definition of Done:**
1. Developers can run `cargo test` to verify UI changes
2. Tab bar, overlays, command palette tested automatically
3. Tests run in CI without display server
4. Test suite runs in < 10 seconds
5. Documentation explains how to add new tests

---

## Appendix: Example Test Cases

### Tab Bar

```rust
#[test]
fn tab_bar_shows_all_tabs() {
    let mut test = HeadlessTestHarness::new();
    test.spawn_tabs(&["Editor", "Terminal", "Files"]);

    test.assert_component_count::<TabMarker>(3);
}

#[test]
fn tab_bar_highlights_active_tab() {
    let mut test = HeadlessTestHarness::new();
    test.spawn_tabs(&["Tab1", "Tab2"]);
    test.set_active_tab(1);

    let tabs = test.get_components::<TabMarker>();
    assert!(tabs[1].is_active);
    assert!(!tabs[0].is_active);
}
```

### Command Palette

```rust
#[test]
fn command_palette_filters_by_query() {
    let mut test = HeadlessTestHarness::new();
    test.open_command_palette();

    test.type_text("file");
    test.update();

    let items = test.get_palette_items();
    assert!(items.iter().all(|i| i.name.to_lowercase().contains("file")));
}
```

### Link Hints

```rust
#[test]
fn link_hints_position_correctly() {
    let mut test = HeadlessTestHarness::new();
    test.set_grid_text(5, "URL: https://example.com");

    test.activate_link_hints();

    let hints = test.get_components::<LinkHintMarker>();
    let hint_pos = hints[0].position;

    // Should be near "https" start (col ~5)
    assert!(hint_pos.x >= 5.0 * CELL_WIDTH);
    assert!(hint_pos.y == 5.0 * CELL_HEIGHT);
}
```

---

**Document:** 05-FRONTEND-TESTING-SOLUTIONS.md
**Recommendation:** Implement Solution 1 (Headless Harness) immediately
**Timeline:** 4 weeks to full implementation
