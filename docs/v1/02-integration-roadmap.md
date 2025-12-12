# Integration Roadmap: Scryforge & Sigilforge as Scarab Plugins

**Version**: 1.0
**Date**: 2025-12-11

---

## Vision

Transform Scarab into a unified terminal workspace where users can:
1. Browse YouTube subscriptions, playlists, and videos
2. Navigate content using Vimium-style keyboard hints
3. Manage OAuth credentials seamlessly through Sigilforge
4. Switch between terminal and content views via status bar tabs

---

## Phase 0: Upstream Dependencies (BLOCKING)

### 0.1 Fusabi Version Alignment

**Problem**: Core ecosystem packages use incompatible Fusabi versions.

| Package | Current | Target |
|---------|---------|--------|
| bevy-fusabi | 0.17.0 | 0.21.0 |
| fusabi-tui | 0.16.0 | 0.21.0 |
| fusabi-host | 0.18-0.19 | 0.21.0 |

**Tasks**:
- [ ] Fork and update `bevy-fusabi` to Fusabi 0.21.0
- [ ] Update `fusabi-tui` to Fusabi 0.21.0
- [ ] Update `fusabi-host` compatibility documentation
- [ ] Publish coordinated releases to crates.io

**Owner**: fusabi-lang maintainer
**Estimate**: 3-5 days
**GitHub Issues**: See [03-upstream-work.md](./03-upstream-work.md)

### 0.2 Sigilforge Security Hardening

**Problem**: Socket lacks authentication; any process can request tokens.

**Tasks**:
- [ ] Implement `SO_PEERCRED` peer credential verification (Linux)
- [ ] Add process allowlist configuration
- [ ] Implement rate limiting (10 requests/second default)
- [ ] Add audit logging for token access

**Owner**: sigilforge maintainer
**Estimate**: 2-3 days

---

## Phase 1: Plugin Infrastructure

### 1.1 TuiPluginBridge Adapter

**Goal**: Bridge Ratatui's `Buffer` rendering to Scarab's GPU mesh pipeline.

**Architecture**:
```
Scryforge Widget → Ratatui Buffer → TuiPluginBridge → Scarab Mesh
```

**Components**:

```rust
// scarab-tui-bridge/src/lib.rs
pub struct TuiPluginBridge {
    buffer: Buffer,
    mesh_cache: HashMap<(u16, u16), MeshHandle>,
}

impl TuiPluginBridge {
    /// Render Ratatui buffer to Scarab mesh data
    pub fn render_to_mesh(&self, region: Rect) -> TerminalMeshData;

    /// Handle scroll events
    pub fn scroll(&mut self, delta: i16);

    /// Convert Scarab input to Ratatui events
    pub fn translate_input(&self, event: ScarabInput) -> Option<Event>;
}
```

**Tasks**:
- [ ] Create `scarab-tui-bridge` crate
- [ ] Implement Buffer → Mesh conversion
- [ ] Handle colors (ANSI 256 + RGB)
- [ ] Support styled text (bold, italic, underline)
- [ ] Implement scroll viewport
- [ ] Add mouse event translation

**Estimate**: 5-7 days

### 1.2 Scryforge Plugin Wrapper

**Goal**: Wrap Scryforge TUI as a Scarab plugin.

**Structure**:
```rust
// scarab-scryforge-plugin/src/lib.rs
pub struct ScryforgePlugin {
    bridge: TuiPluginBridge,
    daemon_client: DaemonClient,
    state: AppState,
}

#[async_trait]
impl Plugin for ScryforgePlugin {
    async fn on_load(&mut self, ctx: &mut PluginContext) -> Result<()>;
    async fn on_remote_command(&mut self, id: &str, ctx: &PluginContext) -> Result<()>;
    fn get_menu(&self) -> Vec<MenuItem>;
}
```

**Tasks**:
- [ ] Create `scarab-scryforge-plugin` crate
- [ ] Implement `Plugin` trait
- [ ] Wire daemon client connection
- [ ] Add menu items (Open, Sync, Settings)
- [ ] Register status bar items

**Estimate**: 3-4 days

### 1.3 Plugin Installation Testing

**Goal**: Verify plugin discovery, loading, and lifecycle.

**Test Cases**:
```rust
#[test]
fn test_plugin_discovery() {
    let registry = PluginRegistry::new();
    registry.scan_plugins("~/.config/scarab/plugins/")?;
    assert!(registry.contains("scryforge"));
}

#[test]
fn test_plugin_lifecycle() {
    let mut plugin = ScryforgePlugin::new();
    let mut ctx = MockPluginContext::new();
    plugin.on_load(&mut ctx).await?;
    assert!(ctx.registered_menu_items.len() > 0);
    plugin.on_unload().await?;
}
```

**Tasks**:
- [ ] Write plugin discovery tests
- [ ] Write lifecycle tests (load/unload)
- [ ] Write capability verification tests
- [ ] Add integration tests with ratatui-testlib

**Estimate**: 2-3 days

---

## Phase 2: Minimal Working Navigation

### 2.1 Basic Focusable Registration

**Goal**: Scryforge registers its UI elements as navigation targets.

**Implementation**:
```rust
// In ScryforgePlugin render loop
fn register_focusables(&self, ctx: &PluginContext) {
    let nav = ctx.navigation();

    // Register stream list items
    for (i, stream) in self.state.streams.iter().enumerate() {
        nav.register_focusable(PluginFocusable {
            x: 0, y: i as u16 + 1,
            width: 20, height: 1,
            label: stream.name.clone(),
            action: PluginFocusableAction::Custom(format!("select_stream:{}", i)),
        })?;
    }

    // Register item list entries
    for (i, item) in self.state.items.iter().enumerate() {
        nav.register_focusable(PluginFocusable {
            x: 21, y: i as u16 + 1,
            width: 40, height: 1,
            label: item.title.clone(),
            action: PluginFocusableAction::Custom(format!("select_item:{}", i)),
        })?;
    }
}
```

**Tasks**:
- [ ] Add focusable registration to ScryforgePlugin
- [ ] Handle Custom action dispatch
- [ ] Implement stream selection via hints
- [ ] Implement item selection via hints

**Estimate**: 2-3 days

### 2.2 Hint Mode Integration

**Goal**: `f` key activates hint mode showing labels on all interactive elements.

**Flow**:
1. User presses `f` in Scryforge view
2. Scarab enters hint mode via `NavEnterHintMode`
3. Labels rendered on all registered focusables
4. User types label (e.g., "ad")
5. Action executed (`select_item:5`)
6. Scarab exits hint mode

**Tasks**:
- [ ] Wire `f` keybinding to hint mode
- [ ] Implement hint label rendering (Vimium style)
- [ ] Add Custom action handler in plugin
- [ ] Test with 50+ focusables (label overflow)

**Estimate**: 2 days

---

## Phase 3: Full Scarab-Style Navigation

### 3.1 Multi-Region Navigation

**Goal**: Navigate between Scryforge panes using vim keys.

**Keybindings**:
| Key | Action |
|-----|--------|
| `h` | Focus left pane (streams) |
| `l` | Focus right pane (preview) |
| `j` | Move down in current list |
| `k` | Move up in current list |
| `Enter` | Activate selected item |
| `Tab` | Cycle panes |
| `f` | Enter hint mode |
| `Esc` | Exit hint mode / return to terminal |

**Implementation**:
```rust
fn handle_key(&mut self, key: KeyCode, ctx: &PluginContext) -> Result<Action> {
    match (self.focused_pane, key) {
        (Pane::Streams, KeyCode::Char('j')) => self.select_next_stream(),
        (Pane::Items, KeyCode::Char('j')) => self.select_next_item(),
        (_, KeyCode::Char('h')) => self.focus_pane(Pane::Streams),
        (_, KeyCode::Char('l')) => self.focus_pane(Pane::Preview),
        (_, KeyCode::Char('f')) => ctx.navigation().enter_hint_mode(),
        _ => Ok(Action::Continue),
    }
}
```

**Tasks**:
- [ ] Implement pane focus tracking
- [ ] Add vim-style navigation keys
- [ ] Highlight focused pane visually
- [ ] Add key binding documentation

**Estimate**: 2 days

### 3.2 URL Detection & Opening

**Goal**: Detect URLs in content and allow opening in browser.

**Implementation**:
```rust
fn detect_urls(text: &str) -> Vec<UrlMatch> {
    let url_regex = Regex::new(r"https?://[^\s]+").unwrap();
    url_regex.find_iter(text)
        .map(|m| UrlMatch { start: m.start(), end: m.end(), url: m.as_str().to_string() })
        .collect()
}

// Register URL focusables
for url in detect_urls(&preview_content) {
    nav.register_focusable(PluginFocusable {
        x: url.col, y: url.row,
        width: url.len, height: 1,
        label: url.url.clone(),
        action: PluginFocusableAction::OpenUrl(url.url),
    })?;
}
```

**Tasks**:
- [ ] Add URL regex detection
- [ ] Register URLs as focusables
- [ ] Implement OpenUrl action handler
- [ ] Test with various URL formats

**Estimate**: 1 day

---

## Phase 4: YouTube Integration

### 4.1 Sigilforge Authentication Flow

**Goal**: OAuth flow for YouTube via Sigilforge.

**Flow**:
1. User selects "Add YouTube Account" from menu
2. Scryforge calls `sigilforge.add_account("youtube", "default")`
3. Sigilforge initiates Device Code flow
4. User shown verification URL and code
5. User authorizes in browser
6. Token stored in keyring

**Implementation**:
```rust
// scarab-scryforge-plugin/src/auth.rs
async fn add_youtube_account(ctx: &PluginContext) -> Result<()> {
    let client = SigilforgeClient::new().await?;

    // Start device code flow
    let flow = client.start_device_flow("youtube", &["https://www.googleapis.com/auth/youtube.readonly"]).await?;

    // Show user the verification URL
    ctx.notify(&format!(
        "Visit {} and enter code: {}",
        flow.verification_url,
        flow.user_code
    ))?;

    // Poll for completion
    let token = client.poll_device_flow(&flow).await?;

    ctx.notify_success("YouTube account connected!")?;
    Ok(())
}
```

**Tasks**:
- [ ] Add Sigilforge client to plugin
- [ ] Implement device code flow UI
- [ ] Handle token refresh
- [ ] Add "Disconnect Account" option

**Estimate**: 2-3 days

### 4.2 YouTube Provider Integration

**Goal**: Fetch and display YouTube content.

**Data Flow**:
```
Sigilforge Token → YouTubeProvider → Scryforge Daemon → TUI Widget
```

**Streams Displayed**:
- Subscriptions (channels)
- Playlists
- Watch Later
- Liked Videos

**Item Actions**:
| Action | Hint Key | Behavior |
|--------|----------|----------|
| Open | `o` | Open in browser |
| Copy URL | `y` | Copy to clipboard |
| Save | `s` | Add to Watch Later |
| Play | `Enter` | Open video page |

**Tasks**:
- [ ] Wire YouTubeProvider to Scryforge daemon
- [ ] Implement subscription listing
- [ ] Implement playlist browsing
- [ ] Add video actions (open, copy, save)
- [ ] Display video metadata (duration, views, date)

**Estimate**: 3-4 days

### 4.3 Video Thumbnails (Optional)

**Goal**: Display video thumbnails using Sixel/Kitty graphics.

**Implementation**:
```rust
fn render_thumbnail(video: &Video, area: Rect, frame: &mut Frame) {
    if let Some(thumbnail_url) = &video.thumbnail_url {
        // Fetch and cache thumbnail
        let image = thumbnail_cache.get_or_fetch(thumbnail_url)?;

        // Render as Sixel if supported
        if frame.terminal_capabilities().supports_sixel() {
            frame.render_sixel(&image, area);
        } else {
            // Fallback: colored block characters
            frame.render_block_image(&image, area);
        }
    }
}
```

**Tasks**:
- [ ] Add thumbnail fetching with caching
- [ ] Implement Sixel rendering
- [ ] Add Kitty graphics protocol support
- [ ] Implement fallback block rendering

**Estimate**: 3-5 days (optional, can defer)

---

## Phase 5: Status Bar Integration

### 5.1 Plugin Tab Widget

**Goal**: Status bar shows tabs for switching between terminal and plugins.

**Layout**:
```
[Terminal] [YouTube] [RSS] [Email]  |  user@host  12:34
```

**Implementation**:
```rust
// scarab-plugin-api/src/status_bar/tabs.rs
pub struct PluginTab {
    pub plugin_id: String,
    pub icon: String,      // e.g., ""
    pub label: String,     // e.g., "YouTube"
    pub shortcut: String,  // e.g., "Alt+2"
    pub unread_count: Option<u32>,
}

pub fn render_tabs(tabs: &[PluginTab], active: usize) -> Vec<RenderItem> {
    tabs.iter().enumerate().flat_map(|(i, tab)| {
        let is_active = i == active;
        vec![
            RenderItem::Background(if is_active { Color::Named("accent") } else { Color::Named("bg") }),
            RenderItem::Text(format!(" {} {} ", tab.icon, tab.label)),
            if let Some(count) = tab.unread_count {
                RenderItem::Text(format!("({})", count))
            } else {
                RenderItem::Text(String::new())
            },
        ]
    }).collect()
}
```

**Tasks**:
- [ ] Add PluginTab struct to status_bar API
- [ ] Implement tab rendering in status bar
- [ ] Add Alt+N keyboard shortcuts
- [ ] Show active tab highlight
- [ ] Add unread count badges

**Estimate**: 2 days

### 5.2 Tab Switching Protocol

**Goal**: Keyboard shortcuts switch between terminal and plugin views.

**Keybindings**:
| Key | Action |
|-----|--------|
| `Alt+1` | Switch to terminal |
| `Alt+2` | Switch to first plugin (YouTube) |
| `Alt+3` | Switch to second plugin |
| `Ctrl+Tab` | Cycle tabs forward |
| `Ctrl+Shift+Tab` | Cycle tabs backward |

**Implementation**:
```rust
pub enum ViewMode {
    Terminal,
    Plugin(String),  // plugin_id
}

pub fn handle_tab_switch(key: KeyEvent, current: ViewMode) -> Option<ViewMode> {
    match key {
        KeyEvent { code: KeyCode::Char('1'), modifiers: ALT } => Some(ViewMode::Terminal),
        KeyEvent { code: KeyCode::Char('2'), modifiers: ALT } => Some(ViewMode::Plugin("scryforge".into())),
        KeyEvent { code: KeyCode::Tab, modifiers: CTRL } => Some(cycle_next(current)),
        _ => None,
    }
}
```

**Tasks**:
- [ ] Add ViewMode enum to client state
- [ ] Implement tab switch handlers
- [ ] Add visual transition between views
- [ ] Persist last active tab

**Estimate**: 2 days

---

## Phase 6: Testing & Polish

### 6.1 Integration Tests

**Goal**: Comprehensive test coverage using ratatui-testlib.

**Test Categories**:

1. **Plugin Lifecycle**
```rust
#[test]
async fn test_scryforge_plugin_loads() {
    let harness = ScarabTestHarness::connect()?;
    harness.send_command(":plugin load scryforge")?;
    harness.wait_for_text("Scryforge loaded")?;
}
```

2. **Navigation**
```rust
#[test]
async fn test_hint_mode_navigation() {
    let harness = ScarabTestHarness::connect()?;
    harness.send_key(KeyCode::Char('f'))?;  // Enter hint mode
    harness.wait_for_text("a")?;  // First hint label
    harness.send_text("a")?;  // Activate first hint
    harness.assert_text_at(1, 0, "[selected]")?;
}
```

3. **YouTube Integration**
```rust
#[test]
async fn test_youtube_subscription_list() {
    let harness = ScarabTestHarness::connect()?;
    harness.send_keys(&[ALT, KeyCode::Char('2')])?;  // Switch to YouTube
    harness.wait_for_text("Subscriptions")?;
    harness.assert_contains("@channelname")?;
}
```

**Tasks**:
- [ ] Write plugin lifecycle tests
- [ ] Write navigation tests
- [ ] Write YouTube integration tests
- [ ] Add performance benchmarks
- [ ] Set up CI pipeline

**Estimate**: 3-4 days

### 6.2 Documentation

**Goal**: User and developer documentation.

**User Docs**:
- Installation guide
- YouTube setup walkthrough
- Keyboard shortcuts reference
- Troubleshooting guide

**Developer Docs**:
- Plugin API reference
- TuiPluginBridge usage
- Navigation registration guide
- Contributing guide

**Tasks**:
- [ ] Write user installation guide
- [ ] Write YouTube setup guide
- [ ] Generate API documentation
- [ ] Add inline code comments
- [ ] Create demo GIF/video

**Estimate**: 2-3 days

---

## Timeline Summary

| Phase | Description | Estimate | Dependencies |
|-------|-------------|----------|--------------|
| 0 | Upstream Dependencies | 5-8 days | None (BLOCKING) |
| 1 | Plugin Infrastructure | 10-14 days | Phase 0 |
| 2 | Minimal Navigation | 4-6 days | Phase 1 |
| 3 | Full Navigation | 3-4 days | Phase 2 |
| 4 | YouTube Integration | 8-12 days | Phase 3 |
| 5 | Status Bar Tabs | 4 days | Phase 1 |
| 6 | Testing & Polish | 5-7 days | All phases |

**Total Estimate**: 39-55 days (8-11 weeks)

---

## Success Criteria

### MVP (End of Phase 3)
- [ ] Scryforge loads as Scarab plugin
- [ ] Navigate streams/items with `j`/`k`
- [ ] Hint mode shows labels on all items
- [ ] Select items via hint keys

### Full Release (End of Phase 6)
- [ ] YouTube OAuth via Sigilforge
- [ ] Browse subscriptions, playlists, watch later
- [ ] Open videos in browser via hint
- [ ] Status bar tabs for view switching
- [ ] 80%+ test coverage
- [ ] User documentation complete

---

## Risk Mitigation

| Risk | Impact | Mitigation |
|------|--------|------------|
| Fusabi version conflicts | High | Upstream PRs first |
| Ratatui→Bevy bridge complexity | Medium | Start with text-only, add styling incrementally |
| OAuth flow UX | Medium | Use device code (no browser popup needed) |
| Performance with many items | Low | Virtualized scrolling, lazy loading |
| Sixel support varies | Low | Block character fallback |
