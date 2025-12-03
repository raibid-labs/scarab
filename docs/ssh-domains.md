# SSH Domains - Remote Terminal Multiplexing

SSH Domains enable Scarab to spawn and manage terminal panes on remote servers over SSH connections. This provides:

- **Persistent remote sessions** - Sessions survive local client disconnects
- **Connection multiplexing** - Single SSH connection supports multiple panes
- **Automatic reconnection** - Network failures are handled transparently
- **Cross-domain splits** - Mix local and remote panes in the same session

## Architecture

Scarab's domain system abstracts terminal execution environments:

```
┌─────────────────────────────────────────────────────────┐
│                   Domain Registry                        │
├─────────────────────────────────────────────────────────┤
│  LocalDomain          │  SshDomain (server1)            │
│  - PTY processes      │  - SSH connection               │
│  - Direct I/O         │  - Multiplexed channels         │
│                       │  - Remote PTY allocation        │
│  SshDomain (server2)  │  SshDomain (server3)            │
│  - Another server     │  - Yet another server           │
└─────────────────────────────────────────────────────────┘
```

Each domain implements the `Domain` trait, providing:
- `spawn_pane()` - Create new terminal pane
- `write_to_pane()` - Send input data
- `read_from_pane()` - Receive output data
- `resize_pane()` - Update terminal dimensions
- `close_pane()` - Terminate pane
- `reconnect()` - Re-establish connection (for remote domains)

## Configuration

### TOML Configuration

Add SSH domains to your `~/.config/scarab/config.toml`:

```toml
# Local machine (always available, no configuration needed)

# SSH domain with agent authentication
[[ssh_domains]]
id = "dev"
name = "Development Server"
host = "dev.example.com"
port = 22
user = "developer"
auth_type = "agent"
connect_timeout = 10
forward_agent = false

# SSH domain with public key
[[ssh_domains]]
id = "prod"
name = "Production Server"
host = "prod.example.com"
port = 2222
user = "admin"
auth_type = "publickey"
key_path = "/home/user/.ssh/prod_key"
passphrase = "optional_passphrase"
remote_cwd = "/var/www"

# SSH domain with password (less secure)
[[ssh_domains]]
id = "legacy"
name = "Legacy Server"
host = "legacy.internal"
user = "ops"
auth_type = "password"
password = "secret123"  # Consider using environment variable instead
```

### Fusabi Configuration

Using Fusabi script (`~/.config/scarab/config.fsx`):

```fsharp
// SSH domain configuration
let devServer = {
    id = "dev"
    name = "Development Server"
    host = "dev.example.com"
    port = 22
    user = "developer"
    auth_type = "agent"
    connect_timeout = 10
    forward_agent = false
    remote_cwd = None
}

let prodServer = {
    id = "prod"
    name = "Production"
    host = "prod.example.com"
    user = "admin"
    auth_type = "publickey"
    key_path = "/home/user/.ssh/prod_key"
    passphrase = None
    remote_cwd = Some "/var/www"
}

config.ssh_domains <- [devServer; prodServer]
```

## Usage

### Spawning Remote Panes

From Scarab command palette (`Ctrl+Shift+P`):

1. **New Pane in Domain**
   - Select domain from list (local, dev, prod, etc.)
   - Pane spawns on selected domain

2. **Split Pane** (within same domain)
   - Horizontal split: `Ctrl+|`
   - Vertical split: `Ctrl+-`
   - New pane inherits domain from current pane

3. **Cross-Domain Split**
   - Command palette: "Split Pane in Different Domain"
   - Select target domain
   - Creates split with pane in different domain

### Connection Management

SSH domains automatically:
- Connect on first pane spawn
- Multiplex channels over single connection
- Reconnect on network failure
- Persist sessions across reconnects

Monitor connection status:
```bash
# From Scarab command palette
:domain list
# Shows: local (connected), dev (connected), prod (reconnecting)
```

## Authentication Methods

### 1. SSH Agent (Recommended)

Most secure and convenient:

```toml
[[ssh_domains]]
id = "myserver"
auth_type = "agent"
```

Setup:
```bash
# Start SSH agent
eval "$(ssh-agent -s)"

# Add your key
ssh-add ~/.ssh/id_rsa

# Verify
ssh-add -l
```

### 2. Public Key File

Direct key file authentication:

```toml
[[ssh_domains]]
id = "myserver"
auth_type = "publickey"
key_path = "/home/user/.ssh/myserver_key"
passphrase = "optional_passphrase"  # Omit if key not encrypted
```

### 3. Password (Least Secure)

Not recommended for production:

```toml
[[ssh_domains]]
id = "myserver"
auth_type = "password"
password = "secret"  # Use environment variable: ${MYSERVER_PASSWORD}
```

## Advanced Features

### Agent Forwarding

Forward your SSH agent to remote server:

```toml
[[ssh_domains]]
id = "jumphost"
forward_agent = true  # Enable agent forwarding
```

Allows remote server to authenticate to other servers using your local keys.

### Custom Working Directory

Set default directory for new panes:

```toml
[[ssh_domains]]
id = "webserver"
remote_cwd = "/var/www/html"
```

All panes spawned in this domain start in `/var/www/html`.

### Connection Timeout

Adjust timeout for slow networks:

```toml
[[ssh_domains]]
id = "slowserver"
connect_timeout = 30  # seconds (default: 10)
```

## Troubleshooting

### Connection Failures

**Problem**: "SSH connection failed"

Solutions:
1. Verify server is reachable: `ping dev.example.com`
2. Test SSH manually: `ssh user@dev.example.com`
3. Check Scarab logs: `~/.local/share/scarab/daemon.log`
4. Increase `connect_timeout` if network is slow

### Authentication Failures

**Problem**: "Authentication failed"

Solutions:
1. **Agent auth**: Ensure agent is running and key is loaded
   ```bash
   ssh-add -l  # List loaded keys
   ```

2. **Public key**: Verify key path and permissions
   ```bash
   ls -l ~/.ssh/mykey
   # Should be: -rw------- (600)
   ```

3. **Password**: Check for typos, try manual SSH first

### Host Key Verification

**Problem**: "Host key verification failed"

Currently, Scarab accepts all host keys (similar to `ssh -o StrictHostKeyChecking=no`).

For production, manually accept host keys first:
```bash
ssh user@dev.example.com  # Accept host key interactively
```

Future versions will implement proper host key verification.

### Reconnection Issues

**Problem**: Panes become unresponsive after network drop

Scarab automatically attempts reconnection. If issues persist:

1. Check domain status: `:domain list`
2. Manually reconnect: `:domain reconnect dev`
3. Restart daemon if needed

## Performance Considerations

### Connection Multiplexing

Single SSH connection per domain, shared by all panes:
- **Pros**: Lower overhead, faster pane spawning
- **Cons**: Connection failure affects all panes in domain

### Bandwidth

Remote panes consume bandwidth for:
- Terminal output (text, minimal)
- Input commands (keystrokes, negligible)
- Images (iTerm2 protocol, can be significant)

Optimize for slow connections:
- Disable iTerm2 image protocol for remote panes
- Reduce scrollback buffer size
- Use compression (future feature)

### Latency

Input latency depends on network round-trip time:
- **Local**: <1ms
- **LAN**: 1-10ms
- **Internet**: 20-100ms
- **High latency**: 100-500ms (noticeable lag)

For high-latency connections, consider:
- Using `mosh` wrapper (future feature)
- Local development with rsync deployment
- Jump host in closer datacenter

## Security Best Practices

1. **Use SSH Agent** for key management
2. **Encrypt private keys** with strong passphrases
3. **Rotate keys** periodically
4. **Limit sudo access** on remote servers
5. **Use jump hosts** for sensitive servers
6. **Audit connections** regularly

Never:
- Store passwords in plain text config files
- Use same key for all servers
- Disable host key checking in production
- Forward agent to untrusted servers

## Examples

### Development Workflow

```toml
# Local development
[[ssh_domains]]
id = "local"
# No configuration needed

# Staging environment
[[ssh_domains]]
id = "staging"
host = "staging.myapp.com"
user = "developer"
auth_type = "agent"
remote_cwd = "/home/developer/myapp"

# Production (limited access)
[[ssh_domains]]
id = "prod"
host = "prod.myapp.com"
user = "readonly"
auth_type = "publickey"
key_path = "/home/user/.ssh/prod_readonly"
```

Workflow:
1. Edit code locally
2. Test in staging domain
3. View logs in prod domain (read-only)

### Multi-Server Monitoring

```toml
[[ssh_domains]]
id = "web1"
host = "web1.example.com"
user = "ops"
auth_type = "agent"

[[ssh_domains]]
id = "web2"
host = "web2.example.com"
user = "ops"
auth_type = "agent"

[[ssh_domains]]
id = "db"
host = "db.example.com"
user = "ops"
auth_type = "agent"
```

Create layout:
- Tab 1: Local
- Tab 2: web1 (top-htop), web2 (bottom-htop)
- Tab 3: db (tail -f logs)

## Roadmap

Future enhancements:

- [ ] Host key verification and management
- [ ] Connection compression
- [ ] SOCKS proxy support
- [ ] Jump host / ProxyCommand
- [ ] Docker domain (containers as execution environment)
- [ ] Kubernetes domain (pods as execution environment)
- [ ] Mosh integration for high-latency networks
- [ ] Connection pooling and sharing across Scarab instances

## API Reference

See [Domain Trait Documentation](../crates/scarab-session/src/domain.rs) for implementation details.
