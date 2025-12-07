//! Recording format definitions for diagnostic session recording/replay
//!
//! This module defines the on-disk format for terminal session recordings,
//! including metadata, events, and serialization/deserialization logic.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

/// Current recording format version
pub const FORMAT_VERSION: &str = "1.0";

/// Complete recording structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Recording {
    /// Format version for compatibility checking
    pub version: String,
    /// Recording metadata
    pub metadata: RecordingMetadata,
    /// Recorded events in chronological order
    pub events: Vec<RecordedEvent>,
}

impl Recording {
    /// Create a new empty recording with current metadata
    pub fn new(terminal_cols: u16, terminal_rows: u16) -> Self {
        Self {
            version: FORMAT_VERSION.to_string(),
            metadata: RecordingMetadata::new(terminal_cols, terminal_rows),
            events: Vec::new(),
        }
    }

    /// Load recording from JSON file
    pub fn from_file(path: impl AsRef<Path>) -> Result<Self> {
        let data =
            std::fs::read_to_string(path.as_ref()).context("Failed to read recording file")?;

        let recording: Recording =
            serde_json::from_str(&data).context("Failed to parse recording JSON")?;

        // Validate version compatibility
        if recording.version != FORMAT_VERSION {
            anyhow::bail!(
                "Unsupported recording version: {} (expected {})",
                recording.version,
                FORMAT_VERSION
            );
        }

        Ok(recording)
    }

    /// Save recording to JSON file
    pub fn to_file(&self, path: impl AsRef<Path>) -> Result<()> {
        let json = serde_json::to_string_pretty(self).context("Failed to serialize recording")?;

        std::fs::write(path.as_ref(), json).context("Failed to write recording file")?;

        Ok(())
    }

    /// Get total recording duration in milliseconds
    pub fn duration_ms(&self) -> u64 {
        self.events.last().map(|e| e.timestamp_ms).unwrap_or(0)
    }

    /// Get event count
    pub fn event_count(&self) -> usize {
        self.events.len()
    }

    /// Add an event to the recording
    pub fn add_event(&mut self, event: RecordedEvent) {
        self.events.push(event);
    }
}

/// Recording metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordingMetadata {
    /// When the recording was created (ISO 8601 timestamp)
    pub recorded_at: String,
    /// Terminal size at recording time [cols, rows]
    pub terminal_size: [u16; 2],
    /// Scarab version used for recording
    pub scarab_version: String,
    /// Optional user-provided title
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    /// Optional description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Environment variables captured at recording time
    #[serde(skip_serializing_if = "Option::is_none")]
    pub environment: Option<std::collections::HashMap<String, String>>,
}

impl RecordingMetadata {
    /// Create new metadata with current timestamp
    pub fn new(terminal_cols: u16, terminal_rows: u16) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default();

        // Format as ISO 8601
        let recorded_at = chrono::DateTime::from_timestamp(now.as_secs() as i64, 0)
            .map(|dt| dt.format("%Y-%m-%dT%H:%M:%SZ").to_string())
            .unwrap_or_else(|| "unknown".to_string());

        Self {
            recorded_at,
            terminal_size: [terminal_cols, terminal_rows],
            scarab_version: env!("CARGO_PKG_VERSION").to_string(),
            title: None,
            description: None,
            environment: None,
        }
    }

    /// Set title for the recording
    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Set description for the recording
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Capture specific environment variables
    pub fn with_env_vars(mut self, vars: &[&str]) -> Self {
        let mut env_map = std::collections::HashMap::new();
        for var in vars {
            if let Ok(value) = std::env::var(var) {
                env_map.insert(var.to_string(), value);
            }
        }
        if !env_map.is_empty() {
            self.environment = Some(env_map);
        }
        self
    }
}

/// A single recorded event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordedEvent {
    /// Timestamp in milliseconds from recording start
    #[serde(rename = "t")]
    pub timestamp_ms: u64,
    /// Event type
    #[serde(rename = "type")]
    pub event_type: EventType,
    /// Event-specific data
    #[serde(flatten)]
    pub data: EventData,
}

impl RecordedEvent {
    /// Create a new event with timestamp
    pub fn new(timestamp_ms: u64, event_type: EventType, data: EventData) -> Self {
        Self {
            timestamp_ms,
            event_type,
            data,
        }
    }
}

/// Event type enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EventType {
    /// User input to terminal
    Input,
    /// Output from terminal
    Output,
    /// Terminal resize
    Resize,
    /// Marker/annotation
    Marker,
}

/// Event data (type-specific payload)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum EventData {
    /// Input or output data
    Data {
        /// UTF-8 encoded string or base64 for binary data
        data: String,
    },
    /// Resize event
    Resize {
        /// New column count
        cols: u16,
        /// New row count
        rows: u16,
    },
    /// Marker/annotation
    Marker {
        /// Marker label
        label: String,
        /// Optional marker metadata
        #[serde(skip_serializing_if = "Option::is_none")]
        metadata: Option<serde_json::Value>,
    },
}

impl EventData {
    /// Create data payload for input/output
    pub fn data(bytes: &[u8]) -> Self {
        // Try to decode as UTF-8, fall back to base64 for binary data
        let data = if let Ok(s) = std::str::from_utf8(bytes) {
            s.to_string()
        } else {
            // Encode as base64 with prefix to indicate encoding
            format!("base64:{}", base64_encode(bytes))
        };

        Self::Data { data }
    }

    /// Create resize payload
    pub fn resize(cols: u16, rows: u16) -> Self {
        Self::Resize { cols, rows }
    }

    /// Create marker payload
    pub fn marker(label: impl Into<String>) -> Self {
        Self::Marker {
            label: label.into(),
            metadata: None,
        }
    }

    /// Get data bytes (decode base64 if needed)
    pub fn as_bytes(&self) -> Option<Vec<u8>> {
        match self {
            Self::Data { data } => {
                if let Some(b64) = data.strip_prefix("base64:") {
                    base64_decode(b64).ok()
                } else {
                    Some(data.as_bytes().to_vec())
                }
            }
            _ => None,
        }
    }
}

/// Simple base64 encoding (using standard library)
fn base64_encode(bytes: &[u8]) -> String {
    // Use base64 crate if available, otherwise implement simple encoding
    use std::io::Write;
    let mut buf = Vec::new();
    {
        let mut encoder =
            base64::write::EncoderWriter::new(&mut buf, &base64::engine::general_purpose::STANDARD);
        encoder.write_all(bytes).unwrap();
    }
    String::from_utf8(buf).unwrap()
}

/// Simple base64 decoding
fn base64_decode(s: &str) -> Result<Vec<u8>> {
    use base64::Engine;
    base64::engine::general_purpose::STANDARD
        .decode(s)
        .context("Failed to decode base64")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_recording_creation() {
        let recording = Recording::new(80, 24);
        assert_eq!(recording.version, FORMAT_VERSION);
        assert_eq!(recording.metadata.terminal_size, [80, 24]);
        assert_eq!(recording.events.len(), 0);
    }

    #[test]
    fn test_event_data_encoding() {
        // UTF-8 data
        let utf8_data = EventData::data(b"hello world");
        assert_eq!(utf8_data.as_bytes().unwrap(), b"hello world");

        // Binary data
        let binary_data = EventData::data(&[0xFF, 0xFE, 0xFD]);
        let decoded = binary_data.as_bytes().unwrap();
        assert_eq!(decoded, vec![0xFF, 0xFE, 0xFD]);
    }

    #[test]
    fn test_serialization() {
        let mut recording = Recording::new(80, 24);
        recording.add_event(RecordedEvent::new(
            0,
            EventType::Input,
            EventData::data(b"ls -la\n"),
        ));
        recording.add_event(RecordedEvent::new(
            100,
            EventType::Output,
            EventData::data(b"total 42\n"),
        ));

        let json = serde_json::to_string_pretty(&recording).unwrap();
        let deserialized: Recording = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.events.len(), 2);
        assert_eq!(deserialized.events[0].timestamp_ms, 0);
        assert_eq!(deserialized.events[1].timestamp_ms, 100);
    }
}
