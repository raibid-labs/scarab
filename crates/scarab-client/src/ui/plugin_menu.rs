//! Plugin Menu Renderer
//!
//! Displays plugin menus as keyboard-navigable overlays when a plugin is activated
//! from the dock. Supports nested submenus, icons, and keyboard shortcuts.

use crate::ipc::{IpcChannel, RemoteMessageEvent};
use crate::ui::command_palette::CommandExecutedEvent;
// use crate::ui::dock::RequestPluginMenuEvent;
use bevy::input::keyboard::KeyCode;
use bevy::prelude::*;
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
        self.active = true;
        self.plugin_name = plugin_name;
        self.current_items = items;
        self.selected_index = 0;
        self.menu_stack.clear();
        self.loading = false;
        self.error = None;
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
    #[allow(dead_code)]
    index: usize,
}

/// Handle show menu events (legacy direct menu passing)
fn handle_show_menu_event(
    mut events: EventReader<ShowPluginMenuEvent>,
    mut menu_state: ResMut<MenuState>,
) {
    for event in events.read() {
        menu_state.open(event.plugin_name.clone(), event.items.clone());
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
                        warn!("Failed to deserialize menu for plugin '{}': {}", plugin_name, e);
                        menu_state.set_error(format!("Invalid menu data: {}", e));
                    }
                }
            }
            DaemonMessage::PluginMenuError {
                plugin_name,
                error,
            } => {
                warn!(
                    "Error loading menu for plugin '{}': {}",
                    plugin_name, error
                );
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
        render_loading_state(&mut commands, &menu_state);
        return;
    }

    // Show error state
    if let Some(error) = &menu_state.error {
        render_error_state(&mut commands, &menu_state, error);
        return;
    }

    // Don't render if no items
    if menu_state.current_items.is_empty() {
        return;
    }

    // Calculate menu dimensions
    let menu_width = 400.0;
    let item_height = 50.0;
    let menu_height = (menu_state.current_items.len() as f32 * item_height).min(500.0);

    // Create menu container
    commands
        .spawn((
            MenuUI,
            Node {
                width: Val::Px(menu_width),
                height: Val::Px(menu_height + 60.0), // Extra space for header
                position_type: PositionType::Absolute,
                left: Val::Px(300.0),
                top: Val::Px(150.0),
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
                    height: Val::Px(menu_height),
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
fn render_loading_state(commands: &mut Commands, menu_state: &MenuState) {
    let menu_width = 400.0;
    let menu_height = 150.0;

    commands
        .spawn((
            MenuUI,
            Node {
                width: Val::Px(menu_width),
                height: Val::Px(menu_height),
                position_type: PositionType::Absolute,
                left: Val::Px(300.0),
                top: Val::Px(150.0),
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
fn render_error_state(commands: &mut Commands, menu_state: &MenuState, error: &str) {
    let menu_width = 400.0;
    let menu_height = 200.0;

    commands
        .spawn((
            MenuUI,
            Node {
                width: Val::Px(menu_width),
                height: Val::Px(menu_height),
                position_type: PositionType::Absolute,
                left: Val::Px(300.0),
                top: Val::Px(150.0),
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
                MenuItem::new("Item 1".to_string(), MenuAction::Command("cmd1".to_string())),
                MenuItem::new("Item 2".to_string(), MenuAction::Command("cmd2".to_string())),
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
