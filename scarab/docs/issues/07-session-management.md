# Issue #7: Session Management & Multiplexing

**Phase**: 3A - Advanced Features
**Priority**: 🟢 Medium
**Workstream**: Session Management
**Estimated Effort**: 2 weeks
**Assignee**: Session Management Specialist Agent

---

## 🎯 Objective

Implement tmux-like session management with named sessions, persistence across client disconnects, and multiple PTY support in a single daemon.

---

## 📋 Background

Currently, the daemon manages a single PTY. We need:
- Named sessions (like tmux sessions)
- Session persistence when clients disconnect
- Multiple PTYs in one daemon process
- Tab/split management
- Session attach/detach from different machines

---

## ✅ Acceptance Criteria

- [ ] Named session creation/deletion
- [ ] Session persistence across client disconnects
- [ ] Multiple PTY processes per daemon
- [ ] Session listing and discovery
- [ ] Tab/split management API
- [ ] Session attach/detach commands
- [ ] Session state serialization
- [ ] Session resurrection after daemon restart
- [ ] Resource cleanup when sessions end
- [ ] Support 50+ concurrent sessions

---

## 🔧 Technical Approach

### Step 1: Session Model
```rust
#[derive(Clone, Debug)]
pub struct Session {
    id: SessionId,
    name: String,
    pty: Arc<Mutex<PtyPair>>,
    grid_state: Arc<RwLock<GridState>>,
    created_at: SystemTime,
    last_attached: SystemTime,
    attached_clients: HashSet<ClientId>,
}

pub struct SessionManager {
    sessions: HashMap<SessionId, Session>,
    session_store: SessionStore, // Persistence
}
```

### Step 2: Session Commands
```rust
pub enum SessionCommand {
    Create { name: String },
    Delete { id: SessionId },
    List,
    Attach { id: SessionId, client_id: ClientId },
    Detach { id: SessionId, client_id: ClientId },
    Rename { id: SessionId, new_name: String },
}
```

### Step 3: Persistence Layer
```rust
pub struct SessionStore {
    db_path: PathBuf, // ~/.local/share/scarab/sessions.db
}

impl SessionStore {
    pub fn save_session(&self, session: &Session) -> Result<()> {
        // Serialize session state to SQLite
    }

    pub fn load_sessions(&self) -> Result<Vec<Session>> {
        // Restore sessions from disk
    }
}
```

---

## 📦 Deliverables

1. **Code**: `scarab-daemon/src/session/` module
2. **Persistence**: SQLite-based session storage
3. **API**: Session management commands via IPC
4. **Tests**: Session lifecycle tests
5. **Documentation**: Session management guide

---

## 🔗 Dependencies

- **Depends On**: Issue #3 (IPC) - needs control channel
- **Blocks**: None (advanced feature)

---

## 📚 Resources

- [tmux Source Code](https://github.com/tmux/tmux)
- [Zellij Architecture](https://github.com/zellij-org/zellij)
- [SQLite Rust Bindings](https://docs.rs/rusqlite/)

---

## 🎯 Success Metrics

- ✅ Sessions survive client crashes
- ✅ <10ms session switch time
- ✅ Support 50+ concurrent sessions
- ✅ Zero data loss on daemon restart
- ✅ Memory: <50MB per idle session

---

**Created**: 2025-11-21
**Labels**: `phase-3`, `medium-priority`, `session-management`, `tmux-like`
