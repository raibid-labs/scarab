//! Plugin registry resource and related components
//!
//! Tracks all registered plugins and their associated resources (overlays,
//! status items, keybindings).

use crate::events::StatusSide;
use bevy::prelude::*;
use std::collections::HashMap;

/// Information about a registered plugin
///
/// Each plugin is uniquely identified by its ID and can own multiple
/// resources (overlays, status items, keybindings).
#[derive(Clone, Debug)]
pub struct RegisteredPlugin {
    /// Unique identifier for this plugin
    pub id: String,
    /// Human-readable plugin name
    pub name: String,
    /// Plugin version string (e.g., "1.0.0")
    pub version: String,
    /// Whether the plugin is currently enabled
    ///
    /// Disabled plugins cannot perform actions and their resources
    /// may be hidden.
    pub enabled: bool,
    /// IDs of overlays owned by this plugin
    pub overlay_ids: Vec<u64>,
    /// IDs of status items owned by this plugin
    pub status_item_ids: Vec<u64>,
    /// Action IDs of keybindings registered by this plugin
    pub keybinding_ids: Vec<String>,
}

/// Resource tracking all registered plugins
///
/// This is the central registry for plugin management. It tracks plugin
/// metadata, enables/disables plugins, and manages resource allocation.
#[derive(Resource, Default)]
pub struct PluginRegistry {
    /// Map of plugin ID -> plugin metadata
    pub plugins: HashMap<String, RegisteredPlugin>,
    /// Next overlay ID to allocate
    next_overlay_id: u64,
    /// Next status item ID to allocate
    next_status_item_id: u64,
}

impl PluginRegistry {
    /// Create a new empty plugin registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a new plugin
    ///
    /// If a plugin with the same ID already exists, it will be replaced.
    ///
    /// # Arguments
    /// - `id`: Unique plugin identifier (e.g., "github.com/user/myplugin")
    /// - `name`: Human-readable name (e.g., "My Plugin")
    /// - `version`: Semantic version string (e.g., "1.0.0")
    ///
    /// # Returns
    /// Reference to the newly registered plugin
    pub fn register(&mut self, id: String, name: String, version: String) {
        info!(
            plugin_id = %id,
            plugin_name = %name,
            version = %version,
            "Registering plugin"
        );

        self.plugins.insert(
            id.clone(),
            RegisteredPlugin {
                id,
                name,
                version,
                enabled: true,
                overlay_ids: Vec::new(),
                status_item_ids: Vec::new(),
                keybinding_ids: Vec::new(),
            },
        );
    }

    /// Unregister a plugin and clean up its resources
    ///
    /// Returns the plugin metadata if it was registered, or None if
    /// the plugin was not found.
    ///
    /// # Arguments
    /// - `id`: Plugin ID to unregister
    ///
    /// # Returns
    /// The removed plugin's metadata, if it existed
    pub fn unregister(&mut self, id: &str) -> Option<RegisteredPlugin> {
        info!(plugin_id = %id, "Unregistering plugin");
        self.plugins.remove(id)
    }

    /// Get the next available overlay ID
    ///
    /// This is an atomic counter that ensures overlay IDs are unique
    /// across all plugins.
    pub fn next_overlay_id(&mut self) -> u64 {
        let id = self.next_overlay_id;
        self.next_overlay_id = self.next_overlay_id.wrapping_add(1);
        id
    }

    /// Get the next available status item ID
    ///
    /// This is an atomic counter that ensures status item IDs are unique
    /// across all plugins.
    pub fn next_status_item_id(&mut self) -> u64 {
        let id = self.next_status_item_id;
        self.next_status_item_id = self.next_status_item_id.wrapping_add(1);
        id
    }

    /// Check if a plugin is enabled
    ///
    /// Returns false if the plugin is not registered or is disabled.
    ///
    /// # Arguments
    /// - `id`: Plugin ID to check
    pub fn is_enabled(&self, id: &str) -> bool {
        self.plugins.get(id).map_or(false, |p| p.enabled)
    }

    /// Enable or disable a plugin
    ///
    /// Disabled plugins cannot perform actions. Their existing resources
    /// (overlays, status items) remain but may be hidden.
    ///
    /// # Arguments
    /// - `id`: Plugin ID
    /// - `enabled`: Whether to enable (true) or disable (false)
    pub fn set_enabled(&mut self, id: &str, enabled: bool) {
        if let Some(plugin) = self.plugins.get_mut(id) {
            info!(
                plugin_id = %id,
                enabled = enabled,
                "Plugin enabled state changed"
            );
            plugin.enabled = enabled;
        } else {
            warn!(plugin_id = %id, "Attempted to set enabled state for non-existent plugin");
        }
    }

    /// Get a reference to a registered plugin
    ///
    /// # Arguments
    /// - `id`: Plugin ID
    ///
    /// # Returns
    /// Reference to the plugin metadata, or None if not found
    pub fn get(&self, id: &str) -> Option<&RegisteredPlugin> {
        self.plugins.get(id)
    }

    /// Get a mutable reference to a registered plugin
    ///
    /// # Arguments
    /// - `id`: Plugin ID
    ///
    /// # Returns
    /// Mutable reference to the plugin metadata, or None if not found
    pub fn get_mut(&mut self, id: &str) -> Option<&mut RegisteredPlugin> {
        self.plugins.get_mut(id)
    }

    /// Add an overlay ID to a plugin's tracked resources
    ///
    /// # Arguments
    /// - `plugin_id`: Plugin that owns the overlay
    /// - `overlay_id`: Overlay ID to track
    pub fn add_overlay(&mut self, plugin_id: &str, overlay_id: u64) {
        if let Some(plugin) = self.plugins.get_mut(plugin_id) {
            plugin.overlay_ids.push(overlay_id);
            debug!(
                plugin_id = %plugin_id,
                overlay_id = overlay_id,
                "Overlay added to plugin"
            );
        }
    }

    /// Remove an overlay ID from a plugin's tracked resources
    ///
    /// # Arguments
    /// - `plugin_id`: Plugin that owns the overlay
    /// - `overlay_id`: Overlay ID to remove
    pub fn remove_overlay(&mut self, plugin_id: &str, overlay_id: u64) {
        if let Some(plugin) = self.plugins.get_mut(plugin_id) {
            plugin.overlay_ids.retain(|&id| id != overlay_id);
            debug!(
                plugin_id = %plugin_id,
                overlay_id = overlay_id,
                "Overlay removed from plugin"
            );
        }
    }

    /// Add a status item ID to a plugin's tracked resources
    ///
    /// # Arguments
    /// - `plugin_id`: Plugin that owns the status item
    /// - `item_id`: Status item ID to track
    pub fn add_status_item(&mut self, plugin_id: &str, item_id: u64) {
        if let Some(plugin) = self.plugins.get_mut(plugin_id) {
            plugin.status_item_ids.push(item_id);
            debug!(
                plugin_id = %plugin_id,
                item_id = item_id,
                "Status item added to plugin"
            );
        }
    }

    /// Remove a status item ID from a plugin's tracked resources
    ///
    /// # Arguments
    /// - `plugin_id`: Plugin that owns the status item
    /// - `item_id`: Status item ID to remove
    pub fn remove_status_item(&mut self, plugin_id: &str, item_id: u64) {
        if let Some(plugin) = self.plugins.get_mut(plugin_id) {
            plugin.status_item_ids.retain(|&id| id != item_id);
            debug!(
                plugin_id = %plugin_id,
                item_id = item_id,
                "Status item removed from plugin"
            );
        }
    }

    /// Add a keybinding action ID to a plugin's tracked resources
    ///
    /// # Arguments
    /// - `plugin_id`: Plugin that registered the keybinding
    /// - `action_id`: Action ID to track
    pub fn add_keybinding(&mut self, plugin_id: &str, action_id: String) {
        if let Some(plugin) = self.plugins.get_mut(plugin_id) {
            plugin.keybinding_ids.push(action_id.clone());
            debug!(
                plugin_id = %plugin_id,
                action_id = %action_id,
                "Keybinding added to plugin"
            );
        }
    }

    /// Remove a keybinding action ID from a plugin's tracked resources
    ///
    /// # Arguments
    /// - `plugin_id`: Plugin that registered the keybinding
    /// - `action_id`: Action ID to remove
    pub fn remove_keybinding(&mut self, plugin_id: &str, action_id: &str) {
        if let Some(plugin) = self.plugins.get_mut(plugin_id) {
            plugin.keybinding_ids.retain(|id| id != action_id);
            debug!(
                plugin_id = %plugin_id,
                action_id = %action_id,
                "Keybinding removed from plugin"
            );
        }
    }

    /// Get the total number of registered plugins
    pub fn len(&self) -> usize {
        self.plugins.len()
    }

    /// Check if the registry is empty
    pub fn is_empty(&self) -> bool {
        self.plugins.is_empty()
    }
}

/// Component marking an overlay as owned by a plugin
///
/// Attached to overlay entities spawned by plugins for tracking and cleanup.
#[derive(Component, Clone, Debug)]
pub struct PluginOverlay {
    /// Plugin that owns this overlay
    pub plugin_id: String,
    /// Unique ID for this overlay
    pub overlay_id: u64,
}

/// Component marking a notification as spawned by a plugin
///
/// Notifications auto-expire based on the `expires_at` timestamp.
#[derive(Component, Clone, Debug)]
pub struct PluginNotification {
    /// Plugin that spawned this notification
    pub plugin_id: String,
    /// Timestamp (in seconds since app start) when this notification expires
    pub expires_at: f64,
}

/// Component marking a status bar item as owned by a plugin
///
/// Status items are persistent UI elements in the status bar.
#[derive(Component, Clone, Debug)]
pub struct PluginStatusItem {
    /// Plugin that owns this status item
    pub plugin_id: String,
    /// Unique ID for this status item
    pub item_id: u64,
    /// Which side of the status bar to display on
    pub side: StatusSide,
    /// Sort priority within the side (higher = more prominent)
    pub priority: i32,
    /// Text content to display
    pub content: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plugin_registry_register() {
        let mut registry = PluginRegistry::new();
        assert!(registry.is_empty());

        registry.register(
            "test.plugin".to_string(),
            "Test Plugin".to_string(),
            "1.0.0".to_string(),
        );

        assert_eq!(registry.len(), 1);
        assert!(registry.is_enabled("test.plugin"));
    }

    #[test]
    fn test_plugin_registry_unregister() {
        let mut registry = PluginRegistry::new();
        registry.register(
            "test.plugin".to_string(),
            "Test Plugin".to_string(),
            "1.0.0".to_string(),
        );

        let removed = registry.unregister("test.plugin");
        assert!(removed.is_some());
        assert!(registry.is_empty());

        // Unregistering non-existent plugin returns None
        let removed = registry.unregister("nonexistent");
        assert!(removed.is_none());
    }

    #[test]
    fn test_plugin_registry_enabled_state() {
        let mut registry = PluginRegistry::new();
        registry.register(
            "test.plugin".to_string(),
            "Test Plugin".to_string(),
            "1.0.0".to_string(),
        );

        assert!(registry.is_enabled("test.plugin"));

        registry.set_enabled("test.plugin", false);
        assert!(!registry.is_enabled("test.plugin"));

        registry.set_enabled("test.plugin", true);
        assert!(registry.is_enabled("test.plugin"));
    }

    #[test]
    fn test_overlay_id_generation() {
        let mut registry = PluginRegistry::new();

        let id1 = registry.next_overlay_id();
        let id2 = registry.next_overlay_id();
        let id3 = registry.next_overlay_id();

        assert_eq!(id1, 0);
        assert_eq!(id2, 1);
        assert_eq!(id3, 2);
    }

    #[test]
    fn test_status_item_id_generation() {
        let mut registry = PluginRegistry::new();

        let id1 = registry.next_status_item_id();
        let id2 = registry.next_status_item_id();
        let id3 = registry.next_status_item_id();

        assert_eq!(id1, 0);
        assert_eq!(id2, 1);
        assert_eq!(id3, 2);
    }

    #[test]
    fn test_overlay_tracking() {
        let mut registry = PluginRegistry::new();
        registry.register(
            "test.plugin".to_string(),
            "Test Plugin".to_string(),
            "1.0.0".to_string(),
        );

        registry.add_overlay("test.plugin", 100);
        registry.add_overlay("test.plugin", 101);

        let plugin = registry.get("test.plugin").unwrap();
        assert_eq!(plugin.overlay_ids.len(), 2);
        assert!(plugin.overlay_ids.contains(&100));
        assert!(plugin.overlay_ids.contains(&101));

        registry.remove_overlay("test.plugin", 100);
        let plugin = registry.get("test.plugin").unwrap();
        assert_eq!(plugin.overlay_ids.len(), 1);
        assert!(!plugin.overlay_ids.contains(&100));
        assert!(plugin.overlay_ids.contains(&101));
    }

    #[test]
    fn test_status_item_tracking() {
        let mut registry = PluginRegistry::new();
        registry.register(
            "test.plugin".to_string(),
            "Test Plugin".to_string(),
            "1.0.0".to_string(),
        );

        registry.add_status_item("test.plugin", 42);
        registry.add_status_item("test.plugin", 43);

        let plugin = registry.get("test.plugin").unwrap();
        assert_eq!(plugin.status_item_ids.len(), 2);

        registry.remove_status_item("test.plugin", 42);
        let plugin = registry.get("test.plugin").unwrap();
        assert_eq!(plugin.status_item_ids.len(), 1);
    }

    #[test]
    fn test_keybinding_tracking() {
        let mut registry = PluginRegistry::new();
        registry.register(
            "test.plugin".to_string(),
            "Test Plugin".to_string(),
            "1.0.0".to_string(),
        );

        registry.add_keybinding("test.plugin", "action1".to_string());
        registry.add_keybinding("test.plugin", "action2".to_string());

        let plugin = registry.get("test.plugin").unwrap();
        assert_eq!(plugin.keybinding_ids.len(), 2);

        registry.remove_keybinding("test.plugin", "action1");
        let plugin = registry.get("test.plugin").unwrap();
        assert_eq!(plugin.keybinding_ids.len(), 1);
    }

    #[test]
    fn test_plugin_get() {
        let mut registry = PluginRegistry::new();
        registry.register(
            "test.plugin".to_string(),
            "Test Plugin".to_string(),
            "1.0.0".to_string(),
        );

        let plugin = registry.get("test.plugin");
        assert!(plugin.is_some());
        assert_eq!(plugin.unwrap().name, "Test Plugin");

        let nonexistent = registry.get("nonexistent");
        assert!(nonexistent.is_none());
    }

    #[test]
    fn test_plugin_get_mut() {
        let mut registry = PluginRegistry::new();
        registry.register(
            "test.plugin".to_string(),
            "Test Plugin".to_string(),
            "1.0.0".to_string(),
        );

        if let Some(plugin) = registry.get_mut("test.plugin") {
            plugin.name = "Modified Plugin".to_string();
        }

        let plugin = registry.get("test.plugin").unwrap();
        assert_eq!(plugin.name, "Modified Plugin");
    }
}
