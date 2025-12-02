// Scarab terminal emulator client library
// Re-exports UI and rendering modules for use in binary and tests

pub mod safe_state;
pub mod terminal;
pub mod ui;
pub mod ui_stub;

pub mod copy_mode;
pub mod input;
pub mod integration;
pub mod ipc;
pub mod rendering;
pub mod scripting;
pub mod tutorial;

#[cfg(feature = "plugin-inspector")]
pub mod plugin_inspector;

pub use rendering::*;

// Re-export commonly used integration types
pub use integration::{extract_grid_text, get_cell_at, IntegrationPlugin, SharedMemoryReader};

// Re-export safe state abstractions
pub use safe_state::{MockTerminalState, SafeSharedState};

// Re-export terminal types
pub use terminal::scrollback::{
    ScrollbackBuffer, ScrollbackLine, ScrollbackPlugin, ScrollbackState,
};

// Re-export UI plugin
pub use ui_stub::AdvancedUIPlugin;

// Re-export copy mode system
pub use copy_mode::{
    copy_mode_active, CopyModeCursorMarker, CopyModePlugin, CopyModeSearchResource,
    CopyModeStateResource, SelectionHighlight,
};

// Re-export scripting system
pub use scripting::{RuntimeContext, ScriptEvent, ScriptManager, ScriptingPlugin};

// Re-export tutorial system
pub use tutorial::{TutorialEvent, TutorialPlugin, TutorialState, TutorialSystem};

// Re-export plugin inspector (feature-gated)
#[cfg(feature = "plugin-inspector")]
pub use plugin_inspector::{PluginInspectorPlugin, PluginInspectorState};
