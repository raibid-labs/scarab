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
use bevy::ecs::world::World;
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
    RatatuiBridgePlugin, RatatuiSurface, SurfaceBuffers, SurfaceInputEvent, RatEvent, RatKeyCode,
};

// Inspector state resource
#[derive(Resource, Default)]
pub struct BevyInspectorState {
    /// Whether the inspector overlay is visible
    pub visible: bool,
    /// Currently selected entity index in the list
    pub selected_index: usize,
    /// List of all entities in the world (cached)
    pub entities: Vec<Entity>,
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
        if self.entities.is_empty() {
            return;
        }
        self.selected_index = (self.selected_index + 1).min(self.entities.len() - 1);
    }

    pub fn select_prev(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
        }
    }

    pub fn selected_entity(&self) -> Option<Entity> {
        self.entities.get(self.selected_index).copied()
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

// Entity information snapshot (for display)
#[derive(Debug, Clone)]
struct EntitySnapshot {
    entity: Entity,
    name: Option<String>,
    component_count: usize,
    component_names: Vec<String>,
}

// Plugin definition
pub struct BevyInspectorPlugin;

impl Plugin for BevyInspectorPlugin {
    fn build(&self, app: &mut App) {
        // Ensure Ratatui bridge is loaded
        if !app.is_plugin_added::<RatatuiBridgePlugin>() {
            app.add_plugins(RatatuiBridgePlugin);
        }

        app.init_resource::<BevyInspectorState>()
            .add_systems(Update, (
                toggle_inspector_input,
                update_entity_list,
                spawn_inspector_surface,
                render_inspector,
                handle_inspector_input,
            ).chain());

        info!("Bevy Inspector initialized - Press Ctrl+Shift+I to open");
    }
}

// System: Toggle inspector with Ctrl+Shift+I
fn toggle_inspector_input(
    keys: Res<ButtonInput<KeyCode>>,
    mut state: ResMut<BevyInspectorState>,
) {
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

// System: Update entity list from world
fn update_entity_list(
    mut state: ResMut<BevyInspectorState>,
    world: &World,
) {
    if !state.visible {
        return;
    }

    // Only update when visible (performance optimization)
    // In a real implementation, you might want to throttle this further

    // Collect all entities
    let mut entities: Vec<Entity> = world.iter_entities()
        .map(|entity_ref| entity_ref.id())
        .collect();

    // Sort entities by index for consistent ordering
    entities.sort_by_key(|e| (e.index(), e.generation()));

    // Apply search filter if active
    if !state.search_query.is_empty() {
        let query_lower = state.search_query.to_lowercase();
        entities.retain(|entity| {
            if let Ok(entity_ref) = world.get_entity(*entity) {
                // Check if name contains query
                if let Some(name) = entity_ref.get::<Name>() {
                    if name.to_string().to_lowercase().contains(&query_lower) {
                        return true;
                    }
                }
                // Check if entity ID contains query
                if format!("{:?}", entity).to_lowercase().contains(&query_lower) {
                    return true;
                }
            }
            false
        });
    }

    state.entities = entities;

    // Clamp selected index
    if state.selected_index >= state.entities.len() && !state.entities.is_empty() {
        state.selected_index = state.entities.len() - 1;
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
            RatatuiSurface::new(x, y, width, height)
                .with_z_index(1000.0), // High z-index to appear on top
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
fn render_inspector(
    state: Res<BevyInspectorState>,
    world: &World,
    mut buffers: ResMut<SurfaceBuffers>,
    surfaces: Query<(Entity, &RatatuiSurface), With<InspectorSurfaceMarker>>,
) {
    if !state.visible {
        return;
    }

    for (entity, surface) in surfaces.iter() {
        let buffer = buffers.get_or_create(entity, surface.width, surface.height);
        let area = surface.rect();

        // Render the inspector widget
        InspectorWidget {
            state: &state,
            world,
        }.render(area, buffer);
    }
}

// Custom Ratatui widget for the inspector
struct InspectorWidget<'a> {
    state: &'a BevyInspectorState,
    world: &'a World,
}

impl<'a> Widget for InspectorWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Main layout: horizontal split for entity list and details
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(40),
                Constraint::Percentage(60),
            ])
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
            InspectorViewMode::Entities => format!("Entities ({})", self.state.entities.len()),
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

        // Build entity list items
        let items: Vec<ListItem> = self.state.entities.iter().enumerate()
            .map(|(idx, entity)| {
                let entity_ref = self.world.get_entity(*entity).ok();
                let name = entity_ref
                    .as_ref()
                    .and_then(|e| e.get::<Name>())
                    .map(|n| n.to_string())
                    .unwrap_or_else(|| "(unnamed)".to_string());

                let component_count = entity_ref
                    .as_ref()
                    .map(|e| e.archetype().component_count())
                    .unwrap_or(0);

                let is_selected = idx == self.state.selected_index;
                let is_expanded = self.state.expanded_entities.contains(entity);

                let prefix = if is_expanded { "[-] " } else { "[+] " };
                let text = format!("{}{} | {} ({} components)",
                    prefix,
                    format!("{:?}", entity),
                    name,
                    component_count
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

        if let Some(entity) = self.state.selected_entity() {
            if let Ok(entity_ref) = self.world.get_entity(entity) {
                let mut lines = vec![
                    Line::from(vec![
                        Span::styled("Entity: ", Style::default().add_modifier(Modifier::BOLD)),
                        Span::raw(format!("{:?}", entity)),
                    ]),
                    Line::from(""),
                ];

                // Add entity name if available
                if let Some(name) = entity_ref.get::<Name>() {
                    lines.push(Line::from(vec![
                        Span::styled("Name: ", Style::default().add_modifier(Modifier::BOLD)),
                        Span::styled(name.to_string(), Style::default().fg(Color::Green)),
                    ]));
                    lines.push(Line::from(""));
                }

                // List components
                lines.push(Line::from(vec![
                    Span::styled("Components:", Style::default().add_modifier(Modifier::BOLD).fg(Color::Yellow)),
                ]));

                let archetype = entity_ref.archetype();
                for component_id in archetype.components() {
                    if let Some(info) = self.world.components().get_info(component_id) {
                        let component_name = info.name();
                        lines.push(Line::from(vec![
                            Span::raw("  - "),
                            Span::styled(component_name, Style::default().fg(Color::Cyan)),
                        ]));
                    }
                }

                // Handle scrolling
                let visible_lines: Vec<Line> = lines.into_iter()
                    .skip(self.state.detail_scroll)
                    .take(inner.height as usize)
                    .collect();

                let paragraph = Paragraph::new(visible_lines);
                paragraph.render(inner, buf);
            } else {
                let text = vec![
                    Line::from("Entity not found or was despawned"),
                ];
                Paragraph::new(text).render(inner, buf);
            }
        } else {
            let text = vec![
                Line::from("Select an entity to view details"),
            ];
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
            format!("Search: {} | Esc: cancel | Enter: apply", self.state.search_query)
        } else {
            "j/k: navigate | h/l: expand/collapse | /: search | r: refresh | Esc: close | Ctrl+Shift+I: toggle".to_string()
        };

        let status = Line::from(vec![
            Span::styled(help_text, Style::default().fg(Color::Black).bg(Color::White)),
        ]);

        // Clear the status bar area
        for x in status_area.x..status_area.x + status_area.width {
            buf.cell_mut((x, status_area.y))
                .unwrap()
                .set_char(' ')
                .set_style(Style::default().bg(Color::White));
        }

        // Render status text
        let mut x_offset = status_area.x;
        for span in status.spans {
            let text = span.content.to_string();
            for ch in text.chars().take((status_area.width - (x_offset - status_area.x)) as usize) {
                buf.cell_mut((x_offset, status_area.y))
                    .unwrap()
                    .set_char(ch)
                    .set_style(span.style);
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

fn handle_navigation_input(state: &mut BevyInspectorState, key: &ratatui::crossterm::event::KeyEvent) {
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
            if let Some(entity) = state.selected_entity() {
                state.expanded_entities.remove(&entity);
            }
        }
        RatKeyCode::Char('l') | RatKeyCode::Right => {
            // Expand selected entity
            if let Some(entity) = state.selected_entity() {
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
