//! Example Scarab Plugin Template
//!
//! This is a template for creating your own Scarab plugins.
//! Copy this directory and modify it to create your custom plugin.

use scarab_plugin_api::{
    Action, Plugin, PluginContext, PluginMetadata, Result,
};
use async_trait::async_trait;

/// Your custom plugin struct
///
/// Add any state your plugin needs here.
pub struct ExamplePlugin {
    metadata: PluginMetadata,
    // Add your plugin state here
    notification_count: u32,
}

impl ExamplePlugin {
    /// Create a new instance of your plugin
    pub fn new() -> Self {
        Self {
            metadata: PluginMetadata::new(
                "example-plugin",
                "0.1.0",
                "An example plugin that demonstrates the Scarab plugin API",
                "Your Name <your.email@example.com>",
            )
            .with_homepage("https://github.com/yourusername/scarab-plugin-example"),
            notification_count: 0,
        }
    }
}

impl Default for ExamplePlugin {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Plugin for ExamplePlugin {
    fn metadata(&self) -> &PluginMetadata {
        &self.metadata
    }

    /// Called when the plugin is loaded
    async fn on_load(&mut self, ctx: &mut PluginContext) -> Result<()> {
        ctx.log(
            scarab_plugin_api::context::LogLevel::Info,
            "Example plugin loaded!",
        );

        // Initialize plugin state here
        self.notification_count = 0;

        Ok(())
    }

    /// Called when the plugin is being unloaded
    async fn on_unload(&mut self) -> Result<()> {
        log::info!(
            "Example plugin unloaded. Sent {} notifications.",
            self.notification_count
        );
        Ok(())
    }

    /// Hook called before output is displayed
    async fn on_output(&mut self, line: &str, ctx: &PluginContext) -> Result<Action> {
        // Example: Detect error keywords and send notification
        let error_keywords = ["ERROR", "FAIL", "PANIC", "FATAL"];

        if error_keywords.iter().any(|&kw| line.contains(kw)) {
            ctx.notify(&format!("Error detected: {}", line));
            self.notification_count += 1;

            // Optionally modify the line to highlight it
            // For now, we just continue
            return Ok(Action::Continue);
        }

        // Pass through unchanged
        Ok(Action::Continue)
    }

    /// Hook called after input is received
    async fn on_input(&mut self, input: &[u8], _ctx: &PluginContext) -> Result<Action> {
        // Example: You could intercept certain key combinations here
        // For now, just pass through
        Ok(Action::Continue)
    }

    /// Hook called before a command is executed
    async fn on_pre_command(&mut self, command: &str, ctx: &PluginContext) -> Result<Action> {
        // Example: Log all commands
        ctx.log(
            scarab_plugin_api::context::LogLevel::Debug,
            &format!("Executing command: {}", command),
        );
        Ok(Action::Continue)
    }

    /// Hook called after a command completes
    async fn on_post_command(
        &mut self,
        command: &str,
        exit_code: i32,
        ctx: &PluginContext,
    ) -> Result<()> {
        // Example: Notify on command failures
        if exit_code != 0 {
            ctx.notify(&format!("Command '{}' failed with exit code {}", command, exit_code));
            self.notification_count += 1;
        }
        Ok(())
    }

    /// Hook called when terminal is resized
    async fn on_resize(&mut self, cols: u16, rows: u16, ctx: &PluginContext) -> Result<()> {
        ctx.log(
            scarab_plugin_api::context::LogLevel::Debug,
            &format!("Terminal resized to {}x{}", cols, rows),
        );
        Ok(())
    }

    /// Hook called when a client attaches
    async fn on_attach(&mut self, client_id: u64, ctx: &PluginContext) -> Result<()> {
        ctx.notify(&format!("Client {} attached", client_id));
        Ok(())
    }

    /// Hook called when a client detaches
    async fn on_detach(&mut self, client_id: u64, ctx: &PluginContext) -> Result<()> {
        ctx.notify(&format!("Client {} detached", client_id));
        Ok(())
    }
}

// Export the plugin creation function
#[no_mangle]
pub extern "C" fn create_plugin() -> Box<dyn Plugin> {
    Box::new(ExamplePlugin::new())
}
