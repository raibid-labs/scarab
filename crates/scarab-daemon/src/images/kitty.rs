//! Kitty Graphics Protocol Parser
//!
//! Implements the Kitty terminal graphics protocol.
//!
//! Format: `ESC _ G <key>=<value>,...;<key>=<value> ; <base64-payload> ESC \`
//!
//! Key commands:
//! - a=t/T/p/d - action (transmit, transmit+display, put, delete)
//! - f=24/32/100 - format (RGB, RGBA, PNG)
//! - t=d/f/t/s - transmission medium (direct, file, temp file, shared mem)
//! - m=0/1 - more chunks (0=final, 1=more coming)
//! - i=N - image ID for later reference
//! - p=N - placement ID
//! - s=W - source width in pixels
//! - v=H - source height in pixels
//! - c=W - columns to display
//! - r=H - rows to display
//! - x=N - x-offset in pixels within source
//! - y=N - y-offset in pixels within source
//! - X=N - x-position on terminal grid (default: cursor column)
//! - Y=N - y-position on terminal grid (default: cursor row)
//! - z=N - z-index for stacking (default: 0)

use base64::{engine::general_purpose::STANDARD, Engine as _};
use log::{debug, warn};
use std::collections::HashMap;

/// Image format for Kitty protocol
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KittyImageFormat {
    /// RGB format (24-bit)
    Rgb,
    /// RGBA format (32-bit)
    Rgba,
    /// PNG format
    Png,
}

impl KittyImageFormat {
    /// Parse format from Kitty 'f' parameter
    fn from_code(code: u32) -> Option<Self> {
        match code {
            24 => Some(Self::Rgb),
            32 => Some(Self::Rgba),
            100 => Some(Self::Png),
            _ => None,
        }
    }

    /// Convert to protocol format value
    pub fn to_protocol_u8(self) -> u8 {
        match self {
            Self::Png => 0,
            Self::Rgb => 3,  // Raw RGB data
            Self::Rgba => 3, // Raw RGBA data (treated as RGB in protocol)
        }
    }
}

/// Kitty protocol action
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KittyAction {
    /// Transmit image data (a=t)
    Transmit,
    /// Transmit and display (a=T)
    TransmitAndDisplay,
    /// Display previously transmitted image (a=p)
    Put,
    /// Delete image (a=d)
    Delete,
}

impl KittyAction {
    fn from_char(c: char) -> Option<Self> {
        match c {
            't' => Some(Self::Transmit),
            'T' => Some(Self::TransmitAndDisplay),
            'p' => Some(Self::Put),
            'd' => Some(Self::Delete),
            _ => None,
        }
    }
}

/// Transmission medium
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransmissionMedium {
    /// Direct transmission (base64 in command)
    Direct,
    /// File path
    File,
    /// Temporary file
    TempFile,
    /// Shared memory
    SharedMem,
}

impl TransmissionMedium {
    fn from_char(c: char) -> Option<Self> {
        match c {
            'd' => Some(Self::Direct),
            'f' => Some(Self::File),
            't' => Some(Self::TempFile),
            's' => Some(Self::SharedMem),
            _ => None,
        }
    }
}

/// A complete Kitty graphics command
#[derive(Debug, Clone)]
pub struct KittyCommand {
    /// Action to perform
    pub action: KittyAction,
    /// Image format
    pub format: KittyImageFormat,
    /// Transmission medium
    pub medium: TransmissionMedium,
    /// More chunks coming (true if m=1)
    pub more_chunks: bool,
    /// Image ID for reference
    pub image_id: Option<u32>,
    /// Placement ID
    pub placement_id: Option<u32>,
    /// Source image width in pixels
    pub source_width: Option<u32>,
    /// Source image height in pixels
    pub source_height: Option<u32>,
    /// Display width in terminal columns
    pub display_columns: Option<u16>,
    /// Display height in terminal rows
    pub display_rows: Option<u16>,
    /// X offset in pixels within source image
    pub offset_x: Option<u32>,
    /// Y offset in pixels within source image
    pub offset_y: Option<u32>,
    /// X position on terminal grid (column)
    pub grid_x: Option<u16>,
    /// Y position on terminal grid (row)
    pub grid_y: Option<u16>,
    /// Z-index for stacking
    pub z_index: i32,
    /// Decoded payload data
    pub payload: Vec<u8>,
}

impl Default for KittyCommand {
    fn default() -> Self {
        Self {
            action: KittyAction::TransmitAndDisplay,
            format: KittyImageFormat::Png,
            medium: TransmissionMedium::Direct,
            more_chunks: false,
            image_id: None,
            placement_id: None,
            source_width: None,
            source_height: None,
            display_columns: None,
            display_rows: None,
            offset_x: None,
            offset_y: None,
            grid_x: None,
            grid_y: None,
            z_index: 0,
            payload: Vec::new(),
        }
    }
}

/// State manager for chunked image transfers
#[derive(Debug, Default)]
pub struct ChunkedTransferState {
    /// Incomplete transfers keyed by image ID
    transfers: HashMap<u32, Vec<u8>>,
}

impl ChunkedTransferState {
    /// Create a new empty state
    pub fn new() -> Self {
        Self {
            transfers: HashMap::new(),
        }
    }

    /// Add a chunk to an ongoing transfer
    ///
    /// Returns `Some(complete_data)` when transfer is complete (m=0)
    pub fn add_chunk(&mut self, image_id: u32, chunk: Vec<u8>, is_final: bool) -> Option<Vec<u8>> {
        if is_final {
            // Final chunk - append and return complete data
            if let Some(mut existing) = self.transfers.remove(&image_id) {
                existing.extend_from_slice(&chunk);
                debug!(
                    "Completed chunked transfer for image {}, total size: {} bytes",
                    image_id,
                    existing.len()
                );
                Some(existing)
            } else {
                // No previous chunks, this is the only chunk
                debug!(
                    "Single-chunk transfer for image {}, size: {} bytes",
                    image_id,
                    chunk.len()
                );
                Some(chunk)
            }
        } else {
            // Not final - accumulate chunk
            self.transfers
                .entry(image_id)
                .or_insert_with(Vec::new)
                .extend_from_slice(&chunk);
            debug!(
                "Added chunk to image {}, current size: {} bytes",
                image_id,
                self.transfers[&image_id].len()
            );
            None
        }
    }

    /// Clear all incomplete transfers
    pub fn clear(&mut self) {
        self.transfers.clear();
    }

    /// Get the number of incomplete transfers
    pub fn pending_count(&self) -> usize {
        self.transfers.len()
    }
}

/// Parse a Kitty graphics protocol sequence
///
/// Format: `G<key>=<value>,...;<key>=<value>;<base64-payload>`
///
/// The input should be everything after the `ESC _` APC introducer and before
/// the `ESC \` terminator, excluding the leading 'G'.
///
/// # Arguments
/// * `params` - The raw bytes after "G" in the APC sequence
///
/// # Returns
/// * `Some(KittyCommand)` if parsing succeeds
/// * `None` if the sequence is malformed
pub fn parse_kitty_graphics(params: &[u8]) -> Option<KittyCommand> {
    // Convert to string for parsing
    let params_str = std::str::from_utf8(params).ok()?;

    // Split on ';' - last part is optional base64 payload
    let parts: Vec<&str> = params_str.split(';').collect();

    if parts.is_empty() {
        debug!("Kitty graphics sequence is empty");
        return None;
    }

    let mut cmd = KittyCommand::default();

    // Parse key=value pairs
    // All parts except the last might be key-value pairs
    // The last part could be either a key-value pair or base64 data
    let mut payload_str = "";

    for (idx, part) in parts.iter().enumerate() {
        if part.is_empty() {
            continue;
        }

        // Check if this looks like a key=value pair
        if part.contains('=') {
            // Parse as key-value
            for kv_pair in part.split(',') {
                if kv_pair.is_empty() {
                    continue;
                }

                let kv: Vec<&str> = kv_pair.splitn(2, '=').collect();
                if kv.len() != 2 {
                    debug!("Skipping malformed key-value: {}", kv_pair);
                    continue;
                }

                let (key, value) = (kv[0], kv[1]);
                parse_key_value(&mut cmd, key, value);
            }
        } else if idx == parts.len() - 1 {
            // Last part without '=' is likely base64 payload
            payload_str = part;
        }
    }

    // Decode base64 payload if present
    if !payload_str.is_empty() {
        match STANDARD.decode(payload_str) {
            Ok(data) => {
                cmd.payload = data;
            }
            Err(e) => {
                warn!("Failed to decode Kitty graphics payload: {}", e);
                return None;
            }
        }
    }

    Some(cmd)
}

/// Parse a single key=value pair into the command
fn parse_key_value(cmd: &mut KittyCommand, key: &str, value: &str) {
    match key {
        "a" => {
            if let Some(c) = value.chars().next() {
                if let Some(action) = KittyAction::from_char(c) {
                    cmd.action = action;
                }
            }
        }
        "f" => {
            if let Ok(code) = value.parse::<u32>() {
                if let Some(format) = KittyImageFormat::from_code(code) {
                    cmd.format = format;
                }
            }
        }
        "t" => {
            if let Some(c) = value.chars().next() {
                if let Some(medium) = TransmissionMedium::from_char(c) {
                    cmd.medium = medium;
                }
            }
        }
        "m" => {
            cmd.more_chunks = value == "1";
        }
        "i" => {
            if let Ok(id) = value.parse::<u32>() {
                cmd.image_id = Some(id);
            }
        }
        "p" => {
            if let Ok(id) = value.parse::<u32>() {
                cmd.placement_id = Some(id);
            }
        }
        "s" => {
            if let Ok(w) = value.parse::<u32>() {
                cmd.source_width = Some(w);
            }
        }
        "v" => {
            if let Ok(h) = value.parse::<u32>() {
                cmd.source_height = Some(h);
            }
        }
        "c" => {
            if let Ok(w) = value.parse::<u16>() {
                cmd.display_columns = Some(w);
            }
        }
        "r" => {
            if let Ok(h) = value.parse::<u16>() {
                cmd.display_rows = Some(h);
            }
        }
        "x" => {
            if let Ok(x) = value.parse::<u32>() {
                cmd.offset_x = Some(x);
            }
        }
        "y" => {
            if let Ok(y) = value.parse::<u32>() {
                cmd.offset_y = Some(y);
            }
        }
        "X" => {
            if let Ok(x) = value.parse::<u16>() {
                cmd.grid_x = Some(x);
            }
        }
        "Y" => {
            if let Ok(y) = value.parse::<u16>() {
                cmd.grid_y = Some(y);
            }
        }
        "z" => {
            if let Ok(z) = value.parse::<i32>() {
                cmd.z_index = z;
            }
        }
        _ => {
            debug!("Unknown Kitty graphics key: {}", key);
        }
    }
}

/// Convert raw RGB/RGBA data to PNG format
///
/// This is needed because Kitty can send raw pixel data (f=24 or f=32)
/// but we want to store everything as PNG for consistency.
///
/// # Arguments
/// * `data` - Raw pixel data (RGB or RGBA)
/// * `width` - Image width in pixels
/// * `height` - Image height in pixels
/// * `format` - Pixel format (RGB or RGBA)
///
/// # Returns
/// * `Some(png_bytes)` if encoding succeeds
/// * `None` if encoding fails or parameters are invalid
pub fn convert_raw_to_png(
    data: &[u8],
    width: u32,
    height: u32,
    format: KittyImageFormat,
) -> Option<Vec<u8>> {
    use std::io::Cursor;

    let (color_type, expected_size) = match format {
        KittyImageFormat::Rgb => (
            png::ColorType::Rgb,
            (width * height * 3) as usize,
        ),
        KittyImageFormat::Rgba => (
            png::ColorType::Rgba,
            (width * height * 4) as usize,
        ),
        KittyImageFormat::Png => {
            // Already PNG, return as-is
            return Some(data.to_vec());
        }
    };

    // Validate data size
    if data.len() != expected_size {
        warn!(
            "Invalid raw image data size: expected {}, got {}",
            expected_size,
            data.len()
        );
        return None;
    }

    // Encode to PNG
    let mut png_data = Vec::new();
    let mut encoder = png::Encoder::new(Cursor::new(&mut png_data), width, height);
    encoder.set_color(color_type);
    encoder.set_depth(png::BitDepth::Eight);

    let mut writer = match encoder.write_header() {
        Ok(w) => w,
        Err(e) => {
            warn!("Failed to create PNG encoder: {}", e);
            return None;
        }
    };

    if let Err(e) = writer.write_image_data(data) {
        warn!("Failed to write PNG data: {}", e);
        return None;
    }

    drop(writer); // Finalize PNG

    debug!(
        "Converted raw {:?} ({}x{}) to PNG ({} bytes)",
        format,
        width,
        height,
        png_data.len()
    );

    Some(png_data)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_transmit() {
        let sequence = b"a=t,f=100,t=d;iVBORw0KGgoAAAANSUhEUg==";
        let cmd = parse_kitty_graphics(sequence).unwrap();

        assert_eq!(cmd.action, KittyAction::Transmit);
        assert_eq!(cmd.format, KittyImageFormat::Png);
        assert_eq!(cmd.medium, TransmissionMedium::Direct);
        assert!(!cmd.more_chunks);
        assert!(!cmd.payload.is_empty());
    }

    #[test]
    fn test_parse_transmit_and_display() {
        let sequence = b"a=T,f=100;iVBORw0KGgoAAAANSUhEUg==";
        let cmd = parse_kitty_graphics(sequence).unwrap();

        assert_eq!(cmd.action, KittyAction::TransmitAndDisplay);
        assert_eq!(cmd.format, KittyImageFormat::Png);
    }

    #[test]
    fn test_parse_with_image_id() {
        let sequence = b"a=t,f=100,i=42;iVBORw0KGgoAAAANSUhEUg==";
        let cmd = parse_kitty_graphics(sequence).unwrap();

        assert_eq!(cmd.image_id, Some(42));
    }

    #[test]
    fn test_parse_with_placement_id() {
        let sequence = b"a=p,i=10,p=5";
        let cmd = parse_kitty_graphics(sequence).unwrap();

        assert_eq!(cmd.action, KittyAction::Put);
        assert_eq!(cmd.image_id, Some(10));
        assert_eq!(cmd.placement_id, Some(5));
    }

    #[test]
    fn test_parse_with_dimensions() {
        let sequence = b"a=T,f=24,s=800,v=600,c=40,r=20;";
        let cmd = parse_kitty_graphics(sequence).unwrap();

        assert_eq!(cmd.format, KittyImageFormat::Rgb);
        assert_eq!(cmd.source_width, Some(800));
        assert_eq!(cmd.source_height, Some(600));
        assert_eq!(cmd.display_columns, Some(40));
        assert_eq!(cmd.display_rows, Some(20));
    }

    #[test]
    fn test_parse_with_offsets() {
        let sequence = b"a=T,f=100,x=10,y=20,X=5,Y=3;";
        let cmd = parse_kitty_graphics(sequence).unwrap();

        assert_eq!(cmd.offset_x, Some(10));
        assert_eq!(cmd.offset_y, Some(20));
        assert_eq!(cmd.grid_x, Some(5));
        assert_eq!(cmd.grid_y, Some(3));
    }

    #[test]
    fn test_parse_with_z_index() {
        let sequence = b"a=T,f=100,z=-5;";
        let cmd = parse_kitty_graphics(sequence).unwrap();

        assert_eq!(cmd.z_index, -5);
    }

    #[test]
    fn test_parse_chunked_more() {
        let sequence = b"a=t,f=100,i=1,m=1;aGVsbG8=";
        let cmd = parse_kitty_graphics(sequence).unwrap();

        assert!(cmd.more_chunks);
        assert_eq!(cmd.image_id, Some(1));
        assert_eq!(cmd.payload, b"hello");
    }

    #[test]
    fn test_parse_chunked_final() {
        let sequence = b"a=t,f=100,i=1,m=0;d29ybGQ=";
        let cmd = parse_kitty_graphics(sequence).unwrap();

        assert!(!cmd.more_chunks);
        assert_eq!(cmd.image_id, Some(1));
        assert_eq!(cmd.payload, b"world");
    }

    #[test]
    fn test_parse_rgba_format() {
        let sequence = b"a=T,f=32,s=100,v=100;";
        let cmd = parse_kitty_graphics(sequence).unwrap();

        assert_eq!(cmd.format, KittyImageFormat::Rgba);
    }

    #[test]
    fn test_parse_delete_action() {
        let sequence = b"a=d,i=42";
        let cmd = parse_kitty_graphics(sequence).unwrap();

        assert_eq!(cmd.action, KittyAction::Delete);
        assert_eq!(cmd.image_id, Some(42));
    }

    #[test]
    fn test_parse_empty_sequence() {
        let result = parse_kitty_graphics(b"");
        assert!(result.is_some()); // Should return default command
    }

    #[test]
    fn test_parse_no_payload() {
        let sequence = b"a=t,f=100,i=10";
        let cmd = parse_kitty_graphics(sequence).unwrap();

        assert_eq!(cmd.action, KittyAction::Transmit);
        assert_eq!(cmd.image_id, Some(10));
        assert!(cmd.payload.is_empty());
    }

    #[test]
    fn test_parse_invalid_base64() {
        let sequence = b"a=t,f=100;!!!invalid!!!";
        let result = parse_kitty_graphics(sequence);
        assert!(result.is_none());
    }

    #[test]
    fn test_chunked_transfer_single_chunk() {
        let mut state = ChunkedTransferState::new();
        let result = state.add_chunk(1, b"hello world".to_vec(), true);

        assert_eq!(result, Some(b"hello world".to_vec()));
        assert_eq!(state.pending_count(), 0);
    }

    #[test]
    fn test_chunked_transfer_multiple_chunks() {
        let mut state = ChunkedTransferState::new();

        // First chunk
        let result1 = state.add_chunk(1, b"hello ".to_vec(), false);
        assert_eq!(result1, None);
        assert_eq!(state.pending_count(), 1);

        // Second chunk
        let result2 = state.add_chunk(1, b"world".to_vec(), false);
        assert_eq!(result2, None);
        assert_eq!(state.pending_count(), 1);

        // Final chunk
        let result3 = state.add_chunk(1, b"!".to_vec(), true);
        assert_eq!(result3, Some(b"hello world!".to_vec()));
        assert_eq!(state.pending_count(), 0);
    }

    #[test]
    fn test_chunked_transfer_multiple_images() {
        let mut state = ChunkedTransferState::new();

        // Start two transfers
        state.add_chunk(1, b"image1-".to_vec(), false);
        state.add_chunk(2, b"image2-".to_vec(), false);
        assert_eq!(state.pending_count(), 2);

        // Complete first transfer
        let result1 = state.add_chunk(1, b"done".to_vec(), true);
        assert_eq!(result1, Some(b"image1-done".to_vec()));
        assert_eq!(state.pending_count(), 1);

        // Complete second transfer
        let result2 = state.add_chunk(2, b"finished".to_vec(), true);
        assert_eq!(result2, Some(b"image2-finished".to_vec()));
        assert_eq!(state.pending_count(), 0);
    }

    #[test]
    fn test_chunked_transfer_clear() {
        let mut state = ChunkedTransferState::new();

        state.add_chunk(1, b"chunk1".to_vec(), false);
        state.add_chunk(2, b"chunk2".to_vec(), false);
        assert_eq!(state.pending_count(), 2);

        state.clear();
        assert_eq!(state.pending_count(), 0);
    }

    #[test]
    fn test_format_to_protocol() {
        assert_eq!(KittyImageFormat::Png.to_protocol_u8(), 0);
        assert_eq!(KittyImageFormat::Rgb.to_protocol_u8(), 3);
        assert_eq!(KittyImageFormat::Rgba.to_protocol_u8(), 3);
    }

    #[test]
    fn test_parse_multiple_key_value_in_one_segment() {
        // Kitty protocol allows multiple key=value pairs separated by commas
        let sequence = b"a=T,f=100,i=5,s=100,v=100;dGVzdA==";
        let cmd = parse_kitty_graphics(sequence).unwrap();

        assert_eq!(cmd.action, KittyAction::TransmitAndDisplay);
        assert_eq!(cmd.format, KittyImageFormat::Png);
        assert_eq!(cmd.image_id, Some(5));
        assert_eq!(cmd.source_width, Some(100));
        assert_eq!(cmd.source_height, Some(100));
        assert_eq!(cmd.payload, b"test");
    }

    #[test]
    fn test_parse_complex_sequence() {
        let sequence = b"a=T,f=32,t=d,i=100,p=50,s=1920,v=1080,c=80,r=40,x=100,y=200,X=10,Y=5,z=1;aGVsbG8=";
        let cmd = parse_kitty_graphics(sequence).unwrap();

        assert_eq!(cmd.action, KittyAction::TransmitAndDisplay);
        assert_eq!(cmd.format, KittyImageFormat::Rgba);
        assert_eq!(cmd.medium, TransmissionMedium::Direct);
        assert_eq!(cmd.image_id, Some(100));
        assert_eq!(cmd.placement_id, Some(50));
        assert_eq!(cmd.source_width, Some(1920));
        assert_eq!(cmd.source_height, Some(1080));
        assert_eq!(cmd.display_columns, Some(80));
        assert_eq!(cmd.display_rows, Some(40));
        assert_eq!(cmd.offset_x, Some(100));
        assert_eq!(cmd.offset_y, Some(200));
        assert_eq!(cmd.grid_x, Some(10));
        assert_eq!(cmd.grid_y, Some(5));
        assert_eq!(cmd.z_index, 1);
        assert_eq!(cmd.payload, b"hello");
    }
}
