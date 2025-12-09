use crate::vte::TerminalState;
use anyhow::Result;
use parking_lot::RwLock;
use portable_pty::{CommandBuilder, NativePtySystem, PtySize, PtySystem};
use std::sync::{Arc, Mutex};
use std::time::SystemTime;

/// Unique identifier for a pane
pub type PaneId = u64;

/// Viewport rectangle for pane positioning in the layout
#[derive(Clone, Copy, Debug, Default)]
pub struct Rect {
    pub x: u16,
    pub y: u16,
    pub width: u16,
    pub height: u16,
}

impl Rect {
    pub fn new(x: u16, y: u16, width: u16, height: u16) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    /// Create a full-screen rect for the given dimensions
    pub fn full(cols: u16, rows: u16) -> Self {
        Self {
            x: 0,
            y: 0,
            width: cols,
            height: rows,
        }
    }
}

/// A single terminal pane with its own PTY and terminal state
///
/// Each pane represents an independent terminal instance that can be
/// displayed as part of a tab's layout. The pane owns:
/// - A PTY master for I/O with the shell process
/// - A PTY writer for sending input to the shell
/// - A TerminalState with its own Grid for VTE parsing
/// - A viewport defining its position within the parent layout
pub struct Pane {
    pub id: PaneId,
    /// PTY master for reading terminal output
    /// Wrapped in Arc<Mutex<Option<...>>> to ensure Sync.
    /// MasterPty is Send, Mutex makes it Sync.
    pub pty_master: Arc<Mutex<Option<Box<dyn portable_pty::MasterPty + Send>>>>,
    /// PTY writer for sending input to the shell
    pub pty_writer: Arc<Mutex<Option<Box<dyn std::io::Write + Send>>>>,
    /// Terminal state with its own grid (VTE parser + cell buffer)
    pub terminal_state: Arc<RwLock<TerminalState>>,
    /// Position and size within the parent layout
    pub viewport: Rect,
    /// Shell command running in this pane
    pub shell: String,
    /// Working directory for this pane
    pub cwd: Option<String>,
    /// Timestamp when pane was created
    pub created_at: SystemTime,
}

// Pane is Sync because all interior mutability is behind locks
unsafe impl Sync for Pane {}

impl Pane {
    /// Create a new pane with a PTY running the specified shell
    pub fn new(id: PaneId, shell: &str, cols: u16, rows: u16, cwd: Option<String>) -> Result<Self> {
        if std::env::var("SCARAB_FORCE_PTY_FAIL")
            .map(|v| v == "1")
            .unwrap_or(false)
        {
            anyhow::bail!("PTY creation forced to fail via SCARAB_FORCE_PTY_FAIL=1");
        }

        let pty_system = NativePtySystem::default();
        let pair = pty_system.openpty(PtySize {
            rows,
            cols,
            pixel_width: 0,
            pixel_height: 0,
        })?;

        // Build shell command
        let mut cmd = CommandBuilder::new(shell);

        // Set working directory if provided, otherwise use HOME
        if let Some(ref dir) = cwd {
            cmd.cwd(dir);
        } else if let Ok(home) = std::env::var("HOME") {
            cmd.cwd(home);
        }

        // Set environment variable for Navigation Protocol
        cmd.env("SCARAB_NAV_SOCKET", "/tmp/scarab-nav.sock");

        // Set TERM so the shell knows what terminal capabilities we support
        cmd.env("TERM", "xterm-256color");

        // Spawn shell in PTY
        let _child = pair.slave.spawn_command(cmd)?;

        // Get the writer from the master before storing it
        let writer = pair.master.take_writer()?;

        // NativePtySystem::openpty returns MasterPty which is Send on supported platforms.
        let master: Box<dyn portable_pty::MasterPty + Send> = pair.master;

        // Release slave handle in parent process
        drop(pair.slave);

        Ok(Self {
            id,
            pty_master: Arc::new(Mutex::new(Some(master))),
            pty_writer: Arc::new(Mutex::new(Some(writer))),
            terminal_state: Arc::new(RwLock::new(TerminalState::new(cols, rows))),
            viewport: Rect::full(cols, rows),
            shell: shell.to_string(),
            cwd,
            created_at: SystemTime::now(),
        })
    }

    /// Create a pane without spawning a PTY (for restoration)
    pub fn restore(id: PaneId, cols: u16, rows: u16, shell: String, cwd: Option<String>) -> Self {
        Self {
            id,
            pty_master: Arc::new(Mutex::new(None)),
            pty_writer: Arc::new(Mutex::new(None)),
            terminal_state: Arc::new(RwLock::new(TerminalState::new(cols, rows))),
            viewport: Rect::full(cols, rows),
            shell,
            cwd,
            created_at: SystemTime::now(),
        }
    }

    /// Resize the pane's PTY and terminal state
    pub fn resize(&self, cols: u16, rows: u16) -> Result<()> {
        // Resize PTY
        if let Some(ref master) = *self.pty_master.lock().unwrap() {
            master.resize(PtySize {
                rows,
                cols,
                pixel_width: 0,
                pixel_height: 0,
            })?;
        }

        // Resize terminal state (includes grid)
        let mut state = self.terminal_state.write();
        state.resize(cols, rows);

        Ok(())
    }

    /// Update the viewport position and size
    pub fn set_viewport(&mut self, viewport: Rect) {
        self.viewport = viewport;
    }

    /// Get the PTY master for I/O operations (reading output)
    pub fn pty_master(&self) -> Arc<Mutex<Option<Box<dyn portable_pty::MasterPty + Send>>>> {
        Arc::clone(&self.pty_master)
    }

    /// Get the PTY writer for sending input to the shell
    pub fn pty_writer(&self) -> Arc<Mutex<Option<Box<dyn std::io::Write + Send>>>> {
        Arc::clone(&self.pty_writer)
    }

    /// Check if this pane has an active PTY
    pub fn has_pty(&self) -> bool {
        self.pty_master.lock().unwrap().is_some()
    }

    /// Get the pane's dimensions
    pub fn dimensions(&self) -> (u16, u16) {
        let state = self.terminal_state.read();
        state.dimensions()
    }

    /// Process PTY output through this pane's terminal state
    pub fn process_output(&self, data: &[u8]) {
        let mut state = self.terminal_state.write();
        state.process_output(data);
    }

    /// Get a reference to the terminal state for blitting
    pub fn terminal_state(&self) -> &Arc<RwLock<TerminalState>> {
        &self.terminal_state
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pane_creation() {
        let pane = Pane::new(1, "bash", 80, 24, None).unwrap();
        assert_eq!(pane.id, 1);
        assert!(pane.has_pty());
        assert_eq!(pane.dimensions(), (80, 24));
    }

    #[test]
    fn test_pane_viewport() {
        let mut pane = Pane::restore(1, 80, 24, "bash".to_string(), None);
        assert_eq!(pane.viewport.x, 0);
        assert_eq!(pane.viewport.y, 0);

        pane.set_viewport(Rect::new(10, 5, 40, 12));
        assert_eq!(pane.viewport.x, 10);
        assert_eq!(pane.viewport.y, 5);
        assert_eq!(pane.viewport.width, 40);
        assert_eq!(pane.viewport.height, 12);
    }

    #[test]
    fn test_rect_full() {
        let rect = Rect::full(120, 40);
        assert_eq!(rect.x, 0);
        assert_eq!(rect.y, 0);
        assert_eq!(rect.width, 120);
        assert_eq!(rect.height, 40);
    }
}
