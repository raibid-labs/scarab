//! Integration Tests for Z-Order Rendering Layers
//!
//! This module verifies that all rendering systems use the correct z-order
//! constants and that the visual stacking order is correct.
//!
//! Issue #41: Rendering - Verify z-order layering for hints with images/shaders

#[cfg(test)]
mod tests {
    use crate::rendering::layers::*;
    use crate::rendering::{HintOverlay, HintOverlayConfig};

    /// Test that all layer constants are correctly ordered
    #[test]
    fn test_complete_layer_ordering() {
        // Verify the complete stack from bottom to top
        assert!(
            LAYER_TERMINAL_BG < LAYER_TERMINAL_TEXT,
            "Terminal text must be above background"
        );
        assert!(
            LAYER_TERMINAL_TEXT < LAYER_TEXT_DECORATIONS,
            "Text decorations must be above text"
        );
        assert!(
            LAYER_TEXT_DECORATIONS < LAYER_IMAGES,
            "Images must be above text decorations"
        );
        assert!(
            LAYER_IMAGES < LAYER_HINTS,
            "Hints must be above images (Issue #41 requirement)"
        );
        assert!(
            LAYER_HINTS < LAYER_FOCUS,
            "Focus indicators must be above hints"
        );
        assert!(
            LAYER_FOCUS < LAYER_MODALS,
            "Modals must be above focus indicators (Issue #41 requirement)"
        );
        assert!(
            LAYER_MODALS < LAYER_NOTIFICATIONS,
            "Notifications must be above modals"
        );
    }

    /// Test Issue #41 specific requirement: hints above images
    #[test]
    fn test_issue_41_hints_above_images() {
        assert!(
            LAYER_HINTS > LAYER_IMAGES,
            "Issue #41: Hints must render above images"
        );

        // Verify sufficient spacing for visual clarity
        let spacing = LAYER_HINTS - LAYER_IMAGES;
        assert!(
            spacing >= 100.0,
            "Hints and images should have significant z-spacing, got: {}",
            spacing
        );
    }

    /// Test Issue #41 specific requirement: modals above hints
    #[test]
    fn test_issue_41_modals_above_hints() {
        assert!(
            LAYER_MODALS > LAYER_HINTS,
            "Issue #41: Modals must render above hints"
        );

        // Verify sufficient spacing
        let spacing = LAYER_MODALS - LAYER_HINTS;
        assert!(
            spacing >= 50.0,
            "Modals and hints should have significant z-spacing, got: {}",
            spacing
        );
    }

    /// Test that HintOverlay uses the correct layer constant
    #[test]
    fn test_hint_overlay_uses_layer_constant() {
        let overlay = HintOverlay::default();
        assert_eq!(
            overlay.z_layer, LAYER_HINTS,
            "HintOverlay must use LAYER_HINTS constant"
        );
    }

    /// Test that HintOverlayConfig uses the correct layer constant
    #[test]
    fn test_hint_overlay_config_uses_layer_constant() {
        let config = HintOverlayConfig::default();
        assert_eq!(
            config.z_layer, LAYER_HINTS,
            "HintOverlayConfig must use LAYER_HINTS constant"
        );
    }

    /// Test terminal layer spacing for blur shader exclusion
    ///
    /// When implementing blur effects, we need to distinguish between
    /// terminal content (which can be blurred) and overlays (which should not be).
    #[test]
    fn test_blur_shader_layer_boundaries() {
        // Terminal content layers (can be blurred for focus effects)
        let terminal_max = LAYER_TEXT_DECORATIONS;

        // Image layer (can optionally be blurred)
        let images = LAYER_IMAGES;

        // Interactive layers (should NEVER be blurred)
        let interactive_min = LAYER_HINTS;

        // Verify clear separation
        assert!(
            terminal_max < images,
            "Terminal layers must be below images"
        );
        assert!(
            images < interactive_min,
            "Images must be below interactive layers"
        );

        // Hints, focus, modals, and notifications should be excluded from blur
        assert!(
            LAYER_HINTS >= interactive_min,
            "Hints should not be blurred"
        );
        assert!(
            LAYER_FOCUS >= interactive_min,
            "Focus should not be blurred"
        );
        assert!(
            LAYER_MODALS >= interactive_min,
            "Modals should not be blurred"
        );
        assert!(
            LAYER_NOTIFICATIONS >= interactive_min,
            "Notifications should not be blurred"
        );
    }

    /// Test that layer values are reasonable for f32 precision
    #[test]
    fn test_layer_values_within_f32_precision() {
        // All layers should be well within f32 precision limits
        // and have enough difference to avoid floating point equality issues
        let all_layers = [
            LAYER_TERMINAL_BG,
            LAYER_TERMINAL_TEXT,
            LAYER_TEXT_DECORATIONS,
            LAYER_IMAGES,
            LAYER_HINTS,
            LAYER_FOCUS,
            LAYER_MODALS,
            LAYER_NOTIFICATIONS,
        ];

        for &layer in &all_layers {
            // Should be positive and reasonable
            assert!(layer >= 0.0, "Layer should be non-negative: {}", layer);
            assert!(layer < 1000.0, "Layer should be < 1000: {}", layer);
        }

        // Check that each layer is distinguishable (no duplicates)
        for (i, &layer1) in all_layers.iter().enumerate() {
            for (j, &layer2) in all_layers.iter().enumerate() {
                if i != j {
                    assert_ne!(
                        layer1, layer2,
                        "Layers {} and {} have same value: {}",
                        i, j, layer1
                    );
                }
            }
        }
    }

    /// Test that terminal text layers have minimal z-spacing
    ///
    /// Background, text, and decorations should be close together
    /// to minimize z-fighting while maintaining correct order.
    #[test]
    fn test_terminal_text_layer_spacing() {
        let bg_to_text = LAYER_TERMINAL_TEXT - LAYER_TERMINAL_BG;
        let text_to_decorations = LAYER_TEXT_DECORATIONS - LAYER_TERMINAL_TEXT;

        // Should be small but distinguishable
        assert!(
            bg_to_text > 0.0 && bg_to_text <= 0.2,
            "Background to text spacing should be minimal: {}",
            bg_to_text
        );
        assert!(
            text_to_decorations > 0.0 && text_to_decorations <= 0.2,
            "Text to decorations spacing should be minimal: {}",
            text_to_decorations
        );
    }

    /// Test that interactive layers have sufficient spacing
    ///
    /// Hints, focus, modals, and notifications should be well-separated
    /// to allow for future sub-layers and visual effects.
    #[test]
    fn test_interactive_layer_spacing() {
        let hints_to_focus = LAYER_FOCUS - LAYER_HINTS;
        let focus_to_modals = LAYER_MODALS - LAYER_FOCUS;
        let modals_to_notifications = LAYER_NOTIFICATIONS - LAYER_MODALS;

        assert!(
            hints_to_focus >= 5.0,
            "Hints to focus should have spacing >= 5.0: {}",
            hints_to_focus
        );
        assert!(
            focus_to_modals >= 10.0,
            "Focus to modals should have spacing >= 10.0: {}",
            focus_to_modals
        );
        assert!(
            modals_to_notifications >= 10.0,
            "Modals to notifications should have spacing >= 10.0: {}",
            modals_to_notifications
        );
    }

    /// Test layer constant documentation accuracy
    ///
    /// Ensure the documented z-values match the actual constants.
    #[test]
    fn test_layer_documentation_accuracy() {
        // These values should match the documentation in layers.rs
        assert_eq!(LAYER_TERMINAL_BG, 0.0, "Documentation mismatch");
        assert_eq!(LAYER_TERMINAL_TEXT, 0.1, "Documentation mismatch");
        assert_eq!(LAYER_TEXT_DECORATIONS, 0.15, "Documentation mismatch");
        assert_eq!(LAYER_IMAGES, 50.0, "Documentation mismatch");
        assert_eq!(LAYER_HINTS, 200.0, "Documentation mismatch");
        assert_eq!(LAYER_FOCUS, 210.0, "Documentation mismatch");
        assert_eq!(LAYER_MODALS, 300.0, "Documentation mismatch");
        assert_eq!(LAYER_NOTIFICATIONS, 400.0, "Documentation mismatch");
    }

    /// Test z-order for a simulated rendering scenario
    ///
    /// This test simulates what would happen when multiple elements are rendered
    /// at the same screen position to verify visual stacking.
    #[test]
    fn test_simulated_rendering_stack() {
        // Simulate rendering elements at the same position
        let mut render_stack = vec![
            ("terminal_bg", LAYER_TERMINAL_BG),
            ("terminal_text", LAYER_TERMINAL_TEXT),
            ("text_decoration", LAYER_TEXT_DECORATIONS),
            ("image", LAYER_IMAGES),
            ("hint", LAYER_HINTS),
            ("focus", LAYER_FOCUS),
            ("modal", LAYER_MODALS),
            ("notification", LAYER_NOTIFICATIONS),
        ];

        // Sort by z-order (what the GPU would see)
        render_stack.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());

        // Verify rendering order from back to front
        let expected_order = vec![
            "terminal_bg",
            "terminal_text",
            "text_decoration",
            "image",
            "hint",
            "focus",
            "modal",
            "notification",
        ];

        let actual_order: Vec<&str> = render_stack.iter().map(|(name, _)| *name).collect();

        assert_eq!(
            actual_order, expected_order,
            "Rendering order should match expected stack"
        );
    }
}
