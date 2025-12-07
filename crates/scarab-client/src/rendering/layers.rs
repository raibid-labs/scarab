//! Z-Order Rendering Layers for Scarab Terminal
//!
//! This module defines constants for z-ordering of visual elements in the terminal.
//! Higher z-values render on top of lower z-values.
//!
//! ## Layering Architecture
//!
//! The rendering stack is organized into distinct layers to ensure proper visual hierarchy:
//!
//! ```text
//! Layer                    Z-Value    Description
//! ─────────────────────────────────────────────────────────────────────────────
//! LAYER_TERMINAL_BG        0.0        Terminal background (solid color)
//! LAYER_TERMINAL_TEXT      0.1        Terminal text glyphs and cell backgrounds
//! LAYER_TEXT_DECORATIONS   0.15       Underlines, strikethroughs (on text)
//! LAYER_IMAGES             50.0       Inline images (iTerm2/Kitty protocol)
//! LAYER_HINTS              200.0      Navigation hints (Vimium-style overlays)
//! LAYER_FOCUS              210.0      Focus indicators and visual feedback
//! LAYER_MODALS             300.0      Modal dialogs and command palette
//! LAYER_NOTIFICATIONS      400.0      Toast notifications (always on top)
//! ```
//!
//! ## Usage
//!
//! Import these constants when creating entities with z-positioning:
//!
//! ```rust,ignore
//! use crate::rendering::layers::*;
//!
//! Transform::from_xyz(x, y, LAYER_HINTS)
//! ```
//!
//! ## Shader and Blur Effects
//!
//! When implementing blur or other post-processing shaders:
//! - Hints (200.0+) should NOT be blurred (crisp, readable)
//! - Terminal content (0.0-0.15) can be blurred for focus effects
//! - Images (50.0) can be blurred if background dimming is desired
//! - Modals (300.0+) should never be blurred (always sharp)
//!
//! ## Implementation Notes
//!
//! - These are f32 constants suitable for Bevy's Transform z-component
//! - Spacing between layers allows for sub-layers if needed
//! - All rendering systems should use these constants, never hardcoded values
//! - Changes to these values require testing visual stacking order

/// Terminal background layer (solid color, lowest z-order)
///
/// This layer represents the terminal's base background color. Nothing should
/// render below this layer.
pub const LAYER_TERMINAL_BG: f32 = 0.0;

/// Terminal text layer (glyphs and cell backgrounds)
///
/// This layer contains the main terminal grid content:
/// - Cell background colors (z = 0.0)
/// - Text glyphs (z = 0.1)
///
/// Text is slightly above cell backgrounds to ensure proper visibility.
pub const LAYER_TERMINAL_TEXT: f32 = 0.1;

/// Text decorations layer (underlines, strikethroughs)
///
/// Visual decorations that appear on top of text glyphs:
/// - Underlines
/// - Strikethroughs
/// - Double underlines
/// - Dotted/dashed underlines
///
/// Must be above text to be visible, but below images.
pub const LAYER_TEXT_DECORATIONS: f32 = 0.15;

/// Inline images layer (iTerm2/Kitty image protocol)
///
/// Images embedded in the terminal via escape sequences. These render above
/// the terminal text but below interactive overlays.
///
/// Spacing from text (50.0 - 0.15 = 49.85) allows for future layers like:
/// - Image borders/frames
/// - Image selection indicators
/// - Image loading placeholders
pub const LAYER_IMAGES: f32 = 50.0;

/// Navigation hints layer (Vimium-style keyboard hints)
///
/// Interactive hint overlays for keyboard-driven navigation:
/// - Hint badges with labels (e.g., "AB", "CD")
/// - Hint backgrounds (colored rectangles)
/// - Hint text (white/black labels)
///
/// Must be above images to ensure hints are visible when overlaying images.
/// Must be below modals so command palette/dialogs can obscure hints.
///
/// Spacing from images (200.0 - 50.0 = 150.0) allows for future layers like:
/// - Link underlines
/// - Selection highlights
/// - Search result markers
pub const LAYER_HINTS: f32 = 200.0;

/// Focus indicators layer (visual feedback)
///
/// Visual feedback elements that indicate focus or active state:
/// - Cursor position indicators
/// - Active pane borders
/// - Focused element highlights
///
/// Slightly above hints to ensure focus feedback is always visible.
pub const LAYER_FOCUS: f32 = 210.0;

/// Modal dialogs layer (command palette, dialogs)
///
/// Full-screen or large overlay UI elements:
/// - Command palette
/// - Remote modals from daemon
/// - Confirmation dialogs
/// - Settings panels
///
/// Must be above all terminal content and hints to ensure full attention.
///
/// Spacing from focus (300.0 - 210.0 = 90.0) allows for:
/// - Modal backdrops/dimming
/// - Modal shadows
/// - Modal animation layers
pub const LAYER_MODALS: f32 = 300.0;

/// Notification toasts layer (always on top)
///
/// Temporary notification messages that appear in corners:
/// - Plugin notifications
/// - System messages
/// - Error alerts
/// - Success confirmations
///
/// These are always on top to ensure critical messages are never obscured.
pub const LAYER_NOTIFICATIONS: f32 = 400.0;

/// Helper function to validate z-ordering at compile time
///
/// This function exists to ensure the layer constants maintain their ordering
/// relationship. It's used in tests to verify the layer hierarchy.
#[cfg(test)]
pub const fn validate_layer_ordering() -> bool {
    LAYER_TERMINAL_BG < LAYER_TERMINAL_TEXT
        && LAYER_TERMINAL_TEXT < LAYER_TEXT_DECORATIONS
        && LAYER_TEXT_DECORATIONS < LAYER_IMAGES
        && LAYER_IMAGES < LAYER_HINTS
        && LAYER_HINTS < LAYER_FOCUS
        && LAYER_FOCUS < LAYER_MODALS
        && LAYER_MODALS < LAYER_NOTIFICATIONS
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_layer_ordering() {
        assert!(LAYER_TERMINAL_BG < LAYER_TERMINAL_TEXT);
        assert!(LAYER_TERMINAL_TEXT < LAYER_TEXT_DECORATIONS);
        assert!(LAYER_TEXT_DECORATIONS < LAYER_IMAGES);
        assert!(LAYER_IMAGES < LAYER_HINTS);
        assert!(LAYER_HINTS < LAYER_FOCUS);
        assert!(LAYER_FOCUS < LAYER_MODALS);
        assert!(LAYER_MODALS < LAYER_NOTIFICATIONS);
    }

    #[test]
    fn test_compile_time_validation() {
        const VALID: bool = validate_layer_ordering();
        assert!(VALID);
    }

    #[test]
    fn test_layer_absolute_values() {
        // Verify expected absolute values
        assert_eq!(LAYER_TERMINAL_BG, 0.0);
        assert_eq!(LAYER_TERMINAL_TEXT, 0.1);
        assert_eq!(LAYER_TEXT_DECORATIONS, 0.15);
        assert_eq!(LAYER_IMAGES, 50.0);
        assert_eq!(LAYER_HINTS, 200.0);
        assert_eq!(LAYER_FOCUS, 210.0);
        assert_eq!(LAYER_MODALS, 300.0);
        assert_eq!(LAYER_NOTIFICATIONS, 400.0);
    }

    #[test]
    fn test_layer_spacing_sufficient() {
        // Ensure there's enough spacing for sub-layers
        assert!(
            (LAYER_IMAGES - LAYER_TEXT_DECORATIONS) > 10.0,
            "Insufficient spacing between text and images"
        );
        assert!(
            (LAYER_HINTS - LAYER_IMAGES) > 10.0,
            "Insufficient spacing between images and hints"
        );
        assert!(
            (LAYER_FOCUS - LAYER_HINTS) > 5.0,
            "Insufficient spacing between hints and focus"
        );
        assert!(
            (LAYER_MODALS - LAYER_FOCUS) > 10.0,
            "Insufficient spacing between focus and modals"
        );
        assert!(
            (LAYER_NOTIFICATIONS - LAYER_MODALS) > 10.0,
            "Insufficient spacing between modals and notifications"
        );
    }

    #[test]
    fn test_hints_above_images() {
        // Critical requirement from Issue #41
        assert!(LAYER_HINTS > LAYER_IMAGES, "Hints must render above images");
    }

    #[test]
    fn test_modals_above_hints() {
        // Critical requirement from Issue #41
        assert!(LAYER_MODALS > LAYER_HINTS, "Modals must render above hints");
    }
}
