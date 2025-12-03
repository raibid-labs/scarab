//! Kitty Graphics Protocol Demo
//!
//! Demonstrates how to use the Kitty graphics protocol parser.
//!
//! Run with: `cargo run --example kitty_graphics_demo`

use scarab_daemon::images::{
    parse_kitty_graphics, ChunkedTransferState, KittyAction, KittyImageFormat,
};

fn main() {
    env_logger::init();

    println!("Kitty Graphics Protocol Demo\n");

    // Example 1: Simple PNG transmission
    demo_simple_png();

    // Example 2: Chunked transfer
    demo_chunked_transfer();

    // Example 3: Display previously transmitted image
    demo_display_existing();

    // Example 4: RGB format with dimensions
    demo_rgb_format();
}

fn demo_simple_png() {
    println!("=== Demo 1: Simple PNG Transmission ===");

    // Minimal 1x1 red PNG (base64 encoded)
    let png_b64 = "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mP8z8DwHwAFBQIAX8jx0gAAAABJRU5ErkJggg==";
    let sequence = format!("a=T,f=100,i=42,c=10,r=5;{}", png_b64);

    if let Some(cmd) = parse_kitty_graphics(sequence.as_bytes()) {
        println!("  Action: {:?}", cmd.action);
        println!("  Format: {:?}", cmd.format);
        println!("  Image ID: {:?}", cmd.image_id);
        println!("  Display: {}x{} cells", cmd.display_columns.unwrap_or(0), cmd.display_rows.unwrap_or(0));
        println!("  Payload size: {} bytes", cmd.payload.len());
        assert_eq!(cmd.action, KittyAction::TransmitAndDisplay);
        assert_eq!(cmd.format, KittyImageFormat::Png);
        assert_eq!(cmd.image_id, Some(42));
    } else {
        println!("  Failed to parse!");
    }
    println!();
}

fn demo_chunked_transfer() {
    println!("=== Demo 2: Chunked Transfer ===");

    let mut state = ChunkedTransferState::new();

    // Send first chunk (m=1 means more chunks coming)
    let chunk1 = "a=t,f=100,i=1,m=1;aGVsbG8="; // "hello" in base64
    if let Some(cmd) = parse_kitty_graphics(chunk1.as_bytes()) {
        println!("  Chunk 1: {} bytes, more={}", cmd.payload.len(), cmd.more_chunks);
        let result = state.add_chunk(
            cmd.image_id.unwrap(),
            cmd.payload.clone(),
            !cmd.more_chunks,
        );
        assert!(result.is_none(), "First chunk should not complete transfer");
    }

    // Send second chunk (m=1 still)
    let chunk2 = "a=t,f=100,i=1,m=1;IHdvcmxk"; // " world" in base64
    if let Some(cmd) = parse_kitty_graphics(chunk2.as_bytes()) {
        println!("  Chunk 2: {} bytes, more={}", cmd.payload.len(), cmd.more_chunks);
        let result = state.add_chunk(
            cmd.image_id.unwrap(),
            cmd.payload.clone(),
            !cmd.more_chunks,
        );
        assert!(result.is_none(), "Second chunk should not complete transfer");
    }

    // Send final chunk (m=0 means final chunk)
    let chunk3 = "a=t,f=100,i=1,m=0;IQ=="; // "!" in base64
    if let Some(cmd) = parse_kitty_graphics(chunk3.as_bytes()) {
        println!("  Chunk 3: {} bytes, more={}", cmd.payload.len(), cmd.more_chunks);
        let result = state.add_chunk(
            cmd.image_id.unwrap(),
            cmd.payload.clone(),
            !cmd.more_chunks,
        );
        if let Some(complete_data) = result {
            println!("  Transfer complete! Total size: {} bytes", complete_data.len());
            println!("  Data: {:?}", String::from_utf8(complete_data).unwrap());
            assert_eq!(String::from_utf8_lossy(&complete_data), "hello world!");
        }
    }
    println!();
}

fn demo_display_existing() {
    println!("=== Demo 3: Display Existing Image ===");

    // Display previously transmitted image (a=p for "put")
    let sequence = "a=p,i=42,X=10,Y=5,c=20,r=10";

    if let Some(cmd) = parse_kitty_graphics(sequence.as_bytes()) {
        println!("  Action: {:?}", cmd.action);
        println!("  Image ID: {:?}", cmd.image_id);
        println!("  Grid position: ({}, {})", cmd.grid_x.unwrap_or(0), cmd.grid_y.unwrap_or(0));
        println!("  Display size: {}x{} cells", cmd.display_columns.unwrap_or(0), cmd.display_rows.unwrap_or(0));
        assert_eq!(cmd.action, KittyAction::Put);
        assert_eq!(cmd.image_id, Some(42));
    }
    println!();
}

fn demo_rgb_format() {
    println!("=== Demo 4: RGB Format ===");

    // 2x2 RGB image (12 bytes: 2*2*3)
    // Red, Green, Blue, Yellow pixels
    let rgb_data = base64::encode(&[
        255, 0, 0,    // Red
        0, 255, 0,    // Green
        0, 0, 255,    // Blue
        255, 255, 0,  // Yellow
    ]);

    let sequence = format!("a=T,f=24,s=2,v=2,c=4,r=4;{}", rgb_data);

    if let Some(cmd) = parse_kitty_graphics(sequence.as_bytes()) {
        println!("  Format: {:?} (f=24 means RGB)", cmd.format);
        println!("  Source dimensions: {}x{} pixels", cmd.source_width.unwrap_or(0), cmd.source_height.unwrap_or(0));
        println!("  Display size: {}x{} cells", cmd.display_columns.unwrap_or(0), cmd.display_rows.unwrap_or(0));
        println!("  Payload size: {} bytes (should be 12 for 2x2 RGB)", cmd.payload.len());
        assert_eq!(cmd.format, KittyImageFormat::Rgb);
        assert_eq!(cmd.source_width, Some(2));
        assert_eq!(cmd.source_height, Some(2));
        assert_eq!(cmd.payload.len(), 12);
    }
    println!();
}

// Helper to encode base64
mod base64 {
    use base64::{engine::general_purpose::STANDARD, Engine as _};

    pub fn encode(data: &[u8]) -> String {
        STANDARD.encode(data)
    }
}
