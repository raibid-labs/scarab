# Navigation Developer Guide

This guide covers the navigation system internals for Scarab developers.

## Quickstart

### Enabling Navigation
- Add NavigationPlugin to your Bevy app
- Configure NavConfig in scarab-config
- Set up keymap bindings

### Basic Usage
- Enter hint mode: press configured key (default: `f`)
- Type hint characters to select target
- Press Enter or action key to execute

## Architecture

### ECS Components

The navigation system is built on the following core components:

#### NavFocus
Marks the currently focused entity in the navigation system.

```rust
#[derive(Component)]
pub struct NavFocus;
```

#### NavHint
Contains the hint label and associated action for a focusable element.

```rust
#[derive(Component)]
pub struct NavHint {
    pub label: String,
    pub position: Vec2,
    pub action: NavAction,
}
```

#### NavMode
Represents the current navigation mode.

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NavMode {
    Normal,
    Hints,
    Insert,
    CommandPalette,
    Copy,
    Search,
}
```

#### FocusableRegion
Represents a detected focusable area with type, bounds, and content.

```rust
#[derive(Component)]
pub struct FocusableRegion {
    pub region_type: FocusableType,
    pub grid_start: (u16, u16),
    pub grid_end: (u16, u16),
    pub content: String,
    pub source: FocusableSource,
    pub screen_position: Option<Vec2>,
}
```

#### NavAnchor
Marks entities as prompt markers for semantic navigation.

```rust
#[derive(Component)]
pub struct NavAnchor {
    pub marker_type: PromptMarkerType,
    pub grid_line: u16,
    pub exit_code: Option<i32>,
}
```

#### NavGroup
Logical grouping for related focusables.

```rust
#[derive(Component)]
pub struct NavGroup(pub String);
```

### Resources

#### NavState
Per-pane navigation state tracking mode, history, and filter.

```rust
#[derive(Resource)]
pub struct NavState {
    pub current_mode: NavMode,
    pub mode_stack: Vec<NavMode>,
    pub focus_history: Vec<Entity>,
    pub hint_filter: String,
    pub max_history_size: usize,
}
```

#### NavStateRegistry
Manages multiple per-pane NavState instances.

```rust
#[derive(Resource)]
pub struct NavStateRegistry {
    states: HashMap<PaneId, NavState>,
    active_pane: Option<PaneId>,
}
```

#### NavMetrics
Performance and usage counters.

```rust
#[derive(Resource, Default)]
pub struct NavMetrics {
    pub focusables_scanned: u64,
    pub focusables_dropped: u64,
    pub hint_mode_entries: u64,
    pub plugin_actions_rejected: u64,
    pub focusables_registered: u64,
}
```

### Events

#### EnterHintModeEvent
Triggers hint mode entry and focusable scanning.

```rust
#[derive(Event)]
pub struct EnterHintModeEvent {
    pub filter_zone: Option<PromptZone>,
}
```

#### ExitHintModeEvent
Triggers hint mode exit and cleanup.

```rust
#[derive(Event)]
pub struct ExitHintModeEvent;
```

#### NavActionEvent
Represents a navigation action to be executed.

```rust
#[derive(Event)]
pub struct NavActionEvent {
    pub action: NavAction,
}
```

#### FocusChangedEvent
Fired when focus changes between focusable elements.

```rust
#[derive(Event)]
pub struct FocusChangedEvent {
    pub old_focus: Option<Entity>,
    pub new_focus: Option<Entity>,
}
```

#### JumpToPromptEvent
Triggers jumping to a specific prompt marker.

```rust
#[derive(Event)]
pub struct JumpToPromptEvent {
    pub target_line: u32,
    pub direction: JumpDirection,
}
```

#### PromptZoneFocusedEvent
Fired when a prompt zone becomes focused for filtering.

```rust
#[derive(Event)]
pub struct PromptZoneFocusedEvent {
    pub start_line: u32,
    pub end_line: u32,
}
```

#### LinkActivatedEvent
Fired when a focusable link is activated.

```rust
#[derive(Event)]
pub struct LinkActivatedEvent {
    pub url: String,
    pub focusable_type: FocusableType,
}
```

## Per-Pane Behavior

### State Isolation

Each terminal pane maintains its own independent NavState:

- **Navigation mode** - Current mode (Normal, Hints, Copy, etc.)
- **Mode stack** - For nested mode transitions
- **Focus history** - Track navigation path
- **Hint filter** - Current filter string for hint matching

This ensures that operations in one pane don't affect others. Users can have different modes active in different panes simultaneously.

### Lifecycle

**Pane creation:**
```rust
// Fresh NavState allocated
registry.create_state_for_pane(pane_id);
```

**Pane switch:**
```rust
// Save current state, restore target state
registry.set_active_pane(new_pane_id);
// Previous pane's state is preserved
// New pane's state becomes active
```

**Pane destruction:**
```rust
// State cleaned up, entities despawned
registry.remove_state_for_pane(pane_id);
// All associated focusables and hints are despawned
```

### Rehydration

When switching panes:

1. **Save current NavState** - Mode, filter, and history preserved
2. **Restore target NavState** - Load previous state or create fresh state
3. **Re-detect focusables** - Scan terminal content for new pane
4. **Increment generation** - FocusableGeneration tracks pane switches
5. **Cleanup stale entities** - Old focusables from previous generation are despawned

Generation tracking prevents stale focusables from previous panes:

```rust
#[derive(Resource)]
pub struct FocusableGeneration {
    pub current: u64,
}

// Incremented on pane change
fn on_pane_switch(mut generation: ResMut<FocusableGeneration>) {
    generation.current += 1;
}

// Used to filter stale entities
fn cleanup_stale_focusables(
    mut commands: Commands,
    generation: Res<FocusableGeneration>,
    focusables: Query<(Entity, &FocusableGeneration)>,
) {
    for (entity, fg) in focusables.iter() {
        if fg.generation < generation.current {
            commands.entity(entity).despawn_recursive();
        }
    }
}
```

## Plugin Bridge Usage

### Registering Focusables

Plugins can register custom focusable elements that appear in hint mode:

```rust
// Plugin context API
let id = ctx.nav().register_focusable(PluginFocusable {
    x: 10,
    y: 5,
    width: 20,
    height: 1,
    label: "Custom Action".to_string(),
    action: PluginFocusableAction::Custom("my-action".to_string()),
})?;

// Later, unregister when done
ctx.nav().unregister_focusable(id)?;
```

**Plugin Focusable Types:**

```rust
pub struct PluginFocusable {
    pub x: u16,
    pub y: u16,
    pub width: u16,
    pub height: u16,
    pub label: String,
    pub action: PluginFocusableAction,
}

pub enum PluginFocusableAction {
    OpenUrl(String),
    OpenFile(String),
    Custom(String),
}
```

### Triggering Actions

Plugins can programmatically trigger navigation actions:

```rust
// Enter hint mode
ctx.nav().enter_hint_mode()?;

// Exit hint mode
ctx.nav().exit_hint_mode()?;

// Jump to prompt
ctx.nav().jump_to_prompt(JumpDirection::Previous)?;
```

### Capability Limits

Plugins must declare navigation capabilities in their manifest:

```toml
[plugin.capabilities]
can_enter_hint_mode = true
can_register_focusables = true
can_trigger_actions = true
```

**Enforcement:**
- Max focusables per plugin: 50 (configurable, default 100)
- Rate limit: 10 actions/second per plugin
- Input validation: bounds checked, URLs sanitized
- Operations rejected if plugin lacks required capabilities
- Rejections logged to telemetry (`nav.plugin_actions_rejected`)

**Validation Rules:**
- Coordinates must be non-negative
- Width and height must be > 0 and < 1000
- Focusables should be within viewport
- Out-of-bounds focusables are rejected silently

## Ratatui Overlay Integration

### Rendering Layers

The navigation system uses z-ordering to ensure hints appear above content:

```rust
pub const LAYER_TERMINAL_BG: f32 = 0.0;
pub const LAYER_TERMINAL_TEXT: f32 = 10.0;
pub const LAYER_IMAGES: f32 = 50.0;
pub const LAYER_HINTS: f32 = 200.0;
pub const LAYER_MODALS: f32 = 300.0;
```

### HintOverlay Component

Hint labels are rendered as overlays:

```rust
#[derive(Component)]
pub struct HintOverlay {
    pub label: String,
    pub position: Vec2,
    pub style: HintStyle,
    pub fade_alpha: f32,
}

pub struct HintStyle {
    pub bg_color: Color,
    pub fg_color: Color,
    pub font_size: f32,
    pub padding: f32,
}
```

**Features:**
- Supports fade animations on enter/exit
- Z-ordered above images, sixel graphics, and terminal content
- Styled independently from terminal theme
- Can be themed via configuration

## Configuration Options

### Keymap Styles

Three built-in keymap styles are supported:

#### Vimium (Default)
Browser extension inspired bindings:
- `f` or `Ctrl+F` - Enter hint mode
- `Esc` - Exit hint mode
- `Ctrl+Up/Down` - Navigate prompts
- `a-z` - Type hint characters

#### Cosmos
Space-based leader key (partially implemented):
- `Space-n` prefix for navigation commands
- `f` - Enter hint mode
- `Esc` - Exit modes

#### Spacemacs
SPC prefix pattern (partially implemented):
- `SPC-n` prefix for navigation
- `f` - Enter hint mode
- `Esc` - Exit modes

### Config Structure (scarab-config)

```toml
[navigation]
style = "vimium"  # or "cosmos", "spacemacs"
hint_chars = "asdfghjkl"
show_labels = true
exit_hints_on_pane_switch = false
clear_filter_on_pane_switch = false

[navigation.keybindings]
enter_hints = "f"
exit_hints = "Escape"
prev_prompt = "Ctrl+Up"
next_prompt = "Ctrl+Down"

[navigation.detection]
max_focusables = 500
url_pattern = "https?://[^\\s<>{}|\\^~\\[\\]`]+|www\\.[^\\s<>{}|\\^~\\[\\]`]+"
filepath_pattern = "(?:~|\\.{1,2}|/)?(?:[a-zA-Z0-9_\\-./]+/)*[a-zA-Z0-9_\\-.]+\\.[a-zA-Z]{2,5}"
email_pattern = "[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\\.[a-zA-Z]{2,}"

[navigation.plugin_limits]
max_focusables_per_plugin = 50
rate_limit_actions_per_second = 10
```

## Testing

### Unit Tests

Navigation system tests are located in:

```
crates/scarab-client/src/navigation/tests.rs
crates/scarab-client/src/navigation/focusable/tests.rs
```

**Test Coverage:**
- Mode transitions and state management
- Focusable detection and regex matching
- Per-pane state isolation
- Generation tracking and cleanup
- Plugin capability enforcement
- Rate limiting
- Bounds validation

### Running Tests

```bash
# Run all navigation tests
cargo test -p scarab-client --lib navigation

# Run specific test module
cargo test -p scarab-client --lib navigation::tests

# Run with output
cargo test -p scarab-client --lib navigation -- --nocapture

# Run smoke test (if available via justfile)
just nav-smoke
```

### Integration Tests

Integration tests verify navigation with full Bevy app:

```bash
# Run integration tests
cargo test -p scarab-client --test navigation_integration
```

### Manual Testing

**Test Per-Pane Behavior:**
1. Create multiple panes (Ctrl+Shift+E)
2. Enter hint mode in pane 1 (`f`)
3. Type partial hint filter (e.g., "a")
4. Switch to pane 2 (Ctrl+Tab)
5. Verify pane 2 is in Normal mode, not Hints
6. Switch back to pane 1
7. Verify hint mode and filter "a" are restored

**Test Focusable Detection:**
1. Output content with URLs, file paths, and emails
2. Enter hint mode (`f`)
3. Verify hints appear for all detected focusables
4. Type hint labels to select and activate
5. Verify correct action is executed (open URL, file, etc.)

**Test Prompt Navigation:**
1. Run several commands to create prompt markers
2. Press Ctrl+Up to jump to previous prompt
3. Verify scroll position moves to previous prompt
4. Press Ctrl+Down to jump to next prompt
5. Verify gutter markers (blue/green/red dots) are visible

## Best Practices

### 1. Always Check pane_id When Creating Focusables

Ensure focusables are associated with the correct pane:

```rust
fn register_focusable_for_pane(
    pane_id: PaneId,
    focusable: FocusableRegion,
    mut commands: Commands,
) {
    commands.spawn((
        focusable,
        PaneAssociation(pane_id), // Tag with pane ID
        FocusableGeneration { generation: current_gen },
    ));
}
```

### 2. Use Generation Tracking to Detect Stale Entities

Track focusable generations to prevent leaks:

```rust
#[derive(Component)]
pub struct FocusableGeneration {
    pub generation: u64,
}

// Clean up stale focusables
fn cleanup_stale_focusables(
    mut commands: Commands,
    current: Res<CurrentGeneration>,
    query: Query<(Entity, &FocusableGeneration)>,
) {
    for (entity, gen) in query.iter() {
        if gen.generation < current.value {
            commands.entity(entity).despawn_recursive();
        }
    }
}
```

### 3. Rate Limit Plugin-Sourced Focusables

Use debouncing to avoid rate limit rejections:

```rust
use std::time::{Duration, Instant};

struct PluginState {
    last_registration: Instant,
    min_interval: Duration,
}

impl PluginState {
    fn try_register_focusable(&mut self, ctx: &PluginContext, f: PluginFocusable) -> Result<()> {
        if self.last_registration.elapsed() >= self.min_interval {
            ctx.nav().register_focusable(f)?;
            self.last_registration = Instant::now();
        }
        Ok(())
    }
}
```

### 4. Clear State on Pane Destroy to Prevent Leaks

Always clean up when panes are destroyed:

```rust
fn on_pane_destroyed(
    mut commands: Commands,
    destroyed_pane: Res<DestroyedPaneId>,
    mut registry: ResMut<NavStateRegistry>,
    focusables: Query<(Entity, &PaneAssociation)>,
) {
    let pane_id = destroyed_pane.0;

    // Remove state from registry
    registry.remove_state_for_pane(pane_id);

    // Despawn all associated focusables
    for (entity, assoc) in focusables.iter() {
        if assoc.0 == pane_id {
            commands.entity(entity).despawn_recursive();
        }
    }
}
```

### 5. Test with Multiple Panes to Verify Isolation

Always test navigation features with multiple panes:

```rust
#[test]
fn test_nav_state_isolation() {
    let mut app = App::new();
    app.add_plugins(NavigationPlugin);

    // Create two panes
    let pane1 = create_pane(&mut app);
    let pane2 = create_pane(&mut app);

    // Enter hint mode in pane1
    app.world.resource_mut::<NavStateRegistry>()
        .set_active_pane(pane1);
    app.world.send_event(EnterHintModeEvent { filter_zone: None });
    app.update();

    // Switch to pane2
    app.world.resource_mut::<NavStateRegistry>()
        .set_active_pane(pane2);
    app.update();

    // Verify pane2 is in Normal mode
    let registry = app.world.resource::<NavStateRegistry>();
    assert_eq!(registry.get_state(pane2).current_mode, NavMode::Normal);

    // Switch back to pane1
    app.world.resource_mut::<NavStateRegistry>()
        .set_active_pane(pane1);
    app.update();

    // Verify pane1 is still in Hints mode
    let registry = app.world.resource::<NavStateRegistry>();
    assert_eq!(registry.get_state(pane1).current_mode, NavMode::Hints);
}
```

## Common Patterns

### Adding Custom Focusable Types

```rust
// 1. Define new focusable type
#[derive(Debug, Clone, PartialEq)]
pub enum FocusableType {
    Url,
    FilePath,
    Email,
    PromptMarker,
    Widget,
    CustomType(String), // Your custom type
}

// 2. Implement detection logic
fn detect_custom_focusables(
    terminal_text: &str,
    commands: &mut Commands,
) {
    let pattern = Regex::new(r"YOUR_PATTERN_HERE").unwrap();

    for cap in pattern.captures_iter(terminal_text) {
        let content = cap.get(0).unwrap().as_str();

        commands.spawn(FocusableRegion {
            region_type: FocusableType::CustomType("my_type".into()),
            content: content.to_string(),
            // ... other fields
        });
    }
}

// 3. Implement action handler
fn handle_custom_focusable_action(
    mut events: EventReader<NavActionEvent>,
) {
    for event in events.read() {
        match &event.action {
            NavAction::Custom(data) => {
                // Handle your custom action
                execute_custom_action(data);
            }
            _ => {}
        }
    }
}
```

### Extending Keybindings

```rust
fn add_custom_keybindings(mut router: ResMut<NavInputRouter>) {
    let mut bindings = router.current_bindings().to_vec();

    // Add custom binding
    bindings.push(
        KeyBinding::new(
            KeyCode::KeyG,
            NavAction::Custom("my_action".into())
        )
        .with_ctrl()
        .in_mode(NavMode::Normal)
    );

    router.set_bindings(NavStyle::VimiumStyle, bindings);
}
```

### Filtering Focusables by Zone

```rust
fn filter_focusables_by_prompt_zone(
    mut focusables: Query<(&mut Visibility, &FocusableRegion)>,
    zone_events: EventReader<PromptZoneFocusedEvent>,
) {
    for event in zone_events.read() {
        for (mut vis, region) in focusables.iter_mut() {
            let in_zone = region.grid_start.1 as u32 >= event.start_line
                       && region.grid_start.1 as u32 < event.end_line;

            *vis = if in_zone {
                Visibility::Visible
            } else {
                Visibility::Hidden
            };
        }
    }
}
```

## Troubleshooting

### Hints Not Appearing

**Symptoms:** Hint mode activates but no hints are displayed

**Possible Causes:**
1. No focusables detected in terminal content
2. Max focusables limit reached
3. Focusables filtered out by zone
4. Rendering layer issue

**Solutions:**
```rust
// Check focusable count
fn debug_focusables(
    focusables: Query<&FocusableRegion>,
    metrics: Res<NavMetrics>,
) {
    info!("Focusables: {}", focusables.iter().count());
    info!("Metrics: {:?}", *metrics);
}

// Verify regex patterns are matching
fn test_detection_patterns() {
    let url_regex = Regex::new(r"https?://[^\s<>{}|\^~\[\]`]+").unwrap();
    let test_text = "Visit https://example.com for more info";
    assert!(url_regex.is_match(test_text));
}

// Check z-ordering
fn verify_hint_layers(
    hints: Query<&Transform, With<NavHint>>,
) {
    for transform in hints.iter() {
        assert_eq!(transform.translation.z, LAYER_HINTS);
    }
}
```

### State Not Persisting Across Pane Switches

**Symptoms:** Navigation state resets when switching panes

**Possible Causes:**
1. NavStateRegistry not properly tracking active pane
2. State being cleared on pane switch
3. Generation tracking causing premature cleanup

**Solutions:**
```rust
// Verify state registry is working
fn debug_nav_state_registry(
    registry: Res<NavStateRegistry>,
) {
    info!("Active pane: {:?}", registry.active_pane());
    info!("State count: {}", registry.state_count());

    if let Some(state) = registry.get_active_state() {
        info!("Current mode: {:?}", state.current_mode);
        info!("Filter: {}", state.hint_filter);
    }
}

// Check configuration
fn verify_config(config: Res<NavConfig>) {
    info!("exit_hints_on_pane_switch: {}", config.exit_hints_on_pane_switch);
    info!("clear_filter_on_pane_switch: {}", config.clear_filter_on_pane_switch);
}
```

### Performance Issues with Many Focusables

**Symptoms:** Lag when entering hint mode or typing hint characters

**Possible Causes:**
1. Too many focusables being detected (>500)
2. Inefficient filtering logic
3. Regex patterns too broad

**Solutions:**
```rust
// Adjust max focusables limit
fn configure_focusable_limits(mut config: ResMut<FocusableScanConfig>) {
    config.max_focusables = 200; // Lower limit for better performance
}

// Optimize filtering
fn optimized_hint_filtering(
    hint_filter: &str,
    hints: &Query<(&NavHint, &mut Visibility)>,
) {
    // Use early returns and efficient string matching
    if hint_filter.is_empty() {
        return;
    }

    for (hint, mut vis) in hints.iter() {
        *vis = if hint.label.starts_with(hint_filter) {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
}

// Monitor metrics
fn monitor_performance(metrics: Res<NavMetrics>) {
    if metrics.focusables_registered > 500 {
        warn!("High focusable count: {}", metrics.focusables_registered);
    }
}
```

### Plugin Actions Being Rejected

**Symptoms:** Plugin navigation calls fail with capability errors

**Possible Causes:**
1. Missing capability declarations in manifest
2. Rate limiting triggered
3. Invalid bounds or parameters

**Solutions:**
```toml
# Ensure manifest has required capabilities
[plugin.capabilities]
can_enter_hint_mode = true
can_register_focusables = true
can_trigger_actions = true
```

```rust
// Check telemetry for rejection reasons
fn debug_plugin_rejections(
    telemetry: Res<NavTelemetry>,
) {
    let rejections = telemetry.get_metric("nav.plugin_actions_rejected");
    info!("Plugin actions rejected: {}", rejections);

    // Check rate limiting
    let actions_per_sec = telemetry.get_rate("nav.plugin_actions");
    if actions_per_sec > 10.0 {
        warn!("Plugin exceeding rate limit: {} actions/sec", actions_per_sec);
    }
}
```

## Further Reading

- [User Navigation Guide](../navigation.md) - End-user documentation
- [Plugin Development Guide](../plugin-development-guide.md) - Building plugins with navigation
- [Bevy ECS Documentation](https://docs.rs/bevy/latest/bevy/ecs/) - Understanding Bevy's ECS
- [Vimium Browser Extension](https://github.com/philc/vimium) - Inspiration for hint mode
