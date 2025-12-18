# Tab and Pane Management System Design

**Version:** 0.1.0
**Status:** Initial Implementation
**Last Updated:** 2025-11-25

## Executive Summary

This document describes the design and implementation of Scarab Terminal's tab and pane management system. The system is split into two independent plugins that work together to provide a powerful workspace management experience.

## Architecture Decision

**Selected: Option B - Two Separate Plugins**

### Rationale

1. **Separation of Concerns**:
   - Tabs are primarily UI-focused (client-side)
   - Panes involve PTY management (daemon-side)
   - Clear boundaries enable independent evolution

2. **Modularity**:
   - Users can enable tabs without panes, or vice versa
   - Easier to test, debug, and maintain
   - Follows Unix philosophy: do one thing well

3. **Hot-Reload Support**:
   - Tab UI can be reloaded without affecting PTY sessions
   - Pane logic can be updated without UI disruption

4. **Performance**:
   - Daemon doesn't need to know about tab rendering details
   - Client doesn't need to manage PTY sessions
   - Clear IPC boundaries

5. **Consistency**:
   - Matches existing plugin pattern (scarab-nav, scarab-palette)
   - Leverages proven RemoteCommand architecture

### Components

```
┌─────────────────────────────────────────────────────────┐
│                     scarab-tabs                         │
│  ┌──────────────────────────────────────────────────┐   │
│  │  Tab State Management                            │   │
│  │  - Tab creation/deletion                         │   │
│  │  - Tab switching (next/prev/by-index)            │   │
│  │  - Tab reordering                                │   │
│  │  - Tab metadata (title, session, working dir)    │   │
│  └──────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────┘
                            ↕
                     RemoteCommand
                            ↕
┌─────────────────────────────────────────────────────────┐
│                     scarab-panes                        │
│  ┌──────────────────────────────────────────────────┐   │
│  │  Pane Layout Management                          │   │
│  │  - Pane splitting (horizontal/vertical)          │   │
│  │  - Pane navigation (up/down/left/right)          │   │
│  │  - Pane resizing                                 │   │
│  │  - PTY session per pane (future)                 │   │
│  └──────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────┘
```

## Data Structures

### scarab-tabs

```rust
pub struct Tab {
    pub id: u64,
    pub title: String,
    pub session_id: Option<String>,      // Session this tab belongs to
    pub working_dir: Option<String>,      // Current working directory
    pub active_pane_id: Option<u64>,     // Active pane within this tab
    pub created_at: u64,                 // Unix timestamp
    pub last_active: u64,                // Last time tab was active
}

struct PluginState {
    tabs: Vec<Tab>,                      // Ordered list of tabs
    active_tab_index: usize,             // Currently active tab
    next_tab_id: u64,                    // ID generator
}
```

### scarab-panes

```rust
pub struct Pane {
    pub layout: PaneLayout,
    pub session_id: Option<String>,
    pub working_dir: Option<String>,
    pub created_at: u64,
    // Future: PTY master/slave handles
}

pub struct PaneLayout {
    pub id: u64,
    pub parent_id: Option<u64>,          // Parent pane (for tree structure)
    pub split_direction: Option<SplitDirection>,
    pub x: u16,                          // Terminal coordinates
    pub y: u16,
    pub width: u16,
    pub height: u16,
    pub is_focused: bool,
}

pub enum SplitDirection {
    Horizontal,  // Split top/bottom
    Vertical,    // Split left/right
}

struct PluginState {
    panes: HashMap<u64, Pane>,           // Pane ID -> Pane
    active_pane_id: u64,                 // Currently focused pane
    next_pane_id: u64,                   // ID generator
    terminal_size: (u16, u16),           // (cols, rows)
}
```

## IPC Protocol Extension

### ControlMessage (Client → Daemon)

```rust
// Tab management
TabCreate { title: Option<String> },
TabClose { tab_id: u64 },
TabSwitch { tab_id: u64 },
TabRename { tab_id: u64, new_title: String },
TabList,

// Pane management
PaneSplit { pane_id: u64, direction: SplitDirection },
PaneClose { pane_id: u64 },
PaneFocus { pane_id: u64 },
PaneResize { pane_id: u64, width: u16, height: u16 },
```

### DaemonMessage (Daemon → Client)

```rust
// Tab state updates
TabCreated { tab: TabInfo },
TabClosed { tab_id: u64 },
TabSwitched { tab_id: u64 },
TabListResponse { tabs: Vec<TabInfo> },

// Pane state updates
PaneCreated { pane: PaneInfo },
PaneClosed { pane_id: u64 },
PaneFocused { pane_id: u64 },
PaneLayoutUpdate { panes: Vec<PaneInfo> },
```

## Keybinding Specification

### Tab Keybindings

| Key Combo | ASCII/Sequence | Action | Priority |
|-----------|----------------|--------|----------|
| `Ctrl+Shift+T` | `0x14` | New Tab | High |
| `Ctrl+Shift+W` | `0x17` | Close Tab/Pane | High |
| `Ctrl+Tab` | `0x09` | Next Tab | Medium |
| `Ctrl+Shift+Tab` | Special | Previous Tab | Medium |
| `Ctrl+1` | `0x01` | Switch to Tab 1 | Low |
| `Ctrl+2` | `0x02` | Switch to Tab 2 | Low |
| ... | ... | ... | ... |
| `Ctrl+9` | `0x09` | Switch to Tab 9 | Low |

### Pane Keybindings

| Key Combo | ASCII/Sequence | Action | Priority |
|-----------|----------------|--------|----------|
| `Ctrl+Shift+-` | `0x1F` | Split Horizontal | High |
| `Ctrl+Shift+\|` | `0x1C` | Split Vertical | High |
| `Ctrl+Shift+W` | `0x17` | Close Pane | High |
| `Ctrl+Shift+↑` | Special | Focus Pane Above | Medium |
| `Ctrl+Shift+↓` | Special | Focus Pane Below | Medium |
| `Ctrl+Shift+←` | Special | Focus Pane Left | Medium |
| `Ctrl+Shift+→` | Special | Focus Pane Right | Medium |

**Note:** Some keybindings require special handling in the input pipeline. The actual byte sequences may vary based on terminal emulator and platform.

## State Management Strategy

### Tab State

- **Location**: scarab-tabs plugin (daemon-side for now)
- **Persistence**: Future integration with scarab-session
- **Synchronization**: Tab state updates sent to client via DaemonMessage
- **Lifetime**: Tabs persist until explicitly closed

### Pane State

- **Location**: scarab-panes plugin (daemon-side)
- **Persistence**: Pane layouts are recreated on restart (future: saved layouts)
- **Synchronization**: Pane layout updates sent to client on every change
- **Lifetime**: Panes persist until explicitly closed

### Active State Tracking

- **Active Tab**: Tracked by `active_tab_index` in TabsPlugin
- **Active Pane**: Tracked by `active_pane_id` in PanesPlugin
- **Focus Indicator**: Client renders visual indicators for active tab/pane

## UI Rendering Strategy (Client-Side)

### Tab Bar Rendering

```rust
// Bevy UI system to render tab bar
fn render_tab_bar(
    config: Res<ScarabConfig>,
    tabs: Res<TabState>,
    mut commands: Commands,
) {
    let position = config.ui.tab_position;

    // Render tab bar at configured position
    match position {
        TabPosition::Top => render_tabs_horizontal(&tabs, &mut commands, true),
        TabPosition::Bottom => render_tabs_horizontal(&tabs, &mut commands, false),
        TabPosition::Left => render_tabs_vertical(&tabs, &mut commands, true),
        TabPosition::Right => render_tabs_vertical(&tabs, &mut commands, false),
    }
}
```

### Pane Border Rendering

```rust
// Render pane borders in Bevy
fn render_pane_borders(
    panes: Res<PaneState>,
    mut gizmos: Gizmos,
) {
    for pane in panes.iter() {
        if pane.layout.is_focused {
            // Render thick border for focused pane
            draw_border(&pane.layout, Color::BLUE, 2.0, &mut gizmos);
        } else {
            // Render thin border for inactive panes
            draw_border(&pane.layout, Color::GRAY, 1.0, &mut gizmos);
        }
    }
}
```

## Integration with Existing Systems

### scarab-session

```rust
// Future: Persist tab/pane state
impl SessionManager {
    fn save_workspace(&self, tabs: &[Tab], panes: &HashMap<u64, Pane>) -> Result<()> {
        // Serialize tab/pane state to database
        // Include PTY session IDs, working directories, etc.
    }

    fn restore_workspace(&self) -> Result<(Vec<Tab>, HashMap<u64, Pane>)> {
        // Restore tab/pane state from database
        // Reconnect to existing PTY sessions if available
    }
}
```

### scarab-config

Add new configuration sections:

```toml
[ui]
show_tabs = true
tab_position = "top"  # "top" | "bottom" | "left" | "right"

[tabs]
max_tabs = 20
default_title_template = "Terminal {n}"
close_last_tab_quits = false

[panes]
default_split = "vertical"
border_style = "rounded"  # "rounded" | "square" | "none"
border_color = "#4A90E2"
min_pane_size = 10  # minimum cols/rows
```

## Performance Considerations

### Memory Overhead

- **Tab State**: ~200 bytes per tab (metadata only)
- **Pane State**: ~150 bytes per pane + PTY handles
- **Total for 10 tabs × 4 panes**: ~8 KB

### IPC Message Frequency

- **Tab Operations**: Low frequency (user-initiated)
- **Pane Layout Updates**: Medium frequency (resize events)
- **Optimization**: Batch layout updates during resize

### Rendering Performance

- **Tab Bar**: Static UI, only re-render on tab changes
- **Pane Borders**: Render using GPU-accelerated Bevy gizmos
- **Target**: 60 FPS with 20 tabs and 10 panes

## Testing Strategy

### Unit Tests

- ✅ Tab creation/deletion
- ✅ Tab switching (next/prev/by-index)
- ✅ Tab reordering
- ✅ Pane splitting (horizontal/vertical)
- ✅ Pane navigation
- ✅ Pane closing
- ✅ Layout recalculation

### Integration Tests

- ⏳ Tab-pane interaction (create tab with multiple panes)
- ⏳ IPC message handling
- ⏳ Session persistence
- ⏳ UI rendering

### E2E Tests

- ⏳ Full workflow: create tabs, split panes, switch focus, close
- ⏳ Keybinding handling
- ⏳ Edge cases (last tab, last pane, etc.)

## Roadmap

### Phase 1: Core Implementation ✅

- [x] scarab-tabs plugin scaffolding
- [x] scarab-panes plugin scaffolding
- [x] Protocol extensions (ControlMessage, DaemonMessage)
- [x] Unit tests
- [x] Basic keybindings

### Phase 2: Integration (Next)

- [ ] Register plugins in daemon
- [ ] IPC message handlers in daemon/client
- [ ] Tab bar UI rendering (Bevy)
- [ ] Pane border rendering (Bevy)
- [ ] Configuration integration

### Phase 3: PTY Management

- [ ] PTY session per pane
- [ ] Input routing to active pane
- [ ] Output from each pane to shared memory
- [ ] Scrollback per pane

### Phase 4: Advanced Features

- [ ] Tab/pane persistence (scarab-session)
- [ ] Drag-and-drop tab reordering
- [ ] Pane resizing with mouse
- [ ] Pane zooming (maximize/restore)
- [ ] Saved layouts/workspaces
- [ ] Tab titles with templates

### Phase 5: Polish

- [ ] Animations (tab switching, pane creation)
- [ ] Themes (tab bar colors, pane borders)
- [ ] Accessibility (keyboard-only navigation)
- [ ] Performance optimization
- [ ] Documentation and examples

## Open Questions

1. **Tab-Pane Relationship**: Should tabs own panes, or are they independent?
   - **Decision**: Tabs contain panes. Each tab has its own pane layout.

2. **PTY Ownership**: Where should PTY handles live?
   - **Decision**: PanesPlugin owns PTY handles (daemon-side).

3. **Layout Algorithm**: How to handle complex split layouts?
   - **Decision**: Start with simple grid layout, evolve to tree-based.

4. **Input Routing**: How to route input to the correct pane?
   - **Decision**: Active pane receives input. Daemon routes based on focus.

5. **Shared Memory**: How to display multiple panes?
   - **Decision**: Future: Multiple shared memory regions per pane, or offset pointers.

## Related Work

- **tmux**: Session/window/pane hierarchy
- **Zellij**: Rust-based terminal multiplexer
- **Wezterm**: Tab and split pane support
- **Alacritty**: Single pane, no tabs (delegates to tmux/zellij)

## References

- [Scarab Plugin API](../crates/scarab-plugin-api/README.md)
- [Scarab Protocol](../crates/scarab-protocol/src/lib.rs)
- [Scarab Session Management](../crates/scarab-session/README.md)
- [Bevy UI Guide](https://bevyengine.org/learn/book/ui/)

## Changelog

- **2025-11-25**: Initial design document
  - Two-plugin architecture (scarab-tabs + scarab-panes)
  - Protocol extensions
  - Core implementation complete
  - Unit tests passing
