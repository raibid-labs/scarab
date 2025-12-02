//! Image Protocol Support
//!
//! Implements iTerm2, and eventually Kitty and Sixel image protocols.

mod iterm2;
mod placement;

pub use iterm2::{parse_iterm2_image, ImageData, ImageSize};
pub use placement::{ImagePlacement, ImagePlacementState};
