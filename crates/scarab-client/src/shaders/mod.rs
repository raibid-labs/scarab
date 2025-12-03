//! Post-process shader effects for Scarab terminal
//!
//! This module provides GPU-accelerated post-processing effects including:
//! - Gaussian blur for overlay backgrounds
//! - Border glow for focused elements
//!
//! Effects are configurable and can be disabled for low-power mode.

pub mod blur;
pub mod glow;
pub mod plugin;

pub use blur::{BlurSettings, BlurShaderNode};
pub use glow::{GlowSettings, GlowShaderNode};
pub use plugin::ScarabEffectsPlugin;
