# Technical Audit: Raibid-Labs Ecosystem

**Date**: 2025-12-11
**Scope**: Scarab, Scryforge, Sigilforge, Fusabi-lang, ratatui-testlib, scarab-nav

---

## 1. Executive Summary

This audit examines six interconnected projects in the raibid-labs ecosystem to assess integration feasibility for using Scryforge and Sigilforge as Scarab terminal plugins. The primary goal is achieving seamless YouTube browsing within Scarab with OAuth authentication managed by Sigilforge.

**Key Findings**:
- All projects share compatible architecture patterns (daemon+client, async Rust, trait-based APIs)
- Scarab's plugin system is mature and ready for TUI plugin embedding
- Scryforge/Sigilforge integration requires adapting TUI rendering to Scarab's GPU pipeline
- Fusabi ecosystem needs upstream work to version-align with Scarab requirements
- Navigation (scarab-nav) provides Vimium-style hints across all TUI surfaces

---

## 2. Scarab Terminal Architecture

### 2.1 Overview

Scarab is a high-performance, split-process terminal emulator with:
- **Daemon**: PTY server using `alacritty_terminal` VTE parser
- **Client**: Bevy 0.15 GPU-accelerated renderer using `cosmic-text`
- **IPC**: Zero-copy shared memory (`#[repr(C)]` structs with `bytemuck`)

### 2.2 Workspace Structure

```
scarab/
├── crates/
│   ├── scarab-daemon/        # Headless server, PTY owner
│   ├── scarab-client/        # Bevy GUI renderer
│   ├── scarab-protocol/      # IPC definitions (no_std)
│   ├── scarab-plugin-api/    # Plugin traits and capabilities
│   ├── scarab-config/        # Configuration management
│   ├── scarab-nav/           # Vimium-style navigation
│   ├── scarab-tabs/          # Tab management plugin
│   └── scarab-panes/         # Pane splitting plugin
```

### 2.3 Plugin System Architecture

**Core Trait** (`scarab-plugin-api/src/plugin.rs`):
```rust
pub trait Plugin: Send + Sync {
    fn metadata(&self) -> &PluginMetadata;
    async fn on_load(&mut self, ctx: &mut PluginContext) -> Result<()>;
    async fn on_output(&mut self, line: &str, ctx: &PluginContext) -> Result<Action>;
    async fn on_remote_command(&mut self, id: &str, ctx: &PluginContext) -> Result<()>;
    // ... more hooks
}
```

**Plugin Context** provides:
- Terminal grid read/write access
- Notification APIs
- Command queuing via `RemoteCommand` enum
- Navigation focusable registration

**Capability System**:
- `OutputFiltering`, `InputFiltering`, `ShellExecution`
- `FileSystem`, `Network`, `Clipboard`, `ProcessSpawn`
- `UiOverlay`, `MenuRegistration`, `CommandRegistration`

### 2.4 Navigation API

**PluginFocusable Structure**:
```rust
pub struct PluginFocusable {
    pub x: u16, pub y: u16,
    pub width: u16, pub height: u16,
    pub label: String,
    pub action: PluginFocusableAction,
}

pub enum PluginFocusableAction {
    OpenUrl(String),
    OpenFile(String),
    Custom(String),
}
```

**Extension Trait** (`NavigationExt`):
- `enter_hint_mode()` - Activate Vimium-style labels
- `register_focusable(region)` - Add interactive element
- `unregister_focusable(id)` - Remove element

### 2.5 Status Bar System

**RenderItem Types** (rich formatting):
```rust
pub enum RenderItem {
    Text(String), Icon(String),
    Foreground(Color), Background(Color),
    Bold, Italic, Underline(UnderlineStyle),
    Spacer, Padding(u8), Separator(String),
}
```

**Sides**: `StatusBarSide::Left`, `StatusBarSide::Right`

---

## 3. Scryforge Architecture

### 3.1 Overview

Scryforge is a Fusabi-powered TUI information rolodex with:
- **Daemon**: Provider registry, sync management, JSON-RPC API
- **TUI**: Ratatui 0.29 client with explorer-style navigation

### 3.2 Provider System

**Core Trait**:
```rust
#[async_trait]
pub trait Provider: Send + Sync {
    fn id(&self) -> &'static str;
    fn name(&self) -> &'static str;
    async fn sync(&self) -> Result<SyncResult>;
    fn capabilities(&self) -> ProviderCapabilities;
    async fn available_actions(&self, item: &Item) -> Result<Vec<Action>>;
}
```

**Capability Traits**:
- `HasFeeds` - RSS/YouTube subscriptions
- `HasCollections` - Playlists, folders
- `HasSavedItems` - Watch Later, bookmarks
- `HasCommunities` - Subreddits, forums
- `HasTasks` - Todo lists

### 3.3 YouTube Provider Status

**Location**: `providers/provider-youtube/src/lib.rs`

**API Integration** (YouTube Data API v3):
- `/subscriptions` - Subscribed channels
- `/playlists` - User playlists
- `/playlistItems` - Playlist contents
- `/videos` - Video details

**Current Status**: Implemented but requires OAuth tokens from Sigilforge

**Available Actions**:
- `OpenInBrowser` - Launch in browser
- `CopyLink` - Copy URL
- `Save` - Add to Watch Later

### 3.4 TUI Widget Library

**Widgets** (`fusabi-tui-widgets`):
- `StreamListWidget` - Sidebar with unread badges
- `ItemListWidget` - Scrollable item list
- `PreviewWidget` - Rich item preview
- `StatusBarWidget` - Connection status
- `OmnibarWidget` - Command palette
- `ToastWidget` - Notifications

### 3.5 Integration Status

| Component | Status |
|-----------|--------|
| Core types (Stream, Item) | Complete |
| Provider registry | Complete |
| TUI widgets | Complete |
| Daemon API | Stubbed |
| YouTube provider | Implemented (needs auth) |
| Sigilforge client | Implemented |

---

## 4. Sigilforge Architecture

### 4.1 Overview

Sigilforge is a credential management daemon providing:
- OAuth token storage and refresh
- Keyring backend (macOS Keychain, Linux Secret Service)
- JSON-RPC API over Unix socket

### 4.2 Core Types

```rust
pub struct Token {
    access_token: Secret,
    token_type: String,
    expires_at: Option<DateTime<Utc>>,
    scopes: Vec<String>,
}

pub struct CredentialRef {
    service: ServiceId,
    account: AccountId,
    credential_type: CredentialType,
}
```

### 4.3 Storage Backends

1. **KeyringStore** - OS-level secure storage
2. **MemoryStore** - Development/testing
3. **EncryptedFileStore** - Git-friendly encrypted files (planned)

### 4.4 OAuth Flows

**Supported**:
- PKCE Authorization Code Flow (native apps)
- Device Code Flow (headless systems)

**Pre-configured Providers**: GitHub, Spotify, Google/YouTube

### 4.5 Client API

```rust
pub trait TokenProvider: Send + Sync {
    async fn get_token(&self, service: &str, account: &str) -> Result<AccessToken>;
    async fn resolve(&self, reference: &str) -> Result<SecretValue>;
}

// Usage
let client = SigilforgeClient::new();
let token = client.get_token("youtube", "personal").await?;
```

### 4.6 Security Audit Findings

**Critical Issues**:
1. No socket authentication (any process can request tokens)
2. No per-account authorization
3. Socket permission race conditions

**Recommendations**:
- Add peer credential verification
- Implement capability-based access
- Add rate limiting

---

## 5. Fusabi-Lang Ecosystem

### 5.1 Core Language (v0.21.0)

**Crates**:
- `fusabi-frontend` - Lexer, parser, type checker
- `fusabi-vm` - Stack-based bytecode interpreter

**Features**:
- F# dialect with pattern matching
- Type inference (Hindley-Milner)
- Discriminated unions, records, options
- Pipeline operator (`|>`)

### 5.2 Host Integration (`fusabi-host`)

**Key Features**:
- `EnginePool` - Thread-safe concurrent execution
- Capability-based sandboxing
- Typed host function registration

```rust
engine.registry_mut().register("add", host_fn!(add(a: i64, b: i64) -> i64 {
    a + b
}));
```

### 5.3 Plugin Runtime (`fusabi-plugin-runtime`)

**Features**:
- Manifest-driven loading (`.toml`)
- Hot reload with filesystem watching
- Capability enforcement

**Manifest Example**:
```toml
name = "my-plugin"
version = "1.0.0"
api-version = { major = 0, minor = 18, patch = 0 }
capabilities = ["fs:read", "net:request"]
source = "main.fsx"
```

### 5.4 Version Alignment Issues

| Package | Current Version | Scarab Needs |
|---------|-----------------|--------------|
| fusabi-vm | 0.21.0 | 0.21.0 |
| fusabi-frontend | 0.21.0 | 0.21.0 |
| bevy-fusabi | 0.1.4 (uses 0.17.0) | 0.21.0 |
| fusabi-tui | 0.2.0 (uses 0.16.0) | 0.21.0 |
| fusabi-host | 0.1.0 (0.18-0.19) | 0.21.0 |

**Action Required**: Upstream updates to align all packages with v0.21.0

---

## 6. Scarab-Nav Protocol

### 6.1 Architecture

**Communication**: Protobuf over Unix Domain Socket

**Protocol Messages**:
```protobuf
message UpdateLayout {
    string window_id = 1;
    repeated InteractiveElement elements = 2;
}

message InteractiveElement {
    string id = 1;
    uint32 x, y, width, height = 2-5;
    ElementType type = 6;
    string description = 7;
}
```

### 6.2 Integration Pattern

```rust
// During render
let mut nav_recorder = NavRecorder::new();
nav_recorder.register(area.x, area.y, area.width, area.height,
    ElementType::Button, "Video Title", None);
nav_client.update(nav_recorder.finish());
```

### 6.3 Hint Generation

Labels generated from "asdfghjklqwertyuiopzxcvbnm":
- 1-26 elements: single char (a, s, d, f, ...)
- 27+ elements: two char (aa, ab, ac, ...)

---

## 7. ratatui-testlib Capabilities

### 7.1 Overview

Comprehensive TUI testing library with:
- PTY-based full application testing
- Bevy ECS integration testing
- Split-process (daemon+client) testing
- Graphics protocol verification (Sixel, Kitty, iTerm2)

### 7.2 Key Features

**Test Harnesses**:
- `TuiTestHarness` - PTY-based
- `ScarabTestHarness` - IPC + shared memory
- `BevyTuiTestHarness` - Bevy frame-by-frame

**Assertions**:
```rust
harness.wait_for_text("Ready")?;
harness.assert_text_at(5, 10, "Status")?;
harness.send_key(KeyCode::Enter)?;
```

**Navigation Testing** (`src/navigation.rs`):
- `enter_hint_mode()`
- `visible_hints() -> Vec<HintLabel>`
- `activate_hint(&label)`
- `focus_next()` / `focus_prev()`

### 7.3 Semantic Zones (OSC 133)

```rust
enum ZoneType { Prompt, Command, Output }
let zones = harness.zones();
let output = harness.last_output_zone()?;
```

---

## 8. Architecture Compatibility Matrix

| Aspect | Scarab | Scryforge | Sigilforge | Compatible? |
|--------|--------|-----------|------------|-------------|
| Runtime | Tokio | Tokio | Tokio | Yes |
| Async | async/await | async/await | async/await | Yes |
| IPC | Unix Socket + SHM | JSON-RPC | JSON-RPC | Yes |
| GUI Framework | Bevy | Ratatui | N/A | Needs adapter |
| Plugin Model | Trait-based | Provider trait | N/A | Mappable |
| Navigation | PluginFocusable | NavRecorder | N/A | Compatible |

---

## 9. Risk Assessment

### 9.1 High Risk

1. **Ratatui→Bevy Rendering Bridge**
   - Scryforge uses Ratatui's `Buffer` for rendering
   - Scarab uses Bevy's GPU mesh pipeline
   - **Mitigation**: Create `TuiPluginBridge` adapter

2. **Fusabi Version Fragmentation**
   - Multiple incompatible versions across ecosystem
   - **Mitigation**: Coordinate upstream releases

### 9.2 Medium Risk

1. **OAuth Flow in Terminal**
   - Device code flow needs user interaction
   - **Mitigation**: Use existing Sigilforge CLI or integrate browser popup

2. **Status Bar Space Contention**
   - Multiple plugins competing for limited space
   - **Mitigation**: Priority system with overflow handling

### 9.3 Low Risk

1. **Navigation Protocol Stability**
   - Protobuf provides forward compatibility
   - Already deployed across multiple projects

2. **Testing Coverage**
   - ratatui-testlib provides comprehensive harnesses
   - Navigation testing already implemented

---

## 10. Recommendations

### Immediate Actions

1. **Version Align Fusabi Ecosystem** (Priority: Critical)
   - Update `bevy-fusabi` to Fusabi 0.21.0
   - Update `fusabi-tui` to Fusabi 0.21.0
   - Publish coordinated releases

2. **Create TuiPluginBridge** (Priority: High)
   - Adapter between Ratatui Buffer and Scarab's mesh renderer
   - Handle scrolling, selection, mouse events

3. **Sigilforge Security Hardening** (Priority: High)
   - Implement socket peer credentials
   - Add rate limiting

### Phase 1 Actions

4. **Minimal Scryforge Plugin** (Priority: High)
   - Basic stream list rendering
   - Item preview in terminal overlay
   - YouTube provider with Sigilforge auth

5. **Status Bar Tab Integration** (Priority: Medium)
   - Add plugin tab widget type
   - Define switching protocol

### Phase 2 Actions

6. **Full Navigation Integration** (Priority: Medium)
   - Register all Scryforge UI elements as focusables
   - Implement hint-based video selection

7. **Streaming Video Previews** (Priority: Low)
   - Sixel/Kitty graphics for thumbnails
   - Future: inline video playback

---

## Appendix A: File Reference

### Scarab
- Plugin API: `scarab-plugin-api/src/plugin.rs`
- Navigation: `scarab-plugin-api/src/navigation.rs`
- Status Bar: `scarab-plugin-api/src/status_bar/mod.rs`
- Protocol: `scarab-protocol/src/lib.rs`

### Scryforge
- Provider Core: `crates/scryforge-provider-core/src/lib.rs`
- YouTube: `providers/provider-youtube/src/lib.rs`
- TUI: `scryforge-tui/src/main.rs`
- Sigilforge Client: `scryforge-sigilforge-client/src/lib.rs`

### Sigilforge
- Token Manager: `sigilforge-core/src/token_manager.rs`
- OAuth: `sigilforge-core/src/oauth/pkce.rs`
- Client: `sigilforge-client/src/client.rs`

### Fusabi
- Frontend: `fusabi/rust/crates/fusabi-frontend/`
- VM: `fusabi/rust/crates/fusabi-vm/`
- Host: `fusabi-host/`
- Plugin Runtime: `fusabi-plugin-runtime/`
