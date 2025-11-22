# Issue #2: Text Rendering Engine

**Phase**: 1B - Core Terminal Emulation
**Priority**: 🔴 Critical
**Workstream**: Graphics/Rendering
**Estimated Effort**: 1-2 weeks
**Assignee**: Graphics/Rendering Specialist Agent

---

## 🎯 Objective

Implement GPU-accelerated text rendering using cosmic-text and Bevy to display the terminal grid at 60+ FPS.

---

## 📋 Background

The client currently reads SharedState but doesn't render text. We need to:
1. Integrate cosmic-text for font shaping and glyph rendering
2. Create a GPU texture atlas for cached glyphs
3. Generate a mesh from the grid cells
4. Handle font loading, fallbacks, and Unicode

---

## ✅ Acceptance Criteria

- [ ] cosmic-text integrated with Bevy
- [ ] Font loading from system fonts (FreeType/CoreText)
- [ ] GPU texture atlas for glyph caching
- [ ] Mesh generation from SharedState.cells
- [ ] Support 24-bit RGB colors (fg/bg)
- [ ] Support text attributes (bold, italic, underline)
- [ ] Handle Unicode combining characters
- [ ] Render at 60+ FPS with 200x100 grid
- [ ] Support font fallback chain
- [ ] Dynamic font resizing (Ctrl+/Ctrl-)

---

## 🔧 Technical Approach

### Step 1: cosmic-text Integration
```rust
use cosmic_text::{FontSystem, SwashCache, Buffer, Attrs, Family};

#[derive(Resource)]
struct TextRenderer {
    font_system: FontSystem,
    swash_cache: SwashCache,
    atlas: GlyphAtlas,
}
```

### Step 2: Glyph Atlas
```rust
struct GlyphAtlas {
    texture: Handle<Image>,
    glyph_uvs: HashMap<GlyphKey, Rect>,
    next_pos: (u32, u32),
}

impl GlyphAtlas {
    fn cache_glyph(&mut self, glyph: &Glyph) -> Rect {
        // Rasterize glyph and pack into atlas texture
    }
}
```

### Step 3: Mesh Generation
```rust
fn update_terminal_mesh(
    reader: ResMut<SharedMemoryReader>,
    mut renderer: ResMut<TextRenderer>,
    mut mesh_query: Query<&mut Mesh, With<TerminalGrid>>,
) {
    let state = unsafe { &*(reader.shmem.0.as_ptr() as *const SharedState) };

    // Generate vertices for visible cells
    for (idx, cell) in state.cells.iter().enumerate() {
        let row = idx / GRID_WIDTH;
        let col = idx % GRID_WIDTH;

        // Get glyph UV from atlas
        let uv = renderer.atlas.get_or_cache(cell.char_codepoint);

        // Add quad to mesh
        add_glyph_quad(&mut mesh, row, col, uv, cell.fg, cell.bg);
    }
}
```

### Step 4: Font Configuration
```toml
[font]
family = "JetBrains Mono"
size = 14.0
fallback = ["Noto Sans Mono", "DejaVu Sans Mono"]
```

---

## 📦 Deliverables

1. **Code**: `scarab-client/src/rendering/text.rs` module
2. **Shaders**: Custom shader for glyph rendering (if needed)
3. **Tests**: Visual regression tests with screenshots
4. **Documentation**: Rendering pipeline architecture
5. **Examples**: Demo with various fonts and sizes

---

## 🔗 Dependencies

- **Depends On**: Issue #1 (VTE Parser) - needs correct grid data
- **Blocks**: Issue #4 (Advanced UI) - rendering primitives needed

---

## 📚 Resources

- [cosmic-text Examples](https://github.com/pop-os/cosmic-text/tree/main/examples)
- [Bevy Text Rendering](https://bevyengine.org/examples/ui-ecs-render-primitives/text/)
- [GPU Text Rendering](https://wdobbie.com/post/gpu-text-rendering-with-vector-textures/)
- [Texture Atlas Packing](https://www.codeproject.com/Articles/1001888/Texture-Atlas-Generation-using-Divide-and-Conquer)

---

## 🎯 Success Metrics

- ✅ 60+ FPS sustained at 200x100 grid
- ✅ <50ms frame time P99
- ✅ GPU memory <100MB for atlas
- ✅ Support 10,000+ unique glyphs
- ✅ Font hot-reload <100ms
- ✅ No visual tearing or artifacts

---

## 💡 Implementation Notes

### Performance Optimization
- Only update mesh for dirty regions
- Use instanced rendering for glyphs
- Double-buffer mesh updates
- Profile with Tracy or Bevy diagnostic plugins

### Font Rendering Strategy
- **Monochrome**: Use SDF (Signed Distance Field) for scalability
- **Colored**: Use MSDF for emoji and icons
- **Fallback**: Software rasterization for rare glyphs

### Edge Cases
- Very large fonts (>100pt)
- Emoji and colored glyphs
- Right-to-left text
- Combining diacritics
- Zero-width characters

---

## 🐛 Known Issues

- cosmic-text 0.11 has limited Bevy integration examples
- SDF generation may need separate preprocessing
- Atlas size limits (4096x4096 max on some GPUs)

---

## 🎨 Visual Design

### Color Scheme
- Default: Solarized Dark / Dracula / Nord
- Support 24-bit RGB (16.7 million colors)
- Alpha blending for background opacity

### Font Recommendations
- JetBrains Mono (best for code)
- Fira Code (ligatures)
- Cascadia Code (Microsoft)
- Iosevka (compact)

---

**Created**: 2025-11-21
**Labels**: `phase-1`, `critical`, `rendering`, `graphics`, `bevy`
