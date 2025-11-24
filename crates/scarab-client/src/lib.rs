// Scarab terminal emulator client library
// Re-exports UI and rendering modules for use in binary and tests

pub mod ui;
pub mod ui_stub;

pub mod integration;
pub mod ipc;
pub mod rendering;

pub use rendering::*;

// Re-export commonly used integration types
pub use integration::{extract_grid_text, get_cell_at, IntegrationPlugin, SharedMemoryReader};

// Re-export UI plugin
pub use ui_stub::AdvancedUIPlugin;
