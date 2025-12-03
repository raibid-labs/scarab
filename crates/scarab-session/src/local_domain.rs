//! Local domain implementation using PTY processes
//!
//! LocalDomain spawns processes on the local machine using portable-pty.
//! This is the default domain and always available.

use super::domain::{Domain, DomainId, DomainPaneHandle, DomainStats, DomainType, PaneConfig};
use anyhow::{bail, Result};
use async_trait::async_trait;
use parking_lot::RwLock;
use portable_pty::{CommandBuilder, MasterPty, NativePtySystem, PtySize, PtySystem};
use std::collections::HashMap;
use std::io::{Read, Write};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

/// Local domain using PTY processes
pub struct LocalDomain {
    id: DomainId,
    name: String,
    pty_system: NativePtySystem,
    /// Active panes: pane_id -> (pty_master, pty_writer)
    panes: Arc<RwLock<HashMap<u64, PaneResources>>>,
    /// Next pane ID to assign
    next_pane_id: AtomicU64,
    /// Statistics
    stats: Arc<RwLock<DomainStats>>,
}

/// Resources for a single pane in the local domain
struct PaneResources {
    pty_master: Arc<Mutex<Box<dyn MasterPty + Send>>>,
    pty_writer: Arc<Mutex<Box<dyn Write + Send>>>,
}

impl LocalDomain {
    /// Create a new local domain
    pub fn new() -> Self {
        Self {
            id: "local".to_string(),
            name: "Local Machine".to_string(),
            pty_system: NativePtySystem::default(),
            panes: Arc::new(RwLock::new(HashMap::new())),
            next_pane_id: AtomicU64::new(1),
            stats: Arc::new(RwLock::new(DomainStats {
                last_connected_at: Some(std::time::SystemTime::now()),
                ..Default::default()
            })),
        }
    }

    /// Create a local domain with a custom ID (for testing)
    #[allow(dead_code)]
    pub fn with_id(id: String, name: String) -> Self {
        Self {
            id,
            name,
            pty_system: NativePtySystem::default(),
            panes: Arc::new(RwLock::new(HashMap::new())),
            next_pane_id: AtomicU64::new(1),
            stats: Arc::new(RwLock::new(DomainStats {
                last_connected_at: Some(std::time::SystemTime::now()),
                ..Default::default()
            })),
        }
    }

    /// Get PTY master for a pane (internal use for session manager integration)
    pub fn get_pty_master(&self, pane_id: u64) -> Option<Arc<Mutex<Box<dyn MasterPty + Send>>>> {
        self.panes
            .read()
            .get(&pane_id)
            .map(|r| Arc::clone(&r.pty_master))
    }

    /// Get PTY writer for a pane (internal use for session manager integration)
    pub fn get_pty_writer(&self, pane_id: u64) -> Option<Arc<Mutex<Box<dyn Write + Send>>>> {
        self.panes
            .read()
            .get(&pane_id)
            .map(|r| Arc::clone(&r.pty_writer))
    }
}

impl Default for LocalDomain {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Domain for LocalDomain {
    fn id(&self) -> &DomainId {
        &self.id
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn domain_type(&self) -> DomainType {
        DomainType::Local
    }

    fn is_connected(&self) -> bool {
        // Local domain is always connected
        true
    }

    async fn reconnect(&self) -> Result<()> {
        // Local domain doesn't need reconnection
        Ok(())
    }

    async fn spawn_pane(&self, config: PaneConfig) -> Result<DomainPaneHandle> {
        // Allocate pane ID
        let pane_id = self.next_pane_id.fetch_add(1, Ordering::SeqCst);

        // Create PTY with specified dimensions
        let pair = self.pty_system.openpty(PtySize {
            rows: config.rows,
            cols: config.cols,
            pixel_width: 0,
            pixel_height: 0,
        })?;

        // Build shell command
        let mut cmd = CommandBuilder::new(&config.shell);

        // Set working directory
        if let Some(ref cwd) = config.cwd {
            cmd.cwd(cwd);
        } else if let Ok(home) = std::env::var("HOME") {
            cmd.cwd(home);
        }

        // Add environment variables
        for (key, value) in &config.env {
            cmd.env(key, value);
        }

        // Always set Navigation Protocol socket
        cmd.env("SCARAB_NAV_SOCKET", "/tmp/scarab-nav.sock");

        // Spawn shell in PTY
        let _child = pair.slave.spawn_command(cmd)?;

        // Get the writer before storing the master
        let writer = pair.master.take_writer()?;

        // Store resources
        let resources = PaneResources {
            pty_master: Arc::new(Mutex::new(pair.master)),
            pty_writer: Arc::new(Mutex::new(writer)),
        };

        self.panes.write().insert(pane_id, resources);

        // Update stats
        {
            let mut stats = self.stats.write();
            stats.active_panes = self.panes.read().len();
        }

        log::info!(
            "LocalDomain: spawned pane {} ({}x{}, shell: {})",
            pane_id,
            config.cols,
            config.rows,
            config.shell
        );

        Ok(DomainPaneHandle {
            domain_id: self.id.clone(),
            pane_id,
        })
    }

    async fn attach_pane(&self, pane_id: u64) -> Result<DomainPaneHandle> {
        // Check if pane exists
        if self.panes.read().contains_key(&pane_id) {
            Ok(DomainPaneHandle {
                domain_id: self.id.clone(),
                pane_id,
            })
        } else {
            bail!("Pane {} not found in LocalDomain", pane_id)
        }
    }

    async fn close_pane(&self, handle: &DomainPaneHandle) -> Result<()> {
        if handle.domain_id != self.id {
            bail!("Pane handle domain mismatch");
        }

        // Remove from active panes (PTY will be dropped, terminating the process)
        if self.panes.write().remove(&handle.pane_id).is_some() {
            // Update stats
            let mut stats = self.stats.write();
            stats.active_panes = self.panes.read().len();

            log::info!("LocalDomain: closed pane {}", handle.pane_id);
            Ok(())
        } else {
            bail!("Pane {} not found", handle.pane_id)
        }
    }

    async fn write_to_pane(&self, handle: &DomainPaneHandle, data: &[u8]) -> Result<()> {
        if handle.domain_id != self.id {
            bail!("Pane handle domain mismatch");
        }

        let panes = self.panes.read();
        if let Some(resources) = panes.get(&handle.pane_id) {
            let mut writer = resources.pty_writer.lock().unwrap();
            writer.write_all(data)?;
            writer.flush()?;

            // Update stats
            let mut stats = self.stats.write();
            stats.bytes_sent += data.len() as u64;

            Ok(())
        } else {
            bail!("Pane {} not found", handle.pane_id)
        }
    }

    async fn read_from_pane(&self, handle: &DomainPaneHandle, buf: &mut [u8]) -> Result<usize> {
        if handle.domain_id != self.id {
            bail!("Pane handle domain mismatch");
        }

        let panes = self.panes.read();
        if let Some(resources) = panes.get(&handle.pane_id) {
            let master = resources.pty_master.lock().unwrap();

            // Try to read (non-blocking via try_clone or similar)
            // portable-pty doesn't have built-in non-blocking reads,
            // so we use the reader with standard read() which may block.
            // In practice, this will be called from an async task pool.
            match master.try_clone_reader() {
                Ok(mut reader) => {
                    // Use a timeout-based read
                    let n = reader.read(buf).unwrap_or(0);

                    if n > 0 {
                        // Update stats
                        let mut stats = self.stats.write();
                        stats.bytes_received += n as u64;
                    }

                    Ok(n)
                }
                Err(e) => {
                    log::warn!("Failed to clone reader for pane {}: {}", handle.pane_id, e);
                    Ok(0)
                }
            }
        } else {
            bail!("Pane {} not found", handle.pane_id)
        }
    }

    async fn resize_pane(&self, handle: &DomainPaneHandle, cols: u16, rows: u16) -> Result<()> {
        if handle.domain_id != self.id {
            bail!("Pane handle domain mismatch");
        }

        let panes = self.panes.read();
        if let Some(resources) = panes.get(&handle.pane_id) {
            let master = resources.pty_master.lock().unwrap();
            master.resize(PtySize {
                rows,
                cols,
                pixel_width: 0,
                pixel_height: 0,
            })?;

            log::debug!(
                "LocalDomain: resized pane {} to {}x{}",
                handle.pane_id,
                cols,
                rows
            );
            Ok(())
        } else {
            bail!("Pane {} not found", handle.pane_id)
        }
    }

    fn stats(&self) -> DomainStats {
        self.stats.read().clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_local_domain_creation() {
        let domain = LocalDomain::new();
        assert_eq!(domain.id(), "local");
        assert_eq!(domain.name(), "Local Machine");
        assert_eq!(domain.domain_type(), DomainType::Local);
        assert!(domain.is_connected());
    }

    #[tokio::test]
    async fn test_local_domain_spawn_pane() {
        let domain = LocalDomain::new();

        let config = PaneConfig {
            shell: "bash".to_string(),
            cols: 80,
            rows: 24,
            ..Default::default()
        };

        let handle = domain.spawn_pane(config).await.unwrap();
        assert_eq!(handle.domain_id, "local");
        assert_eq!(handle.pane_id, 1);

        let stats = domain.stats();
        assert_eq!(stats.active_panes, 1);
    }

    #[tokio::test]
    async fn test_local_domain_close_pane() {
        let domain = LocalDomain::new();

        let config = PaneConfig::default();
        let handle = domain.spawn_pane(config).await.unwrap();

        assert_eq!(domain.stats().active_panes, 1);

        domain.close_pane(&handle).await.unwrap();
        assert_eq!(domain.stats().active_panes, 0);
    }

    #[tokio::test]
    async fn test_local_domain_write_read() {
        let domain = LocalDomain::new();

        let config = PaneConfig {
            shell: "cat".to_string(), // Use cat for echo testing
            ..Default::default()
        };

        let handle = domain.spawn_pane(config).await.unwrap();

        // Write some data
        let data = b"hello\n";
        domain.write_to_pane(&handle, data).await.unwrap();

        // Give it a moment to process
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        // Read back (should echo from cat)
        let mut buf = [0u8; 1024];
        let n = domain.read_from_pane(&handle, &mut buf).await.unwrap();

        assert!(n > 0, "Should have read some data");
        assert!(buf[..n].windows(5).any(|w| w == b"hello"));
    }

    #[tokio::test]
    async fn test_local_domain_resize() {
        let domain = LocalDomain::new();

        let config = PaneConfig::default();
        let handle = domain.spawn_pane(config).await.unwrap();

        // Resize should not error
        domain.resize_pane(&handle, 120, 40).await.unwrap();
    }

    #[tokio::test]
    async fn test_local_domain_stats() {
        let domain = LocalDomain::new();

        let config = PaneConfig::default();
        let handle = domain.spawn_pane(config).await.unwrap();

        domain.write_to_pane(&handle, b"test").await.unwrap();

        let stats = domain.stats();
        assert_eq!(stats.active_panes, 1);
        assert!(stats.bytes_sent >= 4);
        assert!(stats.last_connected_at.is_some());
    }
}
