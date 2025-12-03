//! Domain abstraction for terminal multiplexing
//!
//! Domains represent different execution environments where terminal panes can run:
//! - LocalDomain: PTY processes on the local machine
//! - SshDomain: Remote shells over SSH connections
//!
//! This abstraction enables:
//! - Cross-domain pane splits (local + remote panes in same session)
//! - Persistent remote sessions with reconnection
//! - Network-transparent terminal multiplexing

use anyhow::Result;
use async_trait::async_trait;
use std::sync::Arc;

/// Unique identifier for a domain
pub type DomainId = String;

/// Configuration for spawning a new pane
#[derive(Debug, Clone)]
pub struct PaneConfig {
    /// Shell command to run (e.g., "bash", "/bin/zsh")
    pub shell: String,
    /// Working directory (None = use domain default)
    pub cwd: Option<String>,
    /// Terminal dimensions
    pub cols: u16,
    pub rows: u16,
    /// Environment variables
    pub env: Vec<(String, String)>,
}

impl Default for PaneConfig {
    fn default() -> Self {
        Self {
            shell: std::env::var("SHELL").unwrap_or_else(|_| "bash".to_string()),
            cwd: None,
            cols: 80,
            rows: 24,
            env: Vec::new(),
        }
    }
}

/// Handle to a pane spawned in a domain
///
/// This is an opaque identifier returned by Domain::spawn_pane()
/// The actual pane implementation depends on the domain type.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DomainPaneHandle {
    pub domain_id: DomainId,
    pub pane_id: u64,
}

/// Trait for terminal execution domains
///
/// A Domain manages the lifecycle of terminal processes and provides
/// I/O channels for communicating with them. Different domain types
/// implement different transport mechanisms (local PTY, SSH, etc.)
#[async_trait]
pub trait Domain: Send + Sync {
    /// Get the domain's unique identifier
    fn id(&self) -> &DomainId;

    /// Get the domain's human-readable name
    fn name(&self) -> &str;

    /// Get the domain type (e.g., "local", "ssh")
    fn domain_type(&self) -> DomainType;

    /// Check if the domain is currently connected and operational
    fn is_connected(&self) -> bool;

    /// Attempt to reconnect to the domain
    ///
    /// For LocalDomain, this is a no-op (always connected).
    /// For SshDomain, this re-establishes SSH connection.
    async fn reconnect(&self) -> Result<()>;

    /// Spawn a new pane in this domain
    ///
    /// Returns a handle that can be used to interact with the pane.
    /// The actual Pane object is created by the session manager.
    async fn spawn_pane(&self, config: PaneConfig) -> Result<DomainPaneHandle>;

    /// Attach to an existing pane (for reconnection scenarios)
    ///
    /// This is used when restoring sessions after daemon restart.
    async fn attach_pane(&self, pane_id: u64) -> Result<DomainPaneHandle>;

    /// Close a pane and clean up resources
    async fn close_pane(&self, handle: &DomainPaneHandle) -> Result<()>;

    /// Send input data to a pane
    async fn write_to_pane(&self, handle: &DomainPaneHandle, data: &[u8]) -> Result<()>;

    /// Read output from a pane (non-blocking)
    ///
    /// Returns up to `buf.len()` bytes of output.
    /// Returns Ok(0) if no data is available.
    async fn read_from_pane(&self, handle: &DomainPaneHandle, buf: &mut [u8]) -> Result<usize>;

    /// Resize a pane's terminal dimensions
    async fn resize_pane(&self, handle: &DomainPaneHandle, cols: u16, rows: u16) -> Result<()>;

    /// Get connection statistics (for monitoring)
    fn stats(&self) -> DomainStats;
}

/// Type of domain
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DomainType {
    /// Local PTY processes
    Local,
    /// Remote SSH session
    Ssh,
    /// Future: Docker container
    Docker,
    /// Future: Kubernetes pod
    Kubernetes,
}

impl std::fmt::Display for DomainType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DomainType::Local => write!(f, "local"),
            DomainType::Ssh => write!(f, "ssh"),
            DomainType::Docker => write!(f, "docker"),
            DomainType::Kubernetes => write!(f, "kubernetes"),
        }
    }
}

/// Statistics for a domain
#[derive(Debug, Clone, Default)]
pub struct DomainStats {
    /// Number of active panes in this domain
    pub active_panes: usize,
    /// Total bytes sent to panes
    pub bytes_sent: u64,
    /// Total bytes received from panes
    pub bytes_received: u64,
    /// Number of reconnection attempts (for remote domains)
    pub reconnect_attempts: u64,
    /// Last successful connection timestamp (for remote domains)
    pub last_connected_at: Option<std::time::SystemTime>,
}

/// Registry of all available domains
pub struct DomainRegistry {
    domains: parking_lot::RwLock<std::collections::HashMap<DomainId, Arc<dyn Domain>>>,
    default_domain_id: parking_lot::RwLock<Option<DomainId>>,
}

impl DomainRegistry {
    pub fn new() -> Self {
        Self {
            domains: parking_lot::RwLock::new(std::collections::HashMap::new()),
            default_domain_id: parking_lot::RwLock::new(None),
        }
    }

    /// Register a new domain
    pub fn register(&self, domain: Arc<dyn Domain>) {
        let id = domain.id().to_string();
        self.domains.write().insert(id.clone(), domain);

        // Set as default if first domain
        if self.domains.read().len() == 1 {
            *self.default_domain_id.write() = Some(id);
        }
    }

    /// Get a domain by ID
    pub fn get(&self, id: &DomainId) -> Option<Arc<dyn Domain>> {
        self.domains.read().get(id).cloned()
    }

    /// Get the default domain (usually LocalDomain)
    pub fn get_default(&self) -> Option<Arc<dyn Domain>> {
        let default_id = self.default_domain_id.read();
        default_id
            .as_ref()
            .and_then(|id| self.domains.read().get(id).cloned())
    }

    /// List all registered domains
    pub fn list(&self) -> Vec<(DomainId, String, DomainType, bool)> {
        self.domains
            .read()
            .values()
            .map(|d| (d.id().to_string(), d.name().to_string(), d.domain_type(), d.is_connected()))
            .collect()
    }

    /// Remove a domain
    pub fn unregister(&self, id: &DomainId) {
        self.domains.write().remove(id);

        // Update default if removed
        let mut default_id = self.default_domain_id.write();
        if default_id.as_ref() == Some(id) {
            *default_id = self.domains.read().keys().next().cloned();
        }
    }

    /// Get domain count
    pub fn count(&self) -> usize {
        self.domains.read().len()
    }
}

impl Default for DomainRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pane_config_defaults() {
        let config = PaneConfig::default();
        assert_eq!(config.cols, 80);
        assert_eq!(config.rows, 24);
        assert!(config.env.is_empty());
    }

    #[test]
    fn test_domain_type_display() {
        assert_eq!(DomainType::Local.to_string(), "local");
        assert_eq!(DomainType::Ssh.to_string(), "ssh");
        assert_eq!(DomainType::Docker.to_string(), "docker");
        assert_eq!(DomainType::Kubernetes.to_string(), "kubernetes");
    }

    #[test]
    fn test_domain_registry() {
        // Test is just for compilation; we'll add mock domains in integration tests
        let registry = DomainRegistry::new();
        assert_eq!(registry.count(), 0);
        assert!(registry.get_default().is_none());
    }
}
