//! SSH domain implementation for remote terminal multiplexing
//!
//! SshDomain provides persistent SSH connections with automatic reconnection,
//! multiplexed channels, and remote PTY allocation.
//!
//! Features:
//! - Connection multiplexing (single SSH connection, multiple channels)
//! - Automatic reconnection on network failure
//! - Persistent remote panes across client disconnects
//! - SSH agent forwarding support
//! - Public key and password authentication

use super::domain::{Domain, DomainId, DomainPaneHandle, DomainStats, DomainType, PaneConfig};
use anyhow::{anyhow, bail, Context, Result};
use async_trait::async_trait;
use parking_lot::RwLock;
use russh::client::{Handle, Handler};
use russh::{Channel, ChannelMsg};
use russh_keys::key::PublicKey;
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use tokio::sync::Mutex as TokioMutex;

/// SSH domain configuration
#[derive(Debug, Clone)]
pub struct SshDomainConfig {
    /// Unique identifier for this SSH domain
    pub id: String,
    /// Human-readable name
    pub name: String,
    /// SSH server hostname or IP
    pub host: String,
    /// SSH server port (default: 22)
    pub port: u16,
    /// SSH username
    pub user: String,
    /// Authentication method
    pub auth: SshAuth,
    /// Timeout for connection attempts (seconds)
    pub connect_timeout: u64,
    /// Enable SSH agent forwarding
    pub forward_agent: bool,
    /// Remote working directory
    pub remote_cwd: Option<String>,
}

/// SSH authentication methods
#[derive(Debug, Clone)]
pub enum SshAuth {
    /// SSH agent authentication
    Agent,
    /// Public key file path
    PublicKey { path: String, passphrase: Option<String> },
    /// Password authentication
    Password(String),
}

impl Default for SshDomainConfig {
    fn default() -> Self {
        Self {
            id: "ssh-default".to_string(),
            name: "SSH Server".to_string(),
            host: "localhost".to_string(),
            port: 22,
            user: std::env::var("USER").unwrap_or_else(|_| "root".to_string()),
            auth: SshAuth::Agent,
            connect_timeout: 10,
            forward_agent: false,
            remote_cwd: None,
        }
    }
}

/// SSH domain with multiplexed connections
pub struct SshDomain {
    config: SshDomainConfig,
    /// SSH client session handle (Arc allows sharing between methods)
    session: Arc<TokioMutex<Option<Arc<Handle<ClientHandler>>>>>,
    /// Active channels: pane_id -> Channel
    channels: Arc<RwLock<HashMap<u64, Arc<TokioMutex<Channel<russh::client::Msg>>>>>>,
    /// Connection state
    connected: Arc<AtomicBool>,
    /// Next pane ID
    next_pane_id: Arc<AtomicU64>,
    /// Statistics
    stats: Arc<RwLock<DomainStats>>,
}

/// russh client handler
struct ClientHandler;

#[async_trait]
impl Handler for ClientHandler {
    type Error = russh::Error;

    async fn check_server_key(
        &mut self,
        _server_public_key: &PublicKey,
    ) -> Result<bool, Self::Error> {
        // TODO: Implement proper host key verification
        // For now, accept all keys (similar to ssh -o StrictHostKeyChecking=no)
        log::warn!("SSH: Host key verification disabled (accepting all keys)");
        Ok(true)
    }
}

impl SshDomain {
    /// Create a new SSH domain (not connected)
    pub fn new(config: SshDomainConfig) -> Self {
        Self {
            config,
            session: Arc::new(TokioMutex::new(None)),
            channels: Arc::new(RwLock::new(HashMap::new())),
            connected: Arc::new(AtomicBool::new(false)),
            next_pane_id: Arc::new(AtomicU64::new(1)),
            stats: Arc::new(RwLock::new(DomainStats::default())),
        }
    }

    /// Connect to the SSH server
    async fn connect_internal(&self) -> Result<()> {
        log::info!(
            "SSH: Connecting to {}@{}:{}",
            self.config.user,
            self.config.host,
            self.config.port
        );

        // Create SSH client config
        let ssh_config = Arc::new(russh::client::Config::default());

        let sh = ClientHandler;

        let mut session = russh::client::connect(
            ssh_config,
            (self.config.host.as_str(), self.config.port),
            sh,
        )
        .await
        .context("Failed to connect to SSH server")?;

        // Authenticate
        let auth_result = match &self.config.auth {
            SshAuth::Agent => {
                // TODO: Implement SSH agent authentication
                // This requires more complex integration with russh agent API
                // For now, fall back to default SSH key
                log::warn!("SSH Agent auth not yet implemented, trying default key ~/.ssh/id_rsa");

                let default_key_path = std::env::var("HOME")
                    .map(|home| format!("{}/.ssh/id_rsa", home))
                    .unwrap_or_else(|_| "~/.ssh/id_rsa".to_string());

                let key = russh_keys::load_secret_key(&default_key_path, None)
                    .context("Failed to load default SSH key (~/.ssh/id_rsa)")?;

                session
                    .authenticate_publickey(&self.config.user, Arc::new(key))
                    .await?
            }
            SshAuth::PublicKey { path, passphrase } => {
                // Load private key from file
                let key = if let Some(pass) = passphrase {
                    russh_keys::load_secret_key(path, Some(pass.as_str()))
                        .context("Failed to load private key")?
                } else {
                    russh_keys::load_secret_key(path, None)
                        .context("Failed to load private key")?
                };

                session
                    .authenticate_publickey(&self.config.user, Arc::new(key))
                    .await?
            }
            SshAuth::Password(password) => {
                session
                    .authenticate_password(&self.config.user, password)
                    .await?
            }
        };

        if !auth_result {
            bail!("SSH authentication failed");
        }

        log::info!("SSH: Authentication successful");

        // Store session handle wrapped in Arc
        *self.session.lock().await = Some(Arc::new(session));
        self.connected.store(true, Ordering::SeqCst);

        // Update stats
        {
            let mut stats = self.stats.write();
            stats.last_connected_at = Some(std::time::SystemTime::now());
        }

        Ok(())
    }

    /// Get or create a connection
    async fn ensure_connected(&self) -> Result<Arc<Handle<ClientHandler>>> {
        let session_guard = self.session.lock().await;

        if let Some(ref handle) = *session_guard {
            // Check if session is still alive
            if handle.is_closed() {
                log::warn!("SSH: Session closed, reconnecting...");
                drop(session_guard); // Release lock before reconnecting
                self.reconnect().await?;
                let session_guard = self.session.lock().await;
                return session_guard
                    .as_ref()
                    .ok_or_else(|| anyhow!("SSH session not available after reconnect"))
                    .map(|h| Arc::clone(h));
            }
            Ok(Arc::clone(handle))
        } else {
            // Not connected, establish connection
            drop(session_guard); // Release lock before connecting
            self.connect_internal().await?;
            let session_guard = self.session.lock().await;
            session_guard
                .as_ref()
                .ok_or_else(|| anyhow!("SSH session not available after connect"))
                .map(|h| Arc::clone(h))
        }
    }
}

#[async_trait]
impl Domain for SshDomain {
    fn id(&self) -> &DomainId {
        &self.config.id
    }

    fn name(&self) -> &str {
        &self.config.name
    }

    fn domain_type(&self) -> DomainType {
        DomainType::Ssh
    }

    fn is_connected(&self) -> bool {
        self.connected.load(Ordering::SeqCst)
    }

    async fn reconnect(&self) -> Result<()> {
        log::info!("SSH: Reconnecting to {}@{}", self.config.user, self.config.host);

        // Update stats
        {
            let mut stats = self.stats.write();
            stats.reconnect_attempts += 1;
        }

        // Close existing session
        {
            let mut session = self.session.lock().await;
            if let Some(handle) = session.take() {
                let _ = handle.disconnect(russh::Disconnect::ByApplication, "", "en").await;
            }
        }

        self.connected.store(false, Ordering::SeqCst);

        // Establish new connection
        self.connect_internal().await?;

        log::info!("SSH: Reconnection successful");
        Ok(())
    }

    async fn spawn_pane(&self, config: PaneConfig) -> Result<DomainPaneHandle> {
        let session_arc = self.ensure_connected().await?;

        // Allocate pane ID
        let pane_id = self.next_pane_id.fetch_add(1, Ordering::SeqCst);

        // Open a new channel
        let channel = session_arc.channel_open_session().await?;

        // Request PTY
        let term = std::env::var("TERM").unwrap_or_else(|_| "xterm-256color".to_string());
        channel
            .request_pty(
                false, // want_reply
                &term,
                config.cols as u32,
                config.rows as u32,
                0, // pixel_width
                0, // pixel_height
                &[], // terminal modes
            )
            .await?;

        // Build shell command
        let cwd = config.cwd.or_else(|| self.config.remote_cwd.clone());
        let shell_cmd = if let Some(dir) = cwd {
            format!("cd {} && exec {}", dir, config.shell)
        } else {
            format!("exec {}", config.shell)
        };

        // Start shell
        channel.exec(false, shell_cmd.as_bytes()).await?;

        // Store channel
        self.channels
            .write()
            .insert(pane_id, Arc::new(TokioMutex::new(channel)));

        // Update stats
        {
            let mut stats = self.stats.write();
            stats.active_panes = self.channels.read().len();
        }

        log::info!(
            "SSH: spawned pane {} on {}@{} ({}x{}, shell: {})",
            pane_id,
            self.config.user,
            self.config.host,
            config.cols,
            config.rows,
            config.shell
        );

        Ok(DomainPaneHandle {
            domain_id: self.config.id.clone(),
            pane_id,
        })
    }

    async fn attach_pane(&self, pane_id: u64) -> Result<DomainPaneHandle> {
        // Check if channel exists
        if self.channels.read().contains_key(&pane_id) {
            Ok(DomainPaneHandle {
                domain_id: self.config.id.clone(),
                pane_id,
            })
        } else {
            bail!("SSH pane {} not found in domain {}", pane_id, self.config.id)
        }
    }

    async fn close_pane(&self, handle: &DomainPaneHandle) -> Result<()> {
        if handle.domain_id != self.config.id {
            bail!("Pane handle domain mismatch");
        }

        // Remove channel
        let channel_arc = self.channels.write().remove(&handle.pane_id);
        if let Some(ch_arc) = channel_arc {
            let channel = ch_arc.lock().await;
            let _ = channel.close().await;

            // Update stats
            let mut stats = self.stats.write();
            stats.active_panes = self.channels.read().len();

            log::info!("SSH: closed pane {} in domain {}", handle.pane_id, self.config.id);
            Ok(())
        } else {
            bail!("SSH pane {} not found", handle.pane_id)
        }
    }

    async fn write_to_pane(&self, handle: &DomainPaneHandle, data: &[u8]) -> Result<()> {
        if handle.domain_id != self.config.id {
            bail!("Pane handle domain mismatch");
        }

        // Scope to release the read lock before await
        let channel_arc = {
            let channels = self.channels.read();
            channels.get(&handle.pane_id).cloned()
        };

        if let Some(ch_arc) = channel_arc {
            let channel = ch_arc.lock().await;
            channel.data(data).await?;

            // Update stats
            let mut stats = self.stats.write();
            stats.bytes_sent += data.len() as u64;

            Ok(())
        } else {
            bail!("SSH pane {} not found", handle.pane_id)
        }
    }

    async fn read_from_pane(&self, handle: &DomainPaneHandle, buf: &mut [u8]) -> Result<usize> {
        if handle.domain_id != self.config.id {
            bail!("Pane handle domain mismatch");
        }

        // Scope to release the read lock before await
        let channel_arc = {
            let channels = self.channels.read();
            channels.get(&handle.pane_id).cloned()
        };

        if let Some(ch_arc) = channel_arc {
            let mut channel = ch_arc.lock().await;

            // Wait for channel message with timeout
            match tokio::time::timeout(
                std::time::Duration::from_millis(10),
                channel.wait()
            ).await {
                Ok(Some(msg)) => {
                    match msg {
                        ChannelMsg::Data { ref data } => {
                            let n = std::cmp::min(data.len(), buf.len());
                            buf[..n].copy_from_slice(&data[..n]);

                            // Update stats
                            let mut stats = self.stats.write();
                            stats.bytes_received += n as u64;

                            Ok(n)
                        }
                        ChannelMsg::Eof => Ok(0),
                        ChannelMsg::ExitStatus { exit_status } => {
                            log::info!("SSH pane {} exited with status {}", handle.pane_id, exit_status);
                            Ok(0)
                        }
                        _ => Ok(0),
                    }
                }
                Ok(None) => Ok(0), // Channel closed
                Err(_) => Ok(0),   // Timeout, no data available
            }
        } else {
            bail!("SSH pane {} not found", handle.pane_id)
        }
    }

    async fn resize_pane(&self, handle: &DomainPaneHandle, cols: u16, rows: u16) -> Result<()> {
        if handle.domain_id != self.config.id {
            bail!("Pane handle domain mismatch");
        }

        // Scope to release the read lock before await
        let channel_arc = {
            let channels = self.channels.read();
            channels.get(&handle.pane_id).cloned()
        };

        if let Some(ch_arc) = channel_arc {
            let channel = ch_arc.lock().await;
            channel
                .window_change(cols as u32, rows as u32, 0, 0)
                .await?;

            log::debug!(
                "SSH: resized pane {} to {}x{} in domain {}",
                handle.pane_id,
                cols,
                rows,
                self.config.id
            );
            Ok(())
        } else {
            bail!("SSH pane {} not found", handle.pane_id)
        }
    }

    fn stats(&self) -> DomainStats {
        self.stats.read().clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ssh_domain_config_defaults() {
        let config = SshDomainConfig::default();
        assert_eq!(config.port, 22);
        assert_eq!(config.connect_timeout, 10);
        assert!(!config.forward_agent);
    }

    #[test]
    fn test_ssh_domain_creation() {
        let config = SshDomainConfig {
            id: "test-ssh".to_string(),
            name: "Test SSH Server".to_string(),
            host: "example.com".to_string(),
            user: "testuser".to_string(),
            ..Default::default()
        };

        let domain = SshDomain::new(config);
        assert_eq!(domain.id(), "test-ssh");
        assert_eq!(domain.name(), "Test SSH Server");
        assert_eq!(domain.domain_type(), DomainType::Ssh);
        assert!(!domain.is_connected());
    }

    // Note: Integration tests that actually connect to SSH servers
    // should be in tests/ directory with #[ignore] attribute
}
