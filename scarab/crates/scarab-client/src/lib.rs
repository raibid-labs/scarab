// Scarab terminal emulator client library
// Re-exports UI and rendering modules for use in binary and tests

// Temporarily use UI stub during Bevy 0.15 migration
// pub mod ui;
pub mod ui_stub;
pub use ui_stub as ui;

pub mod rendering;
pub mod ipc;
pub mod integration;

pub use rendering::*;

// Re-export commonly used integration types
pub use integration::{IntegrationPlugin, extract_grid_text, get_cell_at};
