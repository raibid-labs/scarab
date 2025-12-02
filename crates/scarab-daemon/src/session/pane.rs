use super::GridState;
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
        Self { x, y, width, height }
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

/// A single terminal pane with its own PTY and grid state
///
/// Each pane represents an independent terminal instance that can be
/// displayed as part of a tab's layout. The pane owns:
/// - A PTY master for I/O with the shell process
/// - A GridState for terminal cell data
/// - A viewport defining its position within the parent layout
pub struct Pane {
    pub id: PaneId,
    /// PTY master for reading/writing terminal data
    /// Wrapped in Arc<Mutex<Option<...>>> to ensure Sync.
    /// MasterPty is Send, Mutex makes it Sync.
    pub pty_master: Arc<Mutex<Option<Box<dyn portable_pty::MasterPty + Send>>>>,
    /// Grid state for this pane's terminal content
    pub grid_state: Arc<RwLock<GridState>>,
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

        // Spawn shell in PTY
        let _child = pair.slave.spawn_command(cmd)?;

        // NativePtySystem::openpty returns MasterPty which is Send on supported platforms.
        let master: Box<dyn portable_pty::MasterPty + Send> = pair.master;

        // Release slave handle in parent process
        drop(pair.slave);

        Ok(Self {
            id,
            pty_master: Arc::new(Mutex::new(Some(master))),
            grid_state: Arc::new(RwLock::new(GridState::new(cols, rows))),
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
            grid_state: Arc::new(RwLock::new(GridState::new(cols, rows))),
            viewport: Rect::full(cols, rows),
            shell,
            cwd,
            created_at: SystemTime::now(),
        }
    }

    /// Resize the pane's PTY and grid state
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

        // Update grid state dimensions
        let mut state = self.grid_state.write();
        state.cols = cols;
        state.rows = rows;

        Ok(())
    }

    /// Update the viewport position and size
    pub fn set_viewport(&mut self, viewport: Rect) {
        self.viewport = viewport;
    }

    /// Get the PTY master for I/O operations
    pub fn pty_master(&self) -> Arc<Mutex<Option<Box<dyn portable_pty::MasterPty + Send>>>> {
        Arc::clone(&self.pty_master)
    }

    /// Check if this pane has an active PTY
    pub fn has_pty(&self) -> bool {
        self.pty_master.lock().unwrap().is_some()
    }

    /// Get the pane's dimensions
    pub fn dimensions(&self) -> (u16, u16) {
        let state = self.grid_state.read();
        (state.cols, state.rows)
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
