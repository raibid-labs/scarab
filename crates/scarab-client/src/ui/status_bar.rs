//! Status Bar rendering system for Scarab terminal emulator
//!
//! Provides Bevy resources, components, and systems for rendering
//! programmable status bars with rich styling and dynamic content.

use bevy::prelude::*;
use scarab_plugin_api::status_bar::{AnsiColor, RenderItem};
use scarab_plugin_api::status_bar::Color as StatusColor;

/// Plugin for status bar functionality
pub struct StatusBarPlugin;

impl Plugin for StatusBarPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<StatusBarState>()
            .add_systems(Startup, setup_status_bar)
            .add_systems(Update, update_status_bar_system);
    }
}

/// Resource holding current status bar state
///
/// Tracks the render items for left and right sections of the status bar
/// along with dirty flags to optimize rendering.
#[derive(Resource, Default)]
pub struct StatusBarState {
    /// Items to display on the left side
    pub left_items: Vec<RenderItem>,
    /// Items to display on the right side
    pub right_items: Vec<RenderItem>,
    /// Whether left side needs re-rendering
    pub left_dirty: bool,
    /// Whether right side needs re-rendering
    pub right_dirty: bool,
}

impl StatusBarState {
    /// Update the left side of the status bar
    pub fn set_left(&mut self, items: Vec<RenderItem>) {
        self.left_items = items;
        self.left_dirty = true;
    }

    /// Update the right side of the status bar
    pub fn set_right(&mut self, items: Vec<RenderItem>) {
        self.right_items = items;
        self.right_dirty = true;
    }

    /// Clear all status bar content
    pub fn clear(&mut self) {
        self.left_items.clear();
        self.right_items.clear();
        self.left_dirty = true;
        self.right_dirty = true;
    }

    /// Clear dirty flags (called after rendering)
    pub fn clear_dirty(&mut self) {
        self.left_dirty = false;
        self.right_dirty = false;
    }
}

/// Marker component for the left section of the status bar
#[derive(Component)]
pub struct StatusBarLeft;

/// Marker component for the right section of the status bar
#[derive(Component)]
pub struct StatusBarRight;

/// Marker component for the status bar container
#[derive(Component)]
pub struct StatusBarContainer;

/// Setup the status bar UI hierarchy
///
/// Creates a horizontal container with left and right text sections.
/// The status bar is positioned at the bottom of the window.
fn setup_status_bar(mut commands: Commands) {
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Px(24.0),
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::Center,
                padding: UiRect::horizontal(Val::Px(8.0)),
                position_type: PositionType::Absolute,
                bottom: Val::Px(0.0),
                left: Val::Px(0.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.1, 0.1, 0.1, 0.9)),
            ZIndex(1000),
            StatusBarContainer,
        ))
        .with_children(|parent| {
            // Left section
            parent.spawn((
                Text::new(""),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(Color::srgb(0.9, 0.9, 0.9)),
                StatusBarLeft,
            ));

            // Right section
            parent.spawn((
                Text::new(""),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(Color::srgb(0.9, 0.9, 0.9)),
                StatusBarRight,
            ));
        });
}

/// Update status bar content when dirty flags are set
///
/// Converts RenderItem sequences to Bevy Text with appropriate styling.
fn update_status_bar_system(
    mut status: ResMut<StatusBarState>,
    mut left_query: Query<&mut Text, (With<StatusBarLeft>, Without<StatusBarRight>)>,
    mut right_query: Query<&mut Text, (With<StatusBarRight>, Without<StatusBarLeft>)>,
) {
    if status.left_dirty {
        if let Ok(mut text) = left_query.get_single_mut() {
            **text = render_items_to_text(&status.left_items);
        }
    }

    if status.right_dirty {
        if let Ok(mut text) = right_query.get_single_mut() {
            **text = render_items_to_text(&status.right_items);
        }
    }

    // Clear dirty flags after rendering
    if status.left_dirty || status.right_dirty {
        status.clear_dirty();
    }
}

/// Convert a sequence of RenderItems to Bevy Text string
///
/// Processes the items sequentially, building up plain text.
/// Note: In Bevy 0.15+, text styling is handled via separate components
/// (TextColor, TextFont) rather than inline sections. For now, we just
/// concatenate the text content and use the first color we encounter.
///
/// # Arguments
///
/// * `items` - Slice of RenderItem elements to convert
///
/// # Returns
///
/// A tuple of (text_string, text_color) for rendering
pub fn render_items_to_text(items: &[RenderItem]) -> String {
    let mut result = String::new();

    for item in items {
        match item {
            RenderItem::Text(s) => {
                result.push_str(s);
            }
            RenderItem::Icon(icon) => {
                // Icons are rendered as text for now
                result.push_str(icon);
            }
            RenderItem::Foreground(_color) => {
                // Color changes are not yet supported in the simplified implementation
                // Future: could spawn multiple text entities with different colors
            }
            RenderItem::ForegroundAnsi(_ansi) => {
                // Color changes are not yet supported
            }
            RenderItem::Background(_color) => {
                // Background colors are not directly supported in Bevy text styling
            }
            RenderItem::BackgroundAnsi(_ansi) => {
                // Background colors are not directly supported
            }
            RenderItem::Bold => {
                // Bold styling not yet supported - would need font variants
            }
            RenderItem::Italic => {
                // Italic styling not yet supported - would need font variants
            }
            RenderItem::Underline(_style) => {
                // Underline not yet supported
            }
            RenderItem::Strikethrough => {
                // Strikethrough not yet supported
            }
            RenderItem::ResetAttributes => {
                // Reset has no effect in simplified implementation
            }
            RenderItem::ResetForeground => {
                // Reset has no effect in simplified implementation
            }
            RenderItem::ResetBackground => {
                // Reset has no effect in simplified implementation
            }
            RenderItem::Spacer => {
                result.push(' ');
            }
            RenderItem::Padding(count) => {
                for _ in 0..*count {
                    result.push(' ');
                }
            }
            RenderItem::Separator(sep) => {
                result.push_str(sep);
            }
        }
    }

    result
}

/// Convert Color enum to Bevy color
#[allow(dead_code)] // Will be used in Phase 2 for color support
fn color_to_bevy(color: &StatusColor) -> Color {
    match color {
        StatusColor::Rgb(r, g, b) => {
            Color::srgb_u8(*r, *g, *b)
        }
        StatusColor::Hex(hex) => {
            parse_hex_color(hex)
        }
        StatusColor::Named(name) => {
            parse_named_color(name)
        }
    }
}

/// Convert AnsiColor to Bevy color
#[allow(dead_code)] // Will be used in Phase 2 for color support
fn ansi_color_to_bevy(ansi: &AnsiColor) -> Color {
    let (r, g, b) = ansi.to_rgb();
    Color::srgb_u8(r, g, b)
}

/// Parse hex color string to Bevy color
///
/// Supports formats: "#RRGGBB" or "RRGGBB"
#[allow(dead_code)] // Will be used in Phase 2 for color support
fn parse_hex_color(hex: &str) -> Color {
    let hex = hex.trim_start_matches('#');

    if hex.len() != 6 {
        // Invalid hex format, return default
        return Color::srgb(0.9, 0.9, 0.9);
    }

    let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(255);
    let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(255);
    let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(255);

    Color::srgb_u8(r, g, b)
}

/// Parse named color to Bevy color
///
/// Supports basic CSS color names
#[allow(dead_code)] // Will be used in Phase 2 for color support
fn parse_named_color(name: &str) -> Color {
    match name.to_lowercase().as_str() {
        "black" => Color::srgb(0.0, 0.0, 0.0),
        "white" => Color::srgb(1.0, 1.0, 1.0),
        "red" => Color::srgb(1.0, 0.0, 0.0),
        "green" => Color::srgb(0.0, 1.0, 0.0),
        "blue" => Color::srgb(0.0, 0.0, 1.0),
        "yellow" => Color::srgb(1.0, 1.0, 0.0),
        "cyan" => Color::srgb(0.0, 1.0, 1.0),
        "magenta" => Color::srgb(1.0, 0.0, 1.0),
        "orange" => Color::srgb(1.0, 0.65, 0.0),
        "purple" => Color::srgb(0.5, 0.0, 0.5),
        "pink" => Color::srgb(1.0, 0.75, 0.8),
        "gray" | "grey" => Color::srgb(0.5, 0.5, 0.5),
        "darkgray" | "darkgrey" => Color::srgb(0.25, 0.25, 0.25),
        "lightgray" | "lightgrey" => Color::srgb(0.75, 0.75, 0.75),
        _ => Color::srgb(0.9, 0.9, 0.9), // Default fallback
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_items_to_text_simple() {
        let items = vec![
            RenderItem::Text("Hello".to_string()),
            RenderItem::Text("World".to_string()),
        ];

        let text = render_items_to_text(&items);
        assert_eq!(text, "HelloWorld");
    }

    #[test]
    fn test_render_items_with_color() {
        let items = vec![
            RenderItem::Foreground(StatusColor::Hex("#ff0000".to_string())),
            RenderItem::Text("Red".to_string()),
            RenderItem::ResetAttributes,
            RenderItem::Text("Normal".to_string()),
        ];

        let text = render_items_to_text(&items);
        assert_eq!(text, "RedNormal");
    }

    #[test]
    fn test_render_items_with_padding() {
        let items = vec![
            RenderItem::Text("A".to_string()),
            RenderItem::Padding(3),
            RenderItem::Text("B".to_string()),
        ];

        let text = render_items_to_text(&items);
        assert_eq!(text, "A   B");
    }

    #[test]
    fn test_render_items_with_separator() {
        let items = vec![
            RenderItem::Text("Section 1".to_string()),
            RenderItem::Separator(" | ".to_string()),
            RenderItem::Text("Section 2".to_string()),
        ];

        let text = render_items_to_text(&items);
        assert_eq!(text, "Section 1 | Section 2");
    }

    #[test]
    fn test_render_items_empty() {
        let items = vec![];
        let text = render_items_to_text(&items);
        assert_eq!(text, "");
    }

    #[test]
    fn test_parse_hex_color_valid() {
        let color = parse_hex_color("#ff0000");
        let Color::Srgba(srgba) = color else {
            panic!("Expected Srgba color");
        };
        assert!((srgba.red - 1.0).abs() < 0.01);
        assert!((srgba.green - 0.0).abs() < 0.01);
        assert!((srgba.blue - 0.0).abs() < 0.01);
    }

    #[test]
    fn test_parse_hex_color_no_hash() {
        let color = parse_hex_color("00ff00");
        let Color::Srgba(srgba) = color else {
            panic!("Expected Srgba color");
        };
        assert!((srgba.green - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_parse_named_color() {
        let red = parse_named_color("red");
        let Color::Srgba(srgba) = red else {
            panic!("Expected Srgba color");
        };
        assert!((srgba.red - 1.0).abs() < 0.01);

        let blue = parse_named_color("blue");
        let Color::Srgba(srgba) = blue else {
            panic!("Expected Srgba color");
        };
        assert!((srgba.blue - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_ansi_color_conversion() {
        let color = ansi_color_to_bevy(&AnsiColor::BrightRed);
        let Color::Srgba(srgba) = color else {
            panic!("Expected Srgba color");
        };
        assert!((srgba.red - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_status_bar_state() {
        let mut state = StatusBarState::default();
        assert!(!state.left_dirty);
        assert!(!state.right_dirty);

        state.set_left(vec![RenderItem::Text("Left".to_string())]);
        assert!(state.left_dirty);
        assert_eq!(state.left_items.len(), 1);

        state.set_right(vec![RenderItem::Text("Right".to_string())]);
        assert!(state.right_dirty);
        assert_eq!(state.right_items.len(), 1);

        state.clear_dirty();
        assert!(!state.left_dirty);
        assert!(!state.right_dirty);

        state.clear();
        assert!(state.left_dirty);
        assert!(state.right_dirty);
        assert_eq!(state.left_items.len(), 0);
        assert_eq!(state.right_items.len(), 0);
    }
}
