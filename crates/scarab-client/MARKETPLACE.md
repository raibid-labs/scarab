# Plugin Marketplace UI Implementation

This document describes the plugin marketplace UI implementation for Scarab terminal emulator.

## Overview

The marketplace provides a full-featured UI for browsing, searching, and installing plugins from the Scarab registry. It's built using the Ratatui bridge and integrates with the existing `scarab-config` registry infrastructure.

## Architecture

### Module Structure

```
crates/scarab-client/src/marketplace/
├── mod.rs              # Plugin definition, events, and mock data
├── overlay.rs          # Main UI rendering and input handling
├── plugin_card.rs      # Plugin display formatting
├── search.rs           # Search and filtering logic
└── installer.rs        # Installation progress tracking
```

### Components

**MarketplaceOverlay** (marker component)
- Identifies the marketplace surface entity

**MarketplaceState** (resource)
```rust
pub struct MarketplaceState {
    pub visible: bool,
    pub view: MarketplaceView,
    pub search: SearchState,
    pub selected_index: usize,
    pub scroll_offset: usize,
    pub filtered_plugins: Vec<PluginEntry>,
    pub category: String,
    pub categories: Vec<String>,
}
```

**PluginListCache** (resource)
```rust
pub struct PluginListCache {
    pub plugins: Vec<PluginEntry>,
    pub last_fetch: Option<u64>,
    pub fetching: bool,
    pub error: Option<String>,
}
```

**InstallProgress** (resource)
```rust
pub struct InstallProgress {
    pub plugin_name: String,
    pub status: InstallStatus,
    pub progress: u8,
    pub message: String,
    pub start_time: Option<u64>,
    pub end_time: Option<u64>,
}
```

### Events

**MarketplaceEvent**
- `Open` - Show marketplace
- `Close` - Hide marketplace
- `Toggle` - Toggle visibility
- `Refresh` - Fetch latest plugins from registry
- `SelectPlugin(usize)` - Select plugin by index
- `ViewDetails(String)` - View plugin details
- `SearchUpdated(String)` - Search query changed
- `CategoryChanged(String)` - Filter category changed

**InstallPluginEvent**
```rust
pub struct InstallPluginEvent {
    pub name: String,
    pub version: Option<String>,
    pub is_update: bool,
}
```

## Features

### 1. Plugin List View

Displays available plugins with:
- Plugin name and version
- Star rating (visual stars + numeric)
- Description
- Author, license, download count
- Tags/categories

**Layout:**
```
┌─────────────────────────────────────────┐
│ Scarab Plugin Marketplace               │
│ Browse, search, and install plugins     │
├─────────────────────────────────────────┤
│ [All] Prompt Editor VCS DevOps Network  │ <- Categories
├─────────────────────────────────────────┤
│ 🔍 Type to search plugins...            │ <- Search bar
├─────────────────────────────────────────┤
│ ★ git-status  v1.2.0 ★★★★★ (4.8)       │
│   Display git branch and status         │
│   by scarab-team | MIT | 15.2K dl       │
│   [git] [prompt] [vcs]                  │
│                                          │
│ ★ syntax-highlight  v2.0.1 ★★★★⯨ (4.6) │
│   Syntax highlighting for file types    │
│   by syntax-team | Apache-2.0 | 28.5K   │
│   [syntax] [highlighting] [editor]      │
│                                          │
│ ... (more plugins)                      │
├─────────────────────────────────────────┤
│ ↑↓ Navigate  Tab Categories  / Search  │
│ Enter Install  d Details  r Refresh     │
│ q Close                                 │
└─────────────────────────────────────────┘
```

### 2. Plugin Details View

Shows comprehensive information:
- Full plugin description
- Author and maintainer info
- License details
- Statistics (downloads, ratings, reviews)
- Repository links
- All available tags

### 3. Installation Progress

Real-time progress indicators:
- Download progress
- Verification status
- Installation status
- Error messages if failed
- Success confirmation

**Status Flow:**
```
Idle → Downloading → Verifying → Installing → Complete/Failed
```

### 4. Search and Filtering

**Search:**
- Searches across: plugin name, description, author, tags
- Case-insensitive
- Real-time filtering as you type

**Category Filters:**
- All
- Prompt (prompt-related plugins)
- Editor (syntax, highlighting)
- VCS (git, version control)
- DevOps (kubernetes, docker)
- Network (ssh, http)

**Sort Orders:**
- Popular (most downloads)
- Rating (highest rated)
- Recent (recently updated)
- Name (alphabetical)

## Keybindings

### Global
- `Ctrl+Shift+M` - Toggle marketplace

### Plugin List View
- `↑` / `k` - Previous plugin
- `↓` / `j` - Next plugin
- `Tab` - Next category
- `Shift+Tab` - Previous category
- `/` or `Ctrl+F` - Focus search
- `Enter` - Install selected plugin
- `d` - View plugin details
- `r` - Refresh plugin list
- `q` or `Escape` - Close marketplace

### Search Mode
- Type to search
- `Backspace` - Delete character
- `Enter` or `Escape` - Exit search mode

### Plugin Details View
- `Enter` or `i` - Install plugin
- `Escape` or `q` or `Backspace` - Back to list

### Installation View
- `Enter` or `Escape` - Return to list (when done)

## Integration

### Adding to Your App

```rust
use scarab_client::marketplace::MarketplacePlugin;

App::new()
    .add_plugins(MarketplacePlugin)
    // ... other plugins
```

### Triggering Marketplace

```rust
// Via events
fn some_system(mut events: EventWriter<MarketplaceEvent>) {
    events.send(MarketplaceEvent::Open);
}

// Via keybinding
fn handle_keys(
    keys: Res<ButtonInput<KeyCode>>,
    mut events: EventWriter<MarketplaceEvent>,
) {
    if keys.pressed(KeyCode::ControlLeft)
        && keys.pressed(KeyCode::ShiftLeft)
        && keys.just_pressed(KeyCode::KeyM)
    {
        events.send(MarketplaceEvent::Toggle);
    }
}
```

### Installing Plugins

```rust
fn install_plugin(mut events: EventWriter<InstallPluginEvent>) {
    events.send(InstallPluginEvent {
        name: "git-status".to_string(),
        version: None,  // Use latest
        is_update: false,
    });
}
```

## Testing

### Unit Tests

Each module has comprehensive unit tests:

```bash
# Run all marketplace tests
cargo test -p scarab-client marketplace

# Run specific test
cargo test -p scarab-client test_marketplace_navigation
```

### Integration Tests

Located in `tests/marketplace_integration_tests.rs`:

```bash
cargo test -p scarab-client marketplace_integration
```

### Demo Example

Run the standalone demo:

```bash
cargo run -p scarab-client --example marketplace_demo
```

## Implementation Notes

### Mock Data

The current implementation includes mock plugin data for testing. In production, this would be replaced with actual registry fetching:

```rust
// TODO: Replace with actual async registry fetching
// let manager = RegistryManager::new().unwrap();
// manager.sync().await.unwrap();
// let plugins = manager.search(&filter).unwrap();
```

### Async Integration

The plugin fetching is currently synchronous with mock data. Real implementation would need:

1. Async task spawning via `bevy_tasks` or similar
2. Channel-based communication between fetch task and UI
3. Progress indication during network operations
4. Error handling for network failures

### Performance Considerations

- Plugin list is cached in `PluginListCache` resource
- Filtering is applied on-demand, not per-frame
- Only visible plugins are rendered (with scroll offset)
- Dirty tracking prevents unnecessary redraws

### Future Enhancements

1. **Plugin versioning**: Show all available versions, not just latest
2. **Dependency management**: Display and handle plugin dependencies
3. **Update notifications**: Highlight plugins with available updates
4. **Screenshots**: Display plugin screenshots if available
5. **User reviews**: Show user reviews and ratings
6. **Installed indicator**: Mark already-installed plugins
7. **Bulk operations**: Install/update multiple plugins at once
8. **Registry selection**: Support multiple plugin registries

## Files Created

### Core Implementation
- `/home/beengud/raibid-labs/scarab/crates/scarab-client/src/marketplace/mod.rs` (332 lines)
- `/home/beengud/raibid-labs/scarab/crates/scarab-client/src/marketplace/overlay.rs` (579 lines)
- `/home/beengud/raibid-labs/scarab/crates/scarab-client/src/marketplace/plugin_card.rs` (217 lines)
- `/home/beengud/raibid-labs/scarab/crates/scarab-client/src/marketplace/search.rs` (450 lines)
- `/home/beengud/raibid-labs/scarab/crates/scarab-client/src/marketplace/installer.rs` (274 lines)

### Tests and Examples
- `/home/beengud/raibid-labs/scarab/crates/scarab-client/tests/marketplace_integration_tests.rs` (395 lines)
- `/home/beengud/raibid-labs/scarab/crates/scarab-client/examples/marketplace_demo.rs` (41 lines)

### Total: ~2,288 lines of code

## API Reference

### Public Functions

**overlay.rs:**
- `open_marketplace()` - Show marketplace
- `close_marketplace()` - Hide marketplace
- `toggle_marketplace()` - Toggle visibility
- `handle_marketplace_input()` - Process input events
- `render_marketplace()` - Render marketplace UI

**plugin_card.rs:**
- `format_plugin_card(plugin, style)` - Format plugin as card
- `format_plugin_card_compact(plugin, style)` - Format as single line

**search.rs:**
- `format_search_bar(state)` - Format search input
- `apply_filters(plugins, search, category)` - Filter plugin list
- `update_search()` - Handle search input

**installer.rs:**
- `update_install_progress()` - Simulate installation progress
- `render_install_progress()` - Render progress UI

### Public Types

See "Components" section above for full type definitions.

## Dependencies

- `bevy` 0.15 - ECS and app framework
- `ratatui` - TUI widgets and rendering
- `scarab-config` - Registry and plugin types
- `scarab-protocol` - Terminal metrics

## License

Same as Scarab project (MIT/Apache-2.0 dual-licensed)
