//! Plugin Marketplace UI Module
//!
//! Provides a comprehensive plugin marketplace interface using the Ratatui bridge.
//! Users can browse, search, filter, and install plugins from the registry.
//!
//! # Architecture
//!
//! The marketplace consists of several components:
//! - `overlay.rs` - Main marketplace surface and layout
//! - `plugin_card.rs` - Individual plugin display cards
//! - `search.rs` - Search input and filtering logic
//! - `installer.rs` - Installation progress UI
//!
//! # Usage
//!
//! The marketplace is triggered via keybindings (Ctrl+Shift+M) or command palette.
//! It overlays the terminal with a full-screen interface showing available plugins.
//!
//! # Keybindings
//!
//! - `/` or `Ctrl+F` - Focus search bar
//! - Tab - Switch between categories
//! - Up/Down - Navigate plugin list
//! - Enter - Install/update selected plugin
//! - `d` - View plugin details
//! - `u` - Uninstall selected plugin
//! - `r` - Refresh plugin list
//! - `q` or Escape - Close marketplace
//!
//! # State Management
//!
//! The marketplace uses Bevy resources and components to track:
//! - Search query and active filters
//! - Scroll position and selected plugin
//! - Fetched plugin data from registry
//! - Active installation progress

mod installer;
mod overlay;
mod plugin_card;
mod search;

pub use installer::{
    render_install_progress, update_install_progress, InstallProgress, InstallStatus,
};
pub use overlay::{
    close_marketplace, handle_marketplace_input, open_marketplace, render_marketplace,
    toggle_marketplace, MarketplaceOverlay, MarketplaceState, MarketplaceView,
};
pub use plugin_card::{format_plugin_card, PluginCardStyle};
pub use search::{apply_filters, format_search_bar, update_search, SearchState};

use bevy::prelude::*;
use scarab_config::registry::types::PluginEntry;

/// Plugin for marketplace functionality
pub struct MarketplacePlugin;

impl Plugin for MarketplacePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MarketplaceState>()
            .init_resource::<PluginListCache>()
            .init_resource::<InstallProgress>()
            .add_event::<MarketplaceEvent>()
            .add_event::<InstallPluginEvent>()
            .add_systems(Startup, initialize_marketplace)
            .add_systems(
                Update,
                (
                    handle_marketplace_input,
                    render_marketplace,
                    update_search,
                    update_install_progress,
                    render_install_progress,
                    handle_install_events,
                    fetch_plugin_list,
                )
                    .chain(),
            );

        info!("MarketplacePlugin initialized");
    }
}

/// Cache of fetched plugin data
#[derive(Resource, Default)]
pub struct PluginListCache {
    /// All available plugins from registry
    pub plugins: Vec<PluginEntry>,
    /// Last fetch timestamp
    pub last_fetch: Option<u64>,
    /// Whether a fetch is in progress
    pub fetching: bool,
    /// Fetch error if any
    pub error: Option<String>,
}

/// Events for marketplace actions
#[derive(Event, Debug, Clone)]
pub enum MarketplaceEvent {
    /// Open the marketplace
    Open,
    /// Close the marketplace
    Close,
    /// Toggle marketplace visibility
    Toggle,
    /// Refresh plugin list from registry
    Refresh,
    /// Select plugin by index
    SelectPlugin(usize),
    /// View plugin details
    ViewDetails(String),
    /// Search query updated
    SearchUpdated(String),
    /// Filter category changed
    CategoryChanged(String),
}

/// Event to trigger plugin installation
#[derive(Event, Debug, Clone)]
pub struct InstallPluginEvent {
    /// Plugin name
    pub name: String,
    /// Optional specific version (defaults to latest)
    pub version: Option<String>,
    /// Whether this is an update operation
    pub is_update: bool,
}

/// Initialize marketplace resources
fn initialize_marketplace(mut commands: Commands) {
    info!("Initializing marketplace UI");

    // Create hidden marketplace surface
    // Full terminal coverage minus borders
    commands.spawn((
        super::ratatui_bridge::RatatuiSurface::new(0, 0, 200, 100)
            .with_z_index(250.0)
            .hidden(),
        MarketplaceOverlay,
    ));
}

/// Handle plugin installation events
fn handle_install_events(
    mut events: EventReader<InstallPluginEvent>,
    mut install_progress: ResMut<InstallProgress>,
) {
    for event in events.read() {
        info!(
            "Installing plugin: {} (version: {:?}, update: {})",
            event.name, event.version, event.is_update
        );

        install_progress.start_installation(event.name.clone(), event.version.clone());
    }
}

/// Async task to fetch plugin list from registry
fn fetch_plugin_list(
    mut cache: ResMut<PluginListCache>,
    mut events: EventReader<MarketplaceEvent>,
) {
    // Check if refresh requested
    let should_fetch = events.read().any(|e| matches!(e, MarketplaceEvent::Refresh));

    if !should_fetch || cache.fetching {
        return;
    }

    // Mark as fetching
    cache.fetching = true;

    // Spawn async task to fetch plugins
    // In a real implementation, this would use bevy_tokio or similar
    // For now, we'll simulate with a mock list
    info!("Fetching plugin list from registry");

    // TODO: Integrate with async runtime
    // let runtime = tokio::runtime::Runtime::new().unwrap();
    // runtime.block_on(async {
    //     let manager = RegistryManager::new().unwrap();
    //     manager.sync().await.unwrap();
    //     let plugins = manager.search(&PluginFilter::default()).unwrap();
    //     cache.plugins = plugins;
    // });

    // For now, use mock data
    cache.plugins = create_mock_plugins();
    cache.last_fetch = Some(
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
    );
    cache.fetching = false;
    cache.error = None;

    info!("Plugin list fetched successfully: {} plugins", cache.plugins.len());
}

/// Create mock plugin data for testing
fn create_mock_plugins() -> Vec<PluginEntry> {
    use scarab_config::registry::types::{PluginStats, PluginVersion};

    vec![
        PluginEntry {
            name: "git-status".to_string(),
            description: "Display git branch and status in prompt".to_string(),
            readme: Some("Shows current git branch, dirty files, and ahead/behind counts.".to_string()),
            author: "scarab-team".to_string(),
            author_email: Some("team@scarab.dev".to_string()),
            homepage: Some("https://github.com/scarab/plugins/git-status".to_string()),
            repository: Some("https://github.com/scarab/plugins".to_string()),
            license: "MIT".to_string(),
            latest_version: "1.2.0".to_string(),
            versions: vec![
                PluginVersion {
                    version: "1.2.0".to_string(),
                    download_url: "https://registry.scarab.dev/v1/plugins/git-status/1.2.0".to_string(),
                    checksum: "abc123".to_string(),
                    signature: None,
                    changelog: Some("Added stash count display".to_string()),
                    api_version: "1.0".to_string(),
                    min_scarab_version: "0.1.0".to_string(),
                    size: 45120,
                    released_at: 1700000000,
                    prerelease: false,
                },
            ],
            tags: vec!["git".to_string(), "prompt".to_string(), "vcs".to_string()],
            stats: PluginStats {
                downloads: 15240,
                downloads_recent: 892,
                rating: 4.8,
                rating_count: 124,
                stars: Some(456),
            },
            created_at: 1690000000,
            updated_at: 1700000000,
        },
        PluginEntry {
            name: "syntax-highlight".to_string(),
            description: "Syntax highlighting for common file types".to_string(),
            readme: Some("Provides syntax highlighting using tree-sitter grammars.".to_string()),
            author: "syntax-team".to_string(),
            author_email: None,
            homepage: None,
            repository: Some("https://github.com/scarab/syntax-highlight".to_string()),
            license: "Apache-2.0".to_string(),
            latest_version: "2.0.1".to_string(),
            versions: vec![],
            tags: vec!["syntax".to_string(), "highlighting".to_string(), "editor".to_string()],
            stats: PluginStats {
                downloads: 28450,
                downloads_recent: 1523,
                rating: 4.6,
                rating_count: 289,
                stars: Some(823),
            },
            created_at: 1680000000,
            updated_at: 1698000000,
        },
        PluginEntry {
            name: "tmux-integration".to_string(),
            description: "Seamless tmux integration and pane management".to_string(),
            readme: Some("Control tmux sessions and panes directly from Scarab.".to_string()),
            author: "mux-masters".to_string(),
            author_email: Some("mux@example.com".to_string()),
            homepage: Some("https://tmux-scarab.dev".to_string()),
            repository: Some("https://github.com/mux-masters/tmux-integration".to_string()),
            license: "MIT".to_string(),
            latest_version: "0.9.3".to_string(),
            versions: vec![],
            tags: vec!["tmux".to_string(), "multiplexer".to_string(), "session".to_string()],
            stats: PluginStats {
                downloads: 8920,
                downloads_recent: 421,
                rating: 4.2,
                rating_count: 67,
                stars: Some(234),
            },
            created_at: 1685000000,
            updated_at: 1695000000,
        },
        PluginEntry {
            name: "ssh-manager".to_string(),
            description: "Manage SSH connections and keys".to_string(),
            readme: Some("Quick SSH connection manager with key authentication.".to_string()),
            author: "net-tools".to_string(),
            author_email: None,
            homepage: None,
            repository: None,
            license: "GPL-3.0".to_string(),
            latest_version: "1.5.2".to_string(),
            versions: vec![],
            tags: vec!["ssh".to_string(), "network".to_string(), "security".to_string()],
            stats: PluginStats {
                downloads: 12340,
                downloads_recent: 678,
                rating: 4.5,
                rating_count: 156,
                stars: Some(389),
            },
            created_at: 1688000000,
            updated_at: 1702000000,
        },
        PluginEntry {
            name: "kubernetes-ctx".to_string(),
            description: "Display current Kubernetes context in prompt".to_string(),
            readme: Some("Shows active k8s context, namespace, and cluster info.".to_string()),
            author: "cloud-native".to_string(),
            author_email: Some("cloud@k8s.io".to_string()),
            homepage: Some("https://k8s-plugins.dev".to_string()),
            repository: Some("https://github.com/cloud-native/k8s-ctx".to_string()),
            license: "Apache-2.0".to_string(),
            latest_version: "3.1.0".to_string(),
            versions: vec![],
            tags: vec!["kubernetes".to_string(), "k8s".to_string(), "devops".to_string()],
            stats: PluginStats {
                downloads: 19850,
                downloads_recent: 1245,
                rating: 4.9,
                rating_count: 203,
                stars: Some(967),
            },
            created_at: 1692000000,
            updated_at: 1703000000,
        },
    ]
}
