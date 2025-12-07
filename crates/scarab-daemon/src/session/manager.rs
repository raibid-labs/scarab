use super::pane::{Pane, PaneId};
use super::tab::{SplitDirection, Tab, TabId};
use super::{ClientId, SessionId, SessionStore, TerminalState};
use anyhow::{bail, Result};
use parking_lot::RwLock;
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

/// A terminal session containing one or more tabs, each with panes
///
/// The session is the top-level container that manages:
/// - Multiple tabs (like browser tabs)
/// - Each tab contains one or more panes (split views)
/// - Tracks the active tab and routes input to the active pane
pub struct Session {
    pub id: SessionId,
    pub name: String,
    /// All tabs in this session
    tabs: RwLock<HashMap<TabId, Tab>>,
    /// The currently active tab
    active_tab_id: RwLock<TabId>,
    /// Next tab ID to assign
    next_tab_id: RwLock<TabId>,
    /// Session creation timestamp
    pub created_at: SystemTime,
    /// Last time a client attached
    pub last_attached: Arc<RwLock<SystemTime>>,
    /// Currently attached clients
    pub attached_clients: Arc<RwLock<HashSet<ClientId>>>,
    /// Default shell for new panes
    default_shell: String,
    /// Default terminal dimensions
    default_cols: u16,
    default_rows: u16,
}

// Session is Sync because all interior mutability is behind locks
unsafe impl Sync for Session {}

impl Session {
    /// Create a new session with a single tab containing one pane
    pub fn new(name: String, cols: u16, rows: u16) -> Result<Self> {
        Self::with_shell(name, cols, rows, "bash")
    }

    /// Create a new session with a specific shell
    pub fn with_shell(name: String, cols: u16, rows: u16, shell: &str) -> Result<Self> {
        let id = Uuid::new_v4().to_string();
        let now = SystemTime::now();

        // Create initial tab with a single pane
        let tab = Tab::new(1, "Tab 1".to_string(), shell, cols, rows)?;
        let mut tabs = HashMap::new();
        tabs.insert(1, tab);

        Ok(Self {
            id,
            name,
            tabs: RwLock::new(tabs),
            active_tab_id: RwLock::new(1),
            next_tab_id: RwLock::new(2),
            created_at: now,
            last_attached: Arc::new(RwLock::new(now)),
            attached_clients: Arc::new(RwLock::new(HashSet::new())),
            default_shell: shell.to_string(),
            default_cols: cols,
            default_rows: rows,
        })
    }

    /// Restore session from persisted data (without PTY initially)
    pub fn restore(
        id: SessionId,
        name: String,
        created_at: SystemTime,
        last_attached: SystemTime,
    ) -> Self {
        Self {
            id,
            name,
            tabs: RwLock::new(HashMap::new()),
            active_tab_id: RwLock::new(0),
            next_tab_id: RwLock::new(1),
            created_at,
            last_attached: Arc::new(RwLock::new(last_attached)),
            attached_clients: Arc::new(RwLock::new(HashSet::new())),
            default_shell: "bash".to_string(),
            default_cols: 80,
            default_rows: 24,
        }
    }

    /// Ensure the session has at least one tab with a PTY
    ///
    /// Called after restoration to spawn a new shell for restored sessions.
    /// Returns Ok(true) if a tab was created, Ok(false) if tabs already existed.
    pub fn ensure_default_tab(&self, shell: &str, cols: u16, rows: u16) -> Result<bool> {
        // Check if we already have tabs
        if self.tab_count() > 0 {
            return Ok(false);
        }

        // Create initial tab with a single pane
        let tab = Tab::new(1, "Tab 1".to_string(), shell, cols, rows)?;

        {
            let mut tabs = self.tabs.write();
            tabs.insert(1, tab);
        }

        {
            let mut active = self.active_tab_id.write();
            *active = 1;
        }

        {
            let mut next_id = self.next_tab_id.write();
            *next_id = 2;
        }

        log::info!(
            "Restored session {} with new tab and PTY ({}x{})",
            self.id,
            cols,
            rows
        );
        Ok(true)
    }

    // ==================== Tab Management ====================

    /// Create a new tab
    pub fn create_tab(&self, title: Option<String>) -> Result<TabId> {
        let tab_id = {
            let mut next_id = self.next_tab_id.write();
            let id = *next_id;
            *next_id += 1;
            id
        };

        let title = title.unwrap_or_else(|| format!("Tab {}", tab_id));
        let tab = Tab::new(
            tab_id,
            title,
            &self.default_shell,
            self.default_cols,
            self.default_rows,
        )?;

        {
            let mut tabs = self.tabs.write();
            tabs.insert(tab_id, tab);
        }

        // If no active tab, make this one active
        {
            let mut active = self.active_tab_id.write();
            if *active == 0 {
                *active = tab_id;
            }
        }

        log::info!("Created tab {} in session {}", tab_id, self.id);
        Ok(tab_id)
    }

    /// Close a tab
    /// Returns the list of pane IDs that were destroyed
    pub fn close_tab(&self, tab_id: TabId) -> Result<Vec<PaneId>> {
        let mut tabs = self.tabs.write();

        if tabs.len() <= 1 {
            bail!("Cannot close the last tab in a session");
        }

        // Get the pane IDs before removing the tab
        let destroyed_panes: Vec<PaneId> = if let Some(tab) = tabs.get(&tab_id) {
            tab.pane_ids()
        } else {
            Vec::new()
        };

        tabs.remove(&tab_id);

        // Update active tab if needed
        let mut active = self.active_tab_id.write();
        if *active == tab_id {
            *active = *tabs.keys().next().unwrap_or(&0);
        }

        log::info!(
            "Closed tab {} in session {} (destroyed {} panes)",
            tab_id,
            self.id,
            destroyed_panes.len()
        );
        Ok(destroyed_panes)
    }

    /// Switch to a different tab
    pub fn switch_tab(&self, tab_id: TabId) -> Result<()> {
        let tabs = self.tabs.read();
        if !tabs.contains_key(&tab_id) {
            bail!("Tab {} not found in session {}", tab_id, self.id);
        }

        *self.active_tab_id.write() = tab_id;
        log::info!("Switched to tab {} in session {}", tab_id, self.id);
        Ok(())
    }

    /// Rename a tab
    pub fn rename_tab(&self, tab_id: TabId, new_title: String) -> Result<()> {
        let mut tabs = self.tabs.write();
        if let Some(tab) = tabs.get_mut(&tab_id) {
            tab.title = new_title;
            Ok(())
        } else {
            bail!("Tab {} not found", tab_id)
        }
    }

    /// Get the active tab ID
    pub fn active_tab_id(&self) -> TabId {
        *self.active_tab_id.read()
    }

    /// Get tab count
    pub fn tab_count(&self) -> usize {
        self.tabs.read().len()
    }

    /// List all tabs
    pub fn list_tabs(&self) -> Vec<(TabId, String, bool, usize)> {
        let tabs = self.tabs.read();
        let active_id = *self.active_tab_id.read();

        tabs.values()
            .map(|tab| {
                (
                    tab.id,
                    tab.title.clone(),
                    tab.id == active_id,
                    tab.pane_count(),
                )
            })
            .collect()
    }

    // ==================== Pane Management ====================

    /// Split the active pane in the active tab
    pub fn split_pane(&self, direction: SplitDirection) -> Result<PaneId> {
        let mut tabs = self.tabs.write();
        let active_tab_id = *self.active_tab_id.read();

        if let Some(tab) = tabs.get_mut(&active_tab_id) {
            tab.split_pane(direction, &self.default_shell)
        } else {
            bail!("No active tab")
        }
    }

    /// Close a pane in the active tab
    pub fn close_pane(&self, pane_id: PaneId) -> Result<()> {
        let mut tabs = self.tabs.write();
        let active_tab_id = *self.active_tab_id.read();

        if let Some(tab) = tabs.get_mut(&active_tab_id) {
            tab.close_pane(pane_id)
        } else {
            bail!("No active tab")
        }
    }

    /// Focus a specific pane in the active tab
    pub fn focus_pane(&self, pane_id: PaneId) -> Result<()> {
        let mut tabs = self.tabs.write();
        let active_tab_id = *self.active_tab_id.read();

        if let Some(tab) = tabs.get_mut(&active_tab_id) {
            tab.set_active_pane(pane_id)
        } else {
            bail!("No active tab")
        }
    }

    /// Get the active pane (the focused pane in the active tab)
    pub fn get_active_pane(&self) -> Option<Arc<Pane>> {
        let tabs = self.tabs.read();
        let active_tab_id = *self.active_tab_id.read();

        tabs.get(&active_tab_id)
            .and_then(|tab| tab.get_active_pane())
    }

    /// Get the active pane's terminal state for VTE processing
    pub fn get_active_terminal_state(&self) -> Option<Arc<RwLock<TerminalState>>> {
        self.get_active_pane()
            .map(|pane| Arc::clone(&pane.terminal_state))
    }

    /// Get the active pane's PTY master for reading output
    pub fn get_active_pty_master(
        &self,
    ) -> Option<Arc<Mutex<Option<Box<dyn portable_pty::MasterPty + Send>>>>> {
        self.get_active_pane().map(|pane| pane.pty_master())
    }

    /// Get the active pane's PTY writer for sending input
    pub fn get_active_pty_writer(
        &self,
    ) -> Option<Arc<Mutex<Option<Box<dyn std::io::Write + Send>>>>> {
        self.get_active_pane().map(|pane| pane.pty_writer())
    }

    // ==================== Legacy Compatibility ====================

    /// Get the terminal state (returns active pane's terminal state for compatibility)
    pub fn terminal_state(&self) -> Arc<RwLock<TerminalState>> {
        self.get_active_terminal_state()
            .unwrap_or_else(|| Arc::new(RwLock::new(TerminalState::new(80, 24))))
    }

    /// Get PTY master (returns active pane's PTY for compatibility)
    #[allow(dead_code)]
    pub fn pty_master(&self) -> Arc<Mutex<Option<Box<dyn portable_pty::MasterPty + Send>>>> {
        self.get_active_pty_master()
            .unwrap_or_else(|| Arc::new(Mutex::new(None)))
    }

    /// Resize the active tab/pane
    pub fn resize(&self, cols: u16, rows: u16) -> Result<()> {
        let tabs = self.tabs.read();
        let active_tab_id = *self.active_tab_id.read();

        if let Some(tab) = tabs.get(&active_tab_id) {
            tab.resize(cols, rows)?;
        }
        Ok(())
    }

    /// Get all panes across all tabs in this session
    pub fn all_panes(&self) -> Vec<Arc<Pane>> {
        let tabs = self.tabs.read();
        tabs.values().flat_map(|tab| tab.panes().cloned()).collect()
    }

    // ==================== Client Management ====================

    /// Attach a client to this session
    pub fn attach_client(&self, client_id: ClientId) {
        let mut clients = self.attached_clients.write();
        clients.insert(client_id);
        *self.last_attached.write() = SystemTime::now();
    }

    /// Detach a client from this session
    pub fn detach_client(&self, client_id: ClientId) {
        let mut clients = self.attached_clients.write();
        clients.remove(&client_id);
    }

    /// Check if session has any attached clients
    pub fn has_attached_clients(&self) -> bool {
        !self.attached_clients.read().is_empty()
    }

    /// Get attached client count
    pub fn attached_client_count(&self) -> usize {
        self.attached_clients.read().len()
    }
}

/// Manages multiple sessions
pub struct SessionManager {
    sessions: Arc<RwLock<HashMap<SessionId, Arc<Session>>>>,
    store: SessionStore,
    default_session_id: Arc<RwLock<Option<SessionId>>>,
}

impl SessionManager {
    /// Create a new session manager
    pub fn new(db_path: std::path::PathBuf) -> Result<Self> {
        let store = SessionStore::new(db_path)?;

        Ok(Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
            store,
            default_session_id: Arc::new(RwLock::new(None)),
        })
    }

    /// Initialize from persisted sessions
    ///
    /// This restores session metadata from the database and spawns new PTYs
    /// for each session. The shell, columns, and rows parameters are used
    /// to configure the new terminal instances.
    pub fn restore_sessions(&self, shell: &str, cols: u16, rows: u16) -> Result<()> {
        let persisted = self.store.load_sessions()?;

        let mut sessions = self.sessions.write();
        for session in persisted {
            // Spawn a new PTY for the restored session
            if let Err(e) = session.ensure_default_tab(shell, cols, rows) {
                log::warn!(
                    "Failed to spawn PTY for restored session {}: {}",
                    session.id,
                    e
                );
                // Continue anyway - session exists but won't have PTY
            }

            let session = Arc::new(session);
            sessions.insert(session.id.clone(), session);
        }

        log::info!("Restored {} sessions from storage", sessions.len());
        Ok(())
    }

    /// Create a new session
    pub fn create_session(&self, name: String, cols: u16, rows: u16) -> Result<SessionId> {
        let session = Session::new(name.clone(), cols, rows)?;
        let id = session.id.clone();

        // Persist to database
        self.store.save_session(&session)?;

        // Add to active sessions
        let mut sessions = self.sessions.write();
        sessions.insert(id.clone(), Arc::new(session));

        // Set as default if first session
        if sessions.len() == 1 {
            *self.default_session_id.write() = Some(id.clone());
        }

        log::info!("Created session: {} ({})", name, id);
        Ok(id)
    }

    /// Delete a session
    pub fn delete_session(&self, id: &SessionId) -> Result<()> {
        let mut sessions = self.sessions.write();

        if let Some(session) = sessions.remove(id) {
            // Check if any clients are attached
            if session.has_attached_clients() {
                bail!("Cannot delete session with attached clients");
            }

            // Remove from database
            self.store.delete_session(id)?;

            // Clear default if this was it
            let mut default_id = self.default_session_id.write();
            if default_id.as_ref() == Some(id) {
                *default_id = sessions.keys().next().cloned();
            }

            log::info!("Deleted session: {}", id);
            Ok(())
        } else {
            bail!("Session not found: {}", id)
        }
    }

    /// Get a session by ID
    pub fn get_session(&self, id: &SessionId) -> Option<Arc<Session>> {
        self.sessions.read().get(id).cloned()
    }

    /// Get default session (or first available) - Public API for session routing
    #[allow(dead_code)]
    pub fn get_default_session(&self) -> Option<Arc<Session>> {
        let sessions = self.sessions.read();

        if let Some(id) = self.default_session_id.read().as_ref() {
            sessions.get(id).cloned()
        } else {
            sessions.values().next().cloned()
        }
    }

    /// List all sessions
    pub fn list_sessions(&self) -> Vec<(SessionId, String, u64, u64, usize)> {
        let sessions = self.sessions.read();
        sessions
            .values()
            .map(|s| {
                let created = s
                    .created_at
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs();
                let last_attached = s
                    .last_attached
                    .read()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs();
                let client_count = s.attached_client_count();

                (
                    s.id.clone(),
                    s.name.clone(),
                    created,
                    last_attached,
                    client_count,
                )
            })
            .collect()
    }

    /// Attach a client to a session
    pub fn attach_client(&self, session_id: &SessionId, client_id: ClientId) -> Result<()> {
        if let Some(session) = self.get_session(session_id) {
            session.attach_client(client_id);

            // Update persistence
            self.store.update_last_attached(session_id)?;

            log::info!("Client {} attached to session {}", client_id, session_id);
            Ok(())
        } else {
            bail!("Session not found: {}", session_id)
        }
    }

    /// Detach a client from a session
    pub fn detach_client(&self, session_id: &SessionId, client_id: ClientId) -> Result<()> {
        if let Some(session) = self.get_session(session_id) {
            session.detach_client(client_id);
            log::info!("Client {} detached from session {}", client_id, session_id);
            Ok(())
        } else {
            bail!("Session not found: {}", session_id)
        }
    }

    /// Rename a session
    pub fn rename_session(&self, session_id: &SessionId, new_name: String) -> Result<()> {
        let sessions = self.sessions.read();

        if let Some(_session) = sessions.get(session_id) {
            // Update in persistence layer (in-memory name would need RwLock wrapper for full mutability)
            self.store.rename_session(session_id, &new_name)?;
            log::info!("Renamed session {} to {}", session_id, new_name);
            Ok(())
        } else {
            bail!("Session not found: {}", session_id)
        }
    }

    /// Get session count
    pub fn session_count(&self) -> usize {
        self.sessions.read().len()
    }

    /// Cleanup sessions with no attached clients (optional maintenance) - Public API
    #[allow(dead_code)]
    pub fn cleanup_detached_sessions(&self, max_age_secs: u64) -> Result<usize> {
        let _now = SystemTime::now();
        let sessions_to_delete: Vec<SessionId> = {
            let sessions = self.sessions.read();
            sessions
                .values()
                .filter(|s| {
                    !s.has_attached_clients()
                        && s.last_attached
                            .read()
                            .elapsed()
                            .map(|d| d.as_secs() > max_age_secs)
                            .unwrap_or(false)
                })
                .map(|s| s.id.clone())
                .collect()
        };

        let count = sessions_to_delete.len();
        for id in sessions_to_delete {
            self.delete_session(&id)?;
        }

        Ok(count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_session_lifecycle() {
        let session = Session::new("test".to_string(), 80, 24).unwrap();
        assert_eq!(session.name, "test");
        assert_eq!(session.attached_client_count(), 0);

        session.attach_client(1);
        assert_eq!(session.attached_client_count(), 1);
        assert!(session.has_attached_clients());

        session.detach_client(1);
        assert_eq!(session.attached_client_count(), 0);
        assert!(!session.has_attached_clients());
    }

    #[test]
    fn test_session_manager_create_delete() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("sessions.db");

        let manager = SessionManager::new(db_path).unwrap();

        let id = manager.create_session("test".to_string(), 80, 24).unwrap();
        assert_eq!(manager.session_count(), 1);

        manager.delete_session(&id).unwrap();
        assert_eq!(manager.session_count(), 0);
    }

    #[test]
    fn test_session_attach_detach() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("sessions.db");

        let manager = SessionManager::new(db_path).unwrap();
        let id = manager.create_session("test".to_string(), 80, 24).unwrap();

        manager.attach_client(&id, 1).unwrap();
        let session = manager.get_session(&id).unwrap();
        assert!(session.has_attached_clients());

        manager.detach_client(&id, 1).unwrap();
        assert!(!session.has_attached_clients());
    }

    #[test]
    fn test_session_has_initial_tab_and_pane() {
        let session = Session::new("test".to_string(), 80, 24).unwrap();

        // Should have one tab
        assert_eq!(session.tab_count(), 1);

        // Should have an active pane
        let pane = session.get_active_pane();
        assert!(pane.is_some());

        // Pane should have PTY
        let pane = pane.unwrap();
        assert!(pane.has_pty());
    }

    #[test]
    fn test_session_tab_management() {
        let session = Session::new("test".to_string(), 80, 24).unwrap();

        // Create a second tab
        let tab_id = session.create_tab(Some("Second Tab".to_string())).unwrap();
        assert_eq!(session.tab_count(), 2);

        // List tabs
        let tabs = session.list_tabs();
        assert_eq!(tabs.len(), 2);

        // Switch to new tab
        session.switch_tab(tab_id).unwrap();
        assert_eq!(session.active_tab_id(), tab_id);

        // Close the new tab
        session.close_tab(tab_id).unwrap();
        assert_eq!(session.tab_count(), 1);
    }

    #[test]
    fn test_session_cannot_close_last_tab() {
        let session = Session::new("test".to_string(), 80, 24).unwrap();
        let active_tab_id = session.active_tab_id();

        // Should fail to close the last tab
        assert!(session.close_tab(active_tab_id).is_err());
    }

    #[test]
    fn test_session_active_pane_routing() {
        let session = Session::new("test".to_string(), 80, 24).unwrap();

        // Get active pane's PTY master
        let pty = session.get_active_pty_master();
        assert!(pty.is_some());

        // Get active pane's terminal state
        let terminal = session.get_active_terminal_state();
        assert!(terminal.is_some());
    }
}
