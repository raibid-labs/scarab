//! Bevy plugin integration for mouse handling
//!
//! This provides the client-side Bevy systems that handle mouse input
//! and render selection/context menus.

use crate::{
    click_handler::{generate_cursor_position_sequence, generate_mouse_sequence, ClickDetector},
    context_menu::ContextMenu,
    selection::{find_word_at, Selection},
    types::{ClickType, MouseButton, MouseEvent, MouseEventKind, MouseMode, Modifiers, Position},
    ClickableItem, ClickableKind, MouseState,
};
use bevy::prelude::*;
use parking_lot::Mutex;
use scarab_clipboard::{ClipboardManager, ClipboardType};
use scarab_protocol::{ControlMessage, TerminalMetrics};
use std::sync::Arc;

/// Event emitted when scrollback buffer should be scrolled in normal mode
#[derive(Event)]
pub struct ScrollbackScrollEvent {
    /// Number of lines to scroll (positive = up, negative = down)
    pub lines: i32,
}

/// Trait for sending IPC messages to the daemon
/// This trait should be implemented by the IpcChannel in scarab-client
pub trait IpcSender: Send + Sync {
    fn send(&self, msg: ControlMessage);
}

/// Wrapper resource for IPC sender to allow dynamic dispatch
#[derive(Resource)]
pub struct MouseIpcSender(pub Arc<dyn IpcSender>);

#[derive(Resource)]

/// Bevy plugin for mouse support
pub struct MousePlugin {
    state: Arc<Mutex<MouseState>>,
}

impl MousePlugin {
    pub fn new(state: Arc<Mutex<MouseState>>) -> Self {
        Self { state }
    }
}

impl Plugin for MousePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(MousePluginState {
            shared_state: Arc::clone(&self.state),
            click_detector: ClickDetector::new(),
            drag_start: None,
            is_dragging: false,
            clipboard: Mutex::new(ClipboardManager::new()),
        })
        .add_event::<ScrollbackScrollEvent>()
        .add_systems(
            Update,
            (
                handle_mouse_input,
                handle_scroll,
                update_selection_rendering,
                handle_context_menu_input,
            )
                .chain(),
        );
    }
}

/// Bevy resource for mouse plugin state
#[derive(Resource)]
struct MousePluginState {
    shared_state: Arc<Mutex<MouseState>>,
    click_detector: ClickDetector,
    drag_start: Option<Position>,
    is_dragging: bool,
    clipboard: Mutex<ClipboardManager>,
}

/// Component for rendered selection overlay
#[derive(Component)]
struct SelectionOverlay;

/// Component for context menu UI
#[allow(dead_code)]
#[derive(Component)]
struct ContextMenuComponent;

/// System to handle mouse button input
fn handle_mouse_input(
    mut plugin_state: ResMut<MousePluginState>,
    mouse_button: Res<ButtonInput<bevy::input::mouse::MouseButton>>,
    windows: Query<&Window>,
    mut commands: Commands,
    ipc: Option<Res<MouseIpcSender>>,
) {
    let window = windows.single();
    let Some(cursor_pos) = window.cursor_position() else {
        return;
    };

    // Convert window coordinates to terminal grid coordinates
    let grid_pos = screen_to_grid(cursor_pos, window.width(), window.height());

    let ipc_ref = ipc.as_ref().map(|r| r.0.as_ref());

    // Handle left mouse button
    if mouse_button.just_pressed(bevy::input::mouse::MouseButton::Left) {
        handle_left_click(&mut plugin_state, grid_pos, ipc_ref);
    }

    if mouse_button.pressed(bevy::input::mouse::MouseButton::Left) {
        handle_left_drag(&mut plugin_state, grid_pos);
    }

    if mouse_button.just_released(bevy::input::mouse::MouseButton::Left) {
        handle_left_release(&mut plugin_state, grid_pos, ipc_ref);
    }

    // Handle right mouse button (context menu)
    if mouse_button.just_pressed(bevy::input::mouse::MouseButton::Right) {
        handle_right_click(&mut plugin_state, grid_pos, &mut commands, ipc_ref);
    }

    // Handle middle mouse button (paste)
    if mouse_button.just_pressed(bevy::input::mouse::MouseButton::Middle) {
        handle_middle_click(&mut plugin_state, grid_pos);
    }
}

/// Handle left mouse button click
fn handle_left_click(
    plugin_state: &mut MousePluginState,
    pos: Position,
    ipc: Option<&dyn IpcSender>,
) {
    let mut state = plugin_state.shared_state.lock();

    // Close context menu if open
    if state.context_menu_visible {
        state.context_menu_visible = false;
        return;
    }

    // Get keyboard modifiers
    // TODO: Get actual modifiers from Bevy input system
    let modifiers = Modifiers::none();

    let event = MouseEvent {
        kind: MouseEventKind::Press,
        position: pos,
        button: Some(MouseButton::Left),
        modifiers,
    };

    // Detect click type
    let click_type = plugin_state.click_detector.handle_press(&event);

    match state.mode {
        MouseMode::Application => {
            // Forward to application
            if let Some(seq) = generate_mouse_sequence(&event) {
                log::debug!("Sending mouse event to application: {:?}", seq);
                if let Some(ipc) = ipc {
                    ipc.send(ControlMessage::Input { data: seq });
                } else {
                    log::warn!("IPC not available, cannot send mouse event to daemon");
                }
            }
        }
        MouseMode::Normal => {
            // Handle in Scarab
            match click_type {
                ClickType::Single => {
                    if modifiers.ctrl {
                        // Ctrl+Click - try to open URL/file
                        handle_ctrl_click(&state, pos);
                    } else if modifiers.shift {
                        // Shift+Click - extend selection
                        extend_selection(&mut state, pos);
                    } else {
                        // Normal click - clear selection and position cursor
                        state.selection = None;
                        plugin_state.drag_start = Some(pos);

                        // Send cursor position to terminal
                        let seq = generate_cursor_position_sequence(pos);
                        log::debug!("Positioning cursor at {:?}: {:?}", pos, seq);
                        if let Some(ipc) = ipc {
                            ipc.send(ControlMessage::Input { data: seq });
                        } else {
                            log::warn!("IPC not available, cannot send cursor position to daemon");
                        }
                    }
                }
                ClickType::Double => {
                    // Select word
                    select_word_at(&mut state, pos);
                }
                ClickType::Triple => {
                    // Select line
                    select_line_at(&mut state, pos);
                }
            }
        }
    }
}

/// Handle left mouse button drag
fn handle_left_drag(plugin_state: &mut MousePluginState, pos: Position) {
    let mut state = plugin_state.shared_state.lock();

    if state.mode == MouseMode::Normal {
        if let Some(start) = plugin_state.drag_start {
            // Start dragging if moved enough
            if !plugin_state.is_dragging && start.distance_to(&pos) > 2.0 {
                plugin_state.is_dragging = true;
            }

            if plugin_state.is_dragging {
                // Update selection
                state.selection = Some(Selection::character(start, pos));
            }
        }
    }
}

/// Handle left mouse button release
fn handle_left_release(
    plugin_state: &mut MousePluginState,
    pos: Position,
    ipc: Option<&dyn IpcSender>,
) {
    plugin_state.drag_start = None;
    plugin_state.is_dragging = false;

    let state = plugin_state.shared_state.lock();
    if state.mode == MouseMode::Application {
        let event = MouseEvent {
            kind: MouseEventKind::Release,
            position: pos,
            button: Some(MouseButton::Left),
            modifiers: Modifiers::none(),
        };

        if let Some(seq) = generate_mouse_sequence(&event) {
            log::debug!("Sending mouse release to application: {:?}", seq);
            if let Some(ipc) = ipc {
                ipc.send(ControlMessage::Input { data: seq });
            } else {
                log::warn!("IPC not available, cannot send mouse release to daemon");
            }
        }
    }
}

/// Handle right mouse button click (context menu)
fn handle_right_click(
    plugin_state: &mut MousePluginState,
    pos: Position,
    _commands: &mut Commands,
    ipc: Option<&dyn IpcSender>,
) {
    let mut state = plugin_state.shared_state.lock();

    if state.mode == MouseMode::Normal {
        // Show context menu
        let has_selection = state.selection.is_some();

        // Check if clicking on a URL or file path
        let _menu = if let Some(item) = find_clickable_at(&state, pos) {
            match item.kind {
                ClickableKind::Url => ContextMenu::url_menu(pos, item.text.clone()),
                ClickableKind::FilePath => ContextMenu::file_menu(pos, item.text.clone()),
            }
        } else {
            ContextMenu::standard(pos, has_selection)
        };

        state.context_menu_visible = true;

        log::debug!("Showing context menu at {:?}", pos);
        // TODO: Spawn context menu UI entity
    } else {
        // Forward to application
        let event = MouseEvent {
            kind: MouseEventKind::Press,
            position: pos,
            button: Some(MouseButton::Right),
            modifiers: Modifiers::none(),
        };

        if let Some(seq) = generate_mouse_sequence(&event) {
            log::debug!("Sending right click to application: {:?}", seq);
            if let Some(ipc) = ipc {
                ipc.send(ControlMessage::Input { data: seq });
            } else {
                log::warn!("IPC not available, cannot send right click to daemon");
            }
        }
    }
}

/// Handle middle mouse button click (paste from primary selection on Linux)
fn handle_middle_click(plugin_state: &mut MousePluginState, pos: Position) {
    let state = plugin_state.shared_state.lock();

    if state.mode == MouseMode::Normal {
        log::info!("Middle click paste at {:?}", pos);

        // On Linux, middle-click pastes from X11 primary selection
        #[cfg(target_os = "linux")]
        {
            let mut clipboard_mgr = plugin_state.clipboard.lock();
            match clipboard_mgr.paste(ClipboardType::Primary) {
                Ok(text) => {
                    if !text.is_empty() {
                        log::info!("Pasting {} characters from primary selection", text.len());
                        // TODO: Send text to terminal via IPC
                        // This would require access to the IpcSender which isn't available in this function
                        // For now, just log the paste operation
                        log::debug!("Primary selection paste text: {:?}", text);
                    } else {
                        log::debug!("Primary selection is empty");
                    }
                }
                Err(e) => {
                    log::warn!("Failed to paste from primary selection: {}", e);
                }
            }
        }

        // On other platforms, middle-click paste is not a standard behavior
        #[cfg(not(target_os = "linux"))]
        {
            log::debug!("Middle-click paste is Linux-specific (X11 primary selection)");
        }
    }
}

/// Handle Ctrl+Click to open URLs and files
fn handle_ctrl_click(state: &MouseState, pos: Position) {
    if let Some(item) = find_clickable_at(state, pos) {
        match item.kind {
            ClickableKind::Url => {
                log::info!("Opening URL: {}", item.text);
                open_url(&item.text);
            }
            ClickableKind::FilePath => {
                log::info!("Opening file: {}", item.text);
                open_file(&item.text);
            }
        }
    }
}

/// Extend selection to position (Shift+Click)
fn extend_selection(state: &mut MouseState, pos: Position) {
    if let Some(selection) = &mut state.selection {
        selection.update_end(pos);
    }
}

/// Select word at position
fn select_word_at(state: &mut MouseState, pos: Position) {
    // TODO: Get actual character at position from terminal grid
    let get_char = |_p: Position| Some('x'); // Placeholder

    if let Some((start, end)) = find_word_at(pos, get_char) {
        state.selection = Some(Selection::word(start, end));
        log::debug!("Selected word from {:?} to {:?}", start, end);
    }
}

/// Select line at position
fn select_line_at(state: &mut MouseState, pos: Position) {
    // TODO: Get actual terminal dimensions
    let cols = 80; // Placeholder

    state.selection = Some(Selection::line(pos.y, pos.y, cols));
    log::debug!("Selected line {}", pos.y);
}

/// Find clickable item at position
fn find_clickable_at(state: &MouseState, pos: Position) -> Option<&ClickableItem> {
    state.clickable_items.iter().find(|item| {
        pos.x >= item.start.x
            && pos.x <= item.end.x
            && pos.y >= item.start.y
            && pos.y <= item.end.y
    })
}

/// Open URL in default browser
fn open_url(url: &str) {
    #[cfg(target_os = "linux")]
    let _ = std::process::Command::new("xdg-open").arg(url).spawn();

    #[cfg(target_os = "macos")]
    let _ = std::process::Command::new("open").arg(url).spawn();

    #[cfg(target_os = "windows")]
    let _ = std::process::Command::new("cmd")
        .args(&["/C", "start", "", url])
        .spawn();
}

/// Open file in default application
fn open_file(path: &str) {
    #[cfg(target_os = "linux")]
    let _ = std::process::Command::new("xdg-open").arg(path).spawn();

    #[cfg(target_os = "macos")]
    let _ = std::process::Command::new("open").arg(path).spawn();

    #[cfg(target_os = "windows")]
    let _ = std::process::Command::new("explorer").arg(path).spawn();
}

/// System to handle scroll wheel input
fn handle_scroll(
    plugin_state: Res<MousePluginState>,
    mut scroll_events: EventReader<bevy::input::mouse::MouseWheel>,
    windows: Query<&Window>,
    ipc: Option<Res<MouseIpcSender>>,
    mut scrollback_events: EventWriter<ScrollbackScrollEvent>,
) {
    let window = windows.single();
    let Some(cursor_pos) = window.cursor_position() else {
        return;
    };

    let grid_pos = screen_to_grid(cursor_pos, window.width(), window.height());

    let ipc_ref = ipc.as_ref().map(|r| r.0.as_ref());

    for event in scroll_events.read() {
        let state = plugin_state.shared_state.lock();

        let button = if event.y > 0.0 {
            MouseButton::ScrollUp
        } else {
            MouseButton::ScrollDown
        };

        let mouse_event = MouseEvent {
            kind: MouseEventKind::Scroll,
            position: grid_pos,
            button: Some(button),
            modifiers: Modifiers::none(),
        };

        match state.mode {
            MouseMode::Application => {
                if let Some(seq) = generate_mouse_sequence(&mouse_event) {
                    log::debug!("Sending scroll to application: {:?}", seq);
                    if let Some(ipc) = ipc_ref {
                        ipc.send(ControlMessage::Input { data: seq });
                    } else {
                        log::warn!("IPC not available, cannot send scroll to daemon");
                    }
                }
            }
            MouseMode::Normal => {
                // Emit scrollback event for scrollback system to handle
                let lines = (event.y * 3.0) as i32; // Keep sign: positive = up, negative = down
                log::debug!("Emitting scrollback scroll event: {} lines", lines);
                scrollback_events.send(ScrollbackScrollEvent { lines });
            }
        }
    }
}

/// System to render selection overlay
fn update_selection_rendering(
    plugin_state: Res<MousePluginState>,
    mut commands: Commands,
    selection_query: Query<Entity, With<SelectionOverlay>>,
    metrics: Option<Res<TerminalMetrics>>,
) {
    let state = plugin_state.shared_state.lock();

    // Clear existing selection overlays
    for entity in &selection_query {
        commands.entity(entity).despawn();
    }

    // Render new selection if exists
    if let Some(selection) = &state.selection {
        // Get terminal metrics (use default if not available)
        let metrics = metrics.as_deref().copied().unwrap_or_default();

        log::trace!("Rendering selection: {:?} with metrics: {:?}", selection, metrics);

        let (start, end) = selection.normalized();

        match selection.kind {
            crate::selection::SelectionKind::Block => {
                // Rectangular/block selection - render a rectangle for each row
                let min_x = start.x.min(end.x);
                let max_x = start.x.max(end.x);
                let min_y = start.y.min(end.y);
                let max_y = start.y.max(end.y);

                for row in min_y..=max_y {
                    spawn_selection_overlay_for_range(
                        &mut commands,
                        &metrics,
                        min_x,
                        max_x,
                        row,
                    );
                }
            }
            crate::selection::SelectionKind::Line => {
                // Line selection - render full lines
                for row in start.y..=end.y {
                    spawn_selection_overlay_for_range(
                        &mut commands,
                        &metrics,
                        0,
                        metrics.columns - 1,
                        row,
                    );
                }
            }
            _ => {
                // Normal/Word selection - linear character-by-character
                if start.y == end.y {
                    // Single line selection
                    spawn_selection_overlay_for_range(
                        &mut commands,
                        &metrics,
                        start.x,
                        end.x,
                        start.y,
                    );
                } else {
                    // Multi-line selection
                    let cols = metrics.columns;

                    // First line: from start.x to end of line
                    spawn_selection_overlay_for_range(
                        &mut commands,
                        &metrics,
                        start.x,
                        cols - 1,
                        start.y,
                    );

                    // Middle lines: full width
                    for row in (start.y + 1)..end.y {
                        spawn_selection_overlay_for_range(
                            &mut commands,
                            &metrics,
                            0,
                            cols - 1,
                            row,
                        );
                    }

                    // Last line: from start of line to end.x
                    if end.y > start.y {
                        spawn_selection_overlay_for_range(
                            &mut commands,
                            &metrics,
                            0,
                            end.x,
                            end.y,
                        );
                    }
                }
            }
        }
    }
}

/// Spawn a selection overlay sprite for a range of columns on a single row
fn spawn_selection_overlay_for_range(
    commands: &mut Commands,
    metrics: &TerminalMetrics,
    start_col: u16,
    end_col: u16,
    row: u16,
) {
    let cell_width = metrics.cell_width;
    let cell_height = metrics.cell_height;

    // Calculate total width of the selection on this row
    let num_cells = (end_col - start_col + 1) as f32;
    let total_width = num_cells * cell_width;

    // Calculate position (using same coordinate system as terminal rendering)
    // Terminal rendering uses: x = col * cell_width, y = -(row * cell_height)
    // Sprite anchor is at center, so we offset by half dimensions
    let x = start_col as f32 * cell_width + total_width / 2.0;
    let y = -(row as f32 * cell_height) - cell_height / 2.0;

    // Selection color: semi-transparent blue (classic terminal selection color)
    let selection_color = Color::srgba(0.3, 0.6, 1.0, 0.3);

    commands.spawn((
        Sprite {
            color: selection_color,
            custom_size: Some(Vec2::new(total_width, cell_height)),
            ..default()
        },
        Transform::from_xyz(x, y, 1.0), // Z=1.0 to render above terminal text (which is at Z=0)
        SelectionOverlay,
    ));
}

/// System to handle context menu keyboard input
fn handle_context_menu_input(
    plugin_state: Res<MousePluginState>,
    _keyboard: Res<ButtonInput<KeyCode>>,
) {
    let state = plugin_state.shared_state.lock();

    if !state.context_menu_visible {
        return;
    }

    // Handle arrow keys, Enter, Escape for context menu navigation
    // TODO: Implement context menu interaction
}

/// Convert screen coordinates to terminal grid coordinates
fn screen_to_grid(cursor_pos: Vec2, window_width: f32, window_height: f32) -> Position {
    // TODO: Use actual font metrics and terminal dimensions
    // This is a placeholder calculation
    let cols = 80;
    let rows = 24;

    let x = ((cursor_pos.x / window_width) * cols as f32) as u16;
    let y = ((cursor_pos.y / window_height) * rows as f32) as u16;

    Position::new(x.min(cols - 1), y.min(rows - 1))
}
