//! Image Protocol Support
//!
//! Implements iTerm2, Kitty, and Sixel image protocols.

mod format;
mod iterm2;
mod kitty;
mod placement;
mod sixel;

pub use format::{detect_image, ImageFormat, ImageMetadata};
pub use iterm2::{parse_iterm2_image, ImageData, ImageSize};
pub use kitty::{
    convert_raw_to_png, parse_kitty_graphics, ChunkedTransferState, KittyAction, KittyCommand,
    KittyImageFormat, TransmissionMedium,
};
pub use placement::{ImagePlacement, ImagePlacementState};
pub use sixel::{parse_sixel_dcs, SixelData};
