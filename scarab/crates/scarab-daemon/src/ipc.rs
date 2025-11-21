use anyhow::{Context, Result};
use scarab_protocol::{ControlMessage, SOCKET_PATH, MAX_MESSAGE_SIZE, MAX_CLIENTS};
use std::path::Path;
use std::sync::Arc;
use tokio::io::AsyncReadExt;
use tokio::net::{UnixListener, UnixStream};
use tokio::sync::{mpsc, RwLock};
use portable_pty::PtySize;

/// Handle to send commands to PTY
/// Using channels for thread-safe communication
#[derive(Clone)]
pub struct PtyHandle {
    input_tx: mpsc::UnboundedSender<Vec<u8>>,
    resize_tx: mpsc::Sender<PtySize>,
}

impl PtyHandle {
    pub fn new(
        input_tx: mpsc::UnboundedSender<Vec<u8>>,
        resize_tx: mpsc::Sender<PtySize>,
    ) -> Self {
        Self {
            input_tx,
            resize_tx,
        }
    }

    pub async fn write_input(&self, data: &[u8]) -> Result<()> {
        self.input_tx
            .send(data.to_vec())
            .context("Failed to send input to PTY channel")?;
        Ok(())
    }

    pub async fn resize(&self, cols: u16, rows: u16) -> Result<()> {
        self.resize_tx
            .send(PtySize {
                rows,
                cols,
                pixel_width: 0,
                pixel_height: 0,
            })
            .await
            .context("Failed to send resize event to PTY")?;
        Ok(())
    }
}

/// IPC server managing multiple client connections
pub struct IpcServer {
    listener: UnixListener,
    pty_handle: PtyHandle,
    client_counter: Arc<RwLock<u64>>,
}

impl IpcServer {
    /// Create new IPC server, removing stale socket if exists
    pub async fn new(pty_handle: PtyHandle) -> Result<Self> {
        // Remove existing socket if present
        if Path::new(SOCKET_PATH).exists() {
            std::fs::remove_file(SOCKET_PATH)
                .context("Failed to remove stale socket")?;
        }

        let listener = UnixListener::bind(SOCKET_PATH)
            .context("Failed to bind Unix socket")?;

        // Set socket permissions to 700 (owner only)
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(
                SOCKET_PATH,
                std::fs::Permissions::from_mode(0o700),
            )
            .context("Failed to set socket permissions")?;
        }

        println!("IPC server listening on: {}", SOCKET_PATH);

        Ok(Self {
            listener,
            pty_handle,
            client_counter: Arc::new(RwLock::new(0)),
        })
    }

    /// Accept client connections in a loop
    pub async fn accept_loop(self) -> Result<()> {
        let active_clients = Arc::new(RwLock::new(0usize));

        loop {
            match self.listener.accept().await {
                Ok((stream, _addr)) => {
                    let client_count = {
                        let mut count = active_clients.write().await;
                        *count += 1;
                        *count
                    };

                    if client_count > MAX_CLIENTS {
                        eprintln!("Max clients ({}) reached, rejecting connection", MAX_CLIENTS);
                        let mut count = active_clients.write().await;
                        *count -= 1;
                        continue;
                    }

                    let client_id = {
                        let mut counter = self.client_counter.write().await;
                        *counter += 1;
                        *counter
                    };

                    println!("Client {} connected (active: {})", client_id, client_count);

                    let pty_handle = self.pty_handle.clone();
                    let active_clients = active_clients.clone();

                    tokio::spawn(async move {
                        if let Err(e) = handle_client(stream, client_id, pty_handle).await {
                            eprintln!("Client {} error: {}", client_id, e);
                        }

                        let mut count = active_clients.write().await;
                        *count -= 1;
                        println!("Client {} disconnected (active: {})", client_id, *count);
                    });
                }
                Err(e) => {
                    eprintln!("Failed to accept client: {}", e);
                }
            }
        }
    }
}

/// Handle individual client connection
async fn handle_client(
    mut stream: UnixStream,
    client_id: u64,
    pty_handle: PtyHandle,
) -> Result<()> {
    let mut buffer = vec![0u8; MAX_MESSAGE_SIZE];

    loop {
        // Read message length prefix (4 bytes)
        let len = match stream.read_u32().await {
            Ok(len) => len as usize,
            Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => {
                // Client disconnected gracefully
                break;
            }
            Err(e) => {
                return Err(e).context("Failed to read message length");
            }
        };

        if len == 0 || len > MAX_MESSAGE_SIZE {
            anyhow::bail!("Invalid message length: {}", len);
        }

        // Read message data
        stream.read_exact(&mut buffer[..len]).await
            .context("Failed to read message data")?;

        // Deserialize with rkyv
        let msg = match rkyv::from_bytes::<ControlMessage>(&buffer[..len]) {
            Ok(msg) => msg,
            Err(e) => {
                eprintln!("Failed to deserialize ControlMessage: {:?}", e);
                anyhow::bail!("Deserialization error");
            }
        };

        // Handle message
        if let Err(e) = handle_message(msg, &pty_handle, client_id).await {
            eprintln!("Client {} message handling error: {}", client_id, e);
            // Don't disconnect on individual message errors
        }
    }

    Ok(())
}

/// Process a control message
async fn handle_message(
    msg: ControlMessage,
    pty_handle: &PtyHandle,
    client_id: u64,
) -> Result<()> {
    match msg {
        ControlMessage::Resize { cols, rows } => {
            println!("Client {} resize: {}x{}", client_id, cols, rows);
            pty_handle.resize(cols, rows).await?;
        }
        ControlMessage::Input { data } => {
            // Validate input size to prevent abuse
            if data.len() > MAX_MESSAGE_SIZE {
                anyhow::bail!("Input data too large: {} bytes", data.len());
            }
            pty_handle.write_input(&data).await?;
        }
        ControlMessage::LoadPlugin { path } => {
            println!("Client {} loading plugin: {}", client_id, path);
            // TODO: Implement plugin loading
            eprintln!("Plugin loading not yet implemented");
        }
        ControlMessage::Ping { timestamp } => {
            println!("Client {} ping: {}", client_id, timestamp);
            // Could respond with pong if bidirectional communication is needed
        }
        ControlMessage::Disconnect { client_id: id } => {
            println!("Client {} requesting disconnect", id);
            // Client will disconnect when this function returns
        }
    }

    Ok(())
}

/// Cleanup socket on server shutdown
impl Drop for IpcServer {
    fn drop(&mut self) {
        if Path::new(SOCKET_PATH).exists() {
            let _ = std::fs::remove_file(SOCKET_PATH);
            println!("Cleaned up socket: {}", SOCKET_PATH);
        }
    }
}
