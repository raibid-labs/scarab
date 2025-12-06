# Fusabi-TUI Integration Plan

This document outlines the integration of fusabi-tui into Scarab's client UI, replacing custom helper functions with standardized widgets and utilities.

## Overview

**fusabi-tui** provides:
- Ratatui bindings for Fusabi scripts
- Pre-built widgets (progress bars, lists, tables, etc.)
- Layout utilities and formatting helpers
- Navigation and focus management
- Color/style utilities

## Current Custom UI Helpers

The following modules contain custom UI code that could potentially use fusabi-tui:

### High Priority (Duplicated Functionality)

1. **`crates/scarab-client/src/ui/grid_utils.rs`** (~7.5KB)
   - Grid layout calculations
   - Cell positioning
   - **Replacement**: Use fusabi-tui layout utilities

2. **`crates/scarab-client/src/ui/animations.rs`** (~8KB)
   - Custom animation helpers
   - Easing functions
   - **Replacement**: Use fusabi-tui animation framework if available

3. **`crates/scarab-client/src/ratatui_bridge/renderer.rs`** (~11.7KB)
   - Custom Ratatui rendering
   - **Replacement**: Use fusabi-tui rendering abstractions

### Medium Priority (Partial Overlap)

4. **`crates/scarab-client/src/ui/overlays.rs`** (~9.3KB)
   - Modal/overlay rendering
   - **Action**: Evaluate fusabi-tui overlay widgets

5. **`crates/scarab-client/src/ui/scroll_indicator.rs`** (~3.6KB)
   - Scrollbar rendering
   - **Action**: Use fusabi-tui scrollbar widget if available

6. **`crates/scarab-client/src/ratatui_bridge/surface.rs`** (~14.9KB)
   - Custom surface abstraction
   - **Action**: Migrate to fusabi-tui surface API

### Low Priority (Domain-Specific)

These are Scarab-specific and should remain:
- `link_hints.rs` - Terminal-specific feature
- `command_palette.rs` - Custom implementation
- `plugin_menu.rs` - Plugin system integration
- `status_bar.rs` - Custom status bar
- `leader_key.rs` - Keybinding system

## Integration Steps

### Phase 1: Add fusabi-tui Imports
```rust
// Add to relevant files
use fusabi_tui::{
    widgets::{Block, List, Table, Scrollbar},
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
};
```

### Phase 2: Replace Grid Utilities
```rust
// Before (custom)
use crate::ui::grid_utils::{calculate_cell_position, GridLayout};

// After (fusabi-tui)
use fusabi_tui::layout::{Layout, Constraint};

let chunks = Layout::default()
    .direction(Direction::Horizontal)
    .constraints([
        Constraint::Percentage(50),
        Constraint::Percentage(50),
    ])
    .split(area);
```

### Phase 3: Standardize Widget Rendering
```rust
// Use fusabi-tui widgets for common UI elements
use fusabi_tui::widgets::{Block, Borders, BorderType};

let block = Block::default()
    .title("My Widget")
    .borders(Borders::ALL)
    .border_type(BorderType::Rounded);
```

### Phase 4: Update Ratatui Bridge
- Migrate `ratatui_bridge/renderer.rs` to use fusabi-tui abstractions
- Simplify `ratatui_bridge/surface.rs` with fusabi-tui surface API
- Remove duplicate functionality

### Phase 5: Update Tests
- Migrate UI snapshot tests to use fusabi-tui test utilities
- Update golden tests with ratatui-testlib integration

## API Compatibility

### fusabi-tui Widget API
```rust
// Example: Using fusabi-tui for command palette
use fusabi_tui::widgets::{List, ListItem, ListState};

fn render_command_palette(items: &[String]) -> List {
    let items: Vec<ListItem> = items
        .iter()
        .map(|i| ListItem::new(i.as_str()))
        .collect();

    List::new(items)
        .block(Block::default().title("Commands").borders(Borders::ALL))
        .highlight_style(Style::default().fg(Color::Yellow))
}
```

### Layout Utilities
```rust
// Before: Custom grid calculations
let cell_x = (col as f32 * cell_width) as u16;
let cell_y = (row as f32 * cell_height) as u16;

// After: fusabi-tui layout
let layout = Layout::default()
    .direction(Direction::Vertical)
    .constraints([
        Constraint::Length(3),  // Header
        Constraint::Min(0),      // Content
        Constraint::Length(1),   // Footer
    ])
    .split(area);
```

## Migration Checklist

- [ ] Audit `ui/` modules for fusabi-tui equivalents
- [ ] Replace grid_utils with fusabi-tui Layout
- [ ] Migrate animation helpers to fusabi-tui
- [ ] Simplify ratatui_bridge using fusabi-tui
- [ ] Update widget rendering to use fusabi-tui widgets
- [ ] Remove duplicated code
- [ ] Update tests with fusabi-tui test utilities
- [ ] Update documentation

## Benefits

1. **Reduced Code**: Remove ~500-1000 lines of custom UI code
2. **Better Maintainability**: Use community-maintained widgets
3. **Consistency**: Standardized UI patterns across Scarab
4. **Fusabi Integration**: Native support for F# UI scripts
5. **Better Testing**: Improved test utilities from fusabi-tui

## Backward Compatibility

All changes are internal to the client. The plugin API remains unchanged.

## Testing Strategy

1. **Unit Tests**: Test widget replacements with fusabi-tui
2. **Integration Tests**: Use ratatui-testlib for UI tests
3. **Visual Regression**: Golden snapshots with ratatui-testlib
4. **Manual Testing**: Verify UI appearance and behavior

## Timeline

- **Week 1**: Audit and planning (✓ This document)
- **Week 2**: Replace grid utilities and layout code
- **Week 3**: Migrate widget rendering
- **Week 4**: Update tests and cleanup

## Related Issues

- #97: Upgrade to fusabi-tui / bevy-fusabi and Remove Duplicated UI Helpers
- #98: Integrate ratatui-testlib Smoke Tests
- #101: Adopt shared libs (fusabi runtime + tests)

## References

- [fusabi-tui Documentation](https://docs.rs/fusabi-tui)
- [ratatui Documentation](https://docs.rs/ratatui)
- [Scarab UI Architecture](../crates/scarab-client/src/ratatui_bridge/ARCHITECTURE.md)
