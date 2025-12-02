# Scrollback Selection Coordinate Conversion Fix

## Summary

Fixed the scrollback selection coordinate conversion issue that prevented proper mouse-based text selection when scrolled up in the terminal history.

## Location

**File:** `/home/beengud/raibid-labs/scarab/crates/scarab-client/src/ui/scrollback_selection.rs`

**Function:** `handle_mouse_selection()` and new helper function `cursor_to_scrollback_coords()`

## Problem

The scrollback selection system had placeholder TODOs for coordinate conversion:
```rust
let line = 0; // TODO: Convert cursor_pos to scrollback line
let col = 0; // TODO: Convert cursor_pos to column
```

This meant that:
1. Mouse clicks in scrollback mode would always select line 0, column 0
2. Users could not properly select text when viewing scrollback history
3. The selection coordinates were not mapped to the actual scrollback buffer indices

## Root Cause

The coordinate conversion required multiple transformations:

1. **Window coordinates → Grid coordinates**: Bevy uses bottom-left origin with Y-up, and the terminal grid is centered in the window
2. **Grid visible row → Scrollback buffer line**: When scrolled up, visible row 0 doesn't correspond to buffer line 0 - it needs offset adjustment
3. **Scroll offset calculation**: The scrollback buffer uses an offset where 0 = bottom (live view), and positive values = scrolled up

## Solution

### 1. Added TextRenderer Dependency

Modified `handle_mouse_selection()` to accept `renderer: Option<Res<TextRenderer>>` to get cell dimensions (cell_width, cell_height) needed for pixel-to-grid conversion.

### 2. Implemented `cursor_to_scrollback_coords()` Helper Function

Created a comprehensive coordinate conversion function that:

```rust
fn cursor_to_scrollback_coords(
    cursor_pos: Vec2,
    scrollback: &ScrollbackBuffer,
    cell_width: f32,
    cell_height: f32,
    window_width: f32,
    window_height: f32,
) -> Option<(usize, usize)>
```

#### Conversion Algorithm

1. **Calculate grid dimensions in pixels**:
   ```rust
   let grid_pixel_width = GRID_WIDTH as f32 * cell_width;
   let grid_pixel_height = GRID_HEIGHT as f32 * cell_height;
   ```

2. **Determine grid position in window** (grid is centered):
   ```rust
   let grid_start_x = (window_width - grid_pixel_width) / 2.0;
   let grid_start_y = (window_height - grid_pixel_height) / 2.0;
   ```

3. **Convert cursor to grid-relative coordinates**:
   ```rust
   let grid_rel_x = cursor_pos.x - grid_start_x;
   let grid_rel_y = cursor_pos.y - grid_start_y;
   ```

4. **Bounds checking** - Return None if outside grid

5. **Calculate column and visible row**:
   ```rust
   let col = (grid_rel_x / cell_width).floor() as usize;
   let visible_row = (grid_rel_y / cell_height).floor() as usize;
   ```

6. **Convert visible row to scrollback buffer line** (the critical fix):
   ```rust
   let scroll_offset = scrollback.scroll_offset();
   let total_lines = scrollback.line_count();

   let scrollback_line = if scroll_offset > 0 {
       // In scrollback: map visible row to buffer line
       total_lines.saturating_sub(scroll_offset).saturating_add(visible_row)
   } else {
       // At bottom (live view)
       total_lines.saturating_sub(GRID_HEIGHT).saturating_add(visible_row)
   };
   ```

### 3. Scrollback Buffer Line Mapping

The key insight is understanding the scrollback offset model:

- **scroll_offset = 0**: At bottom (live view)
  - Visible row 0 → Most recent line minus grid height

- **scroll_offset = N**: Scrolled up N lines
  - Visible row 0 → Buffer line at (total_lines - scroll_offset)
  - Visible row K → Buffer line at (total_lines - scroll_offset + K)

Example with 1000 total lines, grid height 100, scrolled up 50 lines:
- scroll_offset = 50
- total_lines = 1000
- Visible row 0 maps to buffer line 950 (1000 - 50)
- Visible row 10 maps to buffer line 960 (1000 - 50 + 10)

## Code Changes

### Modified Function Signature

```rust
fn handle_mouse_selection(
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    mut selection: ResMut<ScrollbackSelectionState>,
    scrollback: Res<ScrollbackBuffer>,
    scrollback_state: Res<ScrollbackState>,
    renderer: Option<Res<crate::rendering::text::TextRenderer>>, // NEW
) {
```

### Replaced Placeholder TODOs

**Before:**
```rust
let line = 0; // TODO: Convert cursor_pos to scrollback line
let col = 0; // TODO: Convert cursor_pos to column
```

**After:**
```rust
if let Some((grid_col, grid_row)) = cursor_to_scrollback_coords(
    cursor_pos,
    &scrollback,
    renderer.cell_width,
    renderer.cell_height,
    window.width(),
    window.height(),
) {
    let line = grid_row;
    let col = grid_col as u16;

    selection.start_scrollback_selection(line, col, SelectionMode::Character);
    debug!("Started scrollback selection at line {}, col {}", line, col);
}
```

## Testing

### Manual Testing Steps

1. **Build the client**: `cargo build -p scarab-client`
2. **Run daemon**: `cargo run -p scarab-daemon`
3. **Run client**: `cargo run -p scarab-client`
4. **Generate scrollback history**: Run commands to fill terminal output
5. **Scroll up**: Use Shift+PageUp or mouse wheel to scroll up
6. **Test selection**:
   - Click and drag to select text in scrollback
   - Verify selection highlights correct lines
   - Copy selection with Ctrl+C or 'y'
   - Verify clipboard contains correct text

### Expected Behavior

- Mouse clicks should map to correct line/column in scrollback
- Selection should follow mouse drag accurately
- Selected text should match what's visually highlighted
- Clipboard should contain the exact text selected

## Related Files

- `/home/beengud/raibid-labs/scarab/crates/scarab-client/src/terminal/scrollback.rs` - ScrollbackBuffer implementation
- `/home/beengud/raibid-labs/scarab/crates/scarab-client/src/ui/grid_utils.rs` - Grid coordinate utilities
- `/home/beengud/raibid-labs/scarab/crates/scarab-client/src/rendering/text.rs` - TextRenderer with cell dimensions
- `/home/beengud/raibid-labs/scarab/crates/scarab-client/src/ui/visual_selection.rs` - Base selection system

## Technical Details

### Coordinate Systems

1. **Bevy Window Coordinates**:
   - Origin: Bottom-left
   - X: Right-positive
   - Y: Up-positive

2. **Grid Coordinates**:
   - Origin: Top-left of grid
   - X (column): Right-positive
   - Y (row): Down-positive
   - Grid is centered in window

3. **Scrollback Buffer Indices**:
   - Index 0: Oldest line in buffer
   - Index (line_count - 1): Newest line in buffer
   - Scroll offset adjusts mapping from visible rows to buffer indices

### Safety Considerations

- **Bounds checking**: Returns None for out-of-bounds coordinates
- **Saturating arithmetic**: Uses `saturating_sub()` and `saturating_add()` to prevent underflow/overflow
- **Type conversions**: Properly converts between usize, u16, and f32 types
- **Optional renderer**: Gracefully handles case where TextRenderer isn't available yet

## Performance Impact

- Minimal: Coordinate conversion is O(1) with simple arithmetic operations
- No allocations or complex computations
- Only called on mouse events (not every frame)

## Future Improvements

1. **Add unit tests** for `cursor_to_scrollback_coords()` with various scroll offsets
2. **Add integration tests** that verify end-to-end selection behavior in scrollback
3. **Consider caching** cell dimensions if they become a performance bottleneck
4. **Add visual feedback** when hovering over selectable text in scrollback

## References

- Audit finding: "TODO: Convert cursor_pos to scrollback line" in `scrollback_selection.rs`
- Related to Issue: Scrollback selection coordinate conversion
- Bevy coordinate system: https://bevyengine.org/learn/book/getting-started/
