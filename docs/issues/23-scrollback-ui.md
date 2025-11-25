# Issue #23: Scrollback UI Implementation

## 🎯 Goal
Implement comprehensive scrollback buffer visualization and navigation in the Scarab client.

## 🐛 Problem
Users currently cannot:
- Review command history beyond visible terminal area
- Scroll through previous output
- Search through terminal history
- Copy text from scrollback buffer

This is a **CRITICAL** missing feature for terminal usability.

## 💡 Proposed Solution

### 1. Scrollback Buffer Management
**Location**: `crates/scarab-client/src/terminal/scrollback.rs` (new file)

```rust
pub struct ScrollbackBuffer {
    /// Lines stored in scrollback (separate from visible grid)
    lines: VecDeque<ScrollbackLine>,
    /// Maximum lines to keep (default: 10,000)
    max_lines: usize,
    /// Current scroll position (0 = bottom/latest)
    scroll_offset: usize,
}

pub struct ScrollbackLine {
    cells: Vec<Cell>,
    timestamp: SystemTime,
}
```

### 2. Mouse Wheel Scrolling
**Location**: `crates/scarab-client/src/input/mouse.rs`

- Capture mouse wheel events from Bevy
- Update `scroll_offset` in ScrollbackBuffer
- Trigger re-render with scrolled view

### 3. Keyboard Navigation
**Location**: `crates/scarab-client/src/input/keyboard.rs`

Keybindings:
- `Shift+PageUp`: Scroll up one page
- `Shift+PageDown`: Scroll down one page
- `Shift+Home`: Jump to top of scrollback
- `Shift+End`: Jump to bottom (live view)

### 4. Search in Scrollback
**Location**: `crates/scarab-client/src/ui/search_overlay.rs` (new file)

UI Flow:
1. `Ctrl+F` opens search overlay
2. Type search term (with regex support)
3. Highlight all matches in scrollback
4. `Enter` to jump to next match
5. `Shift+Enter` to jump to previous match
6. `Esc` to close search

### 5. Copy from Scrollback
**Location**: `crates/scarab-client/src/input/selection.rs`

- Text selection should work in scrollback area
- Mouse drag to select
- `Ctrl+C` to copy to clipboard
- Integrate with existing selection system

## 📋 Implementation Tasks

### Phase 1: Data Structure (1 day)
- [ ] Create `ScrollbackBuffer` struct
- [ ] Add to client state
- [ ] Connect to terminal grid updates
- [ ] Implement max line limit with LRU eviction

### Phase 2: Scrolling (1 day)
- [ ] Add mouse wheel event handling
- [ ] Implement keyboard shortcuts (Shift+PageUp/Down)
- [ ] Update renderer to show scrollback lines
- [ ] Add scroll position indicator

### Phase 3: Search (1 day)
- [ ] Create search overlay UI component
- [ ] Implement regex search through scrollback
- [ ] Add match highlighting
- [ ] Add next/previous navigation

### Phase 4: Copy Support (half day)
- [ ] Extend selection to work in scrollback
- [ ] Ensure clipboard integration works
- [ ] Add visual feedback for selection

## 🎨 UI Design

### Scroll Indicator
```
┌─────────────────────────┐
│ [Terminal Content]      │ ← Scrollback line 500
│                         │
│                         │ ← Scroll indicator: "↑ 500 lines"
│ $ command               │
└─────────────────────────┘
```

### Search Overlay
```
┌─────────────────────────┐
│ [Terminal Content]      │
│                         │
├─────────────────────────┤
│ Find: [search term___]  │ ← Overlay at bottom
│ 3 of 15 matches         │
└─────────────────────────┘
```

## 🧪 Testing

1. **Scrollback Storage**: Generate 20,000 lines, verify only 10,000 kept
2. **Smooth Scrolling**: Mouse wheel should feel responsive
3. **Search Performance**: Search through 10,000 lines < 100ms
4. **Copy Accuracy**: Selected text matches actual content

## 📊 Success Criteria

- [ ] User can scroll through 10,000+ lines of history
- [ ] Mouse wheel scrolling feels smooth (60 FPS maintained)
- [ ] Search finds matches in < 100ms
- [ ] Copy from scrollback works reliably
- [ ] No memory leaks with long-running sessions

## 📚 References

- Alacritty scrollback implementation: [alacritty/scrollback.rs](https://github.com/alacritty/alacritty/blob/master/alacritty/src/display/content.rs)
- WezTerm scrollback: [wezterm/scrollback](https://github.com/wez/wezterm/blob/main/termwiz/src/surface/line.rs)

## 🔗 Related Issues

- Issue #18: Copy/Paste Enhancement (depends on this)
- Issue #25: Mouse Support (related)

---

**Priority**: 🔴 CRITICAL
**Effort**: 2-3 days
**Assignee**: Frontend Developer Agent
