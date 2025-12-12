//! Plugin Dock UI - Visual display of loaded plugins at the bottom of the window
//!
//! The Dock provides a horizontal bar showing all loaded plugins with their:
//! - Emoji icon (if available)
//! - Plugin name
//! - Status indicator (enabled/disabled/error)
//! - Custom color theming (if provided)
//!
//! Integration with scarab-nav makes dock items keyboard-navigable via hints.

use bevy::input::keyboard::KeyCode;
use bevy::prelude::*;
use prost::Message as ProstMessage;
use scarab_nav_protocol::{ElementType, InteractiveElement, UpdateLayout};
use scarab_protocol::{ControlMessage, DaemonMessage, PluginInspectorInfo};
use std::io::Write;
use std::os::unix::net::UnixStream;

use crate::ipc::{IpcChannel, RemoteMessageEvent};
use crate::ui::plugin_menu::ShowPluginMenuEvent;

/// Marker component for the dock container
#[derive(Component)]
pub struct DockContainer;

/// Marker component for individual dock items
#[derive(Component)]
pub struct DockItem {
    pub plugin_name: String,
    pub index: usize,
}

/// Component to store computed bounds for dock items (for nav protocol)
#[derive(Component)]
pub struct DockItemBounds {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

/// Resource tracking the current state of plugins in the dock
#[derive(Resource, Default)]
pub struct DockState {
    pub plugins: Vec<PluginInspectorInfo>,
    pub visible: bool,
    /// Currently selected dock item index (for keyboard navigation)
    pub selected_index: Option<usize>,
}

impl DockState {
    pub fn new() -> Self {
        Self {
            plugins: Vec::new(),
            visible: true,
            selected_index: None,
        }
    }

    pub fn update_plugins(&mut self, new_plugins: Vec<PluginInspectorInfo>) {
        self.plugins = new_plugins;
        // Reset selection if current selection is out of bounds
        if let Some(idx) = self.selected_index {
            if idx >= self.plugins.len() {
                self.selected_index = None;
            }
        }
    }

    pub fn select_next(&mut self) {
        if self.plugins.is_empty() {
            return;
        }
        self.selected_index = Some(match self.selected_index {
            None => 0,
            Some(idx) => (idx + 1).min(self.plugins.len() - 1),
        });
    }

    pub fn select_prev(&mut self) {
        if self.plugins.is_empty() {
            return;
        }
        self.selected_index = Some(match self.selected_index {
            None => 0,
            Some(idx) => idx.saturating_sub(1),
        });
    }

    pub fn activate_selected(&self) -> Option<&str> {
        self.selected_index
            .and_then(|idx| self.plugins.get(idx))
            .map(|p| p.name.as_str())
    }
}

/// Configuration for dock appearance
#[derive(Resource, Clone)]
pub struct DockConfig {
    /// Height of the dock bar in pixels
    pub height: f32,
    /// Padding around dock items
    pub item_padding: f32,
    /// Spacing between dock items
    pub item_spacing: f32,
    /// Default background color for dock
    pub bg_color: Color,
    /// Default text color
    pub text_color: Color,
    /// Font size for plugin names
    pub font_size: f32,
    /// Font size for emoji
    pub emoji_font_size: f32,
    /// Border color for dock items
    pub border_color: Color,
    /// Border width
    pub border_width: f32,
}

impl Default for DockConfig {
    fn default() -> Self {
        use crate::ui::status_bar::DOCK_HEIGHT;
        Self {
            height: DOCK_HEIGHT,
            item_padding: 8.0,
            item_spacing: 4.0,
            bg_color: Color::srgba(0.15, 0.15, 0.18, 0.95),
            text_color: Color::srgb(0.9, 0.9, 0.9),
            font_size: 14.0,
            emoji_font_size: 18.0,
            border_color: Color::srgba(0.3, 0.3, 0.35, 0.8),
            border_width: 1.0,
        }
    }
}

/// Resource for managing the Unix socket connection to the nav plugin
#[derive(Resource)]
pub struct NavConnection {
    stream: Option<UnixStream>,
}

impl Default for NavConnection {
    fn default() -> Self {
        Self { stream: None }
    }
}

impl NavConnection {
    /// Attempt to connect to the nav plugin socket
    fn ensure_connected(&mut self) {
        if self.stream.is_none() {
            match UnixStream::connect("/tmp/scarab-nav.sock") {
                Ok(stream) => {
                    info!("Dock connected to scarab-nav socket");
                    self.stream = Some(stream);
                }
                Err(e) => {
                    debug!("Could not connect to scarab-nav socket: {}", e);
                }
            }
        }
    }

    /// Send an UpdateLayout message to the nav plugin
    fn send_layout(&mut self, layout: UpdateLayout) {
        self.ensure_connected();

        if let Some(ref mut stream) = self.stream {
            let encoded = layout.encode_to_vec();
            let len = encoded.len() as u32;

            // Send length prefix (u32 little-endian) followed by protobuf payload
            if stream.write_all(&len.to_le_bytes()).is_err() || stream.write_all(&encoded).is_err()
            {
                warn!("Failed to send layout to nav plugin, disconnecting");
                self.stream = None;
            }
        }
    }
}

/// System to initialize the dock on startup
fn spawn_dock(
    mut commands: Commands,
    config: Res<DockConfig>,
    state: Res<DockState>,
    existing_dock: Query<Entity, With<DockContainer>>,
) {
    // Only spawn if not already present and visible
    if !existing_dock.is_empty() || !state.visible {
        return;
    }

    info!("Spawning plugin dock");

    // Spawn the dock container
    // Position above the status bar (STATUS_BAR_HEIGHT = 24.0)
    use crate::ui::status_bar::STATUS_BAR_HEIGHT;
    commands
        .spawn((
            DockContainer,
            Node {
                position_type: PositionType::Absolute,
                bottom: Val::Px(STATUS_BAR_HEIGHT), // Position above status bar
                left: Val::Px(0.0),
                width: Val::Percent(100.0),
                height: Val::Px(config.height),
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                padding: UiRect::all(Val::Px(config.item_padding)),
                column_gap: Val::Px(config.item_spacing),
                ..default()
            },
            BackgroundColor(config.bg_color),
            BorderColor(config.border_color),
            ZIndex(999), // Below status bar (which has 1000)
        ))
        .with_children(|parent| {
            // Initially empty - will be populated by update_dock_items
            parent.spawn((
                Text::new("Loading plugins..."),
                TextFont {
                    font_size: config.font_size,
                    ..default()
                },
                TextColor(config.text_color.with_alpha(0.5)),
            ));
        });
}

/// System to update dock items when plugin list changes
fn update_dock_items(
    mut commands: Commands,
    config: Res<DockConfig>,
    state: Res<DockState>,
    dock_query: Query<Entity, With<DockContainer>>,
    item_query: Query<Entity, With<DockItem>>,
) {
    // Skip if dock doesn't exist or state hasn't changed
    if dock_query.is_empty() || !state.is_changed() {
        return;
    }

    let dock_entity = dock_query.single();

    // Despawn all existing dock items
    for item_entity in item_query.iter() {
        commands.entity(item_entity).despawn_recursive();
    }

    // Spawn new items for each plugin
    commands.entity(dock_entity).with_children(|parent| {
        if state.plugins.is_empty() {
            // Show "no plugins" message
            parent.spawn((
                Text::new("No plugins loaded"),
                TextFont {
                    font_size: config.font_size,
                    ..default()
                },
                TextColor(config.text_color.with_alpha(0.5)),
            ));
            return;
        }

        for (index, plugin) in state.plugins.iter().enumerate() {
            let is_selected = state.selected_index == Some(index);
            let base_color = if let Some(color_hex) = &plugin.color {
                parse_hex_color(color_hex).unwrap_or(Color::srgba(0.25, 0.25, 0.28, 0.9))
            } else {
                Color::srgba(0.25, 0.25, 0.28, 0.9)
            };

            // Brighten selected items
            let item_bg_color = if is_selected {
                let srgba = base_color.to_srgba();
                Color::srgba(
                    (srgba.red + 0.15).min(1.0),
                    (srgba.green + 0.15).min(1.0),
                    (srgba.blue + 0.15).min(1.0),
                    srgba.alpha,
                )
            } else {
                base_color
            };

            // Determine status indicator color
            let status_color = if !plugin.enabled {
                Color::srgba(0.5, 0.5, 0.5, 0.8) // Disabled - gray
            } else if plugin.failure_count > 0 {
                Color::srgba(0.9, 0.3, 0.3, 1.0) // Error - red
            } else {
                Color::srgba(0.3, 0.8, 0.4, 1.0) // Active - green
            };

            parent
                .spawn((
                    DockItem {
                        plugin_name: plugin.name.to_string(),
                        index,
                    },
                    DockItemBounds {
                        x: 0.0,
                        y: 0.0,
                        width: 0.0,
                        height: 0.0,
                    },
                    Node {
                        padding: UiRect::all(Val::Px(config.item_padding)),
                        border: UiRect::all(Val::Px(config.border_width)),
                        flex_direction: FlexDirection::Row,
                        align_items: AlignItems::Center,
                        column_gap: Val::Px(6.0),
                        ..default()
                    },
                    BackgroundColor(item_bg_color),
                    BorderColor(config.border_color),
                    BorderRadius::all(Val::Px(4.0)),
                ))
                .with_children(|item| {
                    // Status indicator dot
                    item.spawn((
                        Node {
                            width: Val::Px(8.0),
                            height: Val::Px(8.0),
                            ..default()
                        },
                        BackgroundColor(status_color),
                        BorderRadius::all(Val::Px(4.0)),
                    ));

                    // Emoji if available
                    if let Some(emoji) = &plugin.emoji {
                        item.spawn((
                            Text::new(emoji.to_string()),
                            TextFont {
                                font_size: config.emoji_font_size,
                                ..default()
                            },
                        ));
                    }

                    // Plugin name
                    item.spawn((
                        Text::new(plugin.name.to_string()),
                        TextFont {
                            font_size: config.font_size,
                            ..default()
                        },
                        TextColor(config.text_color),
                    ));

                    // Failure count badge if > 0
                    if plugin.failure_count > 0 {
                        item.spawn((
                            Text::new(format!("({})", plugin.failure_count)),
                            TextFont {
                                font_size: config.font_size * 0.85,
                                ..default()
                            },
                            TextColor(Color::srgba(0.9, 0.3, 0.3, 1.0)),
                        ));
                    }
                });
        }
    });

    info!("Updated dock with {} plugins", state.plugins.len());
}

/// System to compute and cache dock item bounds after layout
fn compute_dock_item_bounds(
    mut query: Query<(&mut DockItemBounds, &GlobalTransform), With<DockItem>>,
    windows: Query<&Window>,
) {
    if let Ok(window) = windows.get_single() {
        let window_height = window.height();
        let window_width = window.width();

        for (mut bounds, transform) in query.iter_mut() {
            let translation = transform.translation();

            // Estimate size from transform scale (rough approximation)
            // In a real implementation, we'd need computed layout info from Bevy
            // For now, use a fixed size estimate based on typical dock item size
            let estimated_width = 120.0; // Typical dock item width
            let estimated_height = 32.0; // Dock item height

            // Convert from Bevy's centered coordinate system to window coordinates
            // Bevy uses center as origin, we need top-left as (0,0)
            bounds.x = translation.x + (window_width / 2.0);
            bounds.y = (window_height / 2.0) - translation.y;
            bounds.width = estimated_width;
            bounds.height = estimated_height;
        }
    }
}

/// System to send dock item layout to nav plugin
fn send_dock_layout_to_nav(
    query: Query<(&DockItem, &DockItemBounds), Changed<DockItemBounds>>,
    mut nav_connection: ResMut<NavConnection>,
) {
    if query.is_empty() {
        return;
    }

    let elements: Vec<InteractiveElement> = query
        .iter()
        .map(|(dock_item, bounds)| InteractiveElement {
            id: format!("dock-{}", dock_item.plugin_name),
            x: bounds.x as u32,
            y: bounds.y as u32,
            width: bounds.width as u32,
            height: bounds.height as u32,
            r#type: ElementType::Button as i32,
            description: format!("Plugin: {}", dock_item.plugin_name),
            key_hint: String::new(),
        })
        .collect();

    if !elements.is_empty() {
        let layout = UpdateLayout {
            window_id: "dock".to_string(),
            elements,
        };

        nav_connection.send_layout(layout);
        debug!(
            "Sent dock layout to nav plugin with {} items",
            query.iter().count()
        );
    }
}

/// System to handle plugin list messages from daemon
fn handle_plugin_list_updates(
    mut events: EventReader<RemoteMessageEvent>,
    mut state: ResMut<DockState>,
) {
    for event in events.read() {
        if let DaemonMessage::PluginList { plugins } = &event.0 {
            debug!(
                "Dock received plugin list update: {} plugins",
                plugins.len()
            );
            state.update_plugins(plugins.clone());
        }
    }
}

/// System to handle plugin status change messages
fn handle_plugin_status_changes(
    mut events: EventReader<RemoteMessageEvent>,
    mut state: ResMut<DockState>,
) {
    for event in events.read() {
        match &event.0 {
            DaemonMessage::PluginStatusChanged { name, enabled } => {
                if let Some(plugin) = state.plugins.iter_mut().find(|p| p.name == name.as_str()) {
                    plugin.enabled = *enabled;
                    debug!("Dock updated plugin '{}' status: enabled={}", name, enabled);
                }
            }
            DaemonMessage::PluginError { name, error } => {
                if let Some(plugin) = state.plugins.iter_mut().find(|p| p.name == name.as_str()) {
                    plugin.failure_count += 1;
                    warn!("Dock registered error for plugin '{}': {}", name, error);
                }
            }
            _ => {}
        }
    }
}

/// System to request initial plugin list on startup
fn request_initial_plugin_list(ipc: Option<Res<IpcChannel>>, mut ran: Local<bool>) {
    if *ran {
        return;
    }
    *ran = true;

    if let Some(ipc) = ipc {
        info!("Dock requesting initial plugin list");
        ipc.send(scarab_protocol::ControlMessage::PluginListRequest);
    }
}

/// Helper function to parse hex color strings (e.g., "#FF5733")
fn parse_hex_color(hex: &str) -> Option<Color> {
    let hex = hex.trim_start_matches('#');

    if hex.len() != 6 {
        return None;
    }

    let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
    let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
    let b = u8::from_str_radix(&hex[4..6], 16).ok()?;

    Some(Color::srgb(
        r as f32 / 255.0,
        g as f32 / 255.0,
        b as f32 / 255.0,
    ))
}

/// Event to request a plugin menu
#[derive(Event)]
pub struct RequestPluginMenuEvent {
    pub plugin_name: String,
}

/// System to handle keyboard navigation of dock items
fn handle_dock_navigation(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut state: ResMut<DockState>,
    mut menu_events: EventWriter<RequestPluginMenuEvent>,
) {
    // Only handle keyboard if dock is visible
    if !state.visible {
        return;
    }

    // Tab to select first/next dock item
    if keyboard.just_pressed(KeyCode::Tab) {
        state.select_next();
    }

    // Shift+Tab to select previous dock item
    if keyboard.just_pressed(KeyCode::ShiftLeft) || keyboard.just_pressed(KeyCode::ShiftRight) {
        if keyboard.pressed(KeyCode::Tab) {
            state.select_prev();
        }
    }

    // Enter to activate selected dock item
    if keyboard.just_pressed(KeyCode::Enter) {
        if let Some(plugin_name) = state.activate_selected() {
            menu_events.send(RequestPluginMenuEvent {
                plugin_name: plugin_name.to_string(),
            });
        }
    }

    // Number keys (1-9) for quick access to dock items
    for (key_code, index) in [
        (KeyCode::Digit1, 0),
        (KeyCode::Digit2, 1),
        (KeyCode::Digit3, 2),
        (KeyCode::Digit4, 3),
        (KeyCode::Digit5, 4),
        (KeyCode::Digit6, 5),
        (KeyCode::Digit7, 6),
        (KeyCode::Digit8, 7),
        (KeyCode::Digit9, 8),
    ] {
        if keyboard.just_pressed(key_code) && keyboard.pressed(KeyCode::AltLeft) {
            if let Some(plugin) = state.plugins.get(index) {
                menu_events.send(RequestPluginMenuEvent {
                    plugin_name: plugin.name.to_string(),
                });
            }
        }
    }
}

/// System to send plugin menu requests to daemon
fn send_menu_request(mut events: EventReader<RequestPluginMenuEvent>, ipc: Res<IpcChannel>) {
    for event in events.read() {
        info!("Requesting menu for plugin: {}", event.plugin_name);
        ipc.send(ControlMessage::PluginMenuRequest {
            plugin_name: event.plugin_name.clone(),
        });
    }
}

/// System to show plugin menus when we receive menu data from daemon
fn handle_plugin_menu_response(
    mut events: EventReader<RemoteMessageEvent>,
    mut menu_events: EventWriter<ShowPluginMenuEvent>,
) {
    for event in events.read() {
        if let DaemonMessage::PluginMenuResponse {
            plugin_name,
            menu_json,
        } = &event.0
        {
            // Deserialize the menu JSON
            if let Ok(items) =
                serde_json::from_str::<Vec<scarab_plugin_api::menu::MenuItem>>(menu_json)
            {
                debug!(
                    "Received menu for plugin '{}' with {} items",
                    plugin_name,
                    items.len()
                );
                menu_events.send(ShowPluginMenuEvent {
                    plugin_name: plugin_name.clone(),
                    items,
                });
            } else {
                warn!(
                    "Failed to deserialize menu JSON for plugin '{}'",
                    plugin_name
                );
            }
        }
    }
}

/// Plugin for the dock system
pub struct DockPlugin;

impl Plugin for DockPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(DockState::new())
            .insert_resource(DockConfig::default())
            .insert_resource(NavConnection::default())
            .add_event::<RequestPluginMenuEvent>()
            .add_systems(Startup, spawn_dock)
            .add_systems(
                Update,
                (
                    request_initial_plugin_list,
                    handle_plugin_list_updates,
                    handle_plugin_status_changes,
                    update_dock_items,
                    handle_dock_navigation,
                    send_menu_request,
                    handle_plugin_menu_response,
                    compute_dock_item_bounds,
                    send_dock_layout_to_nav,
                )
                    .chain(),
            );

        info!("Dock plugin initialized with nav protocol integration");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_hex_color() {
        // Valid colors
        let color = parse_hex_color("#FF5733").unwrap();
        assert!((color.to_srgba().red - 1.0).abs() < 0.01);

        let color = parse_hex_color("#000000").unwrap();
        assert!(color.to_srgba().red < 0.01);

        let color = parse_hex_color("#FFFFFF").unwrap();
        assert!((color.to_srgba().red - 1.0).abs() < 0.01);

        // Colors without # prefix are also valid (lenient parsing)
        let color = parse_hex_color("FF5733").unwrap();
        assert!((color.to_srgba().red - 1.0).abs() < 0.01);

        // Invalid colors
        assert!(parse_hex_color("#FF57").is_none()); // Too short
        assert!(parse_hex_color("FF57").is_none()); // Too short without #
        assert!(parse_hex_color("#GGGGGG").is_none()); // Invalid hex
    }

    #[test]
    fn test_dock_state() {
        let mut state = DockState::new();
        assert!(state.plugins.is_empty());
        assert!(state.visible);

        use scarab_protocol::PluginVerificationStatus;

        let plugins = vec![PluginInspectorInfo {
            name: "test-plugin".into(),
            version: "1.0.0".into(),
            description: "Test plugin".into(),
            author: "Test Author".into(),
            homepage: None,
            api_version: "1.0.0".into(),
            min_scarab_version: "0.1.0".into(),
            enabled: true,
            failure_count: 0,
            emoji: Some("🦀".into()),
            color: Some("#FF5733".into()),
            verification: PluginVerificationStatus::Unverified {
                warning: "Test plugin".into(),
            },
        }];

        state.update_plugins(plugins.clone());
        assert_eq!(state.plugins.len(), 1);
        assert_eq!(state.plugins[0].name, "test-plugin");
    }
}
