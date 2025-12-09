//! Theme resolver for mapping theme names to color palettes
//!
//! This module provides the `ThemeResolver` which:
//! - Maps theme names to actual color palettes
//! - Applies theme overrides to ColorConfig
//! - Validates theme names
//! - Provides a list of available themes

use crate::{ColorConfig, ColorPalette, ConfigError, Result};
use std::collections::HashMap;
use tracing::debug;

/// Built-in theme definition
struct ThemeDefinition {
    foreground: &'static str,
    background: &'static str,
    cursor: &'static str,
    selection_bg: &'static str,
    selection_fg: &'static str,
    /// ANSI colors 0-15 (black, red, green, yellow, blue, magenta, cyan, white, then bright variants)
    ansi: [&'static str; 16],
}

/// Theme resolver for loading and applying themes
pub struct ThemeResolver {
    themes: HashMap<&'static str, ThemeDefinition>,
}

impl ThemeResolver {
    /// Create a new theme resolver with built-in themes
    pub fn new() -> Self {
        let mut themes = HashMap::new();

        // Slime theme (default)
        themes.insert(
            "slime",
            ThemeDefinition {
                foreground: "#a8df5a",
                background: "#0d1208",
                cursor: "#a8df5a",
                selection_bg: "#3d5c1f",
                selection_fg: "#ffffff",
                ansi: [
                    "#0d1208", "#ff5555", "#a8df5a", "#f1fa8c", "#6272a4", "#ff79c6", "#8be9fd",
                    "#f8f8f2", "#44475a", "#ff6e6e", "#c4f07a", "#ffffa5", "#7c8dbd", "#ff92df",
                    "#a4ffff", "#ffffff",
                ],
            },
        );

        // Dracula theme
        themes.insert(
            "dracula",
            ThemeDefinition {
                foreground: "#f8f8f2",
                background: "#282a36",
                cursor: "#f8f8f2",
                selection_bg: "#44475a",
                selection_fg: "#f8f8f2",
                ansi: [
                    "#21222c", "#ff5555", "#50fa7b", "#f1fa8c", "#bd93f9", "#ff79c6", "#8be9fd",
                    "#f8f8f2", "#6272a4", "#ff6e6e", "#69ff94", "#ffffa5", "#d6acff", "#ff92df",
                    "#a4ffff", "#ffffff",
                ],
            },
        );

        // Nord theme
        themes.insert(
            "nord",
            ThemeDefinition {
                foreground: "#d8dee9",
                background: "#2e3440",
                cursor: "#d8dee9",
                selection_bg: "#434c5e",
                selection_fg: "#d8dee9",
                ansi: [
                    "#3b4252", "#bf616a", "#a3be8c", "#ebcb8b", "#81a1c1", "#b48ead", "#88c0d0",
                    "#e5e9f0", "#4c566a", "#bf616a", "#a3be8c", "#ebcb8b", "#81a1c1", "#b48ead",
                    "#8fbcbb", "#eceff4",
                ],
            },
        );

        // Monokai theme
        themes.insert(
            "monokai",
            ThemeDefinition {
                foreground: "#f8f8f2",
                background: "#272822",
                cursor: "#f8f8f2",
                selection_bg: "#49483e",
                selection_fg: "#f8f8f2",
                ansi: [
                    "#272822", "#f92672", "#a6e22e", "#f4bf75", "#66d9ef", "#ae81ff", "#a1efe4",
                    "#f8f8f2", "#75715e", "#f92672", "#a6e22e", "#f4bf75", "#66d9ef", "#ae81ff",
                    "#a1efe4", "#f9f8f5",
                ],
            },
        );

        // Gruvbox Dark theme
        themes.insert(
            "gruvbox-dark",
            ThemeDefinition {
                foreground: "#ebdbb2",
                background: "#282828",
                cursor: "#ebdbb2",
                selection_bg: "#504945",
                selection_fg: "#ebdbb2",
                ansi: [
                    "#282828", "#cc241d", "#98971a", "#d79921", "#458588", "#b16286", "#689d6a",
                    "#a89984", "#928374", "#fb4934", "#b8bb26", "#fabd2f", "#83a598", "#d3869b",
                    "#8ec07c", "#ebdbb2",
                ],
            },
        );

        // Tokyo Night theme
        themes.insert(
            "tokyo-night",
            ThemeDefinition {
                foreground: "#a9b1d6",
                background: "#1a1b26",
                cursor: "#c0caf5",
                selection_bg: "#33467c",
                selection_fg: "#c0caf5",
                ansi: [
                    "#15161e", "#f7768e", "#9ece6a", "#e0af68", "#7aa2f7", "#bb9af7", "#7dcfff",
                    "#a9b1d6", "#414868", "#f7768e", "#9ece6a", "#e0af68", "#7aa2f7", "#bb9af7",
                    "#7dcfff", "#c0caf5",
                ],
            },
        );

        // Catppuccin Mocha theme
        themes.insert(
            "catppuccin-mocha",
            ThemeDefinition {
                foreground: "#cdd6f4",
                background: "#1e1e2e",
                cursor: "#f5e0dc",
                selection_bg: "#45475a",
                selection_fg: "#cdd6f4",
                ansi: [
                    "#45475a", "#f38ba8", "#a6e3a1", "#f9e2af", "#89b4fa", "#f5c2e7", "#94e2d5",
                    "#bac2de", "#585b70", "#f38ba8", "#a6e3a1", "#f9e2af", "#89b4fa", "#f5c2e7",
                    "#94e2d5", "#a6adc8",
                ],
            },
        );

        // Solarized Dark theme
        themes.insert(
            "solarized-dark",
            ThemeDefinition {
                foreground: "#839496",
                background: "#002b36",
                cursor: "#839496",
                selection_bg: "#073642",
                selection_fg: "#93a1a1",
                ansi: [
                    "#073642", "#dc322f", "#859900", "#b58900", "#268bd2", "#d33682", "#2aa198",
                    "#eee8d5", "#002b36", "#cb4b16", "#586e75", "#657b83", "#839496", "#6c71c4",
                    "#93a1a1", "#fdf6e3",
                ],
            },
        );

        // One Dark theme
        themes.insert(
            "one-dark",
            ThemeDefinition {
                foreground: "#abb2bf",
                background: "#282c34",
                cursor: "#528bff",
                selection_bg: "#3e4451",
                selection_fg: "#abb2bf",
                ansi: [
                    "#282c34", "#e06c75", "#98c379", "#e5c07b", "#61afef", "#c678dd", "#56b6c2",
                    "#abb2bf", "#545862", "#e06c75", "#98c379", "#e5c07b", "#61afef", "#c678dd",
                    "#56b6c2", "#c8ccd4",
                ],
            },
        );

        Self { themes }
    }

    /// Resolve a theme name and apply it to a ColorConfig
    ///
    /// If the theme name is found, the ColorConfig will be updated with the theme's colors.
    /// Any existing custom color overrides in the ColorConfig will be preserved.
    pub fn resolve(&self, config: &mut ColorConfig) -> Result<()> {
        let theme_name = match &config.theme {
            Some(name) => name.as_str(),
            None => {
                debug!("No theme specified in config");
                return Ok(());
            }
        };

        debug!("Resolving theme: {}", theme_name);

        let theme = self.themes.get(theme_name).ok_or_else(|| {
            ConfigError::InvalidTheme(format!(
                "Theme '{}' not found. Available themes: {}",
                theme_name,
                self.available_themes().join(", ")
            ))
        })?;

        debug!("Found theme: {}", theme_name);

        // Only apply theme colors if not already overridden
        if config.foreground.is_none() {
            config.foreground = Some(theme.foreground.to_string());
        }
        if config.background.is_none() {
            config.background = Some(theme.background.to_string());
        }
        if config.cursor.is_none() {
            config.cursor = Some(theme.cursor.to_string());
        }
        if config.selection_background.is_none() {
            config.selection_background = Some(theme.selection_bg.to_string());
        }
        if config.selection_foreground.is_none() {
            config.selection_foreground = Some(theme.selection_fg.to_string());
        }

        // Apply palette (check if it's still the default palette)
        if config.palette == ColorPalette::default() {
            config.palette = ColorPalette {
                black: theme.ansi[0].to_string(),
                red: theme.ansi[1].to_string(),
                green: theme.ansi[2].to_string(),
                yellow: theme.ansi[3].to_string(),
                blue: theme.ansi[4].to_string(),
                magenta: theme.ansi[5].to_string(),
                cyan: theme.ansi[6].to_string(),
                white: theme.ansi[7].to_string(),
                bright_black: theme.ansi[8].to_string(),
                bright_red: theme.ansi[9].to_string(),
                bright_green: theme.ansi[10].to_string(),
                bright_yellow: theme.ansi[11].to_string(),
                bright_blue: theme.ansi[12].to_string(),
                bright_magenta: theme.ansi[13].to_string(),
                bright_cyan: theme.ansi[14].to_string(),
                bright_white: theme.ansi[15].to_string(),
            };
        }

        debug!("Applied theme: {}", theme_name);
        Ok(())
    }

    /// Get a list of all available theme names
    pub fn available_themes(&self) -> Vec<String> {
        self.themes.keys().map(|s| s.to_string()).collect()
    }

    /// Check if a theme exists
    pub fn theme_exists(&self, name: &str) -> bool {
        self.themes.contains_key(name)
    }

    /// Get all dark themes
    pub fn dark_themes(&self) -> Vec<String> {
        // All built-in themes are dark themes
        self.available_themes()
    }

    /// Create a ColorConfig from a theme name
    pub fn config_from_theme(&self, theme_name: &str) -> Result<ColorConfig> {
        let mut config = ColorConfig {
            theme: Some(theme_name.to_string()),
            ..Default::default()
        };
        self.resolve(&mut config)?;
        Ok(config)
    }
}

impl Default for ThemeResolver {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_theme_resolver_initialization() {
        let resolver = ThemeResolver::new();
        let themes = resolver.available_themes();
        assert!(!themes.is_empty(), "Should have built-in themes");
        assert!(resolver.theme_exists("slime"));
        assert!(resolver.theme_exists("dracula"));
        assert!(resolver.theme_exists("nord"));
    }

    #[test]
    fn test_resolve_theme() {
        let resolver = ThemeResolver::new();
        let mut config = ColorConfig {
            theme: Some("dracula".to_string()),
            foreground: None,
            background: None,
            cursor: None,
            selection_background: None,
            selection_foreground: None,
            palette: ColorPalette::default(),
            opacity: 1.0,
            dim_opacity: 0.7,
        };

        resolver.resolve(&mut config).unwrap();

        // Should have applied dracula theme colors
        assert!(config.foreground.is_some());
        assert!(config.background.is_some());
        assert!(config.cursor.is_some());
        assert_eq!(config.foreground, Some("#f8f8f2".to_string()));
        assert_eq!(config.background, Some("#282a36".to_string()));
    }

    #[test]
    fn test_resolve_preserves_overrides() {
        let resolver = ThemeResolver::new();
        let custom_fg = "#ff0000".to_string();
        let mut config = ColorConfig {
            theme: Some("dracula".to_string()),
            foreground: Some(custom_fg.clone()),
            ..Default::default()
        };

        resolver.resolve(&mut config).unwrap();

        // Custom foreground should be preserved
        assert_eq!(config.foreground, Some(custom_fg));
        // But background should be applied from theme
        assert!(config.background.is_some());
    }

    #[test]
    fn test_resolve_invalid_theme() {
        let resolver = ThemeResolver::new();
        let mut config = ColorConfig {
            theme: Some("nonexistent-theme".to_string()),
            ..Default::default()
        };

        let result = resolver.resolve(&mut config);
        assert!(result.is_err());
    }

    #[test]
    fn test_resolve_no_theme() {
        let resolver = ThemeResolver::new();
        let mut config = ColorConfig {
            theme: None,
            ..Default::default()
        };

        // Should not error when no theme is specified
        resolver.resolve(&mut config).unwrap();
    }

    #[test]
    fn test_theme_exists() {
        let resolver = ThemeResolver::new();
        assert!(resolver.theme_exists("dracula"));
        assert!(resolver.theme_exists("nord"));
        assert!(resolver.theme_exists("slime"));
        assert!(!resolver.theme_exists("nonexistent"));
    }

    #[test]
    fn test_config_from_theme() {
        let resolver = ThemeResolver::new();
        let config = resolver.config_from_theme("dracula").unwrap();

        assert_eq!(config.theme, Some("dracula".to_string()));
        assert!(config.foreground.is_some());
        assert!(config.background.is_some());
    }
}
