# Issue #8: Advanced UI/UX Features

**Phase**: 3B - Advanced Features
**Priority**: 🟢 Medium
**Workstream**: UI/UX
**Estimated Effort**: 2 weeks
**Assignee**: UI/UX Specialist Agent

---

## 🎯 Objective

Implement power-user UI features including Vimium-style link hints, Spacemacs-like leader key menus, command palette with fuzzy search, and configurable key bindings.

---

## 📋 Background

With rendering and interpreter in place, we can now build advanced UI overlays:
- Link hints for clickable URLs/paths
- Leader key command palette
- Fuzzy search for commands
- Visual feedback and animations
- Customizable key bindings

---

## ✅ Acceptance Criteria

- [ ] Link hint overlay (Vimium-style)
- [ ] Leader key menu system
- [ ] Command palette with fuzzy search
- [ ] Visual selection mode
- [ ] Configurable key bindings
- [ ] Smooth animations (fade in/out)
- [ ] Theme system integration
- [ ] <200ms menu open time
- [ ] Keyboard-only workflow
- [ ] Integration with Fusabi scripts

---

## 🔧 Technical Approach

### Step 1: Link Hints
```rust
#[derive(Component)]
struct LinkHint {
    url: String,
    position: Vec2,
    hint_key: String, // e.g., "aa", "ab", etc.
}

fn detect_links(grid: &SharedState) -> Vec<LinkHint> {
    // Scan grid for URLs, file paths
    // Regex: https?://, /path/to/file, etc.
}

fn show_hints(mut commands: Commands, hints: Vec<LinkHint>) {
    for hint in hints {
        commands.spawn((
            hint,
            TextBundle {
                text: Text::from_section(hint.hint_key, style),
                ..default()
            },
        ));
    }
}
```

### Step 2: Leader Key System
```rust
#[derive(Resource)]
struct LeaderKeyState {
    active: bool,
    last_press: Instant,
    timeout: Duration,
}

fn handle_leader_key(
    keys: Res<ButtonInput<KeyCode>>,
    mut state: ResMut<LeaderKeyState>,
) {
    if keys.just_pressed(KeyCode::Space) {
        state.active = true;
        state.last_press = Instant::now();
        show_command_menu();
    }
}
```

### Step 3: Command Palette
```rust
#[derive(Clone)]
struct Command {
    name: String,
    description: String,
    keybind: Option<String>,
    action: Arc<dyn Fn() + Send + Sync>,
}

fn fuzzy_search(query: &str, commands: &[Command]) -> Vec<Command> {
    // Use fuzzy-matcher crate
    commands.iter()
        .map(|cmd| (cmd, fuzzy_match(&cmd.name, query)))
        .filter(|(_, score)| score.is_some())
        .sorted_by_key(|(_, score)| -score.unwrap())
        .map(|(cmd, _)| cmd.clone())
        .collect()
}
```

---

## 📦 Deliverables

1. **Code**: `scarab-client/src/ui/` module
2. **Overlays**: Link hints, menus, palette
3. **Keybindings**: Configurable bindings system
4. **Animations**: Smooth transitions
5. **Examples**: Fusabi UI scripts

---

## 🔗 Dependencies

- **Depends On**: Issue #2 (Rendering) - needs UI primitives
- **Depends On**: Issue #5 (Interpreter) - for script customization
- **Blocks**: None

---

## 📚 Resources

- [Vimium Link Hints](https://github.com/philc/vimium)
- [Spacemacs Leader Key](https://www.spacemacs.org/)
- [fuzzy-matcher Crate](https://docs.rs/fuzzy-matcher/)
- [Bevy UI Examples](https://bevyengine.org/examples/UI%20(User%20Interface)/)

---

## 🎯 Success Metrics

- ✅ Link detection accuracy >95%
- ✅ Menu open <200ms
- ✅ Fuzzy search <50ms for 1000 commands
- ✅ Keyboard-only workflow
- ✅ Smooth 60 FPS animations

---

**Created**: 2025-11-21
**Labels**: `phase-3`, `medium-priority`, `ui-ux`, `power-user`
