//! iTerm2 Image Protocol Parser
//!
//! Implements the iTerm2 inline image protocol (OSC 1337).
//!
//! Format: `OSC 1337 ; File=[arguments]:[base64-encoded file contents] ST`
//!
//! Supported arguments:
//! - name=<base64-encoded filename>
//! - size=<file size in bytes>
//! - width=<width> (auto, N, Npx, N%)
//! - height=<height> (auto, N, Npx, N%)
//! - preserveAspectRatio=<0 or 1>
//! - inline=<0 or 1>
//! - doNotMoveCursor=<0 or 1>

use base64::{engine::general_purpose::STANDARD, Engine as _};
use log::{debug, warn};

/// Image size specification
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ImageSize {
    /// Automatically determine size
    Auto,
    /// Size in terminal cells
    Cells(u16),
    /// Size in pixels
    Pixels(u32),
    /// Size as percentage of terminal dimensions
    Percent(f32),
}

impl Default for ImageSize {
    fn default() -> Self {
        Self::Auto
    }
}

/// Parsed image data from iTerm2 protocol
#[derive(Debug, Clone)]
pub struct ImageData {
    /// Decoded image bytes (PNG, JPEG, GIF, etc.)
    pub data: Vec<u8>,
    /// Width specification
    pub width: ImageSize,
    /// Height specification
    pub height: ImageSize,
    /// Whether to preserve aspect ratio when resizing
    pub preserve_aspect_ratio: bool,
    /// Whether image should be displayed inline
    pub inline: bool,
    /// Whether cursor should move after displaying image
    pub do_not_move_cursor: bool,
    /// Original filename (base64-decoded)
    pub filename: Option<String>,
}

impl Default for ImageData {
    fn default() -> Self {
        Self {
            data: Vec::new(),
            width: ImageSize::Auto,
            height: ImageSize::Auto,
            preserve_aspect_ratio: true,
            inline: true,
            do_not_move_cursor: false,
            filename: None,
        }
    }
}

/// Parse an iTerm2 image sequence
///
/// Format: `File=name=<b64>;size=N;width=W;height=H;inline=1:<base64 data>`
///
/// # Arguments
/// * `params` - The raw bytes after "1337;" in the OSC sequence
///
/// # Returns
/// * `Some(ImageData)` if parsing succeeds
/// * `None` if the sequence is malformed
pub fn parse_iterm2_image(params: &[u8]) -> Option<ImageData> {
    // Convert to string for easier parsing
    let params_str = std::str::from_utf8(params).ok()?;

    // Check for "File=" prefix
    if !params_str.starts_with("File=") {
        debug!("iTerm2 image sequence missing 'File=' prefix");
        return None;
    }

    // Remove "File=" prefix
    let content = &params_str[5..];

    // Split on ':' to separate arguments from base64 data
    let parts: Vec<&str> = content.splitn(2, ':').collect();
    if parts.len() != 2 {
        warn!("iTerm2 image sequence missing ':' separator");
        return None;
    }

    let (args_str, base64_data) = (parts[0], parts[1]);

    // Parse key=value arguments
    let mut image_data = ImageData::default();

    for arg in args_str.split(';') {
        if arg.is_empty() {
            continue;
        }

        let kv: Vec<&str> = arg.splitn(2, '=').collect();
        if kv.len() != 2 {
            debug!("Skipping malformed argument: {}", arg);
            continue;
        }

        let (key, value) = (kv[0], kv[1]);

        match key {
            "name" => {
                // Filename is base64-encoded
                if let Ok(decoded) = STANDARD.decode(value) {
                    if let Ok(filename) = String::from_utf8(decoded) {
                        image_data.filename = Some(filename);
                    }
                }
            }
            "width" => {
                image_data.width = parse_image_size(value);
            }
            "height" => {
                image_data.height = parse_image_size(value);
            }
            "preserveAspectRatio" => {
                image_data.preserve_aspect_ratio = value == "1";
            }
            "inline" => {
                image_data.inline = value == "1";
            }
            "doNotMoveCursor" => {
                image_data.do_not_move_cursor = value == "1";
            }
            "size" => {
                // File size in bytes - we could validate against decoded data
                // but for now we'll just ignore it
                debug!("Image size hint: {} bytes", value);
            }
            _ => {
                debug!("Unknown iTerm2 image argument: {}", key);
            }
        }
    }

    // Decode base64 image data
    match STANDARD.decode(base64_data) {
        Ok(data) => {
            if data.is_empty() {
                warn!("iTerm2 image has empty data");
                return None;
            }
            image_data.data = data;
            Some(image_data)
        }
        Err(e) => {
            warn!("Failed to decode iTerm2 image base64 data: {}", e);
            None
        }
    }
}

/// Parse an image size specification
///
/// Formats:
/// - "auto" -> Auto
/// - "N" -> Cells(N)
/// - "Npx" -> Pixels(N)
/// - "N%" -> Percent(N)
fn parse_image_size(s: &str) -> ImageSize {
    let s = s.trim();

    if s.eq_ignore_ascii_case("auto") {
        return ImageSize::Auto;
    }

    // Check for percent
    if s.ends_with('%') {
        if let Ok(percent) = s[..s.len() - 1].parse::<f32>() {
            return ImageSize::Percent(percent);
        }
    }

    // Check for pixels
    if s.ends_with("px") {
        if let Ok(pixels) = s[..s.len() - 2].parse::<u32>() {
            return ImageSize::Pixels(pixels);
        }
    }

    // Default to cells
    if let Ok(cells) = s.parse::<u16>() {
        return ImageSize::Cells(cells);
    }

    // Fallback to auto if we can't parse
    debug!("Could not parse image size: {}, defaulting to auto", s);
    ImageSize::Auto
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_image_size() {
        assert_eq!(parse_image_size("auto"), ImageSize::Auto);
        assert_eq!(parse_image_size("Auto"), ImageSize::Auto);
        assert_eq!(parse_image_size("10"), ImageSize::Cells(10));
        assert_eq!(parse_image_size("100px"), ImageSize::Pixels(100));
        assert_eq!(parse_image_size("50%"), ImageSize::Percent(50.0));
        assert_eq!(parse_image_size("invalid"), ImageSize::Auto);
    }

    #[test]
    fn test_parse_simple_image() {
        // Simple 1x1 red PNG (base64 encoded)
        let png_data = "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mP8z8DwHwAFBQIAX8jx0gAAAABJRU5ErkJggg==";
        let sequence = format!("File=inline=1:{}", png_data);

        let image = parse_iterm2_image(sequence.as_bytes()).unwrap();
        assert!(image.inline);
        assert!(!image.data.is_empty());
        assert_eq!(image.width, ImageSize::Auto);
        assert_eq!(image.height, ImageSize::Auto);
    }

    #[test]
    fn test_parse_with_dimensions() {
        let png_data = "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mP8z8DwHwAFBQIAX8jx0gAAAABJRU5ErkJggg==";
        let sequence = format!("File=width=10;height=auto;inline=1:{}", png_data);

        let image = parse_iterm2_image(sequence.as_bytes()).unwrap();
        assert_eq!(image.width, ImageSize::Cells(10));
        assert_eq!(image.height, ImageSize::Auto);
        assert!(image.inline);
    }

    #[test]
    fn test_parse_with_pixels_and_percent() {
        let png_data = "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mP8z8DwHwAFBQIAX8jx0gAAAABJRU5ErkJggg==";
        let sequence = format!("File=width=200px;height=50%;inline=1:{}", png_data);

        let image = parse_iterm2_image(sequence.as_bytes()).unwrap();
        assert_eq!(image.width, ImageSize::Pixels(200));
        assert_eq!(image.height, ImageSize::Percent(50.0));
    }

    #[test]
    fn test_parse_with_filename() {
        let png_data = "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mP8z8DwHwAFBQIAX8jx0gAAAABJRU5ErkJggg==";
        // "test.png" encoded in base64
        let filename_b64 = "dGVzdC5wbmc=";
        let sequence = format!("File=name={};inline=1:{}", filename_b64, png_data);

        let image = parse_iterm2_image(sequence.as_bytes()).unwrap();
        assert_eq!(image.filename.as_deref(), Some("test.png"));
    }

    #[test]
    fn test_parse_preserve_aspect_ratio() {
        let png_data = "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mP8z8DwHwAFBQIAX8jx0gAAAABJRU5ErkJggg==";
        let sequence = format!("File=preserveAspectRatio=0;inline=1:{}", png_data);

        let image = parse_iterm2_image(sequence.as_bytes()).unwrap();
        assert!(!image.preserve_aspect_ratio);
    }

    #[test]
    fn test_parse_do_not_move_cursor() {
        let png_data = "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mP8z8DwHwAFBQIAX8jx0gAAAABJRU5ErkJggg==";
        let sequence = format!("File=doNotMoveCursor=1;inline=1:{}", png_data);

        let image = parse_iterm2_image(sequence.as_bytes()).unwrap();
        assert!(image.do_not_move_cursor);
    }

    #[test]
    fn test_parse_missing_file_prefix() {
        let result = parse_iterm2_image(b"inline=1:somedata");
        assert!(result.is_none());
    }

    #[test]
    fn test_parse_missing_separator() {
        let result = parse_iterm2_image(b"File=inline=1");
        assert!(result.is_none());
    }

    #[test]
    fn test_parse_invalid_base64() {
        let result = parse_iterm2_image(b"File=inline=1:!!!invalid-base64!!!");
        assert!(result.is_none());
    }

    #[test]
    fn test_parse_empty_data() {
        let result = parse_iterm2_image(b"File=inline=1:");
        assert!(result.is_none());
    }

    #[test]
    fn test_parse_complex_sequence() {
        let png_data = "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mP8z8DwHwAFBQIAX8jx0gAAAABJRU5ErkJggg==";
        let filename_b64 = "dGVzdC5wbmc=";
        let sequence = format!(
            "File=name={};size=68;width=10;height=5;preserveAspectRatio=1;inline=1;doNotMoveCursor=0:{}",
            filename_b64, png_data
        );

        let image = parse_iterm2_image(sequence.as_bytes()).unwrap();
        assert_eq!(image.filename.as_deref(), Some("test.png"));
        assert_eq!(image.width, ImageSize::Cells(10));
        assert_eq!(image.height, ImageSize::Cells(5));
        assert!(image.preserve_aspect_ratio);
        assert!(image.inline);
        assert!(!image.do_not_move_cursor);
        assert!(!image.data.is_empty());
    }
}
