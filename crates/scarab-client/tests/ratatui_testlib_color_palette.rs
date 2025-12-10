//! Issue #172: Use ratatui-testlib ColorPalette for theme verification
//!
//! This test file validates Scarab's theme system and verifies ANSI color mappings
//! using a local ColorPalette implementation until ratatui-testlib v0.5.0 is released.
//!
//! ## Background: Scarab's Theme System
//!
//! Scarab supports themes that define color palettes for:
//! - **ANSI colors (0-15)**: Standard terminal colors (black, red, green, etc.)
//! - **Extended colors (16-255)**: 256-color palette
//! - **True color (RGB)**: 24-bit colors
//! - **UI elements**: Status bar, tab bar, selection, cursor, etc.
//!
//! ### Slime Theme (Default)
//!
//! The default "slime" theme colors (from theme_resolver.rs):
//!
//! ```text
//! Background: #0d1208 (dark green-black)
//! Foreground: #a8df5a (bright slime green)
//!
//! ANSI Colors (0-7):
//! - Black:   #0d1208
//! - Red:     #ff5555
//! - Green:   #a8df5a
//! - Yellow:  #f1fa8c
//! - Blue:    #6272a4
//! - Magenta: #ff79c6
//! - Cyan:    #8be9fd
//! - White:   #f8f8f2
//!
//! Bright ANSI Colors (8-15):
//! - Bright Black:   #44475a
//! - Bright Red:     #ff6e6e
//! - Bright Green:   #c4f07a
//! - Bright Yellow:  #ffffa5
//! - Bright Blue:    #7c8dbd
//! - Bright Magenta: #ff92df
//! - Bright Cyan:    #a4ffff
//! - Bright White:   #ffffff
//! ```
//!
//! ## Implementation Note
//!
//! These tests use Scarab's existing `ThemeResolver` and `ColorConfig` to verify
//! theme colors. When ratatui-testlib v0.5.0 is released with ColorPalette support,
//! these tests can be migrated to use the official API.

use anyhow::{anyhow, Result};
use scarab_config::{ColorConfig, ThemeResolver};

/// Local ColorPalette implementation for testing until ratatui-testlib v0.5.0
///
/// This mirrors the expected v0.5.0 API and can be replaced when released.
#[derive(Debug, Clone, PartialEq)]
struct ColorPalette {
    ansi_colors: [Color; 16],
    background: Color,
    foreground: Color,
}

/// Color representation matching expected ratatui-testlib API
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Color {
    Rgb(u8, u8, u8),
    Reset,
}

impl ColorPalette {
    /// Create a ColorPalette from Scarab's theme name
    ///
    /// This creates a pure theme config without default overrides
    fn from_theme(theme_name: &str) -> Result<Self> {
        let resolver = ThemeResolver::new();
        let mut config = ColorConfig {
            theme: Some(theme_name.to_string()),
            foreground: None,
            background: None,
            cursor: None,
            selection_background: None,
            selection_foreground: None,
            palette: scarab_config::ColorPalette::default(),
            opacity: 1.0,
            dim_opacity: 0.7,
        };
        resolver.resolve(&mut config)?;
        Self::from_config(&config)
    }

    /// Create a ColorPalette from Scarab's ColorConfig
    fn from_config(config: &ColorConfig) -> Result<Self> {
        let bg = Self::parse_hex(config.background.as_deref().unwrap_or("#000000"))?;
        let fg = Self::parse_hex(config.foreground.as_deref().unwrap_or("#ffffff"))?;

        let ansi_colors = [
            Self::parse_hex(&config.palette.black)?,
            Self::parse_hex(&config.palette.red)?,
            Self::parse_hex(&config.palette.green)?,
            Self::parse_hex(&config.palette.yellow)?,
            Self::parse_hex(&config.palette.blue)?,
            Self::parse_hex(&config.palette.magenta)?,
            Self::parse_hex(&config.palette.cyan)?,
            Self::parse_hex(&config.palette.white)?,
            Self::parse_hex(&config.palette.bright_black)?,
            Self::parse_hex(&config.palette.bright_red)?,
            Self::parse_hex(&config.palette.bright_green)?,
            Self::parse_hex(&config.palette.bright_yellow)?,
            Self::parse_hex(&config.palette.bright_blue)?,
            Self::parse_hex(&config.palette.bright_magenta)?,
            Self::parse_hex(&config.palette.bright_cyan)?,
            Self::parse_hex(&config.palette.bright_white)?,
        ];

        Ok(Self {
            ansi_colors,
            background: bg,
            foreground: fg,
        })
    }

    /// Get ANSI color by index (0-15)
    fn get_ansi(&self, index: u8) -> Result<Color> {
        self.ansi_colors
            .get(index as usize)
            .copied()
            .ok_or_else(|| anyhow!("ANSI color index {} out of range", index))
    }

    /// Assert that an ANSI color matches expected value
    fn assert_ansi_color(&self, index: u8, expected: Color) -> Result<()> {
        let actual = self.get_ansi(index)?;
        if actual != expected {
            return Err(anyhow!(
                "ANSI color {} mismatch: expected {:?}, got {:?}",
                index,
                expected,
                actual
            ));
        }
        Ok(())
    }

    /// Parse hex color string to RGB
    fn parse_hex(hex: &str) -> Result<Color> {
        let hex = hex.trim_start_matches('#');
        if hex.len() != 6 {
            return Err(anyhow!("Invalid hex color: {}", hex));
        }

        let r = u8::from_str_radix(&hex[0..2], 16)?;
        let g = u8::from_str_radix(&hex[2..4], 16)?;
        let b = u8::from_str_radix(&hex[4..6], 16)?;

        Ok(Color::Rgb(r, g, b))
    }

    /// Create the slime theme palette
    fn slime() -> Self {
        Self {
            background: Color::Rgb(13, 18, 8),    // #0d1208
            foreground: Color::Rgb(168, 223, 90), // #a8df5a
            ansi_colors: [
                Color::Rgb(13, 18, 8),      // Black: #0d1208
                Color::Rgb(255, 85, 85),    // Red: #ff5555
                Color::Rgb(168, 223, 90),   // Green: #a8df5a
                Color::Rgb(241, 250, 140),  // Yellow: #f1fa8c
                Color::Rgb(98, 114, 164),   // Blue: #6272a4
                Color::Rgb(255, 121, 198),  // Magenta: #ff79c6
                Color::Rgb(139, 233, 253),  // Cyan: #8be9fd
                Color::Rgb(248, 248, 242),  // White: #f8f8f2
                Color::Rgb(68, 71, 90),     // Bright Black: #44475a
                Color::Rgb(255, 110, 110),  // Bright Red: #ff6e6e
                Color::Rgb(196, 240, 122),  // Bright Green: #c4f07a
                Color::Rgb(255, 255, 165),  // Bright Yellow: #ffffa5
                Color::Rgb(124, 141, 189),  // Bright Blue: #7c8dbd
                Color::Rgb(255, 146, 223),  // Bright Magenta: #ff92df
                Color::Rgb(164, 255, 255),  // Bright Cyan: #a4ffff
                Color::Rgb(255, 255, 255),  // Bright White: #ffffff
            ],
        }
    }

    /// Create the dracula theme palette
    fn dracula() -> Self {
        Self {
            background: Color::Rgb(40, 42, 54),   // #282a36
            foreground: Color::Rgb(248, 248, 242), // #f8f8f2
            ansi_colors: [
                Color::Rgb(33, 34, 44),     // Black
                Color::Rgb(255, 85, 85),    // Red
                Color::Rgb(80, 250, 123),   // Green
                Color::Rgb(241, 250, 140),  // Yellow
                Color::Rgb(189, 147, 249),  // Blue
                Color::Rgb(255, 121, 198),  // Magenta
                Color::Rgb(139, 233, 253),  // Cyan
                Color::Rgb(248, 248, 242),  // White
                Color::Rgb(98, 114, 164),   // Bright Black
                Color::Rgb(255, 110, 110),  // Bright Red
                Color::Rgb(105, 255, 148),  // Bright Green
                Color::Rgb(255, 255, 165),  // Bright Yellow
                Color::Rgb(214, 172, 255),  // Bright Blue
                Color::Rgb(255, 146, 223),  // Bright Magenta
                Color::Rgb(164, 255, 255),  // Bright Cyan
                Color::Rgb(255, 255, 255),  // Bright White
            ],
        }
    }

    /// Create the nord theme palette
    fn nord() -> Self {
        Self {
            background: Color::Rgb(46, 52, 64),   // #2e3440
            foreground: Color::Rgb(216, 222, 233), // #d8dee9
            ansi_colors: [
                Color::Rgb(59, 66, 82),     // Black
                Color::Rgb(191, 97, 106),   // Red
                Color::Rgb(163, 190, 140),  // Green
                Color::Rgb(235, 203, 139),  // Yellow
                Color::Rgb(129, 161, 193),  // Blue
                Color::Rgb(180, 142, 173),  // Magenta
                Color::Rgb(136, 192, 208),  // Cyan
                Color::Rgb(229, 233, 240),  // White
                Color::Rgb(76, 86, 106),    // Bright Black
                Color::Rgb(191, 97, 106),   // Bright Red
                Color::Rgb(163, 190, 140),  // Bright Green
                Color::Rgb(235, 203, 139),  // Bright Yellow
                Color::Rgb(129, 161, 193),  // Bright Blue
                Color::Rgb(180, 142, 173),  // Bright Magenta
                Color::Rgb(143, 188, 187),  // Bright Cyan
                Color::Rgb(236, 239, 244),  // Bright White
            ],
        }
    }
}

// ============================================================================
// Tests
// ============================================================================

/// Test 1: Extract color palette from Scarab's ColorConfig
///
/// This test verifies that we can extract the color palette from Scarab's
/// configuration and validate its structure.
#[test]
fn test_extract_color_palette() -> Result<()> {
    let palette = ColorPalette::from_theme("slime")?;

    // Verify basic properties
    assert_eq!(palette.ansi_colors.len(), 16);
    assert_ne!(palette.background, Color::Reset);
    assert_ne!(palette.foreground, Color::Reset);

    // Verify colors are RGB (not Reset)
    for (i, color) in palette.ansi_colors.iter().enumerate() {
        assert!(
            matches!(color, Color::Rgb(_, _, _)),
            "ANSI color {} should be RGB, got {:?}",
            i,
            color
        );
    }

    Ok(())
}

/// Test 2: Verify slime theme ANSI colors
///
/// This test verifies that the slime theme's ANSI colors match the expected
/// RGB values from the theme definition.
#[test]
fn test_slime_theme_ansi_colors() -> Result<()> {
    let palette = ColorPalette::from_theme("slime")?;

    // Verify normal ANSI colors (0-7)
    palette.assert_ansi_color(0, Color::Rgb(13, 18, 8))?;      // Black
    palette.assert_ansi_color(1, Color::Rgb(255, 85, 85))?;    // Red
    palette.assert_ansi_color(2, Color::Rgb(168, 223, 90))?;   // Green
    palette.assert_ansi_color(3, Color::Rgb(241, 250, 140))?;  // Yellow
    palette.assert_ansi_color(4, Color::Rgb(98, 114, 164))?;   // Blue
    palette.assert_ansi_color(5, Color::Rgb(255, 121, 198))?;  // Magenta
    palette.assert_ansi_color(6, Color::Rgb(139, 233, 253))?;  // Cyan
    palette.assert_ansi_color(7, Color::Rgb(248, 248, 242))?;  // White

    Ok(())
}

/// Test 3: Verify slime theme background and foreground
///
/// This test verifies that the slime theme's background and foreground colors
/// match the expected values.
#[test]
fn test_slime_theme_background_foreground() -> Result<()> {
    let palette = ColorPalette::from_theme("slime")?;

    // Slime theme background: #0d1208 (13, 18, 8)
    assert_eq!(palette.background, Color::Rgb(13, 18, 8));

    // Slime theme foreground: #a8df5a (168, 223, 90)
    assert_eq!(palette.foreground, Color::Rgb(168, 223, 90));

    Ok(())
}

/// Test 4: Verify ANSI color retrieval
///
/// This test verifies that we can retrieve ANSI colors by index and that
/// out-of-bounds access is handled properly.
#[test]
fn test_ansi_color_retrieval() -> Result<()> {
    let palette = ColorPalette::from_theme("slime")?;

    // Test valid indices
    for i in 0..16 {
        let color = palette.get_ansi(i)?;
        assert!(matches!(color, Color::Rgb(_, _, _)));
    }

    // Test out-of-bounds access
    let result = palette.get_ansi(16);
    assert!(result.is_err(), "Should fail for out-of-bounds index");

    Ok(())
}

/// Test 5: Verify bright ANSI colors (8-15)
///
/// This test verifies that the slime theme's bright ANSI colors are correctly set.
#[test]
fn test_bright_ansi_colors() -> Result<()> {
    let palette = ColorPalette::from_theme("slime")?;

    // Verify bright ANSI colors (8-15)
    palette.assert_ansi_color(8, Color::Rgb(68, 71, 90))?;     // Bright Black
    palette.assert_ansi_color(9, Color::Rgb(255, 110, 110))?;  // Bright Red
    palette.assert_ansi_color(10, Color::Rgb(196, 240, 122))?; // Bright Green
    palette.assert_ansi_color(11, Color::Rgb(255, 255, 165))?; // Bright Yellow
    palette.assert_ansi_color(12, Color::Rgb(124, 141, 189))?; // Bright Blue
    palette.assert_ansi_color(13, Color::Rgb(255, 146, 223))?; // Bright Magenta
    palette.assert_ansi_color(14, Color::Rgb(164, 255, 255))?; // Bright Cyan
    palette.assert_ansi_color(15, Color::Rgb(255, 255, 255))?; // Bright White

    Ok(())
}

/// Test 6: Verify ColorPalette::slime() factory method
///
/// This test verifies that the ColorPalette::slime() factory method produces
/// the expected palette that matches the theme resolver's slime theme.
#[test]
fn test_color_palette_slime_factory() -> Result<()> {
    let palette_from_theme = ColorPalette::from_theme("slime")?;
    let palette_from_factory = ColorPalette::slime();

    // Both should produce the same palette
    assert_eq!(palette_from_theme, palette_from_factory);

    Ok(())
}

/// Test 7: Verify hex color parsing
///
/// This test verifies that the hex color parser handles various formats correctly.
#[test]
fn test_hex_color_parsing() -> Result<()> {
    // Test with hash prefix
    let color1 = ColorPalette::parse_hex("#ff5555")?;
    assert_eq!(color1, Color::Rgb(255, 85, 85));

    // Test without hash prefix
    let color2 = ColorPalette::parse_hex("ff5555")?;
    assert_eq!(color2, Color::Rgb(255, 85, 85));

    // Test lowercase
    let color3 = ColorPalette::parse_hex("#a8df5a")?;
    assert_eq!(color3, Color::Rgb(168, 223, 90));

    // Test uppercase
    let color4 = ColorPalette::parse_hex("#A8DF5A")?;
    assert_eq!(color4, Color::Rgb(168, 223, 90));

    // Test invalid format (too short)
    let result = ColorPalette::parse_hex("#fff");
    assert!(result.is_err(), "Should fail for invalid format");

    // Test invalid format (too long)
    let result = ColorPalette::parse_hex("#ffffff00");
    assert!(result.is_err(), "Should fail for invalid format");

    Ok(())
}

/// Test 8: Verify theme switching updates palette
///
/// This test verifies that different themes produce different palettes.
#[test]
fn test_theme_switching_updates_palette() -> Result<()> {
    // Get slime theme palette
    let palette_slime = ColorPalette::from_theme("slime")?;

    // Get dracula theme palette
    let palette_dracula = ColorPalette::from_theme("dracula")?;

    // Palettes should be different
    assert_ne!(palette_slime.background, palette_dracula.background);
    assert_ne!(palette_slime.foreground, palette_dracula.foreground);

    // Verify dracula colors
    assert_eq!(palette_dracula.background, Color::Rgb(40, 42, 54));
    assert_eq!(palette_dracula.foreground, Color::Rgb(248, 248, 242));

    Ok(())
}

/// Test 9: Verify theme matches expected palette using factory methods
///
/// This test verifies that factory methods produce palettes that match
/// the theme resolver's configurations.
#[test]
fn test_theme_matches_using_factories() -> Result<()> {
    // Test slime theme
    let palette_from_theme = ColorPalette::from_theme("slime")?;
    let palette_from_factory = ColorPalette::slime();
    assert_eq!(palette_from_theme, palette_from_factory);

    // Test dracula theme
    let palette_from_theme = ColorPalette::from_theme("dracula")?;
    let palette_from_factory = ColorPalette::dracula();
    assert_eq!(palette_from_theme, palette_from_factory);

    // Test nord theme
    let palette_from_theme = ColorPalette::from_theme("nord")?;
    let palette_from_factory = ColorPalette::nord();
    assert_eq!(palette_from_theme, palette_from_factory);

    Ok(())
}

/// Test 10: Verify all available themes can be loaded
///
/// This test verifies that all available themes can be loaded and produce
/// valid color palettes without errors.
#[test]
fn test_all_themes_loadable() -> Result<()> {
    let resolver = ThemeResolver::new();
    let themes = resolver.available_themes();

    // Should have multiple themes
    assert!(!themes.is_empty(), "Should have at least one theme");

    // All themes should be loadable
    for theme_name in themes {
        let palette = ColorPalette::from_theme(&theme_name)?;

        // Verify basic properties
        assert_eq!(palette.ansi_colors.len(), 16);
        assert!(matches!(palette.background, Color::Rgb(_, _, _)));
        assert!(matches!(palette.foreground, Color::Rgb(_, _, _)));
    }

    Ok(())
}
