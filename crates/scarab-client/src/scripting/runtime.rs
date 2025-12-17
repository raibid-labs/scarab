//! Script runtime - executes Fusabi scripts using fusabi-host Engine
//!
//! This module provides a full Fusabi runtime integration using fusabi-host,
//! enabling scripts to use the complete F# syntax and call Scarab APIs.

use std::path::Path;
use std::sync::Arc;

use bevy::prelude::*;
use fusabi_host::{Capabilities, Engine, EngineConfig, HostRegistry, Limits};

use super::api::{ScriptContext, ScriptEvent};
use super::error::{ScriptError, ScriptResult};
use super::ecs_bridge::FusabiActionChannel;
use super::host_functions::register_scarab_functions;

/// Runtime for executing Fusabi scripts
pub struct ScriptRuntime {
    engine: Engine,
    event_sender: crossbeam::channel::Sender<ScriptEvent>,
    event_receiver: crossbeam::channel::Receiver<ScriptEvent>,
}

impl ScriptRuntime {
    /// Create a new script runtime with the given action channel
    pub fn new_with_channel(channel: Arc<FusabiActionChannel>) -> ScriptResult<Self> {
        let (tx, rx) = crossbeam::channel::unbounded();

        // Configure the engine with appropriate limits and capabilities
        let config = EngineConfig::default()
            .with_limits(Limits::default())
            .with_capabilities(Capabilities::safe_defaults())
            .with_debug(cfg!(debug_assertions));

        let mut engine = Engine::new(config).map_err(|e| ScriptError::RuntimeError {
            script: "init".to_string(),
            message: format!("Failed to create Fusabi engine: {}", e),
        })?;

        // Register all Scarab host functions
        register_scarab_functions(engine.registry_mut(), channel);

        Ok(Self {
            engine,
            event_sender: tx,
            event_receiver: rx,
        })
    }

    /// Create a new script runtime with default channel (for backwards compatibility)
    pub fn new() -> Self {
        let channel = Arc::new(FusabiActionChannel::new());
        Self::new_with_channel(channel).expect("Failed to create script runtime")
    }

    /// Execute a script file
    pub fn execute_file(&mut self, path: &Path, _context: &ScriptContext) -> ScriptResult<()> {
        let source = std::fs::read_to_string(path).map_err(|e| ScriptError::LoadError {
            path: path.display().to_string(),
            reason: e.to_string(),
        })?;

        self.execute_source(&source, path.display().to_string().as_str(), _context)
    }

    /// Execute script source code
    pub fn execute_source(
        &mut self,
        source: &str,
        script_name: &str,
        _context: &ScriptContext,
    ) -> ScriptResult<()> {
        // Execute the script using the Fusabi engine
        match self.engine.execute(source) {
            Ok(result) => {
                debug!("Script '{}' executed successfully: {}", script_name, result);
                Ok(())
            }
            Err(e) => {
                // Send error event
                let _ = self.event_sender.send(ScriptEvent::Error {
                    script_name: script_name.to_string(),
                    message: e.to_string(),
                });

                Err(ScriptError::RuntimeError {
                    script: script_name.to_string(),
                    message: e.to_string(),
                })
            }
        }
    }

    /// Execute bytecode directly (for pre-compiled scripts)
    pub fn execute_bytecode(&mut self, bytecode: &[u8], script_name: &str) -> ScriptResult<()> {
        match self.engine.execute_bytecode(bytecode) {
            Ok(result) => {
                debug!(
                    "Bytecode '{}' executed successfully: {}",
                    script_name, result
                );
                Ok(())
            }
            Err(e) => Err(ScriptError::RuntimeError {
                script: script_name.to_string(),
                message: e.to_string(),
            }),
        }
    }

    /// Collect pending events from the runtime
    pub fn collect_events(&self) -> Vec<ScriptEvent> {
        let mut events = Vec::new();
        while let Ok(event) = self.event_receiver.try_recv() {
            events.push(event);
        }
        events
    }

    /// Get a reference to the engine's host registry for additional function registration
    pub fn registry_mut(&mut self) -> &mut HostRegistry {
        self.engine.registry_mut()
    }
}

/// Loaded script with metadata
pub struct LoadedScript {
    pub name: String,
    pub path: std::path::PathBuf,
    pub source: String,
    pub last_modified: std::time::SystemTime,
}

impl LoadedScript {
    /// Load a script from a file
    pub fn from_file(path: &Path) -> ScriptResult<Self> {
        let source = std::fs::read_to_string(path).map_err(|e| ScriptError::LoadError {
            path: path.display().to_string(),
            reason: e.to_string(),
        })?;

        let metadata = std::fs::metadata(path).map_err(|e| ScriptError::LoadError {
            path: path.display().to_string(),
            reason: e.to_string(),
        })?;

        let last_modified = metadata.modified().map_err(|e| ScriptError::LoadError {
            path: path.display().to_string(),
            reason: e.to_string(),
        })?;

        let name = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string();

        Ok(Self {
            name,
            path: path.to_path_buf(),
            source,
            last_modified,
        })
    }

    /// Check if the script has been modified since it was loaded
    pub fn is_modified(&self) -> bool {
        if let Ok(metadata) = std::fs::metadata(&self.path) {
            if let Ok(modified) = metadata.modified() {
                return modified > self.last_modified;
            }
        }
        false
    }

    /// Reload the script from disk
    pub fn reload(&mut self) -> ScriptResult<()> {
        let script = Self::from_file(&self.path)?;
        self.source = script.source;
        self.last_modified = script.last_modified;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_runtime_creation() {
        let channel = Arc::new(FusabiActionChannel::new());
        let runtime = ScriptRuntime::new_with_channel(channel);
        assert!(runtime.is_ok());
    }

    #[test]
    fn test_simple_expression() {
        let channel = Arc::new(FusabiActionChannel::new());
        let mut runtime = ScriptRuntime::new_with_channel(channel).unwrap();

        let context = ScriptContext {
            colors: super::super::api::ColorContext {
                foreground: Color::WHITE,
                background: Color::BLACK,
                cursor: Color::WHITE,
                selection_bg: Color::srgba(0.5, 0.5, 0.5, 0.3),
                selection_fg: Color::WHITE,
                palette: vec![Color::BLACK; 16],
            },
            fonts: super::super::api::FontContext {
                family: "JetBrains Mono".to_string(),
                size: 14.0,
                line_height: 1.2,
            },
            window: super::super::api::WindowContext {
                width: 800.0,
                height: 600.0,
                scale_factor: 1.0,
                title: "Test".to_string(),
            },
            terminal: super::super::api::TerminalContext {
                rows: 24,
                cols: 80,
                scrollback_lines: 10000,
            },
        };

        // Test simple expression execution
        let result = runtime.execute_source("42", "test.fsx", &context);
        assert!(result.is_ok());
    }
}
