# Ratatui-testlib v0.5.0 Test Infrastructure

This directory contains test infrastructure prepared for ratatui-testlib v0.5.0 features.

## Overview

These test files implement Issues #168-173, which add comprehensive test coverage using upcoming ratatui-testlib v0.5.0 APIs. All tests are currently marked with `#[ignore]` attributes and include detailed documentation of the expected APIs.

## Test Files

### Issue #168: CellAttributes for Status Bar Color Verification
**File**: `ratatui_testlib_cell_attributes.rs`

Tests for verifying cell-level attributes (foreground/background colors, text styling flags).

**Tests** (10 total):
1. `test_status_bar_background_color` - Verify status bar background color
2. `test_status_bar_foreground_color` - Verify status bar text color
3. `test_bold_text_cell_flags` - Test BOLD flag on cells
4. `test_italic_text_cell_flags` - Test ITALIC flag on cells
5. `test_underline_text_cell_flags` - Test UNDERLINE flag on cells
6. `test_inverse_video_cell_flags` - Test INVERSE (reverse video) flag
7. `test_strikethrough_text_cell_flags` - Test STRIKETHROUGH flag
8. `test_combined_cell_flags` - Test multiple flags (e.g., BOLD + ITALIC)
9. `test_24bit_rgb_colors` - Test 24-bit RGB color support
10. `test_status_bar_updates_on_tab_change` - Verify status bar updates

**Expected APIs**:
- `TuiTestHarness::get_cell_attributes(row, col) -> CellAttributes`
- `TuiTestHarness::assert_cell_fg(row, col, Color) -> Result<()>`
- `TuiTestHarness::assert_cell_bg(row, col, Color) -> Result<()>`
- `CellFlags` bitflags (BOLD, ITALIC, UNDERLINE, etc.)

---

### Issue #169: SeqlockVerifier for Shared Memory Race Detection
**File**: `ratatui_testlib_seqlock_verifier.rs`

Tests for verifying Scarab's seqlock-based shared memory synchronization.

**Tests** (8 total):
1. `test_seqlock_basic_verification` - Basic seqlock verification (no torn reads)
2. `test_seqlock_high_contention` - Test under high update contention
3. `test_seqlock_stress_concurrent_readers` - Stress test with multiple readers
4. `test_seqlock_large_data_structure` - Test torn read detection on full grid
5. `test_seqlock_sequence_always_even` - Verify sequence number parity
6. `test_seqlock_retry_on_torn_read` - Verify automatic retry logic
7. `test_seqlock_lock_free_no_blocking` - Verify lock-free operation
8. `test_seqlock_integration_with_harness` - Integration with TuiTestHarness

**Expected APIs**:
- `SeqlockVerifier::new() -> Self`
- `SeqlockVerifier::verify_read<T, F>(read_fn) -> Result<T>`
- `SeqlockVerifier::torn_reads() -> usize`
- `SeqlockVerifier::assert_no_torn_reads() -> Result<()>`
- `SeqlockVerifier::stress_test(iterations, threads) -> Result<()>`

---

### Issue #170: OSC 133 Zones for Shell Integration Testing
**File**: `ratatui_testlib_osc133_zones.rs`

Tests for verifying OSC 133 shell integration (semantic zones).

**Tests** (10 total):
1. `test_osc133_detect_prompt_zone` - Detect prompt zones
2. `test_osc133_detect_input_zone` - Detect user input zones
3. `test_osc133_detect_output_zone` - Detect command output zones
4. `test_osc133_capture_exit_codes` - Capture exit codes from OSC 133;D
5. `test_osc133_navigate_to_previous_prompt` - Test Ctrl+Up navigation
6. `test_osc133_navigate_to_next_prompt` - Test Ctrl+Down navigation
7. `test_osc133_select_output_zone` - Test zone selection
8. `test_osc133_zone_boundaries` - Verify zone boundary coordinates
9. `test_osc133_multiple_command_zones` - Multiple commands create multiple zones
10. `test_osc133_scarab_navigation_integration` - Integration with Scarab nav system

**Expected APIs**:
- `SemanticZone` struct with zone_type, start/end coordinates, exit_code
- `ZoneType` enum (Prompt, Input, Output)
- `TuiTestHarness::get_semantic_zones() -> Vec<SemanticZone>`
- `TuiTestHarness::find_zones(ZoneType) -> Vec<SemanticZone>`
- `TuiTestHarness::jump_to_zone(&SemanticZone) -> Result<()>`

---

### Issue #171: UiRegionTester for Status/Tab Bar Testing
**File**: `ratatui_testlib_ui_region_tester.rs`

Tests for verifying UI region positioning and content (status bar, tab bar, panes).

**Tests** (10 total):
1. `test_status_bar_region_definition` - Define and verify status bar region
2. `test_status_bar_content` - Verify status bar text content
3. `test_tab_bar_region_definition` - Define and verify tab bar region
4. `test_content_region` - Verify main content region
5. `test_regions_no_overlap` - Verify regions don't overlap incorrectly
6. `test_regions_update_on_resize` - Test region updates on terminal resize
7. `test_pane_regions_with_splits` - Verify pane regions with splits
8. `test_status_bar_updates_on_tab_switch` - Status bar updates on tab switch
9. `test_tab_bar_shows_multiple_tabs` - Tab bar shows multiple tabs
10. `test_overlay_regions` - Verify overlay regions (command palette, etc.)

**Expected APIs**:
- `UiRegionTester::new(harness) -> Self`
- `UiRegionTester::define_region(name, Rect)`
- `UiRegionTester::auto_detect_regions() -> Result<()>`
- `UiRegionTester::assert_text_in_region(region, text) -> Result<()>`
- `UiRegionTester::assert_no_overlap(region1, region2) -> Result<()>`

---

### Issue #172: ColorPalette for Theme Verification
**File**: `ratatui_testlib_color_palette.rs`

Tests for verifying Scarab's theme system and color palettes.

**Tests** (10 total):
1. `test_extract_color_palette` - Extract color palette from terminal
2. `test_slime_theme_ansi_colors` - Verify slime theme ANSI colors (0-15)
3. `test_slime_theme_background_foreground` - Verify slime theme bg/fg
4. `test_ansi_color_rendering` - Verify ANSI color sequences render correctly
5. `test_bright_ansi_colors` - Verify bright ANSI colors (8-15)
6. `test_256_color_palette` - Verify 256-color palette support
7. `test_true_color_support` - Verify 24-bit RGB color support
8. `test_theme_switching_updates_palette` - Theme switching updates palette
9. `test_theme_matches_slime` - Verify palette matches slime theme
10. `test_status_bar_uses_theme_colors` - Status bar uses theme colors

**Expected APIs**:
- `TuiTestHarness::get_color_palette() -> ColorPalette`
- `ColorPalette::get_ansi(index) -> Color`
- `ColorPalette::assert_ansi_color(index, Color) -> Result<()>`
- `ColorPalette::assert_theme_matches(theme_name) -> Result<()>`

---

### Issue #173: TestAuditor to Replace Placeholder Tests
**File**: `ratatui_testlib_test_auditor.rs`

Tests for using TestAuditor to find and replace placeholder/smoke tests.

**Tests** (10 total + 5 example placeholders):
1. `test_scan_workspace_for_placeholders` - Scan workspace for placeholder tests
2. `test_find_always_passing_tests` - Find tests with assert!(true)
3. `test_find_todo_tests` - Find tests with todo!() or unimplemented!()
4. `test_find_empty_tests` - Find empty test bodies
5. `test_find_tests_without_assertions` - Find tests with no assertions
6. `test_find_ignored_tests_without_reason` - Find #[ignore] without reason
7. `test_generate_replacement_template` - Generate replacement templates
8. `test_scan_specific_crate` - Scan specific crate only
9. `test_generate_audit_report` - Generate comprehensive audit report
10. `test_example_of_good_test` - Example of a good test (not placeholder)

**Example Placeholders** (for auditor to find):
- `example_placeholder_always_passes` - Always passes with assert!(true)
- `example_placeholder_empty_body` - Empty test body
- `example_placeholder_todo` - Uses todo!()
- `example_placeholder_no_assertions` - No assertions
- `example_placeholder_ignored_no_reason` - #[ignore] without reason

**Expected APIs**:
- `TestAuditor::new() -> Self`
- `TestAuditor::scan_workspace() -> Result<()>`
- `TestAuditor::find_placeholder_tests() -> &[PlaceholderTest]`
- `TestAuditor::generate_replacement_template(test) -> String`
- `PlaceholderReason` enum (AlwaysPasses, TodoOrUnimplemented, etc.)

---

## Running the Tests

### Current Status (ratatui-testlib v0.1.0)

All tests are currently marked `#[ignore]` because they require ratatui-testlib v0.5.0 APIs that don't exist yet.

To verify tests compile (when cyclic dependency is fixed):
```bash
cargo test --package scarab-client --tests --no-run
```

To list ignored tests:
```bash
cargo test --package scarab-client --tests -- --ignored --list
```

### When ratatui-testlib v0.5.0 is Released

1. **Update dependency**:
   ```toml
   # In crates/scarab-client/Cargo.toml [dev-dependencies]
   ratatui-testlib = { workspace = true, version = "0.5.0", features = ["mvp", "sixel-image"] }
   ```

2. **Remove #[ignore] attributes**:
   - Remove `#[ignore = "..."]` from test functions
   - Or run with `--ignored` flag to run them anyway

3. **Run tests**:
   ```bash
   # Run all ratatui-testlib v0.5.0 tests
   cargo test --package scarab-client --test ratatui_testlib_cell_attributes
   cargo test --package scarab-client --test ratatui_testlib_seqlock_verifier
   cargo test --package scarab-client --test ratatui_testlib_osc133_zones
   cargo test --package scarab-client --test ratatui_testlib_ui_region_tester
   cargo test --package scarab-client --test ratatui_testlib_color_palette
   cargo test --package scarab-client --test ratatui_testlib_test_auditor
   ```

## Implementation Notes

### Test Documentation

Each test file includes:
- Comprehensive doc comments explaining the feature being tested
- Expected API signatures with examples
- Background information on Scarab's architecture
- Clear TODO comments for implementation
- Issue tracking references

### Test Structure

Each test follows the pattern:
1. **Setup**: Create harness, spawn daemon, configure environment
2. **Action**: Execute commands or trigger UI events
3. **Verification**: Use ratatui-testlib APIs to verify behavior
4. **Cleanup**: Handled automatically by harness Drop implementation

### Coverage Goals

When all tests are implemented, we will have coverage for:
- **Cell-level rendering**: Colors, styling flags, attributes
- **Shared memory synchronization**: Seqlock protocol, torn read detection
- **Shell integration**: OSC 133 zones, prompt navigation
- **UI regions**: Status bar, tab bar, panes, overlays
- **Theme system**: Color palettes, ANSI colors, theme switching
- **Test quality**: Placeholder test detection and replacement

## Related Documentation

- `README_RATATUI_TESTLIB.md` - Current ratatui-testlib v0.1.0 smoke tests
- `ratatui_testlib_smoke.rs` - Existing IPC/PTY smoke tests
- `../../../docs/deprecated/audits/claude-2025-12-01/06-RATATUI-TESTLIB-ISSUES.md` - Feature requests

## Tracking

- **Issues**: #168, #169, #170, #171, #172, #173
- **Blocked by**: ratatui-testlib v0.5.0 release
- **Total tests prepared**: 58 tests across 6 test files
- **Status**: Test infrastructure complete, awaiting API availability
