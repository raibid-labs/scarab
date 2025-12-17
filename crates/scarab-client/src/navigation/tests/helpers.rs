//! Shared test helpers for navigation tests

// Re-export commonly used types
pub use bevy::ecs::system::RunSystemOnce;
pub use bevy::prelude::*;
pub use scarab_protocol::{Cell, SharedState, TerminalMetrics};
pub use shared_memory::*;
pub use std::sync::Arc;

pub use crate::events::{PaneClosedEvent, PaneCreatedEvent, PaneFocusedEvent};
pub use crate::integration::SharedMemoryReader;
pub use crate::prompt_markers::{NavAnchor, PromptAnchorType, PromptMarkers, PromptZoneFocusedEvent};
pub use crate::safe_state::MockTerminalState;

pub use super::super::focusable::*;
pub use super::super::*;

/// Build a minimal headless Bevy app for navigation testing
pub fn build_test_app() -> App {
    let mut app = App::new();

    // Add minimal plugins (no rendering or windowing)
    app.add_plugins(MinimalPlugins);

    // Add navigation plugins (core navigation only, not focusable plugin with its systems)
    app.add_plugins(NavigationPlugin);
    // Skip FocusablePlugin since it requires SharedMemoryReader

    // Create terminal metrics resource
    let metrics = TerminalMetrics {
        cell_width: 10.0,
        cell_height: 20.0,
        columns: 80,
        rows: 24,
    };
    app.insert_resource(metrics);

    // Insert focusable scan config
    app.insert_resource(FocusableScanConfig::default());

    // Insert prompt markers resource
    app.insert_resource(PromptMarkers::default());

    // Insert NavState resource for tests that access it directly
    app.insert_resource(NavState::default());

    app
}

/// Helper to create a scrollback line from a string
pub fn create_scrollback_line(text: &str) -> crate::terminal::scrollback::ScrollbackLine {
    use scarab_protocol::Cell;

    let cells: Vec<Cell> = text
        .chars()
        .map(|c| Cell {
            char_codepoint: c as u32,
            fg: 0xFFFFFF,
            bg: 0x000000,
            flags: 0,
            _padding: [0; 3],
        })
        .collect();
    crate::terminal::scrollback::ScrollbackLine::new(cells)
}
