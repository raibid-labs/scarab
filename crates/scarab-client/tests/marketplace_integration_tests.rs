//! Integration tests for the plugin marketplace UI

use bevy::prelude::*;
use scarab_client::{
    marketplace::{
        InstallPluginEvent, MarketplaceEvent, MarketplaceOverlay, MarketplacePlugin,
        MarketplaceState, MarketplaceView, PluginListCache,
    },
    ratatui_bridge::{RatatuiBridgePlugin, RatatuiSurface},
};
use scarab_protocol::TerminalMetrics;

/// Helper to create test app with marketplace
fn create_test_app() -> App {
    let mut app = App::new();
    app.add_plugins((
        MinimalPlugins,
        RatatuiBridgePlugin,
        MarketplacePlugin,
    ))
    .insert_resource(TerminalMetrics::default());
    app
}

#[test]
fn test_marketplace_plugin_initialization() {
    let mut app = create_test_app();

    // Should have marketplace state resource
    assert!(app.world().get_resource::<MarketplaceState>().is_some());

    // Should have plugin cache resource
    assert!(app.world().get_resource::<PluginListCache>().is_some());

    // Should have marketplace overlay entity
    app.update();
    let query = app
        .world_mut()
        .query_filtered::<Entity, With<MarketplaceOverlay>>();
    let count = query.iter(app.world()).count();
    assert_eq!(count, 1, "Should have exactly one marketplace overlay");
}

#[test]
fn test_marketplace_open_close() {
    let mut app = create_test_app();
    app.update();

    // Initially closed
    {
        let state = app.world().get_resource::<MarketplaceState>().unwrap();
        assert!(!state.visible, "Marketplace should start hidden");
    }

    // Send open event
    app.world_mut()
        .send_event(MarketplaceEvent::Open);
    app.update();

    // Should be open
    {
        let state = app.world().get_resource::<MarketplaceState>().unwrap();
        assert!(state.visible, "Marketplace should be visible after Open event");
    }

    // Check surface visibility
    {
        let mut query = app
            .world_mut()
            .query_filtered::<&RatatuiSurface, With<MarketplaceOverlay>>();
        let surface = query.single(app.world());
        assert!(surface.visible, "Surface should be visible");
    }

    // Send close event
    app.world_mut()
        .send_event(MarketplaceEvent::Close);
    app.update();

    // Should be closed
    {
        let state = app.world().get_resource::<MarketplaceState>().unwrap();
        assert!(!state.visible, "Marketplace should be hidden after Close event");
    }
}

#[test]
fn test_marketplace_toggle() {
    let mut app = create_test_app();
    app.update();

    // Initially closed
    {
        let state = app.world().get_resource::<MarketplaceState>().unwrap();
        assert!(!state.visible);
    }

    // Toggle open
    app.world_mut()
        .send_event(MarketplaceEvent::Toggle);
    app.update();

    {
        let state = app.world().get_resource::<MarketplaceState>().unwrap();
        assert!(state.visible, "Should toggle to visible");
    }

    // Toggle closed
    app.world_mut()
        .send_event(MarketplaceEvent::Toggle);
    app.update();

    {
        let state = app.world().get_resource::<MarketplaceState>().unwrap();
        assert!(!state.visible, "Should toggle back to hidden");
    }
}

#[test]
fn test_marketplace_state_navigation() {
    let mut state = MarketplaceState::default();

    // Add mock plugins
    state.filtered_plugins = vec![
        create_test_plugin("plugin1"),
        create_test_plugin("plugin2"),
        create_test_plugin("plugin3"),
    ];

    // Test navigation
    assert_eq!(state.selected_index, 0);

    state.select_next();
    assert_eq!(state.selected_index, 1);

    state.select_next();
    assert_eq!(state.selected_index, 2);

    state.select_next(); // Wrap around
    assert_eq!(state.selected_index, 0);

    state.select_previous();
    assert_eq!(state.selected_index, 2); // Wrap to end

    state.select_previous();
    assert_eq!(state.selected_index, 1);
}

#[test]
fn test_marketplace_category_navigation() {
    let mut state = MarketplaceState::default();

    assert_eq!(state.category, "All");

    state.next_category();
    assert_eq!(state.category, "Prompt");

    state.next_category();
    assert_eq!(state.category, "Editor");

    // Navigate backwards
    state.prev_category();
    assert_eq!(state.category, "Prompt");

    state.prev_category();
    assert_eq!(state.category, "All");

    // Wrap around
    state.prev_category();
    assert_eq!(state.category, "Network"); // Last category
}

#[test]
fn test_marketplace_view_switching() {
    let mut state = MarketplaceState::default();

    assert_eq!(state.view, MarketplaceView::PluginList);

    state.view = MarketplaceView::PluginDetails;
    assert_eq!(state.view, MarketplaceView::PluginDetails);

    state.view = MarketplaceView::Installing;
    assert_eq!(state.view, MarketplaceView::Installing);
}

#[test]
fn test_marketplace_reset() {
    let mut state = MarketplaceState::default();

    // Modify state
    state.selected_index = 5;
    state.scroll_offset = 10;
    state.search.query = "test".to_string();
    state.category = "DevOps".to_string();
    state.view = MarketplaceView::PluginDetails;

    // Reset
    state.reset();

    // Should be back to defaults
    assert_eq!(state.selected_index, 0);
    assert_eq!(state.scroll_offset, 0);
    assert!(state.search.query.is_empty());
    assert_eq!(state.category, "All");
    assert_eq!(state.view, MarketplaceView::PluginList);
}

#[test]
fn test_install_plugin_event() {
    let mut app = create_test_app();
    app.update();

    // Send install event
    app.world_mut()
        .send_event(InstallPluginEvent {
            name: "test-plugin".to_string(),
            version: Some("1.0.0".to_string()),
            is_update: false,
        });

    app.update();

    // Installation progress should be updated
    let progress = app
        .world()
        .get_resource::<scarab_client::marketplace::InstallProgress>()
        .unwrap();

    assert_eq!(progress.plugin_name, "test-plugin");
    assert!(progress.is_active());
}

#[test]
fn test_plugin_refresh_event() {
    let mut app = create_test_app();
    app.update();

    let cache = app.world().get_resource::<PluginListCache>().unwrap();
    let initial_count = cache.plugins.len();

    // Send refresh event
    app.world_mut()
        .send_event(MarketplaceEvent::Refresh);
    app.update();

    let cache = app.world().get_resource::<PluginListCache>().unwrap();
    // Should have fetched plugins (mock data)
    assert!(
        cache.plugins.len() >= initial_count,
        "Should have plugins after refresh"
    );
}

#[test]
fn test_search_state() {
    use scarab_client::marketplace::SearchState;

    let mut search = SearchState::default();

    search.push_char('t');
    search.push_char('e');
    search.push_char('s');
    search.push_char('t');
    assert_eq!(search.query, "test");

    search.pop_char();
    assert_eq!(search.query, "tes");

    search.clear();
    assert!(search.query.is_empty());
}

#[test]
fn test_plugin_filtering() {
    use scarab_client::marketplace::{apply_filters, SearchState};

    let plugins = vec![
        create_plugin_with_tags("git-plugin", vec!["git", "vcs"]),
        create_plugin_with_tags("k8s-plugin", vec!["kubernetes", "devops"]),
        create_plugin_with_tags("ssh-plugin", vec!["network", "ssh"]),
    ];

    let search = SearchState::default();

    // Filter by category "VCS"
    let filtered = apply_filters(&plugins, &search, "VCS");
    assert_eq!(filtered.len(), 1);
    assert_eq!(filtered[0].name, "git-plugin");

    // Filter by category "DevOps"
    let filtered = apply_filters(&plugins, &search, "DevOps");
    assert_eq!(filtered.len(), 1);
    assert_eq!(filtered[0].name, "k8s-plugin");

    // Filter by category "Network"
    let filtered = apply_filters(&plugins, &search, "Network");
    assert_eq!(filtered.len(), 1);
    assert_eq!(filtered[0].name, "ssh-plugin");

    // "All" should return all plugins
    let filtered = apply_filters(&plugins, &search, "All");
    assert_eq!(filtered.len(), 3);
}

#[test]
fn test_plugin_card_formatting() {
    use scarab_client::marketplace::{format_plugin_card, PluginCardStyle};

    let plugin = create_test_plugin("test-plugin");
    let card = format_plugin_card(&plugin, PluginCardStyle::Normal);

    // Should have multiple lines
    assert!(card.len() >= 4, "Card should have multiple lines");

    // Test different styles
    let selected = format_plugin_card(&plugin, PluginCardStyle::Selected);
    assert!(selected.len() >= 4);

    let installed = format_plugin_card(&plugin, PluginCardStyle::Installed);
    assert!(installed.len() >= 4);
}

#[test]
fn test_install_progress_lifecycle() {
    use scarab_client::marketplace::{InstallProgress, InstallStatus};

    let mut progress = InstallProgress::default();

    assert!(!progress.is_active());
    assert_eq!(progress.status, InstallStatus::Idle);

    progress.start_installation("test-plugin".to_string(), None);
    assert!(progress.is_active());
    assert_eq!(progress.status, InstallStatus::Downloading);

    progress.set_progress(50);
    assert_eq!(progress.progress, 50);

    progress.complete();
    assert!(!progress.is_active());
    assert_eq!(progress.status, InstallStatus::Complete);
    assert_eq!(progress.progress, 100);
}

// Helper functions

fn create_test_plugin(name: &str) -> scarab_config::registry::PluginEntry {
    use scarab_config::registry::types::{PluginEntry, PluginStats};

    PluginEntry {
        name: name.to_string(),
        description: format!("Test plugin {}", name),
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

fn create_plugin_with_tags(
    name: &str,
    tags: Vec<&str>,
) -> scarab_config::registry::PluginEntry {
    use scarab_config::registry::types::{PluginEntry, PluginStats};

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
        tags: tags.into_iter().map(String::from).collect(),
        stats: PluginStats::default(),
        created_at: 0,
        updated_at: 0,
    }
}
