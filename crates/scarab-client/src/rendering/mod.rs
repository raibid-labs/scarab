// Rendering subsystem for Scarab terminal
// GPU-accelerated text rendering with cosmic-text and glyph atlas caching

pub mod atlas;
pub mod config;
pub mod hint_overlay;
pub mod images;
pub mod layers;
pub mod scrollback_render;
pub mod text;

#[cfg(test)]
mod z_order_tests;

pub use atlas::{AtlasRect, GlyphAtlas, GlyphKey};
pub use config::{color, FontConfig, TextAttributes};
pub use hint_overlay::{
    HintFade, HintOverlay, HintOverlayBundle, HintOverlayConfig, HintOverlayPlugin,
};
pub use images::{ImageCache, ImagePlacementComponent, ImagesPlugin, SharedImageReader};
pub use layers::*;
pub use scrollback_render::generate_scrollback_mesh;
pub use text::{
    generate_terminal_mesh, update_terminal_mesh_system, DirtyRegion, TerminalMesh, TextRenderer,
};

// Re-export shader effects from parent shaders module
pub use crate::shaders::{BlurSettings, GlowSettings, ScarabEffectsPlugin};
