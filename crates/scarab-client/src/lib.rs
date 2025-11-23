// Scarab terminal emulator client library
// Re-exports UI and rendering modules for use in binary and tests

// UI features updated for SharedMemoryReader integration but temporarily disabled
// due to Bevy 0.15 API changes (Text, Style, etc.) needing separate migration task
// pub mod ui;
pub mod ui_stub;
pub use ui_stub::*;

pub mod rendering;
pub mod ipc;
pub mod integration;

pub use rendering::*;

// Re-export commonly used integration types
pub use integration::{IntegrationPlugin, extract_grid_text, get_cell_at, SharedMemoryReader};
