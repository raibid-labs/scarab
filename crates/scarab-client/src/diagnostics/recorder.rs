//! Terminal session recording functionality
//!
//! This module provides the DiagnosticsRecorder resource for capturing
//! terminal input, output, and resize events in real-time.

use super::format::{EventData, EventType, RecordedEvent, Recording};
use anyhow::{Context, Result};
use bevy::prelude::*;
use std::path::PathBuf;
use std::time::Instant;

/// Bevy resource for recording terminal sessions
#[derive(Resource)]
pub struct DiagnosticsRecorder {
    /// Whether recording is currently active
    recording: bool,
    /// Current recording data
    current_recording: Option<Recording>,
    /// Recording start time
    start_time: Option<Instant>,
    /// Auto-save path (if configured)
    auto_save_path: Option<PathBuf>,
    /// Recording statistics
    stats: RecordingStats,
}

impl Default for DiagnosticsRecorder {
    fn default() -> Self {
        Self::new()
    }
}

impl DiagnosticsRecorder {
    /// Create a new recorder (not recording by default)
    pub fn new() -> Self {
        Self {
            recording: false,
            current_recording: None,
            start_time: None,
            auto_save_path: None,
            stats: RecordingStats::default(),
        }
    }

    /// Start a new recording session
    pub fn start(&mut self, terminal_cols: u16, terminal_rows: u16) {
        if self.recording {
            warn!("Already recording, stopping previous recording");
            self.stop();
        }

        let recording = Recording::new(terminal_cols, terminal_rows);
        self.current_recording = Some(recording);
        self.start_time = Some(Instant::now());
        self.recording = true;
        self.stats = RecordingStats::default();

        info!(
            "Started recording session ({}x{})",
            terminal_cols, terminal_rows
        );
    }

    /// Start recording with metadata
    pub fn start_with_metadata(
        &mut self,
        terminal_cols: u16,
        terminal_rows: u16,
        title: Option<String>,
        description: Option<String>,
    ) {
        self.start(terminal_cols, terminal_rows);

        if let Some(recording) = &mut self.current_recording {
            if let Some(title) = title {
                recording.metadata.title = Some(title);
            }
            if let Some(description) = description {
                recording.metadata.description = Some(description);
            }
        }
    }

    /// Stop recording and return the completed recording
    pub fn stop(&mut self) -> Option<Recording> {
        if !self.recording {
            warn!("Not currently recording");
            return None;
        }

        self.recording = false;
        let recording = self.current_recording.take();
        self.start_time = None;

        if let Some(ref rec) = recording {
            info!(
                "Stopped recording: {} events, {} ms duration",
                rec.event_count(),
                rec.duration_ms()
            );
        }

        recording
    }

    /// Stop and save recording to file
    pub fn stop_and_save(&mut self, path: impl Into<PathBuf>) -> Result<()> {
        let recording = self.stop().context("No active recording to save")?;

        let path = path.into();
        recording
            .to_file(&path)
            .context("Failed to save recording")?;

        info!("Recording saved to: {}", path.display());
        Ok(())
    }

    /// Record user input event
    pub fn record_input(&mut self, data: &[u8]) {
        if !self.recording {
            return;
        }

        let timestamp_ms = self.elapsed_ms();
        let event = RecordedEvent::new(timestamp_ms, EventType::Input, EventData::data(data));

        if let Some(recording) = &mut self.current_recording {
            recording.add_event(event);
            self.stats.input_events += 1;
            self.stats.input_bytes += data.len();
        }
    }

    /// Record terminal output event
    pub fn record_output(&mut self, data: &[u8]) {
        if !self.recording {
            return;
        }

        let timestamp_ms = self.elapsed_ms();
        let event = RecordedEvent::new(timestamp_ms, EventType::Output, EventData::data(data));

        if let Some(recording) = &mut self.current_recording {
            recording.add_event(event);
            self.stats.output_events += 1;
            self.stats.output_bytes += data.len();
        }
    }

    /// Record terminal resize event
    pub fn record_resize(&mut self, cols: u16, rows: u16) {
        if !self.recording {
            return;
        }

        let timestamp_ms = self.elapsed_ms();
        let event = RecordedEvent::new(
            timestamp_ms,
            EventType::Resize,
            EventData::resize(cols, rows),
        );

        if let Some(recording) = &mut self.current_recording {
            recording.add_event(event);
            self.stats.resize_events += 1;
        }

        info!("Recorded resize event: {}x{}", cols, rows);
    }

    /// Record a marker/annotation
    pub fn record_marker(&mut self, label: impl Into<String>) {
        if !self.recording {
            return;
        }

        let timestamp_ms = self.elapsed_ms();
        let event = RecordedEvent::new(timestamp_ms, EventType::Marker, EventData::marker(label));

        if let Some(recording) = &mut self.current_recording {
            recording.add_event(event);
            self.stats.marker_events += 1;
        }
    }

    /// Check if currently recording
    pub fn is_recording(&self) -> bool {
        self.recording
    }

    /// Get elapsed time in milliseconds since recording start
    fn elapsed_ms(&self) -> u64 {
        self.start_time
            .map(|start| start.elapsed().as_millis() as u64)
            .unwrap_or(0)
    }

    /// Get current recording statistics
    pub fn stats(&self) -> &RecordingStats {
        &self.stats
    }

    /// Set auto-save path (will save on stop)
    pub fn set_auto_save_path(&mut self, path: Option<PathBuf>) {
        self.auto_save_path = path;
    }

    /// Get current recording metadata (if recording)
    pub fn metadata(&self) -> Option<&super::format::RecordingMetadata> {
        self.current_recording.as_ref().map(|r| &r.metadata)
    }
}

/// Recording statistics
#[derive(Debug, Default, Clone)]
pub struct RecordingStats {
    /// Number of input events recorded
    pub input_events: usize,
    /// Total input bytes recorded
    pub input_bytes: usize,
    /// Number of output events recorded
    pub output_events: usize,
    /// Total output bytes recorded
    pub output_bytes: usize,
    /// Number of resize events recorded
    pub resize_events: usize,
    /// Number of marker events recorded
    pub marker_events: usize,
}

impl RecordingStats {
    /// Get total event count
    pub fn total_events(&self) -> usize {
        self.input_events + self.output_events + self.resize_events + self.marker_events
    }

    /// Get total bytes recorded
    pub fn total_bytes(&self) -> usize {
        self.input_bytes + self.output_bytes
    }
}

/// Bevy event to start recording
#[derive(Event)]
pub struct StartRecordingEvent {
    pub terminal_cols: u16,
    pub terminal_rows: u16,
    pub title: Option<String>,
    pub description: Option<String>,
    pub auto_save_path: Option<PathBuf>,
}

/// Bevy event to stop recording
#[derive(Event)]
pub struct StopRecordingEvent {
    /// Optional path to save the recording
    pub save_path: Option<PathBuf>,
}

/// Bevy event to add a marker to the current recording
#[derive(Event)]
pub struct RecordMarkerEvent {
    pub label: String,
}

/// System to handle recording events
pub fn handle_recording_events(
    mut recorder: ResMut<DiagnosticsRecorder>,
    mut start_events: EventReader<StartRecordingEvent>,
    mut stop_events: EventReader<StopRecordingEvent>,
    mut marker_events: EventReader<RecordMarkerEvent>,
) {
    // Handle start recording events
    for event in start_events.read() {
        recorder.start_with_metadata(
            event.terminal_cols,
            event.terminal_rows,
            event.title.clone(),
            event.description.clone(),
        );

        if let Some(path) = &event.auto_save_path {
            recorder.set_auto_save_path(Some(path.clone()));
        }
    }

    // Handle stop recording events
    for event in stop_events.read() {
        if let Some(path) = &event.save_path {
            if let Err(e) = recorder.stop_and_save(path) {
                error!("Failed to save recording: {}", e);
            }
        } else if let Some(path) = recorder.auto_save_path.clone() {
            if let Err(e) = recorder.stop_and_save(path) {
                error!("Failed to auto-save recording: {}", e);
            }
        } else {
            recorder.stop();
        }
    }

    // Handle marker events
    for event in marker_events.read() {
        recorder.record_marker(event.label.clone());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_recorder_lifecycle() {
        let mut recorder = DiagnosticsRecorder::new();
        assert!(!recorder.is_recording());

        recorder.start(80, 24);
        assert!(recorder.is_recording());

        recorder.record_input(b"test input");
        recorder.record_output(b"test output");
        recorder.record_resize(100, 30);

        let recording = recorder.stop().unwrap();
        assert!(!recorder.is_recording());
        assert_eq!(recording.event_count(), 3);
    }

    #[test]
    fn test_recording_stats() {
        let mut recorder = DiagnosticsRecorder::new();
        recorder.start(80, 24);

        recorder.record_input(b"hello");
        recorder.record_output(b"world!");
        recorder.record_marker("checkpoint");

        let stats = recorder.stats();
        assert_eq!(stats.input_events, 1);
        assert_eq!(stats.input_bytes, 5);
        assert_eq!(stats.output_events, 1);
        assert_eq!(stats.output_bytes, 6);
        assert_eq!(stats.marker_events, 1);
        assert_eq!(stats.total_events(), 3);
    }
}
