//! Main marketplace overlay rendering and input handling

use super::{
    format_plugin_card, format_search_bar, InstallProgress, PluginCardStyle,
    PluginListCache, SearchState,
};
use crate::ratatui_bridge::{Buffer, RatatuiSurface, SurfaceBuffers};
use bevy::prelude::*;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Tabs, Widget},
};
use scarab_config::registry::PluginEntry;

/// Marker component for marketplace overlay
#[derive(Component, Debug)]
pub struct MarketplaceOverlay;

/// Current view in marketplace
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MarketplaceView {
    /// List of available plugins
    PluginList,
    /// Details of selected plugin
    PluginDetails,
    /// Installation progress
    Installing,
}

/// Marketplace UI state
#[derive(Resource, Debug, Clone)]
pub struct MarketplaceState {
    /// Whether marketplace is visible
    pub visible: bool,
    /// Current view
    pub view: MarketplaceView,
    /// Search and filter state
    pub search: SearchState,
    /// Selected plugin index
    pub selected_index: usize,
    /// Scroll offset for plugin list
    pub scroll_offset: usize,
    /// Filtered plugin list
    pub filtered_plugins: Vec<PluginEntry>,
    /// Selected category tab
    pub category: String,
    /// Available categories
    pub categories: Vec<String>,
}

impl Default for MarketplaceState {
    fn default() -> Self {
        Self {
            visible: false,
            view: MarketplaceView::PluginList,
            search: SearchState::default(),
            selected_index: 0,
            scroll_offset: 0,
            filtered_plugins: Vec::new(),
            category: "All".to_string(),
            categories: vec![
                "All".to_string(),
                "Prompt".to_string(),
                "Editor".to_string(),
                "VCS".to_string(),
                "DevOps".to_string(),
                "Network".to_string(),
            ],
        }
    }
}

impl MarketplaceState {
    /// Reset state when closing
    pub fn reset(&mut self) {
        self.selected_index = 0;
        self.scroll_offset = 0;
        self.search.clear();
        self.category = "All".to_string();
        self.view = MarketplaceView::PluginList;
    }

    /// Get selected plugin if available
    pub fn selected_plugin(&self) -> Option<&PluginEntry> {
        self.filtered_plugins.get(self.selected_index)
    }

    /// Navigate to next plugin
    pub fn select_next(&mut self) {
        if !self.filtered_plugins.is_empty() {
            self.selected_index = (self.selected_index + 1) % self.filtered_plugins.len();
            self.adjust_scroll();
        }
    }

    /// Navigate to previous plugin
    pub fn select_previous(&mut self) {
        if !self.filtered_plugins.is_empty() {
            self.selected_index = if self.selected_index == 0 {
                self.filtered_plugins.len() - 1
            } else {
                self.selected_index - 1
            };
            self.adjust_scroll();
        }
    }

    /// Adjust scroll offset to keep selected item visible
    fn adjust_scroll(&mut self) {
        const VISIBLE_ITEMS: usize = 10; // Approximate visible items

        if self.selected_index < self.scroll_offset {
            self.scroll_offset = self.selected_index;
        } else if self.selected_index >= self.scroll_offset + VISIBLE_ITEMS {
            self.scroll_offset = self.selected_index.saturating_sub(VISIBLE_ITEMS - 1);
        }
    }

    /// Select next category
    pub fn next_category(&mut self) {
        let current = self.categories.iter().position(|c| c == &self.category).unwrap_or(0);
        let next = (current + 1) % self.categories.len();
        self.category = self.categories[next].clone();
    }

    /// Select previous category
    pub fn prev_category(&mut self) {
        let current = self.categories.iter().position(|c| c == &self.category).unwrap_or(0);
        let prev = if current == 0 {
            self.categories.len() - 1
        } else {
            current - 1
        };
        self.category = self.categories[prev].clone();
    }
}

/// Open marketplace overlay
pub fn open_marketplace(
    mut state: ResMut<MarketplaceState>,
    mut query: Query<&mut RatatuiSurface, With<MarketplaceOverlay>>,
    mut events: EventWriter<super::MarketplaceEvent>,
) {
    if state.visible {
        return;
    }

    info!("Opening plugin marketplace");
    state.visible = true;

    // Show surface
    if let Ok(mut surface) = query.get_single_mut() {
        surface.show();
    }

    // Trigger refresh if no plugins loaded
    events.send(super::MarketplaceEvent::Refresh);
}

/// Close marketplace overlay
pub fn close_marketplace(
    mut state: ResMut<MarketplaceState>,
    mut query: Query<&mut RatatuiSurface, With<MarketplaceOverlay>>,
) {
    if !state.visible {
        return;
    }

    info!("Closing plugin marketplace");
    state.visible = false;
    state.reset();

    // Hide surface
    if let Ok(mut surface) = query.get_single_mut() {
        surface.hide();
    }
}

/// Toggle marketplace visibility
pub fn toggle_marketplace(
    state: Res<MarketplaceState>,
    mut open_events: EventWriter<super::MarketplaceEvent>,
) {
    if state.visible {
        open_events.send(super::MarketplaceEvent::Close);
    } else {
        open_events.send(super::MarketplaceEvent::Open);
    }
}

/// Handle marketplace input events
pub fn handle_marketplace_input(
    mut state: ResMut<MarketplaceState>,
    keys: Res<ButtonInput<KeyCode>>,
    mut install_events: EventWriter<super::InstallPluginEvent>,
    mut marketplace_events: EventWriter<super::MarketplaceEvent>,
) {
    if !state.visible {
        return;
    }

    // Handle Ctrl+Shift+M to toggle
    if keys.pressed(KeyCode::ControlLeft) && keys.pressed(KeyCode::ShiftLeft) && keys.just_pressed(KeyCode::KeyM) {
        marketplace_events.send(super::MarketplaceEvent::Close);
        return;
    }

    // Close on Escape or 'q'
    if keys.just_pressed(KeyCode::Escape) || keys.just_pressed(KeyCode::KeyQ) {
        marketplace_events.send(super::MarketplaceEvent::Close);
        return;
    }

    match state.view {
        MarketplaceView::PluginList => {
            // Navigation
            if keys.just_pressed(KeyCode::ArrowDown) || keys.just_pressed(KeyCode::KeyJ) {
                state.select_next();
            }
            if keys.just_pressed(KeyCode::ArrowUp) || keys.just_pressed(KeyCode::KeyK) {
                state.select_previous();
            }

            // Category navigation
            if keys.just_pressed(KeyCode::Tab) {
                state.next_category();
                marketplace_events.send(super::MarketplaceEvent::CategoryChanged(state.category.clone()));
            }
            if keys.pressed(KeyCode::ShiftLeft) && keys.just_pressed(KeyCode::Tab) {
                state.prev_category();
                marketplace_events.send(super::MarketplaceEvent::CategoryChanged(state.category.clone()));
            }

            // Search focus
            if keys.just_pressed(KeyCode::Slash) || (keys.pressed(KeyCode::ControlLeft) && keys.just_pressed(KeyCode::KeyF)) {
                state.search.focused = true;
            }

            // Actions
            if keys.just_pressed(KeyCode::Enter) {
                if let Some(plugin) = state.selected_plugin() {
                    install_events.send(super::InstallPluginEvent {
                        name: plugin.name.clone(),
                        version: None,
                        is_update: false,
                    });
                    state.view = MarketplaceView::Installing;
                }
            }

            if keys.just_pressed(KeyCode::KeyD) {
                if state.selected_plugin().is_some() {
                    state.view = MarketplaceView::PluginDetails;
                }
            }

            if keys.just_pressed(KeyCode::KeyR) {
                marketplace_events.send(super::MarketplaceEvent::Refresh);
            }
        }
        MarketplaceView::PluginDetails => {
            // Back to list
            if keys.just_pressed(KeyCode::Escape) || keys.just_pressed(KeyCode::KeyQ) || keys.just_pressed(KeyCode::Backspace) {
                state.view = MarketplaceView::PluginList;
            }

            // Install from details
            if keys.just_pressed(KeyCode::Enter) || keys.just_pressed(KeyCode::KeyI) {
                if let Some(plugin) = state.selected_plugin() {
                    install_events.send(super::InstallPluginEvent {
                        name: plugin.name.clone(),
                        version: None,
                        is_update: false,
                    });
                    state.view = MarketplaceView::Installing;
                }
            }
        }
        MarketplaceView::Installing => {
            // Back to list when done
            if keys.just_pressed(KeyCode::Escape) || keys.just_pressed(KeyCode::Enter) {
                state.view = MarketplaceView::PluginList;
            }
        }
    }
}

/// Render marketplace UI
pub fn render_marketplace(
    mut buffers: ResMut<SurfaceBuffers>,
    mut query: Query<(Entity, &mut RatatuiSurface), With<MarketplaceOverlay>>,
    state: Res<MarketplaceState>,
    cache: Res<PluginListCache>,
    install_progress: Res<InstallProgress>,
) {
    if !state.visible {
        return;
    }

    let Ok((entity, mut surface)) = query.get_single_mut() else {
        return;
    };

    if !surface.dirty && !state.is_changed() && !cache.is_changed() {
        return;
    }

    let buffer = buffers.get_or_create(entity, surface.width, surface.height);

    // Render based on current view
    match state.view {
        MarketplaceView::PluginList => {
            render_plugin_list(buffer, &state, &cache);
        }
        MarketplaceView::PluginDetails => {
            render_plugin_details(buffer, &state);
        }
        MarketplaceView::Installing => {
            render_installation(buffer, &install_progress);
        }
    }

    surface.mark_clean();
}

/// Render the plugin list view
fn render_plugin_list(buffer: &mut Buffer, state: &MarketplaceState, cache: &PluginListCache) {
    let area = Rect::new(0, 0, buffer.area().width, buffer.area().height);

    // Create layout: header | categories | search | plugin list | footer
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header
            Constraint::Length(3), // Categories
            Constraint::Length(3), // Search
            Constraint::Min(10),   // Plugin list
            Constraint::Length(2), // Footer
        ])
        .split(area);

    // Render header
    render_header(buffer, chunks[0]);

    // Render category tabs
    render_categories(buffer, chunks[1], state);

    // Render search bar
    let search_text = format_search_bar(&state.search);
    let search_paragraph = Paragraph::new(search_text)
        .block(Block::default().borders(Borders::ALL).border_style(
            if state.search.focused {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default().fg(Color::Gray)
            }
        ));
    search_paragraph.render(chunks[2], buffer);

    // Render plugin list
    render_plugins(buffer, chunks[3], state, cache);

    // Render footer with keybindings
    render_footer(buffer, chunks[4]);
}

/// Render marketplace header
fn render_header(buffer: &mut Buffer, area: Rect) {
    let title = vec![
        Line::from(vec![
            Span::styled("Scarab Plugin Marketplace", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::styled("Browse, search, and install plugins", Style::default().fg(Color::Gray)),
        ]),
    ];

    let header = Paragraph::new(title)
        .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(Color::Cyan)));

    header.render(area, buffer);
}

/// Render category tabs
fn render_categories(buffer: &mut Buffer, area: Rect, state: &MarketplaceState) {
    let selected = state.categories.iter().position(|c| c == &state.category).unwrap_or(0);

    let tabs = Tabs::new(state.categories.clone())
        .select(selected)
        .block(Block::default().borders(Borders::ALL).title("Categories"))
        .style(Style::default().fg(Color::White))
        .highlight_style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD));

    tabs.render(area, buffer);
}

/// Render plugin list
fn render_plugins(buffer: &mut Buffer, area: Rect, state: &MarketplaceState, cache: &PluginListCache) {
    let plugins = &state.filtered_plugins;

    if cache.fetching {
        let loading = Paragraph::new("Fetching plugins from registry...")
            .block(Block::default().borders(Borders::ALL).title("Plugins"))
            .style(Style::default().fg(Color::Yellow));
        loading.render(area, buffer);
        return;
    }

    if let Some(err) = &cache.error {
        let error = Paragraph::new(format!("Error: {}", err))
            .block(Block::default().borders(Borders::ALL).title("Plugins"))
            .style(Style::default().fg(Color::Red));
        error.render(area, buffer);
        return;
    }

    if plugins.is_empty() {
        let empty = Paragraph::new("No plugins found. Try adjusting your search or filters.")
            .block(Block::default().borders(Borders::ALL).title("Plugins"))
            .style(Style::default().fg(Color::Gray));
        empty.render(area, buffer);
        return;
    }

    // Create list items
    let items: Vec<ListItem> = plugins
        .iter()
        .enumerate()
        .skip(state.scroll_offset)
        .take(area.height.saturating_sub(2) as usize)
        .map(|(idx, plugin)| {
            let is_selected = idx == state.selected_index;
            let card = format_plugin_card(
                plugin,
                if is_selected {
                    PluginCardStyle::Selected
                } else {
                    PluginCardStyle::Normal
                },
            );
            ListItem::new(card)
        })
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!("Plugins ({}/{})", state.selected_index + 1, plugins.len()))
        );

    list.render(area, buffer);
}

/// Render footer with keybindings
fn render_footer(buffer: &mut Buffer, area: Rect) {
    let keybindings = Line::from(vec![
        Span::styled("↑↓", Style::default().fg(Color::Yellow)),
        Span::raw(" Navigate  "),
        Span::styled("Tab", Style::default().fg(Color::Yellow)),
        Span::raw(" Categories  "),
        Span::styled("/", Style::default().fg(Color::Yellow)),
        Span::raw(" Search  "),
        Span::styled("Enter", Style::default().fg(Color::Yellow)),
        Span::raw(" Install  "),
        Span::styled("d", Style::default().fg(Color::Yellow)),
        Span::raw(" Details  "),
        Span::styled("r", Style::default().fg(Color::Yellow)),
        Span::raw(" Refresh  "),
        Span::styled("q", Style::default().fg(Color::Yellow)),
        Span::raw(" Close"),
    ]);

    let footer = Paragraph::new(keybindings)
        .block(Block::default().borders(Borders::TOP));

    footer.render(area, buffer);
}

/// Render plugin details view
fn render_plugin_details(buffer: &mut Buffer, state: &MarketplaceState) {
    let area = Rect::new(0, 0, buffer.area().width, buffer.area().height);

    let Some(plugin) = state.selected_plugin() else {
        return;
    };

    // Create detailed view
    let details = vec![
        Line::from(vec![
            Span::styled(&plugin.name, Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::raw("  v"),
            Span::styled(&plugin.latest_version, Style::default().fg(Color::Green)),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled("Description:", Style::default().add_modifier(Modifier::BOLD))]),
        Line::from(plugin.description.as_str()),
        Line::from(""),
        Line::from(vec![Span::styled("Author:", Style::default().add_modifier(Modifier::BOLD))]),
        Line::from(plugin.author.as_str()),
        Line::from(""),
        Line::from(vec![Span::styled("License:", Style::default().add_modifier(Modifier::BOLD))]),
        Line::from(plugin.license.as_str()),
        Line::from(""),
        Line::from(vec![Span::styled("Statistics:", Style::default().add_modifier(Modifier::BOLD))]),
        Line::from(format!(
            "Downloads: {}  Recent: {}  Rating: {:.1}/5.0 ({} reviews)",
            format_number(plugin.stats.downloads),
            format_number(plugin.stats.downloads_recent),
            plugin.stats.rating,
            plugin.stats.rating_count
        )),
        Line::from(""),
        Line::from(vec![Span::styled("Tags:", Style::default().add_modifier(Modifier::BOLD))]),
        Line::from(plugin.tags.join(", ")),
    ];

    let paragraph = Paragraph::new(details)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Plugin Details")
                .border_style(Style::default().fg(Color::Cyan))
        );

    paragraph.render(area, buffer);

    // Render footer
    let footer_area = Rect::new(0, area.height.saturating_sub(2), area.width, 2);
    let footer = Line::from(vec![
        Span::styled("Enter", Style::default().fg(Color::Yellow)),
        Span::raw(" Install  "),
        Span::styled("Esc", Style::default().fg(Color::Yellow)),
        Span::raw(" Back"),
    ]);
    let footer_para = Paragraph::new(footer).block(Block::default().borders(Borders::TOP));
    footer_para.render(footer_area, buffer);
}

/// Render installation progress view
fn render_installation(buffer: &mut Buffer, progress: &InstallProgress) {
    let area = Rect::new(0, 0, buffer.area().width, buffer.area().height);

    let message = match progress.status {
        super::InstallStatus::Idle => "No installation in progress".to_string(),
        super::InstallStatus::Downloading => format!("Downloading {}...", progress.plugin_name),
        super::InstallStatus::Verifying => format!("Verifying {}...", progress.plugin_name),
        super::InstallStatus::Installing => format!("Installing {}...", progress.plugin_name),
        super::InstallStatus::Complete => format!("Successfully installed {}", progress.plugin_name),
        super::InstallStatus::Failed(ref err) => format!("Failed to install {}: {}", progress.plugin_name, err),
    };

    let color = match progress.status {
        super::InstallStatus::Complete => Color::Green,
        super::InstallStatus::Failed(_) => Color::Red,
        _ => Color::Yellow,
    };

    let paragraph = Paragraph::new(message)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Installation")
                .border_style(Style::default().fg(color))
        )
        .style(Style::default().fg(color));

    paragraph.render(area, buffer);
}

/// Format large numbers with commas
fn format_number(n: u64) -> String {
    let s = n.to_string();
    let mut result = String::new();
    for (i, c) in s.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            result.push(',');
        }
        result.push(c);
    }
    result.chars().rev().collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_marketplace_state_navigation() {
        let mut state = MarketplaceState::default();

        // Add some mock plugins
        state.filtered_plugins = vec![
            create_test_plugin("plugin1"),
            create_test_plugin("plugin2"),
            create_test_plugin("plugin3"),
        ];

        assert_eq!(state.selected_index, 0);

        state.select_next();
        assert_eq!(state.selected_index, 1);

        state.select_next();
        assert_eq!(state.selected_index, 2);

        state.select_next(); // Wrap around
        assert_eq!(state.selected_index, 0);

        state.select_previous();
        assert_eq!(state.selected_index, 2);
    }

    #[test]
    fn test_category_navigation() {
        let mut state = MarketplaceState::default();

        assert_eq!(state.category, "All");

        state.next_category();
        assert_eq!(state.category, "Prompt");

        state.prev_category();
        assert_eq!(state.category, "All");

        state.prev_category();
        assert_eq!(state.category, "Network"); // Wraps to last
    }

    #[test]
    fn test_format_number() {
        assert_eq!(format_number(1234), "1,234");
        assert_eq!(format_number(1234567), "1,234,567");
        assert_eq!(format_number(42), "42");
    }

    fn create_test_plugin(name: &str) -> PluginEntry {
        use scarab_config::registry::{PluginStats, PluginVersion};

        PluginEntry {
            name: name.to_string(),
            description: format!("Test plugin {}", name),
            readme: None,
            author: "test".to_string(),
            author_email: None,
            homepage: None,
            repository: None,
            license: "MIT".to_string(),
            latest_version: "1.0.0".to_string(),
            versions: vec![],
            tags: vec![],
            stats: PluginStats::default(),
            created_at: 0,
            updated_at: 0,
        }
    }
}
