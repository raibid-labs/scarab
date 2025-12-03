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
use crate::ui::link_hints::LinkHintsState;

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

    /// Get the current prompt zone bounds (start line to end line)
    ///
    /// Returns the line range of the current prompt block, from the last
    /// PromptStart marker to either the next PromptStart or end of buffer.
    pub fn current_prompt_zone(&self, current_line: u32) -> Option<(u32, u32)> {
        // Find the last prompt start before or at current line
        let start_idx = self.markers
            .iter()
            .enumerate()
            .rev()
            .find(|(_, m)| m.line <= current_line && m.is_prompt_start())
            .map(|(i, _)| i)?;

        let start_line = self.markers[start_idx].line;

        // Find the next prompt start after current position (if any)
        let end_line = self.markers
            .iter()
            .skip(start_idx + 1)
            .find(|m| m.is_prompt_start())
            .map(|m| m.line)
            .unwrap_or(u32::MAX); // If no next prompt, zone extends to end

        Some((start_line, end_line))
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

/// Navigation anchor types for prompt-based navigation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PromptAnchorType {
    /// Start of a new prompt (OSC 133 A marker)
    PromptStart,
    /// End of command execution (OSC 133 D marker with exit code)
    CommandFinished,
    /// Output region (between prompt end and command finished)
    CommandOutput,
}

/// Navigation anchor component for prompt markers
///
/// Attached to entities to mark them as navigation targets for the
/// navigation system. These anchors enable:
/// - Jump-to-prompt navigation (Ctrl+Up/Down)
/// - Semantic zone filtering for hint mode
/// - Command output region selection
#[derive(Component, Debug, Clone)]
pub struct NavAnchor {
    /// Type of navigation anchor
    pub anchor_type: PromptAnchorType,
    /// Terminal line number
    pub line: u32,
    /// Command text for semantic zones (if available)
    pub command_text: Option<String>,
    /// Exit code for CommandFinished anchors
    pub exit_code: Option<i32>,
}

/// Event fired when navigating to a prompt anchor
///
/// This event is triggered when the user navigates to a prompt marker
/// via keyboard shortcuts or other navigation actions.
#[derive(Event, Debug, Clone)]
pub struct JumpToPromptEvent {
    /// Target line to jump to
    pub target_line: u32,
    /// Anchor being jumped to
    pub anchor_type: PromptAnchorType,
}

/// Event fired when a prompt zone becomes the active filter scope
///
/// This event signals that hint mode or other navigation features
/// should filter focusable elements to only those within the
/// specified prompt block.
#[derive(Event, Debug, Clone)]
pub struct PromptZoneFocusedEvent {
    /// Start line of the focused prompt zone
    pub start_line: u32,
    /// End line of the focused prompt zone (exclusive)
    pub end_line: u32,
    /// Command text if available
    pub command_text: Option<String>,
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

/// System to spawn navigation anchor entities from prompt markers
///
/// This system creates NavAnchor entities for each prompt marker, enabling
/// integration with the broader navigation system. NavAnchors are:
/// - Queryable by navigation systems for jump-to-prompt
/// - Used for semantic zone filtering in hint mode
/// - Tagged with command metadata for context-aware navigation
pub fn spawn_nav_anchors(
    mut commands: Commands,
    markers: Res<PromptMarkers>,
    existing_anchors: Query<Entity, With<NavAnchor>>,
) {
    // Only update if markers changed
    if !markers.is_changed() {
        return;
    }

    // Despawn all existing nav anchors
    for entity in existing_anchors.iter() {
        commands.entity(entity).despawn();
    }

    // Spawn new nav anchors for each marker
    for marker in &markers.markers {
        let anchor_type = match marker.marker_type {
            0 => PromptAnchorType::PromptStart,      // OSC 133 A
            3 => PromptAnchorType::CommandFinished,  // OSC 133 D
            _ => continue, // Skip other marker types for now
        };

        // Create NavAnchor entity
        // Note: command_text would need to be extracted from terminal buffer
        // if needed, as it's not part of the marker protocol data
        commands.spawn(NavAnchor {
            anchor_type,
            line: marker.line,
            command_text: None, // TODO: Extract from terminal buffer if needed
            exit_code: marker.exit_code,
        });
    }
}

/// System to handle prompt navigation via keyboard
///
/// Keybindings:
/// - Ctrl+Up: Jump to previous prompt
/// - Ctrl+Down: Jump to next prompt
///
/// This system now emits JumpToPromptEvent for integration with scrollback
/// and other navigation systems.
pub fn prompt_navigation(
    keys: Res<ButtonInput<KeyCode>>,
    mut markers: ResMut<PromptMarkers>,
    mut jump_events: EventWriter<JumpToPromptEvent>,
    scrollback: Res<crate::terminal::scrollback::ScrollbackBuffer>,
) {
    // Check for Ctrl modifier
    let ctrl = keys.pressed(KeyCode::ControlLeft) || keys.pressed(KeyCode::ControlRight);

    if ctrl && keys.just_pressed(KeyCode::ArrowUp) {
        // Jump to previous prompt
        // Get current scroll position from scrollback buffer
        // The scroll_offset represents how many lines we've scrolled up from bottom
        // We need to convert this to a line number in the buffer
        let total_lines = scrollback.line_count() as u32;
        let scroll_offset = scrollback.scroll_offset() as u32;
        let current_line = total_lines.saturating_sub(scroll_offset);

        if let Some(idx) = markers.previous_prompt(current_line) {
            markers.current_index = Some(idx);
            // Get the line number before mutating markers
            if let Some(marker) = markers.markers.get(idx) {
                let line = marker.line;
                markers.target_scroll_line = Some(line);

                // Emit navigation event
                jump_events.send(JumpToPromptEvent {
                    target_line: line,
                    anchor_type: PromptAnchorType::PromptStart,
                });

                println!("Navigate to previous prompt at line {}", line);
            }
        }
    }

    if ctrl && keys.just_pressed(KeyCode::ArrowDown) {
        // Jump to next prompt
        // Get current scroll position from scrollback buffer
        let total_lines = scrollback.line_count() as u32;
        let scroll_offset = scrollback.scroll_offset() as u32;
        let current_line = total_lines.saturating_sub(scroll_offset);

        if let Some(idx) = markers.next_prompt(current_line) {
            markers.current_index = Some(idx);
            // Get the line number before mutating markers
            if let Some(marker) = markers.markers.get(idx) {
                let line = marker.line;
                markers.target_scroll_line = Some(line);

                // Emit navigation event
                jump_events.send(JumpToPromptEvent {
                    target_line: line,
                    anchor_type: PromptAnchorType::PromptStart,
                });

                println!("Navigate to next prompt at line {}", line);
            }
        }
    }
}

/// System to handle jump-to-prompt events and update scrollback position
///
/// This system listens for JumpToPromptEvent and scrolls the viewport to
/// make the target line visible. The target line is centered in the viewport
/// when possible.
pub fn handle_jump_to_prompt(
    mut jump_events: EventReader<JumpToPromptEvent>,
    mut scrollback: ResMut<crate::terminal::scrollback::ScrollbackBuffer>,
    mut scroll_state: ResMut<crate::terminal::scrollback::ScrollbackState>,
) {
    for event in jump_events.read() {
        let target_line = event.target_line as usize;
        let total_lines = scrollback.line_count();

        // Calculate scroll offset to center the target line in viewport
        // scroll_offset is how many lines we've scrolled up from bottom
        // If target_line is near the top of the buffer, we scroll to maximum
        // If target_line is near the bottom, we scroll less

        // We want to position target_line at ~1/3 from top of viewport for better context
        let viewport_offset = scroll_state.lines_per_page / 3;

        if target_line >= total_lines {
            // Target is beyond the buffer, scroll to top
            scrollback.scroll_to_top();
        } else {
            // Calculate how far from bottom the target line is
            let lines_from_bottom = total_lines.saturating_sub(target_line);

            // Add offset to show context above the target
            let desired_offset = lines_from_bottom.saturating_add(viewport_offset);

            // Scroll to position (clamp to valid range)
            let max_scroll = total_lines;
            let scroll_offset = desired_offset.min(max_scroll);

            // Set the scroll position by first going to bottom, then scrolling up
            scrollback.scroll_to_bottom();
            scrollback.scroll_up(scroll_offset);
        }

        // Update scroll state
        scroll_state.is_scrolled = !scrollback.is_at_bottom();

        println!(
            "Jumped to {:?} at line {} (offset: {})",
            event.anchor_type,
            event.target_line,
            scrollback.scroll_offset()
        );
    }
}

/// System to filter focusable elements to current prompt zone when hint mode is active
///
/// When hint mode is activated, this system:
/// 1. Determines the current prompt zone boundaries
/// 2. Emits PromptZoneFocusedEvent to notify navigation systems
/// 3. Allows downstream systems to filter hints/focusables to this zone
///
/// This enables context-aware navigation where only links/items in the current
/// command output are targetable, reducing visual clutter and improving UX.
pub fn prompt_zone_filtering(
    hints_state: Res<LinkHintsState>,
    markers: Res<PromptMarkers>,
    scrollback: Res<crate::terminal::scrollback::ScrollbackBuffer>,
    mut zone_events: EventWriter<PromptZoneFocusedEvent>,
) {
    // Only filter when hint mode becomes active
    if !hints_state.is_changed() || !hints_state.active {
        return;
    }

    // Get actual current viewport line from scrollback
    let total_lines = scrollback.line_count() as u32;
    let scroll_offset = scrollback.scroll_offset() as u32;
    let current_line = total_lines.saturating_sub(scroll_offset);

    // Get the current prompt zone bounds
    if let Some((start_line, end_line)) = markers.current_prompt_zone(current_line) {
        // TODO: Extract command text from terminal buffer if needed
        // Command text is not stored in PromptMarkerInfo protocol data
        let command_text = None;

        // Emit zone focused event for downstream systems to filter
        zone_events.send(PromptZoneFocusedEvent {
            start_line,
            end_line,
            command_text,
        });

        println!(
            "Filtering hints to prompt zone: lines {}-{}",
            start_line, end_line
        );
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
/// - JumpToPromptEvent and PromptZoneFocusedEvent for navigation integration
/// - Gutter rendering system
/// - NavAnchor spawning system
/// - Keyboard navigation system with event emission
/// - Jump-to-prompt scrollback handler system
/// - Prompt zone filtering system for hint mode
/// - IPC message receiver system
pub struct PromptMarkersPlugin;

impl Plugin for PromptMarkersPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PromptMarkers>()
            .add_event::<JumpToPromptEvent>()
            .add_event::<PromptZoneFocusedEvent>()
            .add_systems(
                Update,
                (
                    receive_prompt_markers,
                    render_gutter_markers,
                    spawn_nav_anchors,
                    prompt_navigation,
                    handle_jump_to_prompt, // New: Handle jump events and scroll viewport
                    prompt_zone_filtering,
                )
                    .chain(), // Run in order: receive -> render -> spawn anchors -> navigate -> jump -> filter
            );
    }
}
