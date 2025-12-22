// Vimium-style link hints for clickable URLs, file paths, and status bar plugins
// Detects links in terminal output and provides keyboard shortcuts
// Also provides hints for status bar plugin items

use crate::events::StatusSide;
use crate::integration::SharedMemoryReader;
use crate::plugin_host::PluginStatusItem;
use crate::rendering::text::TextRenderer;
use crate::ui::status_bar::{StatusBarContainer, STATUS_BAR_HEIGHT};
use bevy::input::keyboard::KeyCode;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use regex::Regex;
use std::time::{Duration, Instant};

/// Double-tap detection window for Esc+Esc
const DOUBLE_TAP_WINDOW: Duration = Duration::from_millis(300);

/// Plugin for link hint functionality
pub struct LinkHintsPlugin;

impl Plugin for LinkHintsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<LinkDetector>()
            .init_resource::<LinkHintsState>()
            .init_resource::<EscapeDoubleTap>()
            .add_event::<LinkActivatedEvent>()
            .add_event::<PluginMenuRequestEvent>()
            .add_systems(
                Update,
                (
                    detect_links_system,
                    show_hints_system,
                    handle_hint_input_system,
                    activate_link_system,
                )
                    .chain(),
            );
    }
}

/// Tracks Escape key double-tap state
#[derive(Resource)]
struct EscapeDoubleTap {
    last_escape: Option<Instant>,
}

impl Default for EscapeDoubleTap {
    fn default() -> Self {
        Self { last_escape: None }
    }
}

/// Detected link in terminal output with grid position
#[derive(Component, Clone, Debug)]
pub struct LinkHint {
    pub url: String,
    pub position: Vec2,
    pub grid_col: u16,
    pub grid_row: u16,
    pub hint_key: String,
    pub link_type: LinkType,
}

/// Type of detected link or hintable element
#[derive(Clone, Debug, PartialEq)]
pub enum LinkType {
    Url,
    FilePath,
    Email,
    /// Status bar plugin item (plugin_id stored in url field)
    StatusBarPlugin,
}

/// Link detector with regex patterns
#[derive(Resource)]
pub struct LinkDetector {
    url_regex: Regex,
    filepath_regex: Regex,
    email_regex: Regex,
}

impl Default for LinkDetector {
    fn default() -> Self {
        Self {
            // Match HTTP(S) URLs
            url_regex: Regex::new(r"https?://[^\s<>{}|\^~\[\]`]+|www\.[^\s<>{}|\^~\[\]`]+")
                .unwrap(),
            // Match absolute and relative file paths
            filepath_regex: Regex::new(r"(?:~|\.{1,2}|/)?(?:[a-zA-Z0-9_\-./]+/)*[a-zA-Z0-9_\-.]+")
                .unwrap(),
            // Match email addresses
            email_regex: Regex::new(r"[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}").unwrap(),
        }
    }
}

impl LinkDetector {
    /// Detect all links in text content with their grid positions
    pub fn detect_with_positions(&self, text: &str) -> Vec<(String, LinkType, usize, usize)> {
        let mut links = Vec::new();

        // Split text into lines and track positions
        for (row, line) in text.lines().enumerate() {
            // Detect URLs
            for m in self.url_regex.find_iter(line) {
                links.push((m.as_str().to_string(), LinkType::Url, m.start(), row));
            }

            // Detect file paths (with basic validation)
            for m in self.filepath_regex.find_iter(line) {
                let path = m.as_str();
                // Filter out very short or unlikely paths
                if path.len() > 3 && (path.contains('/') || path.contains('.')) {
                    links.push((path.to_string(), LinkType::FilePath, m.start(), row));
                }
            }

            // Detect emails
            for m in self.email_regex.find_iter(line) {
                links.push((m.as_str().to_string(), LinkType::Email, m.start(), row));
            }
        }

        links
    }

    /// Detect all links in text content (legacy API without positions)
    pub fn detect(&self, text: &str) -> Vec<(String, LinkType)> {
        self.detect_with_positions(text)
            .into_iter()
            .map(|(url, link_type, _, _)| (url, link_type))
            .collect()
    }

    /// Generate hint keys (aa, ab, ac, ..., ba, bb, ...)
    pub fn generate_hint_keys(count: usize) -> Vec<String> {
        let chars = "abcdefghijklmnopqrstuvwxyz";
        let mut keys = Vec::new();

        for i in 0..count {
            let first = i / 26;
            let second = i % 26;
            let key = if first == 0 {
                chars.chars().nth(second).unwrap().to_string()
            } else {
                format!(
                    "{}{}",
                    chars.chars().nth(first - 1).unwrap(),
                    chars.chars().nth(second).unwrap()
                )
            };
            keys.push(key);
        }

        keys
    }
}

/// State of link hints system
#[derive(Resource, Default)]
pub struct LinkHintsState {
    pub active: bool,
    pub hints: Vec<LinkHint>,
    pub current_input: String,
    /// Set true on the frame hints are activated, prevents immediate cancel
    pub just_activated: bool,
}

/// Event fired when a link is activated
#[derive(Event)]
pub struct LinkActivatedEvent {
    pub link: LinkHint,
}

/// Event fired when a status bar plugin is selected via hints
/// This should trigger showing the plugin's command menu
#[derive(Event, Debug, Clone)]
pub struct PluginMenuRequestEvent {
    /// Plugin ID that was selected
    pub plugin_id: String,
    /// Position where the menu should appear (above the status bar item)
    pub position: Vec2,
}

/// Component for hint label display
#[derive(Component)]
struct HintLabel {
    #[allow(dead_code)]
    hint_key: String,
}

/// Extract text from terminal grid via SharedMemoryReader
///
/// Now uses safe SafeSharedState wrapper instead of raw pointers
fn extract_terminal_text(state_reader: &SharedMemoryReader) -> String {
    let safe_state = state_reader.get_safe_state();
    crate::integration::extract_grid_text(&safe_state)
}

/// Detect links in terminal grid and status bar plugins with accurate pixel positioning
fn detect_links_system(
    detector: Res<LinkDetector>,
    mut state: ResMut<LinkHintsState>,
    keyboard: Res<ButtonInput<KeyCode>>,
    state_reader: Res<SharedMemoryReader>,
    renderer: Res<TextRenderer>,
    mut escape_state: ResMut<EscapeDoubleTap>,
    window_query: Query<&Window, With<PrimaryWindow>>,
    // Status bar queries
    status_items: Query<&PluginStatusItem>,
    _status_bar_query: Query<&GlobalTransform, With<StatusBarContainer>>,
) {
    // Toggle link hints with Esc+Esc (double-tap Escape)
    let mut should_toggle = false;

    if keyboard.just_pressed(KeyCode::Escape) {
        let now = Instant::now();

        if let Some(last) = escape_state.last_escape {
            if now.duration_since(last) <= DOUBLE_TAP_WINDOW {
                // Double-tap detected!
                should_toggle = true;
                escape_state.last_escape = None; // Reset
            } else {
                // Too slow, start fresh
                escape_state.last_escape = Some(now);
            }
        } else {
            // First tap
            escape_state.last_escape = Some(now);
        }
    }

    if should_toggle {
        state.active = !state.active;
        state.just_activated = state.active; // Mark as just activated if turning on

        if state.active {
            // Get window size for coordinate calculation
            let Ok(window) = window_query.get_single() else {
                warn!("No primary window found for link hints");
                state.active = false;
                return;
            };

            let mut all_hints = Vec::new();

            // === DETECT STATUS BAR PLUGINS FIRST ===
            // Status bar plugins get hints a, b, c, d...
            // Collect left-side status items sorted by priority
            let mut left_items: Vec<_> = status_items
                .iter()
                .filter(|item| item.side == StatusSide::Left)
                .collect();
            left_items.sort_by(|a, b| b.priority.cmp(&a.priority));

            // Position hints at status bar level in world coordinates
            // Bottom of window is at y = -window.height/2, status bar is 24px tall
            // Account for: TopLeft anchor (hint extends downward 18px) and -2.0 offset in show_hints_system
            // This centers the 18px hint on the 24px status bar
            let status_bar_y = -window.height() * 0.5 + STATUS_BAR_HEIGHT - 1.0;
            let status_bar_x_start = -window.width() * 0.5 + 16.0; // 8px padding + 8px offset

            // Calculate approximate character width for positioning
            let approx_char_width = 8.0;
            let mut current_x = status_bar_x_start;

            for item in &left_items {
                // Create hint for this plugin at this position
                all_hints.push((
                    item.plugin_id.clone(),
                    LinkType::StatusBarPlugin,
                    current_x,
                    status_bar_y,
                    item.content.clone(),
                ));

                // Move x position for next item (approximate based on content length)
                current_x += (item.content.len() as f32 * approx_char_width) + 16.0; // + padding
            }

            // === DETECT TERMINAL CONTENT LINKS ===
            // Get actual terminal text from SharedState
            let terminal_text = extract_terminal_text(&state_reader);

            // Detect links with their grid positions
            let detected_links = detector.detect_with_positions(&terminal_text);

            // Get cell dimensions from renderer for accurate positioning
            let cell_width = renderer.cell_width;
            let cell_height = renderer.cell_height;

            // Terminal grid is positioned at top-left of window
            // Grid origin is at (-window.width/2, +window.height/2) in world coordinates
            let grid_origin_x = -window.width() * 0.5;
            let grid_origin_y = window.height() * 0.5;

            for (url, link_type, col, row) in detected_links {
                // Calculate position relative to grid origin
                // Grid cells go right (+x) and down (-y) from origin
                let x = grid_origin_x + (col as f32 * cell_width);
                let y = grid_origin_y - (row as f32 * cell_height);
                all_hints.push((url, link_type, x, y, String::new()));
            }

            // Generate hint keys for all hints
            let hint_keys = LinkDetector::generate_hint_keys(all_hints.len());

            // Create LinkHint structs
            state.hints = all_hints
                .into_iter()
                .zip(hint_keys)
                .map(|((url, link_type, x, y, _content), hint_key)| {
                    LinkHint {
                        url,
                        position: Vec2::new(x, y),
                        grid_col: 0, // Not used for status bar plugins
                        grid_row: 0,
                        hint_key,
                        link_type,
                    }
                })
                .collect();

            let plugin_count = left_items.len();
            let link_count = state.hints.len() - plugin_count;
            info!(
                "Detected {} status bar plugins and {} terminal links",
                plugin_count, link_count
            );
            state.current_input.clear();
        } else {
            state.hints.clear();
            state.current_input.clear();
        }
    }
}

/// Marker for hint label background sprites (for terminal hints)
#[derive(Component)]
struct HintLabelBackground;

/// Marker for UI-based hint labels (for status bar hints)
#[derive(Component)]
struct HintLabelUI;

/// Show hint labels on screen
fn show_hints_system(
    mut commands: Commands,
    state: Res<LinkHintsState>,
    existing_hints: Query<Entity, With<HintLabel>>,
    existing_backgrounds: Query<Entity, With<HintLabelBackground>>,
    window_query: Query<&Window, With<PrimaryWindow>>,
) {
    // Remove existing hint labels and backgrounds
    // Note: HintLabelUI entities also have HintLabel, so querying HintLabel covers all hints
    for entity in existing_hints.iter() {
        if let Some(mut entity_commands) = commands.get_entity(entity) {
            entity_commands.despawn();
        }
    }
    for entity in existing_backgrounds.iter() {
        if let Some(mut entity_commands) = commands.get_entity(entity) {
            entity_commands.despawn();
        }
    }

    if !state.active {
        return;
    }

    // Get window dimensions for coordinate conversion
    let window = window_query.get_single().ok();

    // Spawn new hint labels with backgrounds
    for hint in &state.hints {
        let mut matched = false;
        let mut partial_match = true;

        if !state.current_input.is_empty() {
            if hint.hint_key.starts_with(&state.current_input) {
                matched = hint.hint_key == state.current_input;
                partial_match = true;
            } else {
                partial_match = false;
            }
        }

        if !partial_match {
            continue;
        }

        // Vimium-style colors: yellow background with dark text, green when matched
        let (bg_color, text_color) = if matched {
            (
                Color::srgb(0.0, 0.8, 0.0),      // Green background
                Color::srgb(1.0, 1.0, 1.0),      // White text
            )
        } else {
            (
                Color::srgb(1.0, 0.9, 0.0),      // Yellow background
                Color::srgb(0.0, 0.0, 0.0),      // Black text
            )
        };

        // Status bar hints render as UI elements (to appear above the status bar)
        if hint.link_type == LinkType::StatusBarPlugin {
            if let Some(win) = window {
                // Convert world coordinates to screen coordinates
                // World: origin at center, Y up
                // Screen: origin at top-left, Y down
                let screen_x = hint.position.x + (win.width() / 2.0);
                let screen_y = (win.height() / 2.0) - hint.position.y;

                // Spawn UI hint element
                commands.spawn((
                    HintLabelUI,
                    HintLabel {
                        hint_key: hint.hint_key.clone(),
                    },
                    Node {
                        position_type: PositionType::Absolute,
                        left: Val::Px(screen_x),
                        top: Val::Px(screen_y - 3.0), // Slight upward adjustment
                        padding: UiRect::axes(Val::Px(4.0), Val::Px(2.0)),
                        ..default()
                    },
                    BackgroundColor(bg_color),
                    ZIndex(2000), // Above status bar (ZIndex 1000)
                ))
                .with_children(|parent| {
                    parent.spawn((
                        Text::new(&hint.hint_key),
                        TextFont {
                            font_size: 12.0,
                            ..default()
                        },
                        TextColor(text_color),
                    ));
                });
            }
            continue;
        }

        // Terminal hints render as sprites (positioned in world space)
        let char_width = 10.0;
        let bg_width = (hint.hint_key.len() as f32 * char_width) + 6.0;
        let bg_height = 18.0;

        // Position hint slightly offset from the link start
        let hint_x = hint.position.x;
        let hint_y = hint.position.y - 2.0; // Slight downward offset

        // Spawn background sprite first (lower z)
        commands.spawn((
            HintLabelBackground,
            bevy::sprite::Sprite {
                color: bg_color,
                custom_size: Some(Vec2::new(bg_width, bg_height)),
                anchor: bevy::sprite::Anchor::TopLeft,
                ..default()
            },
            Transform::from_translation(Vec3::new(hint_x, hint_y, 99.0)),
        ));

        // Spawn text label on top (higher z)
        commands.spawn((
            HintLabel {
                hint_key: hint.hint_key.clone(),
            },
            Text2d::new(&hint.hint_key),
            TextFont {
                font_size: 14.0,
                ..default()
            },
            TextColor(text_color),
            bevy::sprite::Anchor::TopLeft,
            Transform::from_translation(Vec3::new(hint_x + 3.0, hint_y - 2.0, 100.0)),
        ));
    }
}

/// Handle keyboard input for hint selection
fn handle_hint_input_system(
    mut state: ResMut<LinkHintsState>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut event_writer: EventWriter<LinkActivatedEvent>,
) {
    if !state.active {
        return;
    }

    // Skip escape cancel on the same frame hints were activated (Esc+Esc issue)
    if state.just_activated {
        state.just_activated = false;
        return; // Skip all input processing this frame
    }

    // Handle escape to cancel
    if keyboard.just_pressed(KeyCode::Escape) {
        state.active = false;
        state.hints.clear();
        state.current_input.clear();
        return;
    }

    // Handle letter input
    for key_code in keyboard.get_just_pressed() {
        if let Some(char) = keycode_to_char(*key_code) {
            state.current_input.push(char);

            // Check if we have a complete match
            if let Some(hint) = state
                .hints
                .iter()
                .find(|h| h.hint_key == state.current_input)
            {
                event_writer.send(LinkActivatedEvent { link: hint.clone() });
                state.active = false;
                state.hints.clear();
                state.current_input.clear();
            }
        }
    }
}

/// Activate selected link
fn activate_link_system(
    mut events: EventReader<LinkActivatedEvent>,
    mut plugin_menu_events: EventWriter<PluginMenuRequestEvent>,
) {
    for event in events.read() {
        match event.link.link_type {
            LinkType::Url => {
                info!("Opening URL: {}", event.link.url);
                if let Err(e) = open_url(&event.link.url) {
                    error!("Failed to open URL: {}", e);
                }
            }
            LinkType::FilePath => {
                info!("Opening file: {}", event.link.url);
                if let Err(e) = open_file(&event.link.url) {
                    error!("Failed to open file: {}", e);
                }
            }
            LinkType::Email => {
                info!("Opening email: {}", event.link.url);
                if let Err(e) = open_email(&event.link.url) {
                    error!("Failed to open email client: {}", e);
                }
            }
            LinkType::StatusBarPlugin => {
                // Emit event to open plugin command menu
                info!("Opening plugin menu for: {}", event.link.url);
                plugin_menu_events.send(PluginMenuRequestEvent {
                    plugin_id: event.link.url.clone(),
                    position: event.link.position,
                });
            }
        }
    }
}

/// Open a URL in the default browser using platform-specific commands
fn open_url(url: &str) -> Result<(), String> {
    info!("Attempting to open URL in browser: {}", url);

    // Ensure URL has protocol
    let full_url = if url.starts_with("www.") {
        format!("https://{}", url)
    } else {
        url.to_string()
    };

    #[cfg(target_os = "linux")]
    let result = std::process::Command::new("xdg-open")
        .arg(&full_url)
        .spawn()
        .map_err(|e| {
            format!(
                "Failed to launch xdg-open: {}. Make sure xdg-utils is installed.",
                e
            )
        });

    #[cfg(target_os = "macos")]
    let result = std::process::Command::new("open")
        .arg(&full_url)
        .spawn()
        .map_err(|e| format!("Failed to launch open command: {}", e));

    #[cfg(target_os = "windows")]
    let result = std::process::Command::new("cmd")
        .args(&["/C", "start", "", &full_url])
        .spawn()
        .map_err(|e| format!("Failed to launch cmd: {}", e));

    #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    let result = Err("Unsupported platform for opening URLs".to_string());

    result.map(|_| {
        info!("Successfully launched browser for URL: {}", full_url);
    })
}

/// Open a file in the default editor or application
fn open_file(path: &str) -> Result<(), String> {
    info!("Attempting to open file: {}", path);

    // Expand ~ to home directory
    let expanded_path = if path.starts_with('~') {
        if let Ok(home) = std::env::var("HOME") {
            path.replacen('~', &home, 1)
        } else {
            path.to_string()
        }
    } else {
        path.to_string()
    };

    // Check if file exists
    if !std::path::Path::new(&expanded_path).exists() {
        return Err(format!("File does not exist: {}", expanded_path));
    }

    // Try $EDITOR first for text files, then fall back to system default
    if let Ok(editor) = std::env::var("EDITOR") {
        info!("Opening file with $EDITOR ({}): {}", editor, expanded_path);

        let result = std::process::Command::new(&editor)
            .arg(&expanded_path)
            .spawn()
            .map_err(|e| format!("Failed to launch editor '{}': {}", editor, e));

        if result.is_ok() {
            return result.map(|_| ());
        } else {
            warn!("Failed to open with $EDITOR, falling back to system default");
        }
    }

    // Fall back to platform-specific open commands
    #[cfg(target_os = "linux")]
    let result = std::process::Command::new("xdg-open")
        .arg(&expanded_path)
        .spawn()
        .map_err(|e| {
            format!(
                "Failed to launch xdg-open: {}. Make sure xdg-utils is installed.",
                e
            )
        });

    #[cfg(target_os = "macos")]
    let result = std::process::Command::new("open")
        .arg(&expanded_path)
        .spawn()
        .map_err(|e| format!("Failed to launch open command: {}", e));

    #[cfg(target_os = "windows")]
    let result = std::process::Command::new("cmd")
        .args(&["/C", "start", "", &expanded_path])
        .spawn()
        .map_err(|e| format!("Failed to launch cmd: {}", e));

    #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    let result = Err("Unsupported platform for opening files".to_string());

    result.map(|_| {
        info!("Successfully opened file: {}", expanded_path);
    })
}

/// Open email client with mailto: link
fn open_email(email: &str) -> Result<(), String> {
    info!("Attempting to open email client for: {}", email);

    // Construct mailto: URL
    let mailto_url = if email.starts_with("mailto:") {
        email.to_string()
    } else {
        format!("mailto:{}", email)
    };

    #[cfg(target_os = "linux")]
    let result = std::process::Command::new("xdg-open")
        .arg(&mailto_url)
        .spawn()
        .map_err(|e| {
            format!(
                "Failed to launch xdg-open: {}. Make sure xdg-utils is installed.",
                e
            )
        });

    #[cfg(target_os = "macos")]
    let result = std::process::Command::new("open")
        .arg(&mailto_url)
        .spawn()
        .map_err(|e| format!("Failed to launch open command: {}", e));

    #[cfg(target_os = "windows")]
    let result = std::process::Command::new("cmd")
        .args(&["/C", "start", "", &mailto_url])
        .spawn()
        .map_err(|e| format!("Failed to launch cmd: {}", e));

    #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    let result = Err("Unsupported platform for opening email client".to_string());

    result.map(|_| {
        info!("Successfully opened email client for: {}", mailto_url);
    })
}

/// Convert KeyCode to character (a-z)
fn keycode_to_char(keycode: KeyCode) -> Option<char> {
    match keycode {
        KeyCode::KeyA => Some('a'),
        KeyCode::KeyB => Some('b'),
        KeyCode::KeyC => Some('c'),
        KeyCode::KeyD => Some('d'),
        KeyCode::KeyE => Some('e'),
        KeyCode::KeyF => Some('f'),
        KeyCode::KeyG => Some('g'),
        KeyCode::KeyH => Some('h'),
        KeyCode::KeyI => Some('i'),
        KeyCode::KeyJ => Some('j'),
        KeyCode::KeyK => Some('k'),
        KeyCode::KeyL => Some('l'),
        KeyCode::KeyM => Some('m'),
        KeyCode::KeyN => Some('n'),
        KeyCode::KeyO => Some('o'),
        KeyCode::KeyP => Some('p'),
        KeyCode::KeyQ => Some('q'),
        KeyCode::KeyR => Some('r'),
        KeyCode::KeyS => Some('s'),
        KeyCode::KeyT => Some('t'),
        KeyCode::KeyU => Some('u'),
        KeyCode::KeyV => Some('v'),
        KeyCode::KeyW => Some('w'),
        KeyCode::KeyX => Some('x'),
        KeyCode::KeyY => Some('y'),
        KeyCode::KeyZ => Some('z'),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_url_detection() {
        let detector = LinkDetector::default();
        let text = "Visit https://example.com or www.github.com";
        let links = detector.detect(text);

        // Note: The filepath regex also matches parts of URLs containing dots
        // so we verify that URLs are detected correctly
        let url_links: Vec<_> = links.iter().filter(|(_, t)| *t == LinkType::Url).collect();
        assert_eq!(url_links.len(), 2);
        assert_eq!(url_links[0].0, "https://example.com");
        assert_eq!(url_links[1].0, "www.github.com");
    }

    #[test]
    fn test_filepath_detection() {
        let detector = LinkDetector::default();
        let text = "Check /usr/local/bin or ./relative/path.txt";
        let links = detector.detect(text);

        assert!(links.iter().any(|(l, _)| l.contains("/usr/local/bin")));
        assert!(links.iter().any(|(l, _)| l.contains("./relative/path.txt")));
    }

    #[test]
    fn test_hint_key_generation() {
        let keys = LinkDetector::generate_hint_keys(30);

        assert_eq!(keys[0], "a");
        assert_eq!(keys[25], "z");
        assert_eq!(keys[26], "aa");
        assert_eq!(keys[27], "ab");
    }

    #[test]
    fn test_detect_with_positions() {
        let detector = LinkDetector::default();
        let text = "Line 1: https://example.com\nLine 2: test@email.com";
        let links = detector.detect_with_positions(text);

        // Should find URL on line 0 (row 0)
        let url_link = links
            .iter()
            .find(|(url, _, _, _)| url.contains("example.com"));
        assert!(url_link.is_some());
        let (_, _, col, row) = url_link.unwrap();
        assert_eq!(*row, 0);
        assert_eq!(*col, 8); // Position after "Line 1: "

        // Should find email on line 1 (row 1)
        let email_link = links
            .iter()
            .find(|(url, _, _, _)| url.contains("test@email.com"));
        assert!(email_link.is_some());
        let (_, _, col, row) = email_link.unwrap();
        assert_eq!(*row, 1);
        assert_eq!(*col, 8); // Position after "Line 2: "
    }
}
