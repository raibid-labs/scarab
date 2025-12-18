# Bevy 0.15 Migration Guide for Scarab

**Last Updated**: 2025-11-23
**Target**: Developers working on Scarab's UI features
**Bevy Version**: 0.15.3

---

## Overview

This guide helps developers migrate Scarab's UI code to Bevy 0.15's new APIs. Bevy 0.15 introduced significant changes to text rendering, color handling, and UI bundle structures.

### Why This Guide Exists

Scarab's core rendering system has been migrated to Bevy 0.15, but advanced UI features (link hints, command palette, leader key menu) were temporarily disabled to prioritize core integration. This guide documents the API changes needed to re-enable these features.

### Migration Status

- ✅ **Core Rendering**: Migrated to Bevy 0.15
  - Color API (`srgba()`, `to_srgba()`)
  - Mesh generation and texture atlas
  - Terminal grid rendering
- 🔄 **Advanced UI**: In Progress (Phase 5A)
  - Link hints system
  - Command palette
  - Leader key menu
  - Visual selection overlays
  - Animations

---

## API Changes

### 1. Text Rendering

#### 1.1. Text Creation

**Before (Bevy 0.14)**:
```rust
let text = Text::from_section(
    "Hello, World!",
    TextStyle {
        font: font_handle.clone(),
        font_size: 16.0,
        color: Color::WHITE,
    }
);
```

**After (Bevy 0.15)**:
```rust
use bevy::text::{Text, TextSection};

let text = Text::from_sections([
    TextSection::new(
        "Hello, World!",
        TextStyle {
            font: font_handle.clone(),
            font_size: 16.0,
            color: Color::WHITE,
        }
    )
]);
```

**Key Changes**:
- `Text::from_section()` → `Text::from_sections()` (note the plural)
- Takes an array/vec of `TextSection` instead of individual parameters
- `TextSection::new()` replaces the old tuple-based approach

#### 1.2. Multi-Section Text

**Before (Bevy 0.14)**:
```rust
commands.spawn(TextBundle::from_sections([
    ("Label: ", style1),
    ("Value", style2),
]));
```

**After (Bevy 0.15)**:
```rust
commands.spawn(TextBundle::from_sections([
    TextSection::new("Label: ", style1),
    TextSection::new("Value", style2),
]));
```

### 2. Color API

#### 2.1. Color Construction

**Before (Bevy 0.14)**:
```rust
let color = Color::rgba(1.0, 0.5, 0.0, 0.8);  // RGBA in linear space
let color_u8 = Color::rgba_u8(255, 128, 0, 204);
```

**After (Bevy 0.15)**:
```rust
// sRGB color space (standard for UI)
let color = Color::srgba(1.0, 0.5, 0.0, 0.8);
let color_u8 = Color::srgba_u8(255, 128, 0, 204);

// Linear color space (for lighting calculations)
let color_linear = Color::linear_rgba(1.0, 0.5, 0.0, 0.8);
```

**Key Changes**:
- `rgba()` → `srgba()` for UI colors
- `rgba_u8()` → `srgba_u8()` for byte colors
- New `linear_rgba()` for lighting/physics calculations
- Bevy now distinguishes between sRGB (UI/textures) and linear (lighting) color spaces

#### 2.2. Color Conversion

**Before (Bevy 0.14)**:
```rust
let [r, g, b, a] = color.as_rgba_f32();
let [r, g, b, a] = color.as_linear_rgba_f32();
```

**After (Bevy 0.15)**:
```rust
use bevy::color::ColorToComponents;

let [r, g, b, a] = color.to_srgba().to_f32_array();
let [r, g, b, a] = color.to_linear().to_f32_array();
```

**Key Changes**:
- `as_rgba_f32()` → `to_srgba().to_f32_array()`
- Must import `ColorToComponents` trait
- Explicit color space conversion

### 3. UI Bundles

#### 3.1. NodeBundle

**Before (Bevy 0.14)**:
```rust
commands.spawn(NodeBundle {
    style: Style {
        size: Size::new(Val::Px(200.0), Val::Px(100.0)),
        position_type: PositionType::Absolute,
        position: UiRect {
            left: Val::Px(10.0),
            top: Val::Px(20.0),
            ..default()
        },
        ..default()
    },
    background_color: BackgroundColor(Color::rgb(0.1, 0.1, 0.1)),
    ..default()
});
```

**After (Bevy 0.15)**:
```rust
commands.spawn(NodeBundle {
    style: Style {
        width: Val::Px(200.0),
        height: Val::Px(100.0),
        position_type: PositionType::Absolute,
        left: Val::Px(10.0),
        top: Val::Px(20.0),
        ..default()
    },
    background_color: BackgroundColor(Color::srgb(0.1, 0.1, 0.1)),
    ..default()
});
```

**Key Changes**:
- `size: Size::new(w, h)` → `width: w, height: h` (separate fields)
- `position: UiRect { left, top, ... }` → `left: ..., top: ...` (flattened)
- `Color::rgb()` → `Color::srgb()`

#### 3.2. TextBundle

**Before (Bevy 0.14)**:
```rust
commands.spawn(TextBundle::from_section(
    "Click me!",
    TextStyle {
        font: font.clone(),
        font_size: 20.0,
        color: Color::WHITE,
    },
).with_style(Style {
    position_type: PositionType::Absolute,
    ..default()
}));
```

**After (Bevy 0.15)**:
```rust
commands.spawn(TextBundle {
    text: Text::from_sections([
        TextSection::new(
            "Click me!",
            TextStyle {
                font: font.clone(),
                font_size: 20.0,
                color: Color::WHITE,
            }
        )
    ]),
    style: Style {
        position_type: PositionType::Absolute,
        ..default()
    },
    ..default()
});
```

**Key Changes**:
- `TextBundle::from_section()` replaced with explicit struct construction
- `.with_style()` removed - use `style: Style { ... }` directly
- Must use `Text::from_sections()` for the text field

### 4. Sprite Rendering

#### 4.1. SpriteBundle

**Before (Bevy 0.14)**:
```rust
commands.spawn(SpriteBundle {
    sprite: Sprite {
        color: Color::rgba(0.0, 1.0, 0.0, 0.3),
        custom_size: Some(Vec2::new(100.0, 50.0)),
        ..default()
    },
    transform: Transform::from_xyz(0.0, 0.0, 1.0),
    ..default()
});
```

**After (Bevy 0.15)**:
```rust
commands.spawn(SpriteBundle {
    sprite: Sprite {
        color: Color::srgba(0.0, 1.0, 0.0, 0.3),
        custom_size: Some(Vec2::new(100.0, 50.0)),
        ..default()
    },
    transform: Transform::from_xyz(0.0, 0.0, 1.0),
    ..default()
});
```

**Key Changes**:
- `Color::rgba()` → `Color::srgba()`
- Structure mostly unchanged

---

## Migration Checklist for Scarab UI

### Files to Update

#### 1. `crates/scarab-client/src/ui/link_hints.rs`

**Lines 140-180** (approximate):

- [ ] Update `TextBundle` creation for hint labels
- [ ] Change `Text::from_section()` → `Text::from_sections([TextSection::new(...)])`
- [ ] Update `Color::rgba()` → `Color::srgba()`
- [ ] Update `Style` fields (`size` → `width`/`height`)

**Example**:
```rust
// OLD
commands.spawn(TextBundle::from_section(
    hint.key,
    TextStyle {
        font: font.clone(),
        font_size: 14.0,
        color: Color::rgba(0.0, 0.0, 0.0, 1.0),
    }
));

// NEW
commands.spawn(TextBundle {
    text: Text::from_sections([
        TextSection::new(
            hint.key,
            TextStyle {
                font: font.clone(),
                font_size: 14.0,
                color: Color::srgba(0.0, 0.0, 0.0, 1.0),
            }
        )
    ]),
    ..default()
});
```

#### 2. `crates/scarab-client/src/ui/command_palette.rs`

**Lines 230-300** (approximate):

- [ ] Update palette background `NodeBundle`
- [ ] Update search input `TextBundle`
- [ ] Update command list item `TextBundle`s
- [ ] Update category headers
- [ ] Fix `Style.size` → `Style.width`/`height`

#### 3. `crates/scarab-client/src/ui/leader_key.rs`

**Lines 200-280** (approximate):

- [ ] Update menu container `NodeBundle`
- [ ] Update menu item `TextBundle`s
- [ ] Update timeout indicator rendering
- [ ] Update submenu `TextBundle`s

#### 4. `crates/scarab-client/src/ui/visual_selection.rs`

- [ ] Update selection overlay `SpriteBundle`
- [ ] Update `Color::rgba()` → `Color::srgba()` for overlay color

#### 5. `crates/scarab-client/src/ui/animations.rs`

- [ ] Update alpha interpolation to use `to_srgba()` and `to_f32_array()`
- [ ] Ensure `ColorToComponents` trait is imported

---

## Testing After Migration

### 1. Compilation Test

```bash
# Should compile without warnings
cargo check -p scarab-client

# Should build successfully
cargo build -p scarab-client
```

### 2. Unit Tests

```bash
# All UI tests should still pass
cargo test -p scarab-client ui_tests

# Specific test modules
cargo test -p scarab-client test_link_hints
cargo test -p scarab-client test_command_palette
cargo test -p scarab-client test_key_bindings
```

### 3. Integration Tests

After migration, re-enable `AdvancedUIPlugin` in `crates/scarab-client/src/lib.rs`:

```rust
// In the app setup
app.add_plugins(AdvancedUIPlugin);
```

Then test manually:

```bash
# Terminal 1: Start daemon
cargo run -p scarab-daemon

# Terminal 2: Start client
cargo run -p scarab-client
```

**Manual test checklist**:
- [ ] Link hints appear when pressing `Ctrl+K`
- [ ] Command palette opens with `Ctrl+P`
- [ ] Leader menu activates with `Space`
- [ ] Visual selection works with `v`/`V`/`Ctrl+V`
- [ ] Text renders clearly (no color issues)
- [ ] UI animations are smooth (60 FPS)
- [ ] No visual artifacts or rendering glitches

### 4. Performance Validation

Ensure no performance regression:

```bash
# Run performance benchmarks
cargo bench -p scarab-client

# Check targets:
# - Menu open time: <200ms (ideally <100ms)
# - Fuzzy search: <50ms for 1000 commands
# - Animation frame time: <16ms (60 FPS)
```

---

## Common Pitfalls

### 1. Forgetting to Import TextSection

**Error**:
```
error[E0425]: cannot find value `TextSection` in this scope
```

**Solution**:
```rust
use bevy::text::{Text, TextSection};
```

### 2. Using Old Color API

**Error**:
```
error[E0599]: no method named `rgba` found for struct `Color`
```

**Solution**:
```rust
// Change:
Color::rgba(r, g, b, a)
// To:
Color::srgba(r, g, b, a)
```

### 3. Missing ColorToComponents Import

**Error**:
```
error[E0599]: no method named `to_srgba` found for struct `Color`
```

**Solution**:
```rust
use bevy::color::ColorToComponents;
```

### 4. Style Field Access

**Error**:
```
error[E0609]: no field `size` on type `Style`
```

**Solution**:
```rust
// Change:
Style {
    size: Size::new(Val::Px(200.0), Val::Px(100.0)),
    ..default()
}
// To:
Style {
    width: Val::Px(200.0),
    height: Val::Px(100.0),
    ..default()
}
```

---

## Examples

### Complete Example: Link Hint Rendering

**Before (Bevy 0.14)**:
```rust
fn render_link_hints(
    mut commands: Commands,
    hints: Res<LinkHints>,
    font: Res<FontHandle>,
) {
    for hint in &hints.items {
        // Background
        commands.spawn(NodeBundle {
            style: Style {
                size: Size::new(Val::Px(24.0), Val::Px(20.0)),
                position_type: PositionType::Absolute,
                position: UiRect {
                    left: Val::Px(hint.x),
                    top: Val::Px(hint.y),
                    ..default()
                },
                ..default()
            },
            background_color: BackgroundColor(Color::rgba(1.0, 0.9, 0.0, 0.9)),
            ..default()
        })
        .with_children(|parent| {
            // Text
            parent.spawn(TextBundle::from_section(
                &hint.key,
                TextStyle {
                    font: font.0.clone(),
                    font_size: 14.0,
                    color: Color::rgba(0.0, 0.0, 0.0, 1.0),
                }
            ));
        });
    }
}
```

**After (Bevy 0.15)**:
```rust
use bevy::text::{Text, TextSection};

fn render_link_hints(
    mut commands: Commands,
    hints: Res<LinkHints>,
    font: Res<FontHandle>,
) {
    for hint in &hints.items {
        // Background
        commands.spawn(NodeBundle {
            style: Style {
                width: Val::Px(24.0),
                height: Val::Px(20.0),
                position_type: PositionType::Absolute,
                left: Val::Px(hint.x),
                top: Val::Px(hint.y),
                ..default()
            },
            background_color: BackgroundColor(Color::srgba(1.0, 0.9, 0.0, 0.9)),
            ..default()
        })
        .with_children(|parent| {
            // Text
            parent.spawn(TextBundle {
                text: Text::from_sections([
                    TextSection::new(
                        &hint.key,
                        TextStyle {
                            font: font.0.clone(),
                            font_size: 14.0,
                            color: Color::srgba(0.0, 0.0, 0.0, 1.0),
                        }
                    )
                ]),
                ..default()
            });
        });
    }
}
```

---

## Status Tracking

### Expected Completion

**Phase 5A Timeline**: 4-6 hours estimated
- File updates: 2-3 hours
- Testing: 1-2 hours
- Bug fixes: 1 hour buffer

### Success Criteria

- ✅ All UI files compile without warnings
- ✅ All 35+ UI tests pass
- ✅ UI features render correctly in live client
- ✅ No performance regression (<200ms menu open time)
- ✅ No visual artifacts or color issues
- ✅ `AdvancedUIPlugin` re-enabled in main client

---

## Additional Resources

### Official Bevy Documentation

- [Bevy 0.15 Migration Guide](https://bevyengine.org/learn/migration-guides/0.14-0.15/)
- [Bevy UI Module Docs](https://docs.rs/bevy/0.15/bevy/ui/)
- [Bevy Text Module Docs](https://docs.rs/bevy/0.15/bevy/text/)
- [Bevy Color Module Docs](https://docs.rs/bevy/0.15/bevy/color/)

### Scarab-Specific Resources

- [ROADMAP.md](./ROADMAP.md) - Phase 5A details
- [IMPLEMENTATION_SUMMARY.md](./IMPLEMENTATION_SUMMARY.md) - UI feature documentation
- [CLAUDE.md](./CLAUDE.md) - Project architecture overview

---

**Last Updated**: 2025-11-23
**Maintainer**: Scarab Development Team
**Questions?** Open an issue on GitHub with the `bevy-migration` label
