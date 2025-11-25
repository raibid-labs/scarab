use async_trait::async_trait;
use regex::Regex;
use scarab_plugin_api::{
    types::{OverlayStyle, RemoteCommand},
    Action, Plugin, PluginContext, PluginMetadata, Result,
};
use std::sync::Mutex;

pub struct NavigationPlugin {
    metadata: PluginMetadata,
    state: Mutex<PluginState>,
    url_regex: Regex,
}

#[derive(Default)]
struct PluginState {
    active: bool,
    hints: Vec<Hint>,
    input_buffer: String,
}

struct Hint {
    id: u64,
    label: String,
    url: String,
}

impl NavigationPlugin {
    pub fn new() -> Self {
        Self {
            metadata: PluginMetadata::new(
                "scarab-nav",
                "0.1.0",
                "Keyboard navigation and link hints",
                "Scarab Team",
            ),
            state: Mutex::new(PluginState::default()),
            url_regex: Regex::new(r"https?://[^\s\)]+").unwrap(),
        }
    }

    fn generate_labels(count: usize) -> Vec<String> {
        let chars = "asdfghjkl";
        let mut labels = Vec::new();
        // Simple single-char labels for demo
        for (i, c) in chars.chars().enumerate() {
            if i >= count {
                break;
            }
            labels.push(c.to_string());
        }
        labels
    }

    /// Open a URL in the default browser using platform-specific commands
    fn open_url(url: &str) -> std::result::Result<(), String> {
        log::info!("Attempting to open URL: {}", url);

        #[cfg(target_os = "linux")]
        let result = std::process::Command::new("xdg-open")
            .arg(url)
            .spawn()
            .map_err(|e| format!("Failed to launch xdg-open: {}", e));

        #[cfg(target_os = "macos")]
        let result = std::process::Command::new("open")
            .arg(url)
            .spawn()
            .map_err(|e| format!("Failed to launch open: {}", e));

        #[cfg(target_os = "windows")]
        let result = std::process::Command::new("cmd")
            .args(&["/C", "start", "", url])
            .spawn()
            .map_err(|e| format!("Failed to launch cmd: {}", e));

        #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
        let result = Err("Unsupported platform for opening URLs".to_string());

        match result {
            Ok(_) => {
                log::info!("Successfully launched browser for URL: {}", url);
                Ok(())
            }
            Err(e) => {
                log::error!("Failed to open URL {}: {}", url, e);
                Err(e)
            }
        }
    }
}

#[async_trait]
impl Plugin for NavigationPlugin {
    fn metadata(&self) -> &PluginMetadata {
        &self.metadata
    }

    async fn on_input(&mut self, input: &[u8], ctx: &PluginContext) -> Result<Action> {
        let mut state = self.state.lock().unwrap();

        // Trigger key: Ctrl+; (represented as 0x1D? No, usually depends. Let's use Ctrl+f = 0x06)
        // Let's use a simpler trigger for testing: Alt+f (Esc + f)

        if !state.active {
            // Check for activation (Alt+f = Esc, f)
            // This is tricky with raw bytes if split across packets.
            // Let's assume single packet for now: [0x1b, b'f']
            if input == [0x1b, b'f'] {
                log::info!("Activating Link Hints");
                state.active = true;
                state.hints.clear();

                // Scan screen
                let (cols, rows) = ctx.get_size();
                let mut hint_id = 0;
                let mut found_urls = Vec::new();

                for y in 0..rows {
                    if let Some(line) = ctx.get_line(y) {
                        for mat in self.url_regex.find_iter(&line) {
                            found_urls.push((y, mat.start() as u16, mat.as_str().to_string()));
                        }
                    }
                }

                let labels = Self::generate_labels(found_urls.len());

                for (i, (y, x, url)) in found_urls.into_iter().enumerate() {
                    if i >= labels.len() {
                        break;
                    }

                    let label = labels[i].clone();
                    let id = hint_id;
                    hint_id += 1;

                    state.hints.push(Hint {
                        id,
                        label: label.clone(),
                        url,
                    });

                    // Draw Overlay
                    ctx.queue_command(RemoteCommand::DrawOverlay {
                        id,
                        x,
                        y,
                        text: label,
                        style: OverlayStyle {
                            fg: 0xFFFFFFFF, // White
                            bg: 0xFF0000FF, // Red
                            z_index: 100.0,
                        },
                    });
                }

                return Ok(Action::Modify(Vec::new())); // Consume the trigger key
            }

            return Ok(Action::Continue);
        } else {
            // Handle Input while Active

            // Escape cancels
            if input == [0x1b] {
                state.active = false;
                ctx.queue_command(RemoteCommand::ClearOverlays { id: None });
                return Ok(Action::Modify(Vec::new()));
            }

            // Match label
            if let Ok(s) = std::str::from_utf8(input) {
                for hint in &state.hints {
                    if hint.label == s {
                        log::info!("Opening URL: {}", hint.url);

                        // Open URL in default browser
                        match Self::open_url(&hint.url) {
                            Ok(_) => {
                                ctx.notify_success("URL Opened", &format!("Opened: {}", hint.url));
                            }
                            Err(e) => {
                                ctx.notify_error("URL Open Failed", &format!("Failed to open URL: {}", e));
                                log::error!("Error opening URL {}: {}", hint.url, e);
                            }
                        }

                        state.active = false;
                        ctx.queue_command(RemoteCommand::ClearOverlays { id: None });
                        return Ok(Action::Modify(Vec::new()));
                    }
                }
            }

            return Ok(Action::Modify(Vec::new())); // Consume all input while active
        }
    }
}
