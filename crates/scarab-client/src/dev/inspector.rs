// Bevy ECS Inspector
//
// A developer tool for inspecting Bevy entities, components, and resources at runtime.
// Provides a Ratatui-based overlay showing the entity tree, component details, and system metrics.
//
// # Features
//
// - Entity tree view with hierarchical display
// - Component inspection for selected entities
// - Resource browser
// - System execution order visualization
// - Search and filter capabilities
// - Keyboard navigation (vim-style)
//
// # Keybindings
//
// - Ctrl+Shift+I: Toggle inspector visibility
// - j/k or Down/Up: Navigate entity list
// - h/l or Left/Right: Collapse/expand entity nodes
// - /: Search entities
// - r: Refresh entity list
// - Esc: Close inspector
//
// # Architecture
//
// The inspector uses the Ratatui bridge to render a TUI overlay on top of the terminal.
// It queries all entities in the world and displays their components using reflection
// where available.

use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Widget},
};
use std::collections::HashSet;

use crate::ratatui_bridge::{
    RatEvent, RatKeyCode, RatatuiBridgePlugin, RatatuiSurface, SurfaceBuffers, SurfaceInputEvent,
};

/// Cached information about an entity for display purposes
#[derive(Debug, Clone)]
pub struct EntityInfo {
    /// The entity ID
    pub entity: Entity,
    /// Display name (from Name component or formatted entity ID)
    pub name: String,
    /// Number of components on this entity
    pub component_count: usize,
    /// Names of all components on this entity
    pub component_names: Vec<String>,
}

// Inspector state resource
#[derive(Resource, Default)]
pub struct BevyInspectorState {
    /// Whether the inspector overlay is visible
    pub visible: bool,
    /// Currently selected entity index in the list
    pub selected_index: usize,
    /// Cached entity information (updated each frame when visible)
    pub entity_infos: Vec<EntityInfo>,
    /// Set of expanded entities (for hierarchical view)
    pub expanded_entities: HashSet<Entity>,
    /// Search query for filtering entities
    pub search_query: String,
    /// Whether search mode is active
    pub search_mode: bool,
    /// Current view mode
    pub view_mode: InspectorViewMode,
    /// Scroll offset for component details
    pub detail_scroll: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum InspectorViewMode {
    #[default]
    Entities,
    Components,
    Resources,
    Systems,
}

impl BevyInspectorState {
    pub fn toggle(&mut self) {
        self.visible = !self.visible;
        if self.visible {
            // Reset state when opening
            self.selected_index = 0;
            self.search_query.clear();
            self.search_mode = false;
            self.detail_scroll = 0;
        }
    }

    pub fn select_next(&mut self) {
        if self.entity_infos.is_empty() {
            return;
        }
        self.selected_index = (self.selected_index + 1).min(self.entity_infos.len() - 1);
    }

    pub fn select_prev(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
        }
    }

    pub fn selected_entity_info(&self) -> Option<&EntityInfo> {
        self.entity_infos.get(self.selected_index)
    }

    pub fn toggle_expanded(&mut self, entity: Entity) {
        if self.expanded_entities.contains(&entity) {
            self.expanded_entities.remove(&entity);
        } else {
            self.expanded_entities.insert(entity);
        }
    }

    pub fn scroll_detail_down(&mut self) {
        self.detail_scroll = self.detail_scroll.saturating_add(1);
    }

    pub fn scroll_detail_up(&mut self) {
        self.detail_scroll = self.detail_scroll.saturating_sub(1);
    }
}

// Plugin definition
pub struct BevyInspectorPlugin;

impl Plugin for BevyInspectorPlugin {
    fn build(&self, app: &mut App) {
        // Ensure Ratatui bridge is loaded
        if !app.is_plugin_added::<RatatuiBridgePlugin>() {
            app.add_plugins(RatatuiBridgePlugin);
        }

        app.init_resource::<BevyInspectorState>().add_systems(
            Update,
            (
                toggle_inspector_input,
                update_entity_list_exclusive,
                spawn_inspector_surface,
                render_inspector,
                handle_inspector_input,
            )
                .chain(),
        );

        info!("Bevy Inspector initialized - Press Ctrl+Shift+I to open");
    }
}

// System: Toggle inspector with Ctrl+Shift+I
fn toggle_inspector_input(keys: Res<ButtonInput<KeyCode>>, mut state: ResMut<BevyInspectorState>) {
    // Check for Ctrl+Shift+I
    let ctrl = keys.pressed(KeyCode::ControlLeft) || keys.pressed(KeyCode::ControlRight);
    let shift = keys.pressed(KeyCode::ShiftLeft) || keys.pressed(KeyCode::ShiftRight);
    let i_pressed = keys.just_pressed(KeyCode::KeyI);

    if ctrl && shift && i_pressed {
        state.toggle();
        if state.visible {
            info!("Bevy Inspector opened");
        } else {
            info!("Bevy Inspector closed");
        }
    }
}

// Exclusive system: Update entity list from world
// This is an exclusive system because it needs full world access to inspect entities
fn update_entity_list_exclusive(world: &mut World) {
    // Check if visible (need to access resource)
    let visible = world
        .get_resource::<BevyInspectorState>()
        .map(|s| s.visible)
        .unwrap_or(false);

    if !visible {
        return;
    }

    // Get search query
    let search_query = world
        .get_resource::<BevyInspectorState>()
        .map(|s| s.search_query.clone())
        .unwrap_or_default();

    // Collect all entity information
    let mut entity_infos: Vec<EntityInfo> = world
        .iter_entities()
        .map(|entity_ref| {
            let entity = entity_ref.id();

            // Get name
            let name = entity_ref
                .get::<Name>()
                .map(|n| n.to_string())
                .unwrap_or_else(|| format!("{:?}", entity));

            // Get component count and names
            let archetype = entity_ref.archetype();
            let component_count = archetype.component_count();

            let component_names: Vec<String> = archetype
                .components()
                .filter_map(|component_id| {
                    world
                        .components()
                        .get_info(component_id)
                        .map(|info| info.name().to_string())
                })
                .collect();

            EntityInfo {
                entity,
                name,
                component_count,
                component_names,
            }
        })
        .collect();

    // Sort entities by index for consistent ordering
    entity_infos.sort_by_key(|info| (info.entity.index(), info.entity.generation()));

    // Apply search filter if active
    if !search_query.is_empty() {
        let query_lower = search_query.to_lowercase();
        entity_infos.retain(|info| {
            // Check if name contains query
            if info.name.to_lowercase().contains(&query_lower) {
                return true;
            }
            // Check if entity ID contains query
            if format!("{:?}", info.entity)
                .to_lowercase()
                .contains(&query_lower)
            {
                return true;
            }
            false
        });
    }

    // Update state
    if let Some(mut state) = world.get_resource_mut::<BevyInspectorState>() {
        // Clamp selected index
        if state.selected_index >= entity_infos.len() && !entity_infos.is_empty() {
            state.selected_index = entity_infos.len() - 1;
        }
        state.entity_infos = entity_infos;
    }
}

// System: Spawn or update inspector surface
fn spawn_inspector_surface(
    mut commands: Commands,
    state: Res<BevyInspectorState>,
    mut surfaces: Query<(Entity, &mut RatatuiSurface), With<InspectorSurfaceMarker>>,
    windows: Query<&Window, With<PrimaryWindow>>,
) {
    if !state.visible {
        // Despawn surface if inspector is closed
        for (entity, _) in surfaces.iter() {
            commands.entity(entity).despawn_recursive();
        }
        return;
    }

    // Get window dimensions for sizing
    let (width, height) = if let Ok(window) = windows.get_single() {
        // Use 80% of window size for inspector
        let cols = ((window.width() / 12.0) * 0.8) as u16; // Rough char width estimate
        let rows = ((window.height() / 20.0) * 0.8) as u16; // Rough char height estimate
        (cols.max(60), rows.max(20))
    } else {
        (80, 30)
    };

    // Calculate centered position
    let x = (100_u16.saturating_sub(width)) / 2;
    let y = (60_u16.saturating_sub(height)) / 2;

    if surfaces.is_empty() {
        // Create new surface
        commands.spawn((
            RatatuiSurface::new(x, y, width, height).with_z_index(1000.0), // High z-index to appear on top
            InspectorSurfaceMarker,
        ));
    } else {
        // Update existing surface size/position
        for (_, mut surface) in surfaces.iter_mut() {
            surface.x = x;
            surface.y = y;
            surface.width = width;
            surface.height = height;
            surface.dirty = true;
        }
    }
}

// Marker component for inspector surface
#[derive(Component)]
struct InspectorSurfaceMarker;

// System: Render inspector UI using Ratatui
// Note: Uses cached entity info from state, no &World access needed
fn render_inspector(
    state: Res<BevyInspectorState>,
    mut buffers: ResMut<SurfaceBuffers>,
    surfaces: Query<(Entity, &RatatuiSurface), With<InspectorSurfaceMarker>>,
) {
    if !state.visible {
        return;
    }

    for (entity, surface) in surfaces.iter() {
        let buffer = buffers.get_or_create(entity, surface.width, surface.height);
        let area = surface.rect();

        // Render the inspector widget using cached entity info
        InspectorWidget { state: &state }.render(area, buffer);
    }
}

// Custom Ratatui widget for the inspector
struct InspectorWidget<'a> {
    state: &'a BevyInspectorState,
}

impl<'a> Widget for InspectorWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Main layout: horizontal split for entity list and details
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
            .split(area);

        // Render entity list on the left
        self.render_entity_list(chunks[0], buf);

        // Render entity details on the right
        self.render_entity_details(chunks[1], buf);

        // Render status bar at the bottom
        self.render_status_bar(area, buf);
    }
}

impl<'a> InspectorWidget<'a> {
    fn render_entity_list(&self, area: Rect, buf: &mut Buffer) {
        let title = match self.state.view_mode {
            InspectorViewMode::Entities => format!("Entities ({})", self.state.entity_infos.len()),
            InspectorViewMode::Components => "Components".to_string(),
            InspectorViewMode::Resources => "Resources".to_string(),
            InspectorViewMode::Systems => "Systems".to_string(),
        };

        let block = Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan));

        let inner = block.inner(area);
        block.render(area, buf);

        // Build entity list items from cached info
        let items: Vec<ListItem> = self
            .state
            .entity_infos
            .iter()
            .enumerate()
            .map(|(idx, info)| {
                let is_selected = idx == self.state.selected_index;
                let is_expanded = self.state.expanded_entities.contains(&info.entity);

                let prefix = if is_expanded { "[-] " } else { "[+] " };
                let text = format!(
                    "{}{:?} | {} ({} components)",
                    prefix, info.entity, info.name, info.component_count
                );

                let style = if is_selected {
                    Style::default()
                        .fg(Color::Black)
                        .bg(Color::Cyan)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::White)
                };

                ListItem::new(text).style(style)
            })
            .collect();

        let list = List::new(items);
        list.render(inner, buf);
    }

    fn render_entity_details(&self, area: Rect, buf: &mut Buffer) {
        let block = Block::default()
            .title("Details")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan));

        let inner = block.inner(area);
        block.render(area, buf);

        if let Some(info) = self.state.selected_entity_info() {
            let mut lines = vec![
                Line::from(vec![
                    Span::styled("Entity: ", Style::default().add_modifier(Modifier::BOLD)),
                    Span::raw(format!("{:?}", info.entity)),
                ]),
                Line::from(""),
            ];

            // Add entity name
            lines.push(Line::from(vec![
                Span::styled("Name: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::styled(&info.name, Style::default().fg(Color::Green)),
            ]));
            lines.push(Line::from(""));

            // List components
            lines.push(Line::from(vec![Span::styled(
                "Components:",
                Style::default()
                    .add_modifier(Modifier::BOLD)
                    .fg(Color::Yellow),
            )]));

            for component_name in &info.component_names {
                lines.push(Line::from(vec![
                    Span::raw("  - "),
                    Span::styled(component_name.as_str(), Style::default().fg(Color::Cyan)),
                ]));
            }

            // Handle scrolling
            let visible_lines: Vec<Line> = lines
                .into_iter()
                .skip(self.state.detail_scroll)
                .take(inner.height as usize)
                .collect();

            let paragraph = Paragraph::new(visible_lines);
            paragraph.render(inner, buf);
        } else {
            let text = vec![Line::from("Select an entity to view details")];
            Paragraph::new(text).render(inner, buf);
        }
    }

    fn render_status_bar(&self, area: Rect, buf: &mut Buffer) {
        // Status bar at the bottom
        let status_area = Rect {
            x: area.x,
            y: area.y + area.height.saturating_sub(1),
            width: area.width,
            height: 1,
        };

        let help_text = if self.state.search_mode {
            format!(
                "Search: {} | Esc: cancel | Enter: apply",
                self.state.search_query
            )
        } else {
            "j/k: navigate | h/l: expand/collapse | /: search | r: refresh | Esc: close | Ctrl+Shift+I: toggle".to_string()
        };

        let status = Line::from(vec![Span::styled(
            help_text,
            Style::default().fg(Color::Black).bg(Color::White),
        )]);

        // Clear the status bar area
        for x in status_area.x..status_area.x + status_area.width {
            if let Some(cell) = buf.cell_mut((x, status_area.y)) {
                cell.set_char(' ')
                    .set_style(Style::default().bg(Color::White));
            }
        }

        // Render status text
        let mut x_offset = status_area.x;
        for span in status.spans {
            let text = span.content.to_string();
            for ch in text
                .chars()
                .take((status_area.width - (x_offset - status_area.x)) as usize)
            {
                if let Some(cell) = buf.cell_mut((x_offset, status_area.y)) {
                    cell.set_char(ch).set_style(span.style);
                }
                x_offset += 1;
            }
        }
    }
}

// System: Handle keyboard input for inspector
fn handle_inspector_input(
    mut state: ResMut<BevyInspectorState>,
    mut events: EventReader<SurfaceInputEvent>,
    surfaces: Query<Entity, With<InspectorSurfaceMarker>>,
) {
    if !state.visible {
        return;
    }

    // Only handle input if the inspector surface is the target
    let inspector_entity = surfaces.iter().next();
    if inspector_entity.is_none() {
        return;
    }

    for event in events.read() {
        // Only process events for our surface
        if Some(event.surface) != inspector_entity {
            continue;
        }

        match &event.event {
            RatEvent::Key(key) => {
                if state.search_mode {
                    handle_search_input(&mut state, key);
                } else {
                    handle_navigation_input(&mut state, key);
                }
            }
            _ => {}
        }
    }
}

fn handle_navigation_input(
    state: &mut BevyInspectorState,
    key: &ratatui::crossterm::event::KeyEvent,
) {
    match key.code {
        // Navigation
        RatKeyCode::Char('j') | RatKeyCode::Down => {
            state.select_next();
        }
        RatKeyCode::Char('k') | RatKeyCode::Up => {
            state.select_prev();
        }
        RatKeyCode::Char('h') | RatKeyCode::Left => {
            // Collapse selected entity
            if let Some(entity) = state.selected_entity_info().map(|i| i.entity) {
                state.expanded_entities.remove(&entity);
            }
        }
        RatKeyCode::Char('l') | RatKeyCode::Right => {
            // Expand selected entity
            if let Some(info) = state.selected_entity_info() {
                let entity = info.entity;
                state.expanded_entities.insert(entity);
            }
        }
        RatKeyCode::Char('d') => {
            state.scroll_detail_down();
        }
        RatKeyCode::Char('u') => {
            state.scroll_detail_up();
        }

        // Search
        RatKeyCode::Char('/') => {
            state.search_mode = true;
            state.search_query.clear();
        }

        // Refresh
        RatKeyCode::Char('r') => {
            // Entity list is automatically refreshed each frame
            info!("Inspector refreshed");
        }

        // Close
        RatKeyCode::Esc => {
            state.visible = false;
        }

        _ => {}
    }
}

fn handle_search_input(state: &mut BevyInspectorState, key: &ratatui::crossterm::event::KeyEvent) {
    match key.code {
        RatKeyCode::Char(c) => {
            state.search_query.push(c);
        }
        RatKeyCode::Backspace => {
            state.search_query.pop();
        }
        RatKeyCode::Enter => {
            state.search_mode = false;
            // Search is applied automatically via update_entity_list
        }
        RatKeyCode::Esc => {
            state.search_mode = false;
            state.search_query.clear();
        }
        _ => {}
    }
}
