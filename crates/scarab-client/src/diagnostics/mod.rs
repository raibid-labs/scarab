//! Diagnostics and session recording/replay system
//!
//! This module provides comprehensive terminal session recording and replay
//! functionality for debugging, testing, and demonstration purposes.
//!
//! ## Features
//!
//! - **Recording**: Capture all terminal input, output, and resize events with precise timing
//! - **Replay**: Play back recordings with speed control, pause/resume, and seeking
//! - **JSON Format**: Human-readable recording format for easy inspection and editing
//! - **Metadata**: Rich metadata including terminal size, timestamps, and custom annotations
//! - **Markers**: Add custom markers/annotations during recording for navigation
//!
//! ## Usage
//!
//! ### Recording a Session
//!
//! ```rust,ignore
//! use bevy::prelude::*;
//! use scarab_client::diagnostics::*;
//!
//! fn start_recording_system(
//!     mut recorder: ResMut<DiagnosticsRecorder>,
//! ) {
//!     recorder.start(80, 24);
//! }
//!
//! fn record_events_system(
//!     mut recorder: ResMut<DiagnosticsRecorder>,
//! ) {
//!     recorder.record_input(b"ls -la\n");
//!     recorder.record_output(b"total 42\n");
//! }
//!
//! fn stop_recording_system(
//!     mut recorder: ResMut<DiagnosticsRecorder>,
//! ) {
//!     if let Err(e) = recorder.stop_and_save("/tmp/session.json") {
//!         eprintln!("Failed to save: {}", e);
//!     }
//! }
//! ```
//!
//! ### Replaying a Session
//!
//! ```rust,ignore
//! use bevy::prelude::*;
//! use scarab_client::diagnostics::*;
//!
//! fn load_replay_system(
//!     mut replay: ResMut<DiagnosticsReplay>,
//! ) {
//!     if let Err(e) = replay.load("/tmp/session.json") {
//!         eprintln!("Failed to load: {}", e);
//!         return;
//!     }
//!
//!     replay.set_speed(2.0); // 2x speed
//!     replay.play();
//! }
//!
//! fn handle_replay_events_system(
//!     mut events: EventReader<ReplayEvent>,
//! ) {
//!     for event in events.read() {
//!         match event.event.event_type {
//!             EventType::Input => { /* handle input */ }
//!             EventType::Output => { /* handle output */ }
//!             EventType::Resize => { /* handle resize */ }
//!             EventType::Marker => { /* handle marker */ }
//!         }
//!     }
//! }
//! ```
//!
//! ## Recording Format
//!
//! Recordings are stored as JSON with the following structure:
//!
//! ```json
//! {
//!   "version": "1.0",
//!   "metadata": {
//!     "recorded_at": "2025-12-03T12:34:56Z",
//!     "terminal_size": [80, 24],
//!     "scarab_version": "0.2.0"
//!   },
//!   "events": [
//!     {"t": 0, "type": "input", "data": "ls -la\n"},
//!     {"t": 50, "type": "output", "data": "total 42\n"},
//!     {"t": 100, "type": "resize", "cols": 100, "rows": 30}
//!   ]
//! }
//! ```

pub mod format;
pub mod recorder;
pub mod replay;

// Re-export main types
pub use format::{
    EventData, EventType, RecordedEvent, Recording, RecordingMetadata, FORMAT_VERSION,
};

pub use recorder::{
    handle_recording_events, DiagnosticsRecorder, RecordMarkerEvent, RecordingStats,
    StartRecordingEvent, StopRecordingEvent,
};

pub use replay::{
    handle_replay_control, poll_replay_events, DiagnosticsReplay, PlaybackState,
    ReplayControlEvent, ReplayEvent,
};

use bevy::prelude::*;

/// Bevy plugin for diagnostics recording and replay
pub struct DiagnosticsPlugin;

impl Plugin for DiagnosticsPlugin {
    fn build(&self, app: &mut App) {
        app
            // Register resources
            .insert_resource(DiagnosticsRecorder::new())
            .insert_resource(DiagnosticsReplay::new())
            // Register events
            .add_event::<StartRecordingEvent>()
            .add_event::<StopRecordingEvent>()
            .add_event::<RecordMarkerEvent>()
            .add_event::<ReplayEvent>()
            .add_event::<ReplayControlEvent>()
            // Register systems
            .add_systems(
                Update,
                (
                    handle_recording_events,
                    poll_replay_events,
                    handle_replay_control,
                ),
            );

        info!("DiagnosticsPlugin initialized");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plugin_initialization() {
        let mut app = App::new();
        app.add_plugins(DiagnosticsPlugin);

        // Verify resources are registered
        assert!(app.world().contains_resource::<DiagnosticsRecorder>());
        assert!(app.world().contains_resource::<DiagnosticsReplay>());
    }
}
