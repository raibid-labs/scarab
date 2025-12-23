//! File breadcrumb bar showing current directory path with hintable segments
//!
//! Displays the current working directory as a breadcrumb path at the top of
//! the terminal, with each segment hintable for navigation.
//!
//! ## Keyboard Navigation
//!
//! Use the leader key system:
//! - `Space g a` through `Space g l`: Jump to breadcrumb segment
//! - Opens directory picker at the selected path

use crate::ipc::IpcChannel;
use crate::ui::leader_key::LeaderKeyActivatedEvent;
use bevy::prelude::*;
use scarab_protocol::ControlMessage;
use std::path::PathBuf;

/// Height of the breadcrumb bar in pixels
pub const BREADCRUMB_BAR_HEIGHT: f32 = 28.0;

/// Plugin for breadcrumb bar functionality
pub struct BreadcrumbPlugin;

impl Plugin for BreadcrumbPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<BreadcrumbState>()
            .add_event::<BreadcrumbSegmentSelectedEvent>()
            .add_event::<OpenDirectoryPickerEvent>()
            .add_systems(Startup, setup_breadcrumb_bar)
            .add_systems(
                Update,
                (
                    sync_breadcrumb_with_cwd,
                    handle_breadcrumb_leader_key,
                    update_breadcrumb_display,
                    handle_segment_selection,
                    handle_directory_picker_events,
                )
                    .chain(),
            );
    }
}

/// Breadcrumb state resource tracking current path and segments
#[derive(Resource)]
pub struct BreadcrumbState {
    /// Current working directory
    pub current_path: PathBuf,
    /// Path segments with hint keys
    pub segments: Vec<PathSegment>,
    /// Whether the directory picker is active
    pub picker_active: bool,
    /// Target path for directory picker (when active)
    pub picker_target: Option<PathBuf>,
    /// Dirty flag to trigger re-rendering
    pub dirty: bool,
}

impl Default for BreadcrumbState {
    fn default() -> Self {
        // Start with home directory
        let home = std::env::var("HOME").unwrap_or_else(|_| "/".to_string());
        let current_path = PathBuf::from(home);
        let segments = path_to_segments(&current_path);

        Self {
            current_path,
            segments,
            picker_active: false,
            picker_target: None,
            dirty: true,
        }
    }
}

impl BreadcrumbState {
    /// Update the current path and regenerate segments
    pub fn set_path(&mut self, path: PathBuf) {
        if path != self.current_path {
            self.current_path = path.clone();
            self.segments = path_to_segments(&path);
            self.dirty = true;
        }
    }

    /// Open the directory picker for a specific path
    pub fn open_picker(&mut self, path: PathBuf) {
        self.picker_target = Some(path);
        self.picker_active = true;
    }

    /// Close the directory picker
    pub fn close_picker(&mut self) {
        self.picker_active = false;
        self.picker_target = None;
    }
}

/// A single segment in the breadcrumb path
#[derive(Clone, Debug)]
pub struct PathSegment {
    /// Display name (directory name)
    pub name: String,
    /// Full path up to this segment
    pub full_path: PathBuf,
    /// Hint key for navigation (a, s, d, f, ...)
    pub hint_key: String,
    /// Whether this is the home directory
    pub is_home: bool,
}

/// Event fired when a breadcrumb segment is selected via hint
#[derive(Event)]
pub struct BreadcrumbSegmentSelectedEvent {
    pub segment: PathSegment,
}

/// Event to open the directory picker at a specific path
#[derive(Event)]
pub struct OpenDirectoryPickerEvent {
    pub path: PathBuf,
}

/// Marker component for the breadcrumb container
#[derive(Component)]
pub struct BreadcrumbContainer;

/// Marker component for breadcrumb text
#[derive(Component)]
pub struct BreadcrumbText;

/// Setup the breadcrumb bar UI at the top of the window
fn setup_breadcrumb_bar(mut commands: Commands) {
    // Slime theme colors
    let breadcrumb_bg = Color::srgba(0.12, 0.14, 0.14, 0.95); // Dark background
    let text_color = Color::srgb(0.66, 0.87, 0.35); // Slime green

    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Px(BREADCRUMB_BAR_HEIGHT),
                align_items: AlignItems::Center,
                padding: UiRect::horizontal(Val::Px(12.0)),
                position_type: PositionType::Absolute,
                top: Val::Px(0.0),
                left: Val::Px(0.0),
                ..default()
            },
            BackgroundColor(breadcrumb_bg),
            ZIndex(900), // Below hint overlays (1000+), above terminal content
            BreadcrumbContainer,
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new(""),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(text_color),
                BreadcrumbText,
            ));
        });
}

/// Update breadcrumb display when path changes
fn update_breadcrumb_display(
    mut state: ResMut<BreadcrumbState>,
    mut text_query: Query<&mut Text, With<BreadcrumbText>>,
) {
    if !state.dirty {
        return;
    }

    if let Ok(mut text) = text_query.get_single_mut() {
        **text = render_breadcrumb_text(&state.segments);
        state.dirty = false;
    }
}

/// Render breadcrumb segments to display text
fn render_breadcrumb_text(segments: &[PathSegment]) -> String {
    if segments.is_empty() {
        return String::from("/");
    }

    let mut result = String::new();

    for (i, segment) in segments.iter().enumerate() {
        if i > 0 {
            result.push_str(" / ");
        }

        // Show hint key in brackets
        result.push_str(&format!("[{}] ", segment.hint_key));

        // Show name (with home icon for home directory)
        if segment.is_home {
            result.push('~');
        } else {
            result.push_str(&segment.name);
        }
    }

    result
}

/// Sync breadcrumb with the current working directory
/// TODO: Replace with OSC 7 tracking from terminal when available
fn sync_breadcrumb_with_cwd(mut state: ResMut<BreadcrumbState>) {
    // For now, sync with the client process's current directory
    // This will be replaced with proper terminal CWD tracking via OSC 7
    if let Ok(cwd) = std::env::current_dir() {
        state.set_path(cwd);
    }
}

/// Handle breadcrumb segment selection events
fn handle_segment_selection(
    mut events: EventReader<BreadcrumbSegmentSelectedEvent>,
    mut picker_events: EventWriter<OpenDirectoryPickerEvent>,
) {
    for event in events.read() {
        info!("Breadcrumb segment selected: {:?}", event.segment.full_path);

        // Fire event to open the directory picker for this path
        picker_events.send(OpenDirectoryPickerEvent {
            path: event.segment.full_path.clone(),
        });
    }
}

/// Handle leader key events for breadcrumb navigation
/// Listens for "breadcrumb.go.X" commands from the leader key system
fn handle_breadcrumb_leader_key(
    mut leader_events: EventReader<LeaderKeyActivatedEvent>,
    state: Res<BreadcrumbState>,
    mut segment_events: EventWriter<BreadcrumbSegmentSelectedEvent>,
) {
    for event in leader_events.read() {
        // Check if this is a breadcrumb navigation command
        if let Some(hint_key) = event.command.strip_prefix("breadcrumb.go.") {
            // Find segment with matching hint key
            if let Some(segment) = state.segments.iter().find(|s| s.hint_key == hint_key) {
                info!(
                    "Leader key breadcrumb navigation '{}' -> {:?}",
                    hint_key, segment.full_path
                );
                segment_events.send(BreadcrumbSegmentSelectedEvent {
                    segment: segment.clone(),
                });
            } else {
                warn!("No breadcrumb segment for hint key '{}'", hint_key);
            }
        }
    }
}

/// Handle directory picker open events - integrates with Fusabi file browser plugin
fn handle_directory_picker_events(
    mut events: EventReader<OpenDirectoryPickerEvent>,
    mut state: ResMut<BreadcrumbState>,
    ipc: Option<Res<IpcChannel>>,
) {
    use scarab_protocol::MenuActionType;

    for event in events.read() {
        info!("Opening directory picker at: {:?}", event.path);
        state.open_picker(event.path.clone());

        // Send command to Fusabi file browser plugin via IPC
        if let Some(ref ipc) = ipc {
            let path_str = event.path.to_string_lossy();
            // Send plugin command to invoke file browser's picker command
            let command = format!("picker {}", path_str);
            ipc.send(ControlMessage::PluginMenuExecute {
                plugin_name: "scarab-file-browser".to_string(),
                action: MenuActionType::Command { command },
            });
            info!("Sent picker command to file browser plugin: {}", path_str);
        } else {
            warn!("IPC channel not available for directory picker");
        }
    }
}

/// Convert a path to breadcrumb segments
fn path_to_segments(path: &PathBuf) -> Vec<PathSegment> {
    let home = std::env::var("HOME").ok();
    let hint_keys = vec!["a", "s", "d", "f", "g", "h", "j", "k", "l"];

    // Handle home directory substitution
    let display_path = if let Some(ref home_str) = home {
        if let Ok(rel) = path.strip_prefix(home_str) {
            PathBuf::from("~").join(rel)
        } else {
            path.clone()
        }
    } else {
        path.clone()
    };

    let mut segments = Vec::new();
    let mut current = PathBuf::new();
    let mut hint_idx = 0;

    for component in display_path.components() {
        use std::path::Component;

        match component {
            Component::RootDir => {
                current.push("/");
                segments.push(PathSegment {
                    name: "/".to_string(),
                    full_path: current.clone(),
                    hint_key: hint_keys.get(hint_idx).unwrap_or(&"z").to_string(),
                    is_home: false,
                });
                hint_idx += 1;
            }
            Component::Normal(name) => {
                let name_str = name.to_string_lossy().to_string();
                let is_home = name_str == "~";

                if is_home {
                    // Resolve ~ to actual home path
                    if let Some(ref home_str) = home {
                        current = PathBuf::from(home_str);
                    }
                } else {
                    current.push(name);
                }

                segments.push(PathSegment {
                    name: name_str.clone(),
                    full_path: current.clone(),
                    hint_key: hint_keys.get(hint_idx).unwrap_or(&"z").to_string(),
                    is_home,
                });
                hint_idx += 1;
            }
            _ => {}
        }
    }

    segments
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_path_to_segments() {
        let path = PathBuf::from("/home/user/projects/scarab");
        let segments = path_to_segments(&path);

        assert_eq!(segments.len(), 5);
        assert_eq!(segments[0].name, "/");
        assert_eq!(segments[1].name, "home");
        assert_eq!(segments[2].name, "user");
        assert_eq!(segments[3].name, "projects");
        assert_eq!(segments[4].name, "scarab");
    }

    #[test]
    fn test_render_breadcrumb_text() {
        let segments = vec![
            PathSegment {
                name: "~".to_string(),
                full_path: PathBuf::from("/home/user"),
                hint_key: "a".to_string(),
                is_home: true,
            },
            PathSegment {
                name: "projects".to_string(),
                full_path: PathBuf::from("/home/user/projects"),
                hint_key: "s".to_string(),
                is_home: false,
            },
        ];

        let text = render_breadcrumb_text(&segments);
        assert!(text.contains("[a] ~"));
        assert!(text.contains("[s] projects"));
    }
}
