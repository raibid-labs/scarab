//! Context Menu System for Scarab Terminal
//!
//! Provides right-click context menu functionality using the Ratatui bridge.
//! This module integrates with:
//! - scarab-mouse for context menu data structures and position tracking
//! - scarab-clipboard for copy/paste operations
//! - Plugin system for custom menu items
//!
//! # Architecture
//!
//! 1. Mouse Event Detection: Right-click triggers menu spawn
//! 2. Context Detection: Analyzes cursor position for URLs, file paths, selections
//! 3. Menu Construction: Builds appropriate menu items based on context
//! 4. Ratatui Rendering: Displays menu as an overlay using RatatuiSurface
//! 5. Input Handling: Keyboard/mouse navigation and action dispatch
//! 6. Action Execution: Routes actions to appropriate handlers (copy, paste, open URL, etc.)
//!
//! # Usage
//!
//! Add `ContextMenuPlugin` to your Bevy app:
//!
//! ```ignore
//! app.add_plugins(ContextMenuPlugin);
//! ```
//!
//! The plugin automatically handles:
//! - Right-click detection
//! - Menu positioning (with edge detection)
//! - Keyboard navigation (Up/Down/Enter/Esc)
//! - Mouse hover and click
//! - Action dispatch

mod actions;
mod overlay;
mod plugin_items;

pub use actions::{ContextMenuAction, DispatchContextMenuAction, dispatch_action};
pub use overlay::{render_context_menu, ContextMenuOverlay};
pub use plugin_items::get_plugin_menu_items;

use bevy::prelude::*;
use scarab_mouse::context_menu::ContextMenu;
use scarab_mouse::types::Position;

use crate::ratatui_bridge::{RatatuiSurface, SurfaceFocus, SurfaceInputEvent};

/// Marker component for the context menu surface
#[derive(Component)]
pub struct ContextMenuSurface;

/// Resource tracking the current context menu state
#[derive(Resource, Default)]
pub struct ContextMenuState {
    /// Current menu being displayed (None = hidden)
    pub menu: Option<ContextMenu>,
    /// Whether the menu surface entity has been spawned
    pub surface_spawned: bool,
}

impl ContextMenuState {
    /// Show a context menu at the specified position
    pub fn show(&mut self, menu: ContextMenu) {
        self.menu = Some(menu);
    }

    /// Hide the context menu
    pub fn hide(&mut self) {
        self.menu = None;
    }

    /// Check if menu is currently visible
    pub fn is_visible(&self) -> bool {
        self.menu.is_some()
    }

    /// Get the current menu
    pub fn get_menu(&self) -> Option<&ContextMenu> {
        self.menu.as_ref()
    }

    /// Get mutable access to the current menu
    pub fn get_menu_mut(&mut self) -> Option<&mut ContextMenu> {
        self.menu.as_mut()
    }
}

/// Event fired when the user requests a context menu (e.g., right-click)
#[derive(Event)]
pub struct ShowContextMenuEvent {
    /// Position in grid coordinates where the menu should appear
    pub position: Position,
    /// Optional URL detected at the cursor position
    pub url: Option<String>,
    /// Optional file path detected at the cursor position
    pub file_path: Option<String>,
    /// Whether there is text selected
    pub has_selection: bool,
}

/// Event fired when a context menu item is selected
#[derive(Event)]
pub struct ContextMenuItemSelected {
    /// ID of the selected menu item
    pub item_id: String,
    /// The associated data (e.g., URL or file path)
    pub data: Option<String>,
}

/// System to spawn context menu on right-click
pub fn detect_context_menu_request(
    mouse_button: Res<ButtonInput<bevy::input::mouse::MouseButton>>,
    windows: Query<&Window>,
    metrics: Res<scarab_protocol::TerminalMetrics>,
    mut events: EventWriter<ShowContextMenuEvent>,
) {
    use bevy::input::mouse::MouseButton;

    // Check for right-click
    if !mouse_button.just_pressed(MouseButton::Right) {
        return;
    }

    let Ok(window) = windows.get_single() else {
        return;
    };

    let Some(cursor_pos) = window.cursor_position() else {
        return;
    };

    // Convert screen coordinates to grid coordinates
    let (col, row) = metrics.screen_to_grid(cursor_pos.x, cursor_pos.y);

    // TODO: Detect URLs, file paths, and selection at cursor position
    // For now, we'll emit a basic event
    events.send(ShowContextMenuEvent {
        position: Position::new(col, row),
        url: None,
        file_path: None,
        has_selection: false, // TODO: Check actual selection state
    });
}

/// System to create and show context menu based on context
pub fn handle_show_context_menu(
    mut events: EventReader<ShowContextMenuEvent>,
    mut state: ResMut<ContextMenuState>,
    mut focus: ResMut<SurfaceFocus>,
    query: Query<Entity, With<ContextMenuSurface>>,
    metrics: Res<scarab_protocol::TerminalMetrics>,
) {
    for event in events.read() {
        // Determine which menu to show based on context
        let menu = if let Some(url) = &event.url {
            ContextMenu::url_menu(event.position, url.clone())
        } else if let Some(path) = &event.file_path {
            ContextMenu::file_menu(event.position, path.clone())
        } else {
            ContextMenu::standard(event.position, event.has_selection)
        };

        // Adjust position if menu would go off-screen
        let mut adjusted_menu = menu;
        let menu_width = 30u16; // Approximate menu width
        let menu_height = adjusted_menu.items.len() as u16 + 2; // Items + borders

        // Adjust horizontal position
        if adjusted_menu.position.x + menu_width > metrics.columns {
            adjusted_menu.position.x = metrics.columns.saturating_sub(menu_width);
        }

        // Adjust vertical position
        if adjusted_menu.position.y + menu_height > metrics.rows {
            adjusted_menu.position.y = metrics.rows.saturating_sub(menu_height);
        }

        state.show(adjusted_menu);

        // Set focus to context menu
        if let Ok(entity) = query.get_single() {
            focus.push(entity);
        }

        info!("Context menu shown at ({}, {})", event.position.x, event.position.y);
    }
}

/// System to handle input for context menu
pub fn handle_context_menu_input(
    mut state: ResMut<ContextMenuState>,
    mut events: EventReader<SurfaceInputEvent>,
    mut selection_events: EventWriter<ContextMenuItemSelected>,
    mut focus: ResMut<SurfaceFocus>,
    query: Query<Entity, With<ContextMenuSurface>>,
) {
    let Ok(menu_entity) = query.get_single() else {
        return;
    };

    if !state.is_visible() {
        return;
    }

    for event in events.read() {
        if event.surface != menu_entity {
            continue;
        }

        use ratatui::crossterm::event::{Event, KeyCode, MouseButton, MouseEventKind};

        match &event.event {
            Event::Key(key) => match key.code {
                KeyCode::Up => {
                    if let Some(menu) = state.get_menu_mut() {
                        menu.select_prev();
                    }
                }
                KeyCode::Down => {
                    if let Some(menu) = state.get_menu_mut() {
                        menu.select_next();
                    }
                }
                KeyCode::Enter => {
                    if let Some(menu) = state.get_menu() {
                        if let Some(item) = menu.selected_item() {
                            if item.enabled {
                                selection_events.send(ContextMenuItemSelected {
                                    item_id: item.id.clone(),
                                    data: None, // TODO: Pass URL/path data
                                });

                                state.hide();
                                focus.remove(menu_entity);
                            }
                        }
                    }
                }
                KeyCode::Esc => {
                    state.hide();
                    focus.remove(menu_entity);
                }
                _ => {}
            },
            Event::Mouse(mouse) => match mouse.kind {
                MouseEventKind::Down(MouseButton::Left) => {
                    // Find item at mouse position
                    if let Some(menu) = state.get_menu() {
                        // Calculate which item was clicked (row - 1 for border)
                        let item_index = mouse.row.saturating_sub(1) as usize;

                        if item_index < menu.items.len() {
                            let item = &menu.items[item_index];
                            if item.enabled && !item.separator {
                                selection_events.send(ContextMenuItemSelected {
                                    item_id: item.id.clone(),
                                    data: None,
                                });

                                state.hide();
                                focus.remove(menu_entity);
                            }
                        }
                    }
                }
                MouseEventKind::Moved => {
                    // Update selection based on mouse position
                    if let Some(menu) = state.get_menu_mut() {
                        let item_index = mouse.row.saturating_sub(1) as usize;
                        if item_index < menu.items.len() && !menu.items[item_index].separator {
                            menu.selected_index = item_index;
                        }
                    }
                }
                _ => {}
            },
            _ => {}
        }
    }
}

/// System to spawn context menu surface (runs once at startup)
pub fn spawn_context_menu_surface(
    mut commands: Commands,
    existing: Query<Entity, With<ContextMenuSurface>>,
) {
    if !existing.is_empty() {
        return;
    }

    // Spawn hidden initially
    commands.spawn((
        ContextMenuSurface,
        RatatuiSurface::new(0, 0, 40, 15)
            .with_z_index(300.0) // High z-index for context menu
            .hidden(),
    ));

    info!("Context menu surface spawned (hidden)");
}

/// System to update context menu surface visibility and position
pub fn update_context_menu_surface(
    state: Res<ContextMenuState>,
    mut surfaces: Query<&mut RatatuiSurface, With<ContextMenuSurface>>,
) {
    let Ok(mut surface) = surfaces.get_single_mut() else {
        return;
    };

    if let Some(menu) = state.get_menu() {
        // Calculate menu size
        let width = 40u16; // Fixed width for now
        let height = (menu.items.len() as u16 + 2).min(20); // Items + borders, max 20

        // Update surface position and size
        surface.set_position(menu.position.x, menu.position.y);
        surface.set_size(width, height);
        surface.show();
    } else {
        surface.hide();
    }
}

/// Plugin providing context menu functionality
pub struct ContextMenuPlugin;

impl Plugin for ContextMenuPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ContextMenuState>()
            .add_event::<ShowContextMenuEvent>()
            .add_event::<ContextMenuItemSelected>()
            .add_event::<DispatchContextMenuAction>()
            .add_systems(Startup, spawn_context_menu_surface)
            .add_systems(
                Update,
                (
                    detect_context_menu_request,
                    handle_show_context_menu,
                    update_context_menu_surface,
                    handle_context_menu_input,
                    overlay::render_context_menu,
                    actions::handle_context_menu_actions,
                )
                    .chain(),
            );

        info!("ContextMenuPlugin initialized");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_context_menu_state_creation() {
        let state = ContextMenuState::default();
        assert!(!state.is_visible());
        assert!(state.get_menu().is_none());
    }

    #[test]
    fn test_show_hide_menu() {
        let mut state = ContextMenuState::default();

        let menu = ContextMenu::standard(Position::new(10, 10), true);
        state.show(menu);

        assert!(state.is_visible());
        assert!(state.get_menu().is_some());

        state.hide();
        assert!(!state.is_visible());
        assert!(state.get_menu().is_none());
    }

    #[test]
    fn test_menu_positioning_adjustment() {
        // Test that menus are adjusted when they would go off-screen
        let position = Position::new(190, 90); // Near edge of typical 200x100 grid

        let menu = ContextMenu::standard(position, false);
        assert_eq!(menu.position.x, 190);
        assert_eq!(menu.position.y, 90);

        // Adjustment logic is in handle_show_context_menu system
    }
}
