//! Image Format Detection and Metadata Extraction
//!
//! Detects image format from raw bytes and extracts dimensions.
//! Supports PNG, JPEG, and GIF formats without full decoding.

use log::warn;

/// Image format detection result
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImageFormat {
    Png,
    Jpeg,
    Gif,
    Rgba,    // Raw RGBA pixel data (used for Sixel)
    Unknown,
}

impl ImageFormat {
    /// Convert to protocol format enum value
    pub fn to_protocol_u8(self) -> u8 {
        match self {
            ImageFormat::Png => 0,
            ImageFormat::Jpeg => 1,
            ImageFormat::Gif => 2,
            ImageFormat::Rgba => 3,
            ImageFormat::Unknown => 0, // Fallback to PNG
        }
    }
}

/// Image metadata extracted from header
#[derive(Debug, Clone, Copy)]
pub struct ImageMetadata {
    pub format: ImageFormat,
    pub width: u32,
    pub height: u32,
}

/// Detect image format and extract dimensions from raw bytes
///
/// This performs minimal header parsing to avoid full image decoding.
///
/// # Arguments
/// * `data` - Raw image bytes
///
/// # Returns
/// * `Some(ImageMetadata)` if format is recognized and dimensions extracted
/// * `None` if format is unrecognized or header is malformed
pub fn detect_image(data: &[u8]) -> Option<ImageMetadata> {
    if data.len() < 10 {
        return None;
    }

    // Check PNG signature
    if data.len() >= 24 && &data[0..8] == b"\x89PNG\r\n\x1a\n" {
        return parse_png_dimensions(data);
    }

    // Check JPEG signature
    if data.len() >= 2 && data[0] == 0xFF && data[1] == 0xD8 {
        return parse_jpeg_dimensions(data);
    }

    // Check GIF signature
    if data.len() >= 10 && (&data[0..3] == b"GIF" && (&data[3..6] == b"87a" || &data[3..6] == b"89a")) {
        return parse_gif_dimensions(data);
    }

    warn!("Unknown image format (first {} bytes: {:?})", data.len().min(16), &data[..data.len().min(16)]);
    None
}

/// Parse PNG dimensions from IHDR chunk
fn parse_png_dimensions(data: &[u8]) -> Option<ImageMetadata> {
    // PNG structure: 8-byte signature + IHDR chunk
    // IHDR starts at byte 8:
    //   - 4 bytes: chunk length (should be 13 for IHDR)
    //   - 4 bytes: "IHDR"
    //   - 4 bytes: width (big-endian)
    //   - 4 bytes: height (big-endian)

    if data.len() < 24 {
        return None;
    }

    // Verify IHDR chunk
    if &data[12..16] != b"IHDR" {
        warn!("PNG missing IHDR chunk at expected position");
        return None;
    }

    // Extract width and height (big-endian u32)
    let width = u32::from_be_bytes([data[16], data[17], data[18], data[19]]);
    let height = u32::from_be_bytes([data[20], data[21], data[22], data[23]]);

    Some(ImageMetadata {
        format: ImageFormat::Png,
        width,
        height,
    })
}

/// Parse JPEG dimensions from SOF marker
fn parse_jpeg_dimensions(data: &[u8]) -> Option<ImageMetadata> {
    // JPEG structure: sequence of markers
    // We need to find SOF0 (0xFFC0) or SOF2 (0xFFC2) marker
    // Format: FF C0/C2 <length> <precision> <height> <width>

    let mut pos = 2; // Skip initial 0xFFD8

    while pos + 9 < data.len() {
        // Look for marker (0xFF followed by non-0xFF)
        if data[pos] != 0xFF {
            pos += 1;
            continue;
        }

        let marker = data[pos + 1];

        // Skip padding bytes (0xFF 0xFF)
        if marker == 0xFF {
            pos += 1;
            continue;
        }

        // Check if this is a SOF marker (Start Of Frame)
        // SOF0 = 0xC0, SOF1 = 0xC1, SOF2 = 0xC2 (progressive)
        if marker == 0xC0 || marker == 0xC1 || marker == 0xC2 {
            // SOF structure:
            // +0: 0xFF
            // +1: marker
            // +2-3: length (big-endian, includes length bytes)
            // +4: precision (bits per sample)
            // +5-6: height (big-endian)
            // +7-8: width (big-endian)

            if pos + 9 > data.len() {
                break;
            }

            let height = u16::from_be_bytes([data[pos + 5], data[pos + 6]]) as u32;
            let width = u16::from_be_bytes([data[pos + 7], data[pos + 8]]) as u32;

            return Some(ImageMetadata {
                format: ImageFormat::Jpeg,
                width,
                height,
            });
        }

        // Skip to next marker
        if pos + 3 < data.len() {
            let length = u16::from_be_bytes([data[pos + 2], data[pos + 3]]) as usize;
            pos += 2 + length;
        } else {
            break;
        }
    }

    warn!("JPEG SOF marker not found");
    None
}

/// Parse GIF dimensions from logical screen descriptor
fn parse_gif_dimensions(data: &[u8]) -> Option<ImageMetadata> {
    // GIF structure:
    // +0-2: "GIF"
    // +3-5: version ("87a" or "89a")
    // +6-7: width (little-endian)
    // +8-9: height (little-endian)

    if data.len() < 10 {
        return None;
    }

    let width = u16::from_le_bytes([data[6], data[7]]) as u32;
    let height = u16::from_le_bytes([data[8], data[9]]) as u32;

    Some(ImageMetadata {
        format: ImageFormat::Gif,
        width,
        height,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_png() {
        // Minimal PNG header with IHDR
        let png_header = [
            0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, // PNG signature
            0x00, 0x00, 0x00, 0x0D, // IHDR length (13)
            0x49, 0x48, 0x44, 0x52, // "IHDR"
            0x00, 0x00, 0x01, 0x00, // Width = 256
            0x00, 0x00, 0x00, 0x80, // Height = 128
        ];

        let meta = detect_image(&png_header).unwrap();
        assert_eq!(meta.format, ImageFormat::Png);
        assert_eq!(meta.width, 256);
        assert_eq!(meta.height, 128);
    }

    #[test]
    fn test_detect_jpeg() {
        // Minimal JPEG with SOF0 marker
        let jpeg_header = [
            0xFF, 0xD8, // SOI
            0xFF, 0xC0, // SOF0
            0x00, 0x11, // Length
            0x08,       // Precision
            0x01, 0x00, // Height = 256
            0x01, 0x40, // Width = 320
            0x03,       // Components
            0x00, 0x00, 0x00, // Padding to ensure length >= 10
        ];

        let meta = detect_image(&jpeg_header).unwrap();
        assert_eq!(meta.format, ImageFormat::Jpeg);
        assert_eq!(meta.width, 320);
        assert_eq!(meta.height, 256);
    }

    #[test]
    fn test_detect_gif() {
        // Minimal GIF header (exact 10 bytes needed)
        let gif_header = [
            0x47, 0x49, 0x46, // "GIF"
            0x38, 0x39, 0x61, // "89a"
            0x40, 0x01,       // Width = 320 (little-endian)
            0x00, 0x01,       // Height = 256 (little-endian)
        ];

        let meta = detect_image(&gif_header).unwrap();
        assert_eq!(meta.format, ImageFormat::Gif);
        assert_eq!(meta.width, 320);
        assert_eq!(meta.height, 256);
    }

    #[test]
    fn test_unknown_format() {
        let unknown = [0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09];
        assert!(detect_image(&unknown).is_none());
    }

    #[test]
    fn test_too_short() {
        let short = [0x89, 0x50, 0x4E, 0x47];
        assert!(detect_image(&short).is_none());
    }
}
