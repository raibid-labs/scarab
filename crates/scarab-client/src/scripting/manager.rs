//! Script manager - coordinates loading, execution, and hot-reloading
//!
//! This is the central coordinator for the scripting system

use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;

use bevy::prelude::*;

use super::api::ScriptEvent;
use super::context::RuntimeContext;
use super::ecs_bridge::FusabiActionChannel;
use super::error::{ScriptError, ScriptResult};
use super::loader::ScriptLoader;
use super::runtime::{LoadedScript, ScriptRuntime};
use super::watcher::ScriptWatcher;

/// Manages all scripts and their lifecycle
#[derive(Resource)]
pub struct ScriptManager {
    loader: ScriptLoader,
    runtime: ScriptRuntime,
    watcher: ScriptWatcher,
    scripts: HashMap<String, LoadedScript>,
    initialized: bool,
}

impl ScriptManager {
    /// Create a new script manager with the given action channel
    pub fn new_with_channel(
        scripts_directory: std::path::PathBuf,
        channel: Arc<FusabiActionChannel>,
    ) -> Self {
        let runtime = ScriptRuntime::new_with_channel(channel)
            .expect("Failed to create script runtime");

        Self {
            loader: ScriptLoader::new(scripts_directory),
            runtime,
            watcher: ScriptWatcher::new(),
            scripts: HashMap::new(),
            initialized: false,
        }
    }

    /// Create a new script manager (for backwards compatibility)
    pub fn new(scripts_directory: std::path::PathBuf) -> Self {
        let channel = Arc::new(FusabiActionChannel::new());
        Self::new_with_channel(scripts_directory, channel)
    }

    /// Initialize the script manager - load all scripts
    pub fn initialize(&mut self, scripts_directory: &Path) -> ScriptResult<usize> {
        info!(
            "Initializing script manager from: {}",
            scripts_directory.display()
        );

        // Ensure directory exists
        self.loader.ensure_directory()?;

        // Load all scripts
        let loaded_scripts = self.loader.load_all_scripts()?;
        let count = loaded_scripts.len();

        // Register scripts for watching
        for script in loaded_scripts {
            self.watcher.watch(script.path.clone())?;
            self.scripts.insert(script.name.clone(), script);
        }

        self.initialized = true;
        info!("Loaded {} scripts", count);

        Ok(count)
    }

    /// Check for script reloads
    pub fn check_reloads(&mut self) -> ScriptResult<()> {
        if !self.initialized {
            return Ok(());
        }

        let changed_paths = self.watcher.check_changes()?;

        for path in changed_paths {
            // Find the script by path
            let script_name = path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("unknown");

            if let Some(script) = self.scripts.get_mut(script_name) {
                info!("Reloading script: {}", script_name);
                match script.reload() {
                    Ok(_) => {
                        info!("Successfully reloaded: {}", script_name);
                    }
                    Err(e) => {
                        error!("Failed to reload '{}': {}", script_name, e);
                    }
                }
            }
        }

        Ok(())
    }

    /// Execute all loaded scripts
    pub fn execute_all(&mut self, context: &RuntimeContext) -> Vec<ScriptResult<()>> {
        let mut results = Vec::new();

        let script_data: Vec<_> = self.scripts.iter()
            .map(|(name, script)| (name.clone(), script.source.clone()))
            .collect();

        for (name, source) in script_data {
            debug!("Executing script: {}", name);
            let result = self.runtime.execute_source(&source, &name, context.context());
            results.push(result);
        }

        results
    }

    /// Execute a specific script by name
    pub fn execute_script(&mut self, name: &str, context: &RuntimeContext) -> ScriptResult<()> {
        let source = self
            .scripts
            .get(name)
            .ok_or_else(|| ScriptError::ResourceNotFound {
                resource_type: "script".to_string(),
                name: name.to_string(),
            })?
            .source
            .clone();

        self.runtime.execute_source(&source, name, context.context())
    }

    /// Execute scripts on window resize events
    pub fn execute_on_resize(&mut self, context: &RuntimeContext, width: f32, height: f32) {
        debug!("Executing scripts on window resize: {}x{}", width, height);

        let script_data: Vec<_> = self.scripts.iter()
            .filter(|(_, script)| script.source.contains("on_resize") || script.source.contains("window_resize"))
            .map(|(name, script)| (name.clone(), script.source.clone()))
            .collect();

        for (name, source) in script_data {
            debug!("Script '{}' has resize handler", name);
            if let Err(e) = self.runtime.execute_source(&source, &name, context.context()) {
                error!("Failed to execute script '{}' on resize: {}", name, e);
            }
        }
    }

    /// Execute scripts on input events
    pub fn execute_on_input(&mut self, context: &RuntimeContext) {
        debug!("Executing scripts on input event");

        let script_data: Vec<_> = self.scripts.iter()
            .filter(|(_, script)| script.source.contains("on_input") || script.source.contains("keyboard_input"))
            .map(|(name, script)| (name.clone(), script.source.clone()))
            .collect();

        for (name, source) in script_data {
            debug!("Script '{}' has input handler", name);
            if let Err(e) = self.runtime.execute_source(&source, &name, context.context()) {
                error!("Failed to execute script '{}' on input: {}", name, e);
            }
        }
    }

    /// Execute scripts on startup (called once after initialization)
    ///
    /// All scripts are executed on startup. Scripts can check runtime context
    /// or use guards if they only want to run under certain conditions.
    pub fn execute_on_startup(&mut self, context: &RuntimeContext) {
        info!("Executing {} scripts on startup", self.scripts.len());

        let script_data: Vec<_> = self.scripts.iter()
            .map(|(name, script)| (name.clone(), script.source.clone()))
            .collect();

        for (name, source) in script_data {
            info!("Executing script '{}' on startup", name);
            match self.runtime.execute_source(&source, &name, context.context()) {
                Ok(_) => info!("Script '{}' executed successfully", name),
                Err(e) => error!("Failed to execute script '{}' on startup: {}", name, e),
            }
        }
    }

    /// Handle a script event
    pub fn handle_event(&mut self, event: &ScriptEvent) -> ScriptResult<()> {
        // Events are collected from the runtime and processed by Bevy systems
        debug!("Handling script event: {:?}", event);
        Ok(())
    }

    /// Collect events from the runtime
    pub fn collect_events(&self) -> Vec<ScriptEvent> {
        self.runtime.collect_events()
    }

    /// Get the number of loaded scripts
    pub fn script_count(&self) -> usize {
        self.scripts.len()
    }

    /// Get a list of all script names
    pub fn script_names(&self) -> Vec<String> {
        self.scripts.keys().cloned().collect()
    }

    /// Check if a script is loaded
    pub fn has_script(&self, name: &str) -> bool {
        self.scripts.contains_key(name)
    }
}

impl Default for ScriptManager {
    fn default() -> Self {
        // Default to ~/.config/scarab/scripts
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        let scripts_dir = std::path::PathBuf::from(home)
            .join(".config")
            .join("scarab")
            .join("scripts");

        Self::new(scripts_dir)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_manager_creation() {
        let temp_dir = TempDir::new().unwrap();
        let manager = ScriptManager::new(temp_dir.path().to_path_buf());
        assert_eq!(manager.script_count(), 0);
        assert!(!manager.initialized);
    }

    #[test]
    fn test_initialize() {
        let temp_dir = TempDir::new().unwrap();
        let mut manager = ScriptManager::new(temp_dir.path().to_path_buf());

        // Create some test scripts
        fs::write(temp_dir.path().join("script1.fsx"), "// Script 1").unwrap();
        fs::write(temp_dir.path().join("script2.fsx"), "// Script 2").unwrap();

        let count = manager.initialize(temp_dir.path()).unwrap();
        assert_eq!(count, 2);
        assert_eq!(manager.script_count(), 2);
        assert!(manager.initialized);
    }

    #[test]
    fn test_script_names() {
        let temp_dir = TempDir::new().unwrap();
        let mut manager = ScriptManager::new(temp_dir.path().to_path_buf());

        fs::write(temp_dir.path().join("foo.fsx"), "// Foo").unwrap();
        fs::write(temp_dir.path().join("bar.fsx"), "// Bar").unwrap();

        manager.initialize(temp_dir.path()).unwrap();

        let names = manager.script_names();
        assert_eq!(names.len(), 2);
        assert!(names.contains(&"foo".to_string()));
        assert!(names.contains(&"bar".to_string()));
    }

    #[test]
    fn test_has_script() {
        let temp_dir = TempDir::new().unwrap();
        let mut manager = ScriptManager::new(temp_dir.path().to_path_buf());

        fs::write(temp_dir.path().join("test.fsx"), "// Test").unwrap();
        manager.initialize(temp_dir.path()).unwrap();

        assert!(manager.has_script("test"));
        assert!(!manager.has_script("nonexistent"));
    }
}
