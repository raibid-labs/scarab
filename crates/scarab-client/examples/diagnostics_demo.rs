//! Diagnostics Recording and Replay Demo
//!
//! This example demonstrates how to use the diagnostics system to record
//! and replay terminal sessions.
//!
//! Run with:
//! ```bash
//! cargo run --example diagnostics_demo
//! ```

use bevy::prelude::*;
use scarab_client::diagnostics::*;
use std::path::PathBuf;

fn main() {
    App::new()
        .add_plugins(MinimalPlugins)
        .add_plugins(DiagnosticsPlugin)
        .add_systems(Startup, setup_demo)
        .add_systems(Update, (demo_recording, demo_replay))
        .run();
}

fn setup_demo(mut commands: Commands) {
    info!("Diagnostics Demo Started");
    commands.insert_resource(DemoState::default());
}

#[derive(Resource, Default)]
struct DemoState {
    phase: DemoPhase,
    frame_count: u32,
}

#[derive(Default, PartialEq)]
enum DemoPhase {
    #[default]
    Recording,
    Saving,
    Loading,
    Replaying,
    Done,
}

fn demo_recording(
    mut state: ResMut<DemoState>,
    mut recorder: ResMut<DiagnosticsRecorder>,
    mut start_events: EventWriter<StartRecordingEvent>,
    mut stop_events: EventWriter<StopRecordingEvent>,
) {
    state.frame_count += 1;

    match state.phase {
        DemoPhase::Recording => {
            if state.frame_count == 1 {
                info!("=== Phase 1: Recording ===");
                start_events.send(StartRecordingEvent {
                    terminal_cols: 80,
                    terminal_rows: 24,
                    title: Some("Demo Session".to_string()),
                    description: Some("Demonstration of recording functionality".to_string()),
                    auto_save_path: None,
                });
            }

            // Simulate recording events
            if state.frame_count == 30 {
                recorder.record_input(b"echo 'Hello, Scarab!'\n");
                info!("Recorded input: echo 'Hello, Scarab!'");
            }

            if state.frame_count == 60 {
                recorder.record_output(b"Hello, Scarab!\n");
                info!("Recorded output: Hello, Scarab!");
            }

            if state.frame_count == 90 {
                recorder.record_input(b"ls -la\n");
                info!("Recorded input: ls -la");
            }

            if state.frame_count == 120 {
                recorder.record_output(b"total 42\ndrwxr-xr-x  5 user  staff   160 Dec  3 12:00 .\n");
                info!("Recorded output: ls -la results");
            }

            if state.frame_count == 150 {
                recorder.record_marker("Checkpoint 1");
                info!("Recorded marker: Checkpoint 1");
            }

            if state.frame_count == 180 {
                recorder.record_resize(100, 30);
                info!("Recorded resize: 100x30");
            }

            if state.frame_count == 200 {
                let stats = recorder.stats();
                info!("Recording Statistics:");
                info!("  Total events: {}", stats.total_events());
                info!("  Input events: {}", stats.input_events);
                info!("  Output events: {}", stats.output_events);
                info!("  Resize events: {}", stats.resize_events);
                info!("  Marker events: {}", stats.marker_events);
                info!("  Total bytes: {}", stats.total_bytes());

                state.phase = DemoPhase::Saving;
                state.frame_count = 0;
            }
        }
        DemoPhase::Saving => {
            if state.frame_count == 1 {
                info!("=== Phase 2: Saving ===");
                let path = PathBuf::from("/tmp/scarab_demo.json");
                stop_events.send(StopRecordingEvent {
                    save_path: Some(path.clone()),
                });
                info!("Saving recording to: {}", path.display());
            }

            if state.frame_count == 30 {
                state.phase = DemoPhase::Loading;
                state.frame_count = 0;
            }
        }
        _ => {}
    }
}

fn demo_replay(
    mut state: ResMut<DemoState>,
    mut replay: ResMut<DiagnosticsReplay>,
    mut control_events: EventWriter<ReplayControlEvent>,
    mut replay_events: EventReader<ReplayEvent>,
) {
    match state.phase {
        DemoPhase::Loading => {
            if state.frame_count == 1 {
                info!("=== Phase 3: Loading ===");
                let path = PathBuf::from("/tmp/scarab_demo.json");
                control_events.send(ReplayControlEvent::Load { path: path.clone() });
                info!("Loading recording from: {}", path.display());
            }

            if state.frame_count == 30 {
                if let Some(metadata) = replay.metadata() {
                    info!("Recording Metadata:");
                    info!("  Title: {:?}", metadata.title);
                    info!("  Description: {:?}", metadata.description);
                    info!("  Terminal size: {:?}", metadata.terminal_size);
                    info!("  Recorded at: {}", metadata.recorded_at);
                }

                if let Some(duration) = replay.duration_ms() {
                    info!("Recording duration: {}ms", duration);
                }

                state.phase = DemoPhase::Replaying;
                state.frame_count = 0;
            }
        }
        DemoPhase::Replaying => {
            if state.frame_count == 1 {
                info!("=== Phase 4: Replaying ===");
                control_events.send(ReplayControlEvent::SetSpeed { speed: 2.0 });
                info!("Set playback speed to 2x");
                control_events.send(ReplayControlEvent::Play);
                info!("Started playback");
            }

            // Process replay events
            for event in replay_events.read() {
                match event.event.event_type {
                    EventType::Input => {
                        if let Some(data) = event.event.data.as_bytes() {
                            info!(
                                "Replay [{}ms] Input: {:?}",
                                event.event.timestamp_ms,
                                String::from_utf8_lossy(&data)
                            );
                        }
                    }
                    EventType::Output => {
                        if let Some(data) = event.event.data.as_bytes() {
                            info!(
                                "Replay [{}ms] Output: {:?}",
                                event.event.timestamp_ms,
                                String::from_utf8_lossy(&data)
                            );
                        }
                    }
                    EventType::Resize => {
                        info!(
                            "Replay [{}ms] Resize event",
                            event.event.timestamp_ms
                        );
                    }
                    EventType::Marker => {
                        info!(
                            "Replay [{}ms] Marker event",
                            event.event.timestamp_ms
                        );
                    }
                }
            }

            // Check if playback finished
            if state.frame_count > 300 && replay.state() == PlaybackState::Finished {
                info!("Playback finished!");
                info!("Final progress: {:.1}%", replay.progress() * 100.0);

                state.phase = DemoPhase::Done;
                state.frame_count = 0;
            }
        }
        DemoPhase::Done => {
            if state.frame_count == 1 {
                info!("=== Demo Complete ===");
                info!("The recording was saved to /tmp/scarab_demo.json");
                info!("You can inspect it with: cat /tmp/scarab_demo.json | jq");
            }

            if state.frame_count == 60 {
                info!("Exiting demo...");
                std::process::exit(0);
            }
        }
        _ => {}
    }

    state.frame_count += 1;
}
