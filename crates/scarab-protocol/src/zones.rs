//! Semantic zones for deep shell integration
//!
//! This module defines semantic zones that represent different regions of the
//! terminal output based on OSC 133 shell integration markers:
//! - Prompt zones: The shell prompt area
//! - Input zones: User command input
//! - Output zones: Command output
//!
//! These zones enable:
//! - Zone-aware text selection (e.g., select only command output)
//! - Command duration tracking
//! - Exit code display
//! - Output extraction for copy/paste

extern crate alloc;
use alloc::string::String;
use alloc::vec::Vec;

/// Type of semantic zone in the terminal
#[derive(Debug, Clone, Copy, PartialEq, Eq, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
#[archive(check_bytes)]
pub enum ZoneType {
    /// Shell prompt area (between OSC 133;A and 133;B)
    Prompt,
    /// User command input (between OSC 133;B and 133;C)
    Input,
    /// Command output (between OSC 133;C and 133;D)
    Output,
}

/// A semantic zone representing a region between shell integration markers
///
/// Zones are created by correlating OSC 133 markers:
/// - Prompt: From A marker to B marker
/// - Input: From B marker to C marker
/// - Output: From C marker to D marker
#[derive(Debug, Clone, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
#[archive(check_bytes)]
pub struct SemanticZone {
    /// Unique identifier for this zone
    pub id: u64,
    /// Type of zone
    pub zone_type: ZoneType,
    /// Starting line number (absolute, including scrollback)
    pub start_row: u32,
    /// Ending line number (absolute, including scrollback)
    /// For incomplete zones (e.g., still outputting), this may equal start_row
    pub end_row: u32,
    /// Command text extracted from the input zone (if available)
    pub command: Option<String>,
    /// Exit code from OSC 133;D marker (only for completed output zones)
    pub exit_code: Option<i32>,
    /// Timestamp when the zone started (microseconds since UNIX epoch)
    pub started_at: u64,
    /// Duration in microseconds (only for completed zones)
    pub duration_micros: Option<u64>,
    /// Whether this zone is complete (has an end marker)
    pub is_complete: bool,
}

impl SemanticZone {
    /// Create a new prompt zone
    pub fn new_prompt(id: u64, start_row: u32, timestamp: u64) -> Self {
        Self {
            id,
            zone_type: ZoneType::Prompt,
            start_row,
            end_row: start_row,
            command: None,
            exit_code: None,
            started_at: timestamp,
            duration_micros: None,
            is_complete: false,
        }
    }

    /// Create a new input zone
    pub fn new_input(id: u64, start_row: u32, timestamp: u64) -> Self {
        Self {
            id,
            zone_type: ZoneType::Input,
            start_row,
            end_row: start_row,
            command: None,
            exit_code: None,
            started_at: timestamp,
            duration_micros: None,
            is_complete: false,
        }
    }

    /// Create a new output zone
    pub fn new_output(id: u64, start_row: u32, timestamp: u64) -> Self {
        Self {
            id,
            zone_type: ZoneType::Output,
            start_row,
            end_row: start_row,
            command: None,
            exit_code: None,
            started_at: timestamp,
            duration_micros: None,
            is_complete: false,
        }
    }

    /// Mark this zone as complete with an ending row and timestamp
    pub fn complete(&mut self, end_row: u32, end_timestamp: u64) {
        self.end_row = end_row;
        self.is_complete = true;
        if end_timestamp >= self.started_at {
            self.duration_micros = Some(end_timestamp - self.started_at);
        }
    }

    /// Set the command text for this zone (typically for input zones)
    pub fn set_command(&mut self, command: String) {
        self.command = Some(command);
    }

    /// Set the exit code (only for output zones)
    pub fn set_exit_code(&mut self, exit_code: i32) {
        self.exit_code = Some(exit_code);
    }

    /// Check if this zone contains the given line number
    pub fn contains_line(&self, line: u32) -> bool {
        line >= self.start_row && line <= self.end_row
    }

    /// Get the number of lines in this zone
    pub fn line_count(&self) -> u32 {
        if self.end_row >= self.start_row {
            self.end_row - self.start_row + 1
        } else {
            1
        }
    }

    /// Check if this zone represents a successful command (exit code 0)
    pub fn is_success(&self) -> bool {
        self.exit_code == Some(0)
    }

    /// Check if this zone represents a failed command (non-zero exit code)
    pub fn is_failure(&self) -> bool {
        matches!(self.exit_code, Some(code) if code != 0)
    }

    /// Get duration in milliseconds for display
    pub fn duration_millis(&self) -> Option<u64> {
        self.duration_micros.map(|micros| micros / 1000)
    }

    /// Get duration in seconds for display
    pub fn duration_secs(&self) -> Option<f64> {
        self.duration_micros
            .map(|micros| micros as f64 / 1_000_000.0)
    }
}

/// A command block represents a complete prompt-input-output sequence
///
/// This is a higher-level abstraction that groups related zones together
/// for easier reasoning about commands and their results.
#[derive(Debug, Clone, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
#[archive(check_bytes)]
pub struct CommandBlock {
    /// Unique identifier
    pub id: u64,
    /// Prompt zone
    pub prompt_zone: Option<SemanticZone>,
    /// Input zone
    pub input_zone: Option<SemanticZone>,
    /// Output zone
    pub output_zone: Option<SemanticZone>,
    /// Starting line of the entire block
    pub start_row: u32,
    /// Ending line of the entire block
    pub end_row: u32,
    /// Timestamp when the command started
    pub started_at: u64,
    /// Total duration from prompt to command completion
    pub duration_micros: Option<u64>,
}

impl CommandBlock {
    /// Create a new command block starting with a prompt zone
    pub fn new(id: u64, prompt_zone: SemanticZone) -> Self {
        Self {
            id,
            start_row: prompt_zone.start_row,
            end_row: prompt_zone.end_row,
            started_at: prompt_zone.started_at,
            prompt_zone: Some(prompt_zone),
            input_zone: None,
            output_zone: None,
            duration_micros: None,
        }
    }

    /// Add an input zone to this command block
    pub fn add_input_zone(&mut self, zone: SemanticZone) {
        self.end_row = zone.end_row.max(self.end_row);
        self.input_zone = Some(zone);
    }

    /// Add an output zone to this command block
    pub fn add_output_zone(&mut self, zone: SemanticZone) {
        self.end_row = zone.end_row.max(self.end_row);

        // Calculate total duration if output zone is complete
        if zone.is_complete {
            let end_timestamp = zone.started_at + zone.duration_micros.unwrap_or(0);
            if end_timestamp >= self.started_at {
                self.duration_micros = Some(end_timestamp - self.started_at);
            }
        }

        self.output_zone = Some(zone);
    }

    /// Get the command text from the input zone
    pub fn command_text(&self) -> Option<&str> {
        self.input_zone.as_ref()?.command.as_deref()
    }

    /// Get the exit code from the output zone
    pub fn exit_code(&self) -> Option<i32> {
        self.output_zone.as_ref()?.exit_code
    }

    /// Check if this command block is complete (has all zones)
    pub fn is_complete(&self) -> bool {
        self.output_zone.as_ref().map_or(false, |z| z.is_complete)
    }

    /// Check if this command was successful
    pub fn is_success(&self) -> bool {
        self.exit_code() == Some(0)
    }

    /// Check if this command failed
    pub fn is_failure(&self) -> bool {
        matches!(self.exit_code(), Some(code) if code != 0)
    }

    /// Get the output zone bounds for text extraction
    pub fn output_bounds(&self) -> Option<(u32, u32)> {
        self.output_zone.as_ref().map(|z| (z.start_row, z.end_row))
    }

    /// Check if this block contains the given line
    pub fn contains_line(&self, line: u32) -> bool {
        line >= self.start_row && line <= self.end_row
    }

    /// Get duration in seconds for display
    pub fn duration_secs(&self) -> Option<f64> {
        self.duration_micros
            .map(|micros| micros as f64 / 1_000_000.0)
    }
}

/// Zone tracking state for managing semantic zones
///
/// This is not directly serialized for IPC, but used internally by the daemon
/// to track zones and generate SemanticZone messages for the client.
#[derive(Debug, Clone)]
pub struct ZoneTracker {
    /// Next zone ID to assign
    next_zone_id: u64,
    /// Current incomplete zones being tracked
    current_zones: Vec<SemanticZone>,
    /// Completed command blocks (limited to recent history)
    command_blocks: Vec<CommandBlock>,
    /// Maximum command blocks to retain
    max_blocks: usize,
    /// Current command block being built
    current_block: Option<CommandBlock>,
}

impl ZoneTracker {
    /// Create a new zone tracker
    pub fn new(max_blocks: usize) -> Self {
        Self {
            next_zone_id: 1,
            current_zones: Vec::new(),
            command_blocks: Vec::new(),
            max_blocks,
            current_block: None,
        }
    }

    /// Allocate a new zone ID
    fn next_id(&mut self) -> u64 {
        let id = self.next_zone_id;
        self.next_zone_id = self.next_zone_id.wrapping_add(1);
        id
    }

    /// Handle OSC 133;A - Prompt start
    pub fn mark_prompt_start(&mut self, line: u32, timestamp: u64) {
        let id = self.next_id();
        let zone = SemanticZone::new_prompt(id, line, timestamp);

        // Start a new command block
        self.current_block = Some(CommandBlock::new(id, zone.clone()));
        self.current_zones.push(zone);
    }

    /// Handle OSC 133;B - Command/input start
    pub fn mark_command_start(&mut self, line: u32, timestamp: u64) {
        // Complete the previous prompt zone if any
        if let Some(zone) = self
            .current_zones
            .iter_mut()
            .rev()
            .find(|z| z.zone_type == ZoneType::Prompt && !z.is_complete)
        {
            zone.complete(line.saturating_sub(1), timestamp);
        }

        // Create new input zone
        let id = self.next_id();
        let zone = SemanticZone::new_input(id, line, timestamp);

        // Add to current block
        if let Some(ref mut block) = self.current_block {
            block.add_input_zone(zone.clone());
        }

        self.current_zones.push(zone);
    }

    /// Handle OSC 133;C - Command executed, output begins
    pub fn mark_command_executed(&mut self, line: u32, timestamp: u64) {
        // Complete the previous input zone if any
        if let Some(zone) = self
            .current_zones
            .iter_mut()
            .rev()
            .find(|z| z.zone_type == ZoneType::Input && !z.is_complete)
        {
            zone.complete(line.saturating_sub(1), timestamp);
        }

        // Create new output zone
        let id = self.next_id();
        let zone = SemanticZone::new_output(id, line, timestamp);

        // Add to current block
        if let Some(ref mut block) = self.current_block {
            block.add_output_zone(zone.clone());
        }

        self.current_zones.push(zone);
    }

    /// Handle OSC 133;D - Command finished
    pub fn mark_command_finished(&mut self, line: u32, exit_code: i32, timestamp: u64) {
        // Complete the previous output zone if any
        if let Some(zone) = self
            .current_zones
            .iter_mut()
            .rev()
            .find(|z| z.zone_type == ZoneType::Output && !z.is_complete)
        {
            zone.complete(line, timestamp);
            zone.set_exit_code(exit_code);

            // Update the current block's output zone
            if let Some(ref mut block) = self.current_block {
                block.add_output_zone(zone.clone());

                // Move completed block to history
                self.command_blocks.push(block.clone());

                // Trim old blocks
                if self.command_blocks.len() > self.max_blocks {
                    self.command_blocks.remove(0);
                }
            }
        }

        // Clear current block
        self.current_block = None;
    }

    /// Set command text for the most recent input zone
    pub fn set_command_text(&mut self, command: String) {
        if let Some(zone) = self
            .current_zones
            .iter_mut()
            .rev()
            .find(|z| z.zone_type == ZoneType::Input)
        {
            zone.set_command(command.clone());
        }

        // Also update the current block
        if let Some(ref mut block) = self.current_block {
            if let Some(ref mut input_zone) = block.input_zone {
                input_zone.set_command(command);
            }
        }
    }

    /// Get all tracked zones
    pub fn zones(&self) -> &[SemanticZone] {
        &self.current_zones
    }

    /// Get all completed command blocks
    pub fn command_blocks(&self) -> &[CommandBlock] {
        &self.command_blocks
    }

    /// Get the current incomplete command block
    pub fn current_block(&self) -> Option<&CommandBlock> {
        self.current_block.as_ref()
    }

    /// Find the command block containing the given line
    pub fn find_block_at_line(&self, line: u32) -> Option<&CommandBlock> {
        self.command_blocks
            .iter()
            .rev()
            .find(|block| block.contains_line(line))
    }

    /// Find the zone containing the given line
    pub fn find_zone_at_line(&self, line: u32) -> Option<&SemanticZone> {
        self.current_zones
            .iter()
            .rev()
            .find(|zone| zone.contains_line(line))
    }

    /// Get the last output zone for "copy last output" functionality
    pub fn last_output_zone(&self) -> Option<&SemanticZone> {
        self.command_blocks
            .iter()
            .rev()
            .find_map(|block| block.output_zone.as_ref())
    }

    /// Clear all zones (useful for reset operations)
    pub fn clear(&mut self) {
        self.current_zones.clear();
        self.command_blocks.clear();
        self.current_block = None;
    }

    /// Update zone line numbers after scrolling
    ///
    /// When the terminal scrolls, line numbers in scrollback increase.
    /// This method adjusts zone line numbers accordingly.
    pub fn adjust_for_scroll(&mut self, lines_scrolled: i32) {
        if lines_scrolled == 0 {
            return;
        }

        let adjust = |row: u32, delta: i32| -> u32 {
            if delta < 0 {
                row.saturating_sub(delta.abs() as u32)
            } else {
                row.saturating_add(delta as u32)
            }
        };

        // Adjust current zones
        for zone in &mut self.current_zones {
            zone.start_row = adjust(zone.start_row, lines_scrolled);
            zone.end_row = adjust(zone.end_row, lines_scrolled);
        }

        // Adjust command blocks
        for block in &mut self.command_blocks {
            block.start_row = adjust(block.start_row, lines_scrolled);
            block.end_row = adjust(block.end_row, lines_scrolled);

            if let Some(ref mut zone) = block.prompt_zone {
                zone.start_row = adjust(zone.start_row, lines_scrolled);
                zone.end_row = adjust(zone.end_row, lines_scrolled);
            }
            if let Some(ref mut zone) = block.input_zone {
                zone.start_row = adjust(zone.start_row, lines_scrolled);
                zone.end_row = adjust(zone.end_row, lines_scrolled);
            }
            if let Some(ref mut zone) = block.output_zone {
                zone.start_row = adjust(zone.start_row, lines_scrolled);
                zone.end_row = adjust(zone.end_row, lines_scrolled);
            }
        }

        // Adjust current block
        if let Some(ref mut block) = self.current_block {
            block.start_row = adjust(block.start_row, lines_scrolled);
            block.end_row = adjust(block.end_row, lines_scrolled);

            if let Some(ref mut zone) = block.prompt_zone {
                zone.start_row = adjust(zone.start_row, lines_scrolled);
                zone.end_row = adjust(zone.end_row, lines_scrolled);
            }
            if let Some(ref mut zone) = block.input_zone {
                zone.start_row = adjust(zone.start_row, lines_scrolled);
                zone.end_row = adjust(zone.end_row, lines_scrolled);
            }
            if let Some(ref mut zone) = block.output_zone {
                zone.start_row = adjust(zone.start_row, lines_scrolled);
                zone.end_row = adjust(zone.end_row, lines_scrolled);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::string::ToString;

    #[test]
    fn test_semantic_zone_creation() {
        let zone = SemanticZone::new_prompt(1, 10, 1000);
        assert_eq!(zone.id, 1);
        assert_eq!(zone.zone_type, ZoneType::Prompt);
        assert_eq!(zone.start_row, 10);
        assert!(!zone.is_complete);
    }

    #[test]
    fn test_zone_completion() {
        let mut zone = SemanticZone::new_input(1, 10, 1000);
        zone.complete(15, 2000);

        assert!(zone.is_complete);
        assert_eq!(zone.end_row, 15);
        assert_eq!(zone.duration_micros, Some(1000));
        assert_eq!(zone.line_count(), 6);
    }

    #[test]
    fn test_zone_contains_line() {
        let mut zone = SemanticZone::new_output(1, 10, 1000);
        zone.complete(20, 2000);

        assert!(!zone.contains_line(9));
        assert!(zone.contains_line(10));
        assert!(zone.contains_line(15));
        assert!(zone.contains_line(20));
        assert!(!zone.contains_line(21));
    }

    #[test]
    fn test_zone_tracker_prompt_flow() {
        let mut tracker = ZoneTracker::new(100);

        // OSC 133;A - Prompt start
        tracker.mark_prompt_start(0, 1000);
        assert_eq!(tracker.zones().len(), 1);
        assert!(tracker.current_block().is_some());

        // OSC 133;B - Command start
        tracker.mark_command_start(1, 2000);
        assert_eq!(tracker.zones().len(), 2);

        // Check prompt zone was completed
        let prompt_zone = tracker
            .zones()
            .iter()
            .find(|z| z.zone_type == ZoneType::Prompt)
            .unwrap();
        assert!(prompt_zone.is_complete);
        assert_eq!(prompt_zone.end_row, 0);

        // OSC 133;C - Command executed
        tracker.mark_command_executed(2, 3000);
        assert_eq!(tracker.zones().len(), 3);

        // Check input zone was completed
        let input_zone = tracker
            .zones()
            .iter()
            .find(|z| z.zone_type == ZoneType::Input)
            .unwrap();
        assert!(input_zone.is_complete);
        assert_eq!(input_zone.end_row, 1);

        // OSC 133;D - Command finished
        tracker.mark_command_finished(10, 0, 4000);

        // Check output zone was completed
        let output_zone = tracker
            .zones()
            .iter()
            .find(|z| z.zone_type == ZoneType::Output)
            .unwrap();
        assert!(output_zone.is_complete);
        assert_eq!(output_zone.end_row, 10);
        assert_eq!(output_zone.exit_code, Some(0));

        // Check command block was created
        assert_eq!(tracker.command_blocks().len(), 1);
        let block = &tracker.command_blocks()[0];
        assert!(block.is_complete());
        assert!(block.is_success());
        assert_eq!(block.duration_secs(), Some(0.003));
    }

    #[test]
    fn test_command_block_with_failure() {
        let mut tracker = ZoneTracker::new(100);

        tracker.mark_prompt_start(0, 1000);
        tracker.mark_command_start(1, 2000);
        tracker.set_command_text("false".to_string());
        tracker.mark_command_executed(2, 3000);
        tracker.mark_command_finished(3, 1, 4000);

        let block = &tracker.command_blocks()[0];
        assert!(block.is_failure());
        assert_eq!(block.exit_code(), Some(1));
        assert_eq!(block.command_text(), Some("false"));
    }

    #[test]
    fn test_zone_tracker_max_blocks() {
        let mut tracker = ZoneTracker::new(3);

        // Create 5 command blocks
        for i in 0..5u64 {
            let base_line = (i * 10) as u32;
            let base_time = i * 10000;

            tracker.mark_prompt_start(base_line, base_time);
            tracker.mark_command_start(base_line + 1, base_time + 1000);
            tracker.mark_command_executed(base_line + 2, base_time + 2000);
            tracker.mark_command_finished(base_line + 3, 0, base_time + 3000);
        }

        // Should only keep the last 3 blocks
        assert_eq!(tracker.command_blocks().len(), 3);

        // First block should be the 3rd one created (0-indexed 2nd)
        assert_eq!(tracker.command_blocks()[0].start_row, 20);
    }

    #[test]
    fn test_find_zone_at_line() {
        let mut tracker = ZoneTracker::new(100);

        tracker.mark_prompt_start(10, 1000);
        tracker.mark_command_start(11, 2000);
        tracker.mark_command_executed(12, 3000);
        tracker.mark_command_finished(20, 0, 4000);

        // Test finding zones
        let zone = tracker.find_zone_at_line(15).unwrap();
        assert_eq!(zone.zone_type, ZoneType::Output);

        let block = tracker.find_block_at_line(15).unwrap();
        assert_eq!(block.start_row, 10);
    }

    #[test]
    fn test_last_output_zone() {
        let mut tracker = ZoneTracker::new(100);

        // Create two command blocks
        tracker.mark_prompt_start(0, 1000);
        tracker.mark_command_start(1, 2000);
        tracker.mark_command_executed(2, 3000);
        tracker.mark_command_finished(10, 0, 4000);

        tracker.mark_prompt_start(11, 5000);
        tracker.mark_command_start(12, 6000);
        tracker.mark_command_executed(13, 7000);
        tracker.mark_command_finished(20, 0, 8000);

        // Should return the most recent output zone
        let last_output = tracker.last_output_zone().unwrap();
        assert_eq!(last_output.start_row, 13);
        assert_eq!(last_output.end_row, 20);
    }

    #[test]
    fn test_adjust_for_scroll() {
        let mut tracker = ZoneTracker::new(100);

        tracker.mark_prompt_start(10, 1000);
        tracker.mark_command_start(11, 2000);
        tracker.mark_command_executed(12, 3000);

        // Scroll up by 5 lines (line numbers increase in scrollback)
        tracker.adjust_for_scroll(5);

        // Check that all zones were adjusted
        let prompt = tracker
            .zones()
            .iter()
            .find(|z| z.zone_type == ZoneType::Prompt)
            .unwrap();
        assert_eq!(prompt.start_row, 15);

        let input = tracker
            .zones()
            .iter()
            .find(|z| z.zone_type == ZoneType::Input)
            .unwrap();
        assert_eq!(input.start_row, 16);

        let output = tracker
            .zones()
            .iter()
            .find(|z| z.zone_type == ZoneType::Output)
            .unwrap();
        assert_eq!(output.start_row, 17);
    }
}
