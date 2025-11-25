#![no_std]
// This crate defines the data layout shared between Daemon and Client.
// It must be #[repr(C)] to ensure memory layout consistency across processes.

use bytemuck::{Pod, Zeroable};

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

// Plugin information for inspector
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
}

// Messages sent from Daemon to Client (Remote UI & Responses)
#[derive(Debug, Clone, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
#[archive(check_bytes)]
pub enum DaemonMessage {
    // Wrap existing session responses
    Session(SessionResponse),

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

extern crate alloc;
