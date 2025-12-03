//! Context Menu Rendering using Ratatui
//!
//! This module handles the visual rendering of context menus using Ratatui widgets.
//! Menus are displayed as bordered lists with:
//! - Highlighted selection
//! - Disabled item styling
//! - Separator lines
//! - Keyboard shortcuts displayed on the right

use bevy::prelude::*;
use ratatui::{
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Widget},
};

use super::{ContextMenuState, ContextMenuSurface};
use crate::ratatui_bridge::SurfaceBuffers;

/// Marker component for context menu overlay entities
#[derive(Component)]
pub struct ContextMenuOverlay;

/// System to render context menu using Ratatui
pub fn render_context_menu(
    state: Res<ContextMenuState>,
    mut buffers: ResMut<SurfaceBuffers>,
    mut surfaces: Query<(Entity, &mut crate::ratatui_bridge::RatatuiSurface), With<ContextMenuSurface>>,
) {
    let Ok((entity, mut surface)) = surfaces.get_single_mut() else {
        return;
    };

    if !state.is_visible() {
        return;
    }

    let Some(menu) = state.get_menu() else {
        return;
    };

    // Mark surface dirty to trigger re-render
    surface.mark_dirty();

    let buffer = buffers.get_or_create(entity, surface.width, surface.height);
    let area = surface.rect();

    // Clear buffer
    buffer.reset();

    // Build list items
    let items: Vec<ListItem> = menu
        .items
        .iter()
        .enumerate()
        .map(|(i, item)| {
            if item.separator {
                // Render separator
                let sep_line = "─".repeat(surface.width as usize - 2);
                ListItem::new(Line::from(Span::styled(
                    sep_line,
                    Style::default().fg(Color::DarkGray),
                )))
            } else {
                // Determine style based on selection and enabled state
                let is_selected = i == menu.selected_index;
                let base_style = if is_selected {
                    Style::default()
                        .bg(Color::Blue)
                        .fg(Color::White)
                        .add_modifier(Modifier::BOLD)
                } else if !item.enabled {
                    Style::default().fg(Color::DarkGray)
                } else {
                    Style::default()
                };

                // Build spans for the item
                let mut spans = Vec::new();

                // Add selection indicator
                if is_selected {
                    spans.push(Span::styled("> ", base_style));
                } else {
                    spans.push(Span::raw("  "));
                }

                // Add label
                spans.push(Span::styled(&item.label, base_style));

                // Add shortcut if present
                if let Some(shortcut) = &item.shortcut {
                    // Calculate padding to align shortcuts on the right
                    let label_len = item.label.len() + 2; // +2 for selection indicator
                    let shortcut_len = shortcut.len() + 2; // +2 for brackets
                    let available_width = surface.width as usize - 4; // -4 for borders

                    if label_len + shortcut_len < available_width {
                        let padding = available_width - label_len - shortcut_len;
                        spans.push(Span::raw(" ".repeat(padding)));

                        let shortcut_style = if is_selected {
                            base_style
                        } else {
                            Style::default().fg(Color::DarkGray)
                        };
                        spans.push(Span::styled(format!("[{}]", shortcut), shortcut_style));
                    }
                }

                ListItem::new(Line::from(spans))
            }
        })
        .collect();

    // Render list with border
    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Gray))
            .style(Style::default().bg(Color::Black)),
    );

    list.render(area, buffer);

    surface.mark_clean();
}

#[cfg(test)]
mod tests {
    use super::*;
    use scarab_mouse::context_menu::{ContextMenu, MenuItem};
    use scarab_mouse::types::Position;

    #[test]
    fn test_menu_item_rendering() {
        // Test that menu items are properly formatted
        let menu = ContextMenu::standard(Position::new(10, 10), true);

        // Verify we have some items
        assert!(!menu.items.is_empty());

        // Check that shortcuts are present
        let copy_item = menu.get_item("copy");
        assert!(copy_item.is_some());
        assert!(copy_item.unwrap().shortcut.is_some());
    }

    #[test]
    fn test_separator_item() {
        let sep = MenuItem::separator();
        assert!(sep.separator);
        assert!(!sep.enabled);
        assert!(sep.label.is_empty());
    }

    #[test]
    fn test_menu_with_url() {
        let menu = ContextMenu::url_menu(
            Position::new(5, 5),
            "https://example.com".to_string(),
        );

        // Should have URL-specific items
        assert!(menu.get_item("open_url").is_some());
        assert!(menu.get_item("copy_url").is_some());
    }
}
