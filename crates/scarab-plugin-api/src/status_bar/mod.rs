//! Status Bar Rendering API for Scarab terminal emulator
//!
//! This module provides types and utilities for programmable status bars.
//! Similar to WezTerm's status bar API, it allows plugins to dynamically
//! update status bar content with rich styling and formatting.
//!
//! ## Example
//!
//! ```rust
//! use scarab_plugin_api::status_bar::{RenderItem, Color, StatusBarUpdate, StatusBarSide};
//!
//! // Create a styled status bar update
//! let items = vec![
//!     RenderItem::Foreground(Color::Hex("#7aa2f7".to_string())),
//!     RenderItem::Text("~/project".to_string()),
//!     RenderItem::Text(" | ".to_string()),
//!     RenderItem::Bold,
//!     RenderItem::Text("12:34".to_string()),
//!     RenderItem::ResetAttributes,
//! ];
//!
//! let update = StatusBarUpdate {
//!     window_id: 1,
//!     side: StatusBarSide::Right,
//!     items,
//! };
//! ```

use serde::{Deserialize, Serialize};

/// A single rendering element for status bar content
///
/// RenderItems are processed sequentially to build styled text for the status bar.
/// They can represent text content, color changes, text attributes, or layout elements.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum RenderItem {
    /// Display text content
    ///
    /// Renders the provided string with the current text style.
    ///
    /// # Example
    ///
    /// ```rust
    /// use scarab_plugin_api::status_bar::RenderItem;
    ///
    /// let item = RenderItem::Text("Hello, World!".to_string());
    /// ```
    Text(String),

    /// Display a Nerd Font icon
    ///
    /// Renders an icon from the Nerd Font icon set by name.
    /// The icon will use the current foreground color and attributes.
    ///
    /// # Example
    ///
    /// ```rust
    /// use scarab_plugin_api::status_bar::RenderItem;
    ///
    /// let item = RenderItem::Icon("nf-fa-battery_full".to_string());
    /// ```
    Icon(String),

    /// Set foreground (text) color
    ///
    /// Changes the text color for all subsequent text items until
    /// another color change or reset.
    ///
    /// # Example
    ///
    /// ```rust
    /// use scarab_plugin_api::status_bar::{RenderItem, Color};
    ///
    /// let item = RenderItem::Foreground(Color::Rgb(122, 162, 247));
    /// ```
    Foreground(Color),

    /// Set foreground color using ANSI color
    ///
    /// Alternative to `Foreground` using standard ANSI color names.
    ///
    /// # Example
    ///
    /// ```rust
    /// use scarab_plugin_api::status_bar::{RenderItem, AnsiColor};
    ///
    /// let item = RenderItem::ForegroundAnsi(AnsiColor::BrightBlue);
    /// ```
    ForegroundAnsi(AnsiColor),

    /// Set background color
    ///
    /// Changes the background color for all subsequent text items.
    ///
    /// # Example
    ///
    /// ```rust
    /// use scarab_plugin_api::status_bar::{RenderItem, Color};
    ///
    /// let item = RenderItem::Background(Color::Named("darkgray".to_string()));
    /// ```
    Background(Color),

    /// Set background color using ANSI color
    ///
    /// Alternative to `Background` using standard ANSI color names.
    ///
    /// # Example
    ///
    /// ```rust
    /// use scarab_plugin_api::status_bar::{RenderItem, AnsiColor};
    ///
    /// let item = RenderItem::BackgroundAnsi(AnsiColor::Black);
    /// ```
    BackgroundAnsi(AnsiColor),

    /// Enable bold text attribute
    ///
    /// Makes all subsequent text bold until reset or another weight change.
    ///
    /// # Example
    ///
    /// ```rust
    /// use scarab_plugin_api::status_bar::RenderItem;
    ///
    /// let items = vec![
    ///     RenderItem::Bold,
    ///     RenderItem::Text("Important".to_string()),
    /// ];
    /// ```
    Bold,

    /// Enable italic text attribute
    ///
    /// Makes all subsequent text italic until reset.
    ///
    /// # Example
    ///
    /// ```rust
    /// use scarab_plugin_api::status_bar::RenderItem;
    ///
    /// let items = vec![
    ///     RenderItem::Italic,
    ///     RenderItem::Text("Emphasis".to_string()),
    /// ];
    /// ```
    Italic,

    /// Enable underline with specified style
    ///
    /// Underlines all subsequent text with the given style until reset.
    ///
    /// # Example
    ///
    /// ```rust
    /// use scarab_plugin_api::status_bar::{RenderItem, UnderlineStyle};
    ///
    /// let items = vec![
    ///     RenderItem::Underline(UnderlineStyle::Curly),
    ///     RenderItem::Text("Warning".to_string()),
    /// ];
    /// ```
    Underline(UnderlineStyle),

    /// Enable strikethrough text attribute
    ///
    /// Strikes through all subsequent text until reset.
    ///
    /// # Example
    ///
    /// ```rust
    /// use scarab_plugin_api::status_bar::RenderItem;
    ///
    /// let items = vec![
    ///     RenderItem::Strikethrough,
    ///     RenderItem::Text("Deprecated".to_string()),
    /// ];
    /// ```
    Strikethrough,

    /// Reset all text attributes to defaults
    ///
    /// Clears all colors, bold, italic, underline, and strikethrough attributes.
    ///
    /// # Example
    ///
    /// ```rust
    /// use scarab_plugin_api::status_bar::RenderItem;
    ///
    /// let items = vec![
    ///     RenderItem::Bold,
    ///     RenderItem::Text("Bold".to_string()),
    ///     RenderItem::ResetAttributes,
    ///     RenderItem::Text("Normal".to_string()),
    /// ];
    /// ```
    ResetAttributes,

    /// Reset foreground color to default
    ///
    /// Clears only the foreground color, keeping other attributes.
    ///
    /// # Example
    ///
    /// ```rust
    /// use scarab_plugin_api::status_bar::{RenderItem, Color};
    ///
    /// let items = vec![
    ///     RenderItem::Bold,
    ///     RenderItem::Foreground(Color::Hex("#ff0000".to_string())),
    ///     RenderItem::Text("Red & Bold".to_string()),
    ///     RenderItem::ResetForeground,
    ///     RenderItem::Text("Default color, still bold".to_string()),
    /// ];
    /// ```
    ResetForeground,

    /// Reset background color to default
    ///
    /// Clears only the background color, keeping other attributes.
    ///
    /// # Example
    ///
    /// ```rust
    /// use scarab_plugin_api::status_bar::{RenderItem, Color};
    ///
    /// let items = vec![
    ///     RenderItem::Background(Color::Hex("#333333".to_string())),
    ///     RenderItem::Text("Dark background".to_string()),
    ///     RenderItem::ResetBackground,
    ///     RenderItem::Text("Default background".to_string()),
    /// ];
    /// ```
    ResetBackground,

    /// Insert flexible spacing
    ///
    /// Creates space that expands to fill available room.
    /// Useful for pushing content to opposite ends of the status bar.
    ///
    /// # Example
    ///
    /// ```rust
    /// use scarab_plugin_api::status_bar::RenderItem;
    ///
    /// let items = vec![
    ///     RenderItem::Text("Left".to_string()),
    ///     RenderItem::Spacer,
    ///     RenderItem::Text("Right".to_string()),
    /// ];
    /// ```
    Spacer,

    /// Insert fixed spacing
    ///
    /// Adds a fixed number of space characters (cells).
    ///
    /// # Example
    ///
    /// ```rust
    /// use scarab_plugin_api::status_bar::RenderItem;
    ///
    /// let items = vec![
    ///     RenderItem::Text("A".to_string()),
    ///     RenderItem::Padding(3),
    ///     RenderItem::Text("B".to_string()),
    /// ];
    /// ```
    Padding(u8),

    /// Insert a separator string
    ///
    /// Convenience item for common separators like " | " or " • ".
    ///
    /// # Example
    ///
    /// ```rust
    /// use scarab_plugin_api::status_bar::RenderItem;
    ///
    /// let items = vec![
    ///     RenderItem::Text("Section 1".to_string()),
    ///     RenderItem::Separator(" | ".to_string()),
    ///     RenderItem::Text("Section 2".to_string()),
    /// ];
    /// ```
    Separator(String),
}

/// Color specification for text rendering
///
/// Supports RGB values, hex notation, and named colors.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type", content = "value")]
pub enum Color {
    /// RGB color with 8-bit components
    ///
    /// # Example
    ///
    /// ```rust
    /// use scarab_plugin_api::status_bar::Color;
    ///
    /// let color = Color::Rgb(122, 162, 247);  // Light blue
    /// ```
    Rgb(u8, u8, u8),

    /// Hexadecimal color string
    ///
    /// Supports standard hex color notation with or without '#' prefix.
    ///
    /// # Example
    ///
    /// ```rust
    /// use scarab_plugin_api::status_bar::Color;
    ///
    /// let color = Color::Hex("#7aa2f7".to_string());
    /// ```
    Hex(String),

    /// Named color from theme or CSS color names
    ///
    /// Supports standard CSS color names like "red", "blue", "darkgray", etc.
    ///
    /// # Example
    ///
    /// ```rust
    /// use scarab_plugin_api::status_bar::Color;
    ///
    /// let color = Color::Named("cornflowerblue".to_string());
    /// ```
    Named(String),
}

/// Standard 16-color ANSI palette
///
/// Provides the standard ANSI color set with both normal and bright variants.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum AnsiColor {
    /// Standard black (typically #000000)
    Black,
    /// Standard red (typically #800000)
    Red,
    /// Standard green (typically #008000)
    Green,
    /// Standard yellow (typically #808000)
    Yellow,
    /// Standard blue (typically #000080)
    Blue,
    /// Standard magenta (typically #800080)
    Magenta,
    /// Standard cyan (typically #008080)
    Cyan,
    /// Standard white (typically #c0c0c0)
    White,
    /// Bright/bold black, also known as gray (typically #808080)
    BrightBlack,
    /// Bright/bold red (typically #ff0000)
    BrightRed,
    /// Bright/bold green (typically #00ff00)
    BrightGreen,
    /// Bright/bold yellow (typically #ffff00)
    BrightYellow,
    /// Bright/bold blue (typically #0000ff)
    BrightBlue,
    /// Bright/bold magenta (typically #ff00ff)
    BrightMagenta,
    /// Bright/bold cyan (typically #00ffff)
    BrightCyan,
    /// Bright/bold white (typically #ffffff)
    BrightWhite,
}

impl AnsiColor {
    /// Convert ANSI color to RGB values
    ///
    /// Returns a tuple of (r, g, b) values for the color.
    /// Uses a standard terminal color palette.
    ///
    /// # Example
    ///
    /// ```rust
    /// use scarab_plugin_api::status_bar::AnsiColor;
    ///
    /// let (r, g, b) = AnsiColor::BrightBlue.to_rgb();
    /// assert_eq!((r, g, b), (0, 0, 255));
    /// ```
    pub fn to_rgb(&self) -> (u8, u8, u8) {
        match self {
            AnsiColor::Black => (0, 0, 0),
            AnsiColor::Red => (128, 0, 0),
            AnsiColor::Green => (0, 128, 0),
            AnsiColor::Yellow => (128, 128, 0),
            AnsiColor::Blue => (0, 0, 128),
            AnsiColor::Magenta => (128, 0, 128),
            AnsiColor::Cyan => (0, 128, 128),
            AnsiColor::White => (192, 192, 192),
            AnsiColor::BrightBlack => (128, 128, 128),
            AnsiColor::BrightRed => (255, 0, 0),
            AnsiColor::BrightGreen => (0, 255, 0),
            AnsiColor::BrightYellow => (255, 255, 0),
            AnsiColor::BrightBlue => (0, 0, 255),
            AnsiColor::BrightMagenta => (255, 0, 255),
            AnsiColor::BrightCyan => (0, 255, 255),
            AnsiColor::BrightWhite => (255, 255, 255),
        }
    }
}

/// Underline style options
///
/// Defines different visual styles for underlined text.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum UnderlineStyle {
    /// Single straight line
    Single,
    /// Double straight lines
    Double,
    /// Wavy/curly line (often used for spelling errors)
    Curly,
    /// Dotted line
    Dotted,
    /// Dashed line
    Dashed,
}

/// Status bar side/position
///
/// Identifies which side of the status bar to update.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum StatusBarSide {
    /// Left side of the status bar
    Left,
    /// Right side of the status bar
    Right,
}

/// IPC message for status bar updates
///
/// Sent from daemon to client (or from plugin to UI) to update
/// status bar content.
///
/// # Example
///
/// ```rust
/// use scarab_plugin_api::status_bar::{StatusBarUpdate, StatusBarSide, RenderItem};
///
/// let update = StatusBarUpdate {
///     window_id: 1,
///     side: StatusBarSide::Right,
///     items: vec![
///         RenderItem::Text("12:34 PM".to_string()),
///     ],
/// };
/// ```
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StatusBarUpdate {
    /// ID of the window to update
    pub window_id: u64,
    /// Which side of the status bar to update
    pub side: StatusBarSide,
    /// Render items to display
    pub items: Vec<RenderItem>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ansi_color_to_rgb() {
        assert_eq!(AnsiColor::Black.to_rgb(), (0, 0, 0));
        assert_eq!(AnsiColor::Red.to_rgb(), (128, 0, 0));
        assert_eq!(AnsiColor::BrightRed.to_rgb(), (255, 0, 0));
        assert_eq!(AnsiColor::BrightWhite.to_rgb(), (255, 255, 255));
    }

    #[test]
    fn test_render_item_serialization() {
        let item = RenderItem::Text("Hello".to_string());
        let json = serde_json::to_string(&item).unwrap();
        let deserialized: RenderItem = serde_json::from_str(&json).unwrap();

        match deserialized {
            RenderItem::Text(s) => assert_eq!(s, "Hello"),
            _ => panic!("Expected Text variant"),
        }
    }

    #[test]
    fn test_color_serialization() {
        let color = Color::Rgb(255, 128, 64);
        let json = serde_json::to_string(&color).unwrap();
        let deserialized: Color = serde_json::from_str(&json).unwrap();

        match deserialized {
            Color::Rgb(r, g, b) => {
                assert_eq!(r, 255);
                assert_eq!(g, 128);
                assert_eq!(b, 64);
            }
            _ => panic!("Expected Rgb variant"),
        }
    }

    #[test]
    fn test_status_bar_update() {
        let update = StatusBarUpdate {
            window_id: 42,
            side: StatusBarSide::Left,
            items: vec![
                RenderItem::Bold,
                RenderItem::Text("Test".to_string()),
            ],
        };

        assert_eq!(update.window_id, 42);
        assert_eq!(update.items.len(), 2);
    }

    #[test]
    fn test_complex_status_bar_styling() {
        let items = vec![
            RenderItem::Foreground(Color::Hex("#7aa2f7".to_string())),
            RenderItem::Text("~/project".to_string()),
            RenderItem::ResetForeground,
            RenderItem::Separator(" | ".to_string()),
            RenderItem::Bold,
            RenderItem::Foreground(Color::Named("green".to_string())),
            RenderItem::Text("100%".to_string()),
            RenderItem::ResetAttributes,
        ];

        assert_eq!(items.len(), 8);

        // Verify first item
        match &items[0] {
            RenderItem::Foreground(Color::Hex(s)) => assert_eq!(s, "#7aa2f7"),
            _ => panic!("Expected Foreground item"),
        }
    }
}
