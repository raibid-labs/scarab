use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// Accessibility configuration settings
///
/// Controls various accessibility features including screen reader integration,
/// high contrast mode, and export preferences.
#[derive(Debug, Clone, Serialize, Deserialize, Resource)]
pub struct AccessibilityConfig {
    /// Global accessibility features enabled
    pub enabled: bool,

    /// High contrast mode for improved visibility
    pub high_contrast: bool,

    /// Announce terminal output to screen readers (future AT-SPI integration)
    pub announce_output: bool,

    /// Announce cursor movement to screen readers
    pub announce_cursor_movement: bool,

    /// Default export format for accessibility exports
    pub default_export_format: ExportFormat,

    /// Include ANSI color codes in exports (when supported)
    pub preserve_colors_in_export: bool,

    /// Font scaling multiplier (1.0 = normal, 1.5 = 150%, etc.)
    pub text_scale: f32,

    /// Minimum text size in pixels (for enforcing readability)
    pub min_text_size: f32,

    /// Enable keyboard-only navigation enhancements
    pub keyboard_navigation_enhanced: bool,
}

impl Default for AccessibilityConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            high_contrast: false,
            announce_output: false,
            announce_cursor_movement: false,
            default_export_format: ExportFormat::PlainText,
            preserve_colors_in_export: true,
            text_scale: 1.0,
            min_text_size: 12.0,
            keyboard_navigation_enhanced: false,
        }
    }
}

/// Export format options for accessibility
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExportFormat {
    /// Plain text with ANSI stripped
    PlainText,
    /// HTML with CSS color preservation
    Html,
    /// Markdown code block format
    Markdown,
}

impl ExportFormat {
    /// Get file extension for this format
    pub fn extension(&self) -> &'static str {
        match self {
            Self::PlainText => "txt",
            Self::Html => "html",
            Self::Markdown => "md",
        }
    }

    /// Get MIME type for this format
    pub fn mime_type(&self) -> &'static str {
        match self {
            Self::PlainText => "text/plain",
            Self::Html => "text/html",
            Self::Markdown => "text/markdown",
        }
    }

    /// Parse format from string
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "text" | "txt" | "plain" => Some(Self::PlainText),
            "html" | "htm" => Some(Self::Html),
            "markdown" | "md" => Some(Self::Markdown),
            _ => None,
        }
    }
}

impl AccessibilityConfig {
    /// Load configuration from file (future implementation)
    pub fn load() -> Result<Self, std::io::Error> {
        // TODO: Load from config file in ~/.config/scarab/accessibility.toml
        Ok(Self::default())
    }

    /// Save configuration to file (future implementation)
    pub fn save(&self) -> Result<(), std::io::Error> {
        // TODO: Save to config file in ~/.config/scarab/accessibility.toml
        Ok(())
    }

    /// Toggle high contrast mode
    pub fn toggle_high_contrast(&mut self) {
        self.high_contrast = !self.high_contrast;
    }

    /// Set text scale with bounds checking
    pub fn set_text_scale(&mut self, scale: f32) {
        // Clamp between 0.5x and 3.0x
        self.text_scale = scale.clamp(0.5, 3.0);
    }

    /// Increase text scale by step
    pub fn increase_text_scale(&mut self, step: f32) {
        self.set_text_scale(self.text_scale + step);
    }

    /// Decrease text scale by step
    pub fn decrease_text_scale(&mut self, step: f32) {
        self.set_text_scale(self.text_scale - step);
    }

    /// Reset text scale to default
    pub fn reset_text_scale(&mut self) {
        self.text_scale = 1.0;
    }
}

/// Bevy events for accessibility changes
#[derive(Event, Debug, Clone)]
pub enum AccessibilityEvent {
    /// High contrast mode toggled
    HighContrastToggled { enabled: bool },

    /// Text scale changed
    TextScaleChanged { scale: f32 },

    /// Export completed
    ExportCompleted {
        format: ExportFormat,
        path: String,
    },

    /// Export failed
    ExportFailed {
        format: ExportFormat,
        error: String,
    },

    /// Screen reader announcement requested
    ScreenReaderAnnounce { message: String },
}
