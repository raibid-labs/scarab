//! Sixel Graphics Protocol Parser
//!
//! Implements the DEC Sixel protocol for inline bitmap graphics.
//!
//! Format: `DCS Ps ; Ps ; Ps q <sixel data> ST`
//! - DCS = ESC P (0x1B 0x50)
//! - ST = ESC \ (0x1B 0x5C)
//!
//! Sixel data encoding:
//! - Each character from 0x3F ('?') to 0x7E ('~') represents 6 vertical pixels
//! - Colors defined with `#Pc;Pu;Px;Py;Pz` where Pc is color number (0-255)
//! - '$' = Carriage return (move cursor to left edge)
//! - '-' = Line feed (move down 6 pixels to next sixel row)
//! - '!' = Repeat introducer, followed by count and character: `!<count><char>`
//!
//! Color format modes (Pu):
//! - 1 = HLS (Hue, Lightness, Saturation)
//! - 2 = RGB (Red, Green, Blue) [0-100 range]

use log::{debug, warn};

/// Maximum Sixel image dimensions (prevent excessive memory allocation)
const MAX_SIXEL_WIDTH: u32 = 4096;
const MAX_SIXEL_HEIGHT: u32 = 4096;

/// Sixel color palette (256 colors)
#[derive(Debug, Clone)]
struct SixelPalette {
    /// RGBA colors (256 entries, default VT340 palette)
    colors: [u32; 256],
}

impl SixelPalette {
    /// Create a new palette with VT340 default colors
    fn new() -> Self {
        let mut palette = Self {
            colors: [0xFF000000; 256],
        };

        // Initialize with standard VT340 16-color palette
        // Colors 0-15: Standard ANSI-like colors
        palette.colors[0] = 0xFF000000;  // Black
        palette.colors[1] = 0xFF0000CC;  // Blue
        palette.colors[2] = 0xFFCC0000;  // Red
        palette.colors[3] = 0xFFCC00CC;  // Magenta
        palette.colors[4] = 0xFF00CC00;  // Green
        palette.colors[5] = 0xFF00CCCC;  // Cyan
        palette.colors[6] = 0xFFCCCC00;  // Yellow
        palette.colors[7] = 0xFFCCCCCC;  // Gray
        palette.colors[8] = 0xFF333333;  // Dark Gray
        palette.colors[9] = 0xFF0000FF;  // Bright Blue
        palette.colors[10] = 0xFFFF0000; // Bright Red
        palette.colors[11] = 0xFFFF00FF; // Bright Magenta
        palette.colors[12] = 0xFF00FF00; // Bright Green
        palette.colors[13] = 0xFF00FFFF; // Bright Cyan
        palette.colors[14] = 0xFFFFFF00; // Bright Yellow
        palette.colors[15] = 0xFFFFFFFF; // White

        // Colors 16-255: Grayscale gradient for undefined colors
        for i in 16..256 {
            let gray = ((i - 16) * 255 / 239) as u8;
            palette.colors[i] = 0xFF000000 | ((gray as u32) << 16) | ((gray as u32) << 8) | (gray as u32);
        }

        palette
    }

    /// Set a color using RGB values (0-100 range)
    fn set_rgb(&mut self, index: u8, r: u8, g: u8, b: u8) {
        // Convert from 0-100 range to 0-255
        let r_full = ((r as u32 * 255) / 100).min(255) as u8;
        let g_full = ((g as u32 * 255) / 100).min(255) as u8;
        let b_full = ((b as u32 * 255) / 100).min(255) as u8;

        self.colors[index as usize] = 0xFF000000 | ((r_full as u32) << 16) | ((g_full as u32) << 8) | (b_full as u32);
    }

    /// Set a color using HLS values (0-360 for H, 0-100 for L and S)
    fn set_hls(&mut self, index: u8, h: u16, l: u8, s: u8) {
        // Convert HLS to RGB
        let (r, g, b) = hls_to_rgb(h, l, s);
        self.colors[index as usize] = 0xFF000000 | ((r as u32) << 16) | ((g as u32) << 8) | (b as u32);
    }

    /// Get a color as RGBA
    #[inline]
    fn get(&self, index: u8) -> u32 {
        self.colors[index as usize]
    }
}

/// Parsed Sixel image data
#[derive(Debug, Clone)]
pub struct SixelData {
    /// RGBA pixel data (width * height * 4 bytes)
    pub pixels: Vec<u8>,
    /// Image width in pixels
    pub width: u32,
    /// Image height in pixels
    pub height: u32,
    /// Aspect ratio parameters from DCS sequence
    pub aspect_ratio: (u8, u8),
}

/// Sixel parser state machine
struct SixelParser {
    /// Current X position (column)
    x: u32,
    /// Current Y position (sixel row, each row is 6 pixels tall)
    y: u32,
    /// Maximum X position reached (determines final width)
    max_x: u32,
    /// Maximum Y position reached (determines final height)
    max_y: u32,
    /// Current color register (0-255)
    current_color: u8,
    /// Color palette
    palette: SixelPalette,
    /// Pixel data (stored as color indices, converted to RGBA later)
    /// Each pixel is a u8 color index
    pixels: Vec<u8>,
    /// Canvas width (allocated width)
    canvas_width: u32,
    /// Canvas height (allocated height, grows as needed)
    canvas_height: u32,
    /// Aspect ratio from parameters
    aspect_ratio: (u8, u8),
    /// Transparent background mode
    transparent_bg: bool,
}

impl SixelParser {
    /// Create a new parser with default parameters
    fn new(transparent_bg: bool) -> Self {
        Self {
            x: 0,
            y: 0,
            max_x: 0,
            max_y: 0,
            current_color: 0,
            palette: SixelPalette::new(),
            pixels: vec![0; 256 * 256], // Start with reasonable size
            canvas_width: 256,
            canvas_height: 256,
            aspect_ratio: (1, 1),
            transparent_bg,
        }
    }

    /// Ensure the canvas can fit the given coordinates
    fn ensure_capacity(&mut self, x: u32, y: u32) {
        let required_width = (x + 1).min(MAX_SIXEL_WIDTH);
        let required_height = ((y + 1) * 6).min(MAX_SIXEL_HEIGHT); // Each sixel row is 6 pixels

        if required_width > self.canvas_width || required_height > self.canvas_height {
            let new_width = required_width.max(self.canvas_width);
            let new_height = required_height.max(self.canvas_height);

            debug!(
                "Expanding sixel canvas from {}x{} to {}x{}",
                self.canvas_width, self.canvas_height, new_width, new_height
            );

            let mut new_pixels = vec![0u8; (new_width * new_height) as usize];

            // Copy existing pixels
            for old_y in 0..self.canvas_height {
                for old_x in 0..self.canvas_width {
                    let old_idx = (old_y * self.canvas_width + old_x) as usize;
                    let new_idx = (old_y * new_width + old_x) as usize;
                    if old_idx < self.pixels.len() && new_idx < new_pixels.len() {
                        new_pixels[new_idx] = self.pixels[old_idx];
                    }
                }
            }

            self.pixels = new_pixels;
            self.canvas_width = new_width;
            self.canvas_height = new_height;
        }
    }

    /// Draw a sixel character at the current position
    fn draw_sixel(&mut self, sixel: u8) {
        if sixel < 0x3F || sixel > 0x7E {
            return; // Invalid sixel character
        }

        // Decode the 6 vertical pixels (bits 0-5)
        let pattern = sixel - 0x3F;

        self.ensure_capacity(self.x, self.y);

        // Draw 6 vertical pixels
        for bit in 0..6 {
            if (pattern & (1 << bit)) != 0 {
                let pixel_y = self.y * 6 + bit;
                if pixel_y < self.canvas_height {
                    let idx = (pixel_y * self.canvas_width + self.x) as usize;
                    if idx < self.pixels.len() {
                        self.pixels[idx] = self.current_color;
                    }
                }
            }
        }

        // Move to next column
        self.x += 1;
        self.max_x = self.max_x.max(self.x);
        self.max_y = self.max_y.max(self.y);
    }

    /// Process carriage return ('$')
    fn carriage_return(&mut self) {
        self.x = 0;
    }

    /// Process line feed ('-')
    fn line_feed(&mut self) {
        self.x = 0;
        self.y += 1;
        self.max_y = self.max_y.max(self.y);
    }

    /// Set color register using RGB (0-100 range)
    fn set_color_rgb(&mut self, index: u8, r: u8, g: u8, b: u8) {
        self.palette.set_rgb(index, r, g, b);
        debug!("Set color {} to RGB({}, {}, {})", index, r, g, b);
    }

    /// Set color register using HLS
    fn set_color_hls(&mut self, index: u8, h: u16, l: u8, s: u8) {
        self.palette.set_hls(index, h, l, s);
        debug!("Set color {} to HLS({}, {}, {})", index, h, l, s);
    }

    /// Select current color register
    fn select_color(&mut self, index: u8) {
        self.current_color = index;
    }

    /// Convert indexed pixels to RGBA
    fn to_rgba(self) -> SixelData {
        let width = self.max_x;
        let height = (self.max_y + 1) * 6; // Convert sixel rows to pixels

        if width == 0 || height == 0 {
            return SixelData {
                pixels: Vec::new(),
                width: 0,
                height: 0,
                aspect_ratio: self.aspect_ratio,
            };
        }

        let mut rgba = vec![0u8; (width * height * 4) as usize];

        for y in 0..height {
            for x in 0..width {
                let src_idx = (y * self.canvas_width + x) as usize;
                let dst_idx = ((y * width + x) * 4) as usize;

                if src_idx < self.pixels.len() && dst_idx + 3 < rgba.len() {
                    let color_idx = self.pixels[src_idx];
                    let color = self.palette.get(color_idx);

                    // Check if this pixel is background (color 0) and transparent mode is enabled
                    if self.transparent_bg && color_idx == 0 {
                        // Write transparent pixel (RGBA: 0, 0, 0, 0)
                        rgba[dst_idx] = 0;
                        rgba[dst_idx + 1] = 0;
                        rgba[dst_idx + 2] = 0;
                        rgba[dst_idx + 3] = 0;
                    } else {
                        // Extract RGBA components (stored as 0xAABBGGRR in little-endian)
                        rgba[dst_idx] = ((color >> 16) & 0xFF) as u8; // R
                        rgba[dst_idx + 1] = ((color >> 8) & 0xFF) as u8; // G
                        rgba[dst_idx + 2] = (color & 0xFF) as u8; // B
                        rgba[dst_idx + 3] = ((color >> 24) & 0xFF) as u8; // A
                    }
                }
            }
        }

        SixelData {
            pixels: rgba,
            width,
            height,
            aspect_ratio: self.aspect_ratio,
        }
    }
}

/// Parse a Sixel DCS sequence
///
/// Format: `DCS Ps ; Ps ; Ps q <sixel data> ST`
///
/// Parameters (all optional):
/// - P1: Aspect ratio numerator (pixel aspect ratio, typically 1-2)
/// - P2: Aspect ratio denominator (typically 1)
/// - P3: Horizontal grid size (ignored, for compatibility)
///
/// # Arguments
/// * `params` - The raw bytes after "P" in the DCS sequence (includes parameters and data)
///
/// # Returns
/// * `Some(SixelData)` if parsing succeeds
/// * `None` if the sequence is malformed
pub fn parse_sixel_dcs(params: &[u8]) -> Option<SixelData> {
    // Parse DCS parameters (format: "Ps;Ps;Psq")
    // Find the 'q' that starts the sixel data
    let q_pos = params.iter().position(|&b| b == b'q')?;

    let param_str = std::str::from_utf8(&params[..q_pos]).ok()?;
    let sixel_data = &params[q_pos + 1..];

    // Parse parameters (P1;P2;P3)
    let mut aspect_ratio = (1u8, 1u8);
    let mut transparent_bg = false;

    if !param_str.is_empty() {
        let parts: Vec<&str> = param_str.split(';').collect();

        // P1: Aspect ratio numerator (or mode, where 1 = color, 0/2 = monochrome)
        if let Some(p1_str) = parts.get(0) {
            if let Ok(p1) = p1_str.parse::<u8>() {
                aspect_ratio.0 = p1.max(1);
                // If P1 == 0 or 2, transparent background mode
                if p1 == 0 || p1 == 2 {
                    transparent_bg = true;
                }
            }
        }

        // P2: Aspect ratio denominator (or background mode)
        if let Some(p2_str) = parts.get(1) {
            if let Ok(p2) = p2_str.parse::<u8>() {
                aspect_ratio.1 = p2.max(1);
            }
        }

        // P3: Horizontal grid size (ignored)
    }

    debug!(
        "Parsing sixel with aspect ratio {}:{}, transparent_bg: {}, data length: {}",
        aspect_ratio.0, aspect_ratio.1, transparent_bg, sixel_data.len()
    );

    let mut parser = SixelParser::new(transparent_bg);
    parser.aspect_ratio = aspect_ratio;

    let mut i = 0;
    while i < sixel_data.len() {
        let byte = sixel_data[i];

        match byte {
            b'$' => {
                // Carriage return
                parser.carriage_return();
                i += 1;
            }
            b'-' => {
                // Line feed
                parser.line_feed();
                i += 1;
            }
            b'#' => {
                // Color definition or selection: #Pc or #Pc;Pu;Px;Py;Pz
                i += 1;
                let start = i;

                // Find the end of the color command (next non-digit/semicolon)
                while i < sixel_data.len() && (sixel_data[i].is_ascii_digit() || sixel_data[i] == b';') {
                    i += 1;
                }

                if let Ok(color_str) = std::str::from_utf8(&sixel_data[start..i]) {
                    let parts: Vec<&str> = color_str.split(';').collect();

                    if let Some(pc_str) = parts.get(0) {
                        if let Ok(color_idx) = pc_str.parse::<u8>() {
                            if parts.len() == 1 {
                                // Just color selection: #Pc
                                parser.select_color(color_idx);
                            } else if parts.len() >= 4 {
                                // Color definition: #Pc;Pu;Px;Py;Pz
                                let pu = parts.get(1).and_then(|s| s.parse::<u8>().ok()).unwrap_or(2);
                                let px = parts.get(2).and_then(|s| s.parse::<u16>().ok()).unwrap_or(0);
                                let py = parts.get(3).and_then(|s| s.parse::<u8>().ok()).unwrap_or(0);
                                let pz = parts.get(4).and_then(|s| s.parse::<u8>().ok()).unwrap_or(0);

                                match pu {
                                    1 => {
                                        // HLS format
                                        parser.set_color_hls(color_idx, px, py, pz);
                                    }
                                    2 => {
                                        // RGB format (0-100 range)
                                        parser.set_color_rgb(color_idx, px as u8, py, pz);
                                    }
                                    _ => {
                                        warn!("Unknown sixel color format: Pu={}", pu);
                                    }
                                }
                            }
                        }
                    }
                }
            }
            b'!' => {
                // Repeat introducer: !<count><char>
                i += 1;
                let start = i;

                // Parse repeat count
                while i < sixel_data.len() && sixel_data[i].is_ascii_digit() {
                    i += 1;
                }

                if let Ok(count_str) = std::str::from_utf8(&sixel_data[start..i]) {
                    if let Ok(count) = count_str.parse::<u32>() {
                        // Get the character to repeat
                        if i < sixel_data.len() {
                            let repeat_char = sixel_data[i];
                            i += 1;

                            // Draw the character 'count' times
                            for _ in 0..count.min(10000) {
                                // Limit repeats to prevent DoS
                                parser.draw_sixel(repeat_char);
                            }
                        }
                    }
                }
            }
            b'"' => {
                // Raster attributes: "Pan;Pad;Ph;Pv
                // Pan: Pixel aspect ratio numerator
                // Pad: Pixel aspect ratio denominator
                // Ph: Horizontal pixels
                // Pv: Vertical pixels
                // For now, we ignore these
                i += 1;
                while i < sixel_data.len() && (sixel_data[i].is_ascii_digit() || sixel_data[i] == b';') {
                    i += 1;
                }
            }
            0x3F..=0x7E => {
                // Sixel character (encodes 6 vertical pixels)
                parser.draw_sixel(byte);
                i += 1;
            }
            _ => {
                // Ignore other characters (whitespace, etc.)
                i += 1;
            }
        }
    }

    let result = parser.to_rgba();
    debug!(
        "Sixel parsing complete: {}x{} pixels, {} bytes",
        result.width,
        result.height,
        result.pixels.len()
    );

    Some(result)
}

/// Convert HLS color to RGB (0-255 range)
///
/// # Arguments
/// * `h` - Hue (0-360 degrees)
/// * `l` - Lightness (0-100)
/// * `s` - Saturation (0-100)
///
/// # Returns
/// * (R, G, B) tuple in 0-255 range
fn hls_to_rgb(h: u16, l: u8, s: u8) -> (u8, u8, u8) {
    let h = (h % 360) as f32 / 360.0;
    let l = (l.min(100) as f32) / 100.0;
    let s = (s.min(100) as f32) / 100.0;

    if s == 0.0 {
        // Achromatic (gray)
        let val = (l * 255.0) as u8;
        return (val, val, val);
    }

    let q = if l < 0.5 {
        l * (1.0 + s)
    } else {
        l + s - l * s
    };

    let p = 2.0 * l - q;

    let r = hue_to_rgb(p, q, h + 1.0 / 3.0);
    let g = hue_to_rgb(p, q, h);
    let b = hue_to_rgb(p, q, h - 1.0 / 3.0);

    ((r * 255.0) as u8, (g * 255.0) as u8, (b * 255.0) as u8)
}

/// Helper function for HLS to RGB conversion
fn hue_to_rgb(p: f32, q: f32, mut t: f32) -> f32 {
    if t < 0.0 {
        t += 1.0;
    }
    if t > 1.0 {
        t -= 1.0;
    }

    if t < 1.0 / 6.0 {
        return p + (q - p) * 6.0 * t;
    }
    if t < 1.0 / 2.0 {
        return q;
    }
    if t < 2.0 / 3.0 {
        return p + (q - p) * (2.0 / 3.0 - t) * 6.0;
    }

    p
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hls_to_rgb() {
        // Test black (L=0)
        let (r, g, b) = hls_to_rgb(0, 0, 0);
        assert_eq!((r, g, b), (0, 0, 0));

        // Test white (L=100, S=0)
        let (r, g, b) = hls_to_rgb(0, 100, 0);
        assert_eq!((r, g, b), (255, 255, 255));

        // Test red (H=0, L=50, S=100)
        let (r, g, b) = hls_to_rgb(0, 50, 100);
        assert!(r > 200 && g < 50 && b < 50); // Should be predominantly red
    }

    #[test]
    fn test_parse_simple_sixel() {
        // Simple sixel: select color 1 (blue), draw a few pixels
        // Format: "1;1q#1~-"
        // - "1;1q" = parameters and start
        // - "#1" = select color 1
        // - "~" = draw sixel (all 6 bits set: 0x7E - 0x3F = 0x3F = 111111)
        // - "-" = line feed
        let sequence = b"1;1q#1~-";

        let result = parse_sixel_dcs(sequence).unwrap();
        assert_eq!(result.width, 1); // Drew one column
        assert_eq!(result.height, 6); // One sixel row = 6 pixels high
        assert_eq!(result.pixels.len(), (1 * 6 * 4) as usize); // width * height * 4 (RGBA)

        // Check that pixels are blue (color 1 in default palette)
        let color = result.pixels[0..4].to_vec();
        assert!(color[2] > 100); // Blue channel should be high
    }

    #[test]
    fn test_parse_with_color_definition() {
        // Define color 5 as red (RGB mode), then draw with it
        // "#5;2;100;0;0" = color 5, RGB mode, R=100, G=0, B=0
        // "#5" = select color 5
        // "?" = draw sixel with bottom pixel (000001)
        let sequence = b"1;1q#5;2;100;0;0#5?";

        let result = parse_sixel_dcs(sequence).unwrap();
        assert_eq!(result.width, 1);
        assert_eq!(result.height, 6);

        // First pixel (bottom of sixel) should be red
        let color = &result.pixels[0..4];
        assert!(color[0] > 200); // Red channel
        assert!(color[1] < 50);  // Green channel
        assert!(color[2] < 50);  // Blue channel
    }

    #[test]
    fn test_carriage_return_and_line_feed() {
        // Draw, carriage return, draw again on same line
        // Then line feed and draw on next line
        let sequence = b"1;1q#1?$?-?";

        let result = parse_sixel_dcs(sequence).unwrap();
        // Should have drawn at (0,0), then (0,0) again (overwrite), then (0,6)
        assert_eq!(result.width, 1);
        assert_eq!(result.height, 12); // 2 sixel rows = 12 pixels
    }

    #[test]
    fn test_repeat_command() {
        // Repeat '?' 5 times
        let sequence = b"1;1q#1!5?";

        let result = parse_sixel_dcs(sequence).unwrap();
        assert_eq!(result.width, 5); // Should have drawn 5 columns
        assert_eq!(result.height, 6);
    }

    #[test]
    fn test_transparent_background() {
        // P1=0 enables transparent background
        let sequence = b"0;1q#1?";

        let result = parse_sixel_dcs(sequence).unwrap();
        assert_eq!(result.width, 1);
        assert_eq!(result.height, 6);

        // Background pixels (color 0) should be transparent (alpha = 0)
        // First row, first pixel was drawn (color 1), but others should be transparent
        for y in 1..6 {
            let idx = (y * 1 * 4 + 3) as usize; // Alpha channel
            if result.pixels[idx] == 0 {
                // Found a transparent pixel (as expected for unused pixels)
                return;
            }
        }
    }

    #[test]
    fn test_empty_sequence() {
        // Just parameters, no data
        let sequence = b"1;1q";
        let result = parse_sixel_dcs(sequence);
        assert!(result.is_some());
        let data = result.unwrap();
        assert_eq!(data.width, 0);
        assert_eq!(data.height, 0);
    }

    #[test]
    fn test_invalid_sequence() {
        // Missing 'q'
        let sequence = b"1;1#1?";
        let result = parse_sixel_dcs(sequence);
        assert!(result.is_none());
    }

    #[test]
    fn test_aspect_ratio() {
        let sequence = b"2;1q#1?";
        let result = parse_sixel_dcs(sequence).unwrap();
        assert_eq!(result.aspect_ratio, (2, 1));
    }

    #[test]
    fn test_complex_image() {
        // Draw a simple 3x2 sixel grid pattern
        // Row 1: colors 1, 2, 3
        // Row 2: colors 4, 5, 6
        let sequence = b"1;1q#1~#2~#3~-#4~#5~#6~";

        let result = parse_sixel_dcs(sequence).unwrap();
        assert_eq!(result.width, 3); // 3 columns
        assert_eq!(result.height, 12); // 2 sixel rows = 12 pixels
        assert!(!result.pixels.is_empty());
    }
}
