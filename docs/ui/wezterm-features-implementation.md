# Wezterm Features Implementation - Sprint 2

This document summarizes the implementation of wezterm-inspired features for Scarab terminal emulator.

## Overview

Sprint 2 implemented core modal system, pickers, and mode indicator integration based on the UI Implementation Plan (Section 2).

## Files Created

### 1. Modal System
**File**: `/home/beengud/raibid-labs/scarab/crates/scarab-client/src/ui/modes.rs`

Features:
- `ScarabMode` enum with 7 modes: Normal, Copy, Search, Window, Font, Pick, Hint
- Each mode has:
  - Unique icon/emoji (e.g., "➤" for Normal, "📋" for Copy)
  - Distinct color for status bar indication
  - Help text for user guidance
  - Optional timeout (Font mode has 3s timeout)
- `ModeState` resource tracking current mode, previous mode, and entry time
- Mode switching via keyboard (Ctrl+Shift+C/F/W/P) or programmatically
- Global Escape key to return to Normal mode
- Mode timeout system for auto-exit

Example mode switch:
```rust
// Switch to Copy mode
mode_state.enter_mode(ScarabMode::Copy);

// Modes with colors:
ScarabMode::Normal  => Slime green (0.66, 0.87, 0.35)
ScarabMode::Copy    => Blue (0.4, 0.7, 1.0)
ScarabMode::Search  => Yellow (1.0, 0.8, 0.0)
ScarabMode::Window  => Pink (0.9, 0.4, 0.6)
ScarabMode::Font    => Purple (0.7, 0.5, 1.0)
ScarabMode::Pick    => Cyan (0.0, 0.9, 0.7)
ScarabMode::Hint    => Orange (1.0, 0.5, 0.0)
```

### 2. Mode Indicator
**File**: `/home/beengud/raibid-labs/scarab/crates/scarab-client/src/ui/mode_indicator.rs`

Features:
- Integrates with status bar right section
- Updates mode indicator when `ModeChangeEvent` is fired
- Shows current mode icon + name with mode-specific color
- Example: "➤ NORMAL" in slime green, "📋 COPY" in blue

### 3. Pickers System
**File**: `/home/beengud/raibid-labs/scarab/crates/scarab-client/src/ui/pickers.rs`

Features:
- Generic picker framework for selections
- Four picker types:
  - `ColorScheme` - Theme selection
  - `FontFamily` - Font picker
  - `FontSize` - Size adjustment
  - `Custom` - For plugin use
- Fuzzy search using `SkimMatcherV2`
- Keyboard navigation:
  - Type to search (fuzzy match)
  - ↑/↓ or j/k for navigation
  - Enter to select
  - Backspace to edit query
  - Escape to cancel
- Built-in item lists:
  - `create_colorscheme_items()` - 8 popular themes
  - `create_fontsize_items()` - 8pt to 24pt sizes
- Beautiful UI:
  - Centered overlay with slime green accents
  - Item descriptions and preview data
  - Shows max 10 items with "... and N more" indicator
  - Selected item highlighted with green tint

Example usage:
```rust
// Open colorscheme picker
let items = create_colorscheme_items();
picker_state.open(PickerType::ColorScheme, items);

// Listen for selections
for event in picker_select_events.read() {
    info!("Selected: {} ({})", event.item_label, event.item_id);
}
```

## Integration Points

### Status Bar Integration
- Mode indicator displays on right side of status bar
- Replaces static "NORMAL" text with dynamic mode display
- Color changes based on current mode
- Icon provides visual distinction

### Leader Key Integration
Leader key system (already implemented in `leader_key.rs`) can trigger mode switches:
```rust
// In leader key menu:
<leader>c → Copy mode
<leader>s → Search mode
<leader>w → Window mode
<leader>f → Font mode
<leader>p → Pick mode
```

### Plugin Registration
Updated `/home/beengud/raibid-labs/scarab/crates/scarab-client/src/ui/mod.rs`:
```rust
pub use modes::{ModeActionEvent, ModeChangeEvent, ModeState, ModesPlugin, ScarabMode};
pub use mode_indicator::{ModeIndicatorPlugin, ModeIndicatorText};
pub use pickers::{
    create_colorscheme_items, create_fontsize_items, PickerItem,
    PickerSelectEvent, PickerState, PickerType, PickersPlugin,
};

// In AdvancedUIPlugin:
app.add_plugins((
    // ... other plugins
    ModesPlugin,
    ModeIndicatorPlugin,
    PickersPlugin,
    // ...
))
```

## Keyboard Shortcuts

### Mode Switching (Direct)
- `Ctrl+Shift+C` - Copy mode
- `Ctrl+Shift+F` - Search mode
- `Ctrl+Shift+W` - Window mode
- `Ctrl+Shift+P` - Pick mode
- `Escape` - Always returns to Normal mode

### Picker Navigation
- Type characters - Fuzzy search
- `↑` or `k` - Move selection up
- `↓` or `j` - Move selection down
- `Enter` - Select current item
- `Backspace` - Delete last search character
- `Escape` - Close picker

## Future Enhancements

### Phase 2: Responsive Status Bar (Pending)
- Calculate available width
- Show CWD, hostname, time based on space
- Implement `ResponsiveContent` struct

### Phase 3: Additional Modes (Pending)
Modes are defined but not fully implemented:
- **Copy Mode**: Vim-like text selection (h/j/k/l navigation, v for visual, y to copy)
- **Search Mode**: Pattern search in terminal buffer (n/N for next/prev)
- **Window Mode**: Pane management (s for split, v for vsplit, hjkl navigation)
- **Font Mode**: Font size adjustment (+/- keys, 0 to reset)

### Phase 4: Additional Pickers (Pending)
- Font family picker (system fonts)
- Plugin picker (available plugins)
- Session picker (saved sessions)

## Testing

### Unit Tests
All three new modules include comprehensive unit tests:
- `modes.rs`: 4 tests covering state transitions, properties, timeout
- `pickers.rs`: 4 tests covering open/close, navigation, fuzzy search, item creation

Run tests:
```bash
cargo test -p scarab-client modes
cargo test -p scarab-client pickers
```

### Manual Testing
1. Build client: `cargo build -p scarab-client`
2. Run client: `cargo run -p scarab-client`
3. Test mode switching:
   - Press `Ctrl+Shift+C` - status bar should show "📋 COPY" in blue
   - Press `Escape` - should return to "➤ NORMAL" in slime green
4. Test picker (when integrated with keybindings):
   - Trigger picker open
   - Type to search
   - Navigate with arrow keys
   - Select with Enter

## Implementation Notes

### Simplified Keyboard Input
The picker system uses a simplified keyboard input approach (KeyCode to char conversion) rather than ReceivedCharacter events. This is sufficient for fuzzy search but may need enhancement for full unicode support.

### Mode-Specific Keybindings
Each mode should have its own keybinding context. The `modes.rs` file sets up the framework, but individual mode implementations (copy mode selection, search mode navigation, etc.) are deferred to future work.

### Plugin Tuple Limit
Bevy has a limit on tuple size (16 items). The AdvancedUIPlugin splits plugin registration across multiple `add_plugins` calls to work around this.

## Dependencies

No new dependencies added. Uses existing:
- `bevy` - UI and ECS
- `fuzzy_matcher` - Already in Cargo.toml for fuzzy search

## API Examples

### Mode System
```rust
// Get current mode
let current_mode = mode_state.current;
info!("Current mode: {}", current_mode.name());

// Switch mode
mode_state.enter_mode(ScarabMode::Copy);

// Listen for mode changes
for event in mode_change_events.read() {
    info!("Mode changed from {:?} to {:?}", event.from, event.to);
}

// Trigger mode action
mode_action_events.send(ModeActionEvent {
    mode: ScarabMode::Copy,
    action: "yank".to_string(),
});
```

### Picker System
```rust
// Create custom picker items
let items = vec![
    PickerItem::new("item-1", "First Item")
        .with_description("A great choice"),
    PickerItem::new("item-2", "Second Item")
        .with_description("Also good"),
];

// Open picker
picker_state.open(PickerType::Custom, items);

// Handle selection
for event in picker_select_events.read() {
    match event.picker_type {
        PickerType::ColorScheme => {
            // Apply color scheme
            apply_theme(&event.item_id);
        },
        PickerType::FontSize => {
            // Change font size
            let size: u32 = event.item_id.parse().unwrap();
            update_font_size(size);
        },
        _ => {}
    }
}
```

## Completion Status

### Completed
- ✅ Modal system with 7 modes
- ✅ Mode state tracking and transitions
- ✅ Mode indicator in status bar
- ✅ Picker framework with fuzzy search
- ✅ Colorscheme and font size pickers
- ✅ Keyboard navigation for pickers
- ✅ Unit tests
- ✅ Integration with AdvancedUIPlugin

### In Progress
- 🔄 Leader key mode bindings (framework exists, needs hookup)

### Pending
- ⏸ Responsive status bar
- ⏸ Copy mode implementation
- ⏸ Search mode implementation
- ⏸ Window mode implementation
- ⏸ Font mode implementation
- ⏸ Font family picker
- ⏸ Integration testing

## Next Steps

1. **Connect Leader Key to Modes**: Update leader key menu to include mode switching items
2. **Implement Copy Mode**: Add vim-like text selection in terminal buffer
3. **Implement Search Mode**: Add pattern search with n/N navigation
4. **Responsive Status Bar**: Implement width-aware content showing
5. **Font Family Picker**: Query system fonts and populate picker

## References

- UI Implementation Plan: `/home/beengud/raibid-labs/scarab/docs/UI_IMPLEMENTATION_PLAN.md` (Section 2)
- Wezterm Config Analysis: Used as inspiration for mode system design
- Bevy 0.15 API: Used for UI components and event handling
