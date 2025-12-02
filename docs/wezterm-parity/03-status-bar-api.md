# WS-3: Status Bar Rendering API

**Workstream ID:** WS-3
**Priority:** P1 (High Value)
**Estimated Complexity:** Medium
**Dependencies:** WS-1 (Object Model), WS-2 (Event System)

## Overview

WezTerm's most visible customization feature is its programmable status bar. Users write Lua functions that return styled text, which WezTerm renders at the top/bottom of the window. Scarab currently hardcodes its status bar—this workstream makes it fully programmable.

## Current State Analysis

### Scarab's Current Status Bar

The status bar (if any) is defined in Rust in `scarab-client/src/ui/`:

```rust
// Hardcoded UI elements
fn setup_status_bar(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(TextBundle {
        text: Text::from_section("Scarab Terminal", TextStyle { ... }),
        // Fixed position, fixed content
    });
}
```

**Limitations:**
- No way to customize from scripts
- No dynamic content (time, CWD, process info)
- No styling API

### WezTerm's Approach

```lua
wezterm.on('update-right-status', function(window, pane)
  local cwd = pane:get_current_working_dir()
  local time = wezterm.strftime '%H:%M'

  window:set_right_status(wezterm.format {
    { Foreground = { Color = '#7aa2f7' } },
    { Text = cwd and cwd.file_path or '' },
    { Text = ' | ' },
    { Foreground = { Color = '#bb9af7' } },
    { Text = time },
  })
end)
```

**Power:**
- Full control over content
- Rich styling (colors, attributes)
- Dynamic updates
- Access to terminal state

## Target API

### Fusabi Syntax

```fsx
// In ~/.config/scarab/config.fsx
module StatusBar

open Scarab.UI

// Register status bar update handler
On(EventType.UpdateRightStatus, fun window pane ->
    let cwd = pane.GetCurrentWorkingDir()
    let time = DateTime.Now.ToString("HH:mm")

    window.SetRightStatus([
        Foreground("#7aa2f7")
        Text(cwd |> Option.defaultValue "")
        Text(" | ")
        Foreground("#bb9af7")
        Text(time)
    ])
)

On(EventType.UpdateLeftStatus, fun window pane ->
    let process = pane.GetForegroundProcessName()
    window.SetLeftStatus([
        Bold
        Text(process)
    ])
)
```

### RenderItem Types

```rust
// In scarab-plugin-api
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum RenderItem {
    // Content
    Text(String),
    Icon(String),          // Nerd font icon name

    // Foreground colors
    Foreground(Color),
    ForegroundAnsi(AnsiColor),

    // Background colors
    Background(Color),
    BackgroundAnsi(AnsiColor),

    // Attributes
    Bold,
    Italic,
    Underline(UnderlineStyle),
    Strikethrough,

    // Reset
    ResetAttributes,
    ResetForeground,
    ResetBackground,

    // Layout
    Spacer,               // Flexible space
    Padding(u8),          // Fixed space (cells)
    Separator(String),    // e.g., " | "
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Color {
    Rgb(u8, u8, u8),
    Hex(String),          // "#ff0000"
    Named(String),        // "red", "blue"
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum AnsiColor {
    Black, Red, Green, Yellow, Blue, Magenta, Cyan, White,
    BrightBlack, BrightRed, BrightGreen, BrightYellow,
    BrightBlue, BrightMagenta, BrightCyan, BrightWhite,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum UnderlineStyle {
    Single,
    Double,
    Curly,
    Dotted,
    Dashed,
}
```

### Window Methods

```rust
impl WindowProxy {
    /// Set the right-side status bar content
    pub fn set_right_status(&self, items: Vec<RenderItem>) -> Result<()>;

    /// Set the left-side status bar content
    pub fn set_left_status(&self, items: Vec<RenderItem>) -> Result<()>;

    /// Clear status bar
    pub fn clear_status(&self) -> Result<()>;
}
```

## Architecture Design

### Status Bar Data Flow

```
┌─────────────────────────────────────────────────────────────────┐
│ Event: UpdateStatus (triggered every 100ms or on change)        │
└───────────────────────────┬─────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────────┐
│ Fusabi Script Execution                                          │
│                                                                  │
│   On(UpdateRightStatus, fun window pane ->                      │
│       window.SetRightStatus([...])                              │
│   )                                                             │
└───────────────────────────┬─────────────────────────────────────┘
                            │ Vec<RenderItem>
                            ▼
┌─────────────────────────────────────────────────────────────────┐
│ Status Bar Resource (Bevy)                                       │
│                                                                  │
│   pub struct StatusBarState {                                   │
│       left_items: Vec<RenderItem>,                              │
│       right_items: Vec<RenderItem>,                             │
│       dirty: bool,                                              │
│   }                                                             │
└───────────────────────────┬─────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────────┐
│ Status Bar Renderer (Bevy System)                                │
│                                                                  │
│   - Convert RenderItem to TextSection                           │
│   - Apply colors, styles                                         │
│   - Layout left/right sections                                   │
│   - Update UI entities                                          │
└─────────────────────────────────────────────────────────────────┘
```

### Bevy Integration

```rust
// In scarab-client/src/ui/status_bar.rs

#[derive(Resource, Default)]
pub struct StatusBarState {
    pub left_items: Vec<RenderItem>,
    pub right_items: Vec<RenderItem>,
    pub left_dirty: bool,
    pub right_dirty: bool,
}

#[derive(Component)]
pub struct StatusBarLeft;

#[derive(Component)]
pub struct StatusBarRight;

pub fn setup_status_bar(mut commands: Commands) {
    // Container
    commands.spawn(NodeBundle {
        style: Style {
            width: Val::Percent(100.0),
            height: Val::Px(24.0),
            justify_content: JustifyContent::SpaceBetween,
            ..default()
        },
        background_color: Color::srgba(0.1, 0.1, 0.1, 0.9).into(),
        ..default()
    })
    .with_children(|parent| {
        // Left section
        parent.spawn((
            TextBundle::default(),
            StatusBarLeft,
        ));

        // Right section
        parent.spawn((
            TextBundle::default(),
            StatusBarRight,
        ));
    });
}

pub fn update_status_bar_system(
    status: Res<StatusBarState>,
    mut left_query: Query<&mut Text, (With<StatusBarLeft>, Without<StatusBarRight>)>,
    mut right_query: Query<&mut Text, (With<StatusBarRight>, Without<StatusBarLeft>)>,
) {
    if status.left_dirty {
        if let Ok(mut text) = left_query.get_single_mut() {
            *text = render_items_to_text(&status.left_items);
        }
    }

    if status.right_dirty {
        if let Ok(mut text) = right_query.get_single_mut() {
            *text = render_items_to_text(&status.right_items);
        }
    }
}

fn render_items_to_text(items: &[RenderItem]) -> Text {
    let mut sections = Vec::new();
    let mut current_style = TextStyle::default();

    for item in items {
        match item {
            RenderItem::Text(s) => {
                sections.push(TextSection::new(s.clone(), current_style.clone()));
            }
            RenderItem::Foreground(color) => {
                current_style.color = color_to_bevy(color);
            }
            RenderItem::Bold => {
                // Note: Bevy text doesn't support weight directly
                // We'd need custom font handling
            }
            RenderItem::ResetAttributes => {
                current_style = TextStyle::default();
            }
            RenderItem::Spacer => {
                // Add flexible space - handled at layout level
            }
            // ... handle other items
        }
    }

    Text::from_sections(sections)
}
```

### IPC Protocol

Status bar updates from scripts need to reach the client:

```rust
// When Fusabi calls window.SetRightStatus(items)
pub enum DaemonMessage {
    // ... existing variants ...

    StatusBarUpdate {
        window_id: u64,
        side: StatusBarSide,
        items: Vec<RenderItem>,
    },
}

pub enum StatusBarSide {
    Left,
    Right,
}
```

## Format Function

Provide a convenience function similar to `wezterm.format`:

```fsx
// Fusabi helper
let format (items: FormatItem list) : RenderItem list =
    items |> List.collect parseFormatItem

// Usage
window.SetRightStatus(format [
    { Foreground = Color.Hex "#7aa2f7" }
    { Text = "Hello" }
    { Attribute = Bold }
    { Text = "World" }
])
```

### FormatItem Type

```rust
// Alternative syntax matching WezTerm
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum FormatItem {
    Text(String),
    Foreground { color: ColorSpec },
    Background { color: ColorSpec },
    Attribute { attr: TextAttribute },
    ResetAttributes,
}

pub enum ColorSpec {
    AnsiColor(AnsiColor),
    Color(String),  // Hex or named
}

pub enum TextAttribute {
    Underline(UnderlineStyle),
    Intensity(Intensity),
    Italic(bool),
}

pub enum Intensity {
    Normal,
    Bold,
    Half,  // Dim
}
```

## Trigger Events

### Periodic Updates

```rust
fn status_update_timer_system(
    time: Res<Time>,
    mut timer: ResMut<StatusUpdateTimer>,
    event_registry: Res<EventRegistry>,
    window_query: Query<Entity, With<PrimaryWindow>>,
) {
    if timer.0.tick(time.delta()).just_finished() {
        // Dispatch UpdateStatus event
        for window_entity in window_query.iter() {
            let args = EventArgs {
                event_type: EventType::UpdateStatus,
                window: Some(object_registry.get_handle(window_entity)),
                ..default()
            };
            event_registry.dispatch(&args);
        }
    }
}

#[derive(Resource)]
struct StatusUpdateTimer(Timer);

impl Default for StatusUpdateTimer {
    fn default() -> Self {
        Self(Timer::from_seconds(0.1, TimerMode::Repeating))  // 10 Hz
    }
}
```

### Change-Driven Updates

Status updates also trigger on state changes:

```rust
// On focus change
fn on_window_focus_changed(/* ... */) {
    dispatch_event(EventType::UpdateStatus, /* ... */);
}

// On title change
fn on_pane_title_changed(/* ... */) {
    dispatch_event(EventType::UpdateStatus, /* ... */);
}

// On CWD change
fn on_cwd_changed(/* ... */) {
    dispatch_event(EventType::UpdateStatus, /* ... */);
}
```

## Built-in Status Components

Provide pre-built components users can include:

```fsx
// Built-in status components
module Scarab.StatusComponents

let clock() = [
    Text(DateTime.Now.ToString("HH:mm"))
]

let cwd(pane: Pane) = [
    Text(pane.GetCurrentWorkingDir() |> Option.defaultValue "~")
]

let processName(pane: Pane) = [
    Text(pane.GetForegroundProcessName())
]

let hostname() = [
    Text(Environment.MachineName)
]

let battery() = [
    // Platform-specific battery info
    Text(Scarab.Platform.GetBatteryLevel() + "%")
]

// Usage
On(EventType.UpdateRightStatus, fun window pane ->
    window.SetRightStatus([
        yield! cwd(pane)
        Text(" | ")
        yield! clock()
    ])
)
```

## Implementation Plan

### Phase 1: Core Infrastructure (Week 1)

1. Define `RenderItem` enum in `scarab-plugin-api`
2. Create `StatusBarState` resource in client
3. Implement `setup_status_bar` Bevy system
4. Implement `update_status_bar_system`

### Phase 2: Window Methods (Week 1)

1. Add `set_right_status` to `WindowProxy`
2. Add `set_left_status` to `WindowProxy`
3. Wire up IPC for daemon->client status updates
4. Test basic status bar updates

### Phase 3: Event Integration (Week 2)

1. Add `UpdateStatus` event type
2. Implement periodic timer trigger
3. Add change-driven triggers (focus, title)
4. Test event flow

### Phase 4: Fusabi Integration (Week 2)

1. Create `format` helper function
2. Expose `RenderItem` types to Fusabi
3. Document status bar API
4. Create example configurations

### Phase 5: Built-in Components (Week 3)

1. Implement `clock()`, `cwd()`, `processName()`
2. Add platform-specific components (battery, etc.)
3. Create default status bar config
4. Write user documentation

## Styling Considerations

### Font Support

Status bar needs monospace font with:
- Nerd Font icons (for Icon items)
- Bold/italic variants
- Unicode support

```rust
// Font handling
fn get_status_bar_font(weight: FontWeight, style: FontStyle) -> Handle<Font> {
    // Load appropriate font variant
}
```

### Color Theming

Colors should respect the current theme:

```rust
impl Color {
    pub fn resolve(&self, theme: &Theme) -> bevy::color::Color {
        match self {
            Color::Rgb(r, g, b) => bevy::color::Color::srgb_u8(*r, *g, *b),
            Color::Named(name) => theme.get_color(name).unwrap_or(Color::WHITE),
            Color::Hex(hex) => parse_hex_color(hex),
        }
    }
}
```

## Testing Strategy

### Unit Tests

```rust
#[test]
fn test_render_items_to_text() {
    let items = vec![
        RenderItem::Foreground(Color::Hex("#ff0000".into())),
        RenderItem::Text("Hello".into()),
        RenderItem::ResetAttributes,
        RenderItem::Text("World".into()),
    ];

    let text = render_items_to_text(&items);

    assert_eq!(text.sections.len(), 2);
    assert_eq!(text.sections[0].value, "Hello");
    assert_eq!(text.sections[1].value, "World");
}
```

### Integration Tests

```fsx
// test_status_bar.fsx
let mutable statusUpdates = 0

On(EventType.UpdateRightStatus, fun window pane ->
    statusUpdates <- statusUpdates + 1
    window.SetRightStatus([Text "Test"])
    Continue
)

// Wait for timer
Thread.Sleep(200)
assert (statusUpdates >= 1)
```

## Success Criteria

- [ ] `window.SetRightStatus([Text "hello"])` renders text
- [ ] Colors work: `Foreground(Color.Hex "#ff0000")`
- [ ] Periodic updates trigger at configured interval
- [ ] Multiple plugins can contribute to status bar
- [ ] Status bar updates don't impact terminal performance
- [ ] Built-in components (clock, cwd) work out of box
