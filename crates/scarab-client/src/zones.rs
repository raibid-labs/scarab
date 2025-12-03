//! Client-side semantic zone handling for deep shell integration
//!
//! This module handles:
//! - Receiving zone updates from the daemon
//! - Rendering zone indicators (duration, exit status)
//! - Zone-aware text selection
//! - Copy last output functionality

use bevy::prelude::*;
use bevy::input::keyboard::KeyCode;
use bevy::input::mouse::MouseButton;
use bevy::input::ButtonInput;
use scarab_protocol::{SemanticZone, CommandBlock, ZoneType, DaemonMessage};
use crate::ipc::RemoteMessageEvent;
use crate::terminal::scrollback::ScrollbackBuffer;

/// Resource storing semantic zones from the daemon
#[derive(Resource, Default)]
pub struct SemanticZones {
    /// All tracked zones
    pub zones: Vec<SemanticZone>,
    /// Completed command blocks
    pub command_blocks: Vec<CommandBlock>,
    /// Currently selected zone (for zone-aware operations)
    pub selected_zone_id: Option<u64>,
}

impl SemanticZones {
    /// Find a zone by ID
    pub fn find_zone(&self, id: u64) -> Option<&SemanticZone> {
        self.zones.iter().find(|z| z.id == id)
    }

    /// Find a command block by ID
    pub fn find_block(&self, id: u64) -> Option<&CommandBlock> {
        self.command_blocks.iter().find(|b| b.id == id)
    }

    /// Find the zone containing a specific line
    pub fn find_zone_at_line(&self, line: u32) -> Option<&SemanticZone> {
        self.zones.iter()
            .rev()
            .find(|zone| zone.contains_line(line))
    }

    /// Find the command block containing a specific line
    pub fn find_block_at_line(&self, line: u32) -> Option<&CommandBlock> {
        self.command_blocks.iter()
            .rev()
            .find(|block| block.contains_line(line))
    }

    /// Get the last output zone for copy operations
    pub fn last_output_zone(&self) -> Option<&SemanticZone> {
        self.zones.iter()
            .rev()
            .find(|z| z.zone_type == ZoneType::Output && z.is_complete)
    }

    /// Get all output zones (for displaying in UI)
    pub fn output_zones(&self) -> impl Iterator<Item = &SemanticZone> {
        self.zones.iter()
            .filter(|z| z.zone_type == ZoneType::Output)
    }

    /// Get recently completed command blocks (for UI display)
    pub fn recent_blocks(&self, count: usize) -> &[CommandBlock] {
        let start = self.command_blocks.len().saturating_sub(count);
        &self.command_blocks[start..]
    }
}

/// Component marking zone indicator entities in the gutter
#[derive(Component)]
pub struct ZoneIndicator {
    pub zone_id: u64,
    pub zone_type: ZoneType,
}

/// Event fired when requesting to copy the last command output
#[derive(Event, Debug, Clone)]
pub struct CopyLastOutputEvent;

/// Event fired when a zone is selected
#[derive(Event, Debug, Clone)]
pub struct SelectZoneEvent {
    pub zone_id: u64,
}

/// System to receive zone updates from the daemon
pub fn receive_zone_updates(
    mut events: EventReader<RemoteMessageEvent>,
    mut zones: ResMut<SemanticZones>,
) {
    for event in events.read() {
        match &event.0 {
            DaemonMessage::SemanticZonesUpdate { zones: new_zones } => {
                println!("Received {} semantic zones from daemon", new_zones.len());
                zones.zones = new_zones.clone();
            }
            DaemonMessage::CommandBlocksUpdate { blocks } => {
                println!("Received {} command blocks from daemon", blocks.len());
                zones.command_blocks = blocks.clone();
            }
            _ => {}
        }
    }
}

/// System to render zone indicators in the gutter
///
/// Shows:
/// - Exit status markers (green checkmark for success, red X for failure)
/// - Duration indicators for long-running commands
///
/// NOTE: Full rendering implementation pending - currently logs to console
pub fn render_zone_indicators(
    zones: Res<SemanticZones>,
    metrics: Res<scarab_protocol::TerminalMetrics>,
) {
    // Only update if zones changed
    if !zones.is_changed() {
        return;
    }

    // Log indicators for output zones with exit codes
    for zone in zones.output_zones() {
        if !zone.is_complete {
            continue;
        }

        let status = if zone.is_success() {
            "✓ SUCCESS"
        } else if zone.is_failure() {
            format!("✗ FAILED (exit code: {})", zone.exit_code.unwrap_or(-1))
        } else {
            "? UNKNOWN".to_string()
        };

        println!(
            "Zone indicator: line {} - {}",
            zone.start_row,
            status
        );
    }

    // Log duration labels for long-running commands
    for block in zones.command_blocks.iter().rev().take(20) {
        if let Some(duration_secs) = block.duration_secs() {
            if duration_secs >= 1.0 {
                println!(
                    "Command duration: line {} - {}",
                    block.start_row,
                    format_duration(duration_secs)
                );
            }
        }
    }
}

/// Format duration for display
fn format_duration(seconds: f64) -> String {
    if seconds < 60.0 {
        format!("{:.1}s", seconds)
    } else if seconds < 3600.0 {
        let mins = (seconds / 60.0).floor();
        let secs = seconds % 60.0;
        format!("{}m {:.0}s", mins, secs)
    } else {
        let hours = (seconds / 3600.0).floor();
        let mins = ((seconds % 3600.0) / 60.0).floor();
        format!("{}h {}m", hours, mins)
    }
}

/// System to handle copy last output keybinding
///
/// Keybinding: Ctrl+Shift+Y (copy last output)
pub fn handle_copy_last_output(
    keys: Res<ButtonInput<KeyCode>>,
    zones: Res<SemanticZones>,
    scrollback: Res<ScrollbackBuffer>,
    mut copy_events: EventWriter<CopyLastOutputEvent>,
) {
    let ctrl = keys.pressed(KeyCode::ControlLeft) || keys.pressed(KeyCode::ControlRight);
    let shift = keys.pressed(KeyCode::ShiftLeft) || keys.pressed(KeyCode::ShiftRight);

    if ctrl && shift && keys.just_pressed(KeyCode::KeyY) {
        // Find the last output zone
        if let Some(output_zone) = zones.last_output_zone() {
            println!(
                "Copy last output: lines {}-{} (zone ID: {})",
                output_zone.start_row, output_zone.end_row, output_zone.id
            );

            // Extract text from the output zone
            let output_text = extract_zone_text(output_zone, &scrollback);

            // Copy to clipboard
            if let Ok(mut ctx) = arboard::Clipboard::new() {
                if let Err(e) = ctx.set_text(output_text) {
                    eprintln!("Failed to copy to clipboard: {}", e);
                } else {
                    println!("Copied last output to clipboard");
                    copy_events.send(CopyLastOutputEvent);
                }
            }
        } else {
            println!("No output zone found to copy");
        }
    }
}

/// Extract text from a zone
fn extract_zone_text(zone: &SemanticZone, scrollback: &ScrollbackBuffer) -> String {
    let mut lines = Vec::new();

    for line_num in zone.start_row..=zone.end_row {
        if let Some(line) = scrollback.get_line(line_num as usize) {
            lines.push(line.to_string());
        }
    }

    lines.join("\n")
}

/// System to handle click-to-select-zone
///
/// When clicking in a zone, select that entire zone for copying
pub fn handle_zone_selection(
    mouse_button: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    zones: Res<SemanticZones>,
    mut select_events: EventWriter<SelectZoneEvent>,
    metrics: Res<scarab_protocol::TerminalMetrics>,
) {
    if mouse_button.just_pressed(MouseButton::Left) {
        // Get cursor position
        if let Ok(window) = windows.get_single() {
            if let Some(cursor_pos) = window.cursor_position() {
                // Convert screen coordinates to grid coordinates
                let (col, row) = metrics.screen_to_grid(cursor_pos.x, cursor_pos.y);

                // Find zone at this position
                if let Some(zone) = zones.find_zone_at_line(row as u32) {
                    println!("Clicked in zone {} (type: {:?})", zone.id, zone.zone_type);
                    select_events.send(SelectZoneEvent { zone_id: zone.id });
                }
            }
        }
    }
}

/// System to highlight selected zone
///
/// Logs selected zone info (full rendering implementation pending)
pub fn highlight_selected_zone(
    zones: Res<SemanticZones>,
    mut select_events: EventReader<SelectZoneEvent>,
) {
    for event in select_events.read() {
        if let Some(zone) = zones.find_zone(event.zone_id) {
            println!(
                "Selected zone: ID={}, type={:?}, lines={}-{}",
                zone.id, zone.zone_type, zone.start_row, zone.end_row
            );
        }
    }
}

/// Plugin for semantic zones functionality
pub struct SemanticZonesPlugin;

impl Plugin for SemanticZonesPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<SemanticZones>()
            .add_event::<CopyLastOutputEvent>()
            .add_event::<SelectZoneEvent>()
            .add_systems(
                Update,
                (
                    receive_zone_updates,
                    render_zone_indicators,
                    handle_copy_last_output,
                    handle_zone_selection,
                    highlight_selected_zone,
                ).chain(),
            );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_duration() {
        assert_eq!(format_duration(1.5), "1.5s");
        assert_eq!(format_duration(65.0), "1m 5s");
        assert_eq!(format_duration(3725.0), "1h 2m");
    }

    #[test]
    fn test_find_zone_at_line() {
        let mut zones = SemanticZones::default();

        zones.zones.push(SemanticZone {
            id: 1,
            zone_type: ZoneType::Output,
            start_row: 10,
            end_row: 20,
            command: None,
            exit_code: Some(0),
            started_at: 1000,
            duration_micros: Some(5000),
            is_complete: true,
        });

        assert!(zones.find_zone_at_line(15).is_some());
        assert_eq!(zones.find_zone_at_line(15).unwrap().id, 1);
        assert!(zones.find_zone_at_line(5).is_none());
        assert!(zones.find_zone_at_line(25).is_none());
    }

    #[test]
    fn test_last_output_zone() {
        let mut zones = SemanticZones::default();

        // Add multiple zones
        zones.zones.push(SemanticZone {
            id: 1,
            zone_type: ZoneType::Output,
            start_row: 10,
            end_row: 20,
            command: None,
            exit_code: Some(0),
            started_at: 1000,
            duration_micros: Some(5000),
            is_complete: true,
        });

        zones.zones.push(SemanticZone {
            id: 2,
            zone_type: ZoneType::Output,
            start_row: 25,
            end_row: 30,
            command: None,
            exit_code: Some(1),
            started_at: 2000,
            duration_micros: Some(3000),
            is_complete: true,
        });

        let last = zones.last_output_zone().unwrap();
        assert_eq!(last.id, 2);
        assert_eq!(last.start_row, 25);
    }
}
