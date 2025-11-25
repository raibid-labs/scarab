# Issue: Critical - UI Overlays use Hardcoded Font Metrics

**Priority:** Critical
**Component:** scarab-client
**Status:** Open

## Description
The UI overlay systems in `scarab-client` (`link_hints.rs`, `visual_selection.rs`) utilize hardcoded pixel values for rendering positioning and sizing.

- `link_hints.rs`: Uses `Vec2::new(100.0, 100.0 + i * 20.0)` for positioning hints.
- `visual_selection.rs`: Uses `cell_width = 8.0` and `cell_height = 16.0`.

## Impact
- Link hints will not appear over the actual text they correspond to.
- Visual selection highlights will be misaligned (too small or too large) for any font size other than the one implicitly hardcoded (likely 8x16).
- The terminal is effectively unusable with custom fonts or DPI scaling.

## Proposed Fix
1. Expose a `FontMetrics` resource in `scarab-client` derived from `cosmic_text::Buffer` or `TextRenderer`.
2. Update `detect_links_system` and `render_selection_system` to inject this resource.
3. Calculate positions as: `x = cell_x * metrics.width`, `y = cell_y * metrics.height`.
