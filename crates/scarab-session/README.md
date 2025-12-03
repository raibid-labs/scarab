# scarab-session

Session management and domain abstraction for Scarab terminal emulator.

## Overview

This crate provides the foundation for Scarab's terminal multiplexing capabilities, including:

- **Domain Abstraction**: Unified interface for local and remote terminal processes
- **LocalDomain**: PTY processes on the local machine
- **SshDomain**: Remote shells over SSH connections with multiplexing
- **Session Plugin**: Command palette integration for session management

## Features

### Domain System

The `Domain` trait provides a consistent API for spawning and managing terminal panes across different execution environments:

```rust
pub trait Domain: Send + Sync {
    fn id(&self) -> &DomainId;
    fn name(&self) -> &str;
    fn domain_type(&self) -> DomainType;
    fn is_connected(&self) -> bool;

    async fn reconnect(&self) -> Result<()>;
    async fn spawn_pane(&self, config: PaneConfig) -> Result<DomainPaneHandle>;
    async fn write_to_pane(&self, handle: &DomainPaneHandle, data: &[u8]) -> Result<()>;
    async fn read_from_pane(&self, handle: &DomainPaneHandle, buf: &mut [u8]) -> Result<usize>;
    async fn resize_pane(&self, handle: &DomainPaneHandle, cols: u16, rows: u16) -> Result<()>;
    async fn close_pane(&self, handle: &DomainPaneHandle) -> Result<()>;

    fn stats(&self) -> DomainStats;
}
```

### LocalDomain

Manages PTY processes on the local machine using `portable-pty`:

```rust
use scarab_session::{LocalDomain, PaneConfig};

let domain = LocalDomain::new();

let config = PaneConfig {
    shell: "bash".to_string(),
    cols: 80,
    rows: 24,
    cwd: Some("/home/user".to_string()),
    env: vec![("EDITOR".to_string(), "vim".to_string())],
};

let handle = domain.spawn_pane(config).await?;
```

### SshDomain

Remote terminal sessions over SSH with connection multiplexing:

```rust
use scarab_session::{SshDomain, SshDomainConfig, SshAuth};

let config = SshDomainConfig {
    id: "prod".to_string(),
    name: "Production Server".to_string(),
    host: "prod.example.com".to_string(),
    port: 22,
    user: "admin".to_string(),
    auth: SshAuth::PublicKey {
        path: "/home/user/.ssh/prod_key".to_string(),
        passphrase: None,
    },
    connect_timeout: 10,
    forward_agent: false,
    remote_cwd: Some("/var/www".to_string()),
};

let domain = SshDomain::new(config);
domain.reconnect().await?;  // Establish connection

let pane = domain.spawn_pane(PaneConfig::default()).await?;
```

### DomainRegistry

Central registry for managing multiple domains:

```rust
use scarab_session::{DomainRegistry, LocalDomain, SshDomain};
use std::sync::Arc;

let registry = DomainRegistry::new();

// Register local domain
let local = Arc::new(LocalDomain::new());
registry.register(local);

// Register SSH domains
let ssh = Arc::new(SshDomain::new(ssh_config));
registry.register(ssh);

// List all domains
for (id, name, dtype, connected) in registry.list() {
    println!("{}: {} ({}) - {}", id, name, dtype,
        if connected { "connected" } else { "disconnected" });
}

// Get domain by ID
let domain = registry.get(&"prod".to_string())?;
```

## SSH Authentication

Three authentication methods are supported:

### 1. SSH Agent (Default)

```rust
let config = SshDomainConfig {
    auth: SshAuth::Agent,
    ..Default::default()
};
```

**Note**: Currently falls back to `~/.ssh/id_rsa` if available. Full agent support coming soon.

### 2. Public Key File

```rust
let config = SshDomainConfig {
    auth: SshAuth::PublicKey {
        path: "/path/to/private/key".to_string(),
        passphrase: Some("key_passphrase".to_string()),
    },
    ..Default::default()
};
```

### 3. Password

```rust
let config = SshDomainConfig {
    auth: SshAuth::Password("secret".to_string()),
    ..Default::default()
};
```

## Connection Management

SSH domains handle connection lifecycle automatically:

- **Multiplexing**: Single SSH connection per domain, shared by all panes
- **Reconnection**: Automatic reconnection on network failure
- **Persistence**: Panes survive local client disconnects (daemon-side)

```rust
// Check connection status
if !domain.is_connected() {
    domain.reconnect().await?;
}

// Get statistics
let stats = domain.stats();
println!("Active panes: {}", stats.active_panes);
println!("Bytes sent: {}", stats.bytes_sent);
println!("Bytes received: {}", stats.bytes_received);
println!("Reconnect attempts: {}", stats.reconnect_attempts);
```

## Testing

### Unit Tests

```bash
cargo test -p scarab-session --lib
```

### Integration Tests

```bash
# Run non-SSH tests
cargo test -p scarab-session --test ssh_domain_integration

# Run all tests including SSH tests (requires SSH server)
cargo test -p scarab-session --test ssh_domain_integration -- --include-ignored
```

### SSH Server Test Setup

For full SSH integration testing:

```bash
# Start SSH server
sudo systemctl start sshd

# Ensure SSH key is set up
ssh-keygen -t rsa -f ~/.ssh/id_rsa
ssh-copy-id localhost

# Run SSH tests
cargo test -p scarab-session --test ssh_domain_integration -- --ignored
```

## Configuration

See `scarab-config` crate for TOML/Fusabi configuration:

```toml
[[ssh_domains]]
id = "dev"
name = "Development Server"
host = "dev.example.com"
port = 22
user = "developer"
auth_type = "publickey"
key_path = "/home/user/.ssh/dev_key"
remote_cwd = "/home/developer/projects"
```

## Architecture

```
┌─────────────────────────────────────┐
│        DomainRegistry               │
├─────────────────────────────────────┤
│  LocalDomain  │  SshDomain (server1)│
│  - PTY        │  - SSH connection   │
│  - Direct I/O │  - Multiplexed      │
│               │  - Remote PTY       │
│  SshDomain    │  SshDomain          │
│  (server2)    │  (server3)          │
└─────────────────────────────────────┘
         │
         ↓
┌─────────────────────────────────────┐
│       SessionManager                │
│  - Session/Tab/Pane hierarchy       │
│  - Routes I/O to domains            │
└─────────────────────────────────────┘
```

## Dependencies

- `russh` 0.44 - SSH protocol implementation
- `russh-keys` 0.44 - SSH key management
- `portable-pty` 0.8 - Local PTY handling
- `tokio` 1.36 - Async runtime
- `parking_lot` 0.12 - Synchronization primitives

## Roadmap

- [ ] Full SSH agent authentication support
- [ ] Host key verification and management
- [ ] Connection compression
- [ ] SOCKS proxy support
- [ ] Jump host / ProxyCommand
- [ ] Docker domain (containers)
- [ ] Kubernetes domain (pods)
- [ ] Mosh integration for high-latency networks

## License

MIT OR Apache-2.0
