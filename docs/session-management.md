# Session Management & Multiplexing

## Overview

Scarab's session management provides tmux-like functionality, allowing multiple terminal sessions to persist across client disconnects and daemon restarts. This enables powerful workflows like remote session management, multi-window setups, and zero-downtime terminal sessions.

## Features

### Core Capabilities

- **Named Sessions**: Create and manage multiple terminal sessions with descriptive names
- **Persistence**: Sessions survive client crashes and daemon restarts
- **Multiplexing**: Run 50+ concurrent sessions in a single daemon process
- **Attach/Detach**: Connect and disconnect from sessions without losing state
- **Session Resurrection**: Automatically restore sessions after daemon restart

### Performance Characteristics

- **Session Switch Time**: <10ms for attach/detach operations
- **Memory Efficiency**: <50MB per idle session
- **Scalability**: Supports 50+ concurrent sessions
- **Zero Data Loss**: Session state persisted to SQLite on every operation

## Architecture

### Components

```
┌─────────────────────────────────────────────┐
│           Session Manager                   │
│  ┌────────────────────────────────────────┐ │
│  │  SessionManager                        │ │
│  │  - Create/Delete/List sessions         │ │
│  │  - Attach/Detach clients               │ │
│  │  - Session lifecycle management        │ │
│  └────────────────────────────────────────┘ │
│                    │                        │
│  ┌────────────────────────────────────────┐ │
│  │  SessionStore (SQLite)                 │ │
│  │  - Persist session metadata            │ │
│  │  - Restore sessions on daemon startup  │ │
│  │  - ~/.local/share/scarab/sessions.db   │ │
│  └────────────────────────────────────────┘ │
│                    │                        │
│  ┌────────────────────────────────────────┐ │
│  │  Session (Individual)                  │ │
│  │  - PTY pair (master/slave)             │ │
│  │  - Grid state (terminal buffer)        │ │
│  │  - Client attachment tracking          │ │
│  └────────────────────────────────────────┘ │
└─────────────────────────────────────────────┘
```

### Session Lifecycle

1. **Creation**: `SessionManager::create_session()` spawns PTY and persists to DB
2. **Active**: Session runs shell process, clients can attach/detach
3. **Detached**: Session continues running even when no clients attached
4. **Resurrection**: On daemon restart, sessions restored from SQLite
5. **Deletion**: Session removed only when explicitly deleted (with no attached clients)

## Usage

### Creating a Session

```rust
use scarab_daemon::session::SessionManager;

let db_path = std::path::PathBuf::from("~/.local/share/scarab/sessions.db");
let manager = SessionManager::new(db_path)?;

// Create a new session
let session_id = manager.create_session(
    "my-session".to_string(),
    80,  // cols
    24   // rows
)?;

println!("Created session: {}", session_id);
```

### Attaching to a Session

```rust
// Attach client to session
let client_id = 123;
manager.attach_client(&session_id, client_id)?;

// Get session details
let session = manager.get_session(&session_id).unwrap();
println!("Attached to: {} (clients: {})",
    session.name,
    session.attached_client_count()
);
```

### Listing Sessions

```rust
// List all sessions
let sessions = manager.list_sessions();

for (id, name, created_at, last_attached, client_count) in sessions {
    println!("{}: {} (clients: {})", id, name, client_count);
}
```

### Detaching from a Session

```rust
// Detach client (session keeps running)
manager.detach_client(&session_id, client_id)?;
```

### Renaming a Session

```rust
// Rename a session
manager.rename_session(&session_id, "new-name".to_string())?;
```

### Deleting a Session

```rust
// Delete session (only if no attached clients)
manager.delete_session(&session_id)?;
```

### Session Resurrection

Sessions are automatically restored when the daemon starts:

```rust
// On daemon startup
let manager = SessionManager::new(db_path)?;
manager.restore_sessions()?;

println!("Restored {} sessions", manager.session_count());
```

## Protocol Messages

### Control Messages

Session commands are sent via IPC using `ControlMessage` enum:

```rust
// Create session
ControlMessage::SessionCreate {
    name: "my-session".into()
}

// Delete session
ControlMessage::SessionDelete {
    id: session_id.into()
}

// List sessions
ControlMessage::SessionList

// Attach to session
ControlMessage::SessionAttach {
    id: session_id.into()
}

// Detach from session
ControlMessage::SessionDetach {
    id: session_id.into()
}

// Rename session
ControlMessage::SessionRename {
    id: session_id.into(),
    new_name: "new-name".into()
}
```

### Response Messages

The daemon responds with `SessionResponse`:

```rust
SessionResponse::Created {
    id: String,
    name: String
}

SessionResponse::Deleted {
    id: String
}

SessionResponse::List {
    sessions: Vec<SessionInfo>
}

SessionResponse::Attached {
    id: String
}

SessionResponse::Detached {
    id: String
}

SessionResponse::Renamed {
    id: String,
    new_name: String
}

SessionResponse::Error {
    message: String
}
```

## Database Schema

Sessions are persisted to SQLite at `~/.local/share/scarab/sessions.db`:

```sql
CREATE TABLE sessions (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    created_at INTEGER NOT NULL,
    last_attached INTEGER NOT NULL,
    cols INTEGER NOT NULL DEFAULT 80,
    rows INTEGER NOT NULL DEFAULT 24
);

CREATE INDEX idx_sessions_name ON sessions(name);
```

## Best Practices

### Session Naming

Use descriptive names for easy identification:
- `dev-server` - Development server session
- `logs-prod` - Production log monitoring
- `build-process` - Long-running build tasks

### Client Management

Track which clients are attached to avoid accidental disconnects:

```rust
let session = manager.get_session(&id).unwrap();
if session.has_attached_clients() {
    println!("Warning: {} clients attached", session.attached_client_count());
}
```

### Cleanup

Periodically cleanup old detached sessions:

```rust
// Remove sessions detached for more than 7 days
let removed = manager.cleanup_detached_sessions(7 * 24 * 60 * 60)?;
println!("Cleaned up {} old sessions", removed);
```

### Error Handling

Always handle session operations gracefully:

```rust
match manager.attach_client(&id, client_id) {
    Ok(_) => println!("Attached to session"),
    Err(e) => {
        eprintln!("Failed to attach: {}", e);
        // Fall back to creating new session
        let new_id = manager.create_session("fallback".to_string(), 80, 24)?;
        manager.attach_client(&new_id, client_id)?;
    }
}
```

## Performance Considerations

### Memory Usage

Each session maintains:
- PTY pair (~1KB)
- Grid state (~200KB for 200x100 grid)
- Metadata (~1KB)

Total: ~202KB per session in memory, plus scrollback buffer.

### Session Switch Time

Attach/detach operations are optimized for <10ms:
- No PTY reallocation
- Atomic state updates
- Lock-free grid access where possible

### Scalability

The system is designed to handle 50+ concurrent sessions:
- Efficient HashMap lookups (O(1))
- Lazy PTY initialization
- Shared VTE parser state

## Troubleshooting

### Session Not Found

```
Error: Session not found: abc-123
```

**Solution**: Session may have been deleted or ID is incorrect. List sessions to verify.

### Cannot Delete Session

```
Error: Cannot delete session with attached clients
```

**Solution**: Detach all clients before deletion:

```rust
let session = manager.get_session(&id).unwrap();
let clients: Vec<u64> = session.attached_clients.read().iter().copied().collect();
for client_id in clients {
    manager.detach_client(&id, client_id)?;
}
manager.delete_session(&id)?;
```

### Database Locked

```
Error: database is locked
```

**Solution**: Ensure only one daemon instance is running. Check for stale processes:

```bash
ps aux | grep scarab-daemon
killall scarab-daemon
```

## Future Enhancements

Planned features for session management:

- **Tab/Split Management**: Multiple panes per session
- **Session Groups**: Organize sessions into workspaces
- **Session Templates**: Quick-start configurations
- **Remote Sessions**: Attach from different machines
- **Session Recording**: Playback session history
- **Resource Limits**: Per-session CPU/memory caps

## Related Documentation

- [IPC Protocol](./ipc-protocol.md)
- [PTY Management](./pty-management.md)
- [VTE Parser](./vte-parser.md)
- [Client Architecture](./client-architecture.md)

---

**Implementation Status**: ✅ Complete (Issue #7)
**Last Updated**: 2025-11-21
**Version**: 0.1.0
