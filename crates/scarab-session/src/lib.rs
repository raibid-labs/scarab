use async_trait::async_trait;
use scarab_plugin_api::{Plugin, PluginContext, PluginMetadata, Result};
use scarab_protocol::ModalItem;

// Domain abstraction for terminal multiplexing
pub mod domain;
pub mod local_domain;
pub mod ssh_domain;

pub use domain::{
    Domain, DomainId, DomainPaneHandle, DomainRegistry, DomainStats, DomainType, PaneConfig,
};
pub use local_domain::LocalDomain;
pub use ssh_domain::{SshAuth, SshDomain, SshDomainConfig};

pub struct SessionPlugin {
    metadata: PluginMetadata,
}

impl SessionPlugin {
    pub fn new() -> Self {
        Self {
            metadata: PluginMetadata::new(
                "scarab-session",
                "0.1.0",
                "Session management commands",
                "Scarab Team",
            ),
        }
    }
}

#[async_trait]
impl Plugin for SessionPlugin {
    fn metadata(&self) -> &PluginMetadata {
        &self.metadata
    }

    fn get_commands(&self) -> Vec<ModalItem> {
        vec![
            ModalItem {
                id: "session.new_tab".to_string(),
                label: "New Tab".to_string(),
                description: Some("Open a new tab in current window".to_string()),
            },
            ModalItem {
                id: "session.close_tab".to_string(),
                label: "Close Tab".to_string(),
                description: Some("Close current tab".to_string()),
            },
            ModalItem {
                id: "session.detach".to_string(),
                label: "Detach Session".to_string(),
                description: Some("Detach client from session".to_string()),
            },
        ]
    }

    async fn on_remote_command(&mut self, id: &str, ctx: &PluginContext) -> Result<()> {
        match id {
            "session.new_tab" => {
                log::info!("Creating new tab");
                // Send TabCreate control message to daemon
                // This will be handled by the session manager to create an actual tab
                // Note: The plugin context doesn't directly send control messages,
                // but the client will handle this command and send the appropriate message.
                // For now, we'll log a proper message indicating this should trigger tab creation.
                ctx.notify_success("New Tab", "Creating new tab...");
                log::debug!("Session plugin: new_tab command should trigger TabCreate control message");
            }
            "session.close_tab" => {
                log::info!("Closing current tab");
                ctx.notify_info("Close Tab", "Closing current tab...");
                log::debug!("Session plugin: close_tab command should trigger TabClose control message");
            }
            "session.detach" => {
                log::info!("Detaching session");
                ctx.notify_info("Detach", "Detaching from session...");
                log::debug!("Session plugin: detach command should trigger SessionDetach control message");
            }
            _ => {}
        }
        Ok(())
    }
}
