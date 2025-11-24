// Text rendering module for Scarab terminal emulator
// Integrates cosmic-text with Bevy for GPU-accelerated text rendering

pub mod atlas;
pub mod config;
pub mod text;

pub use atlas::{AtlasRect, GlyphAtlas, GlyphKey};
pub use config::FontConfig;
pub use text::{DirtyRegion, TerminalMesh, TextRenderer};
