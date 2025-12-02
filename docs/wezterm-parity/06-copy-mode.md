# WS-6: Copy Mode & Advanced Selection

**Workstream ID:** WS-6
**Priority:** P2 (Feature Parity)
**Estimated Complexity:** Medium
**Dependencies:** WS-4 (Key Tables)

## Overview

Copy mode provides vim-like keyboard navigation and selection in the terminal scrollback. Users can enter copy mode, navigate with hjkl keys, select text with visual mode, and yank to clipboard. This is essential for keyboard-driven workflows.

## Current State Analysis

### Scarab's Current Selection

The `scarab-mouse` crate provides basic mouse selection:

```rust
// Basic click-drag selection
fn handle_selection(/* ... */) {
    // Start selection on click
    // Extend selection on drag
    // Copy on release
}
```

**Limitations:**
- Mouse-only selection
- No keyboard navigation
- No vim-style visual modes
- No semantic zone selection

### WezTerm's Copy Mode

```lua
-- Enter copy mode with key binding
{ key = 'c', mods = 'LEADER', action = act.ActivateCopyMode }

-- Copy mode key table
config.key_tables = {
  copy_mode = {
    -- Navigation
    { key = 'h', action = act.CopyMode 'MoveLeft' },
    { key = 'j', action = act.CopyMode 'MoveDown' },
    { key = 'k', action = act.CopyMode 'MoveUp' },
    { key = 'l', action = act.CopyMode 'MoveRight' },
    { key = 'w', action = act.CopyMode 'MoveForwardWord' },
    { key = 'b', action = act.CopyMode 'MoveBackwardWord' },
    { key = '0', action = act.CopyMode 'MoveToStartOfLine' },
    { key = '$', action = act.CopyMode 'MoveToEndOfLine' },
    { key = 'g', action = act.CopyMode 'MoveToScrollbackTop' },
    { key = 'G', action = act.CopyMode 'MoveToScrollbackBottom' },

    -- Selection
    { key = 'v', action = act.CopyMode 'ToggleSelectionByCell' },
    { key = 'V', action = act.CopyMode 'ToggleSelectionByLine' },
    { key = 'o', action = act.CopyMode 'MoveToSelectionOtherEnd' },

    -- Clipboard
    { key = 'y', action = act.CopyMode 'Copy' },

    -- Exit
    { key = 'Escape', action = act.CopyMode 'Close' },
    { key = 'q', action = act.CopyMode 'Close' },
  },
}
```

## Target Architecture

### Copy Mode State

```rust
#[derive(Resource, Default)]
pub struct CopyModeState {
    pub active: bool,
    pub cursor: CopyModeCursor,
    pub selection: Option<Selection>,
    pub selection_mode: SelectionMode,
    pub viewport_offset: i32,  // Lines scrolled from bottom
}

#[derive(Default, Clone, Copy)]
pub struct CopyModeCursor {
    pub x: u16,
    pub y: i32,  // Can be negative (in scrollback)
}

#[derive(Clone)]
pub struct Selection {
    pub anchor: CopyModeCursor,
    pub active: CopyModeCursor,
}

#[derive(Default, Clone, Copy, PartialEq)]
pub enum SelectionMode {
    #[default]
    None,
    Cell,     // Character-by-character
    Line,     // Whole lines
    Block,    // Rectangular selection
    Word,     // Word-by-word (semantic)
}
```

### Copy Mode Actions

```rust
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum CopyModeAction {
    // Navigation
    MoveLeft,
    MoveRight,
    MoveUp,
    MoveDown,
    MoveForwardWord,
    MoveBackwardWord,
    MoveToStartOfLine,
    MoveToEndOfLine,
    MoveToStartOfLineContent,  // First non-whitespace
    MoveToScrollbackTop,
    MoveToScrollbackBottom,
    MoveToViewportTop,
    MoveToViewportMiddle,
    MoveToViewportBottom,

    // Page navigation
    PageUp,
    PageDown,
    HalfPageUp,
    HalfPageDown,

    // Selection
    ToggleSelection,          // Toggle cell selection (v)
    ToggleLineSelection,      // Toggle line selection (V)
    ToggleBlockSelection,     // Toggle block selection (Ctrl+v)
    ClearSelection,
    SelectAll,
    MoveToSelectionOtherEnd,  // Swap anchor/active (o)

    // Semantic
    MoveToSemanticZone(SemanticDirection),
    SelectSemanticZone,

    // Clipboard
    Copy,
    CopyAndExit,

    // Search
    SearchForward,
    SearchBackward,
    NextMatch,
    PrevMatch,

    // Exit
    Exit,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum SemanticDirection {
    Forward,
    Backward,
}
```

### Integration with Key Tables

Copy mode is implemented as a key table:

```rust
impl Default for KeyTableRegistry {
    fn default() -> Self {
        let mut registry = Self::new();

        registry.register("copy_mode", KeyTable {
            name: "copy_mode".into(),
            bindings: Self::default_copy_mode_bindings(),
        });

        registry
    }

    fn default_copy_mode_bindings() -> HashMap<KeyCombo, KeyAction> {
        hashmap! {
            // Navigation
            KeyCombo::key(KeyCode::KeyH) => KeyAction::CopyMode(CopyModeAction::MoveLeft),
            KeyCombo::key(KeyCode::KeyJ) => KeyAction::CopyMode(CopyModeAction::MoveDown),
            KeyCombo::key(KeyCode::KeyK) => KeyAction::CopyMode(CopyModeAction::MoveUp),
            KeyCombo::key(KeyCode::KeyL) => KeyAction::CopyMode(CopyModeAction::MoveRight),
            KeyCombo::key(KeyCode::ArrowLeft) => KeyAction::CopyMode(CopyModeAction::MoveLeft),
            KeyCombo::key(KeyCode::ArrowDown) => KeyAction::CopyMode(CopyModeAction::MoveDown),
            KeyCombo::key(KeyCode::ArrowUp) => KeyAction::CopyMode(CopyModeAction::MoveUp),
            KeyCombo::key(KeyCode::ArrowRight) => KeyAction::CopyMode(CopyModeAction::MoveRight),

            // Word movement
            KeyCombo::key(KeyCode::KeyW) => KeyAction::CopyMode(CopyModeAction::MoveForwardWord),
            KeyCombo::key(KeyCode::KeyB) => KeyAction::CopyMode(CopyModeAction::MoveBackwardWord),
            KeyCombo::key(KeyCode::KeyE) => KeyAction::CopyMode(CopyModeAction::MoveForwardWordEnd),

            // Line movement
            KeyCombo::key(KeyCode::Digit0) => KeyAction::CopyMode(CopyModeAction::MoveToStartOfLine),
            KeyCombo::shift(KeyCode::Digit4) => KeyAction::CopyMode(CopyModeAction::MoveToEndOfLine),  // $
            KeyCombo::shift(KeyCode::Digit6) => KeyAction::CopyMode(CopyModeAction::MoveToStartOfLineContent),  // ^

            // Document movement
            KeyCombo::key(KeyCode::KeyG) => KeyAction::CopyMode(CopyModeAction::MoveToScrollbackBottom),
            KeyCombo::shift(KeyCode::KeyG) => KeyAction::CopyMode(CopyModeAction::MoveToScrollbackTop),

            // Page movement
            KeyCombo::ctrl(KeyCode::KeyU) => KeyAction::CopyMode(CopyModeAction::HalfPageUp),
            KeyCombo::ctrl(KeyCode::KeyD) => KeyAction::CopyMode(CopyModeAction::HalfPageDown),
            KeyCombo::ctrl(KeyCode::KeyB) => KeyAction::CopyMode(CopyModeAction::PageUp),
            KeyCombo::ctrl(KeyCode::KeyF) => KeyAction::CopyMode(CopyModeAction::PageDown),

            // Viewport
            KeyCombo::shift(KeyCode::KeyH) => KeyAction::CopyMode(CopyModeAction::MoveToViewportTop),
            KeyCombo::shift(KeyCode::KeyM) => KeyAction::CopyMode(CopyModeAction::MoveToViewportMiddle),
            KeyCombo::shift(KeyCode::KeyL) => KeyAction::CopyMode(CopyModeAction::MoveToViewportBottom),

            // Selection
            KeyCombo::key(KeyCode::KeyV) => KeyAction::CopyMode(CopyModeAction::ToggleSelection),
            KeyCombo::shift(KeyCode::KeyV) => KeyAction::CopyMode(CopyModeAction::ToggleLineSelection),
            KeyCombo::ctrl(KeyCode::KeyV) => KeyAction::CopyMode(CopyModeAction::ToggleBlockSelection),
            KeyCombo::key(KeyCode::KeyO) => KeyAction::CopyMode(CopyModeAction::MoveToSelectionOtherEnd),

            // Clipboard
            KeyCombo::key(KeyCode::KeyY) => KeyAction::CopyMode(CopyModeAction::CopyAndExit),

            // Search
            KeyCombo::key(KeyCode::Slash) => KeyAction::CopyMode(CopyModeAction::SearchForward),
            KeyCombo::shift(KeyCode::Slash) => KeyAction::CopyMode(CopyModeAction::SearchBackward),  // ?
            KeyCombo::key(KeyCode::KeyN) => KeyAction::CopyMode(CopyModeAction::NextMatch),
            KeyCombo::shift(KeyCode::KeyN) => KeyAction::CopyMode(CopyModeAction::PrevMatch),

            // Exit
            KeyCombo::key(KeyCode::Escape) => KeyAction::CopyMode(CopyModeAction::Exit),
            KeyCombo::key(KeyCode::KeyQ) => KeyAction::CopyMode(CopyModeAction::Exit),
            KeyCombo::ctrl(KeyCode::KeyC) => KeyAction::CopyMode(CopyModeAction::Exit),
        }
    }
}
```

## Bevy Implementation

### Copy Mode Systems

```rust
pub struct CopyModePlugin;

impl Plugin for CopyModePlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<CopyModeState>()
            .add_event::<CopyModeActionEvent>()
            .add_systems(Update, (
                handle_copy_mode_actions,
                update_copy_mode_cursor,
                render_copy_mode_overlay,
                render_selection_highlight,
            ).chain().run_if(copy_mode_active));
    }
}

fn copy_mode_active(state: Res<CopyModeState>) -> bool {
    state.active
}

fn handle_copy_mode_actions(
    mut events: EventReader<KeyActionEvent>,
    mut state: ResMut<CopyModeState>,
    scrollback: Res<ScrollbackBuffer>,
    metrics: Res<TerminalMetrics>,
    mut clipboard: ResMut<ClipboardContext>,
) {
    for KeyActionEvent(action) in events.read() {
        if let KeyAction::CopyMode(action) = action {
            match action {
                CopyModeAction::MoveLeft => {
                    if state.cursor.x > 0 {
                        state.cursor.x -= 1;
                    }
                }
                CopyModeAction::MoveRight => {
                    if state.cursor.x < metrics.columns - 1 {
                        state.cursor.x += 1;
                    }
                }
                CopyModeAction::MoveUp => {
                    state.cursor.y -= 1;
                    // Clamp to scrollback bounds
                    let min_y = -(scrollback.len() as i32);
                    state.cursor.y = state.cursor.y.max(min_y);
                }
                CopyModeAction::MoveDown => {
                    state.cursor.y += 1;
                    state.cursor.y = state.cursor.y.min(metrics.rows as i32 - 1);
                }
                CopyModeAction::MoveForwardWord => {
                    state.cursor = find_next_word(&scrollback, state.cursor, metrics);
                }
                CopyModeAction::ToggleSelection => {
                    if state.selection_mode == SelectionMode::Cell {
                        state.selection_mode = SelectionMode::None;
                        state.selection = None;
                    } else {
                        state.selection_mode = SelectionMode::Cell;
                        state.selection = Some(Selection {
                            anchor: state.cursor,
                            active: state.cursor,
                        });
                    }
                }
                CopyModeAction::CopyAndExit => {
                    if let Some(text) = get_selection_text(&state, &scrollback) {
                        clipboard.set_text(text);
                    }
                    state.active = false;
                    state.selection = None;
                    state.selection_mode = SelectionMode::None;
                }
                CopyModeAction::Exit => {
                    state.active = false;
                    state.selection = None;
                    state.selection_mode = SelectionMode::None;
                }
                // ... handle other actions
            }

            // Update selection if active
            if let Some(ref mut selection) = state.selection {
                selection.active = state.cursor;
            }
        }
    }
}
```

### Selection Text Extraction

```rust
fn get_selection_text(
    state: &CopyModeState,
    scrollback: &ScrollbackBuffer,
) -> Option<String> {
    let selection = state.selection.as_ref()?;

    let (start, end) = normalize_selection(selection);

    match state.selection_mode {
        SelectionMode::Cell => {
            extract_cell_selection(scrollback, start, end)
        }
        SelectionMode::Line => {
            extract_line_selection(scrollback, start.y, end.y)
        }
        SelectionMode::Block => {
            extract_block_selection(scrollback, start, end)
        }
        SelectionMode::None => None,
    }
}

fn normalize_selection(selection: &Selection) -> (CopyModeCursor, CopyModeCursor) {
    let a = selection.anchor;
    let b = selection.active;

    if a.y < b.y || (a.y == b.y && a.x <= b.x) {
        (a, b)
    } else {
        (b, a)
    }
}

fn extract_cell_selection(
    scrollback: &ScrollbackBuffer,
    start: CopyModeCursor,
    end: CopyModeCursor,
) -> Option<String> {
    let mut result = String::new();

    for y in start.y..=end.y {
        let line = scrollback.get_line(y)?;

        let start_x = if y == start.y { start.x } else { 0 };
        let end_x = if y == end.y { end.x } else { line.len() as u16 - 1 };

        for x in start_x..=end_x {
            if let Some(cell) = line.get(x as usize) {
                result.push(char::from_u32(cell.char_codepoint).unwrap_or(' '));
            }
        }

        if y < end.y {
            result.push('\n');
        }
    }

    Some(result)
}
```

### Visual Rendering

```rust
fn render_copy_mode_overlay(
    mut commands: Commands,
    state: Res<CopyModeState>,
    metrics: Res<TerminalMetrics>,
    mut cursor_query: Query<&mut Transform, With<CopyModeCursorMarker>>,
) {
    if !state.active {
        return;
    }

    // Update cursor position
    let screen_y = state.cursor.y + state.viewport_offset;
    let x = state.cursor.x as f32 * metrics.cell_width;
    let y = -(screen_y as f32 * metrics.cell_height);

    for mut transform in cursor_query.iter_mut() {
        transform.translation.x = x;
        transform.translation.y = y;
    }
}

fn render_selection_highlight(
    mut commands: Commands,
    state: Res<CopyModeState>,
    metrics: Res<TerminalMetrics>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    highlight_query: Query<Entity, With<SelectionHighlight>>,
) {
    // Remove old highlights
    for entity in highlight_query.iter() {
        commands.entity(entity).despawn();
    }

    let Some(selection) = &state.selection else { return };

    let (start, end) = normalize_selection(selection);
    let highlight_color = Color::srgba(0.3, 0.5, 0.8, 0.4);

    match state.selection_mode {
        SelectionMode::Cell => {
            // Draw highlight rectangles for each selected row
            for y in start.y..=end.y {
                let start_x = if y == start.y { start.x } else { 0 };
                let end_x = if y == end.y { end.x } else { metrics.columns - 1 };

                spawn_highlight_rect(
                    &mut commands,
                    start_x, y, end_x - start_x + 1,
                    &metrics, highlight_color,
                    &mut meshes, &mut materials,
                );
            }
        }
        SelectionMode::Line => {
            for y in start.y..=end.y {
                spawn_highlight_rect(
                    &mut commands,
                    0, y, metrics.columns,
                    &metrics, highlight_color,
                    &mut meshes, &mut materials,
                );
            }
        }
        SelectionMode::Block => {
            let min_x = start.x.min(end.x);
            let max_x = start.x.max(end.x);

            for y in start.y..=end.y {
                spawn_highlight_rect(
                    &mut commands,
                    min_x, y, max_x - min_x + 1,
                    &metrics, highlight_color,
                    &mut meshes, &mut materials,
                );
            }
        }
        SelectionMode::None => {}
    }
}

#[derive(Component)]
struct SelectionHighlight;

#[derive(Component)]
struct CopyModeCursorMarker;
```

### Mode Indicator Integration

```rust
// In status bar or overlay
fn copy_mode_indicator(state: &CopyModeState) -> Vec<RenderItem> {
    if !state.active {
        return vec![];
    }

    let mode_text = match state.selection_mode {
        SelectionMode::None => "COPY",
        SelectionMode::Cell => "VISUAL",
        SelectionMode::Line => "V-LINE",
        SelectionMode::Block => "V-BLOCK",
    };

    vec![
        RenderItem::Background(Color::Hex("#ff9e64".into())),
        RenderItem::Foreground(Color::Hex("#1a1b26".into())),
        RenderItem::Bold,
        RenderItem::Text(format!(" {} ", mode_text)),
        RenderItem::ResetAttributes,
    ]
}
```

## Search in Copy Mode

### Search State

```rust
#[derive(Resource, Default)]
pub struct CopyModeSearch {
    pub active: bool,
    pub query: String,
    pub direction: SearchDirection,
    pub matches: Vec<SearchMatch>,
    pub current_match: Option<usize>,
}

#[derive(Clone, Copy)]
pub enum SearchDirection {
    Forward,
    Backward,
}

#[derive(Clone)]
pub struct SearchMatch {
    pub start: CopyModeCursor,
    pub end: CopyModeCursor,
}
```

### Search Implementation

```rust
fn handle_search_action(
    action: &CopyModeAction,
    state: &mut CopyModeState,
    search: &mut CopyModeSearch,
    scrollback: &ScrollbackBuffer,
) {
    match action {
        CopyModeAction::SearchForward => {
            search.active = true;
            search.direction = SearchDirection::Forward;
            // Open search input UI
        }
        CopyModeAction::SearchBackward => {
            search.active = true;
            search.direction = SearchDirection::Backward;
        }
        CopyModeAction::NextMatch => {
            if let Some(current) = search.current_match {
                let next = match search.direction {
                    SearchDirection::Forward => (current + 1) % search.matches.len(),
                    SearchDirection::Backward => {
                        if current == 0 { search.matches.len() - 1 } else { current - 1 }
                    }
                };
                search.current_match = Some(next);
                state.cursor = search.matches[next].start;
            }
        }
        CopyModeAction::PrevMatch => {
            // Opposite of NextMatch
        }
        _ => {}
    }
}

fn find_matches(query: &str, scrollback: &ScrollbackBuffer) -> Vec<SearchMatch> {
    let mut matches = Vec::new();

    for y in scrollback.min_y()..=scrollback.max_y() {
        let line = scrollback.get_line_text(y);
        for (idx, _) in line.match_indices(query) {
            matches.push(SearchMatch {
                start: CopyModeCursor { x: idx as u16, y },
                end: CopyModeCursor { x: (idx + query.len()) as u16 - 1, y },
            });
        }
    }

    matches
}
```

## Implementation Plan

### Phase 1: Core Copy Mode (Week 1)

1. Define `CopyModeState` and `CopyModeAction`
2. Implement basic cursor movement
3. Integrate with key tables system
4. Visual cursor rendering

### Phase 2: Selection (Week 1-2)

1. Implement cell selection (visual mode)
2. Implement line selection (V-LINE)
3. Implement block selection (V-BLOCK)
4. Selection highlight rendering

### Phase 3: Text Extraction (Week 2)

1. Selection text extraction
2. Clipboard integration
3. Copy and exit flow

### Phase 4: Search (Week 2-3)

1. Search input UI
2. Match finding algorithm
3. Match highlighting
4. Next/previous navigation

### Phase 5: Polish (Week 3)

1. Mode indicator
2. Semantic zone selection
3. Custom key bindings
4. Documentation

## Testing Strategy

### Unit Tests

```rust
#[test]
fn test_selection_normalization() {
    let sel = Selection {
        anchor: CopyModeCursor { x: 10, y: 5 },
        active: CopyModeCursor { x: 5, y: 3 },
    };

    let (start, end) = normalize_selection(&sel);

    assert_eq!(start.y, 3);
    assert_eq!(end.y, 5);
}

#[test]
fn test_cell_selection_single_line() {
    let mut scrollback = MockScrollback::new();
    scrollback.add_line("Hello, World!");

    let start = CopyModeCursor { x: 0, y: 0 };
    let end = CopyModeCursor { x: 4, y: 0 };

    let text = extract_cell_selection(&scrollback, start, end).unwrap();
    assert_eq!(text, "Hello");
}
```

### Integration Tests

```fsx
// test_copy_mode.fsx
// Enter copy mode
TestHelper.PressKey(KeyCode.C, Modifiers.Leader)
assert (Scarab.GetCurrentMode() = "COPY")

// Navigate
TestHelper.PressKey(KeyCode.L, Modifiers.None)  // Move right
TestHelper.PressKey(KeyCode.V, Modifiers.None)  // Start selection
TestHelper.PressKey(KeyCode.W, Modifiers.None)  // Select word

// Copy
TestHelper.PressKey(KeyCode.Y, Modifiers.None)
assert (Scarab.GetCurrentMode() = "NORMAL")
assert (Clipboard.GetText().Length > 0)
```

## Success Criteria

- [ ] Enter/exit copy mode with key binding
- [ ] hjkl navigation works in scrollback
- [ ] Visual mode (v) selects characters
- [ ] Line mode (V) selects whole lines
- [ ] Block mode (Ctrl+v) selects rectangles
- [ ] y copies selection to clipboard
- [ ] / starts forward search
- [ ] n/N navigate search matches
- [ ] Mode indicator shows current mode
