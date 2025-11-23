// Vimium-style link hints for clickable URLs and file paths
// Detects links in terminal output and provides keyboard shortcuts

use bevy::prelude::*;
use bevy::input::keyboard::KeyCode;
use bevy::text::{Text, TextStyle, TextSection};
use regex::Regex;
use std::sync::Arc;
use crate::integration::SharedMemoryReader;
use scarab_protocol::SharedState;

/// Plugin for link hint functionality
pub struct LinkHintsPlugin;

impl Plugin for LinkHintsPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<LinkDetector>()
            .init_resource::<LinkHintsState>()
            .add_event::<LinkActivatedEvent>()
            .add_systems(Update, (
                detect_links_system,
                show_hints_system,
                handle_hint_input_system,
                activate_link_system,
            ).chain());
    }
}

/// Detected link in terminal output
#[derive(Component, Clone, Debug)]
pub struct LinkHint {
    pub url: String,
    pub position: Vec2,
    pub hint_key: String,
    pub link_type: LinkType,
}

/// Type of detected link
#[derive(Clone, Debug, PartialEq)]
pub enum LinkType {
    Url,
    FilePath,
    Email,
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
            url_regex: Regex::new(
                r"https?://[^\s<>{}|\^~\[\]`]+|www\.[^\s<>{}|\^~\[\]`]+"
            ).unwrap(),
            // Match absolute and relative file paths
            filepath_regex: Regex::new(
                r"(?:~|\.{1,2}|/)?(?:[a-zA-Z0-9_\-./]+/)*[a-zA-Z0-9_\-.]+"
            ).unwrap(),
            // Match email addresses
            email_regex: Regex::new(
                r"[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}"
            ).unwrap(),
        }
    }
}

impl LinkDetector {
    /// Detect all links in text content
    pub fn detect(&self, text: &str) -> Vec<(String, LinkType)> {
        let mut links = Vec::new();

        // Detect URLs
        for cap in self.url_regex.find_iter(text) {
            links.push((cap.as_str().to_string(), LinkType::Url));
        }

        // Detect file paths (with basic validation)
        for cap in self.filepath_regex.find_iter(text) {
            let path = cap.as_str();
            // Filter out very short or unlikely paths
            if path.len() > 3 && (path.contains('/') || path.contains('.')) {
                links.push((path.to_string(), LinkType::FilePath));
            }
        }

        // Detect emails
        for cap in self.email_regex.find_iter(text) {
            links.push((cap.as_str().to_string(), LinkType::Email));
        }

        links
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
}

/// Event fired when a link is activated
#[derive(Event)]
pub struct LinkActivatedEvent {
    pub link: LinkHint,
}

/// Component for hint label display
#[derive(Component)]
struct HintLabel {
    hint_key: String,
}

/// Extract text from terminal grid via SharedMemoryReader
fn extract_terminal_text(state_reader: &SharedMemoryReader) -> String {
    let shared_ptr = state_reader.shmem.0.as_ptr() as *const SharedState;

    unsafe {
        let state = &*shared_ptr;
        crate::integration::extract_grid_text(state)
    }
}

/// Detect links in terminal grid
fn detect_links_system(
    detector: Res<LinkDetector>,
    mut state: ResMut<LinkHintsState>,
    keyboard: Res<ButtonInput<KeyCode>>,
    state_reader: Res<SharedMemoryReader>,
) {
    // Toggle link hints with Ctrl+K
    if keyboard.just_pressed(KeyCode::ControlLeft) && keyboard.pressed(KeyCode::KeyK) {
        state.active = !state.active;

        if state.active {
            // Get actual terminal text from SharedState
            let terminal_text = extract_terminal_text(&state_reader);
            let detected_links = detector.detect(&terminal_text);

            let hint_keys = LinkDetector::generate_hint_keys(detected_links.len());

            state.hints = detected_links
                .into_iter()
                .zip(hint_keys)
                .enumerate()
                .map(|(i, ((url, link_type), hint_key))| LinkHint {
                    url,
                    position: Vec2::new(100.0, 100.0 + i as f32 * 20.0), // TODO: Calculate from grid position
                    hint_key,
                    link_type,
                })
                .collect();

            info!("Detected {} links in terminal output", state.hints.len());
            state.current_input.clear();
        } else {
            state.hints.clear();
            state.current_input.clear();
        }
    }
}

/// Show hint labels on screen
fn show_hints_system(
    mut commands: Commands,
    state: Res<LinkHintsState>,
    existing_hints: Query<Entity, With<HintLabel>>,
    asset_server: Res<AssetServer>,
) {
    // Remove existing hint labels
    for entity in existing_hints.iter() {
        commands.entity(entity).despawn();
    }

    if !state.active {
        return;
    }

    // Spawn new hint labels
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

        let color = if matched {
            Color::srgb(0.0, 1.0, 0.0) // Green when matched
        } else {
            Color::srgb(1.0, 1.0, 0.0) // Yellow for hints
        };

        commands.spawn((
            HintLabel {
                hint_key: hint.hint_key.clone(),
            },
            Text2dBundle {
                text: Text::from_section(
                    &hint.hint_key,
                    TextStyle {
                        font_size: 16.0,
                        color,
                        ..default()
                    },
                ),
                transform: Transform::from_translation(hint.position.extend(100.0)),
                ..default()
            },
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
            if let Some(hint) = state.hints.iter().find(|h| h.hint_key == state.current_input) {
                event_writer.send(LinkActivatedEvent {
                    link: hint.clone(),
                });
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
) {
    for event in events.read() {
        match event.link.link_type {
            LinkType::Url => {
                info!("Opening URL: {}", event.link.url);
                // Open URL in browser using xdg-open (Linux), open (macOS), or start (Windows)
                #[cfg(target_os = "linux")]
                {
                    std::process::Command::new("xdg-open")
                        .arg(&event.link.url)
                        .spawn()
                        .ok();
                }
                #[cfg(target_os = "macos")]
                {
                    std::process::Command::new("open")
                        .arg(&event.link.url)
                        .spawn()
                        .ok();
                }
                #[cfg(target_os = "windows")]
                {
                    std::process::Command::new("cmd")
                        .args(&["/C", "start", &event.link.url])
                        .spawn()
                        .ok();
                }
            }
            LinkType::FilePath => {
                info!("Opening file: {}", event.link.url);
                // TODO: Open file in default editor or with $EDITOR
            }
            LinkType::Email => {
                info!("Opening email: {}", event.link.url);
                // TODO: Open email client
            }
        }
    }
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

        assert_eq!(links.len(), 2);
        assert_eq!(links[0].0, "https://example.com");
        assert_eq!(links[1].0, "www.github.com");
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
}
