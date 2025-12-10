//! Search and filtering logic for plugin marketplace

use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use scarab_config::registry::types::{PluginEntry, PluginFilter, SortOrder};

/// Search and filter state
#[derive(Debug, Clone, Default)]
pub struct SearchState {
    /// Current search query
    pub query: String,
    /// Whether search input is focused
    pub focused: bool,
    /// Active sort order
    pub sort: SortOrder,
    /// Minimum rating filter
    pub min_rating: Option<f32>,
    /// Tag filter
    pub tag_filter: Option<String>,
}

impl SearchState {
    /// Clear search state
    pub fn clear(&mut self) {
        self.query.clear();
        self.focused = false;
        self.tag_filter = None;
    }

    /// Add character to search query
    pub fn push_char(&mut self, c: char) {
        self.query.push(c);
    }

    /// Remove last character from search query
    pub fn pop_char(&mut self) {
        self.query.pop();
    }

    /// Set search query
    pub fn set_query(&mut self, query: String) {
        self.query = query;
    }

    /// Build PluginFilter from current state
    pub fn to_filter(&self) -> PluginFilter {
        PluginFilter {
            query: if self.query.is_empty() {
                None
            } else {
                Some(self.query.clone())
            },
            tag: self.tag_filter.clone(),
            author: None,
            min_rating: self.min_rating,
            sort: self.sort,
            limit: None,
        }
    }
}

/// Format search bar for display
pub fn format_search_bar(state: &SearchState) -> Line<'_> {
    let icon = if state.focused { "🔍" } else { "🔎" };
    let query_text = if state.query.is_empty() {
        "Type to search plugins...".to_string()
    } else {
        state.query.clone()
    };

    let query_style = if state.focused {
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::Gray)
    };

    let cursor = if state.focused { "_" } else { "" };

    Line::from(vec![
        Span::styled(" ", Style::default()),
        Span::styled(icon, Style::default().fg(Color::Cyan)),
        Span::styled(" ", Style::default()),
        Span::styled(query_text, query_style),
        Span::styled(cursor, Style::default().fg(Color::Yellow)),
    ])
}

/// Apply filters to plugin list
pub fn apply_filters(
    plugins: &[PluginEntry],
    search: &SearchState,
    category: &str,
) -> Vec<PluginEntry> {
    let mut filtered: Vec<PluginEntry> = plugins
        .iter()
        .filter(|plugin| {
            // Category filter
            if category != "All" && !plugin_matches_category(plugin, category) {
                return false;
            }

            // Search query filter
            if !search.query.is_empty() && !plugin_matches_query(plugin, &search.query) {
                return false;
            }

            // Tag filter
            if let Some(ref tag) = search.tag_filter {
                if !plugin.tags.iter().any(|t| t.eq_ignore_ascii_case(tag)) {
                    return false;
                }
            }

            // Rating filter
            if let Some(min_rating) = search.min_rating {
                if plugin.stats.rating < min_rating {
                    return false;
                }
            }

            true
        })
        .cloned()
        .collect();

    // Apply sorting
    sort_plugins(&mut filtered, search.sort);

    filtered
}

/// Check if plugin matches category
fn plugin_matches_category(plugin: &PluginEntry, category: &str) -> bool {
    match category {
        "All" => true,
        "Prompt" => plugin.tags.iter().any(|t| t.eq_ignore_ascii_case("prompt")),
        "Editor" => plugin.tags.iter().any(|t| {
            t.eq_ignore_ascii_case("editor")
                || t.eq_ignore_ascii_case("syntax")
                || t.eq_ignore_ascii_case("highlighting")
        }),
        "VCS" => plugin.tags.iter().any(|t| {
            t.eq_ignore_ascii_case("git")
                || t.eq_ignore_ascii_case("vcs")
                || t.eq_ignore_ascii_case("version-control")
        }),
        "DevOps" => plugin.tags.iter().any(|t| {
            t.eq_ignore_ascii_case("devops")
                || t.eq_ignore_ascii_case("kubernetes")
                || t.eq_ignore_ascii_case("k8s")
                || t.eq_ignore_ascii_case("docker")
        }),
        "Network" => plugin.tags.iter().any(|t| {
            t.eq_ignore_ascii_case("network")
                || t.eq_ignore_ascii_case("ssh")
                || t.eq_ignore_ascii_case("http")
        }),
        _ => false,
    }
}

/// Check if plugin matches search query
fn plugin_matches_query(plugin: &PluginEntry, query: &str) -> bool {
    let query_lower = query.to_lowercase();

    // Search in name
    if plugin.name.to_lowercase().contains(&query_lower) {
        return true;
    }

    // Search in description
    if plugin.description.to_lowercase().contains(&query_lower) {
        return true;
    }

    // Search in author
    if plugin.author.to_lowercase().contains(&query_lower) {
        return true;
    }

    // Search in tags
    if plugin
        .tags
        .iter()
        .any(|t| t.to_lowercase().contains(&query_lower))
    {
        return true;
    }

    false
}

/// Sort plugins by specified order
fn sort_plugins(plugins: &mut [PluginEntry], sort: SortOrder) {
    match sort {
        SortOrder::Popular => {
            plugins.sort_by(|a, b| b.stats.downloads.cmp(&a.stats.downloads));
        }
        SortOrder::Rating => {
            plugins.sort_by(|a, b| {
                b.stats
                    .rating
                    .partial_cmp(&a.stats.rating)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });
        }
        SortOrder::Recent => {
            plugins.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
        }
        SortOrder::Name => {
            plugins.sort_by(|a, b| a.name.cmp(&b.name));
        }
    }
}

/// Update search state from keyboard input
pub fn update_search(
    mut search: bevy::prelude::ResMut<super::MarketplaceState>,
    cache: bevy::prelude::ResMut<super::PluginListCache>,
    keys: bevy::prelude::Res<bevy::input::ButtonInput<bevy::input::keyboard::KeyCode>>,
) {
    use bevy::input::keyboard::KeyCode;

    if !search.visible || !search.search.focused {
        return;
    }

    // Handle text input
    // Note: In a real implementation, this would use Bevy's text input events
    // For now, we'll handle basic alphanumeric keys

    // Exit search on Escape
    if keys.just_pressed(KeyCode::Escape) {
        search.search.focused = false;
        return;
    }

    // Backspace
    if keys.just_pressed(KeyCode::Backspace) {
        search.search.pop_char();
        update_filtered_list(&mut search, &cache);
        return;
    }

    // Enter to unfocus
    if keys.just_pressed(KeyCode::Enter) {
        search.search.focused = false;
        return;
    }

    // Handle alphanumeric input
    // In a real implementation, this would use proper text input events
    // This is a simplified version for demonstration
    let mut changed = false;

    for key in keys.get_just_pressed() {
        if let Some(c) = keycode_to_char(*key, keys.pressed(KeyCode::ShiftLeft)) {
            search.search.push_char(c);
            changed = true;
        }
    }

    if changed {
        update_filtered_list(&mut search, &cache);
    }
}

/// Convert KeyCode to character (simplified)
fn keycode_to_char(key: bevy::input::keyboard::KeyCode, shift: bool) -> Option<char> {
    use bevy::input::keyboard::KeyCode;

    match key {
        KeyCode::KeyA => Some(if shift { 'A' } else { 'a' }),
        KeyCode::KeyB => Some(if shift { 'B' } else { 'b' }),
        KeyCode::KeyC => Some(if shift { 'C' } else { 'c' }),
        KeyCode::KeyD => Some(if shift { 'D' } else { 'd' }),
        KeyCode::KeyE => Some(if shift { 'E' } else { 'e' }),
        KeyCode::KeyF => Some(if shift { 'F' } else { 'f' }),
        KeyCode::KeyG => Some(if shift { 'G' } else { 'g' }),
        KeyCode::KeyH => Some(if shift { 'H' } else { 'h' }),
        KeyCode::KeyI => Some(if shift { 'I' } else { 'i' }),
        KeyCode::KeyJ => Some(if shift { 'J' } else { 'j' }),
        KeyCode::KeyK => Some(if shift { 'K' } else { 'k' }),
        KeyCode::KeyL => Some(if shift { 'L' } else { 'l' }),
        KeyCode::KeyM => Some(if shift { 'M' } else { 'm' }),
        KeyCode::KeyN => Some(if shift { 'N' } else { 'n' }),
        KeyCode::KeyO => Some(if shift { 'O' } else { 'o' }),
        KeyCode::KeyP => Some(if shift { 'P' } else { 'p' }),
        KeyCode::KeyQ => Some(if shift { 'Q' } else { 'q' }),
        KeyCode::KeyR => Some(if shift { 'R' } else { 'r' }),
        KeyCode::KeyS => Some(if shift { 'S' } else { 's' }),
        KeyCode::KeyT => Some(if shift { 'T' } else { 't' }),
        KeyCode::KeyU => Some(if shift { 'U' } else { 'u' }),
        KeyCode::KeyV => Some(if shift { 'V' } else { 'v' }),
        KeyCode::KeyW => Some(if shift { 'W' } else { 'w' }),
        KeyCode::KeyX => Some(if shift { 'X' } else { 'x' }),
        KeyCode::KeyY => Some(if shift { 'Y' } else { 'y' }),
        KeyCode::KeyZ => Some(if shift { 'Z' } else { 'z' }),
        KeyCode::Digit0 => Some(if shift { ')' } else { '0' }),
        KeyCode::Digit1 => Some(if shift { '!' } else { '1' }),
        KeyCode::Digit2 => Some(if shift { '@' } else { '2' }),
        KeyCode::Digit3 => Some(if shift { '#' } else { '3' }),
        KeyCode::Digit4 => Some(if shift { '$' } else { '4' }),
        KeyCode::Digit5 => Some(if shift { '%' } else { '5' }),
        KeyCode::Digit6 => Some(if shift { '^' } else { '6' }),
        KeyCode::Digit7 => Some(if shift { '&' } else { '7' }),
        KeyCode::Digit8 => Some(if shift { '*' } else { '8' }),
        KeyCode::Digit9 => Some(if shift { '(' } else { '9' }),
        KeyCode::Space => Some(' '),
        KeyCode::Minus => Some(if shift { '_' } else { '-' }),
        _ => None,
    }
}

/// Update filtered plugin list based on current search
fn update_filtered_list(
    state: &mut bevy::prelude::ResMut<super::MarketplaceState>,
    cache: &super::PluginListCache,
) {
    state.filtered_plugins = apply_filters(&cache.plugins, &state.search, &state.category);
    state.selected_index = 0;
    state.scroll_offset = 0;
}

#[cfg(test)]
mod tests {
    use super::*;
    use scarab_config::registry::types::{PluginStats, PluginVersion};

    #[test]
    fn test_search_state() {
        let mut state = SearchState::default();

        state.push_char('t');
        state.push_char('e');
        state.push_char('s');
        state.push_char('t');
        assert_eq!(state.query, "test");

        state.pop_char();
        assert_eq!(state.query, "tes");

        state.clear();
        assert!(state.query.is_empty());
    }

    #[test]
    fn test_plugin_matches_query() {
        let plugin = create_test_plugin();

        assert!(plugin_matches_query(&plugin, "test"));
        assert!(plugin_matches_query(&plugin, "TEST")); // Case insensitive
        assert!(plugin_matches_query(&plugin, "plugin"));
        assert!(plugin_matches_query(&plugin, "author"));
        assert!(!plugin_matches_query(&plugin, "nonexistent"));
    }

    #[test]
    fn test_plugin_matches_category() {
        let mut plugin = create_test_plugin();
        plugin.tags = vec!["git".to_string(), "vcs".to_string()];

        assert!(plugin_matches_category(&plugin, "All"));
        assert!(plugin_matches_category(&plugin, "VCS"));
        assert!(!plugin_matches_category(&plugin, "DevOps"));
    }

    #[test]
    fn test_apply_filters() {
        let plugins = vec![
            create_plugin_with_name_and_downloads("popular", 10000),
            create_plugin_with_name_and_downloads("unpopular", 100),
            create_plugin_with_name_and_downloads("medium", 5000),
        ];

        let mut search = SearchState::default();
        search.sort = SortOrder::Popular;

        let filtered = apply_filters(&plugins, &search, "All");

        // Should be sorted by downloads (descending)
        assert_eq!(filtered[0].name, "popular");
        assert_eq!(filtered[1].name, "medium");
        assert_eq!(filtered[2].name, "unpopular");
    }

    #[test]
    fn test_sort_plugins() {
        let mut plugins = vec![
            create_plugin_with_name_and_downloads("a", 100),
            create_plugin_with_name_and_downloads("c", 300),
            create_plugin_with_name_and_downloads("b", 200),
        ];

        sort_plugins(&mut plugins, SortOrder::Name);
        assert_eq!(plugins[0].name, "a");
        assert_eq!(plugins[1].name, "b");
        assert_eq!(plugins[2].name, "c");

        sort_plugins(&mut plugins, SortOrder::Popular);
        assert_eq!(plugins[0].name, "c");
        assert_eq!(plugins[1].name, "b");
        assert_eq!(plugins[2].name, "a");
    }

    fn create_test_plugin() -> PluginEntry {
        PluginEntry {
            name: "test-plugin".to_string(),
            description: "A test plugin".to_string(),
            readme: None,
            author: "test-author".to_string(),
            author_email: None,
            homepage: None,
            repository: None,
            license: "MIT".to_string(),
            latest_version: "1.0.0".to_string(),
            versions: vec![],
            tags: vec!["test".to_string()],
            stats: PluginStats::default(),
            created_at: 0,
            updated_at: 0,
        }
    }

    fn create_plugin_with_name_and_downloads(name: &str, downloads: u64) -> PluginEntry {
        PluginEntry {
            name: name.to_string(),
            description: format!("Plugin {}", name),
            readme: None,
            author: "author".to_string(),
            author_email: None,
            homepage: None,
            repository: None,
            license: "MIT".to_string(),
            latest_version: "1.0.0".to_string(),
            versions: vec![],
            tags: vec![],
            stats: PluginStats {
                downloads,
                downloads_recent: 0,
                rating: 4.0,
                rating_count: 10,
                stars: None,
            },
            created_at: 0,
            updated_at: 0,
        }
    }
}
