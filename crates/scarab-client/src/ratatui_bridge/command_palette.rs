// Command Palette Widget
//
// A searchable command palette overlay built with Ratatui widgets.
// This demonstrates the full Ratatui bridge integration with:
// - Widget rendering to surfaces
// - Input handling and filtering
// - Focus management
// - Toggle visibility

use bevy::prelude::*;
use fusabi_tui_core::{Constraint, Direction, Layout, Color, Modifier, Style};
use fusabi_tui_widgets::{
    Block, Borders, List, ListItem, ListState, Paragraph, Widget, StatefulWidget, Line, Span,
};

use super::{
    input::{SurfaceFocus, SurfaceInputEvent},
    surface::{RatatuiSurface, SurfaceBuffers},
};

/// A command entry in the palette
#[derive(Clone, Debug)]
pub struct PaletteCommand {
    pub id: String,
    pub label: String,
    pub description: Option<String>,
    pub shortcut: Option<String>,
}

/// Resource managing command palette state
#[derive(Resource)]
pub struct CommandPaletteState {
    /// All available commands
    pub commands: Vec<PaletteCommand>,
    /// Current filter text
    pub filter: String,
    /// Filtered command indices
    pub filtered: Vec<usize>,
    /// Selected index in filtered list
    pub selected: usize,
    /// Whether palette is visible
    pub visible: bool,
}

impl Default for CommandPaletteState {
    fn default() -> Self {
        let mut state = Self {
            commands: default_commands(),
            filter: String::new(),
            filtered: Vec::new(),
            selected: 0,
            visible: false,
        };
        // Initialize filtered list with all commands
        state.update_filter();
        state
    }
}

impl CommandPaletteState {
    /// Update filtered list based on current filter
    pub fn update_filter(&mut self) {
        let filter_lower = self.filter.to_lowercase();
        self.filtered = self
            .commands
            .iter()
            .enumerate()
            .filter(|(_, cmd)| {
                cmd.label.to_lowercase().contains(&filter_lower)
                    || cmd
                        .description
                        .as_ref()
                        .map_or(false, |d| d.to_lowercase().contains(&filter_lower))
            })
            .map(|(i, _)| i)
            .collect();

        // Reset selection if out of bounds
        if self.filtered.is_empty() {
            self.selected = 0;
        } else if self.selected >= self.filtered.len() {
            self.selected = self.filtered.len() - 1;
        }
    }

    /// Move selection up
    pub fn select_previous(&mut self) {
        if self.selected > 0 {
            self.selected -= 1;
        }
    }

    /// Move selection down
    pub fn select_next(&mut self) {
        if !self.filtered.is_empty() && self.selected + 1 < self.filtered.len() {
            self.selected += 1;
        }
    }

    /// Get currently selected command
    pub fn selected_command(&self) -> Option<&PaletteCommand> {
        self.filtered
            .get(self.selected)
            .and_then(|&i| self.commands.get(i))
    }

    /// Show the palette
    pub fn show(&mut self) {
        self.visible = true;
        self.filter.clear();
        self.update_filter();
    }

    /// Hide the palette
    pub fn hide(&mut self) {
        self.visible = false;
    }
}

/// Default commands for the palette
fn default_commands() -> Vec<PaletteCommand> {
    vec![
        PaletteCommand {
            id: "new_tab".into(),
            label: "New Tab".into(),
            description: Some("Create a new terminal tab".into()),
            shortcut: Some("Ctrl+T".into()),
        },
        PaletteCommand {
            id: "close_tab".into(),
            label: "Close Tab".into(),
            description: Some("Close current tab".into()),
            shortcut: Some("Ctrl+W".into()),
        },
        PaletteCommand {
            id: "split_horizontal".into(),
            label: "Split Horizontal".into(),
            description: Some("Split pane horizontally".into()),
            shortcut: Some("Ctrl+Shift+H".into()),
        },
        PaletteCommand {
            id: "split_vertical".into(),
            label: "Split Vertical".into(),
            description: Some("Split pane vertically".into()),
            shortcut: Some("Ctrl+Shift+V".into()),
        },
        PaletteCommand {
            id: "copy_mode".into(),
            label: "Enter Copy Mode".into(),
            description: Some("Enter vim-like copy mode".into()),
            shortcut: Some("Ctrl+Shift+C".into()),
        },
        PaletteCommand {
            id: "search".into(),
            label: "Search".into(),
            description: Some("Search terminal output".into()),
            shortcut: Some("Ctrl+F".into()),
        },
        PaletteCommand {
            id: "settings".into(),
            label: "Open Settings".into(),
            description: Some("Open configuration file".into()),
            shortcut: None,
        },
        PaletteCommand {
            id: "toggle_theme".into(),
            label: "Toggle Theme".into(),
            description: Some("Switch between light and dark theme".into()),
            shortcut: Some("Ctrl+Shift+T".into()),
        },
        PaletteCommand {
            id: "clear_scrollback".into(),
            label: "Clear Scrollback".into(),
            description: Some("Clear terminal scrollback buffer".into()),
            shortcut: Some("Ctrl+K".into()),
        },
        PaletteCommand {
            id: "zoom_in".into(),
            label: "Zoom In".into(),
            description: Some("Increase font size".into()),
            shortcut: Some("Ctrl+=".into()),
        },
        PaletteCommand {
            id: "zoom_out".into(),
            label: "Zoom Out".into(),
            description: Some("Decrease font size".into()),
            shortcut: Some("Ctrl+-".into()),
        },
    ]
}

/// Marker component for command palette surface
#[derive(Component)]
pub struct CommandPaletteSurface;

/// Event fired when a command is selected
#[derive(Event)]
pub struct CommandSelected {
    pub command_id: String,
}

/// System to toggle command palette visibility
pub fn toggle_command_palette(
    keys: Res<ButtonInput<KeyCode>>,
    mut state: ResMut<CommandPaletteState>,
    mut focus: ResMut<SurfaceFocus>,
    palette_query: Query<Entity, With<CommandPaletteSurface>>,
) {
    // Ctrl+Shift+P to toggle
    let ctrl = keys.pressed(KeyCode::ControlLeft) || keys.pressed(KeyCode::ControlRight);
    let shift = keys.pressed(KeyCode::ShiftLeft) || keys.pressed(KeyCode::ShiftRight);

    if ctrl && shift && keys.just_pressed(KeyCode::KeyP) {
        if state.visible {
            state.hide();
            if let Ok(entity) = palette_query.get_single() {
                focus.remove(entity);
            }
        } else {
            state.show();
            if let Ok(entity) = palette_query.get_single() {
                focus.push(entity);
            }
        }
    }

    // Escape to close
    if state.visible && keys.just_pressed(KeyCode::Escape) {
        state.hide();
        if let Ok(entity) = palette_query.get_single() {
            focus.remove(entity);
        }
    }
}

/// System to handle input for command palette
pub fn handle_palette_input(
    mut state: ResMut<CommandPaletteState>,
    mut events: EventReader<SurfaceInputEvent>,
    mut command_events: EventWriter<CommandSelected>,
    mut focus: ResMut<SurfaceFocus>,
    palette_query: Query<Entity, With<CommandPaletteSurface>>,
) {
    let Ok(palette_entity) = palette_query.get_single() else {
        return;
    };

    for event in events.read() {
        if event.surface != palette_entity {
            continue;
        }

        use crossterm::event::{Event, KeyCode};
        if let Event::Key(key) = &event.event {
            match key.code {
                KeyCode::Up => state.select_previous(),
                KeyCode::Down => state.select_next(),
                KeyCode::Enter => {
                    if let Some(cmd) = state.selected_command() {
                        command_events.send(CommandSelected {
                            command_id: cmd.id.clone(),
                        });
                        state.hide();
                        focus.remove(palette_entity);
                    }
                }
                KeyCode::Esc => {
                    state.hide();
                    focus.remove(palette_entity);
                }
                KeyCode::Backspace => {
                    state.filter.pop();
                    state.update_filter();
                }
                KeyCode::Char(c) => {
                    state.filter.push(c);
                    state.update_filter();
                }
                _ => {}
            }
        }
    }
}

/// System to render command palette
pub fn render_command_palette(
    state: Res<CommandPaletteState>,
    mut buffers: ResMut<SurfaceBuffers>,
    mut surfaces: Query<(Entity, &mut RatatuiSurface), With<CommandPaletteSurface>>,
) {
    let Ok((entity, mut surface)) = surfaces.get_single_mut() else {
        return;
    };

    surface.visible = state.visible;
    if !state.visible {
        return;
    }

    surface.mark_dirty();

    let buffer = buffers.get_or_create(entity, surface.width, surface.height);
    let area = surface.rect();

    // Clear buffer
    buffer.clear();

    // Layout: input at top, list below
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(&[
            Constraint::Length(3), // Input box
            Constraint::Min(1),    // Command list
        ])
        .split(area);

    // Render input box
    let input_block = Block::default()
        .title(" Command Palette ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    let input_text = if state.filter.is_empty() {
        Span::styled("Type to filter...", Style::default().fg(Color::DarkGray))
    } else {
        Span::raw(&state.filter)
    };

    let input = Paragraph::new(input_text).block(input_block);
    input.render(chunks[0], buffer);

    // Render command list
    let items: Vec<ListItem> = state
        .filtered
        .iter()
        .enumerate()
        .map(|(i, &cmd_idx)| {
            let cmd = &state.commands[cmd_idx];
            let style = if i == state.selected {
                Style::default()
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            let mut spans = vec![Span::styled(&cmd.label, style)];

            // Add description
            if let Some(desc) = &cmd.description {
                spans.push(Span::raw(" - "));
                spans.push(Span::styled(desc, Style::default().fg(Color::DarkGray)));
            }

            // Add shortcut at the end
            if let Some(shortcut) = &cmd.shortcut {
                spans.push(Span::raw(" "));
                spans.push(Span::styled(
                    format!("[{}]", shortcut),
                    Style::default().fg(Color::DarkGray),
                ));
            }

            ListItem::new(Line::from(spans))
        })
        .collect();

    let list_title = format!(
        " Commands ({}/{}) ",
        state.filtered.len(),
        state.commands.len()
    );

    let list = List::new(items).block(Block::default().title(list_title).borders(Borders::ALL));
    let mut list_state = ListState::default();
    list_state.select(Some(state.selected));
    list.render(chunks[1], buffer, &mut list_state);
}

/// System to spawn command palette surface
pub fn spawn_command_palette(
    mut commands: Commands,
    existing: Query<Entity, With<CommandPaletteSurface>>,
) {
    if !existing.is_empty() {
        return;
    }

    // Center the palette (60x15 cells)
    // Position it roughly centered for a 200-col terminal
    let width = 60;
    let height = 15;
    let x = 70; // Roughly centered for 200-col terminal
    let y = 10;

    commands.spawn((
        CommandPaletteSurface,
        RatatuiSurface::new(x, y, width, height).with_z_index(200.0), // High z-index for overlay
    ));

    info!("Command palette surface spawned at grid ({}, {})", x, y);
}

/// System to log selected commands (for debugging)
pub fn log_selected_commands(mut events: EventReader<CommandSelected>) {
    for event in events.read() {
        info!("Command selected: {}", event.command_id);
    }
}

/// Plugin for command palette
pub struct CommandPalettePlugin;

impl Plugin for CommandPalettePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CommandPaletteState>()
            .add_event::<CommandSelected>()
            .add_systems(Startup, spawn_command_palette)
            .add_systems(
                Update,
                (
                    toggle_command_palette,
                    handle_palette_input,
                    render_command_palette,
                    log_selected_commands,
                )
                    .chain(),
            );

        info!("CommandPalettePlugin initialized");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_palette_state_creation() {
        let state = CommandPaletteState::default();
        assert!(!state.visible);
        assert!(state.filter.is_empty());
        assert_eq!(state.selected, 0);
        // Should have all commands in filtered list initially
        assert_eq!(state.filtered.len(), state.commands.len());
    }

    #[test]
    fn test_filter_update() {
        let mut state = CommandPaletteState::default();
        let total_commands = state.commands.len();

        // Filter for "tab"
        state.filter = "tab".to_string();
        state.update_filter();

        // Should find "New Tab" and "Close Tab"
        assert!(state.filtered.len() < total_commands);
        assert!(state.filtered.len() >= 2);

        // Clear filter
        state.filter.clear();
        state.update_filter();
        assert_eq!(state.filtered.len(), total_commands);
    }

    #[test]
    fn test_selection_navigation() {
        let mut state = CommandPaletteState::default();

        // Initial selection
        assert_eq!(state.selected, 0);

        // Navigate down
        state.select_next();
        assert_eq!(state.selected, 1);

        state.select_next();
        assert_eq!(state.selected, 2);

        // Navigate up
        state.select_previous();
        assert_eq!(state.selected, 1);

        state.select_previous();
        assert_eq!(state.selected, 0);

        // Can't go below 0
        state.select_previous();
        assert_eq!(state.selected, 0);
    }

    #[test]
    fn test_selection_bounds() {
        let mut state = CommandPaletteState::default();
        let max_index = state.filtered.len() - 1;

        // Try to go past the end
        state.selected = max_index;
        state.select_next();
        assert_eq!(state.selected, max_index); // Should stay at max
    }

    #[test]
    fn test_selected_command() {
        let mut state = CommandPaletteState::default();

        // Get first command
        state.selected = 0;
        let cmd = state.selected_command();
        assert!(cmd.is_some());

        // Get last command
        state.selected = state.filtered.len() - 1;
        let cmd = state.selected_command();
        assert!(cmd.is_some());
    }

    #[test]
    fn test_show_hide() {
        let mut state = CommandPaletteState::default();

        // Initially hidden
        assert!(!state.visible);

        // Show
        state.show();
        assert!(state.visible);
        assert!(state.filter.is_empty());

        // Add filter
        state.filter = "test".to_string();

        // Hide
        state.hide();
        assert!(!state.visible);

        // Show again - filter should be cleared
        state.show();
        assert!(state.visible);
        assert!(state.filter.is_empty());
    }

    #[test]
    fn test_filter_resets_selection() {
        let mut state = CommandPaletteState::default();

        // Select item in the middle
        state.selected = 5;

        // Filter to single result
        state.filter = "xyz_nonexistent".to_string();
        state.update_filter();

        // Selection should be reset
        assert_eq!(state.selected, 0);
    }

    #[test]
    fn test_default_commands() {
        let commands = default_commands();
        assert!(!commands.is_empty());

        // Check that we have some expected commands
        let has_new_tab = commands.iter().any(|c| c.id == "new_tab");
        let has_settings = commands.iter().any(|c| c.id == "settings");
        assert!(has_new_tab);
        assert!(has_settings);
    }

    #[test]
    fn test_command_structure() {
        let cmd = PaletteCommand {
            id: "test_cmd".into(),
            label: "Test Command".into(),
            description: Some("A test description".into()),
            shortcut: Some("Ctrl+T".into()),
        };

        assert_eq!(cmd.id, "test_cmd");
        assert_eq!(cmd.label, "Test Command");
        assert_eq!(cmd.description.unwrap(), "A test description");
        assert_eq!(cmd.shortcut.unwrap(), "Ctrl+T");
    }
}
