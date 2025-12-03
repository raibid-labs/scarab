//! Plugin installation progress tracking and UI

use bevy::prelude::*;
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Gauge, Paragraph, Widget},
};

use crate::ratatui_bridge::{Buffer, RatatuiSurface, SurfaceBuffers};

/// Installation status
#[derive(Debug, Clone, PartialEq)]
pub enum InstallStatus {
    /// No installation in progress
    Idle,
    /// Downloading plugin
    Downloading,
    /// Verifying checksum and signature
    Verifying,
    /// Installing to disk
    Installing,
    /// Installation complete
    Complete,
    /// Installation failed
    Failed(String),
}

/// Resource tracking installation progress
#[derive(Resource, Debug, Clone)]
pub struct InstallProgress {
    /// Name of plugin being installed
    pub plugin_name: String,
    /// Current installation status
    pub status: InstallStatus,
    /// Progress percentage (0-100)
    pub progress: u8,
    /// Detailed status message
    pub message: String,
    /// Installation start time
    pub start_time: Option<u64>,
    /// Installation end time
    pub end_time: Option<u64>,
}

impl Default for InstallProgress {
    fn default() -> Self {
        Self {
            plugin_name: String::new(),
            status: InstallStatus::Idle,
            progress: 0,
            message: String::new(),
            start_time: None,
            end_time: None,
        }
    }
}

impl InstallProgress {
    /// Start a new installation
    pub fn start_installation(&mut self, plugin_name: String, _version: Option<String>) {
        self.plugin_name = plugin_name;
        self.status = InstallStatus::Downloading;
        self.progress = 0;
        self.message = "Starting download...".to_string();
        self.start_time = Some(current_timestamp());
        self.end_time = None;
    }

    /// Update progress percentage
    pub fn set_progress(&mut self, progress: u8) {
        self.progress = progress.min(100);
    }

    /// Set status message
    pub fn set_message(&mut self, message: String) {
        self.message = message;
    }

    /// Mark installation as complete
    pub fn complete(&mut self) {
        self.status = InstallStatus::Complete;
        self.progress = 100;
        self.message = format!("Successfully installed {}", self.plugin_name);
        self.end_time = Some(current_timestamp());
    }

    /// Mark installation as failed
    pub fn fail(&mut self, error: String) {
        self.status = InstallStatus::Failed(error.clone());
        self.message = format!("Installation failed: {}", error);
        self.end_time = Some(current_timestamp());
    }

    /// Get elapsed time in seconds
    pub fn elapsed_seconds(&self) -> u64 {
        if let Some(start) = self.start_time {
            let end = self.end_time.unwrap_or_else(current_timestamp);
            end.saturating_sub(start)
        } else {
            0
        }
    }

    /// Check if installation is in progress
    pub fn is_active(&self) -> bool {
        !matches!(
            self.status,
            InstallStatus::Idle | InstallStatus::Complete | InstallStatus::Failed(_)
        )
    }
}

/// Get current timestamp in seconds
fn current_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

/// Update installation progress (simulates async installation)
pub fn update_install_progress(mut progress: ResMut<InstallProgress>, time: Res<Time>) {
    if !progress.is_active() {
        return;
    }

    // Simulate progress updates
    // In real implementation, this would track actual download/install progress
    let delta = (time.delta_secs() * 20.0) as u8; // 20% per second

    match progress.status {
        InstallStatus::Downloading => {
            progress.progress = (progress.progress + delta).min(30);
            if progress.progress >= 30 {
                progress.status = InstallStatus::Verifying;
                progress.message = "Verifying plugin integrity...".to_string();
            }
        }
        InstallStatus::Verifying => {
            progress.progress = (progress.progress + delta).min(60);
            if progress.progress >= 60 {
                progress.status = InstallStatus::Installing;
                progress.message = "Installing plugin files...".to_string();
            }
        }
        InstallStatus::Installing => {
            progress.progress = (progress.progress + delta).min(100);
            if progress.progress >= 100 {
                progress.complete();
            }
        }
        _ => {}
    }
}

/// Render installation progress UI
pub fn render_install_progress(
    mut buffers: ResMut<SurfaceBuffers>,
    mut query: Query<(Entity, &mut RatatuiSurface), With<InstallProgressOverlay>>,
    progress: Res<InstallProgress>,
) {
    // Only render if installation is active
    if !progress.is_active() && progress.status != InstallStatus::Complete {
        // Hide overlay if idle
        if let Ok((_, mut surface)) = query.get_single_mut() {
            surface.hide();
        }
        return;
    }

    // Get or spawn overlay
    let (entity, mut surface) = match query.get_single_mut() {
        Ok(result) => result,
        Err(_) => return, // Overlay not spawned yet
    };

    // Show overlay
    if !surface.visible {
        surface.show();
    }

    if !surface.dirty && !progress.is_changed() {
        return;
    }

    let buffer = buffers.get_or_create(entity, surface.width, surface.height);
    render_progress_widget(buffer, &progress);
    surface.mark_clean();
}

/// Render progress widget to buffer
fn render_progress_widget(buffer: &mut Buffer, progress: &InstallProgress) {
    let area = buffer.area();

    // Center the progress widget
    let width = 60.min(area.width);
    let height = 10.min(area.height);
    let x = (area.width.saturating_sub(width)) / 2;
    let y = (area.height.saturating_sub(height)) / 2;

    let widget_area = Rect::new(x, y, width, height);

    // Determine colors based on status
    let (title_color, border_color) = match progress.status {
        InstallStatus::Complete => (Color::Green, Color::Green),
        InstallStatus::Failed(_) => (Color::Red, Color::Red),
        _ => (Color::Cyan, Color::Cyan),
    };

    // Create content lines
    let mut lines = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("Plugin: ", Style::default().fg(Color::Gray)),
            Span::styled(
                &progress.plugin_name,
                Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Status: ", Style::default().fg(Color::Gray)),
            Span::styled(status_text(&progress.status), Style::default().fg(title_color)),
        ]),
        Line::from(""),
    ];

    // Add progress bar if in progress
    if progress.is_active() {
        let gauge = Gauge::default()
            .block(Block::default())
            .gauge_style(Style::default().fg(Color::Cyan).bg(Color::DarkGray))
            .percent(progress.progress as u16);

        // Render gauge to a sub-area
        let gauge_area = Rect::new(
            widget_area.x + 2,
            widget_area.y + 6,
            widget_area.width.saturating_sub(4),
            1,
        );
        gauge.render(gauge_area, buffer);
    }

    // Add message
    lines.push(Line::from(""));
    lines.push(Line::from(vec![Span::styled(
        &progress.message,
        Style::default().fg(Color::Gray),
    )]));

    // Add elapsed time for completed/failed
    if !progress.is_active() {
        lines.push(Line::from(""));
        lines.push(Line::from(vec![
            Span::styled("Time: ", Style::default().fg(Color::Gray)),
            Span::styled(
                format!("{}s", progress.elapsed_seconds()),
                Style::default().fg(Color::White),
            ),
        ]));
    }

    // Render paragraph
    let paragraph = Paragraph::new(lines).block(
        Block::default()
            .borders(Borders::ALL)
            .title("Plugin Installation")
            .border_style(Style::default().fg(border_color)),
    );

    paragraph.render(widget_area, buffer);
}

/// Get status text
fn status_text(status: &InstallStatus) -> &str {
    match status {
        InstallStatus::Idle => "Idle",
        InstallStatus::Downloading => "Downloading...",
        InstallStatus::Verifying => "Verifying...",
        InstallStatus::Installing => "Installing...",
        InstallStatus::Complete => "Complete",
        InstallStatus::Failed(_) => "Failed",
    }
}

/// Marker component for installation progress overlay
#[derive(Component, Debug)]
pub struct InstallProgressOverlay;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_install_progress_lifecycle() {
        let mut progress = InstallProgress::default();

        assert!(!progress.is_active());
        assert_eq!(progress.status, InstallStatus::Idle);

        progress.start_installation("test-plugin".to_string(), None);
        assert!(progress.is_active());
        assert_eq!(progress.status, InstallStatus::Downloading);
        assert_eq!(progress.progress, 0);

        progress.set_progress(50);
        assert_eq!(progress.progress, 50);

        progress.complete();
        assert!(!progress.is_active());
        assert_eq!(progress.status, InstallStatus::Complete);
        assert_eq!(progress.progress, 100);
    }

    #[test]
    fn test_install_progress_failure() {
        let mut progress = InstallProgress::default();

        progress.start_installation("test-plugin".to_string(), None);
        assert!(progress.is_active());

        progress.fail("Network error".to_string());
        assert!(!progress.is_active());
        assert!(matches!(progress.status, InstallStatus::Failed(_)));
    }

    #[test]
    fn test_progress_clamping() {
        let mut progress = InstallProgress::default();

        progress.set_progress(150); // Over 100
        assert_eq!(progress.progress, 100);
    }

    #[test]
    fn test_status_text() {
        assert_eq!(status_text(&InstallStatus::Idle), "Idle");
        assert_eq!(status_text(&InstallStatus::Downloading), "Downloading...");
        assert_eq!(status_text(&InstallStatus::Complete), "Complete");
        assert_eq!(
            status_text(&InstallStatus::Failed("error".to_string())),
            "Failed"
        );
    }
}
