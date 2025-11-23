//! Plugin lifecycle management and hook dispatch

use scarab_plugin_api::{
    Action, Plugin, PluginConfig, PluginContext, PluginDiscovery, PluginError, PluginInfo, Result,
};
use std::{
    path::{Path, PathBuf},
    sync::Arc,
    time::Duration,
};
use tokio::time::timeout;

mod fusabi_adapter;
use fusabi_adapter::{FusabiBytecodePlugin, FusabiScriptPlugin};

/// Plugin wrapper with failure tracking
struct ManagedPlugin {
    /// The actual plugin instance
    plugin: Box<dyn Plugin>,
    /// Plugin configuration (retained for future use in hot-reload)
    #[allow(dead_code)]
    config: PluginConfig,
    /// Number of consecutive failures
    failure_count: u32,
    /// Whether plugin is currently enabled
    enabled: bool,
    /// Maximum failures before auto-disable
    max_failures: u32,
}

impl ManagedPlugin {
    fn new(plugin: Box<dyn Plugin>, config: PluginConfig) -> Self {
        Self {
            plugin,
            config,
            failure_count: 0,
            enabled: true,
            max_failures: 3,
        }
    }

    /// Record a failure and potentially disable the plugin
    fn record_failure(&mut self) {
        self.failure_count += 1;
        if self.failure_count >= self.max_failures {
            log::error!(
                "Plugin '{}' disabled after {} consecutive failures",
                self.plugin.metadata().name,
                self.failure_count
            );
            self.enabled = false;
        }
    }

    /// Record successful execution
    fn record_success(&mut self) {
        self.failure_count = 0;
    }

    /// Get plugin info
    fn info(&self) -> PluginInfo {
        let meta = self.plugin.metadata();
        PluginInfo {
            name: meta.name.clone(),
            version: meta.version.clone(),
            description: meta.description.clone(),
            author: meta.author.clone(),
            homepage: meta.homepage.clone(),
            api_version: meta.api_version.clone(),
            min_scarab_version: meta.min_scarab_version.clone(),
            enabled: self.enabled,
            failure_count: self.failure_count,
        }
    }
}

/// Plugin manager for loading, managing, and dispatching to plugins
pub struct PluginManager {
    /// Loaded plugins
    plugins: Vec<ManagedPlugin>,
    /// Plugin discovery
    discovery: PluginDiscovery,
    /// Hook execution timeout (milliseconds)
    hook_timeout: Duration,
    /// Plugin context
    context: Arc<PluginContext>,
}

impl PluginManager {
    /// Create new plugin manager
    pub fn new(context: Arc<PluginContext>) -> Self {
        Self {
            plugins: Vec::new(),
            discovery: PluginDiscovery::new(),
            hook_timeout: Duration::from_millis(1000),
            context,
        }
    }

    /// Set hook execution timeout
    pub fn with_timeout(mut self, timeout_ms: u64) -> Self {
        self.hook_timeout = Duration::from_millis(timeout_ms);
        self
    }

    /// Load plugins from configuration file
    pub async fn load_from_config(&mut self, config_path: Option<&Path>) -> Result<usize> {
        let configs = self.discovery.load_config(config_path)?;
        let mut loaded = 0;

        for config in configs {
            if !config.enabled {
                log::info!("Skipping disabled plugin: {}", config.name);
                continue;
            }

            match self.load_plugin_from_config(config).await {
                Ok(_) => loaded += 1,
                Err(e) => log::error!("Failed to load plugin: {}", e),
            }
        }

        log::info!("Loaded {} plugins", loaded);
        Ok(loaded)
    }

    /// Discover and load all plugins from search paths
    pub async fn discover_and_load(&mut self) -> Result<usize> {
        let plugin_files = self.discovery.discover();
        let mut loaded = 0;

        for path in plugin_files {
            // Create minimal config for discovered plugin
            let config = PluginConfig {
                name: path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("unknown")
                    .to_string(),
                path: path.clone(),
                enabled: true,
                config: Default::default(),
            };

            match self.load_plugin_from_config(config).await {
                Ok(_) => loaded += 1,
                Err(e) => log::warn!("Failed to load plugin from {:?}: {}", path, e),
            }
        }

        Ok(loaded)
    }

    /// Load a single plugin from configuration
    async fn load_plugin_from_config(&mut self, config: PluginConfig) -> Result<()> {
        let path = config.expanded_path();

        log::info!("Loading plugin: {} from {:?}", config.name, path);

        // Check if file exists
        if !path.exists() {
            return Err(PluginError::NotFound(format!("{:?}", path)));
        }

        // Load plugin based on file extension
        let plugin: Box<dyn Plugin> = match path.extension().and_then(|e| e.to_str()) {
            Some("fzb") => {
                log::debug!("Loading compiled bytecode plugin: {:?}", path);
                Box::new(FusabiBytecodePlugin::load(&path)?)
            }
            Some("fsx") => {
                log::debug!("Loading script plugin: {:?}", path);
                Box::new(FusabiScriptPlugin::load(&path)?)
            }
            _ => {
                return Err(PluginError::LoadError(format!(
                    "Unsupported plugin format: {:?}",
                    path
                )))
            }
        };

        // Register the loaded plugin
        self.register_plugin(plugin).await
    }

    /// Manually register a plugin
    pub async fn register_plugin(&mut self, mut plugin: Box<dyn Plugin>) -> Result<()> {
        // Clone metadata values we need before calling on_load
        let plugin_name = plugin.metadata().name.clone();
        let plugin_version = plugin.metadata().version.clone();
        let api_version = plugin.metadata().api_version.clone();

        // Check API compatibility
        if !plugin.metadata().is_compatible(scarab_plugin_api::API_VERSION) {
            return Err(PluginError::VersionIncompatible {
                required: scarab_plugin_api::API_VERSION.to_string(),
                actual: api_version,
            });
        }

        log::info!("Registering plugin: {} v{}", plugin_name, plugin_version);

        // Call on_load with timeout
        let mut ctx = (*self.context).clone();
        let timeout_duration = self.hook_timeout;

        // Call on_load directly with timeout
        let load_result = timeout(timeout_duration, plugin.on_load(&mut ctx)).await;

        match load_result {
            Ok(Ok(_)) => {
                let config = PluginConfig {
                    name: plugin_name.clone(),
                    path: PathBuf::new(),
                    enabled: true,
                    config: Default::default(),
                };
                self.plugins.push(ManagedPlugin::new(plugin, config));
                log::info!("Plugin registered successfully: {}", plugin_name);
                Ok(())
            }
            Ok(Err(e)) => {
                log::error!("Failed to initialize plugin: {}", e);
                Err(e)
            }
            Err(_) => {
                let error = PluginError::Timeout(timeout_duration.as_millis() as u64);
                log::error!("Plugin '{}' initialization timed out", plugin_name);
                Err(error)
            }
        }
    }

    /// Dispatch output hook to all enabled plugins
    pub async fn dispatch_output(&mut self, line: &str) -> Result<String> {
        let mut data = line.to_string();

        for managed in &mut self.plugins {
            if !managed.enabled {
                continue;
            }

            let plugin_name = managed.plugin.metadata().name.clone();
            let current_data = data.clone();
            let ctx = self.context.clone();

            // Apply timeout to plugin call
            let result = timeout(
                self.hook_timeout,
                managed.plugin.on_output(&current_data, &ctx)
            ).await;

            match result {
                Ok(Ok(Action::Continue)) => {
                    managed.record_success();
                }
                Ok(Ok(Action::Stop)) => {
                    managed.record_success();
                    break;
                }
                Ok(Ok(Action::Modify(new_data))) => {
                    managed.record_success();
                    data = String::from_utf8(new_data).unwrap_or(data);
                }
                Ok(Err(e)) => {
                    log::error!("Plugin '{}' output hook failed: {}", plugin_name, e);
                    managed.record_failure();
                }
                Err(_) => {
                    log::error!("Plugin '{}' output hook timed out", plugin_name);
                    managed.record_failure();
                }
            }
        }

        Ok(data)
    }

    /// Dispatch input hook to all enabled plugins
    pub async fn dispatch_input(&mut self, input: &[u8]) -> Result<Vec<u8>> {
        let mut data = input.to_vec();

        for managed in &mut self.plugins {
            if !managed.enabled {
                continue;
            }

            let plugin_name = managed.plugin.metadata().name.clone();
            let current_data = data.clone();
            let ctx = self.context.clone();

            let result = timeout(
                self.hook_timeout,
                managed.plugin.on_input(&current_data, &ctx)
            ).await;

            match result {
                Ok(Ok(Action::Continue)) => {
                    managed.record_success();
                }
                Ok(Ok(Action::Stop)) => {
                    managed.record_success();
                    break;
                }
                Ok(Ok(Action::Modify(new_data))) => {
                    managed.record_success();
                    data = new_data;
                }
                Ok(Err(e)) => {
                    log::error!("Plugin '{}' input hook failed: {}", plugin_name, e);
                    managed.record_failure();
                }
                Err(_) => {
                    log::error!("Plugin '{}' input hook timed out", plugin_name);
                    managed.record_failure();
                }
            }
        }

        Ok(data)
    }

    /// Dispatch resize event to all enabled plugins
    pub async fn dispatch_resize(&mut self, cols: u16, rows: u16) -> Result<()> {
        for managed in &mut self.plugins {
            if !managed.enabled {
                continue;
            }

            let plugin_name = managed.plugin.metadata().name.clone();
            let ctx = self.context.clone();

            let result = timeout(
                self.hook_timeout,
                managed.plugin.on_resize(cols, rows, &ctx)
            ).await;

            match result {
                Ok(Ok(_)) => managed.record_success(),
                Ok(Err(e)) => {
                    log::error!("Plugin '{}' resize hook failed: {}", plugin_name, e);
                    managed.record_failure();
                }
                Err(_) => {
                    log::error!("Plugin '{}' resize hook timed out", plugin_name);
                    managed.record_failure();
                }
            }
        }

        Ok(())
    }

    /// Get information about all loaded plugins
    pub fn list_plugins(&self) -> Vec<PluginInfo> {
        self.plugins.iter().map(|p| p.info()).collect()
    }

    /// Get count of enabled plugins
    pub fn enabled_count(&self) -> usize {
        self.plugins.iter().filter(|p| p.enabled).count()
    }

    /// Unload all plugins
    pub async fn unload_all(&mut self) -> Result<()> {
        log::info!("Unloading {} plugins", self.plugins.len());

        for managed in &mut self.plugins {
            let plugin_name = managed.plugin.metadata().name.clone();

            let result = timeout(
                self.hook_timeout,
                managed.plugin.on_unload()
            ).await;

            match result {
                Ok(Ok(_)) => {},
                Ok(Err(e)) => {
                    log::error!("Error unloading plugin '{}': {}", plugin_name, e);
                }
                Err(_) => {
                    log::error!("Plugin '{}' unload timed out", plugin_name);
                }
            }
        }

        self.plugins.clear();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use scarab_plugin_api::context::PluginSharedState;
    use scarab_plugin_api::PluginMetadata;
    use std::sync::Arc;

    // Mock plugin for testing
    struct MockPlugin {
        metadata: PluginMetadata,
        should_panic: bool,
    }

    #[async_trait::async_trait]
    impl Plugin for MockPlugin {
        fn metadata(&self) -> &PluginMetadata {
            &self.metadata
        }

        async fn on_output(&mut self, line: &str, _ctx: &PluginContext) -> Result<Action> {
            if self.should_panic {
                panic!("Intentional panic");
            }
            if line.contains("MODIFY") {
                Ok(Action::Modify(b"MODIFIED".to_vec()))
            } else if line.contains("STOP") {
                Ok(Action::Stop)
            } else {
                Ok(Action::Continue)
            }
        }
    }

    fn create_test_context() -> Arc<PluginContext> {
        let state = Arc::new(parking_lot::Mutex::new(PluginSharedState::new(80, 24)));
        Arc::new(PluginContext::new(
            Default::default(),
            state,
            "test-plugin",
        ))
    }

    #[tokio::test]
    async fn test_plugin_registration() {
        let ctx = create_test_context();
        let mut manager = PluginManager::new(ctx);

        let plugin = Box::new(MockPlugin {
            metadata: PluginMetadata::new("test", "1.0.0", "Test plugin", "Test Author"),
            should_panic: false,
        });

        assert!(manager.register_plugin(plugin).await.is_ok());
        assert_eq!(manager.enabled_count(), 1);
    }

    #[tokio::test]
    async fn test_output_dispatch() {
        let ctx = create_test_context();
        let mut manager = PluginManager::new(ctx);

        let plugin = Box::new(MockPlugin {
            metadata: PluginMetadata::new("test", "1.0.0", "Test plugin", "Test Author"),
            should_panic: false,
        });

        manager.register_plugin(plugin).await.unwrap();

        let result = manager.dispatch_output("test").await.unwrap();
        assert_eq!(result, "test");

        let result = manager.dispatch_output("MODIFY this").await.unwrap();
        assert_eq!(result, "MODIFIED");
    }

    #[tokio::test]
    async fn test_panic_handling() {
        let ctx = create_test_context();
        let mut manager = PluginManager::new(ctx);

        let plugin = Box::new(MockPlugin {
            metadata: PluginMetadata::new("test", "1.0.0", "Test plugin", "Test Author"),
            should_panic: true,
        });

        manager.register_plugin(plugin).await.unwrap();

        // Should not crash, plugin should be disabled after failures
        for _ in 0..5 {
            let _ = manager.dispatch_output("test").await;
        }

        assert_eq!(manager.enabled_count(), 0);
    }

    #[tokio::test]
    async fn test_load_fusabi_bytecode() {
        use std::io::Write;
        use tempfile::NamedTempFile;

        let ctx = create_test_context();
        let mut manager = PluginManager::new(ctx);

        // Create temporary bytecode file
        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(b"FZB\0").unwrap();
        temp_file.write_all(&[0u8; 100]).unwrap();
        temp_file.flush().unwrap();

        let config = PluginConfig {
            name: "test_bytecode".to_string(),
            path: temp_file.path().to_path_buf(),
            enabled: true,
            config: Default::default(),
        };

        let result = manager.load_plugin_from_config(config).await;
        assert!(result.is_ok());
        assert_eq!(manager.enabled_count(), 1);
    }

    #[tokio::test]
    async fn test_load_fusabi_script() {
        use std::io::Write;
        use tempfile::NamedTempFile;

        let ctx = create_test_context();
        let mut manager = PluginManager::new(ctx);

        // Create temporary script file
        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file
            .write_all(b"let greeting = \"Hello, Fusabi!\"")
            .unwrap();
        temp_file.flush().unwrap();

        let config = PluginConfig {
            name: "test_script".to_string(),
            path: temp_file.path().to_path_buf(),
            enabled: true,
            config: Default::default(),
        };

        let result = manager.load_plugin_from_config(config).await;
        assert!(result.is_ok());
        assert_eq!(manager.enabled_count(), 1);
    }

    #[tokio::test]
    async fn test_load_unsupported_format() {
        use std::io::Write;
        use tempfile::NamedTempFile;

        let ctx = create_test_context();
        let mut manager = PluginManager::new(ctx);

        // Create file with unsupported extension
        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(b"invalid content").unwrap();
        temp_file.flush().unwrap();

        let mut path = temp_file.path().to_path_buf();
        path.set_extension("txt");

        let config = PluginConfig {
            name: "test_invalid".to_string(),
            path: path.clone(),
            enabled: true,
            config: Default::default(),
        };

        // Create the file with .txt extension
        std::fs::write(&path, b"invalid content").unwrap();

        let result = manager.load_plugin_from_config(config).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Unsupported"));

        // Clean up
        let _ = std::fs::remove_file(&path);
    }
}
