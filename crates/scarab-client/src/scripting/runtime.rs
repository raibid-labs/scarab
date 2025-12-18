//! Script runtime - executes Fusabi scripts using fusabi-frontend + fusabi-vm
//!
//! This module provides a full Fusabi runtime integration that compiles
//! F# source code to bytecode and executes it with Scarab host functions.

use std::collections::HashMap;
use std::path::Path;
use std::sync::{Arc, Mutex};

use bevy::prelude::*;
use fusabi_frontend::compile_program_from_source;
use fusabi_vm::{Value, Vm};

use super::api::{ScriptContext, ScriptEvent};
use super::ecs_bridge::FusabiActionChannel;
use super::error::{ScriptError, ScriptResult};

/// Runtime for executing Fusabi scripts
pub struct ScriptRuntime {
    channel: Arc<FusabiActionChannel>,
    event_sender: crossbeam::channel::Sender<ScriptEvent>,
    event_receiver: crossbeam::channel::Receiver<ScriptEvent>,
}

impl ScriptRuntime {
    /// Create a new script runtime with the given action channel
    pub fn new_with_channel(channel: Arc<FusabiActionChannel>) -> ScriptResult<Self> {
        let (tx, rx) = crossbeam::channel::unbounded();

        Ok(Self {
            channel,
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
        // Compile source to bytecode using fusabi-frontend
        let chunk = match compile_program_from_source(source) {
            Ok(chunk) => chunk,
            Err(e) => {
                let msg = format!("{}", e);
                // Send error event
                let _ = self.event_sender.send(ScriptEvent::Error {
                    script_name: script_name.to_string(),
                    message: msg.clone(),
                });

                return Err(ScriptError::RuntimeError {
                    script: script_name.to_string(),
                    message: msg,
                });
            }
        };

        // Create VM and register host functions
        let mut vm = Vm::new();
        fusabi_vm::stdlib::register_stdlib(&mut vm);

        // Register Scarab functions and create Scarab module
        register_scarab_module(&mut vm, self.channel.clone());

        // Execute the bytecode
        match vm.execute(chunk) {
            Ok(result) => {
                debug!("Script '{}' executed successfully: {}", script_name, result);
                Ok(())
            }
            Err(e) => {
                let msg = format!("{}", e);
                // Send error event
                let _ = self.event_sender.send(ScriptEvent::Error {
                    script_name: script_name.to_string(),
                    message: msg.clone(),
                });

                Err(ScriptError::RuntimeError {
                    script: script_name.to_string(),
                    message: msg,
                })
            }
        }
    }

    /// Execute bytecode directly (for pre-compiled scripts)
    pub fn execute_bytecode(&mut self, bytecode: &[u8], script_name: &str) -> ScriptResult<()> {
        let chunk = fusabi_vm::deserialize_chunk(bytecode).map_err(|e| ScriptError::RuntimeError {
            script: script_name.to_string(),
            message: format!("Failed to deserialize bytecode: {}", e),
        })?;

        let mut vm = Vm::new();
        fusabi_vm::stdlib::register_stdlib(&mut vm);

        // Register Scarab functions and create Scarab module
        register_scarab_module(&mut vm, self.channel.clone());

        match vm.execute(chunk) {
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
}

/// Register Scarab functions in VM's host registry and as direct globals
fn register_scarab_module(vm: &mut Vm, channel: Arc<FusabiActionChannel>) {
    use crate::events::{NotificationLevel, PluginAction, StatusSide};

    let default_plugin_id = "script".to_string();

    // Helper to create NativeFn value
    let native = |name: &str, arity: u8| Value::NativeFn {
        name: name.to_string(),
        arity,
        args: vec![],
    };

    // Register functions in host_registry
    {
        let mut registry = vm.host_registry.lock().unwrap();

        // status_add(side, content, priority)
        {
            let channel = channel.clone();
            let plugin_id = default_plugin_id.clone();
            registry.register("Scarab.status_add", move |_vm, args| {
                let side = args.first().and_then(|v| v.as_str()).unwrap_or("right");
                let content = args.get(1).and_then(|v| v.as_str()).unwrap_or("").to_string();
                let priority = args.get(2).and_then(|v| v.as_int()).unwrap_or(0) as i32;

                let side = match side {
                    "left" => StatusSide::Left,
                    _ => StatusSide::Right,
                };

                info!(
                    plugin_id = %plugin_id,
                    side = ?side,
                    content = %content,
                    priority = priority,
                    "Scarab.status_add called from script"
                );

                channel.send_action(PluginAction::AddStatusItem {
                    plugin_id: plugin_id.clone(),
                    side,
                    content,
                    priority,
                });

                Ok(Value::Unit)
            });
        }

        // Scarab.status_remove(item_id)
        {
            let channel = channel.clone();
            let plugin_id = default_plugin_id.clone();
            registry.register("Scarab.status_remove", move |_vm, args| {
                let item_id = args.first().and_then(|v| v.as_int()).unwrap_or(0) as u64;
                channel.send_action(PluginAction::RemoveStatusItem {
                    plugin_id: plugin_id.clone(),
                    item_id,
                });
                Ok(Value::Unit)
            });
        }

        // Scarab.notify(title, message, level)
        {
            let channel = channel.clone();
            let plugin_id = default_plugin_id.clone();
            registry.register("Scarab.notify", move |_vm, args| {
                let title = args.first().and_then(|v| v.as_str()).unwrap_or("Notification").to_string();
                let message = args.get(1).and_then(|v| v.as_str()).unwrap_or("").to_string();
                let level_str = args.get(2).and_then(|v| v.as_str()).unwrap_or("info");

                let level = match level_str {
                    "info" => NotificationLevel::Info,
                    "warning" => NotificationLevel::Warning,
                    "error" => NotificationLevel::Error,
                    "success" => NotificationLevel::Success,
                    _ => NotificationLevel::Info,
                };

                info!(plugin_id = %plugin_id, title = %title, "Scarab.notify called from script");

                channel.send_action(PluginAction::ShowNotification {
                    plugin_id: plugin_id.clone(),
                    title,
                    message,
                    level,
                    duration_ms: 5000,
                });

                Ok(Value::Unit)
            });
        }

        // Scarab.spawn_overlay(x, y, width, height, content, z_index)
        {
            let channel = channel.clone();
            let plugin_id = default_plugin_id.clone();
            registry.register("Scarab.spawn_overlay", move |_vm, args| {
                let x = args.first().and_then(|v| v.as_int()).unwrap_or(0) as u16;
                let y = args.get(1).and_then(|v| v.as_int()).unwrap_or(0) as u16;
                let width = args.get(2).and_then(|v| v.as_int()).unwrap_or(10) as u16;
                let height = args.get(3).and_then(|v| v.as_int()).unwrap_or(5) as u16;
                let content = args.get(4).and_then(|v| v.as_str()).unwrap_or("").to_string();
                let z_index = args.get(5).and_then(|v| v.as_float()).unwrap_or(1.0) as f32;

                channel.send_action(PluginAction::SpawnOverlay {
                    plugin_id: plugin_id.clone(),
                    x, y, width, height, content, z_index,
                });

                Ok(Value::Unit)
            });
        }

        // Scarab.despawn_overlay(overlay_id)
        {
            let channel = channel.clone();
            let plugin_id = default_plugin_id.clone();
            registry.register("Scarab.despawn_overlay", move |_vm, args| {
                let overlay_id = args.first().and_then(|v| v.as_int()).unwrap_or(0) as u64;
                channel.send_action(PluginAction::DespawnOverlay {
                    plugin_id: plugin_id.clone(),
                    overlay_id,
                });
                Ok(Value::Unit)
            });
        }

        // Scarab.send_input(data)
        {
            let channel = channel.clone();
            let plugin_id = default_plugin_id.clone();
            registry.register("Scarab.send_input", move |_vm, args| {
                let data = args.first().and_then(|v| v.as_str()).unwrap_or("").as_bytes().to_vec();
                channel.send_action(PluginAction::SendInput {
                    plugin_id: plugin_id.clone(),
                    data,
                });
                Ok(Value::Unit)
            });
        }

        // Scarab.get_terminal_rows(start_row, end_row)
        {
            let channel = channel.clone();
            let plugin_id = default_plugin_id.clone();
            registry.register("Scarab.get_terminal_rows", move |_vm, args| {
                let start_row = args.first().and_then(|v| v.as_int()).unwrap_or(0) as u16;
                let end_row = args.get(1).and_then(|v| v.as_int()).unwrap_or(24) as u16;
                channel.send_action(PluginAction::RequestTerminalContent {
                    plugin_id: plugin_id.clone(),
                    start_row,
                    end_row,
                });
                Ok(Value::Unit)
            });
        }

        // Scarab.log(level, message)
        registry.register("Scarab.log", move |_vm, args| {
            let level = args.first().and_then(|v| v.as_str()).unwrap_or("info");
            let message = args.get(1).and_then(|v| v.as_str()).unwrap_or("");

            match level {
                "debug" => debug!("[Script] {}", message),
                "info" => info!("[Script] {}", message),
                "warn" => warn!("[Script] {}", message),
                "error" => error!("[Script] {}", message),
                _ => info!("[Script] {}", message),
            }

            Ok(Value::Unit)
        });

        // Scarab.version()
        registry.register("Scarab.version", move |_vm, _args| {
            Ok(Value::Str(env!("CARGO_PKG_VERSION").to_string()))
        });

        // Scarab.setColor(name, hex) - for theming
        {
            let channel = channel.clone();
            let plugin_id = default_plugin_id.clone();
            registry.register("Scarab.setColor", move |_vm, args| {
                let color_name = args.first().and_then(|v| v.as_str()).unwrap_or("").to_string();
                let hex_color = args.get(1).and_then(|v| v.as_str()).unwrap_or("#ffffff").to_string();

                debug!(plugin_id = %plugin_id, color = %color_name, hex = %hex_color, "Scarab.setColor called");

                channel.send_action(PluginAction::UpdateTheme {
                    plugin_id: plugin_id.clone(),
                    color_name,
                    color_value: hex_color,
                });

                Ok(Value::Unit)
            });
        }

        // Scarab.setWindowTitle(title) - for window customization
        {
            let channel = channel.clone();
            let plugin_id = default_plugin_id.clone();
            registry.register("Scarab.setWindowTitle", move |_vm, args| {
                let title = args.first().and_then(|v| v.as_str()).unwrap_or("Scarab").to_string();

                debug!(plugin_id = %plugin_id, title = %title, "Scarab.setWindowTitle called");

                channel.send_action(PluginAction::SetWindowTitle {
                    plugin_id: plugin_id.clone(),
                    title,
                });

                Ok(Value::Unit)
            });
        }

        // Scarab.setFont(family, size) - for font customization
        {
            let channel = channel.clone();
            let plugin_id = default_plugin_id.clone();
            registry.register("Scarab.setFont", move |_vm, args| {
                let family = args.first().and_then(|v| v.as_str()).unwrap_or("JetBrains Mono").to_string();
                let size = args.get(1).and_then(|v| v.as_float()).unwrap_or(14.0) as f32;

                debug!(plugin_id = %plugin_id, family = %family, size = size, "Scarab.setFont called");

                channel.send_action(PluginAction::SetFont {
                    plugin_id: plugin_id.clone(),
                    family,
                    size,
                });

                Ok(Value::Unit)
            });
        }
    }

    // Register functions as direct globals (like print, printfn in stdlib)
    // This allows scripts to call functions directly: status_add "left" "text" 100
    vm.globals.insert("status_add".to_string(), native("Scarab.status_add", 3));
    vm.globals.insert("status_remove".to_string(), native("Scarab.status_remove", 1));
    vm.globals.insert("notify".to_string(), native("Scarab.notify", 3));
    vm.globals.insert("spawn_overlay".to_string(), native("Scarab.spawn_overlay", 6));
    vm.globals.insert("despawn_overlay".to_string(), native("Scarab.despawn_overlay", 1));
    vm.globals.insert("send_input".to_string(), native("Scarab.send_input", 1));
    vm.globals.insert("get_terminal_rows".to_string(), native("Scarab.get_terminal_rows", 2));
    vm.globals.insert("log".to_string(), native("Scarab.log", 2));
    vm.globals.insert("scarab_version".to_string(), native("Scarab.version", 0));
    vm.globals.insert("set_color".to_string(), native("Scarab.setColor", 2));
    vm.globals.insert("set_window_title".to_string(), native("Scarab.setWindowTitle", 1));
    vm.globals.insert("set_font".to_string(), native("Scarab.setFont", 2));

    // Also create Scarab module record for potential future module-style access
    let mut scarab_fields = HashMap::new();
    scarab_fields.insert("status_add".to_string(), native("Scarab.status_add", 3));
    scarab_fields.insert("status_remove".to_string(), native("Scarab.status_remove", 1));
    scarab_fields.insert("notify".to_string(), native("Scarab.notify", 3));
    scarab_fields.insert("spawn_overlay".to_string(), native("Scarab.spawn_overlay", 6));
    scarab_fields.insert("despawn_overlay".to_string(), native("Scarab.despawn_overlay", 1));
    scarab_fields.insert("send_input".to_string(), native("Scarab.send_input", 1));
    scarab_fields.insert("get_terminal_rows".to_string(), native("Scarab.get_terminal_rows", 2));
    scarab_fields.insert("log".to_string(), native("Scarab.log", 2));
    scarab_fields.insert("version".to_string(), native("Scarab.version", 0));
    scarab_fields.insert("setColor".to_string(), native("Scarab.setColor", 2));
    scarab_fields.insert("setWindowTitle".to_string(), native("Scarab.setWindowTitle", 1));
    scarab_fields.insert("setFont".to_string(), native("Scarab.setFont", 2));

    vm.globals.insert(
        "Scarab".to_string(),
        Value::Record(Arc::new(Mutex::new(scarab_fields))),
    );

    info!("Registered Scarab functions as globals and module (12 functions)");
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

    #[test]
    fn test_scarab_module_calls() {
        let channel = Arc::new(FusabiActionChannel::new());
        let mut runtime = ScriptRuntime::new_with_channel(channel.clone()).unwrap();

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

        // Test direct function call (like print in stdlib)
        let log_test = "log \"info\" \"Test log message\"";
        let result = runtime.execute_source(log_test, "log-test.fsx", &context);
        assert!(result.is_ok(), "log should execute successfully: {:?}", result);

        // Test status_add
        let status_test = "status_add \"left\" \"Test Status\" 100";
        let result = runtime.execute_source(status_test, "status-test.fsx", &context);
        assert!(result.is_ok(), "status_add should execute successfully: {:?}", result);

        // Verify action was queued
        let actions = channel.pending_actions.lock().unwrap();
        assert!(actions.len() >= 1, "Should have at least 1 queued action: {:?}", actions.len());
    }

    #[test]
    fn test_setcolor_function() {
        let channel = Arc::new(FusabiActionChannel::new());
        let mut runtime = ScriptRuntime::new_with_channel(channel.clone()).unwrap();

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

        // Test set_color function (direct global) - single call
        let theme_script = r##"set_color "foreground" "#f8f8f2""##;

        let result = runtime.execute_source(theme_script, "theme-test.fsx", &context);
        assert!(result.is_ok(), "Theme script should execute successfully: {:?}", result);

        // Verify action was queued
        let actions = channel.pending_actions.lock().unwrap();
        assert_eq!(actions.len(), 1, "Should have 1 queued UpdateTheme action");
    }
}
