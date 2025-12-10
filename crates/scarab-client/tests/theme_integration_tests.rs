//! Theme Integration Tests - Issue #168
//!
//! This test file verifies that Scarab's theme system correctly applies colors
//! to the terminal emulator, with focus on the slime theme color palette.
//!
//! ## What This Tests
//!
//! 1. **ANSI Colors (0-15)** - Verify standard and bright ANSI colors
//! 2. **256-Color Palette** - Verify extended color palette support
//! 3. **24-bit RGB Colors** - Verify true color support
//! 4. **Theme Switching** - Verify colors update when theme changes
//! 5. **Default Colors** - Verify foreground/background defaults
//! 6. **Syntax Highlighting** - Verify colors in actual terminal output
//!
//! ## Slime Theme ANSI Palette (from slime-theme.toml)
//!
//! Standard colors (0-7):
//! - Black (0): #666666 (RGB: 102, 102, 102)
//! - Red (1): #cd6564 (RGB: 205, 101, 100)
//! - Green (2): #AEC199 (RGB: 174, 193, 153)
//! - Yellow (3): #fff099 (RGB: 255, 240, 153)
//! - Blue (4): #6D9CBE (RGB: 109, 156, 190)
//! - Magenta (5): #B081B9 (RGB: 176, 129, 185)
//! - Cyan (6): #80B5B3 (RGB: 128, 181, 179)
//! - White (7): #efefef (RGB: 239, 239, 239)
//!
//! Bright colors (8-15):
//! - Bright Black (8): #888888 (RGB: 136, 136, 136)
//! - Bright Red (9): #e08080 (RGB: 224, 128, 128)
//! - Bright Green (10): #c8dba8 (RGB: 200, 219, 168)
//! - Bright Yellow (11): #ffffb0 (RGB: 255, 255, 176)
//! - Bright Blue (12): #8bb8d8 (RGB: 139, 184, 216)
//! - Bright Magenta (13): #c9a0d0 (RGB: 201, 160, 208)
//! - Bright Cyan (14): #9fd0ce (RGB: 159, 208, 206)
//! - Bright White (15): #ffffff (RGB: 255, 255, 255)
//!
//! ## Default Colors
//!
//! - Foreground: #e0e0e0 (RGB: 224, 224, 224)
//! - Background: #1e2324 (RGB: 30, 35, 36)
//! - Cursor: #a8df5a (RGB: 168, 223, 90) - slime green
//!
//! ## Test Architecture
//!
//! These tests use Scarab's actual rendering pipeline:
//! 1. Load theme configuration (slime-theme.toml)
//! 2. Spawn daemon with theme config
//! 3. Send ANSI escape sequences via IPC
//! 4. Read rendered output from shared memory
//! 5. Verify cell colors match theme palette

use anyhow::Result;

// Expected API imports when ratatui-testlib v0.5.0 is released:
// use ratatui_testlib::{CellAttributes, Color, TuiTestHarness};

// =============================================================================
// SLIME THEME COLOR PALETTE
// =============================================================================

mod slime_palette {
    /// Parse hex color string to RGB tuple (placeholder for future implementation)
    ///
    /// Note: All color values are hardcoded below from slime-theme.toml
    #[allow(dead_code)]
    const fn hex_to_rgb(_hex: &str) -> (u8, u8, u8) {
        // This would be a const fn in real implementation
        // For now, all values are hardcoded below
        (0, 0, 0)
    }

    // Standard ANSI colors (0-7)
    pub const BLACK: (u8, u8, u8) = (102, 102, 102); // #666666
    pub const RED: (u8, u8, u8) = (205, 101, 100); // #cd6564
    pub const GREEN: (u8, u8, u8) = (174, 193, 153); // #AEC199
    pub const YELLOW: (u8, u8, u8) = (255, 240, 153); // #fff099
    pub const BLUE: (u8, u8, u8) = (109, 156, 190); // #6D9CBE
    pub const MAGENTA: (u8, u8, u8) = (176, 129, 185); // #B081B9
    pub const CYAN: (u8, u8, u8) = (128, 181, 179); // #80B5B3
    pub const WHITE: (u8, u8, u8) = (239, 239, 239); // #efefef

    // Bright ANSI colors (8-15)
    pub const BRIGHT_BLACK: (u8, u8, u8) = (136, 136, 136); // #888888
    pub const BRIGHT_RED: (u8, u8, u8) = (224, 128, 128); // #e08080
    pub const BRIGHT_GREEN: (u8, u8, u8) = (200, 219, 168); // #c8dba8
    pub const BRIGHT_YELLOW: (u8, u8, u8) = (255, 255, 176); // #ffffb0
    pub const BRIGHT_BLUE: (u8, u8, u8) = (139, 184, 216); // #8bb8d8
    pub const BRIGHT_MAGENTA: (u8, u8, u8) = (201, 160, 208); // #c9a0d0
    pub const BRIGHT_CYAN: (u8, u8, u8) = (159, 208, 206); // #9fd0ce
    pub const BRIGHT_WHITE: (u8, u8, u8) = (255, 255, 255); // #ffffff

    // Default colors
    pub const FOREGROUND: (u8, u8, u8) = (224, 224, 224); // #e0e0e0
    pub const BACKGROUND: (u8, u8, u8) = (30, 35, 36); // #1e2324
    pub const CURSOR: (u8, u8, u8) = (168, 223, 90); // #a8df5a
}

// =============================================================================
// TEST 1: STANDARD ANSI COLORS (0-7)
// =============================================================================

/// Test that standard ANSI color 0 (black) renders correctly
#[test]
#[ignore = "Blocked: Awaiting ratatui-testlib v0.5.0 with CellAttributes API"]
fn test_ansi_color_black() -> Result<()> {
    // TODO(#168): Uncomment when ratatui-testlib v0.5.0 is released
    //
    // let harness = TuiTestHarness::with_theme("slime")?;
    //
    // // Send text with ANSI black foreground: \x1b[30mTEXT\x1b[0m
    // harness.send_input("\x1b[30mBLACK\x1b[0m\n")?;
    // harness.wait_for_render()?;
    //
    // // Verify first character 'B' has black foreground
    // let (r, g, b) = slime_palette::BLACK;
    // harness.assert_cell_fg(0, 0, Color::Rgb { r, g, b })?;
    //
    // Ok(())

    Ok(())
}

/// Test ANSI red (color 1)
#[test]
#[ignore = "Blocked: Awaiting ratatui-testlib v0.5.0 with CellAttributes API"]
fn test_ansi_color_red() -> Result<()> {
    // TODO(#168): Uncomment when ratatui-testlib v0.5.0 is released
    //
    // let harness = TuiTestHarness::with_theme("slime")?;
    // harness.send_input("\x1b[31mRED\x1b[0m\n")?;
    // harness.wait_for_render()?;
    //
    // let (r, g, b) = slime_palette::RED;
    // harness.assert_cell_fg(0, 0, Color::Rgb { r, g, b })?;
    //
    // Ok(())

    Ok(())
}

/// Test ANSI green (color 2)
#[test]
#[ignore = "Blocked: Awaiting ratatui-testlib v0.5.0 with CellAttributes API"]
fn test_ansi_color_green() -> Result<()> {
    // TODO(#168): Uncomment when ratatui-testlib v0.5.0 is released
    //
    // let harness = TuiTestHarness::with_theme("slime")?;
    // harness.send_input("\x1b[32mGREEN\x1b[0m\n")?;
    // harness.wait_for_render()?;
    //
    // let (r, g, b) = slime_palette::GREEN;
    // harness.assert_cell_fg(0, 0, Color::Rgb { r, g, b })?;
    //
    // Ok(())

    Ok(())
}

/// Test ANSI yellow (color 3)
#[test]
#[ignore = "Blocked: Awaiting ratatui-testlib v0.5.0 with CellAttributes API"]
fn test_ansi_color_yellow() -> Result<()> {
    // TODO(#168): Uncomment when ratatui-testlib v0.5.0 is released
    //
    // let harness = TuiTestHarness::with_theme("slime")?;
    // harness.send_input("\x1b[33mYELLOW\x1b[0m\n")?;
    // harness.wait_for_render()?;
    //
    // let (r, g, b) = slime_palette::YELLOW;
    // harness.assert_cell_fg(0, 0, Color::Rgb { r, g, b })?;
    //
    // Ok(())

    Ok(())
}

/// Test ANSI blue (color 4)
#[test]
#[ignore = "Blocked: Awaiting ratatui-testlib v0.5.0 with CellAttributes API"]
fn test_ansi_color_blue() -> Result<()> {
    // TODO(#168): Uncomment when ratatui-testlib v0.5.0 is released
    //
    // let harness = TuiTestHarness::with_theme("slime")?;
    // harness.send_input("\x1b[34mBLUE\x1b[0m\n")?;
    // harness.wait_for_render()?;
    //
    // let (r, g, b) = slime_palette::BLUE;
    // harness.assert_cell_fg(0, 0, Color::Rgb { r, g, b })?;
    //
    // Ok(())

    Ok(())
}

/// Test ANSI magenta (color 5)
#[test]
#[ignore = "Blocked: Awaiting ratatui-testlib v0.5.0 with CellAttributes API"]
fn test_ansi_color_magenta() -> Result<()> {
    // TODO(#168): Uncomment when ratatui-testlib v0.5.0 is released
    //
    // let harness = TuiTestHarness::with_theme("slime")?;
    // harness.send_input("\x1b[35mMAGENTA\x1b[0m\n")?;
    // harness.wait_for_render()?;
    //
    // let (r, g, b) = slime_palette::MAGENTA;
    // harness.assert_cell_fg(0, 0, Color::Rgb { r, g, b })?;
    //
    // Ok(())

    Ok(())
}

/// Test ANSI cyan (color 6)
#[test]
#[ignore = "Blocked: Awaiting ratatui-testlib v0.5.0 with CellAttributes API"]
fn test_ansi_color_cyan() -> Result<()> {
    // TODO(#168): Uncomment when ratatui-testlib v0.5.0 is released
    //
    // let harness = TuiTestHarness::with_theme("slime")?;
    // harness.send_input("\x1b[36mCYAN\x1b[0m\n")?;
    // harness.wait_for_render()?;
    //
    // let (r, g, b) = slime_palette::CYAN;
    // harness.assert_cell_fg(0, 0, Color::Rgb { r, g, b })?;
    //
    // Ok(())

    Ok(())
}

/// Test ANSI white (color 7)
#[test]
#[ignore = "Blocked: Awaiting ratatui-testlib v0.5.0 with CellAttributes API"]
fn test_ansi_color_white() -> Result<()> {
    // TODO(#168): Uncomment when ratatui-testlib v0.5.0 is released
    //
    // let harness = TuiTestHarness::with_theme("slime")?;
    // harness.send_input("\x1b[37mWHITE\x1b[0m\n")?;
    // harness.wait_for_render()?;
    //
    // let (r, g, b) = slime_palette::WHITE;
    // harness.assert_cell_fg(0, 0, Color::Rgb { r, g, b })?;
    //
    // Ok(())

    Ok(())
}

// =============================================================================
// TEST 2: BRIGHT ANSI COLORS (8-15)
// =============================================================================

/// Test bright black (color 8) - also known as "gray"
#[test]
#[ignore = "Blocked: Awaiting ratatui-testlib v0.5.0 with CellAttributes API"]
fn test_ansi_bright_black() -> Result<()> {
    // TODO(#168): Uncomment when ratatui-testlib v0.5.0 is released
    //
    // let harness = TuiTestHarness::with_theme("slime")?;
    // harness.send_input("\x1b[90mGRAY\x1b[0m\n")?;
    // harness.wait_for_render()?;
    //
    // let (r, g, b) = slime_palette::BRIGHT_BLACK;
    // harness.assert_cell_fg(0, 0, Color::Rgb { r, g, b })?;
    //
    // Ok(())

    Ok(())
}

/// Test bright red (color 9)
#[test]
#[ignore = "Blocked: Awaiting ratatui-testlib v0.5.0 with CellAttributes API"]
fn test_ansi_bright_red() -> Result<()> {
    // TODO(#168): Uncomment when ratatui-testlib v0.5.0 is released
    //
    // let harness = TuiTestHarness::with_theme("slime")?;
    // harness.send_input("\x1b[91mBRIGHT RED\x1b[0m\n")?;
    // harness.wait_for_render()?;
    //
    // let (r, g, b) = slime_palette::BRIGHT_RED;
    // harness.assert_cell_fg(0, 0, Color::Rgb { r, g, b })?;
    //
    // Ok(())

    Ok(())
}

/// Test bright green (color 10)
#[test]
#[ignore = "Blocked: Awaiting ratatui-testlib v0.5.0 with CellAttributes API"]
fn test_ansi_bright_green() -> Result<()> {
    // TODO(#168): Uncomment when ratatui-testlib v0.5.0 is released
    //
    // let harness = TuiTestHarness::with_theme("slime")?;
    // harness.send_input("\x1b[92mBRIGHT GREEN\x1b[0m\n")?;
    // harness.wait_for_render()?;
    //
    // let (r, g, b) = slime_palette::BRIGHT_GREEN;
    // harness.assert_cell_fg(0, 0, Color::Rgb { r, g, b })?;
    //
    // Ok(())

    Ok(())
}

/// Test bright yellow (color 11)
#[test]
#[ignore = "Blocked: Awaiting ratatui-testlib v0.5.0 with CellAttributes API"]
fn test_ansi_bright_yellow() -> Result<()> {
    // TODO(#168): Uncomment when ratatui-testlib v0.5.0 is released
    //
    // let harness = TuiTestHarness::with_theme("slime")?;
    // harness.send_input("\x1b[93mBRIGHT YELLOW\x1b[0m\n")?;
    // harness.wait_for_render()?;
    //
    // let (r, g, b) = slime_palette::BRIGHT_YELLOW;
    // harness.assert_cell_fg(0, 0, Color::Rgb { r, g, b })?;
    //
    // Ok(())

    Ok(())
}

/// Test bright blue (color 12)
#[test]
#[ignore = "Blocked: Awaiting ratatui-testlib v0.5.0 with CellAttributes API"]
fn test_ansi_bright_blue() -> Result<()> {
    // TODO(#168): Uncomment when ratatui-testlib v0.5.0 is released
    //
    // let harness = TuiTestHarness::with_theme("slime")?;
    // harness.send_input("\x1b[94mBRIGHT BLUE\x1b[0m\n")?;
    // harness.wait_for_render()?;
    //
    // let (r, g, b) = slime_palette::BRIGHT_BLUE;
    // harness.assert_cell_fg(0, 0, Color::Rgb { r, g, b })?;
    //
    // Ok(())

    Ok(())
}

/// Test bright magenta (color 13)
#[test]
#[ignore = "Blocked: Awaiting ratatui-testlib v0.5.0 with CellAttributes API"]
fn test_ansi_bright_magenta() -> Result<()> {
    // TODO(#168): Uncomment when ratatui-testlib v0.5.0 is released
    //
    // let harness = TuiTestHarness::with_theme("slime")?;
    // harness.send_input("\x1b[95mBRIGHT MAGENTA\x1b[0m\n")?;
    // harness.wait_for_render()?;
    //
    // let (r, g, b) = slime_palette::BRIGHT_MAGENTA;
    // harness.assert_cell_fg(0, 0, Color::Rgb { r, g, b })?;
    //
    // Ok(())

    Ok(())
}

/// Test bright cyan (color 14)
#[test]
#[ignore = "Blocked: Awaiting ratatui-testlib v0.5.0 with CellAttributes API"]
fn test_ansi_bright_cyan() -> Result<()> {
    // TODO(#168): Uncomment when ratatui-testlib v0.5.0 is released
    //
    // let harness = TuiTestHarness::with_theme("slime")?;
    // harness.send_input("\x1b[96mBRIGHT CYAN\x1b[0m\n")?;
    // harness.wait_for_render()?;
    //
    // let (r, g, b) = slime_palette::BRIGHT_CYAN;
    // harness.assert_cell_fg(0, 0, Color::Rgb { r, g, b })?;
    //
    // Ok(())

    Ok(())
}

/// Test bright white (color 15)
#[test]
#[ignore = "Blocked: Awaiting ratatui-testlib v0.5.0 with CellAttributes API"]
fn test_ansi_bright_white() -> Result<()> {
    // TODO(#168): Uncomment when ratatui-testlib v0.5.0 is released
    //
    // let harness = TuiTestHarness::with_theme("slime")?;
    // harness.send_input("\x1b[97mBRIGHT WHITE\x1b[0m\n")?;
    // harness.wait_for_render()?;
    //
    // let (r, g, b) = slime_palette::BRIGHT_WHITE;
    // harness.assert_cell_fg(0, 0, Color::Rgb { r, g, b })?;
    //
    // Ok(())

    Ok(())
}

// =============================================================================
// TEST 3: BACKGROUND COLORS
// =============================================================================

/// Test ANSI background color (red background)
#[test]
#[ignore = "Blocked: Awaiting ratatui-testlib v0.5.0 with CellAttributes API"]
fn test_ansi_background_color() -> Result<()> {
    // TODO(#168): Uncomment when ratatui-testlib v0.5.0 is released
    //
    // let harness = TuiTestHarness::with_theme("slime")?;
    //
    // // Red background: \x1b[41mTEXT\x1b[0m
    // harness.send_input("\x1b[41mRED BG\x1b[0m\n")?;
    // harness.wait_for_render()?;
    //
    // let (r, g, b) = slime_palette::RED;
    // harness.assert_cell_bg(0, 0, Color::Rgb { r, g, b })?;
    //
    // Ok(())

    Ok(())
}

/// Test bright background color (bright green background)
#[test]
#[ignore = "Blocked: Awaiting ratatui-testlib v0.5.0 with CellAttributes API"]
fn test_ansi_bright_background() -> Result<()> {
    // TODO(#168): Uncomment when ratatui-testlib v0.5.0 is released
    //
    // let harness = TuiTestHarness::with_theme("slime")?;
    //
    // // Bright green background: \x1b[102mTEXT\x1b[0m
    // harness.send_input("\x1b[102mBRIGHT GREEN BG\x1b[0m\n")?;
    // harness.wait_for_render()?;
    //
    // let (r, g, b) = slime_palette::BRIGHT_GREEN;
    // harness.assert_cell_bg(0, 0, Color::Rgb { r, g, b })?;
    //
    // Ok(())

    Ok(())
}

// =============================================================================
// TEST 4: 24-BIT RGB COLORS
// =============================================================================

/// Test 24-bit RGB foreground color
#[test]
#[ignore = "Blocked: Awaiting ratatui-testlib v0.5.0 with CellAttributes API"]
fn test_24bit_rgb_foreground() -> Result<()> {
    // TODO(#168): Uncomment when ratatui-testlib v0.5.0 is released
    //
    // let harness = TuiTestHarness::with_theme("slime")?;
    //
    // // Custom RGB color: \x1b[38;2;255;128;64mTEXT\x1b[0m
    // harness.send_input("\x1b[38;2;255;128;64mRGB\x1b[0m\n")?;
    // harness.wait_for_render()?;
    //
    // // Verify exact RGB color
    // harness.assert_cell_fg(0, 0, Color::Rgb { r: 255, g: 128, b: 64 })?;
    //
    // Ok(())

    Ok(())
}

/// Test 24-bit RGB background color
#[test]
#[ignore = "Blocked: Awaiting ratatui-testlib v0.5.0 with CellAttributes API"]
fn test_24bit_rgb_background() -> Result<()> {
    // TODO(#168): Uncomment when ratatui-testlib v0.5.0 is released
    //
    // let harness = TuiTestHarness::with_theme("slime")?;
    //
    // // Custom RGB background: \x1b[48;2;64;128;255mTEXT\x1b[0m
    // harness.send_input("\x1b[48;2;64;128;255mRGB BG\x1b[0m\n")?;
    // harness.wait_for_render()?;
    //
    // harness.assert_cell_bg(0, 0, Color::Rgb { r: 64, g: 128, b: 255 })?;
    //
    // Ok(())

    Ok(())
}

// =============================================================================
// TEST 5: DEFAULT COLORS
// =============================================================================

/// Test default foreground color
#[test]
#[ignore = "Blocked: Awaiting ratatui-testlib v0.5.0 with CellAttributes API"]
fn test_default_foreground_color() -> Result<()> {
    // TODO(#168): Uncomment when ratatui-testlib v0.5.0 is released
    //
    // let harness = TuiTestHarness::with_theme("slime")?;
    //
    // // Send plain text (no color codes)
    // harness.send_input("PLAIN TEXT\n")?;
    // harness.wait_for_render()?;
    //
    // // Should use default foreground color
    // let (r, g, b) = slime_palette::FOREGROUND;
    // harness.assert_cell_fg(0, 0, Color::Rgb { r, g, b })?;
    //
    // Ok(())

    Ok(())
}

/// Test default background color
#[test]
#[ignore = "Blocked: Awaiting ratatui-testlib v0.5.0 with CellAttributes API"]
fn test_default_background_color() -> Result<()> {
    // TODO(#168): Uncomment when ratatui-testlib v0.5.0 is released
    //
    // let harness = TuiTestHarness::with_theme("slime")?;
    // harness.send_input("PLAIN TEXT\n")?;
    // harness.wait_for_render()?;
    //
    // // Should use default background color
    // let (r, g, b) = slime_palette::BACKGROUND;
    // harness.assert_cell_bg(0, 0, Color::Rgb { r, g, b })?;
    //
    // Ok(())

    Ok(())
}

// =============================================================================
// TEST 6: COLOR RESET
// =============================================================================

/// Test that color reset (\x1b[0m) restores default colors
#[test]
#[ignore = "Blocked: Awaiting ratatui-testlib v0.5.0 with CellAttributes API"]
fn test_color_reset() -> Result<()> {
    // TODO(#168): Uncomment when ratatui-testlib v0.5.0 is released
    //
    // let harness = TuiTestHarness::with_theme("slime")?;
    //
    // // Red text, then reset
    // harness.send_input("\x1b[31mRED\x1b[0mNORMAL\n")?;
    // harness.wait_for_render()?;
    //
    // // First 3 chars should be red
    // let (r, g, b) = slime_palette::RED;
    // harness.assert_cell_fg(0, 0, Color::Rgb { r, g, b })?;
    //
    // // Char at position 3 (after reset) should be default foreground
    // let (fg_r, fg_g, fg_b) = slime_palette::FOREGROUND;
    // harness.assert_cell_fg(0, 3, Color::Rgb { r: fg_r, g: fg_g, b: fg_b })?;
    //
    // Ok(())

    Ok(())
}

// =============================================================================
// TEST 7: COMPREHENSIVE INTEGRATION TEST
// =============================================================================

/// Test multiple colors in sequence (simulating syntax highlighting)
#[test]
#[ignore = "Blocked: Awaiting ratatui-testlib v0.5.0 with CellAttributes API"]
fn test_syntax_highlighting_simulation() -> Result<()> {
    // TODO(#168): Uncomment when ratatui-testlib v0.5.0 is released
    //
    // let harness = TuiTestHarness::with_theme("slime")?;
    //
    // // Simulate syntax-highlighted code:
    // // fn main() { println!("Hello"); }
    // //
    // // fn (keyword - magenta)
    // // main (function - blue)
    // // () (default)
    // // { (default)
    // // println! (macro - cyan)
    // // ("Hello") (string - green)
    // // } (default)
    //
    // harness.send_input(
    //     "\x1b[35mfn\x1b[0m \x1b[34mmain\x1b[0m() { \x1b[36mprintln!\x1b[0m(\x1b[32m\"Hello\"\x1b[0m); }\n"
    // )?;
    // harness.wait_for_render()?;
    //
    // // Verify "fn" is magenta
    // let (m_r, m_g, m_b) = slime_palette::MAGENTA;
    // harness.assert_cell_fg(0, 0, Color::Rgb { r: m_r, g: m_g, b: m_b })?;
    //
    // // Verify "main" is blue (starts at col 3)
    // let (b_r, b_g, b_b) = slime_palette::BLUE;
    // harness.assert_cell_fg(0, 3, Color::Rgb { r: b_r, g: b_g, b: b_b })?;
    //
    // // Verify "println!" is cyan (starts at col 10)
    // let (c_r, c_g, c_b) = slime_palette::CYAN;
    // harness.assert_cell_fg(0, 10, Color::Rgb { r: c_r, g: c_g, b: c_b })?;
    //
    // // Verify "\"Hello\"" is green (starts at col 19)
    // let (g_r, g_g, g_b) = slime_palette::GREEN;
    // harness.assert_cell_fg(0, 19, Color::Rgb { r: g_r, g: g_g, b: g_b })?;
    //
    // Ok(())

    Ok(())
}

// =============================================================================
// HELPER TESTS - VERIFY COLOR PALETTE CONSTANTS
// =============================================================================

#[test]
fn test_slime_palette_constants() {
    // Standard colors
    assert_eq!(slime_palette::BLACK, (102, 102, 102));
    assert_eq!(slime_palette::RED, (205, 101, 100));
    assert_eq!(slime_palette::GREEN, (174, 193, 153));
    assert_eq!(slime_palette::YELLOW, (255, 240, 153));
    assert_eq!(slime_palette::BLUE, (109, 156, 190));
    assert_eq!(slime_palette::MAGENTA, (176, 129, 185));
    assert_eq!(slime_palette::CYAN, (128, 181, 179));
    assert_eq!(slime_palette::WHITE, (239, 239, 239));

    // Bright colors
    assert_eq!(slime_palette::BRIGHT_BLACK, (136, 136, 136));
    assert_eq!(slime_palette::BRIGHT_RED, (224, 128, 128));
    assert_eq!(slime_palette::BRIGHT_GREEN, (200, 219, 168));
    assert_eq!(slime_palette::BRIGHT_YELLOW, (255, 255, 176));
    assert_eq!(slime_palette::BRIGHT_BLUE, (139, 184, 216));
    assert_eq!(slime_palette::BRIGHT_MAGENTA, (201, 160, 208));
    assert_eq!(slime_palette::BRIGHT_CYAN, (159, 208, 206));
    assert_eq!(slime_palette::BRIGHT_WHITE, (255, 255, 255));

    // Default colors
    assert_eq!(slime_palette::FOREGROUND, (224, 224, 224));
    assert_eq!(slime_palette::BACKGROUND, (30, 35, 36));
    assert_eq!(slime_palette::CURSOR, (168, 223, 90));
}
