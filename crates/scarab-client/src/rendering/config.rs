// Font configuration and management

use serde::{Deserialize, Serialize};

/// Font configuration for the text renderer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FontConfig {
    /// Primary font family name
    pub family: String,

    /// Font size in points
    pub size: f32,

    /// Fallback font families (in order of preference)
    pub fallback: Vec<String>,

    /// Line height multiplier
    pub line_height: f32,

    /// Letter spacing adjustment
    pub letter_spacing: f32,

    /// Enable font hinting
    pub hinting: bool,

    /// Enable subpixel positioning
    pub subpixel: bool,
}

impl Default for FontConfig {
    fn default() -> Self {
        Self {
            family: "JetBrains Mono".to_string(),
            size: 14.0,
            fallback: vec![
                "Fira Code".to_string(),
                "Cascadia Code".to_string(),
                "DejaVu Sans Mono".to_string(),
                "Noto Sans Mono".to_string(),
            ],
            line_height: 1.2,
            letter_spacing: 0.0,
            hinting: true,
            subpixel: true,
        }
    }
}

impl FontConfig {
    /// Create a new font configuration
    pub fn new(family: impl Into<String>, size: f32) -> Self {
        Self {
            family: family.into(),
            size,
            ..Default::default()
        }
    }

    /// Get all font families (primary + fallbacks)
    pub fn all_families(&self) -> Vec<&str> {
        let mut families = vec![self.family.as_str()];
        families.extend(self.fallback.iter().map(|s| s.as_str()));
        families
    }

    /// Calculate cell dimensions based on font metrics
    pub fn cell_dimensions(&self) -> (f32, f32) {
        // Approximate cell dimensions (will be refined with actual font metrics)
        let width = self.size * 0.6; // Monospace width ratio
        let height = self.size * self.line_height;
        (width, height)
    }
}

/// Text attributes flags (matches SharedState Cell flags)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TextAttributes {
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
    pub strikethrough: bool,
    pub dim: bool,
    pub reverse: bool,
}

impl TextAttributes {
    pub fn from_flags(flags: u8) -> Self {
        Self {
            bold: flags & 0x01 != 0,
            italic: flags & 0x02 != 0,
            underline: flags & 0x04 != 0,
            strikethrough: flags & 0x08 != 0,
            dim: flags & 0x10 != 0,
            reverse: flags & 0x20 != 0,
        }
    }

    pub fn to_flags(&self) -> u8 {
        let mut flags = 0u8;
        if self.bold {
            flags |= 0x01;
        }
        if self.italic {
            flags |= 0x02;
        }
        if self.underline {
            flags |= 0x04;
        }
        if self.strikethrough {
            flags |= 0x08;
        }
        if self.dim {
            flags |= 0x10;
        }
        if self.reverse {
            flags |= 0x20;
        }
        flags
    }
}

impl Default for TextAttributes {
    fn default() -> Self {
        Self {
            bold: false,
            italic: false,
            underline: false,
            strikethrough: false,
            dim: false,
            reverse: false,
        }
    }
}

/// Color helper functions for converting between formats
pub mod color {
    use bevy::color::Srgba;
    use bevy::prelude::{Color, ColorToComponents};

    /// Convert u32 ARGB to Bevy Color in LINEAR space
    ///
    /// The daemon uses ARGB format (0xAARRGGBB):
    /// - High byte (bits 24-31): Alpha
    /// - Next byte (bits 16-23): Red
    /// - Next byte (bits 8-15): Green
    /// - Low byte (bits 0-7): Blue
    ///
    /// IMPORTANT: Returns color in LINEAR space for use as vertex colors.
    /// Bevy's 2D mesh shader expects vertex colors in linear space.
    /// The input values are interpreted as sRGB and converted to linear.
    pub fn from_rgba(argb: u32) -> Color {
        let a = ((argb >> 24) & 0xFF) as f32 / 255.0;
        let r = ((argb >> 16) & 0xFF) as f32 / 255.0;
        let g = ((argb >> 8) & 0xFF) as f32 / 255.0;
        let b = (argb & 0xFF) as f32 / 255.0;
        // Create sRGB color and convert to linear for vertex colors
        let srgb = Srgba::new(r, g, b, a);
        Color::linear_rgba(
            srgb_to_linear(srgb.red),
            srgb_to_linear(srgb.green),
            srgb_to_linear(srgb.blue),
            a, // Alpha doesn't need conversion
        )
    }

    /// Convert sRGB component to linear
    /// Uses the standard sRGB to linear conversion formula
    #[inline]
    fn srgb_to_linear(c: f32) -> f32 {
        if c <= 0.04045 {
            c / 12.92
        } else {
            ((c + 0.055) / 1.055).powf(2.4)
        }
    }

    /// Convert Bevy Color to u32 ARGB
    pub fn to_rgba(color: Color) -> u32 {
        let [r, g, b, a] = color.to_srgba().to_f32_array();
        let a = (a * 255.0) as u32;
        let r = (r * 255.0) as u32;
        let g = (g * 255.0) as u32;
        let b = (b * 255.0) as u32;
        (a << 24) | (r << 16) | (g << 8) | b
    }
}
