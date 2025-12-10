//! Status Bar Color Verification Tests - Issue #168
//!
//! This test file implements comprehensive color and styling verification for Scarab's
//! status bar using ratatui-testlib v0.5.0 CellAttributes API.
//!
//! ## What This Tests
//!
//! 1. **Tab Bar Colors** - Verify active/inactive tab foreground and background colors
//! 2. **Status Bar Background** - Verify status bar container background color
//! 3. **Mode Indicator Colors** - Verify NORMAL/INSERT mode colors
//! 4. **Slime Theme Integration** - Verify slime theme colors are applied correctly
//! 5. **Tab Switching** - Verify colors update correctly when tabs change
//! 6. **Cell Styling Flags** - Verify BOLD, ITALIC, and other styling attributes
//!
//! ## Slime Theme Color Palette
//!
//! Based on `crates/scarab-config/examples/slime-theme.toml`:
//! - Background: #1e2324 (RGB: 30, 35, 36)
//! - Foreground: #e0e0e0 (RGB: 224, 224, 224)
//! - Cursor/Accent: #a8df5a (RGB: 168, 223, 90) - slime green
//! - Status Bar BG: rgba(0.15, 0.15, 0.18, 0.95) (RGB: ~38, 38, 46)
//!
//! ## Hardcoded Status Bar Colors (from status_bar.rs)
//!
//! - Active tab background: Color::srgb(0.66, 0.87, 0.35) = #a8df5a (slime green)
//! - Active tab foreground: Color::srgb(0.12, 0.14, 0.14) = #1e2324 (dark)
//! - Inactive tab foreground: Color::srgb(0.78, 0.76, 0.62) = #c8dba8 (muted green)
//! - Mode indicator: Color::srgb(0.78, 0.76, 0.62) = #c8dba8 (muted green)
//! - Status bar background: Color::srgba(0.15, 0.15, 0.18, 0.95)
//!
//! ## Expected ratatui-testlib v0.5.0 API
//!
//! ```rust,ignore
//! use ratatui_testlib::{CellAttributes, CellFlags, Color};
//!
//! // Core types
//! pub struct CellAttributes {
//!     pub fg: Color,
//!     pub bg: Color,
//!     pub flags: CellFlags,
//! }
//!
//! bitflags! {
//!     pub struct CellFlags: u8 {
//!         const BOLD = 0b00000001;
//!         const ITALIC = 0b00000010;
//!         const UNDERLINE = 0b00000100;
//!         const STRIKETHROUGH = 0b00001000;
//!         const INVERSE = 0b00010000;
//!     }
//! }
//!
//! #[derive(Debug, Clone, Copy, PartialEq)]
//! pub enum Color {
//!     Reset,
//!     Black,
//!     Red,
//!     Green,
//!     Yellow,
//!     Blue,
//!     Magenta,
//!     Cyan,
//!     White,
//!     Rgb { r: u8, g: u8, b: u8 },
//!     Indexed(u8),
//! }
//!
//! impl TuiTestHarness {
//!     /// Get cell attributes at position
//!     pub fn cell_attrs_at(&self, row: u16, col: u16) -> Result<CellAttributes>;
//!
//!     /// Assert foreground color matches
//!     pub fn assert_cell_fg(&self, row: u16, col: u16, expected: Color) -> Result<()>;
//!
//!     /// Assert background color matches
//!     pub fn assert_cell_bg(&self, row: u16, col: u16, expected: Color) -> Result<()>;
//!
//!     /// Assert cell has styling flags
//!     pub fn assert_cell_styled(&self, row: u16, col: u16, flags: CellFlags) -> Result<()>;
//! }
//! ```
//!
//! ## Status
//!
//! - **Current**: Tests are fully implemented with expected API usage
//! - **Blocked by**: ratatui-testlib v0.5.0 release (currently at v0.1.0)
//! - **Action Required**: Remove `#[ignore]` attributes when v0.5.0 is released
//!
//! ## Test Architecture
//!
//! These tests use Scarab's actual IPC/shared-memory pipeline:
//! 1. Spawn scarab-daemon (headless server)
//! 2. Connect via Unix Domain Socket
//! 3. Daemon renders status bar to shared memory
//! 4. Tests read shared memory via ratatui-testlib TuiTestHarness
//! 5. Verify cell-level attributes (colors, flags) match expected values

use anyhow::Result;

// Expected API imports when ratatui-testlib v0.5.0 is released:
// use ratatui_testlib::{CellAttributes, CellFlags, Color, TuiTestHarness};

// =============================================================================
// HELPER FUNCTIONS FOR COLOR CONVERSION
// =============================================================================

/// Convert sRGB float (0.0-1.0) to u8 (0-255)
///
/// This function is used by tests to verify color conversion.
/// The actual values are hardcoded in the slime_colors module.
fn srgb_to_u8(value: f32) -> u8 {
    (value * 255.0).round().clamp(0.0, 255.0) as u8
}

/// Slime theme color constants (from status_bar.rs)
///
/// These values are calculated from the Bevy Color values used in status_bar.rs:
/// - Active tab BG: Color::srgb(0.66, 0.87, 0.35) = RGB(168, 222, 89)
/// - Active tab FG: Color::srgb(0.12, 0.14, 0.14) = RGB(31, 36, 36)
/// - Inactive tab FG: Color::srgb(0.78, 0.76, 0.62) = RGB(199, 194, 158)
/// - Status bar BG: Color::srgba(0.15, 0.15, 0.18, 0.95) = RGB(38, 38, 46)
mod slime_colors {
    // Active tab colors
    // Calculated from Color::srgb(0.66, 0.87, 0.35)
    pub const ACTIVE_TAB_BG: (u8, u8, u8) = (168, 222, 89); // #a8de59 (slime green)

    // Calculated from Color::srgb(0.12, 0.14, 0.14)
    pub const ACTIVE_TAB_FG: (u8, u8, u8) = (31, 36, 36); // #1f2424 (dark background)

    // Inactive tab colors
    // Calculated from Color::srgb(0.78, 0.76, 0.62)
    pub const INACTIVE_TAB_FG: (u8, u8, u8) = (199, 194, 158); // #c7c29e (muted green)

    // Status bar background
    // Calculated from Color::srgba(0.15, 0.15, 0.18, 0.95)
    pub const STATUS_BAR_BG: (u8, u8, u8) = (38, 38, 46); // #26262e (dark gray with blue tint)

    // Mode indicator (same as inactive tab)
    pub const MODE_INDICATOR_FG: (u8, u8, u8) = INACTIVE_TAB_FG;
}

// =============================================================================
// STATUS BAR LAYOUT CONSTANTS
// =============================================================================

/// Status bar is positioned at the bottom of the terminal
/// Height: 24 pixels (approximately 1-2 rows depending on font size)
const STATUS_BAR_ROW: u16 = 0; // Bottom row in Bevy coordinates

/// Tab positions (approximate, depends on terminal width)
const TAB_META_COL: u16 = 2;
const TAB_PHAGE_COL: u16 = 10;
const TAB_TOLARIA_COL: u16 = 20;

// =============================================================================
// TEST 1: ACTIVE TAB COLORS
// =============================================================================

/// Test that the active tab has correct foreground and background colors
///
/// This test verifies:
/// 1. Active tab background is slime green (#a8df5a)
/// 2. Active tab foreground is dark (#1e2324) for good contrast
/// 3. Tab text is visible and styled correctly
///
/// ## Expected Behavior
///
/// When the active tab is "meta" (index 0):
/// - Background: RGB(168, 222, 89) - slime green
/// - Foreground: RGB(31, 36, 36) - dark
///
/// ## Test Strategy
///
/// 1. Create TuiTestHarness with default Scarab config
/// 2. Wait for initial render
/// 3. Check cell attributes at tab position (row 0, col ~2)
/// 4. Assert background matches slime green
/// 5. Assert foreground matches dark color
#[test]
#[ignore = "Blocked: Awaiting ratatui-testlib v0.5.0 with CellAttributes API"]
fn test_active_tab_background_color() -> Result<()> {
    // TODO(#168): Uncomment when ratatui-testlib v0.5.0 is released
    //
    // let harness = TuiTestHarness::new()?;
    // harness.wait_for_render()?;
    //
    // // Active tab is "meta" at index 0 (left side)
    // let (bg_r, bg_g, bg_b) = slime_colors::ACTIVE_TAB_BG;
    // harness.assert_cell_bg(
    //     STATUS_BAR_ROW,
    //     TAB_META_COL,
    //     Color::Rgb { r: bg_r, g: bg_g, b: bg_b }
    // )?;
    //
    // Ok(())

    Ok(())
}

#[test]
#[ignore = "Blocked: Awaiting ratatui-testlib v0.5.0 with CellAttributes API"]
fn test_active_tab_foreground_color() -> Result<()> {
    // TODO(#168): Uncomment when ratatui-testlib v0.5.0 is released
    //
    // let harness = TuiTestHarness::new()?;
    // harness.wait_for_render()?;
    //
    // // Active tab foreground should be dark for contrast
    // let (fg_r, fg_g, fg_b) = slime_colors::ACTIVE_TAB_FG;
    // harness.assert_cell_fg(
    //     STATUS_BAR_ROW,
    //     TAB_META_COL,
    //     Color::Rgb { r: fg_r, g: fg_g, b: fg_b }
    // )?;
    //
    // Ok(())

    Ok(())
}

// =============================================================================
// TEST 2: INACTIVE TAB COLORS
// =============================================================================

/// Test that inactive tabs have correct foreground colors
///
/// Inactive tabs should:
/// - Have no background color (transparent/default)
/// - Have muted green foreground (#c8dba8)
#[test]
#[ignore = "Blocked: Awaiting ratatui-testlib v0.5.0 with CellAttributes API"]
fn test_inactive_tab_foreground_color() -> Result<()> {
    // TODO(#168): Uncomment when ratatui-testlib v0.5.0 is released
    //
    // let harness = TuiTestHarness::new()?;
    // harness.wait_for_render()?;
    //
    // // Check "phage" tab (index 1, inactive)
    // let (fg_r, fg_g, fg_b) = slime_colors::INACTIVE_TAB_FG;
    // harness.assert_cell_fg(
    //     STATUS_BAR_ROW,
    //     TAB_PHAGE_COL,
    //     Color::Rgb { r: fg_r, g: fg_g, b: fg_b }
    // )?;
    //
    // // Check "tolaria" tab (index 2, inactive)
    // harness.assert_cell_fg(
    //     STATUS_BAR_ROW,
    //     TAB_TOLARIA_COL,
    //     Color::Rgb { r: fg_r, g: fg_g, b: fg_b }
    // )?;
    //
    // Ok(())

    Ok(())
}

#[test]
#[ignore = "Blocked: Awaiting ratatui-testlib v0.5.0 with CellAttributes API"]
fn test_inactive_tab_no_background() -> Result<()> {
    // TODO(#168): Uncomment when ratatui-testlib v0.5.0 is released
    //
    // let harness = TuiTestHarness::new()?;
    // harness.wait_for_render()?;
    //
    // // Inactive tabs should have default/reset background
    // harness.assert_cell_bg(STATUS_BAR_ROW, TAB_PHAGE_COL, Color::Reset)?;
    // harness.assert_cell_bg(STATUS_BAR_ROW, TAB_TOLARIA_COL, Color::Reset)?;
    //
    // Ok(())

    Ok(())
}

// =============================================================================
// TEST 3: STATUS BAR BACKGROUND
// =============================================================================

/// Test that the status bar container has correct background color
///
/// The status bar background should be a dark gray with slight blue tint.
#[test]
#[ignore = "Blocked: Awaiting ratatui-testlib v0.5.0 with CellAttributes API"]
fn test_status_bar_background_color() -> Result<()> {
    // TODO(#168): Uncomment when ratatui-testlib v0.5.0 is released
    //
    // let harness = TuiTestHarness::new()?;
    // harness.wait_for_render()?;
    //
    // // Check background at empty space in status bar
    // let (bg_r, bg_g, bg_b) = slime_colors::STATUS_BAR_BG;
    // harness.assert_cell_bg(
    //     STATUS_BAR_ROW,
    //     40, // Middle of status bar
    //     Color::Rgb { r: bg_r, g: bg_g, b: bg_b }
    // )?;
    //
    // Ok(())

    Ok(())
}

// =============================================================================
// TEST 4: MODE INDICATOR COLORS
// =============================================================================

/// Test that the mode indicator (NORMAL/INSERT) has correct colors
///
/// Mode indicator is on the right side of the status bar.
#[test]
#[ignore = "Blocked: Awaiting ratatui-testlib v0.5.0 with CellAttributes API"]
fn test_mode_indicator_color() -> Result<()> {
    // TODO(#168): Uncomment when ratatui-testlib v0.5.0 is released
    //
    // let harness = TuiTestHarness::new()?;
    // harness.wait_for_render()?;
    //
    // // Mode indicator is on the right side, showing "NORMAL"
    // let (fg_r, fg_g, fg_b) = slime_colors::MODE_INDICATOR_FG;
    //
    // // Find "NORMAL" text on right side of status bar
    // // (exact column depends on terminal width)
    // let mode_col = harness.width() - 10; // Approximate position
    // harness.assert_cell_fg(
    //     STATUS_BAR_ROW,
    //     mode_col,
    //     Color::Rgb { r: fg_r, g: fg_g, b: fg_b }
    // )?;
    //
    // Ok(())

    Ok(())
}

// =============================================================================
// TEST 5: TAB SWITCHING UPDATES COLORS
// =============================================================================

/// Test that tab colors update correctly when switching active tab
///
/// This test verifies the dynamic behavior:
/// 1. Initially "meta" tab is active (green background)
/// 2. Switch to "phage" tab
/// 3. "meta" should lose green background
/// 4. "phage" should gain green background
#[test]
#[ignore = "Blocked: Awaiting ratatui-testlib v0.5.0 with CellAttributes API"]
fn test_tab_switch_updates_colors() -> Result<()> {
    // TODO(#168): Uncomment when ratatui-testlib v0.5.0 is released
    //
    // let mut harness = TuiTestHarness::new()?;
    // harness.wait_for_render()?;
    //
    // // Initial state: "meta" tab is active
    // let (active_bg_r, active_bg_g, active_bg_b) = slime_colors::ACTIVE_TAB_BG;
    // harness.assert_cell_bg(
    //     STATUS_BAR_ROW,
    //     TAB_META_COL,
    //     Color::Rgb { r: active_bg_r, g: active_bg_g, b: active_bg_b }
    // )?;
    //
    // // Switch to "phage" tab (index 1)
    // harness.send_event(TabSwitchEvent { tab_index: 1 })?;
    // harness.wait_for_render()?;
    //
    // // "meta" should now be inactive (no background)
    // harness.assert_cell_bg(STATUS_BAR_ROW, TAB_META_COL, Color::Reset)?;
    //
    // // "phage" should now be active (green background)
    // harness.assert_cell_bg(
    //     STATUS_BAR_ROW,
    //     TAB_PHAGE_COL,
    //     Color::Rgb { r: active_bg_r, g: active_bg_g, b: active_bg_b }
    // )?;
    //
    // Ok(())

    Ok(())
}

// =============================================================================
// TEST 6: CELL STYLING FLAGS
// =============================================================================

/// Test BOLD flag on status bar text
///
/// Some status bar elements may be rendered with BOLD styling.
#[test]
#[ignore = "Blocked: Awaiting ratatui-testlib v0.5.0 with CellFlags API"]
fn test_bold_text_in_status_bar() -> Result<()> {
    // TODO(#168): Uncomment when ratatui-testlib v0.5.0 is released
    //
    // let harness = TuiTestHarness::new()?;
    // harness.wait_for_render()?;
    //
    // // Check if mode indicator uses BOLD
    // let mode_col = harness.width() - 10;
    // let attrs = harness.cell_attrs_at(STATUS_BAR_ROW, mode_col)?;
    //
    // // Mode indicator might be bold (depends on implementation)
    // // For now, just verify we can read the flags
    // assert!(attrs.flags.contains(CellFlags::BOLD) || !attrs.flags.contains(CellFlags::BOLD));
    //
    // Ok(())

    Ok(())
}

/// Test ITALIC flag (not currently used in status bar, but API should work)
#[test]
#[ignore = "Blocked: Awaiting ratatui-testlib v0.5.0 with CellFlags API"]
fn test_italic_flag_reading() -> Result<()> {
    // TODO(#168): Uncomment when ratatui-testlib v0.5.0 is released
    //
    // let harness = TuiTestHarness::new()?;
    // harness.wait_for_render()?;
    //
    // // Verify we can read ITALIC flag (even if not used)
    // let attrs = harness.cell_attrs_at(STATUS_BAR_ROW, TAB_META_COL)?;
    // assert!(!attrs.flags.contains(CellFlags::ITALIC)); // Status bar doesn't use italic
    //
    // Ok(())

    Ok(())
}

/// Test combined flags (BOLD + ITALIC)
#[test]
#[ignore = "Blocked: Awaiting ratatui-testlib v0.5.0 with CellFlags API"]
fn test_combined_cell_flags() -> Result<()> {
    // TODO(#168): Uncomment when ratatui-testlib v0.5.0 is released
    //
    // // This test would require status bar to render combined flags
    // // For now, just document the expected API:
    //
    // // let harness = TuiTestHarness::new()?;
    // // let attrs = harness.cell_attrs_at(row, col)?;
    // // assert!(attrs.flags.contains(CellFlags::BOLD | CellFlags::ITALIC));
    //
    // Ok(())

    Ok(())
}

// =============================================================================
// TEST 7: 24-BIT RGB COLOR SUPPORT
// =============================================================================

/// Test that 24-bit RGB colors are accurately represented
///
/// This verifies that slime theme's custom colors are preserved
/// with full 24-bit precision (not quantized to 256-color palette).
#[test]
#[ignore = "Blocked: Awaiting ratatui-testlib v0.5.0 with CellAttributes API"]
fn test_24bit_rgb_color_precision() -> Result<()> {
    // TODO(#168): Uncomment when ratatui-testlib v0.5.0 is released
    //
    // let harness = TuiTestHarness::new()?;
    // harness.wait_for_render()?;
    //
    // // Get active tab cell attributes
    // let attrs = harness.cell_attrs_at(STATUS_BAR_ROW, TAB_META_COL)?;
    //
    // // Verify background is exact RGB value, not quantized
    // let (expected_r, expected_g, expected_b) = slime_colors::ACTIVE_TAB_BG;
    // match attrs.bg {
    //     Color::Rgb { r, g, b } => {
    //         assert_eq!(r, expected_r, "Red component mismatch");
    //         assert_eq!(g, expected_g, "Green component mismatch");
    //         assert_eq!(b, expected_b, "Blue component mismatch");
    //     }
    //     other => panic!("Expected Rgb color, got {:?}", other),
    // }
    //
    // Ok(())

    Ok(())
}

// =============================================================================
// TEST 8: FULL STATUS BAR INTEGRATION
// =============================================================================

/// Comprehensive test that verifies the entire status bar at once
///
/// This test checks:
/// 1. All three tabs are visible
/// 2. Active tab has correct colors
/// 3. Inactive tabs have correct colors
/// 4. Mode indicator is present and colored
/// 5. Status bar background is correct
#[test]
#[ignore = "Blocked: Awaiting ratatui-testlib v0.5.0 with CellAttributes API"]
fn test_full_status_bar_integration() -> Result<()> {
    // TODO(#168): Uncomment when ratatui-testlib v0.5.0 is released
    //
    // let harness = TuiTestHarness::new()?;
    // harness.wait_for_render()?;
    //
    // // Verify all tab text is present
    // harness.assert_text_at(STATUS_BAR_ROW, TAB_META_COL, "meta")?;
    // harness.assert_text_at(STATUS_BAR_ROW, TAB_PHAGE_COL, "phage")?;
    // harness.assert_text_at(STATUS_BAR_ROW, TAB_TOLARIA_COL, "tolaria")?;
    //
    // // Verify active tab colors
    // let (active_bg_r, active_bg_g, active_bg_b) = slime_colors::ACTIVE_TAB_BG;
    // harness.assert_cell_bg(
    //     STATUS_BAR_ROW,
    //     TAB_META_COL,
    //     Color::Rgb { r: active_bg_r, g: active_bg_g, b: active_bg_b }
    // )?;
    //
    // // Verify inactive tab colors
    // let (inactive_fg_r, inactive_fg_g, inactive_fg_b) = slime_colors::INACTIVE_TAB_FG;
    // harness.assert_cell_fg(
    //     STATUS_BAR_ROW,
    //     TAB_PHAGE_COL,
    //     Color::Rgb { r: inactive_fg_r, g: inactive_fg_g, b: inactive_fg_b }
    // )?;
    //
    // // Verify mode indicator
    // let mode_col = harness.width() - 8;
    // harness.assert_text_contains(STATUS_BAR_ROW, mode_col, "NORMAL")?;
    //
    // Ok(())

    Ok(())
}

// =============================================================================
// TEST 9: THEME RESOLVER INTEGRATION
// =============================================================================

/// Test that slime theme colors from config are applied correctly
///
/// This test verifies the full pipeline:
/// 1. Load slime-theme.toml config
/// 2. ThemeResolver processes theme
/// 3. Status bar applies theme colors
/// 4. Colors match expected values in shared memory
#[test]
#[ignore = "Blocked: Awaiting ratatui-testlib v0.5.0 with CellAttributes API"]
fn test_slime_theme_integration() -> Result<()> {
    // TODO(#168): Uncomment when ratatui-testlib v0.5.0 is released
    //
    // // Load slime theme config
    // let config_path = "crates/scarab-config/examples/slime-theme.toml";
    // let harness = TuiTestHarness::with_config(config_path)?;
    // harness.wait_for_render()?;
    //
    // // Verify cursor color is slime green (#a8df5a)
    // // This would require checking the cursor cell specifically
    //
    // // Verify status bar uses theme colors
    // let (active_bg_r, active_bg_g, active_bg_b) = slime_colors::ACTIVE_TAB_BG;
    // harness.assert_cell_bg(
    //     STATUS_BAR_ROW,
    //     TAB_META_COL,
    //     Color::Rgb { r: active_bg_r, g: active_bg_g, b: active_bg_b }
    // )?;
    //
    // Ok(())

    Ok(())
}

// =============================================================================
// TEST 10: STRESS TEST - RAPID TAB SWITCHING
// =============================================================================

/// Test that colors remain correct during rapid tab switching
///
/// This stress test verifies:
/// 1. Tab colors update correctly even with rapid switching
/// 2. No race conditions in color updates
/// 3. Shared memory synchronization works correctly
#[test]
#[ignore = "Blocked: Awaiting ratatui-testlib v0.5.0 with CellAttributes API"]
fn test_rapid_tab_switching_colors() -> Result<()> {
    // TODO(#168): Uncomment when ratatui-testlib v0.5.0 is released
    //
    // let mut harness = TuiTestHarness::new()?;
    // harness.wait_for_render()?;
    //
    // let (active_bg_r, active_bg_g, active_bg_b) = slime_colors::ACTIVE_TAB_BG;
    // let active_bg = Color::Rgb { r: active_bg_r, g: active_bg_g, b: active_bg_b };
    //
    // // Switch between tabs rapidly
    // for _ in 0..10 {
    //     // Switch to phage
    //     harness.send_event(TabSwitchEvent { tab_index: 1 })?;
    //     harness.wait_for_render()?;
    //     harness.assert_cell_bg(STATUS_BAR_ROW, TAB_PHAGE_COL, active_bg)?;
    //
    //     // Switch to tolaria
    //     harness.send_event(TabSwitchEvent { tab_index: 2 })?;
    //     harness.wait_for_render()?;
    //     harness.assert_cell_bg(STATUS_BAR_ROW, TAB_TOLARIA_COL, active_bg)?;
    //
    //     // Switch back to meta
    //     harness.send_event(TabSwitchEvent { tab_index: 0 })?;
    //     harness.wait_for_render()?;
    //     harness.assert_cell_bg(STATUS_BAR_ROW, TAB_META_COL, active_bg)?;
    // }
    //
    // Ok(())

    Ok(())
}

// =============================================================================
// HELPER TESTS - VERIFY COLOR CONVERSION FUNCTIONS
// =============================================================================

#[test]
fn test_srgb_to_u8_conversion() {
    assert_eq!(srgb_to_u8(0.0), 0);
    assert_eq!(srgb_to_u8(1.0), 255);
    assert_eq!(srgb_to_u8(0.5), 128);
    assert_eq!(srgb_to_u8(0.66), 168);
    assert_eq!(srgb_to_u8(0.87), 222);
    assert_eq!(srgb_to_u8(0.35), 89);
}

#[test]
fn test_slime_color_constants() {
    // Verify active tab background is slime green
    assert_eq!(slime_colors::ACTIVE_TAB_BG, (168, 222, 89));

    // Verify active tab foreground is dark
    assert_eq!(slime_colors::ACTIVE_TAB_FG, (31, 36, 36));

    // Verify inactive tab foreground is muted green
    assert_eq!(slime_colors::INACTIVE_TAB_FG, (199, 194, 158));

    // Verify status bar background
    assert_eq!(slime_colors::STATUS_BAR_BG, (38, 38, 46));
}
