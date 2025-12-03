# Session Management

Manage terminal sessions with persistence and restoration.

## Quick Links

For complete session documentation, see:
- [Session Management Documentation](../../../session-management.md) - Complete session guide

## Overview

Scarab provides robust session management with SQLite-backed persistence. Sessions survive client crashes and can be restored after reboot.

## Basic Session Operations

### Creating a Session

Via command palette:
1. Press **Ctrl+Shift+P**
2. Type `session: new`
3. Enter session name

Via configuration:
```toml
[sessions.my-session]
shell = "/bin/bash"
working_directory = "~/projects"
```

### Switching Sessions

1. Press **Ctrl+Shift+P**
2. Type `session: switch`
3. Select session from list

### Saving Sessions

Sessions are automatically saved to:
```
~/.local/share/scarab/sessions.db
```

Manual save:
1. Press **Ctrl+Shift+P**
2. Type `session: save`

### Loading Sessions

Sessions automatically restore on startup. To load a specific session:

```bash
scarab-client --session my-session
```

## Session Persistence

### What's Saved

- Working directory
- Environment variables
- Scrollback buffer
- Window layout (tabs/panes)
- Plugin state

### What's NOT Saved

- Running processes (use tmux/screen for that)
- Shell history (managed by shell)

## Session Configuration

Configure session behavior:

```toml
[session]
auto_save = true
save_interval = 30  # seconds
restore_on_startup = true
max_saved_sessions = 10
```

### Per-Session Settings

Override settings per session:

```toml
[sessions.development]
shell = "/bin/zsh"
working_directory = "~/projects"

[sessions.development.appearance]
theme = "light"
font_size = 16.0

[sessions.production]
shell = "/bin/bash"
working_directory = "/var/www"

[sessions.production.appearance]
theme = "dark"
font_size = 12.0
```

## Session Lifecycle

1. **Create** - Session created in daemon
2. **Attach** - Client connects to session
3. **Detach** - Client disconnects (session persists)
4. **Restore** - Reconnect to existing session
5. **Delete** - Permanently remove session

## Advanced Features

### Session Templates

Create reusable session templates:

```toml
[session_templates.web_dev]
shell = "/bin/zsh"
working_directory = "~/projects/web"
plugins = ["git-status", "npm-watch"]

[session_templates.devops]
shell = "/bin/bash"
working_directory = "~/infra"
plugins = ["kubectl-status", "terraform-watch"]
```

Create from template:
```bash
scarab-client --template web_dev --session my-webapp
```

### Remote Sessions

Connect to remote sessions (future feature):

```bash
scarab-client --remote user@host:session-name
```

## Session Commands

| Command | Description |
|---------|-------------|
| `session: new` | Create new session |
| `session: switch` | Switch to another session |
| `session: save` | Save current session |
| `session: load` | Load saved session |
| `session: delete` | Delete a session |
| `session: rename` | Rename current session |
| `session: list` | List all sessions |

## Troubleshooting

### Session Not Restoring

1. Check session database exists:
   ```bash
   ls -la ~/.local/share/scarab/sessions.db
   ```

2. Enable debug logging:
   ```bash
   RUST_LOG=debug cargo run -p scarab-daemon
   ```

3. Verify auto-restore is enabled:
   ```toml
   [session]
   restore_on_startup = true
   ```

### Session Database Corruption

If the session database is corrupted:

```bash
# Backup old database
mv ~/.local/share/scarab/sessions.db ~/.local/share/scarab/sessions.db.bak

# Restart daemon (creates new database)
cargo run --release -p scarab-daemon
```

## See Also

- [Session Management Documentation](../../../session-management.md) - Complete guide
- [Configuration](./configuration.md) - Configuration options
- [Plugins](./plugins.md) - Session-aware plugins
