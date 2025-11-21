// Text rendering module for Scarab terminal emulator
// Integrates cosmic-text with Bevy for GPU-accelerated text rendering

pub mod atlas;
pub mod text;
pub mod config;

pub use atlas::{GlyphAtlas, GlyphKey, AtlasRect};
pub use text::{TextRenderer, TerminalMesh, DirtyRegion};
pub use config::FontConfig;
