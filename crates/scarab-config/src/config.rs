//! Core configuration structures

use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Root configuration structure
#[derive(Debug, Clone, Deserialize, Serialize, Resource)]
#[serde(default)]
pub struct ScarabConfig {
    pub terminal: TerminalConfig,
    pub font: FontConfig,
    pub colors: ColorConfig,
    pub keybindings: KeyBindings,
    pub ui: UiConfig,
    pub plugins: PluginConfig,
    pub sessions: SessionConfig,
    pub telemetry: TelemetryConfig,
}

impl Default for ScarabConfig {
    fn default() -> Self {
        Self {
            terminal: TerminalConfig::default(),
            font: FontConfig::default(),
            colors: ColorConfig::default(),
            keybindings: KeyBindings::default(),
            ui: UiConfig::default(),
            plugins: PluginConfig::default(),
            sessions: SessionConfig::default(),
            telemetry: TelemetryConfig::default(),
        }
    }
}

impl ScarabConfig {
    /// Merge another config into this one (for local overrides)
    pub fn merge(&mut self, other: ScarabConfig) {
        // Terminal settings
        if other.terminal != TerminalConfig::default() {
            self.terminal = other.terminal;
        }

        // Font settings
        if other.font != FontConfig::default() {
            self.font = other.font;
        }

        // Colors
        if other.colors != ColorConfig::default() {
            self.colors = other.colors;
        }

        // Keybindings
        self.keybindings.custom.extend(other.keybindings.custom);

        // UI settings
        if other.ui != UiConfig::default() {
            self.ui = other.ui;
        }

        // Plugins
        self.plugins.enabled.extend(other.plugins.enabled);
        self.plugins.config.extend(other.plugins.config);

        // Sessions
        if other.sessions != SessionConfig::default() {
            self.sessions = other.sessions;
        }

        // Telemetry
        if other.telemetry != TelemetryConfig::default() {
            self.telemetry = other.telemetry;
        }
    }
}

/// Terminal emulator settings
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(default)]
pub struct TerminalConfig {
    pub default_shell: String,
    pub scrollback_lines: u32,
    pub alt_screen: bool,
    pub scroll_multiplier: f32,
    pub auto_scroll: bool,
    pub columns: u16,
    pub rows: u16,
}

impl Default for TerminalConfig {
    fn default() -> Self {
        Self {
            default_shell: std::env::var("SHELL").unwrap_or_else(|_| "/bin/zsh".to_string()),
            scrollback_lines: 10_000,
            alt_screen: true,
            scroll_multiplier: 3.0,
            auto_scroll: true,
            columns: 80,
            rows: 24,
        }
    }
}

/// Font configuration
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(default)]
pub struct FontConfig {
    pub family: String,
    pub size: f32,
    pub line_height: f32,
    pub fallback: Vec<String>,
    pub bold_is_bright: bool,
    pub use_thin_strokes: bool,
}

impl Default for FontConfig {
    fn default() -> Self {
        Self {
            family: "JetBrains Mono".to_string(),
            size: 14.0,
            line_height: 1.2,
            fallback: vec![
                "Fira Code".to_string(),
                "DejaVu Sans Mono".to_string(),
                "Menlo".to_string(),
            ],
            bold_is_bright: true,
            use_thin_strokes: false,
        }
    }
}

/// Color configuration
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(default)]
pub struct ColorConfig {
    /// Theme name (e.g., "dracula", "nord", "monokai")
    pub theme: Option<String>,

    /// Custom colors (override theme)
    pub foreground: Option<String>,
    pub background: Option<String>,
    pub cursor: Option<String>,
    pub selection_background: Option<String>,
    pub selection_foreground: Option<String>,

    /// Color palette (16 colors)
    pub palette: ColorPalette,

    /// Transparency settings
    pub opacity: f32,
    pub dim_opacity: f32,
}

impl Default for ColorConfig {
    fn default() -> Self {
        Self {
            theme: Some("dracula".to_string()),
            foreground: None,
            background: None,
            cursor: None,
            selection_background: None,
            selection_foreground: None,
            palette: ColorPalette::default(),
            opacity: 1.0,
            dim_opacity: 0.7,
        }
    }
}

/// 16-color ANSI palette
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(default)]
pub struct ColorPalette {
    // Normal colors
    pub black: String,
    pub red: String,
    pub green: String,
    pub yellow: String,
    pub blue: String,
    pub magenta: String,
    pub cyan: String,
    pub white: String,

    // Bright colors
    pub bright_black: String,
    pub bright_red: String,
    pub bright_green: String,
    pub bright_yellow: String,
    pub bright_blue: String,
    pub bright_magenta: String,
    pub bright_cyan: String,
    pub bright_white: String,
}

impl Default for ColorPalette {
    fn default() -> Self {
        // Dracula theme colors
        Self {
            black: "#21222c".to_string(),
            red: "#ff5555".to_string(),
            green: "#50fa7b".to_string(),
            yellow: "#f1fa8c".to_string(),
            blue: "#bd93f9".to_string(),
            magenta: "#ff79c6".to_string(),
            cyan: "#8be9fd".to_string(),
            white: "#f8f8f2".to_string(),
            bright_black: "#6272a4".to_string(),
            bright_red: "#ff6e6e".to_string(),
            bright_green: "#69ff94".to_string(),
            bright_yellow: "#ffffa5".to_string(),
            bright_blue: "#d6acff".to_string(),
            bright_magenta: "#ff92df".to_string(),
            bright_cyan: "#a4ffff".to_string(),
            bright_white: "#ffffff".to_string(),
        }
    }
}

/// Key bindings configuration
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(default)]
pub struct KeyBindings {
    pub leader_key: String,
    pub copy_mode: String,
    pub paste: String,
    pub search: String,
    pub command_palette: String,
    pub new_window: String,
    pub close_window: String,
    pub next_tab: String,
    pub prev_tab: String,

    /// Custom keybindings (action -> key)
    pub custom: HashMap<String, String>,
}

impl Default for KeyBindings {
    fn default() -> Self {
        Self {
            leader_key: "Space".to_string(),
            copy_mode: "Ctrl+Shift+C".to_string(),
            paste: "Ctrl+Shift+V".to_string(),
            search: "Ctrl+Shift+F".to_string(),
            command_palette: "Ctrl+Shift+P".to_string(),
            new_window: "Ctrl+Shift+N".to_string(),
            close_window: "Ctrl+Shift+W".to_string(),
            next_tab: "Ctrl+Tab".to_string(),
            prev_tab: "Ctrl+Shift+Tab".to_string(),
            custom: HashMap::new(),
        }
    }
}

/// UI behavior configuration
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(default)]
pub struct UiConfig {
    pub link_hints: bool,
    pub command_palette: bool,
    pub animations: bool,
    pub smooth_scroll: bool,
    pub show_tabs: bool,
    pub tab_position: TabPosition,
    pub cursor_style: CursorStyle,
    pub cursor_blink: bool,
    pub cursor_blink_interval: u32,
    pub window_icon: Option<String>, // Path to custom icon (PNG format, optional)
    pub search_case_sensitive: bool, // Case-sensitive search by default
    pub search_use_regex: bool,      // Use regex search by default
}

impl Default for UiConfig {
    fn default() -> Self {
        Self {
            link_hints: true,
            command_palette: true,
            animations: true,
            smooth_scroll: true,
            show_tabs: true,
            tab_position: TabPosition::Top,
            cursor_style: CursorStyle::Block,
            cursor_blink: true,
            cursor_blink_interval: 750,
            window_icon: None, // No custom icon by default
            search_case_sensitive: false,
            search_use_regex: false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum TabPosition {
    Top,
    Bottom,
    Left,
    Right,
}

#[derive(Debug, Clone, Copy, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum CursorStyle {
    Block,
    Beam,
    Underline,
}

/// Plugin configuration
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(default)]
pub struct PluginConfig {
    pub enabled: Vec<String>,
    pub config: HashMap<String, serde_json::Value>,
}

impl Default for PluginConfig {
    fn default() -> Self {
        Self {
            enabled: vec![],
            config: HashMap::new(),
        }
    }
}

/// Session management configuration
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(default)]
pub struct SessionConfig {
    pub restore_on_startup: bool,
    pub auto_save_interval: u32,
    pub save_scrollback: bool,
    pub working_directory: Option<String>,
}

impl Default for SessionConfig {
    fn default() -> Self {
        Self {
            restore_on_startup: false,
            auto_save_interval: 300, // 5 minutes
            save_scrollback: true,
            working_directory: None,
        }
    }
}

/// Telemetry and logging configuration
///
/// Controls observability features for development and debugging.
/// All settings are opt-in (disabled by default) to avoid performance impact.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(default)]
pub struct TelemetryConfig {
    /// Log compositor FPS every N seconds (0 = disabled)
    ///
    /// Example: Set to 5 to log FPS stats every 5 seconds
    /// Output: [INFO] Compositor: 60.2 fps (avg over 5s), 3012 frames
    pub fps_log_interval_secs: u64,

    /// Log sequence number changes in compositor
    ///
    /// Helps debug shared memory synchronization issues
    /// Output: [DEBUG] Sequence: 1234 -> 1235, dirty_cells: 847
    pub log_sequence_changes: bool,

    /// Log dirty region sizes when blitting to shared memory
    ///
    /// Useful for understanding update patterns and performance
    /// Output: [DEBUG] Blit: 847 dirty cells (4.2% of grid)
    pub log_dirty_regions: bool,

    /// Log pane lifecycle events (create, destroy, reader status)
    ///
    /// Validates tab/pane flow in the orchestrator
    /// Output: [INFO] PaneOrchestrator: Pane 1 created, reader task spawned
    pub log_pane_events: bool,
}

impl Default for TelemetryConfig {
    fn default() -> Self {
        Self {
            fps_log_interval_secs: 0,
            log_sequence_changes: false,
            log_dirty_regions: false,
            log_pane_events: false,
        }
    }
}

impl TelemetryConfig {
    /// Create telemetry config from environment variables
    ///
    /// Environment variables override config file settings:
    /// - SCARAB_LOG_FPS=5 - Log FPS every 5 seconds
    /// - SCARAB_LOG_SEQUENCE=1 - Enable sequence logging
    /// - SCARAB_LOG_DIRTY=1 - Enable dirty region logging
    /// - SCARAB_LOG_PANES=1 - Enable pane lifecycle logging
    pub fn from_env(&self) -> Self {
        let mut config = self.clone();

        // FPS logging interval
        if let Ok(val) = std::env::var("SCARAB_LOG_FPS") {
            if let Ok(secs) = val.parse::<u64>() {
                config.fps_log_interval_secs = secs;
            }
        }

        // Sequence number logging
        if let Ok(val) = std::env::var("SCARAB_LOG_SEQUENCE") {
            config.log_sequence_changes = val == "1" || val.to_lowercase() == "true";
        }

        // Dirty region logging
        if let Ok(val) = std::env::var("SCARAB_LOG_DIRTY") {
            config.log_dirty_regions = val == "1" || val.to_lowercase() == "true";
        }

        // Pane events logging
        if let Ok(val) = std::env::var("SCARAB_LOG_PANES") {
            config.log_pane_events = val == "1" || val.to_lowercase() == "true";
        }

        config
    }

    /// Check if any telemetry is enabled
    pub fn is_enabled(&self) -> bool {
        self.fps_log_interval_secs > 0
            || self.log_sequence_changes
            || self.log_dirty_regions
            || self.log_pane_events
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = ScarabConfig::default();
        assert_eq!(config.font.size, 14.0);
        assert_eq!(config.terminal.scrollback_lines, 10_000);
        assert!(config.ui.link_hints);
    }

    #[test]
    fn test_config_merge() {
        let mut base = ScarabConfig::default();
        let mut override_config = ScarabConfig::default();
        override_config.font.size = 16.0;

        base.merge(override_config);
        assert_eq!(base.font.size, 16.0);
    }

    #[test]
    fn test_serialize_deserialize() {
        let config = ScarabConfig::default();
        let toml = toml::to_string(&config).unwrap();
        let parsed: ScarabConfig = toml::from_str(&toml).unwrap();
        assert_eq!(config.font.size, parsed.font.size);
    }

    #[test]
    fn test_telemetry_default_disabled() {
        let config = TelemetryConfig::default();
        assert!(!config.is_enabled());
        assert_eq!(config.fps_log_interval_secs, 0);
        assert!(!config.log_sequence_changes);
        assert!(!config.log_dirty_regions);
        assert!(!config.log_pane_events);
    }

    #[test]
    fn test_telemetry_is_enabled() {
        let mut config = TelemetryConfig::default();
        config.fps_log_interval_secs = 5;
        assert!(config.is_enabled());

        config.fps_log_interval_secs = 0;
        config.log_pane_events = true;
        assert!(config.is_enabled());
    }
}
