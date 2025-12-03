//! Shell integration prompt marker rendering and navigation
//!
//! This module handles the client-side visualization and navigation of OSC 133
//! prompt markers received from the daemon. Features:
//! - Gutter indicators for prompt locations
//! - Color-coded markers (blue for prompts, green/red for command results)
//! - Keyboard navigation (Ctrl+Up/Down to jump between prompts)

use bevy::prelude::*;
use bevy::sprite::MeshMaterial2d;
use bevy::render::mesh::Mesh2d;
use scarab_protocol::{PromptMarkerInfo, TerminalMetrics};

use crate::ipc::RemoteMessageEvent;

/// Resource storing received prompt markers from the daemon
#[derive(Resource, Default)]
pub struct PromptMarkers {
    /// List of prompt markers from daemon (sorted by line number)
    pub markers: Vec<PromptMarkerInfo>,
    /// Currently highlighted marker index (for keyboard navigation)
    pub current_index: Option<usize>,
    /// Target scroll line for navigation (set when navigating)
    pub target_scroll_line: Option<u32>,
}

impl PromptMarkers {
    /// Find the previous prompt marker from a given line
    ///
    /// Searches backwards through markers to find the nearest prompt start
    /// marker before the specified line.
    pub fn previous_prompt(&self, from_line: u32) -> Option<usize> {
        self.markers
            .iter()
            .enumerate()
            .rev()
            .find(|(_, m)| m.line < from_line && m.is_prompt_start())
            .map(|(i, _)| i)
    }

    /// Find the next prompt marker from a given line
    ///
    /// Searches forwards through markers to find the nearest prompt start
    /// marker after the specified line.
    pub fn next_prompt(&self, from_line: u32) -> Option<usize> {
        self.markers
            .iter()
            .enumerate()
            .find(|(_, m)| m.line > from_line && m.is_prompt_start())
            .map(|(i, _)| i)
    }

    /// Update markers from daemon message
    pub fn update_markers(&mut self, new_markers: Vec<PromptMarkerInfo>) {
        self.markers = new_markers;
        // Sort by line number to ensure binary search works
        self.markers.sort_by_key(|m| m.line);

        // Reset current index if out of bounds
        if let Some(idx) = self.current_index {
            if idx >= self.markers.len() {
                self.current_index = None;
            }
        }
    }
}

/// Marker component for gutter indicator entities
#[derive(Component)]
pub struct PromptGutterMarker {
    pub line: u32,
    pub marker_type: u8,
}

/// Determine the color for a marker based on its type and exit code
///
/// Color scheme:
/// - PromptStart (0): Blue - indicates a new prompt
/// - CommandFinished (3): Green (success) or Red (failure) - based on exit code
/// - Other types: Gray - less important markers
fn marker_color(marker_type: u8, exit_code: Option<i32>) -> Color {
    match marker_type {
        0 => Color::srgb(0.3, 0.7, 1.0), // PromptStart - blue
        3 => {
            // CommandFinished - green for success, red for failure
            if exit_code == Some(0) {
                Color::srgb(0.3, 0.9, 0.3) // Success - green
            } else {
                Color::srgb(0.9, 0.3, 0.3) // Failure - red
            }
        }
        _ => Color::srgb(0.5, 0.5, 0.5), // Other markers - gray
    }
}

/// System to render gutter markers for prompts
///
/// This system:
/// 1. Despawns all existing gutter markers when markers change
/// 2. Spawns new marker entities for each prompt/command marker
/// 3. Positions markers in the gutter to the left of the terminal grid
pub fn render_gutter_markers(
    mut commands: Commands,
    markers: Res<PromptMarkers>,
    metrics: Res<TerminalMetrics>,
    existing: Query<Entity, With<PromptGutterMarker>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    // Only update if markers changed
    if !markers.is_changed() {
        return;
    }

    // Despawn all existing markers
    for entity in existing.iter() {
        commands.entity(entity).despawn();
    }

    // Gutter configuration
    let gutter_width = 8.0; // Pixels from left edge
    let marker_radius = 3.0; // Circle radius in pixels

    // Spawn new markers for each prompt/command marker
    for marker in &markers.markers {
        // Only show prompt start and command finished markers in the gutter
        // (Other marker types are less visually important)
        if marker.marker_type != 0 && marker.marker_type != 3 {
            continue;
        }

        // Calculate vertical position based on line number
        // Note: Y-axis is positive upward in Bevy's default 2D coordinate system
        let y = marker.line as f32 * metrics.cell_height;

        // Create circular gutter indicator
        let mesh = meshes.add(Circle::new(marker_radius));
        let color = marker_color(marker.marker_type, marker.exit_code);
        let material = materials.add(ColorMaterial::from(color));

        // Spawn marker entity
        commands.spawn((
            PromptGutterMarker {
                line: marker.line,
                marker_type: marker.marker_type,
            },
            Mesh2d(mesh),
            MeshMaterial2d(material),
            Transform::from_xyz(
                -gutter_width / 2.0,             // Left of grid
                -y - metrics.cell_height / 2.0,  // Center on line (Y-down to Y-up conversion)
                50.0,                             // Z-index: above background, below overlays
            ),
        ));
    }
}

/// System to handle prompt navigation via keyboard
///
/// Keybindings:
/// - Ctrl+Up: Jump to previous prompt
/// - Ctrl+Down: Jump to next prompt
///
/// TODO: This system currently sets target_scroll_line but doesn't actually
/// scroll the view. Integration with scrollback system is needed.
pub fn prompt_navigation(
    keys: Res<ButtonInput<KeyCode>>,
    mut markers: ResMut<PromptMarkers>,
    // TODO: Add scroll control resource to actually perform scrolling
    // scroll_state: Option<ResMut<ScrollbackState>>,
) {
    // Check for Ctrl modifier
    let ctrl = keys.pressed(KeyCode::ControlLeft) || keys.pressed(KeyCode::ControlRight);

    if ctrl && keys.just_pressed(KeyCode::ArrowUp) {
        // Jump to previous prompt
        // TODO: Get current scroll position from scrollback system
        let current_line = 0; // Placeholder - should be actual viewport top line

        if let Some(idx) = markers.previous_prompt(current_line) {
            markers.current_index = Some(idx);
            // Get the line number before mutating markers
            if let Some(marker) = markers.markers.get(idx) {
                let line = marker.line;
                markers.target_scroll_line = Some(line);
                // TODO: Actually scroll to marker.line
                // if let Some(mut scroll) = scroll_state {
                //     scroll.scroll_to_line(line);
                // }
                println!("Navigate to previous prompt at line {}", line);
            }
        }
    }

    if ctrl && keys.just_pressed(KeyCode::ArrowDown) {
        // Jump to next prompt
        // TODO: Get current scroll position from scrollback system
        let current_line = 0; // Placeholder - should be actual viewport top line

        if let Some(idx) = markers.next_prompt(current_line) {
            markers.current_index = Some(idx);
            // Get the line number before mutating markers
            if let Some(marker) = markers.markers.get(idx) {
                let line = marker.line;
                markers.target_scroll_line = Some(line);
                // TODO: Actually scroll to marker.line
                // if let Some(mut scroll) = scroll_state {
                //     scroll.scroll_to_line(line);
                // }
                println!("Navigate to next prompt at line {}", line);
            }
        }
    }
}

/// System to receive prompt marker updates from the daemon
///
/// Listens for PromptMarkersUpdate messages and updates the PromptMarkers resource
pub fn receive_prompt_markers(
    mut events: EventReader<RemoteMessageEvent>,
    mut markers: ResMut<PromptMarkers>,
) {
    for event in events.read() {
        if let scarab_protocol::DaemonMessage::PromptMarkersUpdate { markers: new_markers } = &event.0 {
            println!("Received {} prompt markers from daemon", new_markers.len());
            markers.update_markers(new_markers.clone());
        }
    }
}

/// Plugin for prompt marker functionality
///
/// Adds:
/// - PromptMarkers resource for tracking markers
/// - Gutter rendering system
/// - Keyboard navigation system
/// - IPC message receiver system
pub struct PromptMarkersPlugin;

impl Plugin for PromptMarkersPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PromptMarkers>()
            .add_systems(
                Update,
                (
                    receive_prompt_markers,
                    render_gutter_markers,
                    prompt_navigation,
                )
                    .chain(), // Run in order: receive -> render -> navigate
            );
    }
}
