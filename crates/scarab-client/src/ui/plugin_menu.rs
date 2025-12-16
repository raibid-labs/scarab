//! Plugin Menu Renderer
//!
//! Displays plugin menus as keyboard-navigable overlays when a plugin is activated
//! from the dock. Supports nested submenus, icons, and keyboard shortcuts.

use crate::ipc::{IpcChannel, RemoteMessageEvent};
use crate::ui::command_palette::CommandExecutedEvent;
use crate::ui::dock::NavConnection;
// use crate::ui::dock::RequestPluginMenuEvent;
use bevy::input::keyboard::KeyCode;
use bevy::prelude::*;
use scarab_nav_protocol::{ElementType, InteractiveElement, UpdateLayout};
use scarab_plugin_api::menu::{MenuAction, MenuItem};
use scarab_protocol::{ControlMessage, DaemonMessage, MenuActionType};

/// Plugin for rendering plugin menus
pub struct PluginMenuPlugin;

impl Plugin for PluginMenuPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MenuState>()
            .add_event::<ShowPluginMenuEvent>()
            .add_event::<MenuActionEvent>()
            .add_systems(
                Update,
                (
                    // handle_request_plugin_menu,
                    handle_show_menu_event,
                    handle_daemon_menu_response,
                    handle_menu_input_system,
                    render_menu_system,
                    compute_menu_item_bounds,
                    send_menu_layout_to_nav,
                    unregister_menu_on_close,
                    execute_menu_action_system,
                )
                    .chain(),
            );
    }
}

// /// Handle dock requests to open plugin menus
// fn handle_request_plugin_menu(
//     mut events: EventReader<RequestPluginMenuEvent>,
//     mut menu_state: ResMut<MenuState>,
// ) {
//     for event in events.read() {
//         info!("Starting menu load for plugin: {}", event.plugin_name);
//         menu_state.start_loading(event.plugin_name.clone());
//     }
// }

/// Event to show a plugin menu
#[derive(Event)]
pub struct ShowPluginMenuEvent {
    pub plugin_name: String,
    pub items: Vec<MenuItem>,
    /// Optional position to show the menu at (defaults to center if None)
    pub position: Option<MenuPosition>,
}

/// Position for the menu
#[derive(Debug, Clone, Copy)]
pub struct MenuPosition {
    pub x: f32,
    pub y: f32,
}

/// Event fired when a menu action is triggered
#[derive(Event)]
pub struct MenuActionEvent {
    pub action: MenuAction,
}

/// Resource tracking the current menu state
#[derive(Resource, Default)]
pub struct MenuState {
    /// Whether a menu is currently displayed
    pub active: bool,
    /// Name of the plugin this menu belongs to
    pub plugin_name: String,
    /// Currently visible menu items (changes with submenus)
    pub current_items: Vec<MenuItem>,
    /// Index of the selected item
    pub selected_index: usize,
    /// Stack of previous menus (for submenu navigation)
    pub menu_stack: Vec<Vec<MenuItem>>,
    /// Loading state while waiting for menu from daemon
    pub loading: bool,
    /// Error message if menu loading failed
    pub error: Option<String>,
    /// Position to display the menu (None = center)
    pub position: Option<MenuPosition>,
}

impl MenuState {
    /// Start loading a menu
    pub fn start_loading(&mut self, plugin_name: String) {
        self.active = true;
        self.plugin_name = plugin_name;
        self.loading = true;
        self.error = None;
        self.current_items.clear();
        self.menu_stack.clear();
        self.selected_index = 0;
    }

    /// Open a new menu
    pub fn open(&mut self, plugin_name: String, items: Vec<MenuItem>) {
        self.open_with_position(plugin_name, items, None);
    }

    /// Open a new menu with a specific position
    pub fn open_with_position(&mut self, plugin_name: String, items: Vec<MenuItem>, position: Option<MenuPosition>) {
        self.active = true;
        self.plugin_name = plugin_name;
        self.current_items = items;
        self.selected_index = 0;
        self.menu_stack.clear();
        self.loading = false;
        self.error = None;
        self.position = position;
    }

    /// Set error state
    pub fn set_error(&mut self, error: String) {
        self.loading = false;
        self.error = Some(error);
    }

    /// Navigate into a submenu
    pub fn enter_submenu(&mut self, items: Vec<MenuItem>) {
        // Push current menu onto stack
        self.menu_stack.push(self.current_items.clone());
        self.current_items = items;
        self.selected_index = 0;
    }

    /// Navigate back to previous menu
    pub fn go_back(&mut self) -> bool {
        if let Some(previous_menu) = self.menu_stack.pop() {
            self.current_items = previous_menu;
            self.selected_index = 0;
            true
        } else {
            false
        }
    }

    /// Close the menu completely
    pub fn close(&mut self) {
        self.active = false;
        self.current_items.clear();
        self.menu_stack.clear();
        self.selected_index = 0;
        self.plugin_name.clear();
        self.loading = false;
        self.error = None;
    }

    /// Get the currently selected item
    pub fn selected_item(&self) -> Option<&MenuItem> {
        self.current_items.get(self.selected_index)
    }
}

/// Component for menu UI elements
#[derive(Component)]
struct MenuUI;

/// Component for menu items
#[derive(Component)]
struct MenuItemComponent {
    index: usize,
}

/// Component to store computed bounds for menu items (for nav protocol)
#[derive(Component)]
struct MenuItemBounds {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

/// Handle show menu events (legacy direct menu passing)
fn handle_show_menu_event(
    mut events: EventReader<ShowPluginMenuEvent>,
    mut menu_state: ResMut<MenuState>,
) {
    for event in events.read() {
        menu_state.open_with_position(event.plugin_name.clone(), event.items.clone(), event.position);
    }
}

/// Handle menu responses from daemon
fn handle_daemon_menu_response(
    mut events: EventReader<RemoteMessageEvent>,
    mut menu_state: ResMut<MenuState>,
) {
    for event in events.read() {
        match &event.0 {
            DaemonMessage::PluginMenuResponse {
                plugin_name,
                menu_json,
            } => {
                // Deserialize the menu items from JSON
                match serde_json::from_str::<Vec<MenuItem>>(menu_json) {
                    Ok(items) => {
                        info!(
                            "Received menu for plugin '{}' with {} items",
                            plugin_name,
                            items.len()
                        );
                        menu_state.open(plugin_name.clone(), items);
                    }
                    Err(e) => {
                        warn!(
                            "Failed to deserialize menu for plugin '{}': {}",
                            plugin_name, e
                        );
                        menu_state.set_error(format!("Invalid menu data: {}", e));
                    }
                }
            }
            DaemonMessage::PluginMenuError { plugin_name, error } => {
                warn!("Error loading menu for plugin '{}': {}", plugin_name, error);
                menu_state.set_error(error.clone());
            }
            _ => {}
        }
    }
}

/// Handle keyboard input for menu navigation
fn handle_menu_input_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut menu_state: ResMut<MenuState>,
    mut action_events: EventWriter<MenuActionEvent>,
) {
    if !menu_state.active {
        return;
    }

    // Close menu with Escape
    if keyboard.just_pressed(KeyCode::Escape) {
        // Try to go back to previous menu, or close if at root
        if !menu_state.go_back() {
            menu_state.close();
        }
        return;
    }

    // Navigate with arrow keys
    if keyboard.just_pressed(KeyCode::ArrowDown) {
        if menu_state.selected_index < menu_state.current_items.len().saturating_sub(1) {
            menu_state.selected_index += 1;
        }
    }

    if keyboard.just_pressed(KeyCode::ArrowUp) {
        menu_state.selected_index = menu_state.selected_index.saturating_sub(1);
    }

    // Select item with Enter or Space
    if keyboard.just_pressed(KeyCode::Enter) || keyboard.just_pressed(KeyCode::Space) {
        if let Some(item) = menu_state.selected_item() {
            let action = item.action.clone();

            match &action {
                MenuAction::SubMenu(items) => {
                    // Enter the submenu
                    menu_state.enter_submenu(items.clone());
                }
                _ => {
                    // Execute the action and close menu
                    action_events.send(MenuActionEvent { action });
                    menu_state.close();
                }
            }
        }
    }
}

/// Render the menu UI
fn render_menu_system(
    mut commands: Commands,
    menu_state: Res<MenuState>,
    existing_ui: Query<Entity, With<MenuUI>>,
    windows: Query<&Window>,
) {
    // Remove existing menu UI
    for entity in existing_ui.iter() {
        commands.entity(entity).despawn_recursive();
    }

    if !menu_state.active {
        return;
    }

    // Show loading state
    if menu_state.loading {
        render_loading_state(&mut commands, &menu_state, &windows);
        return;
    }

    // Show error state
    if let Some(error) = &menu_state.error {
        render_error_state(&mut commands, &menu_state, error, &windows);
        return;
    }

    // Don't render if no items
    if menu_state.current_items.is_empty() {
        return;
    }

    // Calculate menu dimensions
    let menu_width = 400.0;
    let item_height = 50.0;
    let header_height = 50.0;
    let footer_height = 30.0;
    let menu_content_height = (menu_state.current_items.len() as f32 * item_height).min(500.0);
    let total_menu_height = menu_content_height + header_height + footer_height;

    // Get window dimensions
    let (window_width, window_height) = if let Ok(window) = windows.get_single() {
        (window.width(), window.height())
    } else {
        (1920.0, 1080.0) // Default fallback
    };

    // Calculate position (from dock item or default center)
    let (mut menu_x, mut menu_y) = if let Some(pos) = menu_state.position {
        // Position menu above the dock item
        // The dock is at the bottom, so we position the menu growing upward
        use crate::ui::status_bar::{STATUS_BAR_HEIGHT, DOCK_HEIGHT};
        let _dock_bottom = STATUS_BAR_HEIGHT;
        let dock_top = STATUS_BAR_HEIGHT + DOCK_HEIGHT;

        // Position menu right above the dock
        let menu_bottom = dock_top;
        let menu_top = menu_bottom + total_menu_height;

        // X position centered on dock item, but ensure it stays in viewport
        let menu_left = pos.x - (menu_width / 2.0);

        (menu_left, window_height - menu_top)
    } else {
        // Default center position
        (
            (window_width - menu_width) / 2.0,
            (window_height - total_menu_height) / 2.0,
        )
    };

    // Ensure menu stays within viewport bounds
    menu_x = menu_x.max(10.0).min(window_width - menu_width - 10.0);
    menu_y = menu_y.max(10.0).min(window_height - total_menu_height - 10.0);

    // Create menu container
    commands
        .spawn((
            MenuUI,
            Node {
                width: Val::Px(menu_width),
                height: Val::Px(total_menu_height),
                position_type: PositionType::Absolute,
                left: Val::Px(menu_x),
                top: Val::Px(menu_y),
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(0.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.1, 0.1, 0.1, 0.96)),
            BorderColor(Color::srgba(0.4, 0.4, 0.5, 0.8)),
        ))
        .with_children(|parent| {
            // Menu header
            parent
                .spawn((
                    Node {
                        width: Val::Percent(100.0),
                        height: Val::Px(50.0),
                        padding: UiRect::all(Val::Px(15.0)),
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.15, 0.15, 0.2, 1.0)),
                ))
                .with_children(|header| {
                    // Plugin name and breadcrumb
                    let breadcrumb = if menu_state.menu_stack.is_empty() {
                        menu_state.plugin_name.clone()
                    } else {
                        format!("{} > Menu", menu_state.plugin_name)
                    };

                    header.spawn((
                        Text::new(&breadcrumb),
                        TextFont {
                            font_size: 18.0,
                            ..default()
                        },
                        TextColor(Color::srgba(0.9, 0.9, 1.0, 1.0)),
                    ));
                });

            // Scrollable menu items container
            parent
                .spawn((Node {
                    width: Val::Percent(100.0),
                    height: Val::Px(menu_content_height),
                    flex_direction: FlexDirection::Column,
                    overflow: Overflow::clip_y(),
                    ..default()
                },))
                .with_children(|items_container| {
                    // Render each menu item
                    for (index, item) in menu_state.current_items.iter().enumerate() {
                        let is_selected = index == menu_state.selected_index;
                        let bg_color = if is_selected {
                            Color::srgba(0.3, 0.35, 0.45, 0.9)
                        } else {
                            Color::srgba(0.12, 0.12, 0.15, 0.6)
                        };

                        items_container
                            .spawn((
                                MenuItemComponent { index },
                                MenuItemBounds {
                                    x: 0.0,
                                    y: 0.0,
                                    width: 0.0,
                                    height: 0.0,
                                },
                                Node {
                                    width: Val::Percent(100.0),
                                    height: Val::Px(item_height),
                                    padding: UiRect::all(Val::Px(12.0)),
                                    margin: UiRect::bottom(Val::Px(1.0)),
                                    align_items: AlignItems::Center,
                                    justify_content: JustifyContent::SpaceBetween,
                                    ..default()
                                },
                                BackgroundColor(bg_color),
                            ))
                            .with_children(|item_row| {
                                // Left side: icon + label
                                item_row
                                    .spawn(Node {
                                        flex_direction: FlexDirection::Row,
                                        align_items: AlignItems::Center,
                                        column_gap: Val::Px(10.0),
                                        ..default()
                                    })
                                    .with_children(|left| {
                                        // Icon
                                        if let Some(icon) = &item.icon {
                                            left.spawn((
                                                Text::new(icon),
                                                TextFont {
                                                    font_size: 20.0,
                                                    ..default()
                                                },
                                                TextColor(Color::WHITE),
                                            ));
                                        }

                                        // Label
                                        left.spawn((
                                            Text::new(&item.label),
                                            TextFont {
                                                font_size: 16.0,
                                                ..default()
                                            },
                                            TextColor(Color::WHITE),
                                        ));
                                    });

                                // Right side: shortcut or submenu indicator
                                item_row
                                    .spawn(Node {
                                        flex_direction: FlexDirection::Row,
                                        align_items: AlignItems::Center,
                                        ..default()
                                    })
                                    .with_children(|right| {
                                        if let Some(shortcut) = &item.shortcut {
                                            right.spawn((
                                                Text::new(shortcut),
                                                TextFont {
                                                    font_size: 13.0,
                                                    ..default()
                                                },
                                                TextColor(Color::srgba(0.6, 0.6, 0.7, 1.0)),
                                            ));
                                        }

                                        // Show ">" for submenus
                                        if item.action.is_submenu() {
                                            right.spawn((
                                                Text::new("  >"),
                                                TextFont {
                                                    font_size: 16.0,
                                                    ..default()
                                                },
                                                TextColor(Color::srgba(0.7, 0.7, 0.8, 1.0)),
                                            ));
                                        }
                                    });
                            });
                    }
                });

            // Footer with hints
            parent
                .spawn((
                    Node {
                        width: Val::Percent(100.0),
                        height: Val::Px(30.0),
                        padding: UiRect::all(Val::Px(8.0)),
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.08, 0.08, 0.1, 1.0)),
                ))
                .with_children(|footer| {
                    let hint_text = if menu_state.menu_stack.is_empty() {
                        "↑↓ Navigate  •  Enter Select  •  Esc Close"
                    } else {
                        "↑↓ Navigate  •  Enter Select  •  Esc Back"
                    };

                    footer.spawn((
                        Text::new(hint_text),
                        TextFont {
                            font_size: 12.0,
                            ..default()
                        },
                        TextColor(Color::srgba(0.5, 0.5, 0.6, 1.0)),
                    ));
                });
        });
}

/// Render loading state while waiting for menu from daemon
fn render_loading_state(commands: &mut Commands, menu_state: &MenuState, windows: &Query<&Window>) {
    let menu_width = 400.0;
    let menu_height = 150.0;

    // Get window dimensions for centering
    let (window_width, window_height) = if let Ok(window) = windows.get_single() {
        (window.width(), window.height())
    } else {
        (1920.0, 1080.0)
    };

    let menu_x = (window_width - menu_width) / 2.0;
    let menu_y = (window_height - menu_height) / 2.0;

    commands
        .spawn((
            MenuUI,
            Node {
                width: Val::Px(menu_width),
                height: Val::Px(menu_height),
                position_type: PositionType::Absolute,
                left: Val::Px(menu_x),
                top: Val::Px(menu_y),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                padding: UiRect::all(Val::Px(20.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.1, 0.1, 0.1, 0.96)),
            BorderColor(Color::srgba(0.4, 0.4, 0.5, 0.8)),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new(format!("Loading menu for {}...", menu_state.plugin_name)),
                TextFont {
                    font_size: 16.0,
                    ..default()
                },
                TextColor(Color::srgba(0.9, 0.9, 1.0, 1.0)),
            ));

            parent.spawn((
                Text::new("Please wait"),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(Color::srgba(0.6, 0.6, 0.7, 1.0)),
            ));
        });
}

/// Render error state when menu loading fails
fn render_error_state(commands: &mut Commands, menu_state: &MenuState, error: &str, windows: &Query<&Window>) {
    let menu_width = 400.0;
    let menu_height = 200.0;

    // Get window dimensions for centering
    let (window_width, window_height) = if let Ok(window) = windows.get_single() {
        (window.width(), window.height())
    } else {
        (1920.0, 1080.0)
    };

    let menu_x = (window_width - menu_width) / 2.0;
    let menu_y = (window_height - menu_height) / 2.0;

    commands
        .spawn((
            MenuUI,
            Node {
                width: Val::Px(menu_width),
                height: Val::Px(menu_height),
                position_type: PositionType::Absolute,
                left: Val::Px(menu_x),
                top: Val::Px(menu_y),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                padding: UiRect::all(Val::Px(20.0)),
                row_gap: Val::Px(10.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.15, 0.1, 0.1, 0.96)),
            BorderColor(Color::srgba(0.8, 0.3, 0.3, 0.8)),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new(format!("Error loading menu for {}", menu_state.plugin_name)),
                TextFont {
                    font_size: 16.0,
                    ..default()
                },
                TextColor(Color::srgba(1.0, 0.5, 0.5, 1.0)),
            ));

            parent.spawn((
                Text::new(error),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(Color::srgba(0.9, 0.7, 0.7, 1.0)),
            ));

            parent.spawn((
                Text::new("Press Esc to close"),
                TextFont {
                    font_size: 12.0,
                    ..default()
                },
                TextColor(Color::srgba(0.6, 0.6, 0.7, 1.0)),
            ));
        });
}

/// System to compute and cache menu item bounds after layout
fn compute_menu_item_bounds(
    mut query: Query<(&mut MenuItemBounds, &GlobalTransform), With<MenuItemComponent>>,
    windows: Query<&Window>,
) {
    if let Ok(window) = windows.get_single() {
        let window_height = window.height();
        let window_width = window.width();

        for (mut bounds, transform) in query.iter_mut() {
            let translation = transform.translation();

            // Estimate size from transform (rough approximation)
            let estimated_width = 400.0; // Menu width
            let estimated_height = 50.0; // Menu item height

            // Convert from Bevy's centered coordinate system to window coordinates
            // Bevy uses center as origin, we need top-left as (0,0)
            bounds.x = translation.x + (window_width / 2.0);
            bounds.y = (window_height / 2.0) - translation.y;
            bounds.width = estimated_width;
            bounds.height = estimated_height;
        }
    }
}

/// System to send menu item layout to nav plugin
fn send_menu_layout_to_nav(
    query: Query<(&MenuItemComponent, &MenuItemBounds)>,
    menu_state: Res<MenuState>,
    nav_connection: Option<ResMut<NavConnection>>,
) {
    // NavConnection might not be available if DockPlugin hasn't been added
    let Some(mut nav_connection) = nav_connection else {
        return;
    };

    // Only send if menu is active
    if !menu_state.active || menu_state.loading || menu_state.error.is_some() {
        return;
    }

    if query.is_empty() {
        return;
    }

    let elements: Vec<InteractiveElement> = query
        .iter()
        .filter_map(|(item, bounds)| {
            // Get the menu item label from menu_state
            menu_state.current_items.get(item.index).map(|menu_item| {
                InteractiveElement {
                    id: format!("menu-item-{}", item.index),
                    x: bounds.x as u32,
                    y: bounds.y as u32,
                    width: bounds.width as u32,
                    height: bounds.height as u32,
                    r#type: ElementType::ListItem as i32,
                    description: menu_item.label.clone(),
                    key_hint: String::new(),
                }
            })
        })
        .collect();

    if !elements.is_empty() {
        let layout = UpdateLayout {
            window_id: "plugin-menu".to_string(),
            elements,
        };

        nav_connection.send_layout(layout);
        debug!(
            "Sent menu layout to nav plugin with {} items",
            query.iter().count()
        );
    }
}

/// System to unregister menu elements from nav when menu closes
fn unregister_menu_on_close(
    menu_state: Res<MenuState>,
    nav_connection: Option<ResMut<NavConnection>>,
    mut last_active: Local<bool>,
) {
    // Detect menu close transition
    if *last_active && !menu_state.active {
        // NavConnection might not be available if DockPlugin hasn't been added
        if let Some(mut nav_connection) = nav_connection {
            // Send empty layout to unregister all menu items
            let layout = UpdateLayout {
                window_id: "plugin-menu".to_string(),
                elements: vec![],
            };

            nav_connection.send_layout(layout);
            debug!("Unregistered menu items from nav plugin");
        }
    }

    *last_active = menu_state.active;
}

/// Execute menu actions
fn execute_menu_action_system(
    mut events: EventReader<MenuActionEvent>,
    ipc: Res<IpcChannel>,
    _command_events: EventWriter<CommandExecutedEvent>,
    menu_state: Res<MenuState>,
) {
    for event in events.read() {
        let plugin_name = menu_state.plugin_name.clone();

        match &event.action {
            MenuAction::Command(command_str) => {
                info!(
                    "Executing command action for plugin '{}': {}",
                    plugin_name, command_str
                );
                // Send to daemon as a plugin menu execute command
                ipc.send(ControlMessage::PluginMenuExecute {
                    plugin_name,
                    action: MenuActionType::Command {
                        command: command_str.clone(),
                    },
                });
            }
            MenuAction::Remote(id) => {
                info!(
                    "Executing remote action for plugin '{}': {}",
                    plugin_name, id
                );
                // Send remote action to daemon
                ipc.send(ControlMessage::PluginMenuExecute {
                    plugin_name,
                    action: MenuActionType::Remote { id: id.clone() },
                });
            }
            MenuAction::SubMenu(_) => {
                // Submenus are handled in the input system
                // This shouldn't be reached, but we'll log it just in case
                warn!("SubMenu action reached execute system - should be handled in input");
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_menu_state_navigation() {
        let mut state = MenuState::default();

        // Open a menu
        state.open(
            "test-plugin".to_string(),
            vec![
                MenuItem::new(
                    "Item 1".to_string(),
                    MenuAction::Command("cmd1".to_string()),
                ),
                MenuItem::new(
                    "Item 2".to_string(),
                    MenuAction::Command("cmd2".to_string()),
                ),
            ],
        );

        assert!(state.active);
        assert_eq!(state.current_items.len(), 2);
        assert_eq!(state.selected_index, 0);

        // Enter a submenu
        state.enter_submenu(vec![MenuItem::new(
            "Sub Item".to_string(),
            MenuAction::Command("sub".to_string()),
        )]);

        assert_eq!(state.menu_stack.len(), 1);
        assert_eq!(state.current_items.len(), 1);

        // Go back
        assert!(state.go_back());
        assert_eq!(state.current_items.len(), 2);
        assert_eq!(state.menu_stack.len(), 0);

        // Can't go back further
        assert!(!state.go_back());
    }

    #[test]
    fn test_menu_state_close() {
        let mut state = MenuState::default();

        state.open("test".to_string(), vec![]);
        state.enter_submenu(vec![]);

        state.close();

        assert!(!state.active);
        assert_eq!(state.current_items.len(), 0);
        assert_eq!(state.menu_stack.len(), 0);
    }
}
