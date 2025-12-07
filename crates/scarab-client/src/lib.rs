// Scarab terminal emulator client library
// Re-exports UI and rendering modules for use in binary and tests

pub mod safe_state;
pub mod terminal;
pub mod ui;
pub mod ui_stub;

pub mod accessibility;
pub mod context_menu;
pub mod copy_mode;
pub mod diagnostics;
pub mod events;
pub mod input;
pub mod integration;
pub mod ipc;
pub mod marketplace;
pub mod navigation;
pub mod plugin_host;
pub mod prompt_markers;
pub mod ratatui_bridge;
pub mod rendering;
pub mod scripting;
pub mod shaders;
pub mod telemetry_integration;
pub mod tutorial;
pub mod zones;

#[cfg(feature = "plugin-inspector")]
pub mod plugin_inspector;

pub mod graphics_inspector;

// Developer tools (debug builds only)
#[cfg(debug_assertions)]
pub mod dev;

pub use rendering::*;

// Re-export commonly used integration types
pub use integration::{extract_grid_text, get_cell_at, IntegrationPlugin, SharedMemoryReader};

// Re-export safe state abstractions
pub use safe_state::{MockTerminalState, SafeSharedState};

// Re-export terminal types
pub use terminal::scrollback::{
    ScrollbackBuffer, ScrollbackLine, ScrollbackPlugin, ScrollbackState,
};

// Re-export chunk system
pub use terminal::chunks::{
    ChunkGrid, ChunkMesh, ChunkPlugin, TerminalChunk, CHUNK_HEIGHT, CHUNK_WIDTH,
};

// Re-export UI plugin
pub use ui_stub::AdvancedUIPlugin;

// Re-export copy mode system
pub use copy_mode::{
    copy_mode_active, CopyModeCursorMarker, CopyModePlugin, CopyModeSearchResource,
    CopyModeStateResource, SelectionHighlight,
};

// Re-export scripting system
pub use scripting::{
    FusabiActionChannel, FusabiEcsBridgePlugin, FusabiNatives, RuntimeContext, ScriptEvent,
    ScriptManager, ScriptingPlugin,
};

// Re-export tutorial system
pub use tutorial::{TutorialEvent, TutorialPlugin, TutorialState, TutorialSystem};

// Re-export events system
pub use events::{
    DaemonEvent, EventsPlugin, ModalItem, NotificationLevel, PluginAction, PluginResponse,
    StatusSide, TerminalCell, TerminalRow, WindowFocusChangedEvent, WindowResizedEvent,
};

// Re-export ratatui bridge
pub use ratatui_bridge::{
    CommandPalettePlugin, CommandPaletteState, CommandSelected, PaletteCommand, RatatuiBridgePlugin,
};

// Re-export context menu system
pub use context_menu::{
    ContextMenuAction, ContextMenuItemSelected, ContextMenuPlugin, ContextMenuState,
    ShowContextMenuEvent,
};

// Re-export plugin host system
pub use plugin_host::{
    PluginNotification, PluginOverlay, PluginRegistry, PluginStatusItem, RegisteredPlugin,
    ScarabPluginHostPlugin,
};

// Re-export navigation system
pub use navigation::{
    EnterHintModeEvent, ExitHintModeEvent, FocusChangedEvent, NavAction, NavActionEvent, NavFocus,
    NavGroup, NavHint, NavMetrics, NavMetricsPlugin, NavMetricsReport, NavMode, NavState,
    NavStateRegistry, NavSystemSet, NavigationPlugin,
};

// Re-export prompt markers system
pub use prompt_markers::{
    JumpToPromptEvent, NavAnchor, PromptAnchorType, PromptGutterMarker, PromptMarkers,
    PromptMarkersPlugin, PromptZoneFocusedEvent,
};

// Re-export marketplace system
pub use marketplace::{
    InstallPluginEvent, MarketplaceEvent, MarketplaceOverlay, MarketplacePlugin, MarketplaceState,
    MarketplaceView, PluginListCache,
};

// Re-export plugin inspector (feature-gated)
#[cfg(feature = "plugin-inspector")]
pub use plugin_inspector::{PluginInspectorPlugin, PluginInspectorState};

// Re-export graphics inspector
pub use graphics_inspector::{GraphicsInspectorPlugin, GraphicsInspectorState};

// Re-export shaders and effects system
pub use shaders::{
    BlurSettings, BlurShaderNode, GlowSettings, GlowShaderNode, ScarabEffectsPlugin,
};

// Re-export diagnostics system
pub use diagnostics::{
    DiagnosticsPlugin, DiagnosticsRecorder, DiagnosticsReplay, EventData, EventType, PlaybackState,
    RecordMarkerEvent, RecordedEvent, Recording, RecordingMetadata, RecordingStats,
    ReplayControlEvent, ReplayEvent, StartRecordingEvent, StopRecordingEvent, FORMAT_VERSION,
};

// Re-export telemetry integration
pub use telemetry_integration::ScarabTelemetryPlugin;

// Re-export developer tools (debug builds only)
#[cfg(debug_assertions)]
pub use dev::{BevyInspectorPlugin, BevyInspectorState};

// Re-export accessibility system
pub use accessibility::{
    parse_accessibility_command, parse_export_command, AccessibilityCommand, AccessibilityConfig,
    AccessibilityEvent, AccessibilityPlugin, Announcement, AnnouncementPriority, AtSpiIntegration,
    ChangeTextScaleEvent, ExportFormat, ExportGridEvent, ScreenReaderAnnounceEvent,
    ScreenReaderState, TerminalExporter, ToggleHighContrastEvent,
};

// // Re-export zones system
// pub use zones::{
//     ZoneAction, ZoneEvent, ZoneManager, ZoneMembership, ZonePlugin, ZoneProperties, ZoneRole,
//     ZoneState,
// };
