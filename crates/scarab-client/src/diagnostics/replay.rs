//! Terminal session replay functionality
//!
//! This module provides the DiagnosticsReplay resource for playing back
//! recorded terminal sessions with timing and speed control.

use super::format::{Recording, RecordedEvent};
use anyhow::{Context, Result};
use bevy::prelude::*;
use std::path::Path;
use std::time::Instant;

/// Bevy resource for replaying terminal sessions
#[derive(Resource)]
pub struct DiagnosticsReplay {
    /// Loaded recording
    recording: Option<Recording>,
    /// Current playback position (event index)
    position: usize,
    /// Playback speed multiplier (1.0 = normal speed, 2.0 = 2x, etc.)
    speed: f32,
    /// Playback state
    state: PlaybackState,
    /// Time when playback started/resumed
    playback_start: Option<Instant>,
    /// Accumulated playback time in milliseconds (for pause/resume)
    accumulated_time_ms: u64,
    /// Whether to loop playback
    loop_playback: bool,
}

impl Default for DiagnosticsReplay {
    fn default() -> Self {
        Self::new()
    }
}

/// Playback state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlaybackState {
    /// Not loaded or stopped
    Stopped,
    /// Currently playing
    Playing,
    /// Paused
    Paused,
    /// Finished playing
    Finished,
}

impl DiagnosticsReplay {
    /// Create a new replay instance
    pub fn new() -> Self {
        Self {
            recording: None,
            position: 0,
            speed: 1.0,
            state: PlaybackState::Stopped,
            playback_start: None,
            accumulated_time_ms: 0,
            loop_playback: false,
        }
    }

    /// Load a recording from file
    pub fn load(&mut self, path: impl AsRef<Path>) -> Result<()> {
        let recording = Recording::from_file(path.as_ref())
            .context("Failed to load recording")?;

        info!(
            "Loaded recording: {} events, {} ms duration",
            recording.event_count(),
            recording.duration_ms()
        );

        self.recording = Some(recording);
        self.position = 0;
        self.state = PlaybackState::Stopped;
        self.playback_start = None;
        self.accumulated_time_ms = 0;

        Ok(())
    }

    /// Load a recording from a Recording struct
    pub fn load_recording(&mut self, recording: Recording) {
        info!(
            "Loaded recording: {} events, {} ms duration",
            recording.event_count(),
            recording.duration_ms()
        );

        self.recording = Some(recording);
        self.position = 0;
        self.state = PlaybackState::Stopped;
        self.playback_start = None;
        self.accumulated_time_ms = 0;
    }

    /// Start or resume playback
    pub fn play(&mut self) {
        if self.recording.is_none() {
            warn!("No recording loaded");
            return;
        }

        match self.state {
            PlaybackState::Stopped => {
                self.position = 0;
                self.accumulated_time_ms = 0;
                self.playback_start = Some(Instant::now());
                self.state = PlaybackState::Playing;
                info!("Started playback at {}x speed", self.speed);
            }
            PlaybackState::Paused => {
                self.playback_start = Some(Instant::now());
                self.state = PlaybackState::Playing;
                info!("Resumed playback");
            }
            PlaybackState::Finished => {
                if self.loop_playback {
                    self.position = 0;
                    self.accumulated_time_ms = 0;
                    self.playback_start = Some(Instant::now());
                    self.state = PlaybackState::Playing;
                    info!("Restarted playback (looping)");
                }
            }
            PlaybackState::Playing => {
                // Already playing
            }
        }
    }

    /// Pause playback
    pub fn pause(&mut self) {
        if self.state != PlaybackState::Playing {
            return;
        }

        // Save accumulated time
        if let Some(start) = self.playback_start {
            let elapsed_real_ms = start.elapsed().as_millis() as u64;
            let elapsed_playback_ms = (elapsed_real_ms as f32 * self.speed) as u64;
            self.accumulated_time_ms += elapsed_playback_ms;
        }

        self.state = PlaybackState::Paused;
        self.playback_start = None;
        info!("Paused playback at {}ms", self.accumulated_time_ms);
    }

    /// Stop playback and reset to beginning
    pub fn stop(&mut self) {
        self.position = 0;
        self.accumulated_time_ms = 0;
        self.playback_start = None;
        self.state = PlaybackState::Stopped;
        info!("Stopped playback");
    }

    /// Set playback speed (1.0 = normal, 2.0 = 2x, 0.5 = half speed)
    pub fn set_speed(&mut self, speed: f32) {
        if speed <= 0.0 {
            warn!("Invalid playback speed: {}", speed);
            return;
        }

        // If currently playing, update accumulated time before changing speed
        if self.state == PlaybackState::Playing {
            if let Some(start) = self.playback_start {
                let elapsed_real_ms = start.elapsed().as_millis() as u64;
                let elapsed_playback_ms = (elapsed_real_ms as f32 * self.speed) as u64;
                self.accumulated_time_ms += elapsed_playback_ms;
                self.playback_start = Some(Instant::now());
            }
        }

        self.speed = speed;
        info!("Set playback speed to {}x", speed);
    }

    /// Enable or disable loop playback
    pub fn set_loop(&mut self, enabled: bool) {
        self.loop_playback = enabled;
        info!("Loop playback: {}", if enabled { "enabled" } else { "disabled" });
    }

    /// Seek to specific position in milliseconds
    pub fn seek(&mut self, timestamp_ms: u64) {
        let Some(recording) = &self.recording else {
            return;
        };

        // Find the event closest to the timestamp
        let mut new_position = 0;
        for (i, event) in recording.events.iter().enumerate() {
            if event.timestamp_ms > timestamp_ms {
                break;
            }
            new_position = i;
        }

        self.position = new_position;
        self.accumulated_time_ms = timestamp_ms;

        if self.state == PlaybackState::Playing {
            self.playback_start = Some(Instant::now());
        }

        info!("Seeked to {}ms (event {})", timestamp_ms, new_position);
    }

    /// Get events that should be emitted at current playback time
    pub fn poll_events(&mut self) -> Vec<RecordedEvent> {
        let Some(recording) = &self.recording else {
            return Vec::new();
        };

        if self.state != PlaybackState::Playing {
            return Vec::new();
        }

        // Calculate current playback time
        let current_time_ms = self.current_playback_time_ms();

        let mut events = Vec::new();

        // Emit all events up to current playback time
        while self.position < recording.events.len() {
            let event = &recording.events[self.position];

            if event.timestamp_ms > current_time_ms {
                break;
            }

            events.push(event.clone());
            self.position += 1;
        }

        // Check if finished
        if self.position >= recording.events.len() {
            if self.loop_playback {
                self.position = 0;
                self.accumulated_time_ms = 0;
                self.playback_start = Some(Instant::now());
                info!("Playback finished, looping...");
            } else {
                self.state = PlaybackState::Finished;
                self.playback_start = None;
                info!("Playback finished");
            }
        }

        events
    }

    /// Get current playback time in milliseconds
    fn current_playback_time_ms(&self) -> u64 {
        if let Some(start) = self.playback_start {
            let elapsed_real_ms = start.elapsed().as_millis() as u64;
            let elapsed_playback_ms = (elapsed_real_ms as f32 * self.speed) as u64;
            self.accumulated_time_ms + elapsed_playback_ms
        } else {
            self.accumulated_time_ms
        }
    }

    /// Get current playback state
    pub fn state(&self) -> PlaybackState {
        self.state
    }

    /// Get current playback position (event index)
    pub fn position(&self) -> usize {
        self.position
    }

    /// Get playback speed
    pub fn speed(&self) -> f32 {
        self.speed
    }

    /// Get recording metadata
    pub fn metadata(&self) -> Option<&super::format::RecordingMetadata> {
        self.recording.as_ref().map(|r| &r.metadata)
    }

    /// Get total recording duration in milliseconds
    pub fn duration_ms(&self) -> Option<u64> {
        self.recording.as_ref().map(|r| r.duration_ms())
    }

    /// Get current playback progress (0.0 to 1.0)
    pub fn progress(&self) -> f32 {
        if let (Some(duration), Some(recording)) = (self.duration_ms(), &self.recording) {
            if duration > 0 && !recording.events.is_empty() {
                let current = self.current_playback_time_ms();
                (current as f32 / duration as f32).min(1.0)
            } else {
                0.0
            }
        } else {
            0.0
        }
    }

    /// Check if a recording is loaded
    pub fn is_loaded(&self) -> bool {
        self.recording.is_some()
    }
}

/// Bevy event emitted when a replay event should be processed
#[derive(Event)]
pub struct ReplayEvent {
    pub event: RecordedEvent,
}

/// Bevy event to control replay
#[derive(Event)]
pub enum ReplayControlEvent {
    Load { path: std::path::PathBuf },
    Play,
    Pause,
    Stop,
    SetSpeed { speed: f32 },
    SetLoop { enabled: bool },
    Seek { timestamp_ms: u64 },
}

/// System to poll replay events and emit them to Bevy
pub fn poll_replay_events(
    mut replay: ResMut<DiagnosticsReplay>,
    mut replay_events: EventWriter<ReplayEvent>,
) {
    let events = replay.poll_events();

    for event in events {
        replay_events.send(ReplayEvent { event });
    }
}

/// System to handle replay control events
pub fn handle_replay_control(
    mut replay: ResMut<DiagnosticsReplay>,
    mut control_events: EventReader<ReplayControlEvent>,
) {
    for event in control_events.read() {
        match event {
            ReplayControlEvent::Load { path } => {
                if let Err(e) = replay.load(path) {
                    error!("Failed to load recording: {}", e);
                }
            }
            ReplayControlEvent::Play => replay.play(),
            ReplayControlEvent::Pause => replay.pause(),
            ReplayControlEvent::Stop => replay.stop(),
            ReplayControlEvent::SetSpeed { speed } => replay.set_speed(*speed),
            ReplayControlEvent::SetLoop { enabled } => replay.set_loop(*enabled),
            ReplayControlEvent::Seek { timestamp_ms } => replay.seek(*timestamp_ms),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diagnostics::format::{EventData, Recording};

    fn create_test_recording() -> Recording {
        let mut recording = Recording::new(80, 24);
        recording.add_event(RecordedEvent::new(
            0,
            EventType::Input,
            EventData::data(b"test1"),
        ));
        recording.add_event(RecordedEvent::new(
            100,
            EventType::Output,
            EventData::data(b"test2"),
        ));
        recording.add_event(RecordedEvent::new(
            200,
            EventType::Output,
            EventData::data(b"test3"),
        ));
        recording
    }

    #[test]
    fn test_replay_load() {
        let mut replay = DiagnosticsReplay::new();
        let recording = create_test_recording();

        replay.load_recording(recording);
        assert!(replay.is_loaded());
        assert_eq!(replay.state(), PlaybackState::Stopped);
    }

    #[test]
    fn test_replay_speed() {
        let mut replay = DiagnosticsReplay::new();

        replay.set_speed(2.0);
        assert_eq!(replay.speed(), 2.0);

        replay.set_speed(0.5);
        assert_eq!(replay.speed(), 0.5);
    }

    #[test]
    fn test_replay_state_transitions() {
        let mut replay = DiagnosticsReplay::new();
        let recording = create_test_recording();
        replay.load_recording(recording);

        assert_eq!(replay.state(), PlaybackState::Stopped);

        replay.play();
        assert_eq!(replay.state(), PlaybackState::Playing);

        replay.pause();
        assert_eq!(replay.state(), PlaybackState::Paused);

        replay.play();
        assert_eq!(replay.state(), PlaybackState::Playing);

        replay.stop();
        assert_eq!(replay.state(), PlaybackState::Stopped);
    }
}
