mod commands;
mod manager;
pub mod pane;
mod store;
pub mod tab;

pub use commands::{handle_pane_command, handle_session_command, handle_tab_command, TabCommandResult};
pub use manager::{Session, SessionManager};
pub use pane::{Pane, PaneId, Rect};
pub use store::SessionStore;
pub use tab::{Tab, TabId, SplitDirection};

// Re-export TerminalState for pane usage
pub use crate::vte::TerminalState;

/// Client identifier
pub type ClientId = u64;

/// Session identifier (UUID)
pub type SessionId = String;
