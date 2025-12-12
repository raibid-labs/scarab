#![no_std]
// This crate defines the data layout shared between Daemon and Client.
// It must be #[repr(C)] to ensure memory layout consistency across processes.

use bytemuck::{Pod, Zeroable};

// Safe abstraction layer for SharedState access
pub mod terminal_state;
pub use terminal_state::TerminalStateReader;

// Semantic zones for deep shell integration
pub mod zones;
pub use zones::{CommandBlock, SemanticZone, ZoneTracker, ZoneType};

/// Default shared memory path for terminal state.
/// Can be overridden via SCARAB_SHMEM_PATH environment variable.
pub const SHMEM_PATH: &str = "/scarab_shm_v1";

/// Environment variable to override the shared memory path.
/// Useful for sandboxed environments where /dev/shm is not writable.
pub const SHMEM_PATH_ENV: &str = "SCARAB_SHMEM_PATH";

/// Environment variable to override the image shared memory path.
pub const IMAGE_SHMEM_PATH_ENV: &str = "SCARAB_IMAGE_SHMEM_PATH";
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
            fg: 0xFFA8DF5A, // Slime green foreground (ARGB: #a8df5a)
            bg: 0xFF0D1208, // Slime dark background (ARGB: #0d1208)
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
    pub error_mode: u8, // 0 = normal mode, 1 = error mode (PTY/SHM unavailable)
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

// Image buffer constants
/// Maximum number of concurrent image placements
pub const MAX_IMAGES: usize = 64;

/// Maximum total image buffer size (16MB)
pub const IMAGE_BUFFER_SIZE: usize = 16 * 1024 * 1024;

/// Default shared memory path for image buffer (separate from terminal state).
/// Can be overridden via SCARAB_IMAGE_SHMEM_PATH environment variable.
pub const IMAGE_SHMEM_PATH: &str = "/scarab_img_shm_v1";

/// Image placement metadata for shared memory
#[repr(C)]
#[derive(Copy, Clone)]
pub struct SharedImagePlacement {
    /// Unique identifier for this placement
    pub image_id: u64,
    /// Column position in terminal grid
    pub x: u16,
    /// Row position in terminal grid
    pub y: u16,
    /// Width in terminal cells
    pub width_cells: u16,
    /// Height in terminal cells
    pub height_cells: u16,
    /// Pixel width of decoded image
    pub pixel_width: u32,
    /// Pixel height of decoded image
    pub pixel_height: u32,
    /// Offset into blob_data buffer
    pub blob_offset: u32,
    /// Size of image data in bytes
    pub blob_size: u32,
    /// Image format (0=PNG, 1=JPEG, 2=GIF, 3=RGBA)
    pub format: u8,
    /// Flags (bit 0: valid/active)
    pub flags: u8,
    /// Padding for alignment
    pub _padding: [u8; 6],
}

// Manual Pod/Zeroable implementations
unsafe impl Pod for SharedImagePlacement {}
unsafe impl Zeroable for SharedImagePlacement {}

impl SharedImagePlacement {
    /// Check if this placement is valid/active
    pub const fn is_valid(&self) -> bool {
        (self.flags & 0x01) != 0
    }

    /// Mark this placement as valid/active
    pub fn set_valid(&mut self) {
        self.flags |= 0x01;
    }

    /// Mark this placement as invalid/inactive
    pub fn set_invalid(&mut self) {
        self.flags &= !0x01;
    }
}

/// Shared memory buffer for image data
#[repr(C)]
#[derive(Copy, Clone)]
pub struct SharedImageBuffer {
    /// Sequence number for synchronization (increment on any change)
    pub sequence_number: u64,
    /// Number of active placements
    pub count: u32,
    /// Next blob write offset (circular buffer pointer)
    pub next_blob_offset: u32,
    /// Image placement metadata array
    pub placements: [SharedImagePlacement; MAX_IMAGES],
    /// Raw image blob data (circular buffer)
    pub blob_data: [u8; IMAGE_BUFFER_SIZE],
}

// Manual Pod/Zeroable implementations for large array
unsafe impl Pod for SharedImageBuffer {}
unsafe impl Zeroable for SharedImageBuffer {}

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
    Command { command: alloc::string::String },
    Remote { id: alloc::string::String },
}

// Navigation focusable action types
#[derive(Debug, Clone, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
#[archive(check_bytes)]
pub enum NavFocusableAction {
    /// Open a URL in the default browser
    OpenUrl(alloc::string::String),
    /// Open a file in the configured editor
    OpenFile(alloc::string::String),
    /// Custom plugin-defined action
    Custom(alloc::string::String),
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
    /// Focus the next pane in the current tab (for navigation)
    PaneFocusNext,
    /// Focus the previous pane in the current tab (for navigation)
    PaneFocusPrev,

    // Tab navigation commands
    /// Switch to the next tab
    TabNext,
    /// Switch to the previous tab
    TabPrev,

    // Mouse input commands
    /// Send mouse click event to the terminal
    MouseClick {
        col: u16,
        row: u16,
        button: u8,
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

    // Navigation API commands (sent from plugins via client to daemon/client)
    NavEnterHintMode {
        plugin_name: alloc::string::String,
    },
    NavExitMode {
        plugin_name: alloc::string::String,
    },
    NavRegisterFocusable {
        plugin_name: alloc::string::String,
        x: u16,
        y: u16,
        width: u16,
        height: u16,
        label: alloc::string::String,
        action: NavFocusableAction,
    },
    NavUnregisterFocusable {
        plugin_name: alloc::string::String,
        focusable_id: u64,
    },

    // Semantic zone commands (deep shell integration)
    /// Request semantic zones update from daemon
    ZonesRequest,

    /// Copy the output from the last completed command
    CopyLastOutput,

    /// Select a specific zone by ID
    SelectZone {
        zone_id: u64,
    },

    /// Extract text from a zone
    ExtractZoneText {
        zone_id: u64,
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
    /// Verification status
    pub verification: PluginVerificationStatus,
}

/// Verification status for plugins (zero-copy compatible)
#[derive(Debug, Clone, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
#[archive(check_bytes)]
pub enum PluginVerificationStatus {
    /// Plugin was verified with valid GPG signature
    Verified {
        key_fingerprint: alloc::string::String,
        signature_timestamp: u64,
    },
    /// Plugin checksum was verified but no signature
    ChecksumOnly { checksum: alloc::string::String },
    /// Plugin was installed without verification
    Unverified { warning: alloc::string::String },
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

    // Shell prompt markers update (OSC 133 shell integration)
    PromptMarkersUpdate {
        /// List of current prompt markers from daemon
        markers: alloc::vec::Vec<PromptMarkerInfo>,
    },

    // Semantic zones update (deep shell integration)
    SemanticZonesUpdate {
        /// List of current semantic zones (prompt, input, output regions)
        zones: alloc::vec::Vec<SemanticZone>,
    },

    // Command blocks update (grouped command sequences)
    CommandBlocksUpdate {
        /// List of completed command blocks
        blocks: alloc::vec::Vec<CommandBlock>,
    },

    /// Response to ExtractZoneText with the zone's text content
    ZoneTextExtracted {
        zone_id: u64,
        text: alloc::string::String,
    },

    // Event forwarding to clients
    Event(EventMessage),

    // Navigation API responses
    NavFocusableRegistered {
        plugin_name: alloc::string::String,
        focusable_id: u64,
    },
    NavFocusableUnregistered {
        plugin_name: alloc::string::String,
        focusable_id: u64,
    },
    NavModeEntered {
        plugin_name: alloc::string::String,
    },
    NavModeExited {
        plugin_name: alloc::string::String,
    },
    /// Forward focusable registration from daemon to client
    NavRegisterFocusable {
        plugin_name: alloc::string::String,
        x: u16,
        y: u16,
        width: u16,
        height: u16,
        label: alloc::string::String,
        action: NavFocusableAction,
    },
    /// Forward focusable unregistration from daemon to client
    NavUnregisterFocusable {
        plugin_name: alloc::string::String,
        focusable_id: u64,
    },
    /// Spawn an overlay at a given position
    SpawnOverlay {
        plugin_name: alloc::string::String,
        overlay_id: u64,
        x: u16,
        y: u16,
        content: alloc::string::String,
        style: OverlayStyle,
    },
    /// Remove a previously spawned overlay
    RemoveOverlay {
        plugin_name: alloc::string::String,
        overlay_id: u64,
    },
    /// Add a status bar item
    AddStatusItem {
        plugin_name: alloc::string::String,
        item_id: u64,
        label: alloc::string::String,
        content: alloc::string::String,
        priority: i32,
    },
    /// Remove a status bar item
    RemoveStatusItem {
        plugin_name: alloc::string::String,
        item_id: u64,
    },
    /// Trigger prompt jump navigation
    PromptJump {
        plugin_name: alloc::string::String,
        direction: PromptJumpDirection,
    },
    /// Apply a theme by name
    ThemeApply {
        theme_name: alloc::string::String,
    },
    /// Set a specific palette color
    PaletteColorSet {
        color_name: alloc::string::String,
        value: alloc::string::String,
    },
    /// Response with current theme info
    ThemeInfoResponse {
        plugin_name: alloc::string::String,
        theme_name: alloc::string::String,
    },
}

/// Direction for prompt jump navigation
#[derive(Debug, Clone, Copy, PartialEq, Eq, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
#[archive(check_bytes)]
pub enum PromptJumpDirection {
    Up,
    Down,
    First,
    Last,
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

#[derive(Debug, Clone, Copy, PartialEq, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
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
            cell_width: font_size * 0.6, // Typical monospace ratio
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

/// Shell prompt marker for IPC (OSC 133 shell integration)
///
/// This is a simplified, serializable version of the daemon's internal
/// PromptMarker type. Used to communicate shell integration markers
/// from daemon to client for features like:
/// - Semantic prompt navigation (jump to previous/next prompt)
/// - Command output extraction
/// - Command duration tracking
#[derive(Debug, Clone, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
#[archive(check_bytes)]
pub struct PromptMarkerInfo {
    /// Marker type encoded as u8:
    /// - 0 = PromptStart (OSC 133;A)
    /// - 1 = CommandStart (OSC 133;B)
    /// - 2 = CommandExecuted (OSC 133;C)
    /// - 3 = CommandFinished (OSC 133;D)
    pub marker_type: u8,
    /// Absolute line number in scrollback
    pub line: u32,
    /// Exit code (only valid for CommandFinished markers)
    pub exit_code: Option<i32>,
    /// Timestamp in microseconds since UNIX epoch
    pub timestamp_micros: u64,
}

impl PromptMarkerInfo {
    /// Create a PromptStart marker
    pub fn prompt_start(line: u32, timestamp_micros: u64) -> Self {
        Self {
            marker_type: 0,
            line,
            exit_code: None,
            timestamp_micros,
        }
    }

    /// Create a CommandStart marker
    pub fn command_start(line: u32, timestamp_micros: u64) -> Self {
        Self {
            marker_type: 1,
            line,
            exit_code: None,
            timestamp_micros,
        }
    }

    /// Create a CommandExecuted marker
    pub fn command_executed(line: u32, timestamp_micros: u64) -> Self {
        Self {
            marker_type: 2,
            line,
            exit_code: None,
            timestamp_micros,
        }
    }

    /// Create a CommandFinished marker with exit code
    pub fn command_finished(line: u32, exit_code: i32, timestamp_micros: u64) -> Self {
        Self {
            marker_type: 3,
            line,
            exit_code: Some(exit_code),
            timestamp_micros,
        }
    }

    /// Check if this is a PromptStart marker
    pub fn is_prompt_start(&self) -> bool {
        self.marker_type == 0
    }

    /// Check if this is a CommandStart marker
    pub fn is_command_start(&self) -> bool {
        self.marker_type == 1
    }

    /// Check if this is a CommandExecuted marker
    pub fn is_command_executed(&self) -> bool {
        self.marker_type == 2
    }

    /// Check if this is a CommandFinished marker
    pub fn is_command_finished(&self) -> bool {
        self.marker_type == 3
    }
}

extern crate alloc;
