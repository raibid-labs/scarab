# Status Bar and Theme Color Verification Tests - Issue #168

This directory contains comprehensive test coverage for Scarab's color rendering system using ratatui-testlib v0.5.0 CellAttributes API.

## Overview

These tests verify that:
1. Status bar tabs display correct colors (active/inactive states)
2. Slime theme ANSI palette (0-15) is applied correctly
3. 24-bit RGB colors render with full precision
4. Tab switching updates colors dynamically
5. Cell-level styling attributes (BOLD, ITALIC, etc.) work correctly

## Test Files

### 1. `status_bar_color_tests.rs` - Status Bar Color Verification

**Purpose**: Verify that Scarab's status bar renders with correct colors from the slime theme.

**Test Count**: 12 tests (10 integration + 2 helper)

**Key Tests**:
- `test_active_tab_background_color` - Active tab has slime green background (#a8df5a)
- `test_active_tab_foreground_color` - Active tab has dark foreground for contrast (#1e2324)
- `test_inactive_tab_foreground_color` - Inactive tabs have muted green foreground (#c8dba8)
- `test_status_bar_background_color` - Status bar container has dark background
- `test_tab_switch_updates_colors` - Colors update correctly when switching tabs
- `test_24bit_rgb_color_precision` - RGB colors are not quantized to 256-color palette
- `test_full_status_bar_integration` - Comprehensive end-to-end test
- `test_slime_theme_integration` - Theme config loads and applies correctly
- `test_rapid_tab_switching_colors` - Stress test for tab switching

**Expected API Usage**:
```rust
use ratatui_testlib::{CellAttributes, CellFlags, Color, TuiTestHarness};

let harness = TuiTestHarness::new()?;
harness.wait_for_render()?;

// Assert cell background color
harness.assert_cell_bg(row, col, Color::Rgb { r, g, b })?;

// Assert cell foreground color
harness.assert_cell_fg(row, col, Color::Rgb { r, g, b })?;

// Get cell attributes
let attrs = harness.cell_attrs_at(row, col)?;
assert!(attrs.flags.contains(CellFlags::BOLD));
```

### 2. `theme_integration_tests.rs` - Theme Color Palette Verification

**Purpose**: Verify that the slime theme ANSI color palette is applied correctly to terminal output.

**Test Count**: 29 tests (28 integration + 1 helper)

**Key Tests**:

**Standard ANSI Colors (0-7)**:
- `test_ansi_color_black` through `test_ansi_color_white` - Verify all 8 standard colors

**Bright ANSI Colors (8-15)**:
- `test_ansi_bright_black` through `test_ansi_bright_white` - Verify all 8 bright colors

**Background Colors**:
- `test_ansi_background_color` - ANSI background colors work
- `test_ansi_bright_background` - Bright background colors work

**24-bit RGB**:
- `test_24bit_rgb_foreground` - Custom RGB foreground colors
- `test_24bit_rgb_background` - Custom RGB background colors

**Default Colors**:
- `test_default_foreground_color` - Default FG is #e0e0e0
- `test_default_background_color` - Default BG is #1e2324

**Color Reset**:
- `test_color_reset` - `\x1b[0m` restores default colors

**Integration**:
- `test_syntax_highlighting_simulation` - Multiple colors in sequence (like syntax highlighting)

**Expected API Usage**:
```rust
let harness = TuiTestHarness::with_theme("slime")?;

// Send ANSI escape sequence
harness.send_input("\x1b[31mRED TEXT\x1b[0m\n")?;
harness.wait_for_render()?;

// Verify color matches slime theme palette
harness.assert_cell_fg(0, 0, Color::Rgb { r: 205, g: 101, b: 100 })?;
```

## Slime Theme Color Reference

### Default Colors

| Color | Hex | RGB | Usage |
|-------|-----|-----|-------|
| Foreground | #e0e0e0 | (224, 224, 224) | Default text color |
| Background | #1e2324 | (30, 35, 36) | Terminal background |
| Cursor | #a8df5a | (168, 223, 90) | Cursor and active tab |

### ANSI Colors (0-7)

| Color | Hex | RGB | ANSI Code |
|-------|-----|-----|-----------|
| Black | #666666 | (102, 102, 102) | `\x1b[30m` |
| Red | #cd6564 | (205, 101, 100) | `\x1b[31m` |
| Green | #AEC199 | (174, 193, 153) | `\x1b[32m` |
| Yellow | #fff099 | (255, 240, 153) | `\x1b[33m` |
| Blue | #6D9CBE | (109, 156, 190) | `\x1b[34m` |
| Magenta | #B081B9 | (176, 129, 185) | `\x1b[35m` |
| Cyan | #80B5B3 | (128, 181, 179) | `\x1b[36m` |
| White | #efefef | (239, 239, 239) | `\x1b[37m` |

### Bright ANSI Colors (8-15)

| Color | Hex | RGB | ANSI Code |
|-------|-----|-----|-----------|
| Bright Black | #888888 | (136, 136, 136) | `\x1b[90m` |
| Bright Red | #e08080 | (224, 128, 128) | `\x1b[91m` |
| Bright Green | #c8dba8 | (200, 219, 168) | `\x1b[92m` |
| Bright Yellow | #ffffb0 | (255, 255, 176) | `\x1b[93m` |
| Bright Blue | #8bb8d8 | (139, 184, 216) | `\x1b[94m` |
| Bright Magenta | #c9a0d0 | (201, 160, 208) | `\x1b[95m` |
| Bright Cyan | #9fd0ce | (159, 208, 206) | `\x1b[96m` |
| Bright White | #ffffff | (255, 255, 255) | `\x1b[97m` |

### Status Bar Colors (Hardcoded)

| Element | RGB | Calculated From |
|---------|-----|-----------------|
| Active Tab BG | (168, 222, 89) | `Color::srgb(0.66, 0.87, 0.35)` |
| Active Tab FG | (31, 36, 36) | `Color::srgb(0.12, 0.14, 0.14)` |
| Inactive Tab FG | (199, 194, 158) | `Color::srgb(0.78, 0.76, 0.62)` |
| Status Bar BG | (38, 38, 46) | `Color::srgba(0.15, 0.15, 0.18, 0.95)` |

## Test Architecture

These tests use Scarab's actual IPC/shared-memory pipeline:

```
┌─────────────────┐
│ TuiTestHarness  │ (ratatui-testlib)
└────────┬────────┘
         │ Creates
         ▼
┌─────────────────┐
│ scarab-daemon   │ (spawned process)
│                 │
│ ┌─────────────┐ │
│ │ VTE Parser  │ │ Parses ANSI escape sequences
│ └──────┬──────┘ │
│        │        │
│ ┌──────▼──────┐ │
│ │ Grid State  │ │ Updates terminal grid with colors
│ └──────┬──────┘ │
│        │        │
│ ┌──────▼──────┐ │
│ │ Shared Mem  │ │ Writes to /scarab_shm_v1
│ └─────────────┘ │
└─────────────────┘
         │
         │ Zero-copy read
         ▼
┌─────────────────┐
│ Test Harness    │
│ Reads Grid      │ Verifies cell colors via CellAttributes
└─────────────────┘
```

## Current Status

**Blocked**: These tests are awaiting ratatui-testlib v0.5.0 release.

- **Current Version**: ratatui-testlib 0.1.0
- **Required Version**: ratatui-testlib 0.5.0 (not yet released)
- **Test Status**: All tests marked with `#[ignore]` attribute
- **Implementation Status**: Tests are fully implemented with expected API

## Running the Tests

### Current Behavior (v0.1.0)

All tests are currently ignored. To verify they compile:

```bash
# Build tests without running
cargo test --package scarab-client --test status_bar_color_tests --no-run
cargo test --package scarab-client --test theme_integration_tests --no-run

# List ignored tests
cargo test --package scarab-client --test status_bar_color_tests -- --ignored --list
cargo test --package scarab-client --test theme_integration_tests -- --ignored --list
```

### When v0.5.0 is Released

1. **Update dependency** in `Cargo.toml`:
   ```toml
   [workspace.dependencies]
   ratatui-testlib = "0.5.0"
   ```

2. **Remove `#[ignore]` attributes** from tests or run with `--ignored`:
   ```bash
   # Run all tests including ignored ones
   cargo test --package scarab-client --test status_bar_color_tests -- --ignored
   cargo test --package scarab-client --test theme_integration_tests -- --ignored
   ```

3. **Verify all tests pass**:
   ```bash
   # Run normally after removing #[ignore]
   cargo test --package scarab-client --test status_bar_color_tests
   cargo test --package scarab-client --test theme_integration_tests
   ```

## Test Design Principles

### 1. Test Real Behavior, Not Mocks

These tests verify the actual IPC/shared-memory pipeline, not mocked implementations. This provides real coverage of production code paths.

### 2. Document Expected API

Each test includes detailed comments showing the expected ratatui-testlib v0.5.0 API usage. This serves as:
- Documentation for future implementers
- Specification for the API we expect
- Examples for other test authors

### 3. Test Edge Cases

Tests cover:
- All 16 ANSI colors
- 24-bit RGB colors
- Background and foreground colors
- Color resets
- Dynamic color updates (tab switching)
- Stress scenarios (rapid switching)

### 4. Verify Precision

Tests explicitly check for:
- Full 24-bit RGB precision (not quantized to 256-color palette)
- Exact color values matching theme config
- Correct sRGB-to-u8 conversion

## Color Conversion Reference

Scarab uses Bevy's color system with sRGB linear color space:

```rust
// Converting Bevy Color to RGB u8
fn srgb_to_u8(value: f32) -> u8 {
    (value * 255.0).round().clamp(0.0, 255.0) as u8
}

// Example: Active tab background
Color::srgb(0.66, 0.87, 0.35)
// = RGB(168, 222, 89)
// = #a8de59 (approximately #a8df5a from slime theme)
```

## Future Syntax Highlighting Tests

When ratatui-testlib v0.5.0 is released, these tests can be extended to verify:

1. **Code Syntax Highlighting**
   - Rust syntax highlighting in embedded `cargo` output
   - Shell prompt syntax highlighting (zsh, bash)
   - Git diff syntax highlighting

2. **LSP Semantic Tokens**
   - Verify LSP-provided semantic tokens render with correct colors

3. **Tree-sitter Integration**
   - Verify tree-sitter syntax highlighting colors

## Related Files

- `/home/beengud/raibid-labs/scarab/crates/scarab-client/src/ui/status_bar.rs` - Status bar implementation
- `/home/beengud/raibid-labs/scarab/crates/scarab-config/examples/slime-theme.toml` - Slime theme config
- `/home/beengud/raibid-labs/scarab/crates/scarab-client/tests/ratatui_testlib_cell_attributes.rs` - Original placeholder tests
- `/home/beengud/raibid-labs/scarab/crates/scarab-client/tests/README_RATATUI_TESTLIB_V0.5.md` - v0.5.0 test infrastructure overview

## Issue Tracking

- **Issue #168**: Use ratatui-testlib CellAttributes for status bar color verification
- **Blocked by**: ratatui-testlib v0.5.0 release
- **Total Tests**: 41 tests (39 integration + 2 helper)
- **Test Files**: 2 files
  - `status_bar_color_tests.rs` - 12 tests
  - `theme_integration_tests.rs` - 29 tests

## Contributing

When adding new color-related features:

1. Add corresponding tests to verify colors render correctly
2. Document expected colors in test comments
3. Use slime theme colors for consistency
4. Verify tests with both 24-bit RGB and 256-color terminals
5. Test color updates during dynamic scenarios (tab switching, theme changing)

## Questions?

See `/home/beengud/raibid-labs/scarab/crates/scarab-client/tests/README_RATATUI_TESTLIB_V0.5.md` for broader test infrastructure documentation.
