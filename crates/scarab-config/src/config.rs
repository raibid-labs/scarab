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
    pub navigation: NavConfig,
    pub effects: EffectsConfig,
    pub ssh_domains: Vec<SshDomainConfig>,
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
            navigation: NavConfig::default(),
            telemetry: TelemetryConfig::default(),
            effects: EffectsConfig::default(),
            ssh_domains: Vec::new(),
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
        if other.telemetry != TelemetryConfig::default() {}

        // Navigation
        if other.navigation != NavConfig::default() {
            self.navigation = other.navigation;
        } else {
            // Merge custom keybindings even if rest is default
            self.navigation
                .keybindings
                .extend(other.navigation.keybindings);
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

    /// Enable telemetry HUD overlay
    ///
    /// Displays real-time performance metrics, cache stats, and navigation hints
    pub hud_enabled: bool,

    /// Telemetry HUD position on screen
    ///
    /// Options: "top-right", "top-left", "bottom-right", "bottom-left"
    pub hud_position: String,

    /// Telemetry HUD hotkey for toggling
    ///
    /// Default: "Ctrl+Shift+T"
    pub hud_hotkey: String,

    /// Include memory usage in HUD
    pub hud_show_memory: bool,

    /// Include cache statistics in HUD
    pub hud_show_cache: bool,

    /// Include navigation hint counts in HUD
    pub hud_show_hints: bool,
}

impl Default for TelemetryConfig {
    fn default() -> Self {
        Self {
            fps_log_interval_secs: 0,
            log_sequence_changes: false,
            log_dirty_regions: false,
            log_pane_events: false,
            hud_enabled: false,
            hud_position: "top-right".to_string(),
            hud_hotkey: "Ctrl+Shift+T".to_string(),
            hud_show_memory: true,
            hud_show_cache: true,
            hud_show_hints: true,
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

/// Visual effects configuration
/// Post-processing visual effects configuration
///
/// Controls GPU-accelerated shader effects for overlays and focused elements.
/// All effects can be disabled for performance or low-power mode.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(default)]
pub struct EffectsConfig {
    /// Enable background blur when overlays are visible
    pub overlay_blur_enabled: bool,

    /// Blur radius in pixels (higher = more blurred, more expensive)
    ///
    /// Typical values:
    /// - 2.0: Subtle blur, minimal performance impact
    /// - 4.0: Moderate blur (default)
    /// - 8.0: Heavy blur, noticeable performance cost
    pub overlay_blur_radius: f32,

    /// Blur intensity (0.0 = no effect, 1.0 = full blur)
    pub overlay_blur_intensity: f32,

    /// Enable glow effect on focused overlay borders
    pub overlay_glow_enabled: bool,

    /// Glow radius in pixels
    pub overlay_glow_radius: f32,

    /// Glow color (hex RGB format, e.g., "#8be9fd")
    pub overlay_glow_color: String,

    /// Glow intensity (0.0 = no glow, 1.0 = full intensity)
    pub overlay_glow_intensity: f32,

    /// Low-power mode: disables all effects to save GPU resources
    ///
    /// When enabled, all post-processing effects are skipped regardless
    /// of individual settings. Useful for battery-powered devices.
    pub low_power_mode: bool,
}

impl Default for EffectsConfig {
    fn default() -> Self {
        Self {
            overlay_blur_enabled: true,
            overlay_blur_radius: 4.0,
            overlay_blur_intensity: 0.8,
            overlay_glow_enabled: true,
            overlay_glow_radius: 6.0,
            overlay_glow_color: "#8be9fd".to_string(), // Dracula cyan
            overlay_glow_intensity: 0.7,
            low_power_mode: false,
        }
    }
}

impl EffectsConfig {
    /// Check if any effects are enabled (respecting low_power_mode)
    pub fn has_effects_enabled(&self) -> bool {
        !self.low_power_mode && (self.overlay_blur_enabled || self.overlay_glow_enabled)
    }

    /// Check if blur should be rendered
    pub fn should_render_blur(&self) -> bool {
        !self.low_power_mode && self.overlay_blur_enabled && self.overlay_blur_intensity > 0.0
    }

    /// Check if glow should be rendered
    pub fn should_render_glow(&self) -> bool {
        !self.low_power_mode && self.overlay_glow_enabled && self.overlay_glow_intensity > 0.0
    }

    /// Parse glow color from hex string to RGB values
    pub fn glow_color_rgb(&self) -> (f32, f32, f32) {
        // Parse hex color (e.g., "#8be9fd" or "8be9fd")
        let color_str = self.overlay_glow_color.trim_start_matches('#');

        if color_str.len() != 6 {
            // Invalid color, return default cyan
            return (0.545, 0.914, 0.992);
        }

        let r = u8::from_str_radix(&color_str[0..2], 16).unwrap_or(139) as f32 / 255.0;
        let g = u8::from_str_radix(&color_str[2..4], 16).unwrap_or(233) as f32 / 255.0;
        let b = u8::from_str_radix(&color_str[4..6], 16).unwrap_or(253) as f32 / 255.0;

        (r, g, b)
    }
}

#[cfg(test)]
mod effects_tests {
    use super::*;

    #[test]
    fn test_effects_config_default() {
        let config = EffectsConfig::default();
        assert!(config.overlay_blur_enabled);
        assert!(config.overlay_glow_enabled);
        assert!(!config.low_power_mode);
        assert_eq!(config.overlay_blur_radius, 4.0);
        assert_eq!(config.overlay_glow_color, "#8be9fd");
    }

    #[test]
    fn test_effects_enabled_check() {
        let mut config = EffectsConfig::default();
        assert!(config.has_effects_enabled());

        config.low_power_mode = true;
        assert!(!config.has_effects_enabled());

        config.low_power_mode = false;
        config.overlay_blur_enabled = false;
        config.overlay_glow_enabled = false;
        assert!(!config.has_effects_enabled());
    }

    #[test]
    fn test_should_render_blur() {
        let mut config = EffectsConfig::default();
        assert!(config.should_render_blur());

        config.low_power_mode = true;
        assert!(!config.should_render_blur());

        config.low_power_mode = false;
        config.overlay_blur_enabled = false;
        assert!(!config.should_render_blur());

        config.overlay_blur_enabled = true;
        config.overlay_blur_intensity = 0.0;
        assert!(!config.should_render_blur());
    }

    #[test]
    fn test_should_render_glow() {
        let mut config = EffectsConfig::default();
        assert!(config.should_render_glow());

        config.low_power_mode = true;
        assert!(!config.should_render_glow());

        config.low_power_mode = false;
        config.overlay_glow_enabled = false;
        assert!(!config.should_render_glow());

        config.overlay_glow_enabled = true;
        config.overlay_glow_intensity = 0.0;
        assert!(!config.should_render_glow());
    }

    #[test]
    fn test_glow_color_parsing() {
        let mut config = EffectsConfig::default();

        // Test default cyan color
        let (r, g, b) = config.glow_color_rgb();
        assert!((r - 0.545).abs() < 0.01);
        assert!((g - 0.914).abs() < 0.01);
        assert!((b - 0.992).abs() < 0.01);

        // Test white
        config.overlay_glow_color = "#ffffff".to_string();
        let (r, g, b) = config.glow_color_rgb();
        assert_eq!(r, 1.0);
        assert_eq!(g, 1.0);
        assert_eq!(b, 1.0);

        // Test without # prefix
        config.overlay_glow_color = "ff0000".to_string();
        let (r, g, b) = config.glow_color_rgb();
        assert_eq!(r, 1.0);
        assert_eq!(g, 0.0);
        assert_eq!(b, 0.0);

        // Test invalid color (should return default)
        config.overlay_glow_color = "invalid".to_string();
        let (r, g, b) = config.glow_color_rgb();
        assert!((r - 0.545).abs() < 0.01);
    }

    #[test]
    fn test_effects_config_serialization() {
        let config = EffectsConfig::default();
        let toml = toml::to_string(&config).unwrap();
        let parsed: EffectsConfig = toml::from_str(&toml).unwrap();
        assert_eq!(config, parsed);
    }

    #[test]
    fn test_effects_config_in_scarab_config() {
        let toml = r##"
            [effects]
            overlay_blur_enabled = false
            overlay_blur_radius = 8.0
            overlay_glow_enabled = true
            overlay_glow_color = "#ff00ff"
            low_power_mode = true
        "##;

        let config: ScarabConfig = toml::from_str(toml).unwrap();
        assert!(!config.effects.overlay_blur_enabled);
        assert_eq!(config.effects.overlay_blur_radius, 8.0);
        assert!(config.effects.overlay_glow_enabled);
        assert_eq!(config.effects.overlay_glow_color, "#ff00ff");
        assert!(config.effects.low_power_mode);
    }
}

/// Navigation style defining the keymap philosophy
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum NavStyle {
    /// Vimium-style: f for hints, Esc to cancel, letter keys for hint selection
    Vimium,
    /// Cosmos-style: space as leader key, then navigation submodes
    Cosmos,
    /// Spacemacs-style: SPC prefix for commands
    Spacemacs,
}

impl Default for NavStyle {
    fn default() -> Self {
        Self::Vimium
    }
}

/// Navigation configuration
///
/// Controls keyboard navigation behavior including hint mode, focusable elements,
/// and custom keybindings. Supports plugin conflict resolution.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(default)]
pub struct NavConfig {
    /// Navigation style (vimium, cosmos, spacemacs)
    pub style: NavStyle,

    /// Allow plugins to enter hint mode (conflict resolution)
    ///
    /// If false, only the built-in navigation system can trigger hints.
    /// Set to false if you have a plugin that conflicts with hint mode.
    pub allow_plugin_hint_mode: bool,

    /// Allow plugins to register focusable elements
    ///
    /// If false, plugins cannot add custom focusable elements to the navigation system.
    /// Set to prevent plugins from interfering with navigation.
    pub allow_plugin_focusables: bool,

    /// Custom keybindings (action_name -> key_combo)
    pub keybindings: HashMap<String, String>,
}

impl Default for NavConfig {
    fn default() -> Self {
        Self {
            style: NavStyle::default(),
            allow_plugin_hint_mode: true,
            allow_plugin_focusables: true,
            keybindings: HashMap::new(),
        }
    }
}

#[test]
fn test_nav_config_default() {
    let config = NavConfig::default();
    assert_eq!(config.style, NavStyle::Vimium);
    assert!(config.allow_plugin_hint_mode);
    assert!(config.allow_plugin_focusables);
    assert!(config.keybindings.is_empty());
}

#[test]
fn test_nav_config_with_custom_keybindings() {
    let toml = r#"
            style = "cosmos"
            allow_plugin_hint_mode = false
            allow_plugin_focusables = true

            [keybindings]
            enter_hints = "Ctrl+F"
            cancel = "Escape"
            prev_prompt = "Ctrl+Up"
        "#;

    let config: NavConfig = toml::from_str(toml).unwrap();
    assert_eq!(config.style, NavStyle::Cosmos);
    assert!(!config.allow_plugin_hint_mode);
    assert!(config.allow_plugin_focusables);
    assert_eq!(config.keybindings.len(), 3);
    assert_eq!(
        config.keybindings.get("enter_hints"),
        Some(&"Ctrl+F".to_string())
    );
    assert_eq!(
        config.keybindings.get("cancel"),
        Some(&"Escape".to_string())
    );
}

#[test]
fn test_nav_style_in_config() {
    let toml = r#"style = "vimium""#;
    let config: NavConfig = toml::from_str(toml).unwrap();
    assert_eq!(config.style, NavStyle::Vimium);

    let toml = r#"style = "cosmos""#;
    let config: NavConfig = toml::from_str(toml).unwrap();
    assert_eq!(config.style, NavStyle::Cosmos);

    let toml = r#"style = "spacemacs""#;
    let config: NavConfig = toml::from_str(toml).unwrap();
    assert_eq!(config.style, NavStyle::Spacemacs);
}

#[test]
fn test_scarab_config_with_navigation() {
    let toml = r#"
            [navigation]
            style = "spacemacs"
            allow_plugin_hint_mode = true

            [navigation.keybindings]
            enter_hints = "Ctrl+Space"
        "#;

    let config: ScarabConfig = toml::from_str(toml).unwrap();
    assert_eq!(config.navigation.style, NavStyle::Spacemacs);
    assert!(config.navigation.allow_plugin_hint_mode);
    assert_eq!(
        config.navigation.keybindings.get("enter_hints"),
        Some(&"Ctrl+Space".to_string())
    );
}

/// SSH domain configuration
///
/// Defines a remote SSH server that can host terminal panes.
/// Multiple SSH domains can be configured for different servers.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(default)]
pub struct SshDomainConfig {
    /// Unique identifier for this SSH domain
    pub id: String,

    /// Human-readable name
    pub name: String,

    /// SSH server hostname or IP address
    pub host: String,

    /// SSH server port (default: 22)
    pub port: u16,

    /// SSH username
    pub user: String,

    /// Authentication method
    #[serde(flatten)]
    pub auth: SshAuthConfig,

    /// Connection timeout in seconds
    pub connect_timeout: u64,

    /// Enable SSH agent forwarding
    pub forward_agent: bool,

    /// Default remote working directory
    pub remote_cwd: Option<String>,
}

impl Default for SshDomainConfig {
    fn default() -> Self {
        Self {
            id: "ssh-default".to_string(),
            name: "SSH Server".to_string(),
            host: "localhost".to_string(),
            port: 22,
            user: std::env::var("USER").unwrap_or_else(|_| "root".to_string()),
            auth: SshAuthConfig::default(),
            connect_timeout: 10,
            forward_agent: false,
            remote_cwd: None,
        }
    }
}

/// SSH authentication configuration
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(tag = "auth_type", rename_all = "lowercase")]
pub enum SshAuthConfig {
    /// SSH agent authentication (default)
    Agent,

    /// Public key file authentication
    PublicKey {
        /// Path to private key file
        key_path: String,
        /// Optional passphrase for encrypted keys
        passphrase: Option<String>,
    },

    /// Password authentication (least secure)
    Password {
        /// Password (consider using environment variable)
        password: String,
    },
}

impl Default for SshAuthConfig {
    fn default() -> Self {
        Self::Agent
    }
}

#[cfg(test)]
mod ssh_config_tests {
    use super::*;

    #[test]
    fn test_ssh_domain_config_default() {
        let config = SshDomainConfig::default();
        assert_eq!(config.port, 22);
        assert_eq!(config.connect_timeout, 10);
        assert!(!config.forward_agent);
        assert!(matches!(config.auth, SshAuthConfig::Agent));
    }

    #[test]
    fn test_ssh_domain_config_deserialize() {
        let toml = r#"
            id = "myserver"
            name = "My Server"
            host = "example.com"
            port = 2222
            user = "alice"
            auth_type = "publickey"
            key_path = "/home/alice/.ssh/id_rsa"
            connect_timeout = 30
            forward_agent = true
            remote_cwd = "/home/alice/projects"
        "#;

        let config: SshDomainConfig = toml::from_str(toml).unwrap();
        assert_eq!(config.id, "myserver");
        assert_eq!(config.name, "My Server");
        assert_eq!(config.host, "example.com");
        assert_eq!(config.port, 2222);
        assert_eq!(config.user, "alice");
        assert_eq!(config.connect_timeout, 30);
        assert!(config.forward_agent);
        assert_eq!(config.remote_cwd, Some("/home/alice/projects".to_string()));

        match config.auth {
            SshAuthConfig::PublicKey { key_path, .. } => {
                assert_eq!(key_path, "/home/alice/.ssh/id_rsa");
            }
            _ => panic!("Expected PublicKey auth"),
        }
    }

    #[test]
    fn test_scarab_config_with_ssh_domains() {
        let toml = r#"
            [[ssh_domains]]
            id = "dev"
            name = "Development Server"
            host = "dev.example.com"
            user = "developer"
            auth_type = "agent"

            [[ssh_domains]]
            id = "prod"
            name = "Production Server"
            host = "prod.example.com"
            user = "admin"
            auth_type = "publickey"
            key_path = "/root/.ssh/prod_key"
        "#;

        let config: ScarabConfig = toml::from_str(toml).unwrap();
        assert_eq!(config.ssh_domains.len(), 2);
        assert_eq!(config.ssh_domains[0].id, "dev");
        assert_eq!(config.ssh_domains[1].id, "prod");
    }
}
