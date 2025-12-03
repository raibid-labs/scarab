# Scarab Navigation System

## Overview

Scarab features an **ECS-native navigation system** built directly on Bevy's Entity Component System architecture. The system provides Vimium-style keyboard-first navigation for terminal emulator workflows, enabling rapid selection and interaction with URLs, file paths, emails, prompt markers, and UI widgets without using the mouse.

The navigation system is designed to be:
- **ECS-native**: Built with components, resources, and events throughout
- **Composable**: Different systems can add focusable elements independently
- **Extensible**: Easy to add new navigation actions and modes
- **Type-safe**: Strong typing for navigation targets and actions
- **High-performance**: Focusables are scanned on-demand, not every frame

## Navigation Modes

The system operates in four primary modes, managed via a mode stack that allows nested modes:

### Normal Mode
**Default terminal mode** - Standard input handling where keyboard input is passed to the PTY. This is the base mode from which other modes are entered.

**Available Actions:**
- Enter hint mode (Ctrl+F or Alt+F)
- Enter copy mode (Ctrl+V)
- Enter command palette (Ctrl+P)
- Navigate between prompts (Ctrl+Up/Down)
- Toggle link hints (Ctrl+K - legacy)

### Hints Mode
**Vimium-style link hints mode** - Displays labeled hints (aa, ab, ac...) over all focusable elements in the terminal. Users type hint labels to select and activate targets.

**Behavior:**
- Scans terminal content for URLs, file paths, emails, and prompt markers
- Generates hint labels using a-z characters (aa, ab, ..., ba, bb, ...)
- Filters hints as user types characters
- Activates target when full hint label is typed
- Supports prompt zone filtering to show only hints in current command output

**Available Actions:**
- Type hint characters (a-z) to filter and select
- Escape to cancel and return to previous mode
- Enter activates currently selected hint

### Insert Mode
**Text input mode** - All keyboard input is passed directly to the terminal/PTY without navigation interception. Used for normal terminal interaction.

### CommandPalette Mode
**Command palette with fuzzy search** - Provides quick access to terminal commands and actions via fuzzy searching.

**Available Actions:**
- Type to filter commands
- Navigate filtered results
- Enter to execute selected command
- Escape to cancel

## Keymaps

Scarab supports three keymap styles, selectable via the `NavInputRouter` resource:

### VimiumStyle (Default)

The default keymap inspired by Vimium browser extension:

| Key Combination | Mode | Action | Description |
|----------------|------|--------|-------------|
| **Ctrl+F** | Normal | EnterHintMode | Enter hint mode for link selection |
| **Alt+F** | Normal | EnterHintMode | Alternate binding for hint mode |
| **F** | Normal | EnterHintMode | Quick hint mode entry |
| **Escape** | Any | CancelAllModes | Exit all modes, return to Normal |
| **Ctrl+Up** | Normal | JumpToPrevPrompt | Jump to previous shell prompt |
| **Ctrl+Down** | Normal | JumpToNextPrompt | Jump to next shell prompt |
| **Ctrl+V** | Normal | EnterCopyMode | Enter copy/visual selection mode |
| **Ctrl+P** | Normal | EnterCommandPalette | Open command palette |
| **Ctrl+K** | Normal | Toggle Link Hints | Legacy link hints toggle |
| **Ctrl+/** | Normal | EnterSearchMode | Enter search mode |
| **a-z** | Hints | HintChar | Type hint label to select target |
| **Escape** | Copy | CopyModeExit | Exit copy mode |
| **N** | Search | NextSearchMatch | Next search match |
| **Shift+N** | Search | PrevSearchMatch | Previous search match |

### CosmosStyle

Space-based leader key approach (partially implemented):

| Key Combination | Mode | Action | Description |
|----------------|------|--------|-------------|
| **F** | Normal | EnterHintMode | Enter hint mode |
| **Escape** | Any | CancelAllModes | Cancel all modes |
| **Ctrl+Up** | Normal | JumpToPrevPrompt | Jump to previous prompt |
| **Ctrl+Down** | Normal | JumpToNextPrompt | Jump to next prompt |

*Note: Full leader key pattern for Space is planned but not yet implemented.*

### SpacemacsStyle

SPC prefix pattern (partially implemented):

| Key Combination | Mode | Action | Description |
|----------------|------|--------|-------------|
| **F** | Normal | EnterHintMode | Enter hint mode |
| **Escape** | Any | CancelAllModes | Cancel all modes |
| **Ctrl+Up** | Normal | JumpToPrevPrompt | Jump to previous prompt |
| **Ctrl+Down** | Normal | JumpToNextPrompt | Jump to next prompt |

*Note: Full SPC prefix pattern is planned but not yet implemented.*

### Switching Keymap Styles

```rust
// Change the active navigation style
nav_router.set_style(NavStyle::CosmosStyle);
```

## Focusable Types

The system automatically detects and tracks the following focusable element types:

### URL
HTTP/HTTPS URLs and www.* links detected via regex pattern:
```
https?://[^\s<>{}|\^~\[\]`]+|www\.[^\s<>{}|\^~\[\]`]+
```

**Examples:**
- `https://example.com`
- `www.github.com`
- `http://localhost:3000`

**Action:** Opens URL in default browser using platform-specific commands (xdg-open, open, cmd)

### FilePath
Absolute and relative file system paths detected via regex pattern:
```
(?:~|\.{1,2}|/)?(?:[a-zA-Z0-9_\-./]+/)*[a-zA-Z0-9_\-.]+\.[a-zA-Z]{2,5}
```

**Examples:**
- `/usr/local/bin/foo.txt`
- `./relative/path.rs`
- `~/Documents/file.md`

**Action:** Opens file in $EDITOR or default application, expands ~ to HOME directory

### Email
Email addresses detected via regex pattern:
```
[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}
```

**Examples:**
- `support@example.com`
- `user.name@domain.co.uk`

**Action:** Opens email client with mailto: link

### PromptMarker
Shell prompt markers from OSC 133 escape sequences. These anchors enable:
- Jump-to-prompt navigation (Ctrl+Up/Down)
- Semantic zone filtering (show only hints in current command output)
- Command output region selection

**Marker Types:**
- **PromptStart** (OSC 133 A): Blue gutter indicator - start of new prompt
- **CommandFinished** (OSC 133 D): Green (success) or Red (failure) gutter indicator
- **CommandOutput**: Region between prompt end and command finished

### Widget
UI widgets and interactive elements from Ratatui overlays (future feature)

## Focusable Sources

Each focusable region tracks its origin for debugging and filtering:

- **Terminal**: Detected by scanning terminal text content with regex
- **Ratatui**: Detected from Ratatui UI overlay (future)
- **PromptMarker**: Derived from OSC 133 prompt marker system

## Architecture

### Component Relationships

```
┌─────────────────────────────────────────────────────────────────┐
│                        Navigation System                        │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  ┌──────────────┐         ┌──────────────┐                    │
│  │  NavState    │         │ ModeStack    │                    │
│  │  Resource    │◄────────┤  Resource    │                    │
│  │              │         │              │                    │
│  │ - mode       │         │ - modes[]    │                    │
│  │ - history    │         │              │                    │
│  │ - filter     │         │              │                    │
│  └──────┬───────┘         └──────────────┘                    │
│         │                                                       │
│         │                                                       │
│  ┌──────▼────────────────────────────────────────┐            │
│  │          NavInputRouter Resource              │            │
│  │                                                │            │
│  │  - current_style: NavStyle                    │            │
│  │  - bindings_by_style: HashMap<NavStyle, []>   │            │
│  │                                                │            │
│  │  Keymaps:                                     │            │
│  │  ├─ VimiumStyle                              │            │
│  │  ├─ CosmosStyle                              │            │
│  │  └─ SpacemacsStyle                           │            │
│  └───────────────────────────────────────────────┘            │
│                                                                 │
│                                                                 │
│  ┌────────────────────────────────────────────────────────┐   │
│  │            Focusable Detection Pipeline                │   │
│  ├────────────────────────────────────────────────────────┤   │
│  │                                                          │   │
│  │  1. EnterHintModeEvent                                 │   │
│  │       ▼                                                  │   │
│  │  2. scan_terminal_focusables                           │   │
│  │     - Read SharedMemoryReader                          │   │
│  │     - Run regex detection (URLs/paths/emails)          │   │
│  │     - Query NavAnchor entities (prompt markers)        │   │
│  │       ▼                                                  │   │
│  │  3. Spawn FocusableRegion entities                     │   │
│  │       ▼                                                  │   │
│  │  4. bounds_to_world_coords                             │   │
│  │     - Convert grid coords → screen coords              │   │
│  │       ▼                                                  │   │
│  │  5. filter_focusables_by_zone                          │   │
│  │     - Apply prompt zone filtering (optional)           │   │
│  │       ▼                                                  │   │
│  │  6. Generate NavHint entities with labels              │   │
│  │       ▼                                                  │   │
│  │  7. Render hint labels on screen                       │   │
│  │                                                          │   │
│  └──────────────────────────────────────────────────────────┘   │
│                                                                 │
│                                                                 │
│  ┌─────────────────────────────────────────────────┐          │
│  │              Core Components                    │          │
│  ├─────────────────────────────────────────────────┤          │
│  │                                                   │          │
│  │  NavFocus                - Current focus marker  │          │
│  │  NavHint                 - Hint label & action   │          │
│  │  NavGroup                - Logical grouping      │          │
│  │  FocusableRegion         - Detected target       │          │
│  │  NavAnchor               - Prompt marker anchor  │          │
│  │  PromptGutterMarker      - Visual gutter marker  │          │
│  │                                                   │          │
│  └─────────────────────────────────────────────────┘          │
│                                                                 │
│                                                                 │
│  ┌─────────────────────────────────────────────────┐          │
│  │                 Events                          │          │
│  ├─────────────────────────────────────────────────┤          │
│  │                                                   │          │
│  │  EnterHintModeEvent      - Enter hints mode      │          │
│  │  ExitHintModeEvent       - Exit hints mode       │          │
│  │  NavActionEvent          - Trigger action        │          │
│  │  FocusChangedEvent       - Focus changed         │          │
│  │  JumpToPromptEvent       - Jump to prompt        │          │
│  │  PromptZoneFocusedEvent  - Prompt zone focus     │          │
│  │  LinkActivatedEvent      - Link activated        │          │
│  │                                                   │          │
│  └─────────────────────────────────────────────────┘          │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

### System Sets and Ordering

The navigation system runs in three ordered phases:

```
NavSystemSet::Input     → Process keyboard/mouse input
    ↓
NavSystemSet::Update    → Update navigation state and components
    ↓
NavSystemSet::Render    → Render hint labels and focus indicators
```

### Data Flow

```
User Input (Keyboard)
    ↓
route_nav_input system
    ↓
NavAction events → ModeStack mutations
    ↓
┌─────────────────────────────────────────┐
│  EnterHintModeEvent triggered           │
└─────────────────────────────────────────┘
    ↓
┌─────────────────────────────────────────┐
│  scan_terminal_focusables               │
│  - Extract terminal text                │
│  - Run regex detection                  │
│  - Query NavAnchor entities             │
└─────────────────────────────────────────┘
    ↓
FocusableRegion entities spawned
    ↓
bounds_to_world_coords (grid → screen)
    ↓
filter_focusables_by_zone (optional)
    ↓
NavHint entities generated with labels
    ↓
Hint labels rendered on screen
    ↓
User types hint characters
    ↓
NavAction::HintChar events
    ↓
Match found → NavAction::Open/Click/etc.
    ↓
Action executed (open URL, file, etc.)
```

## Developer Guide

### Registering Custom Focusable Regions

You can add custom focusable elements to the navigation system by spawning entities with the appropriate components:

```rust
use bevy::prelude::*;
use scarab_client::navigation::*;

fn spawn_custom_focusable(mut commands: Commands) {
    commands.spawn((
        FocusableRegion {
            region_type: FocusableType::Widget,
            grid_start: (10, 5),
            grid_end: (30, 5),
            content: "Custom Widget".to_string(),
            source: FocusableSource::Ratatui,
            screen_position: None, // Will be calculated automatically
        },
        NavHint {
            label: "aa".to_string(),
            position: Vec2::new(100.0, 200.0),
            action: NavAction::Click(10, 5),
        },
        NavGroup("custom_widgets".to_string()),
    ));
}
```

### Adding Custom Navigation Actions

Extend the `NavAction` enum to add new action types:

```rust
// In navigation/mod.rs
#[derive(Debug, Clone, PartialEq)]
pub enum NavAction {
    // ... existing actions ...

    /// Custom action: Copy text to clipboard
    CopyToClipboard(String),

    /// Custom action: Run shell command
    RunCommand(String),
}
```

Then implement a system to handle your custom actions:

```rust
fn handle_custom_actions(
    mut action_events: EventReader<NavActionEvent>,
) {
    for event in action_events.read() {
        match &event.action {
            NavAction::CopyToClipboard(text) => {
                // Implement clipboard copy logic
                info!("Copying to clipboard: {}", text);
            }
            NavAction::RunCommand(cmd) => {
                // Implement command execution logic
                info!("Running command: {}", cmd);
            }
            _ => {}
        }
    }
}
```

Add your system to the app:

```rust
app.add_systems(Update, handle_custom_actions.in_set(NavSystemSet::Update));
```

### Extending Keymaps

Add custom keybindings to an existing style or create a new style:

```rust
fn register_custom_bindings(mut router: ResMut<NavInputRouter>) {
    // Get current bindings
    let mut bindings = router.current_bindings().to_vec();

    // Add custom binding
    bindings.push(
        KeyBinding::new(KeyCode::KeyG, NavAction::CopyToClipboard("example".into()))
            .with_ctrl()
            .in_mode(NavMode::Normal)
    );

    // Register updated bindings
    router.bindings_by_style.insert(NavStyle::VimiumStyle, bindings);
}
```

### Creating Custom Focusable Scanners

Implement custom detection logic by creating a system that spawns `FocusableRegion` entities:

```rust
fn scan_custom_patterns(
    mut commands: Commands,
    mut enter_hint_events: EventReader<EnterHintModeEvent>,
    state_reader: Res<SharedMemoryReader>,
) {
    if enter_hint_events.is_empty() {
        return;
    }
    enter_hint_events.clear();

    let safe_state = state_reader.get_safe_state();
    let terminal_text = extract_grid_text(&safe_state);

    // Custom detection logic
    for (row, line) in terminal_text.lines().enumerate() {
        if line.contains("TODO:") {
            commands.spawn(FocusableRegion {
                region_type: FocusableType::Widget,
                grid_start: (0, row as u16),
                grid_end: (line.len() as u16, row as u16),
                content: line.to_string(),
                source: FocusableSource::Terminal,
                screen_position: None,
            });
        }
    }
}

// Register the system
app.add_systems(Update, scan_custom_patterns.in_set(NavSystemSet::Input));
```

### Customizing Focusable Detection Patterns

Modify the regex patterns used for detection:

```rust
fn configure_focusable_patterns(mut config: ResMut<FocusableScanConfig>) {
    // Add custom URL pattern for local development
    config.url_regex = r"https?://[^\s<>{}|\^~\[\]`]+|www\.[^\s<>{}|\^~\[\]`]+|localhost:\d+".to_string();

    // Adjust max focusables limit
    config.max_focusables = 1000;

    // Enable per-frame scanning (not recommended for performance)
    config.scan_on_frame = false; // Keep false for best performance
}
```

### Listening for Navigation Events

React to navigation events in your custom systems:

```rust
fn handle_focus_changes(
    mut focus_events: EventReader<FocusChangedEvent>,
) {
    for event in focus_events.read() {
        info!("Focus changed from {:?} to {:?}", event.old_focus, event.new_focus);
        // Custom logic: scroll into view, highlight, etc.
    }
}

fn handle_jump_to_prompt(
    mut jump_events: EventReader<JumpToPromptEvent>,
    mut scroll_state: ResMut<ScrollbackState>, // Your scroll state resource
) {
    for event in jump_events.read() {
        info!("Jumping to prompt at line {}", event.target_line);
        scroll_state.scroll_to_line(event.target_line);
    }
}
```

### Integrating with Prompt Zones

Use `PromptZoneFocusedEvent` to filter focusables to the current command output:

```rust
fn apply_prompt_zone_filter(
    mut zone_events: EventReader<PromptZoneFocusedEvent>,
    mut focusables: Query<&mut Visibility, With<FocusableRegion>>,
    focusable_regions: Query<&FocusableRegion>,
) {
    for event in zone_events.read() {
        info!("Filtering to zone: {}-{}", event.start_line, event.end_line);

        for (region, mut visibility) in focusables.iter_mut().zip(focusable_regions.iter()) {
            let in_zone = region.grid_start.1 as u32 >= event.start_line
                       && region.grid_start.1 as u32 < event.end_line;

            *visibility = if in_zone {
                Visibility::Visible
            } else {
                Visibility::Hidden
            };
        }
    }
}
```

## Performance Considerations

- **On-demand scanning**: Focusables are only scanned when entering hint mode, not every frame
- **Max focusables limit**: Default 500, prevents performance issues with large terminal buffers
- **Zone filtering**: Reduces hint clutter by showing only relevant items in current prompt output
- **Compiled regex caching**: Regex patterns are compiled once at startup
- **Lock-free coordination**: Uses Bevy's event system instead of locks for inter-system communication

## Configuration

### Default Configuration

```rust
FocusableScanConfig {
    url_regex: r"https?://[^\s<>{}|\^~\[\]`]+|www\.[^\s<>{}|\^~\[\]`]+",
    filepath_regex: r"(?:~|\.{1,2}|/)?(?:[a-zA-Z0-9_\-./]+/)*[a-zA-Z0-9_\-.]+\.[a-zA-Z]{2,5}",
    email_regex: r"[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}",
    scan_on_frame: false,
    max_focusables: 500,
}
```

### NavState Configuration

```rust
NavState {
    current_mode: NavMode::Normal,
    mode_stack: Vec::new(),
    focus_history: Vec::new(),
    hint_filter: String::new(),
    max_history_size: 50,
}
```

## Prompt Marker Integration

The navigation system integrates with OSC 133 shell integration for semantic navigation:

### OSC 133 Sequences

- **OSC 133 A**: Prompt start - Blue gutter indicator
- **OSC 133 B**: Command input start
- **OSC 133 C**: Command input end
- **OSC 133 D**: Command finished - Green (success) or Red (failure) gutter indicator

### Gutter Markers

Visual indicators in the left gutter show:
- **Blue dots**: Prompt start locations
- **Green dots**: Successful command completion (exit code 0)
- **Red dots**: Failed command completion (exit code ≠ 0)

### Navigation Anchors

`NavAnchor` entities are spawned for each prompt marker, enabling:
- Queryable targets for navigation systems
- Semantic zone boundaries for hint filtering
- Command metadata attachment for context-aware features

## Future Enhancements

- [ ] Full Cosmos-style leader key implementation (Space as leader)
- [ ] Full Spacemacs-style SPC prefix pattern
- [ ] Ratatui UI widget integration for Widget focusable type
- [ ] Command text extraction from terminal buffer for NavAnchors
- [ ] Scrollback integration for accurate viewport-relative navigation
- [ ] Customizable hint label generation (beyond a-z)
- [ ] Multi-character hint filtering (type multiple chars to narrow down)
- [ ] Hint label theming and styling options
- [ ] Copy mode visual selection with navigation integration
- [ ] Search mode with incremental highlighting

## Related Modules

- `crates/scarab-client/src/navigation/mod.rs` - Core navigation types and events
- `crates/scarab-client/src/navigation/focusable.rs` - Focusable detection system
- `crates/scarab-client/src/input/nav_input.rs` - Unified input routing
- `crates/scarab-client/src/ui/link_hints.rs` - Legacy link hints implementation
- `crates/scarab-client/src/prompt_markers.rs` - OSC 133 prompt marker system

## Using Navigation from Plugins

Scarab's plugin system provides a bridge API that allows plugins to interact with the navigation system programmatically. This enables plugins to trigger hint mode, register custom focusable elements, and respond to navigation events.

### Entering Hint Mode Programmatically

Plugins can programmatically enter hint mode using the navigation context:

```rust
// From plugin context
ctx.nav().enter_hint_mode()?;
```

This is useful for plugins that want to provide custom keyboard shortcuts or automation that triggers the hint selection interface.

### Registering Custom Focusables

Plugins can register their own focusable elements that will appear in hint mode alongside terminal-detected focusables:

```rust
let id = ctx.nav().register_focusable(PluginFocusable {
    x: 10, y: 5,
    width: 20, height: 1,
    label: "Open Settings".into(),
    action: PluginFocusableAction::Custom("settings".into()),
})?;

// Later, unregister when done
ctx.nav().unregister_focusable(id)?;
```

**Key Points:**
- Each focusable receives a unique ID for later reference
- Coordinates are in grid cell units (column, row)
- Custom actions are defined by the plugin and handled in plugin code
- Always unregister focusables when they are no longer valid

### Capability Requirements

Plugins must declare appropriate capabilities in their manifest to use navigation features:

```toml
[plugin.capabilities]
can_enter_hint_mode = true        # Required to trigger hint mode
can_register_focusables = true    # Required to register custom focusables
can_trigger_actions = true        # Required to execute navigation actions
```

**Capability Enforcement:**
- Operations are rejected if the plugin lacks required capabilities
- Rejections are logged to telemetry (`nav.plugin_actions_rejected`)
- Sandboxed plugins have stricter restrictions than trusted plugins

## Per-Pane Navigation

Scarab's navigation system maintains independent state for each terminal pane, enabling context-aware navigation across split layouts.

### State Isolation

Each pane maintains its own independent navigation state:

- **Navigation mode** (Normal, Hints, Copy, CommandPalette, Search)
- **Mode stack** for nested mode transitions
- **Focus history** for navigation tracking
- **Hint filter input** for incremental hint filtering

This isolation ensures that operations in one pane don't affect others, allowing users to have different modes active in different panes simultaneously.

### Pane Switch Behavior

When switching between panes, the navigation system preserves and restores state:

1. **Current pane's NavState is preserved** - All mode information is saved
2. **New pane's NavState is restored** - Previous state is loaded, or fresh state is created for new panes
3. **Hint mode may auto-exit** (configurable) - Can be set to exit hints when switching panes to avoid confusion

### Example Workflow

```
Pane A: Hints mode, filter "ab"
  ↓ (User switches to Pane B)
Pane B: Normal mode (independent state)
  ↓ (User switches back to Pane A)
Pane A: Still in Hints mode, filter "ab" preserved
```

**Use Cases:**
- Leave hint mode active in one pane while working in another
- Compare command outputs across panes while maintaining separate focus
- Keep different filtering states for different contexts

### Configuration

Control pane switch behavior via configuration:

```rust
NavConfig {
    exit_hints_on_pane_switch: false,  // Keep hints active across switches
    clear_filter_on_pane_switch: false, // Preserve filter text
    // ...
}
```

## Best Practices

### When to Register Focusables

**Do:**
- Register focusables **after render cycle completes** to ensure accurate positions
- Register in response to terminal output changes or UI updates
- Use entity lifecycle hooks to track when focusables become valid

**Don't:**
- Register during system initialization before the terminal is ready
- Register during the render phase (causes race conditions)
- Register focusables that extend outside the viewport

### Cleanup

Always clean up focusables when they are no longer valid:

```rust
// In plugin code
impl PluginImpl for MyPlugin {
    fn on_unload(&mut self, ctx: &mut PluginContext) {
        // Unregister all focusables before unloading
        for id in &self.focusable_ids {
            ctx.nav().unregister_focusable(*id).ok();
        }
        self.focusable_ids.clear();
    }
}
```

**Key Points:**
- Call `unregister_focusable()` explicitly when elements are removed
- Implement cleanup in plugin `on_unload()` lifecycle hook
- Track focusable IDs for batch cleanup operations
- Silent failures are acceptable during cleanup (use `.ok()`)

### Rate Limiting

The navigation system enforces rate limits to prevent abuse and maintain performance:

- **Action rate limit**: 10 actions/second per plugin
- **Max focusables**: 50 focusables per plugin at any time
- **Burst allowance**: Small burst over limit is tolerated

**Design Guidelines:**
- Batch focusable registrations when possible
- Use debouncing for event-triggered registrations
- Monitor telemetry for rate limit rejections
- Unregister unused focusables promptly to stay under limits

### Bounds Validation

All focusable coordinates are validated before registration:

```rust
// Valid focusable - within reasonable bounds
PluginFocusable {
    x: 10, y: 5,
    width: 20, height: 1,  // Reasonable size
    // ...
}

// Invalid focusable - rejected
PluginFocusable {
    x: -5, y: -10,         // Negative coordinates
    width: 5000, height: 3000,  // Unreasonably large
    // ...
}
```

**Validation Rules:**
- Coordinates must be non-negative
- Width and height must be > 0 and < 1000
- Focusables should be within the current viewport
- Out-of-bounds focusables are rejected silently

## Troubleshooting

### Common Issues

#### Focusables not appearing in hints

**Symptoms:**
- Plugin registers focusables successfully (no errors)
- Hint mode shows no hints for plugin focusables
- Terminal-detected focusables work fine

**Potential Causes:**
1. **Missing capabilities** - Check plugin manifest has `can_register_focusables = true`
2. **Out of viewport** - Focusable coordinates are outside visible terminal area
3. **Generation mismatch** - Focusables registered for old pane generation, cleared on pane switch

**Solutions:**
```rust
// Check capabilities in manifest
[plugin.capabilities]
can_register_focusables = true

// Verify coordinates are within viewport
let (cols, rows) = ctx.terminal().dimensions();
assert!(x < cols && y < rows);

// Re-register focusables after pane switch
ctx.on_pane_focus(|ctx| {
    // Re-register all focusables for new pane
    register_all_focusables(ctx)?;
    Ok(())
});
```

#### Stale focusables after pane switch

**Symptoms:**
- Focusables from previous pane appear in current pane
- Hint labels point to incorrect locations
- Focusable count in telemetry higher than expected

**Cause:**
- Generation mismatch - Focusables not properly associated with pane generation
- Plugin didn't receive pane switch event

**Solution:**
```rust
// Track pane generation in plugin state
struct MyPlugin {
    current_generation: u64,
    focusable_ids: Vec<FocusableId>,
}

// Clear and re-register on pane switch
fn on_pane_switch(&mut self, ctx: &mut PluginContext) {
    // Clear old focusables
    for id in &self.focusable_ids {
        ctx.nav().unregister_focusable(*id).ok();
    }
    self.focusable_ids.clear();

    // Update generation
    self.current_generation = ctx.pane().generation();

    // Re-register focusables for new pane
    self.register_focusables(ctx)?;
}
```

#### Rate limiting rejections

**Symptoms:**
- Plugin actions fail intermittently
- Telemetry shows high `nav.plugin_actions_rejected` count
- Error logs mention rate limiting

**Solutions:**
```rust
// Use debouncing for frequent updates
use std::time::{Duration, Instant};

struct MyPlugin {
    last_registration: Instant,
    min_interval: Duration,
}

impl MyPlugin {
    fn try_update_focusables(&mut self, ctx: &mut PluginContext) {
        // Only update if enough time has passed
        if self.last_registration.elapsed() >= self.min_interval {
            self.update_focusables(ctx)?;
            self.last_registration = Instant::now();
        }
    }
}

// Batch registrations
fn register_multiple_focusables(
    ctx: &mut PluginContext,
    focusables: Vec<PluginFocusable>
) -> Result<Vec<FocusableId>> {
    // Register all at once instead of in a loop with delays
    focusables.into_iter()
        .map(|f| ctx.nav().register_focusable(f))
        .collect()
}
```

### Telemetry to Check

Monitor these telemetry metrics to diagnose navigation issues:

| Metric | Description | Normal Range | Investigation Threshold |
|--------|-------------|--------------|------------------------|
| `nav.focusables_dropped` | Stale focusables removed | 0-10/min | >50/min - Check generation tracking |
| `nav.plugin_actions_rejected` | Actions denied due to capabilities/rate limits | 0/min | >0 - Check capabilities and rate limiting |
| `nav.hint_mode_entries` | Times hint mode was entered | Varies | Unusually high - Check for plugin loops |
| `nav.focusables_registered` | Total focusables registered | 0-500 | >500 - At max limit, cleanup needed |
| `nav.hint_filter_time_ms` | Time to filter hints | <5ms | >50ms - Too many focusables |

**Accessing Telemetry:**
```rust
// In plugin code
let stats = ctx.telemetry().get_nav_stats();
info!("Focusables dropped: {}", stats.focusables_dropped);
info!("Actions rejected: {}", stats.plugin_actions_rejected);

// Query specific metric
let rejected = ctx.telemetry()
    .query("nav.plugin_actions_rejected")
    .last_minute()
    .sum();
```

**Common Patterns:**
- High `focusables_dropped` + pane switches = Generation tracking issue
- High `plugin_actions_rejected` + capability errors = Missing manifest capabilities
- High `plugin_actions_rejected` + rate limit errors = Too frequent operations
- High `hint_filter_time_ms` = Too many focusables, need cleanup
