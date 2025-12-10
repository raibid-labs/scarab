//! Status Bar rendering system for Scarab terminal emulator
//!
//! Provides Bevy resources, components, and systems for rendering
//! programmable status bars with rich styling and dynamic content.

use bevy::prelude::*;

/// Height of the status bar in pixels
pub const STATUS_BAR_HEIGHT: f32 = 24.0;
use scarab_plugin_api::status_bar::Color as StatusColor;
use scarab_plugin_api::status_bar::{AnsiColor, RenderItem};
use scarab_protocol::{DaemonMessage, StatusBarSide as ProtocolStatusBarSide, StatusRenderItem};
use std::sync::{Arc, Mutex};
use tokio::sync::broadcast;

/// Plugin for status bar functionality
pub struct StatusBarPlugin;

impl Plugin for StatusBarPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<StatusBarState>()
            .init_resource::<StatusUpdateTimer>()
            .init_resource::<TabState>()
            .add_event::<StatusUpdateEvent>()
            .add_event::<TabSwitchEvent>()
            .add_systems(Startup, setup_status_bar)
            .add_systems(
                Update,
                (
                    receive_status_updates,
                    trigger_status_update,
                    update_status_bar_system,
                    handle_tab_switch,
                    update_tab_display,
                )
                    .chain(),
            );
    }
}

/// Resource for daemon message receiver (for status bar IPC)
#[derive(Resource)]
pub struct DaemonMessageReceiver(pub Arc<Mutex<broadcast::Receiver<DaemonMessage>>>);

/// Timer resource for triggering status updates
#[derive(Resource)]
pub struct StatusUpdateTimer {
    pub timer: Timer,
}

impl Default for StatusUpdateTimer {
    fn default() -> Self {
        Self {
            timer: Timer::from_seconds(0.1, TimerMode::Repeating), // 100ms interval
        }
    }
}

/// Event to trigger status bar updates
#[derive(Event)]
pub struct StatusUpdateEvent;

/// Event to switch tabs
#[derive(Event)]
pub struct TabSwitchEvent {
    pub tab_index: usize,
}

/// Resource holding current status bar state
///
/// Tracks the render items for left and right sections of the status bar
/// along with dirty flags to optimize rendering.
#[derive(Resource, Default)]
pub struct StatusBarState {
    /// Items to display on the left side
    pub left_items: Vec<RenderItem>,
    /// Items to display on the right side
    pub right_items: Vec<RenderItem>,
    /// Whether left side needs re-rendering
    pub left_dirty: bool,
    /// Whether right side needs re-rendering
    pub right_dirty: bool,
}

impl StatusBarState {
    /// Update the left side of the status bar
    pub fn set_left(&mut self, items: Vec<RenderItem>) {
        self.left_items = items;
        self.left_dirty = true;
    }

    /// Update the right side of the status bar
    pub fn set_right(&mut self, items: Vec<RenderItem>) {
        self.right_items = items;
        self.right_dirty = true;
    }

    /// Clear all status bar content
    pub fn clear(&mut self) {
        self.left_items.clear();
        self.right_items.clear();
        self.left_dirty = true;
        self.right_dirty = true;
    }

    /// Clear dirty flags (called after rendering)
    pub fn clear_dirty(&mut self) {
        self.left_dirty = false;
        self.right_dirty = false;
    }
}

/// Resource holding tab state
#[derive(Resource)]
pub struct TabState {
    pub tabs: Vec<String>,
    pub active_index: usize,
}

impl Default for TabState {
    fn default() -> Self {
        Self {
            tabs: vec![
                "meta".to_string(),
                "phage".to_string(),
                "tolaria".to_string(),
            ],
            active_index: 0,
        }
    }
}

/// Marker component for the left section of the status bar
#[derive(Component)]
pub struct StatusBarLeft;

/// Marker component for the right section of the status bar
#[derive(Component)]
pub struct StatusBarRight;

/// Marker component for the status bar container
#[derive(Component)]
pub struct StatusBarContainer;

/// Marker component for tab container
#[derive(Component)]
pub struct TabContainer;

/// Marker component for individual tabs
#[derive(Component)]
pub struct TabLabel {
    pub index: usize,
}

/// Setup the status bar UI hierarchy
///
/// Creates a horizontal container with left and right text sections.
/// The status bar is positioned at the bottom of the window.
fn setup_status_bar(mut commands: Commands, tab_state: Res<TabState>) {
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Px(STATUS_BAR_HEIGHT),
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::Center,
                padding: UiRect::horizontal(Val::Px(8.0)),
                position_type: PositionType::Absolute,
                bottom: Val::Px(0.0),
                left: Val::Px(0.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.15, 0.15, 0.18, 0.95)),
            ZIndex(1000),
            StatusBarContainer,
        ))
        .with_children(|parent| {
            // Left section - tab container
            parent
                .spawn((
                    Node {
                        display: Display::Flex,
                        flex_direction: FlexDirection::Row,
                        column_gap: Val::Px(4.0),
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    TabContainer,
                ))
                .with_children(|tabs_parent| {
                    // Spawn tab labels
                    for (index, tab_name) in tab_state.tabs.iter().enumerate() {
                        let is_active = index == tab_state.active_index;

                        // Slime theme colors
                        let active_bg = Color::srgb(0.66, 0.87, 0.35); // #a8df5a - slime green
                        let active_fg = Color::srgb(0.12, 0.14, 0.14); // #1e2324 - dark background
                        let inactive_fg = Color::srgb(0.78, 0.76, 0.62); // #c8dba8 - muted green

                        tabs_parent
                            .spawn((
                                Node {
                                    padding: UiRect::axes(Val::Px(8.0), Val::Px(4.0)),
                                    margin: UiRect::right(Val::Px(2.0)),
                                    ..default()
                                },
                                BackgroundColor(if is_active { active_bg } else { Color::NONE }),
                                TabLabel { index },
                            ))
                            .with_children(|label_parent| {
                                label_parent.spawn((
                                    Text::new(tab_name),
                                    TextFont {
                                        font_size: 14.0,
                                        ..default()
                                    },
                                    TextColor(if is_active { active_fg } else { inactive_fg }),
                                ));
                            });
                    }
                });

            // Right section - show mode indicator
            parent.spawn((
                Text::new("NORMAL"),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(Color::srgb(0.78, 0.76, 0.62)), // Slime theme muted green
                StatusBarRight,
            ));
        });
}

/// Update status bar content when dirty flags are set
///
/// Converts RenderItem sequences to Bevy Text with appropriate styling.
fn update_status_bar_system(
    mut status: ResMut<StatusBarState>,
    mut right_query: Query<&mut Text, With<StatusBarRight>>,
) {
    // Note: Left side is now handled by tabs, not by left_items

    if status.right_dirty {
        if let Ok(mut text) = right_query.get_single_mut() {
            **text = render_items_to_text(&status.right_items);
        }
    }

    // Clear dirty flags after rendering
    if status.left_dirty || status.right_dirty {
        status.clear_dirty();
    }
}

/// System to handle tab switch events
fn handle_tab_switch(mut events: EventReader<TabSwitchEvent>, mut tab_state: ResMut<TabState>) {
    for event in events.read() {
        if event.tab_index < tab_state.tabs.len() {
            tab_state.active_index = event.tab_index;
        }
    }
}

/// System to update tab display when active tab changes
fn update_tab_display(
    tab_state: Res<TabState>,
    mut tab_query: Query<(&TabLabel, &mut BackgroundColor, &Children)>,
    mut text_query: Query<&mut TextColor>,
) {
    if !tab_state.is_changed() {
        return;
    }

    // Slime theme colors
    let active_bg = Color::srgb(0.66, 0.87, 0.35); // #a8df5a - slime green
    let active_fg = Color::srgb(0.12, 0.14, 0.14); // #1e2324 - dark background
    let inactive_fg = Color::srgb(0.78, 0.76, 0.62); // #c8dba8 - muted green

    for (tab_label, mut bg_color, children) in tab_query.iter_mut() {
        let is_active = tab_label.index == tab_state.active_index;

        // Update background
        *bg_color = if is_active {
            BackgroundColor(active_bg)
        } else {
            BackgroundColor(Color::NONE)
        };

        // Update text color for child text entity
        for &child in children.iter() {
            if let Ok(mut text_color) = text_query.get_mut(child) {
                *text_color = TextColor(if is_active { active_fg } else { inactive_fg });
            }
        }
    }
}

/// System to receive status bar updates from daemon via IPC
///
/// Processes StatusBarUpdate messages from the daemon and updates the
/// StatusBarState resource accordingly.
fn receive_status_updates(
    receiver: Option<Res<DaemonMessageReceiver>>,
    mut status: ResMut<StatusBarState>,
) {
    let Some(receiver) = receiver else {
        return;
    };

    let mut receiver = receiver.0.lock().unwrap();

    // Process all available messages without blocking
    loop {
        match receiver.try_recv() {
            Ok(DaemonMessage::StatusBarUpdate {
                window_id: _,
                side,
                items,
            }) => {
                // Convert protocol items to RenderItems
                let render_items: Vec<RenderItem> = items
                    .into_iter()
                    .filter_map(convert_protocol_item_to_render_item)
                    .collect();

                // Update the appropriate side
                match side {
                    ProtocolStatusBarSide::Left => {
                        status.set_left(render_items);
                    }
                    ProtocolStatusBarSide::Right => {
                        status.set_right(render_items);
                    }
                }
            }
            Ok(_other_message) => {
                // Other daemon messages - not status bar updates
                // These are handled elsewhere
            }
            Err(broadcast::error::TryRecvError::Empty) => {
                // No more messages available
                break;
            }
            Err(broadcast::error::TryRecvError::Lagged(skipped)) => {
                warn!("Status bar receiver lagged, skipped {} messages", skipped);
                // Continue processing available messages
            }
            Err(broadcast::error::TryRecvError::Closed) => {
                warn!("Daemon message channel closed");
                break;
            }
        }
    }
}

/// System to trigger status bar updates periodically
///
/// Dispatches StatusUpdateEvent at regular intervals (100ms) to allow
/// plugins to update their status bar content.
fn trigger_status_update(
    time: Res<Time>,
    mut timer: ResMut<StatusUpdateTimer>,
    mut events: EventWriter<StatusUpdateEvent>,
) {
    timer.timer.tick(time.delta());

    if timer.timer.just_finished() {
        events.send(StatusUpdateEvent);
    }
}

/// Convert protocol StatusRenderItem to plugin-api RenderItem
///
/// Maps the simplified IPC representation to the full RenderItem type.
fn convert_protocol_item_to_render_item(item: StatusRenderItem) -> Option<RenderItem> {
    match item {
        StatusRenderItem::Text(s) => Some(RenderItem::Text(s)),
        StatusRenderItem::Icon(icon) => Some(RenderItem::Icon(icon)),
        StatusRenderItem::Foreground { r, g, b } => {
            Some(RenderItem::Foreground(StatusColor::Rgb(r, g, b)))
        }
        StatusRenderItem::Background { r, g, b } => {
            Some(RenderItem::Background(StatusColor::Rgb(r, g, b)))
        }
        StatusRenderItem::Bold => Some(RenderItem::Bold),
        StatusRenderItem::Italic => Some(RenderItem::Italic),
        StatusRenderItem::ResetAttributes => Some(RenderItem::ResetAttributes),
        StatusRenderItem::Spacer => Some(RenderItem::Spacer),
        StatusRenderItem::Padding(count) => Some(RenderItem::Padding(count)),
        StatusRenderItem::Separator(sep) => Some(RenderItem::Separator(sep)),
    }
}

/// Styled text segment for status bar rendering
#[derive(Debug, Clone)]
pub struct StyledTextSegment {
    pub text: String,
    pub color: Color,
    pub is_bold: bool,
    pub is_italic: bool,
}

impl Default for StyledTextSegment {
    fn default() -> Self {
        Self {
            text: String::new(),
            color: Color::srgb(0.9, 0.9, 0.9), // Default light gray
            is_bold: false,
            is_italic: false,
        }
    }
}

/// Convert a sequence of RenderItems to styled text segments
///
/// Processes the items sequentially, building up styled text segments.
/// Each segment has its own color and style attributes.
///
/// # Arguments
///
/// * `items` - Slice of RenderItem elements to convert
///
/// # Returns
///
/// A vector of styled text segments for rendering with Bevy Text
pub fn render_items_to_styled_text(items: &[RenderItem]) -> Vec<StyledTextSegment> {
    let mut segments = Vec::new();
    let mut current_segment = StyledTextSegment::default();
    let mut current_fg = Color::srgb(0.9, 0.9, 0.9); // Default foreground
    let mut is_bold = false;
    let mut is_italic = false;

    for item in items {
        match item {
            RenderItem::Text(s) => {
                // If text is being added, finalize current segment if it has content
                // and start a new one if style changed
                if !current_segment.text.is_empty()
                    && (current_segment.color != current_fg
                        || current_segment.is_bold != is_bold
                        || current_segment.is_italic != is_italic)
                {
                    segments.push(current_segment);
                    current_segment = StyledTextSegment {
                        text: String::new(),
                        color: current_fg,
                        is_bold,
                        is_italic,
                    };
                }
                current_segment.color = current_fg;
                current_segment.is_bold = is_bold;
                current_segment.is_italic = is_italic;
                current_segment.text.push_str(s);
            }
            RenderItem::Icon(icon) => {
                current_segment.text.push_str(icon);
            }
            RenderItem::Foreground(color) => {
                // Push current segment if it has content, then change color
                if !current_segment.text.is_empty() {
                    segments.push(current_segment);
                    current_segment = StyledTextSegment::default();
                }
                current_fg = color_to_bevy(color);
            }
            RenderItem::ForegroundAnsi(ansi) => {
                if !current_segment.text.is_empty() {
                    segments.push(current_segment);
                    current_segment = StyledTextSegment::default();
                }
                current_fg = ansi_color_to_bevy(ansi);
            }
            RenderItem::Background(_color) => {
                // Background colors would require spawning separate UI nodes
                // For now, we skip background styling
            }
            RenderItem::BackgroundAnsi(_ansi) => {
                // Background colors not yet supported
            }
            RenderItem::Bold => {
                if !current_segment.text.is_empty() && !current_segment.is_bold {
                    segments.push(current_segment);
                    current_segment = StyledTextSegment::default();
                }
                is_bold = true;
            }
            RenderItem::Italic => {
                if !current_segment.text.is_empty() && !current_segment.is_italic {
                    segments.push(current_segment);
                    current_segment = StyledTextSegment::default();
                }
                is_italic = true;
            }
            RenderItem::Underline(_style) => {
                // Underline not directly supported by Bevy Text
            }
            RenderItem::Strikethrough => {
                // Strikethrough not directly supported by Bevy Text
            }
            RenderItem::ResetAttributes => {
                if !current_segment.text.is_empty() {
                    segments.push(current_segment);
                    current_segment = StyledTextSegment::default();
                }
                current_fg = Color::srgb(0.9, 0.9, 0.9);
                is_bold = false;
                is_italic = false;
            }
            RenderItem::ResetForeground => {
                if !current_segment.text.is_empty() {
                    segments.push(current_segment);
                    current_segment = StyledTextSegment::default();
                }
                current_fg = Color::srgb(0.9, 0.9, 0.9);
            }
            RenderItem::ResetBackground => {
                // Background reset - no-op since we don't render backgrounds
            }
            RenderItem::Spacer => {
                current_segment.text.push(' ');
            }
            RenderItem::Padding(count) => {
                for _ in 0..*count {
                    current_segment.text.push(' ');
                }
            }
            RenderItem::Separator(sep) => {
                current_segment.text.push_str(sep);
            }
        }
    }

    // Push final segment if it has content
    if !current_segment.text.is_empty() {
        segments.push(current_segment);
    }

    segments
}

/// Convert a sequence of RenderItems to Bevy Text string (legacy compatibility)
///
/// Processes the items sequentially, building up plain text.
/// For styled text, use `render_items_to_styled_text` instead.
///
/// # Arguments
///
/// * `items` - Slice of RenderItem elements to convert
///
/// # Returns
///
/// A plain text string (no styling)
pub fn render_items_to_text(items: &[RenderItem]) -> String {
    let mut result = String::new();

    for item in items {
        match item {
            RenderItem::Text(s) => {
                result.push_str(s);
            }
            RenderItem::Icon(icon) => {
                result.push_str(icon);
            }
            RenderItem::Foreground(_) | RenderItem::ForegroundAnsi(_) => {}
            RenderItem::Background(_) | RenderItem::BackgroundAnsi(_) => {}
            RenderItem::Bold | RenderItem::Italic => {}
            RenderItem::Underline(_) | RenderItem::Strikethrough => {}
            RenderItem::ResetAttributes
            | RenderItem::ResetForeground
            | RenderItem::ResetBackground => {}
            RenderItem::Spacer => {
                result.push(' ');
            }
            RenderItem::Padding(count) => {
                for _ in 0..*count {
                    result.push(' ');
                }
            }
            RenderItem::Separator(sep) => {
                result.push_str(sep);
            }
        }
    }

    result
}

/// Convert Color enum to Bevy color
fn color_to_bevy(color: &StatusColor) -> Color {
    match color {
        StatusColor::Rgb(r, g, b) => Color::srgb_u8(*r, *g, *b),
        StatusColor::Hex(hex) => parse_hex_color(hex),
        StatusColor::Named(name) => parse_named_color(name),
    }
}

/// Convert AnsiColor to Bevy color
fn ansi_color_to_bevy(ansi: &AnsiColor) -> Color {
    let (r, g, b) = ansi.to_rgb();
    Color::srgb_u8(r, g, b)
}

/// Parse hex color string to Bevy color
///
/// Supports formats: "#RRGGBB" or "RRGGBB"
fn parse_hex_color(hex: &str) -> Color {
    let hex = hex.trim_start_matches('#');

    if hex.len() != 6 {
        // Invalid hex format, return default
        return Color::srgb(0.9, 0.9, 0.9);
    }

    let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(255);
    let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(255);
    let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(255);

    Color::srgb_u8(r, g, b)
}

/// Parse named color to Bevy color
///
/// Supports basic CSS color names
fn parse_named_color(name: &str) -> Color {
    match name.to_lowercase().as_str() {
        "black" => Color::srgb(0.0, 0.0, 0.0),
        "white" => Color::srgb(1.0, 1.0, 1.0),
        "red" => Color::srgb(1.0, 0.0, 0.0),
        "green" => Color::srgb(0.0, 1.0, 0.0),
        "blue" => Color::srgb(0.0, 0.0, 1.0),
        "yellow" => Color::srgb(1.0, 1.0, 0.0),
        "cyan" => Color::srgb(0.0, 1.0, 1.0),
        "magenta" => Color::srgb(1.0, 0.0, 1.0),
        "orange" => Color::srgb(1.0, 0.65, 0.0),
        "purple" => Color::srgb(0.5, 0.0, 0.5),
        "pink" => Color::srgb(1.0, 0.75, 0.8),
        "gray" | "grey" => Color::srgb(0.5, 0.5, 0.5),
        "darkgray" | "darkgrey" => Color::srgb(0.25, 0.25, 0.25),
        "lightgray" | "lightgrey" => Color::srgb(0.75, 0.75, 0.75),
        _ => Color::srgb(0.9, 0.9, 0.9), // Default fallback
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_items_to_text_simple() {
        let items = vec![
            RenderItem::Text("Hello".to_string()),
            RenderItem::Text("World".to_string()),
        ];

        let text = render_items_to_text(&items);
        assert_eq!(text, "HelloWorld");
    }

    #[test]
    fn test_render_items_with_color() {
        let items = vec![
            RenderItem::Foreground(StatusColor::Hex("#ff0000".to_string())),
            RenderItem::Text("Red".to_string()),
            RenderItem::ResetAttributes,
            RenderItem::Text("Normal".to_string()),
        ];

        let text = render_items_to_text(&items);
        assert_eq!(text, "RedNormal");
    }

    #[test]
    fn test_render_items_with_padding() {
        let items = vec![
            RenderItem::Text("A".to_string()),
            RenderItem::Padding(3),
            RenderItem::Text("B".to_string()),
        ];

        let text = render_items_to_text(&items);
        assert_eq!(text, "A   B");
    }

    #[test]
    fn test_render_items_with_separator() {
        let items = vec![
            RenderItem::Text("Section 1".to_string()),
            RenderItem::Separator(" | ".to_string()),
            RenderItem::Text("Section 2".to_string()),
        ];

        let text = render_items_to_text(&items);
        assert_eq!(text, "Section 1 | Section 2");
    }

    #[test]
    fn test_render_items_empty() {
        let items = vec![];
        let text = render_items_to_text(&items);
        assert_eq!(text, "");
    }

    #[test]
    fn test_parse_hex_color_valid() {
        let color = parse_hex_color("#ff0000");
        let Color::Srgba(srgba) = color else {
            panic!("Expected Srgba color");
        };
        assert!((srgba.red - 1.0).abs() < 0.01);
        assert!((srgba.green - 0.0).abs() < 0.01);
        assert!((srgba.blue - 0.0).abs() < 0.01);
    }

    #[test]
    fn test_parse_hex_color_no_hash() {
        let color = parse_hex_color("00ff00");
        let Color::Srgba(srgba) = color else {
            panic!("Expected Srgba color");
        };
        assert!((srgba.green - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_parse_named_color() {
        let red = parse_named_color("red");
        let Color::Srgba(srgba) = red else {
            panic!("Expected Srgba color");
        };
        assert!((srgba.red - 1.0).abs() < 0.01);

        let blue = parse_named_color("blue");
        let Color::Srgba(srgba) = blue else {
            panic!("Expected Srgba color");
        };
        assert!((srgba.blue - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_ansi_color_conversion() {
        let color = ansi_color_to_bevy(&AnsiColor::BrightRed);
        let Color::Srgba(srgba) = color else {
            panic!("Expected Srgba color");
        };
        assert!((srgba.red - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_status_bar_state() {
        let mut state = StatusBarState::default();
        assert!(!state.left_dirty);
        assert!(!state.right_dirty);

        state.set_left(vec![RenderItem::Text("Left".to_string())]);
        assert!(state.left_dirty);
        assert_eq!(state.left_items.len(), 1);

        state.set_right(vec![RenderItem::Text("Right".to_string())]);
        assert!(state.right_dirty);
        assert_eq!(state.right_items.len(), 1);

        state.clear_dirty();
        assert!(!state.left_dirty);
        assert!(!state.right_dirty);

        state.clear();
        assert!(state.left_dirty);
        assert!(state.right_dirty);
        assert_eq!(state.left_items.len(), 0);
        assert_eq!(state.right_items.len(), 0);
    }

    #[test]
    fn test_convert_protocol_item_to_render_item() {
        // Test Text conversion
        let item = StatusRenderItem::Text("Test".to_string());
        let result = convert_protocol_item_to_render_item(item);
        assert!(matches!(result, Some(RenderItem::Text(_))));

        // Test Foreground conversion
        let item = StatusRenderItem::Foreground {
            r: 255,
            g: 128,
            b: 64,
        };
        let result = convert_protocol_item_to_render_item(item);
        assert!(matches!(result, Some(RenderItem::Foreground(_))));

        // Test Bold conversion
        let item = StatusRenderItem::Bold;
        let result = convert_protocol_item_to_render_item(item);
        assert!(matches!(result, Some(RenderItem::Bold)));

        // Test Padding conversion
        let item = StatusRenderItem::Padding(3);
        let result = convert_protocol_item_to_render_item(item);
        assert!(matches!(result, Some(RenderItem::Padding(3))));
    }

    #[test]
    fn test_status_update_timer() {
        let timer = StatusUpdateTimer::default();
        assert_eq!(timer.timer.duration().as_millis(), 100);
    }

    #[test]
    fn test_tab_state_default() {
        let tab_state = TabState::default();
        assert_eq!(tab_state.tabs.len(), 3);
        assert_eq!(tab_state.tabs[0], "meta");
        assert_eq!(tab_state.tabs[1], "phage");
        assert_eq!(tab_state.tabs[2], "tolaria");
        assert_eq!(tab_state.active_index, 0);
    }
}
