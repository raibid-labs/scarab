# Workflow Integration

**Integrate Scarab into your daily development workflows**

---

## Table of Contents

1. [Git Workflow](#git-workflow)
2. [Docker Development](#docker-development)
3. [SSH Session Management](#ssh-session-management)
4. [Multi-Language Development](#multi-language-development)
5. [Terminal Multiplexing](#terminal-multiplexing)

---

## Git Workflow

Scarab includes powerful Git integrations through the `git-status` plugin.

### Automatic Git Status Display

The git-status plugin shows repository information in real-time:

```toml
# ~/.config/scarab/config.toml
[plugins.git-status]
enabled = true
update_interval = 1000  # Update every second
show_dirty_indicator = true
position = "top-right"
show_branch = true
show_ahead_behind = true
```

**Display Information:**
- Current branch name
- Dirty indicator (`*` for uncommitted changes)
- Ahead/behind count (commits ahead/behind remote)
- Stash count

### Git Command Shortcuts

Create keybindings for common Git operations:

```toml
[keybindings]
"ctrl+g ctrl+s" = "git_status"      # Run git status
"ctrl+g ctrl+d" = "git_diff"        # Show git diff
"ctrl+g ctrl+l" = "git_log"         # Show git log
"ctrl+g ctrl+p" = "git_push"        # Push changes
"ctrl+g ctrl+u" = "git_pull"        # Pull changes
```

### Interactive Git Plugin Example

Create a custom Git plugin for enhanced workflows:

```fsharp
// ~/.config/scarab/plugins/git-enhanced.fsx
open Scarab.PluginApi
open System.Diagnostics

let metadata = {
    Name = "git-enhanced"
    Version = "1.0.0"
    Description = "Enhanced Git workflow integration"
    Author = "Your Name"
    Homepage = None
    ApiVersion = "0.1.0"
    MinScarabVersion = "0.1.0"
}

// Run git command and return output
let runGitCommand (args: string) : string =
    let psi = ProcessStartInfo("git", args)
    psi.RedirectStandardOutput <- true
    psi.UseShellExecute <- false
    use proc = Process.Start(psi)
    proc.WaitForExit()
    proc.StandardOutput.ReadToEnd()

// Get current branch
let getCurrentBranch () : string option =
    try
        let branch = runGitCommand "rev-parse --abbrev-ref HEAD"
        Some (branch.Trim())
    with _ -> None

// Check if repository is dirty
let isDirty () : bool =
    try
        let status = runGitCommand "status --porcelain"
        not (System.String.IsNullOrWhiteSpace(status))
    with _ -> false

// Update git status overlay
let updateOverlay (ctx: PluginContext) =
    match getCurrentBranch() with
    | Some branch ->
        let dirty = if isDirty() then "*" else ""
        let text = sprintf " git:%s%s " branch dirty

        ctx.QueueCommand(RemoteCommand.DrawOverlay {
            Id = 10001UL
            X = 0us  // Top-right corner (calculated by client)
            Y = 0us
            Text = text
            Style = {
                Fg = 0xFFFFFFFFu  // White text
                Bg = 0x00AA00FFu  // Green background
                ZIndex = 100.0f
            }
        })
    | None -> ()

Plugin.Register {
    Metadata = metadata
    OnLoad = Some (fun ctx -> async {
        updateOverlay ctx
        return Ok ()
    })
    OnPostCommand = Some (fun ctx cmd -> async {
        // Update after git commands
        if cmd.StartsWith("git ") then
            updateOverlay ctx
        return Ok ()
    })
    OnResize = Some (fun ctx -> async {
        updateOverlay ctx
        return Ok ()
    })
    // ... other hooks ...
}
```

### Git Commit Message Template

Use Scarab's command palette to open a commit message editor:

```
Ctrl+Shift+P > git commit
```

This opens your configured editor with a template:

```
# Type: feat|fix|docs|style|refactor|test|chore
# Scope: (optional)
#
# Subject: Brief description (50 chars max)
#
# Body: Detailed description (72 chars per line)
#
# Footer: Issue references, breaking changes
```

---

## Docker Development

Scarab integrates seamlessly with Docker workflows.

### Docker Container Management Plugin

```fsharp
// ~/.config/scarab/plugins/docker-manager.fsx
open Scarab.PluginApi
open System.Diagnostics

let metadata = {
    Name = "docker-manager"
    Version = "1.0.0"
    Description = "Docker container management"
    Author = "Your Name"
    Homepage = None
    ApiVersion = "0.1.0"
    MinScarabVersion = "0.1.0"
}

// List running containers
let listContainers () : string list =
    try
        let psi = ProcessStartInfo("docker", "ps --format '{{.Names}}'")
        psi.RedirectStandardOutput <- true
        psi.UseShellExecute <- false
        use proc = Process.Start(psi)
        proc.WaitForExit()

        proc.StandardOutput.ReadToEnd()
            .Split('\n')
            |> Array.filter (fun s -> not (System.String.IsNullOrWhiteSpace(s)))
            |> Array.toList
    with _ -> []

// Provide Docker commands for command palette
let getDockerCommands () : Command list =
    [
        { Name = "docker ps"; Description = "List running containers" }
        { Name = "docker images"; Description = "List images" }
        { Name = "docker compose up"; Description = "Start services" }
        { Name = "docker compose down"; Description = "Stop services" }
        { Name = "docker logs"; Description = "View container logs" }
    ]

Plugin.Register {
    Metadata = metadata
    GetCommands = getDockerCommands
    OnLoad = Some (fun ctx -> async {
        ctx.Log(LogLevel.Info, "Docker plugin loaded")
        return Ok ()
    })
    // ... other hooks ...
}
```

### Docker Compose Integration

Add shortcuts for common Docker Compose operations:

```toml
[keybindings]
"ctrl+d ctrl+u" = "docker_compose_up"
"ctrl+d ctrl+d" = "docker_compose_down"
"ctrl+d ctrl+l" = "docker_compose_logs"
"ctrl+d ctrl+r" = "docker_compose_restart"
```

### Container Log Streaming

Stream container logs with color highlighting:

```bash
# In Scarab terminal
docker logs -f mycontainer

# Or use the command palette
Ctrl+Shift+P > docker logs mycontainer
```

Scarab will automatically:
- Highlight ERROR/WARN/INFO levels
- Detect URLs in logs (use Ctrl+Shift+O to open)
- Provide scrollback for log history

### Development Container Workflow

1. **Start development environment:**
   ```bash
   cd ~/projects/myapp
   docker compose up -d
   ```

2. **Attach to container:**
   ```bash
   docker exec -it myapp_web_1 /bin/bash
   ```

3. **Run commands inside container:**
   ```bash
   npm run dev
   cargo build
   ```

4. **View logs in separate Scarab tab:**
   ```bash
   # Ctrl+Shift+T (new tab)
   docker compose logs -f
   ```

---

## SSH Session Management

Scarab makes managing SSH sessions effortless.

### SSH Connection Manager Plugin

```fsharp
// ~/.config/scarab/plugins/ssh-manager.fsx
open Scarab.PluginApi

let metadata = {
    Name = "ssh-manager"
    Version = "1.0.0"
    Description = "SSH connection management"
    Author = "Your Name"
    Homepage = None
    ApiVersion = "0.1.0"
    MinScarabVersion = "0.1.0"
}

// Common SSH hosts
let sshHosts = [
    ("prod-web", "user@production.example.com")
    ("staging", "user@staging.example.com")
    ("dev-db", "user@dev-database.local")
    ("bastion", "user@bastion.example.com")
]

// Generate SSH commands for palette
let getSshCommands () : Command list =
    sshHosts
    |> List.map (fun (name, host) -> {
        Name = sprintf "ssh %s" name
        Description = sprintf "Connect to %s" host
    })

Plugin.Register {
    Metadata = metadata
    GetCommands = getSshCommands
    // ... other hooks ...
}
```

### SSH Config Integration

Scarab reads your `~/.ssh/config`:

```
# ~/.ssh/config
Host prod-web
    HostName production.example.com
    User deploy
    Port 22
    IdentityFile ~/.ssh/id_rsa_production

Host staging
    HostName staging.example.com
    User deploy
    ProxyJump bastion
```

Then use the command palette:

```
Ctrl+Shift+P > ssh prod-web
```

### SSH Connection Persistence

Scarab sessions persist across disconnections:

1. **Connect to server:**
   ```bash
   ssh prod-web
   ```

2. **If connection drops:**
   - Scarab daemon keeps session alive
   - Reconnect by launching a new client
   - Session state is preserved

3. **Or use SSH multiplexing:**
   ```
   # ~/.ssh/config
   Host *
       ControlMaster auto
       ControlPath ~/.ssh/sockets/%r@%h-%p
       ControlPersist 600
   ```

### Multi-Hop SSH

Connect through jump hosts easily:

```bash
# Direct
ssh -J bastion prod-web

# Or via config
ssh prod-web  # Automatically uses ProxyJump
```

---

## Multi-Language Development

Scarab supports development across multiple languages.

### Language-Specific Plugins

**Rust Development:**
```toml
[plugins.rust-analyzer]
enabled = true
auto_check = true
show_errors = true
```

**Python Development:**
```toml
[plugins.python-venv]
enabled = true
auto_activate = true  # Auto-activate virtualenv
show_venv_indicator = true
```

**Node.js Development:**
```toml
[plugins.node-manager]
enabled = true
show_node_version = true
show_npm_scripts = true
```

### Project-Specific Configuration

Create `.scarab.toml` in project root:

```toml
# ~/projects/rust-project/.scarab.toml
[terminal]
working_directory = "~/projects/rust-project"

[plugins]
enabled = ["rust-analyzer", "git-status"]

[keybindings]
"ctrl+b" = "cargo build"
"ctrl+t" = "cargo test"
"ctrl+r" = "cargo run"

[plugins.rust-analyzer]
clippy_on_save = true
```

### Build Task Integration

Run build tasks from the command palette:

```
Ctrl+Shift+P > cargo build --release
Ctrl+Shift+P > npm run build
Ctrl+Shift+P > make all
```

### Test Running

Quick shortcuts for testing:

```toml
[keybindings]
"ctrl+shift+t" = "run_tests"  # Language-specific
```

Scarab will detect your project type and run:
- `cargo test` for Rust
- `npm test` for Node.js
- `pytest` for Python
- `go test` for Go

---

## Terminal Multiplexing

While Scarab has built-in session management, you can still use tmux/screen.

### Scarab vs tmux

**Scarab Advantages:**
- GPU-accelerated rendering
- Plugin system
- Modern UI (command palette, link hints)
- Zero-copy IPC

**tmux Advantages:**
- Mature and battle-tested
- Remote session sharing
- Extensive configuration

**Best of Both Worlds:**

Use tmux inside Scarab for remote sessions:

```bash
# Local: Use Scarab
scarab-client

# Remote: Use tmux
ssh server
tmux new -s dev
# ... work ...
# Detach: Ctrl+B, D
```

### Scarab Session Management

Scarab has native session support (no tmux needed):

```bash
# Create named session
Ctrl+Shift+P > session new "frontend-dev"

# List sessions
Ctrl+Shift+P > session list

# Switch sessions
Ctrl+Shift+P > session switch "frontend-dev"

# Rename session
Ctrl+Shift+P > session rename "backend-dev"
```

### Multiple Scarab Windows

Run multiple Scarab windows for different projects:

```bash
# Terminal 1
cd ~/projects/frontend
scarab-daemon --session frontend
scarab-client --session frontend

# Terminal 2
cd ~/projects/backend
scarab-daemon --session backend
scarab-client --session backend
```

---

## Advanced Workflows

### Monorepo Development

Working in a monorepo? Create workspace-specific configs:

```
monorepo/
├── .scarab.toml          # Root config
├── frontend/
│   └── .scarab.toml      # Frontend overrides
├── backend/
│   └── .scarab.toml      # Backend overrides
└── shared/
    └── .scarab.toml      # Shared overrides
```

**Root config:**
```toml
# monorepo/.scarab.toml
[plugins]
enabled = ["git-status", "monorepo-tools"]
```

**Frontend config:**
```toml
# monorepo/frontend/.scarab.toml
[theme]
name = "one-light"  # Light theme for frontend

[keybindings]
"ctrl+b" = "npm run build"
"ctrl+t" = "npm test"
```

### CI/CD Integration

Monitor CI/CD pipelines from your terminal:

```fsharp
// ~/.config/scarab/plugins/ci-monitor.fsx
// Monitors GitHub Actions, GitLab CI, Jenkins, etc.

let checkCiStatus () : string =
    // Query CI API
    // Return status: "passing", "failing", "running"
    "passing"

// Update overlay with CI status
let updateCiOverlay (ctx: PluginContext) =
    let status = checkCiStatus()
    let (text, bgColor) =
        match status with
        | "passing" -> ("✓ CI Passing", 0x00AA00FFu)
        | "failing" -> ("✗ CI Failed", 0xFF0000FFu)
        | "running" -> ("⟳ CI Running", 0xFFAA00FFu)
        | _ -> ("? CI Unknown", 0x888888FFu)

    ctx.QueueCommand(RemoteCommand.DrawOverlay {
        Id = 10002UL
        X = 0us
        Y = 0us
        Text = sprintf " %s " text
        Style = { Fg = 0xFFFFFFFFu; Bg = bgColor; ZIndex = 100.0f }
    })
```

### Database Client Integration

Connect to databases directly:

```bash
# PostgreSQL
psql -h localhost -U user -d database

# MySQL
mysql -h localhost -u user -p database

# MongoDB
mongo localhost:27017/database

# Redis
redis-cli
```

Scarab will:
- Highlight SQL syntax
- Detect table names (link hints)
- Provide scrollback for large result sets

---

## Tips and Tricks

### Quick Command Execution

Use the command palette as a quick launcher:

```
Ctrl+Shift+P > ls -la
Ctrl+Shift+P > git status
Ctrl+Shift+P > docker ps
```

### Command History

Scarab maintains command history across sessions:

```bash
# Search history with fzf
Ctrl+R

# Or use command palette
Ctrl+Shift+P > history search
```

### Environment Switching

Quickly switch between environments:

```toml
[keybindings]
"alt+1" = "env development"
"alt+2" = "env staging"
"alt+3" = "env production"
```

### Script Automation

Create shell scripts for common workflows:

```bash
#!/bin/bash
# ~/bin/deploy.sh

echo "Deploying to production..."
git pull origin main
docker compose -f docker-compose.prod.yml up -d
echo "Deployment complete!"
```

Run from command palette:
```
Ctrl+Shift+P > ~/bin/deploy.sh
```

---

## Next Steps

- **[Plugin Development](./03-plugin-development.md)** - Create custom workflow plugins
- **[API Reference](../../docs/api/)** - Detailed API documentation
- **[Example Plugins](../../examples/plugins/)** - Learn from examples

---

**Back to:** [Customization Guide](./02-customization.md) | [Getting Started](./01-getting-started.md) | [README](../../README.md)
