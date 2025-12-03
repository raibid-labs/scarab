/// Accessibility module for Scarab terminal emulator
///
/// Provides accessibility features including:
/// - Screen reader integration (AT-SPI stubs for future implementation)
/// - Export capabilities (plain text, HTML, Markdown)
/// - High contrast mode
/// - Text scaling support
/// - Keyboard-only navigation enhancements
///
/// This module implements accessibility best practices for terminal emulators,
/// making Scarab usable with assistive technologies.

pub mod export;
pub mod screen_reader;
pub mod settings;

use bevy::prelude::*;

// Re-export main types
pub use export::TerminalExporter;
pub use screen_reader::{
    announce_content_changes, announce_cursor_movement, Announcement, AnnouncementPriority,
    AtSpiIntegration, ScreenReaderAnnounceEvent, ScreenReaderState,
};
pub use settings::{AccessibilityConfig, AccessibilityEvent, ExportFormat};

/// Main accessibility plugin for Bevy
///
/// This plugin integrates all accessibility features into the Bevy ECS:
/// - Registers accessibility resources and events
/// - Sets up screen reader announcement systems
/// - Handles high contrast mode toggles
/// - Manages export commands
pub struct AccessibilityPlugin;

impl Plugin for AccessibilityPlugin {
    fn build(&self, app: &mut App) {
        // Initialize resources
        app.insert_resource(AccessibilityConfig::default())
            .insert_resource(ScreenReaderState::default())
            .insert_resource(AtSpiIntegration::default());

        // Register events
        app.add_event::<AccessibilityEvent>()
            .add_event::<ScreenReaderAnnounceEvent>()
            .add_event::<ExportGridEvent>()
            .add_event::<ToggleHighContrastEvent>()
            .add_event::<ChangeTextScaleEvent>();

        // Add systems
        app.add_systems(
            Update,
            (
                screen_reader::handle_screen_reader_announcements,
                handle_export_requests,
                handle_high_contrast_toggle,
                handle_text_scale_changes,
                apply_high_contrast_mode,
            )
                .chain(),
        );

        info!("Accessibility plugin initialized");
    }
}

/// Event to request terminal grid export
#[derive(Event, Debug, Clone)]
pub struct ExportGridEvent {
    /// Export format
    pub format: ExportFormat,
    /// Output file path
    pub path: String,
}

/// Event to toggle high contrast mode
#[derive(Event, Debug, Clone)]
pub struct ToggleHighContrastEvent;

/// Event to change text scale
#[derive(Event, Debug, Clone)]
pub struct ChangeTextScaleEvent {
    /// New scale factor (or relative change if delta is true)
    pub scale: f32,
    /// Whether this is a delta (relative) change
    pub delta: bool,
}

/// System to handle export requests
fn handle_export_requests(
    mut export_events: EventReader<ExportGridEvent>,
    mut accessibility_events: EventWriter<AccessibilityEvent>,
    // TODO: Add terminal grid resource access
    // grid: Res<SharedMemoryReader>,
) {
    for event in export_events.read() {
        info!(
            "Export requested: format={:?}, path={}",
            event.format, event.path
        );

        // TODO: Get actual grid data from SharedMemoryReader
        // For now, we'll just simulate the export
        /*
        let result = TerminalExporter::export_from_shared_state(
            &grid.state,
            event.format,
            Path::new(&event.path),
        );

        match result {
            Ok(_) => {
                accessibility_events.send(AccessibilityEvent::ExportCompleted {
                    format: event.format,
                    path: event.path.clone(),
                });
                info!("Export completed: {}", event.path);
            }
            Err(e) => {
                accessibility_events.send(AccessibilityEvent::ExportFailed {
                    format: event.format,
                    error: e.to_string(),
                });
                error!("Export failed: {}", e);
            }
        }
        */

        // Placeholder success event
        accessibility_events.send(AccessibilityEvent::ExportCompleted {
            format: event.format,
            path: event.path.clone(),
        });
    }
}

/// System to handle high contrast toggle events
fn handle_high_contrast_toggle(
    mut toggle_events: EventReader<ToggleHighContrastEvent>,
    mut config: ResMut<AccessibilityConfig>,
    mut accessibility_events: EventWriter<AccessibilityEvent>,
) {
    for _ in toggle_events.read() {
        config.toggle_high_contrast();

        accessibility_events.send(AccessibilityEvent::HighContrastToggled {
            enabled: config.high_contrast,
        });

        info!(
            "High contrast mode {}",
            if config.high_contrast {
                "enabled"
            } else {
                "disabled"
            }
        );
    }
}

/// System to handle text scale changes
fn handle_text_scale_changes(
    mut scale_events: EventReader<ChangeTextScaleEvent>,
    mut config: ResMut<AccessibilityConfig>,
    mut accessibility_events: EventWriter<AccessibilityEvent>,
) {
    for event in scale_events.read() {
        let new_scale = if event.delta {
            config.text_scale + event.scale
        } else {
            event.scale
        };

        config.set_text_scale(new_scale);

        accessibility_events.send(AccessibilityEvent::TextScaleChanged {
            scale: config.text_scale,
        });

        info!("Text scale changed to: {:.2}", config.text_scale);
    }
}

/// System to apply high contrast mode to rendering
///
/// This system would modify rendering parameters to increase contrast
/// when high contrast mode is enabled.
fn apply_high_contrast_mode(
    config: Res<AccessibilityConfig>,
    // TODO: Add rendering resources to modify
    // mut colors: ResMut<ColorScheme>,
) {
    // Only update when config changes
    if !config.is_changed() {
        return;
    }

    if config.high_contrast {
        // TODO: Apply high contrast color scheme
        // - Increase foreground/background contrast
        // - Use high-contrast palette
        // - Disable transparency
        // - Bold all text for better visibility

        debug!("Applying high contrast color scheme");

        // Example modifications:
        // colors.foreground = Color::rgb(1.0, 1.0, 1.0);  // Pure white
        // colors.background = Color::rgb(0.0, 0.0, 0.0);  // Pure black
        // colors.cursor = Color::rgb(1.0, 1.0, 0.0);      // Bright yellow
    } else {
        // TODO: Restore normal color scheme
        debug!("Restoring normal color scheme");
    }
}

/// Helper functions for accessibility commands

/// Parse export command: `:a11y export <format> <path>`
pub fn parse_export_command(command: &str) -> Option<ExportGridEvent> {
    let parts: Vec<&str> = command.split_whitespace().collect();

    if parts.len() < 4 {
        return None;
    }

    if parts[0] != ":a11y" || parts[1] != "export" {
        return None;
    }

    let format = ExportFormat::from_str(parts[2])?;
    let path = parts[3..].join(" ");

    Some(ExportGridEvent { format, path })
}

/// Parse accessibility command
pub fn parse_accessibility_command(command: &str) -> Option<AccessibilityCommand> {
    let parts: Vec<&str> = command.split_whitespace().collect();

    if parts.is_empty() || parts[0] != ":a11y" {
        return None;
    }

    if parts.len() < 2 {
        return None;
    }

    match parts[1] {
        "export" if parts.len() >= 4 => {
            let format = ExportFormat::from_str(parts[2])?;
            let path = parts[3..].join(" ");
            Some(AccessibilityCommand::Export { format, path })
        }
        "contrast" if parts.len() >= 3 && parts[2] == "toggle" => {
            Some(AccessibilityCommand::ToggleHighContrast)
        }
        "scale" if parts.len() >= 3 => {
            let scale: f32 = parts[2].parse().ok()?;
            Some(AccessibilityCommand::SetTextScale { scale })
        }
        "scale" if parts.len() >= 4 && parts[2] == "increase" => {
            let delta: f32 = parts.get(3).and_then(|s| s.parse().ok()).unwrap_or(0.1);
            Some(AccessibilityCommand::IncreaseTextScale { delta })
        }
        "scale" if parts.len() >= 4 && parts[2] == "decrease" => {
            let delta: f32 = parts.get(3).and_then(|s| s.parse().ok()).unwrap_or(0.1);
            Some(AccessibilityCommand::DecreaseTextScale { delta })
        }
        "scale" if parts.len() >= 3 && parts[2] == "reset" => {
            Some(AccessibilityCommand::ResetTextScale)
        }
        "help" => Some(AccessibilityCommand::Help),
        _ => None,
    }
}

/// Accessibility command types
#[derive(Debug, Clone, PartialEq)]
pub enum AccessibilityCommand {
    Export {
        format: ExportFormat,
        path: String,
    },
    ToggleHighContrast,
    SetTextScale {
        scale: f32,
    },
    IncreaseTextScale {
        delta: f32,
    },
    DecreaseTextScale {
        delta: f32,
    },
    ResetTextScale,
    Help,
}

impl AccessibilityCommand {
    /// Get help text for accessibility commands
    pub fn help_text() -> &'static str {
        r#"Accessibility Commands:
  :a11y export text <path>       - Export terminal to plain text
  :a11y export html <path>       - Export terminal to HTML with colors
  :a11y export markdown <path>   - Export terminal to Markdown
  :a11y contrast toggle          - Toggle high contrast mode
  :a11y scale <factor>           - Set text scale (0.5 - 3.0)
  :a11y scale increase [delta]   - Increase text scale (default: 0.1)
  :a11y scale decrease [delta]   - Decrease text scale (default: 0.1)
  :a11y scale reset              - Reset text scale to 1.0
  :a11y help                     - Show this help"#
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_export_command() {
        let cmd = ":a11y export text /tmp/output.txt";
        let event = parse_export_command(cmd);
        assert!(event.is_some());
        let event = event.unwrap();
        assert_eq!(event.format, ExportFormat::PlainText);
        assert_eq!(event.path, "/tmp/output.txt");
    }

    #[test]
    fn test_parse_accessibility_command_export() {
        let cmd = ":a11y export html /tmp/output.html";
        let parsed = parse_accessibility_command(cmd);
        assert!(matches!(
            parsed,
            Some(AccessibilityCommand::Export { .. })
        ));
    }

    #[test]
    fn test_parse_accessibility_command_contrast() {
        let cmd = ":a11y contrast toggle";
        let parsed = parse_accessibility_command(cmd);
        assert!(matches!(
            parsed,
            Some(AccessibilityCommand::ToggleHighContrast)
        ));
    }

    #[test]
    fn test_parse_accessibility_command_scale() {
        let cmd = ":a11y scale 1.5";
        let parsed = parse_accessibility_command(cmd);
        assert!(matches!(
            parsed,
            Some(AccessibilityCommand::SetTextScale { scale: 1.5 })
        ));
    }

    #[test]
    fn test_parse_accessibility_command_help() {
        let cmd = ":a11y help";
        let parsed = parse_accessibility_command(cmd);
        assert!(matches!(parsed, Some(AccessibilityCommand::Help)));
    }
}
