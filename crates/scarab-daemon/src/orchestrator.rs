//! Pane Orchestrator - Parallel PTY Management
//!
//! This module manages parallel reading from multiple PTYs across all panes.
//! Each pane gets its own reader task that:
//! - Reads from its PTY asynchronously
//! - Updates its TerminalState with VTE parsing
//! - Runs independently of whether the pane is active
//!
//! The compositor (in main.rs) only needs to blit the active pane to SharedState.

use crate::session::{Pane, PaneId, SessionManager};
use parking_lot::RwLock;
use std::collections::HashMap;
use std::io::Read;
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;

/// Message types for pane orchestration
#[derive(Debug)]
pub enum OrchestratorMessage {
    /// A pane was created, start reading from it
    PaneCreated(PaneId),
    /// A pane was destroyed, stop reading from it
    PaneDestroyed(PaneId),
    /// Shutdown all readers
    Shutdown,
}

/// Manages parallel PTY readers for all panes
pub struct PaneOrchestrator {
    /// Session manager reference
    session_manager: Arc<SessionManager>,
    /// Active reader tasks (PaneId -> JoinHandle)
    reader_tasks: Arc<RwLock<HashMap<PaneId, JoinHandle<()>>>>,
    /// Channel to send orchestration commands
    command_tx: mpsc::UnboundedSender<OrchestratorMessage>,
    /// Channel to receive orchestration commands
    command_rx: Option<mpsc::UnboundedReceiver<OrchestratorMessage>>,
}

impl PaneOrchestrator {
    /// Create a new orchestrator
    pub fn new(session_manager: Arc<SessionManager>) -> Self {
        let (command_tx, command_rx) = mpsc::unbounded_channel();

        Self {
            session_manager,
            reader_tasks: Arc::new(RwLock::new(HashMap::new())),
            command_tx,
            command_rx: Some(command_rx),
        }
    }

    /// Get the command sender for external use
    pub fn command_sender(&self) -> mpsc::UnboundedSender<OrchestratorMessage> {
        self.command_tx.clone()
    }

    /// Start the orchestrator - spawns reader tasks for all existing panes
    /// and listens for pane lifecycle events
    pub async fn run(mut self) {
        // Take the receiver (can only run once)
        let mut command_rx = self.command_rx.take().expect("Orchestrator already running");

        // Spawn readers for all existing panes
        self.spawn_all_readers().await;

        log::info!("PaneOrchestrator started with {} panes", self.reader_tasks.read().len());

        // Listen for lifecycle events
        while let Some(msg) = command_rx.recv().await {
            match msg {
                OrchestratorMessage::PaneCreated(pane_id) => {
                    log::debug!("Pane {} created, spawning reader", pane_id);
                    self.spawn_reader_for_pane(pane_id).await;
                }
                OrchestratorMessage::PaneDestroyed(pane_id) => {
                    log::debug!("Pane {} destroyed, stopping reader", pane_id);
                    self.stop_reader_for_pane(pane_id).await;
                }
                OrchestratorMessage::Shutdown => {
                    log::info!("Orchestrator shutting down");
                    self.shutdown_all_readers().await;
                    break;
                }
            }
        }
    }

    /// Spawn reader tasks for all existing panes across all sessions
    async fn spawn_all_readers(&self) {
        // Get all sessions
        let sessions = self.session_manager.list_sessions();

        for (session_id, _, _, _, _) in sessions {
            if let Some(session) = self.session_manager.get_session(&session_id) {
                // Get all panes across all tabs in this session
                let panes = session.all_panes();

                for pane in panes {
                    self.spawn_reader_for_pane_arc(pane).await;
                }
            }
        }
    }

    /// Spawn a reader task for a specific pane by ID
    async fn spawn_reader_for_pane(&self, pane_id: PaneId) {
        // Find the pane across all sessions
        let sessions = self.session_manager.list_sessions();

        for (session_id, _, _, _, _) in sessions {
            if let Some(session) = self.session_manager.get_session(&session_id) {
                // Search all panes in this session
                for pane in session.all_panes() {
                    if pane.id == pane_id {
                        self.spawn_reader_for_pane_arc(pane).await;
                        return;
                    }
                }
            }
        }
    }

    /// Spawn a reader task for a pane Arc
    async fn spawn_reader_for_pane_arc(&self, pane: Arc<Pane>) {
        let pane_id = pane.id;

        // Check if we already have a reader for this pane
        if self.reader_tasks.read().contains_key(&pane_id) {
            log::debug!("Reader already exists for pane {}", pane_id);
            return;
        }

        // Spawn the reader task
        let handle = tokio::spawn(Self::pane_reader_task(pane));

        self.reader_tasks.write().insert(pane_id, handle);
        log::debug!("Spawned reader for pane {}", pane_id);
    }

    /// The reader task for a single pane
    /// Reads from PTY and updates TerminalState continuously
    async fn pane_reader_task(pane: Arc<Pane>) {
        let pane_id = pane.id;
        log::debug!("Reader task started for pane {}", pane_id);

        loop {
            // Get the PTY master
            let pty_master_arc = pane.pty_master();

            // Read from PTY in a blocking task
            let read_result = tokio::task::spawn_blocking({
                let pty_arc = Arc::clone(&pty_master_arc);
                move || {
                    let mut buf = [0u8; 4096];
                    let pty_lock = pty_arc.lock().unwrap();
                    if let Some(ref master) = *pty_lock {
                        match master.try_clone_reader() {
                            Ok(mut reader) => reader.read(&mut buf).map(|n| (n, buf)),
                            Err(e) => Err(std::io::Error::new(
                                std::io::ErrorKind::Other,
                                e.to_string(),
                            )),
                        }
                    } else {
                        // No PTY - signal EOF
                        Ok((0, buf))
                    }
                }
            })
            .await;

            match read_result {
                Ok(Ok((n, buf))) if n > 0 => {
                    let data = &buf[..n];

                    // Process output through the pane's VTE parser
                    // This updates the pane's local grid
                    let terminal_state_arc = pane.terminal_state();
                    let mut terminal_state = terminal_state_arc.write();
                    terminal_state.process_output(data);

                    // Note: We do NOT blit here - the compositor does that
                    // for the active pane only
                }
                Ok(Ok(_)) => {
                    // EOF - shell exited
                    log::info!("PTY EOF for pane {}, shell exited", pane_id);
                    // Wait a bit before retrying (in case pane is restarted)
                    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
                }
                Ok(Err(e)) => {
                    log::warn!("PTY read error for pane {}: {}", pane_id, e);
                    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                }
                Err(e) => {
                    log::error!("Reader task panic for pane {}: {}", pane_id, e);
                    break;
                }
            }
        }

        log::debug!("Reader task ended for pane {}", pane_id);
    }

    /// Stop the reader task for a specific pane
    async fn stop_reader_for_pane(&self, pane_id: PaneId) {
        if let Some(handle) = self.reader_tasks.write().remove(&pane_id) {
            handle.abort();
            log::debug!("Stopped reader for pane {}", pane_id);
        }
    }

    /// Shutdown all reader tasks
    async fn shutdown_all_readers(&self) {
        let mut tasks = self.reader_tasks.write();
        for (pane_id, handle) in tasks.drain() {
            handle.abort();
            log::debug!("Stopped reader for pane {}", pane_id);
        }
    }
}

/// Extension trait for Session to get all panes
pub trait SessionPaneIterator {
    /// Get all panes across all tabs in this session
    fn all_panes(&self) -> Vec<Arc<Pane>>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_orchestrator_creation() {
        // This would require a full session manager setup
        // For now, just verify the types compile
    }
}
