//! Plugin card formatting for marketplace list view

use fusabi_tui_core::{Color, Modifier, Style};
use fusabi_tui_widgets::{Line, Span};
use scarab_config::registry::PluginEntry;

/// Style for plugin card rendering
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PluginCardStyle {
    /// Normal unselected card
    Normal,
    /// Selected/highlighted card
    Selected,
    /// Installed plugin
    Installed,
}

/// Format a plugin as a card for display in the list
pub fn format_plugin_card(plugin: &PluginEntry, style: PluginCardStyle) -> Vec<Line<'_>> {
    let (fg_color, highlight_color, bg_modifier) = match style {
        PluginCardStyle::Normal => (Color::White, Color::Cyan, None),
        PluginCardStyle::Selected => (Color::Black, Color::Yellow, Some(Modifier::REVERSED)),
        PluginCardStyle::Installed => (Color::Green, Color::Cyan, None),
    };

    let mut base_style = Style::default().fg(fg_color);
    if let Some(modifier) = bg_modifier {
        base_style = base_style.add_modifier(modifier);
    }

    // Line 1: Name and version
    let mut name_line = vec![
        Span::styled(
            format!(" {} ", plugin.name),
            base_style.fg(highlight_color).add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            format!("v{} ", plugin.latest_version),
            base_style.fg(Color::Green),
        ),
    ];

    // Add rating stars
    let stars = render_rating(plugin.stats.rating);
    name_line.push(Span::styled(stars, base_style.fg(Color::Yellow)));
    name_line.push(Span::styled(
        format!(" ({:.1})", plugin.stats.rating),
        base_style.fg(Color::DarkGray),
    ));

    // Line 2: Description
    let description_line = vec![Span::styled(
        format!("  {}", truncate(&plugin.description, 80)),
        base_style.fg(Color::DarkGray),
    )];

    // Line 3: Metadata (author, license, downloads)
    let metadata_line = vec![
        Span::styled("  by ", base_style.fg(Color::DarkGray)),
        Span::styled(&plugin.author, base_style.fg(fg_color)),
        Span::styled(" | ", base_style.fg(Color::DarkGray)),
        Span::styled(&plugin.license, base_style.fg(Color::Blue)),
        Span::styled(" | ", base_style.fg(Color::DarkGray)),
        Span::styled(
            format!("{} downloads", format_downloads(plugin.stats.downloads)),
            base_style.fg(Color::Magenta),
        ),
    ];

    // Line 4: Tags
    let tags_line = if !plugin.tags.is_empty() {
        let mut spans = vec![Span::styled("  ", base_style)];
        for (i, tag) in plugin.tags.iter().take(5).enumerate() {
            if i > 0 {
                spans.push(Span::styled(" ", base_style));
            }
            spans.push(Span::styled(
                format!("[{}]", tag),
                base_style.fg(Color::Cyan),
            ));
        }
        spans
    } else {
        vec![Span::styled("", base_style)]
    };

    vec![
        Line::from(name_line),
        Line::from(description_line),
        Line::from(metadata_line),
        Line::from(tags_line),
        Line::from(vec![Span::styled("", base_style)]), // Empty line separator
    ]
}

/// Render rating as stars
fn render_rating(rating: f32) -> String {
    let full_stars = rating.floor() as u32;
    let half_star = (rating - rating.floor()) >= 0.5;
    let empty_stars = 5 - full_stars - if half_star { 1 } else { 0 };

    let mut stars = String::new();
    for _ in 0..full_stars {
        stars.push('★');
    }
    if half_star {
        stars.push('⯨');
    }
    for _ in 0..empty_stars {
        stars.push('☆');
    }
    stars
}

/// Format download count with units
fn format_downloads(count: u64) -> String {
    if count >= 1_000_000 {
        format!("{:.1}M", count as f64 / 1_000_000.0)
    } else if count >= 1_000 {
        format!("{:.1}K", count as f64 / 1_000.0)
    } else {
        count.to_string()
    }
}

/// Truncate string to max length with ellipsis
fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len.saturating_sub(3)])
    }
}

/// Create a compact single-line card format
#[allow(dead_code)]
pub fn format_plugin_card_compact(plugin: &PluginEntry, style: PluginCardStyle) -> Line<'_> {
    let (fg_color, highlight_color, bg_modifier) = match style {
        PluginCardStyle::Normal => (Color::White, Color::Cyan, None),
        PluginCardStyle::Selected => (Color::Black, Color::Yellow, Some(Modifier::REVERSED)),
        PluginCardStyle::Installed => (Color::Green, Color::Cyan, None),
    };

    let mut base_style = Style::default().fg(fg_color);
    if let Some(modifier) = bg_modifier {
        base_style = base_style.add_modifier(modifier);
    }

    let stars = render_rating(plugin.stats.rating);

    Line::from(vec![
        Span::styled(
            format!(" {:20} ", plugin.name),
            base_style.fg(highlight_color).add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            format!("{:8} ", plugin.latest_version),
            base_style.fg(Color::Green),
        ),
        Span::styled(format!("{} ", stars), base_style.fg(Color::Yellow)),
        Span::styled(
            format!("{:40}", truncate(&plugin.description, 40)),
            base_style.fg(Color::DarkGray),
        ),
    ])
}

#[cfg(test)]
mod tests {
    use super::*;
    use scarab_config::registry::types::{PluginStats, PluginVersion};

    #[test]
    fn test_render_rating() {
        assert_eq!(render_rating(5.0), "★★★★★");
        assert_eq!(render_rating(4.5), "★★★★⯨");
        assert_eq!(render_rating(3.0), "★★★☆☆");
        assert_eq!(render_rating(0.0), "☆☆☆☆☆");
    }

    #[test]
    fn test_format_downloads() {
        assert_eq!(format_downloads(42), "42");
        assert_eq!(format_downloads(1_234), "1.2K");
        assert_eq!(format_downloads(15_240), "15.2K");
        assert_eq!(format_downloads(1_234_567), "1.2M");
    }

    #[test]
    fn test_truncate() {
        assert_eq!(truncate("short", 10), "short");
        assert_eq!(truncate("this is a very long string", 10), "this is...");
        assert_eq!(truncate("exact", 5), "exact");
    }

    #[test]
    fn test_format_plugin_card() {
        let plugin = create_test_plugin();
        let card = format_plugin_card(&plugin, PluginCardStyle::Normal);

        // Should have 5 lines (name, description, metadata, tags, empty)
        assert_eq!(card.len(), 5);
    }

    #[test]
    fn test_format_plugin_card_compact() {
        let plugin = create_test_plugin();
        let line = format_plugin_card_compact(&plugin, PluginCardStyle::Selected);

        // Should be a single line with multiple spans
        assert!(!line.spans.is_empty());
    }

    fn create_test_plugin() -> PluginEntry {
        PluginEntry {
            name: "test-plugin".to_string(),
            description: "A test plugin for unit tests".to_string(),
            readme: None,
            author: "test-author".to_string(),
            author_email: Some("test@example.com".to_string()),
            homepage: None,
            repository: None,
            license: "MIT".to_string(),
            latest_version: "1.0.0".to_string(),
            versions: vec![],
            tags: vec!["test".to_string(), "demo".to_string()],
            stats: PluginStats {
                downloads: 1234,
                downloads_recent: 56,
                rating: 4.5,
                rating_count: 10,
                stars: Some(42),
            },
            created_at: 0,
            updated_at: 0,
        }
    }
}
