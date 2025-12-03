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
