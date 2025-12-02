#![no_std]
// This crate defines the data layout shared between Daemon and Client.
// It must be #[repr(C)] to ensure memory layout consistency across processes.

use bytemuck::{Pod, Zeroable};

// Safe abstraction layer for SharedState access
pub mod terminal_state;
pub use terminal_state::TerminalStateReader;

pub const SHMEM_PATH: &str = "/scarab_shm_v1";
pub const GRID_WIDTH: usize = 200;
pub const GRID_HEIGHT: usize = 100;
pub const BUFFER_SIZE: usize = GRID_WIDTH * GRID_HEIGHT;

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct Cell {
    pub char_codepoint: u32,
    pub fg: u32,           // RGBA
    pub bg: u32,           // RGBA
    pub flags: u8,         // Bold, Italic, etc.
    pub _padding: [u8; 3], // Align to 16 bytes
}

impl Default for Cell {
    fn default() -> Self {
        Self {
            char_codepoint: b' ' as u32,
            fg: 0xFFFFFFFF, // White
            bg: 0x000000FF, // Black
            flags: 0,
            _padding: [0; 3],
        }
    }
}

// A double-buffered grid state living in shared memory
#[repr(C)]
#[derive(Copy, Clone)]
pub struct SharedState {
    pub sequence_number: u64, // Atomic sequence for synchronization
    pub dirty_flag: u8,
    pub _padding1: [u8; 1], // Align to u16 boundary
    pub cursor_x: u16,
    pub cursor_y: u16,
    pub _padding2: [u8; 2], // Align to u64 boundary for cells array
    // Fixed size buffer for the "visible" screen.
    // In production, use offset pointers to a larger ring buffer.
    pub cells: [Cell; BUFFER_SIZE],
}

// Manual implementations needed for large arrays
unsafe impl Pod for SharedState {}
unsafe impl Zeroable for SharedState {}

// Log levels for plugin logging
#[derive(Debug, Clone, Copy, PartialEq, Eq, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
#[archive(check_bytes)]
pub enum LogLevel {
    Error,
    Warn,
    Info,
    Debug,
}

// Notification severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
#[archive(check_bytes)]
pub enum NotifyLevel {
    Error,
    Warning,
    Info,
    Success,
}

// Tab/Pane split direction
#[derive(Debug, Clone, Copy, PartialEq, Eq, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
#[archive(check_bytes)]
pub enum SplitDirection {
    Horizontal,
    Vertical,
}

// Menu action types from plugin API
#[derive(Debug, Clone, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
#[archive(check_bytes)]
pub enum MenuActionType {
    Command {
        command: alloc::string::String,
    },
    Remote {
        id: alloc::string::String,
    },
}

// Control messages (Sent via Socket/Pipe, not ShMem)
// Using rkyv for zero-copy serialization
#[derive(Debug, Clone, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
#[archive(check_bytes)]
pub enum ControlMessage {
    Resize {
        cols: u16,
        rows: u16,
    },
    Input {
        data: alloc::vec::Vec<u8>,
    },
    LoadPlugin {
        path: alloc::string::String,
    },
    Ping {
        timestamp: u64,
    },
    Disconnect {
        client_id: u64,
    },

    // Session management commands
    SessionCreate {
        name: alloc::string::String,
    },
    SessionDelete {
        id: alloc::string::String,
    },
    SessionList,
    SessionAttach {
        id: alloc::string::String,
    },
    SessionDetach {
        id: alloc::string::String,
    },
    SessionRename {
        id: alloc::string::String,
        new_name: alloc::string::String,
    },

    // Tab management commands
    TabCreate {
        title: Option<alloc::string::String>,
    },
    TabClose {
        tab_id: u64,
    },
    TabSwitch {
        tab_id: u64,
    },
    TabRename {
        tab_id: u64,
        new_title: alloc::string::String,
    },
    TabList,

    // Pane management commands
    PaneSplit {
        pane_id: u64,
        direction: SplitDirection,
    },
    PaneClose {
        pane_id: u64,
    },
    PaneFocus {
        pane_id: u64,
    },
    PaneResize {
        pane_id: u64,
        width: u16,
        height: u16,
    },

    // Remote UI Responses
    CommandSelected {
        id: alloc::string::String,
    },

    // Plugin inspection commands
    PluginListRequest,
    PluginEnable {
        name: alloc::string::String,
    },
    PluginDisable {
        name: alloc::string::String,
    },
    PluginReload {
        name: alloc::string::String,
    },

    // Plugin menu commands
    PluginMenuRequest {
        plugin_name: alloc::string::String,
    },
    PluginMenuExecute {
        plugin_name: alloc::string::String,
        action: MenuActionType,
    },

    // Plugin logging and notifications (sent from daemon to client)
    PluginLog {
        plugin_name: alloc::string::String,
        level: LogLevel,
        message: alloc::string::String,
    },
    PluginNotify {
        title: alloc::string::String,
        body: alloc::string::String,
        level: NotifyLevel,
    },
}

// Session response messages
#[derive(Debug, Clone, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
#[archive(check_bytes)]
pub enum SessionResponse {
    Created {
        id: alloc::string::String,
        name: alloc::string::String,
    },
    Deleted {
        id: alloc::string::String,
    },
    List {
        sessions: alloc::vec::Vec<SessionInfo>,
    },
    Attached {
        id: alloc::string::String,
    },
    Detached {
        id: alloc::string::String,
    },
    Renamed {
        id: alloc::string::String,
        new_name: alloc::string::String,
    },
    Error {
        message: alloc::string::String,
    },
}

#[derive(Debug, Clone, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
#[archive(check_bytes)]
pub struct SessionInfo {
    pub id: alloc::string::String,
    pub name: alloc::string::String,
    pub created_at: u64,
    pub last_attached: u64,
    pub attached_clients: u32,
}

// Tab information
#[derive(Debug, Clone, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
#[archive(check_bytes)]
pub struct TabInfo {
    pub id: u64,
    pub title: alloc::string::String,
    pub session_id: Option<alloc::string::String>,
    pub is_active: bool,
    pub pane_count: u32,
}

// Pane layout information
#[derive(Debug, Clone, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
#[archive(check_bytes)]
pub struct PaneInfo {
    pub id: u64,
    pub x: u16,
    pub y: u16,
    pub width: u16,
    pub height: u16,
    pub is_focused: bool,
}

// Plugin information for inspector and dock display
#[derive(Debug, Clone, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
#[archive(check_bytes)]
pub struct PluginInspectorInfo {
    pub name: alloc::string::String,
    pub version: alloc::string::String,
    pub description: alloc::string::String,
    pub author: alloc::string::String,
    pub homepage: Option<alloc::string::String>,
    pub api_version: alloc::string::String,
    pub min_scarab_version: alloc::string::String,
    pub enabled: bool,
    pub failure_count: u32,
    /// Plugin emoji for visual display (e.g., "🦠")
    pub emoji: Option<alloc::string::String>,
    /// Plugin color as hex code (e.g., "#FF5733")
    pub color: Option<alloc::string::String>,
}

// Status bar side specification
#[derive(Debug, Clone, Copy, PartialEq, Eq, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
#[archive(check_bytes)]
pub enum StatusBarSide {
    Left,
    Right,
}

// Render item for status bar content
// This is a simplified version for IPC - full version is in scarab-plugin-api
#[derive(Debug, Clone, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
#[archive(check_bytes)]
pub enum StatusRenderItem {
    Text(alloc::string::String),
    Icon(alloc::string::String),
    Foreground { r: u8, g: u8, b: u8 },
    Background { r: u8, g: u8, b: u8 },
    Bold,
    Italic,
    ResetAttributes,
    Spacer,
    Padding(u8),
    Separator(alloc::string::String),
}

// Messages sent from Daemon to Client (Remote UI & Responses)
#[derive(Debug, Clone, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
#[archive(check_bytes)]
pub enum DaemonMessage {
    // Wrap existing session responses
    Session(SessionResponse),

    // Tab state updates
    TabCreated {
        tab: TabInfo,
    },
    TabClosed {
        tab_id: u64,
    },
    TabSwitched {
        tab_id: u64,
    },
    TabListResponse {
        tabs: alloc::vec::Vec<TabInfo>,
    },

    // Pane state updates
    PaneCreated {
        pane: PaneInfo,
    },
    PaneClosed {
        pane_id: u64,
    },
    PaneFocused {
        pane_id: u64,
    },
    PaneLayoutUpdate {
        panes: alloc::vec::Vec<PaneInfo>,
    },

    // Status bar updates
    StatusBarUpdate {
        window_id: u64,
        side: StatusBarSide,
        items: alloc::vec::Vec<StatusRenderItem>,
    },

    // Remote UI Commands
    DrawOverlay {
        id: u64, // UUID-like identifier
        x: u16,
        y: u16,
        text: alloc::string::String,
        style: OverlayStyle,
    },
    ClearOverlays {
        id: Option<u64>, // None = Clear All
    },
    ShowModal {
        title: alloc::string::String,
        items: alloc::vec::Vec<ModalItem>,
    },
    HideModal,

    // Plugin inspection responses
    PluginList {
        plugins: alloc::vec::Vec<PluginInspectorInfo>,
    },
    PluginStatusChanged {
        name: alloc::string::String,
        enabled: bool,
    },
    PluginError {
        name: alloc::string::String,
        error: alloc::string::String,
    },

    // Plugin logging and notifications
    PluginLog {
        plugin_name: alloc::string::String,
        level: LogLevel,
        message: alloc::string::String,
    },
    PluginNotification {
        title: alloc::string::String,
        body: alloc::string::String,
        level: NotifyLevel,
    },

    // Plugin menu response
    PluginMenuResponse {
        plugin_name: alloc::string::String,
        menu_json: alloc::string::String, // Serialized Vec<MenuItem>
    },
    PluginMenuError {
        plugin_name: alloc::string::String,
        error: alloc::string::String,
    },

    // Theme system updates
    ThemeUpdate {
        theme_json: alloc::string::String, // Serialized Theme
    },

    // Event forwarding to clients
    Event(EventMessage),
}

/// Event message for IPC forwarding
#[derive(Debug, Clone, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
#[archive(check_bytes)]
pub struct EventMessage {
    /// Event type name
    pub event_type: alloc::string::String,
    /// Window ID if applicable
    pub window_id: Option<u64>,
    /// Pane ID if applicable
    pub pane_id: Option<u64>,
    /// Tab ID if applicable
    pub tab_id: Option<u64>,
    /// Serialized event data
    pub data: alloc::vec::Vec<u8>,
    /// Timestamp in microseconds since UNIX epoch
    pub timestamp_micros: u64,
}

#[derive(Debug, Clone, Copy, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
#[archive(check_bytes)]
pub struct OverlayStyle {
    pub fg: u32, // RGBA
    pub bg: u32, // RGBA
    pub z_index: f32,
}

impl Default for OverlayStyle {
    fn default() -> Self {
        Self {
            fg: 0xFFFFFFFF, // White
            bg: 0xFF0000FF, // Red background for high visibility by default
            z_index: 100.0,
        }
    }
}

#[derive(Debug, Clone, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
#[archive(check_bytes)]
pub struct ModalItem {
    pub id: alloc::string::String,
    pub label: alloc::string::String,
    pub description: Option<alloc::string::String>,
}

// IPC configuration constants
pub const SOCKET_PATH: &str = "/tmp/scarab-daemon.sock";
pub const MAX_MESSAGE_SIZE: usize = 8192;
pub const MAX_CLIENTS: usize = 16;
pub const RECONNECT_DELAY_MS: u64 = 100;
pub const MAX_RECONNECT_ATTEMPTS: u32 = 10;

/// Terminal display metrics shared between rendering and input systems
///
/// This provides the coordinate conversion information needed by:
/// - Mouse input handlers (screen to grid coordinate conversion)
/// - Text rendering systems (grid to screen coordinate conversion)
/// - Selection and UI overlays (coordinate mapping)
///
/// This type can be used as a Bevy Resource when the bevy feature is enabled.
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "bevy", derive(bevy_ecs::prelude::Resource))]
pub struct TerminalMetrics {
    /// Width of a single character cell in pixels
    pub cell_width: f32,
    /// Height of a single character cell in pixels
    pub cell_height: f32,
    /// Number of columns in the terminal grid
    pub columns: u16,
    /// Number of rows in the terminal grid
    pub rows: u16,
}

impl Default for TerminalMetrics {
    fn default() -> Self {
        Self {
            cell_width: 9.0,   // Typical monospace width at 15px font
            cell_height: 18.0, // Typical line height
            columns: 80,
            rows: 24,
        }
    }
}

impl TerminalMetrics {
    /// Create metrics from font size and terminal dimensions
    pub fn new(font_size: f32, line_height_multiplier: f32, columns: u16, rows: u16) -> Self {
        Self {
            cell_width: font_size * 0.6,  // Typical monospace ratio
            cell_height: font_size * line_height_multiplier,
            columns,
            rows,
        }
    }

    /// Convert screen coordinates to grid coordinates
    ///
    /// # Arguments
    /// * `screen_x` - X coordinate in pixels (from left edge)
    /// * `screen_y` - Y coordinate in pixels (from top edge, Y-down)
    ///
    /// # Returns
    /// Grid position clamped to valid bounds (col, row)
    pub fn screen_to_grid(&self, screen_x: f32, screen_y: f32) -> (u16, u16) {
        let col = (screen_x / self.cell_width).floor() as i32;
        let row = (screen_y / self.cell_height).floor() as i32;

        // Clamp to valid grid bounds
        let col = col.max(0).min((self.columns - 1) as i32) as u16;
        let row = row.max(0).min((self.rows - 1) as i32) as u16;

        (col, row)
    }

    /// Convert grid coordinates to screen coordinates (top-left of cell)
    ///
    /// # Returns
    /// Screen position in pixels (x, y)
    pub fn grid_to_screen(&self, col: u16, row: u16) -> (f32, f32) {
        let x = col as f32 * self.cell_width;
        let y = row as f32 * self.cell_height;
        (x, y)
    }

    /// Get total terminal size in pixels
    pub fn screen_size(&self) -> (f32, f32) {
        (
            self.columns as f32 * self.cell_width,
            self.rows as f32 * self.cell_height,
        )
    }
}

/// Image format specification for image protocol support
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum ImageFormat {
    /// PNG image format
    Png = 0,
    /// JPEG image format
    Jpeg = 1,
    /// GIF image format
    Gif = 2,
    /// Raw RGBA pixel data
    Rgba = 3,
}

/// Represents an image placement in the terminal grid
///
/// Images are transferred via shared memory to avoid protocol overhead.
/// This struct contains the metadata and reference to the image data.
#[derive(Debug, Clone)]
pub struct ImagePlacement {
    /// Unique identifier for this placement
    pub id: u64,
    /// Column position in terminal grid
    pub x: u16,
    /// Row position in terminal grid
    pub y: u16,
    /// Width in terminal cells
    pub width_cells: u16,
    /// Height in terminal cells
    pub height_cells: u16,
    /// Offset into shared memory image buffer
    pub shm_offset: usize,
    /// Size of image data in shared memory
    pub shm_size: usize,
    /// Image format
    pub format: ImageFormat,
}

extern crate alloc;
