// Scarab terminal emulator client library
// Re-exports UI and rendering modules for use in binary and tests

pub mod ui;
pub mod ui_stub;

pub mod rendering;
pub mod ipc;
pub mod integration;

pub use rendering::*;

// Re-export commonly used integration types
pub use integration::{IntegrationPlugin, extract_grid_text, get_cell_at, SharedMemoryReader};
