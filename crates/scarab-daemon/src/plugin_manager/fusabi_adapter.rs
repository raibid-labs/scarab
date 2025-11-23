//! Fusabi plugin adapter for bridging Fusabi runtime to Plugin trait
//!
//! This module provides adapters for both .fzb (compiled bytecode) and .fsx (scripts)
//! to implement the scarab-plugin-api Plugin trait.

use scarab_plugin_api::{
    Action, Plugin, PluginContext, PluginError, PluginMetadata, Result,
};
use async_trait::async_trait;
use std::path::Path;

/// Adapter for compiled Fusabi bytecode (.fzb files)
#[derive(Debug)]
pub struct FusabiBytecodePlugin {
    metadata: PluginMetadata,
    _bytecode: Vec<u8>,
    // TODO: Add fusabi-vm VM instance when available
    // vm: fusabi_vm::VM,
}

impl FusabiBytecodePlugin {
    /// Load a .fzb bytecode file
    pub fn load(path: &Path) -> Result<Self> {
        // Read bytecode file
        let bytecode = std::fs::read(path).map_err(|e| {
            PluginError::LoadError(format!("Failed to read bytecode file: {}", e))
        })?;

        // Validate magic number "FZB\0"
        if bytecode.len() < 4 || &bytecode[0..4] != b"FZB\0" {
            return Err(PluginError::LoadError(
                "Invalid bytecode magic number".to_string(),
            ));
        }

        // Extract metadata from bytecode
        // TODO: Parse actual metadata from bytecode format
        let plugin_name = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string();

        let metadata = PluginMetadata::new(
            plugin_name,
            "0.1.0",
            "Fusabi bytecode plugin",
            "Fusabi VM",
        );

        // TODO: Initialize VM and load bytecode
        // let vm = fusabi_vm::VM::new()?;
        // vm.load_bytecode(&bytecode)?;

        Ok(Self {
            metadata,
            _bytecode: bytecode,
        })
    }
}

#[async_trait]
impl Plugin for FusabiBytecodePlugin {
    fn metadata(&self) -> &PluginMetadata {
        &self.metadata
    }

    async fn on_load(&mut self, ctx: &mut PluginContext) -> Result<()> {
        ctx.log(
            scarab_plugin_api::context::LogLevel::Info,
            &format!("Loading Fusabi bytecode plugin: {}", self.metadata.name),
        );

        // TODO: Execute VM initialization code
        // self.vm.call_function("on_load", &[ctx_value])?;

        Ok(())
    }

    async fn on_output(&mut self, _line: &str, _ctx: &PluginContext) -> Result<Action> {
        // TODO: Call VM hook function
        // let result = self.vm.call_function("on_output", &[Value::String(line.to_string())])?;
        // Parse result into Action

        // For now, just pass through
        log::trace!("Bytecode plugin '{}' processing output", self.metadata.name);
        Ok(Action::Continue)
    }

    async fn on_input(&mut self, input: &[u8], _ctx: &PluginContext) -> Result<Action> {
        // TODO: Call VM hook function
        log::trace!(
            "Bytecode plugin '{}' processing input ({} bytes)",
            self.metadata.name,
            input.len()
        );
        Ok(Action::Continue)
    }

    async fn on_resize(&mut self, cols: u16, rows: u16, _ctx: &PluginContext) -> Result<()> {
        // TODO: Call VM hook function
        log::trace!(
            "Bytecode plugin '{}' handling resize: {}x{}",
            self.metadata.name,
            cols,
            rows
        );
        Ok(())
    }

    async fn on_unload(&mut self) -> Result<()> {
        log::info!("Unloading Fusabi bytecode plugin: {}", self.metadata.name);
        // TODO: Execute VM cleanup code
        // self.vm.call_function("on_unload", &[])?;
        Ok(())
    }
}

/// Adapter for Fusabi script files (.fsx files)
#[derive(Debug)]
pub struct FusabiScriptPlugin {
    metadata: PluginMetadata,
    _script_source: String,
    // TODO: Add fusabi-frontend interpreter instance when available
    // interpreter: fusabi_frontend::Interpreter,
}

impl FusabiScriptPlugin {
    /// Load a .fsx script file
    pub fn load(path: &Path) -> Result<Self> {
        // Read script file
        let script_source = std::fs::read_to_string(path).map_err(|e| {
            PluginError::LoadError(format!("Failed to read script file: {}", e))
        })?;

        // Extract metadata from script
        // TODO: Parse metadata from script comments/attributes
        let plugin_name = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string();

        let metadata = PluginMetadata::new(
            plugin_name,
            "0.1.0",
            "Fusabi script plugin",
            "Fusabi Frontend",
        );

        // TODO: Parse and initialize interpreter
        // let ast = fusabi_frontend::parse(&script_source)?;
        // let interpreter = fusabi_frontend::Interpreter::new();
        // interpreter.load_ast(ast)?;

        Ok(Self {
            metadata,
            _script_source: script_source,
        })
    }

    /// Hot-reload the script from disk
    pub fn reload(&mut self, path: &Path) -> Result<()> {
        let script_source = std::fs::read_to_string(path).map_err(|e| {
            PluginError::LoadError(format!("Failed to reload script: {}", e))
        })?;

        // TODO: Reparse and update interpreter
        // let ast = fusabi_frontend::parse(&script_source)?;
        // self.interpreter.reload_ast(ast)?;

        self._script_source = script_source;
        log::info!("Hot-reloaded script plugin: {}", self.metadata.name);

        Ok(())
    }
}

#[async_trait]
impl Plugin for FusabiScriptPlugin {
    fn metadata(&self) -> &PluginMetadata {
        &self.metadata
    }

    async fn on_load(&mut self, ctx: &mut PluginContext) -> Result<()> {
        ctx.log(
            scarab_plugin_api::context::LogLevel::Info,
            &format!("Loading Fusabi script plugin: {}", self.metadata.name),
        );

        // TODO: Execute interpreter initialization
        // self.interpreter.call_function("on_load", &[ctx_value])?;

        Ok(())
    }

    async fn on_output(&mut self, _line: &str, _ctx: &PluginContext) -> Result<Action> {
        // TODO: Call interpreter hook function
        // let result = self.interpreter.eval_function("on_output", &[Value::String(line.to_string())])?;
        // Parse result into Action

        // For now, just pass through
        log::trace!("Script plugin '{}' processing output", self.metadata.name);
        Ok(Action::Continue)
    }

    async fn on_input(&mut self, input: &[u8], _ctx: &PluginContext) -> Result<Action> {
        // TODO: Call interpreter hook function
        log::trace!(
            "Script plugin '{}' processing input ({} bytes)",
            self.metadata.name,
            input.len()
        );
        Ok(Action::Continue)
    }

    async fn on_resize(&mut self, cols: u16, rows: u16, _ctx: &PluginContext) -> Result<()> {
        // TODO: Call interpreter hook function
        log::trace!(
            "Script plugin '{}' handling resize: {}x{}",
            self.metadata.name,
            cols,
            rows
        );
        Ok(())
    }

    async fn on_unload(&mut self) -> Result<()> {
        log::info!("Unloading Fusabi script plugin: {}", self.metadata.name);
        // TODO: Execute interpreter cleanup
        // self.interpreter.call_function("on_unload", &[])?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_load_invalid_bytecode() {
        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(b"INVALID").unwrap();
        temp_file.flush().unwrap();

        let result = FusabiBytecodePlugin::load(temp_file.path());
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Invalid bytecode magic number"));
    }

    #[test]
    fn test_load_valid_bytecode_header() {
        let mut temp_file = NamedTempFile::new().unwrap();
        // Write valid magic number
        temp_file.write_all(b"FZB\0").unwrap();
        // Write dummy version and data
        temp_file.write_all(&[0u8; 100]).unwrap();
        temp_file.flush().unwrap();

        let result = FusabiBytecodePlugin::load(temp_file.path());
        assert!(result.is_ok());
        let plugin = result.unwrap();
        assert!(plugin.metadata.name.len() > 0);
    }

    #[test]
    fn test_load_script() {
        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file
            .write_all(b"let greeting = \"Hello, Fusabi!\"\nprintln greeting")
            .unwrap();
        temp_file.flush().unwrap();

        let result = FusabiScriptPlugin::load(temp_file.path());
        assert!(result.is_ok());
        let plugin = result.unwrap();
        assert!(plugin.metadata.name.len() > 0);
    }

    #[tokio::test]
    async fn test_bytecode_plugin_lifecycle() {
        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(b"FZB\0").unwrap();
        temp_file.write_all(&[0u8; 100]).unwrap();
        temp_file.flush().unwrap();

        let mut plugin = FusabiBytecodePlugin::load(temp_file.path()).unwrap();

        // Create test context
        let state = std::sync::Arc::new(parking_lot::Mutex::new(
            scarab_plugin_api::context::PluginSharedState::new(80, 24),
        ));
        let mut ctx = PluginContext::new(Default::default(), state, "test");

        // Test lifecycle
        assert!(plugin.on_load(&mut ctx).await.is_ok());
        assert!(plugin.on_output("test line", &ctx).await.is_ok());
        assert!(plugin.on_input(b"test input", &ctx).await.is_ok());
        assert!(plugin.on_resize(100, 30, &ctx).await.is_ok());
        assert!(plugin.on_unload().await.is_ok());
    }

    #[tokio::test]
    async fn test_script_plugin_lifecycle() {
        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file
            .write_all(b"let greeting = \"Hello, Fusabi!\"")
            .unwrap();
        temp_file.flush().unwrap();

        let mut plugin = FusabiScriptPlugin::load(temp_file.path()).unwrap();

        // Create test context
        let state = std::sync::Arc::new(parking_lot::Mutex::new(
            scarab_plugin_api::context::PluginSharedState::new(80, 24),
        ));
        let mut ctx = PluginContext::new(Default::default(), state, "test");

        // Test lifecycle
        assert!(plugin.on_load(&mut ctx).await.is_ok());
        assert!(plugin.on_output("test line", &ctx).await.is_ok());
        assert!(plugin.on_input(b"test input", &ctx).await.is_ok());
        assert!(plugin.on_resize(100, 30, &ctx).await.is_ok());
        assert!(plugin.on_unload().await.is_ok());
    }

    #[test]
    fn test_script_hot_reload() {
        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(b"let x = 1").unwrap();
        temp_file.flush().unwrap();

        let mut plugin = FusabiScriptPlugin::load(temp_file.path()).unwrap();

        // Modify file
        temp_file.write_all(b"\nlet y = 2").unwrap();
        temp_file.flush().unwrap();

        // Reload
        assert!(plugin.reload(temp_file.path()).is_ok());
    }
}
