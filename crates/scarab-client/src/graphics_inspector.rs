//! Graphics Inspector for Inline Terminal Images
//!
//! Provides a visual debugger for inline images using Sixel, Kitty, and iTerm2 protocols.
//! Features:
//! - List of active image placements with metadata
//! - Protocol type, dimensions, position tracking
//! - Memory usage statistics
//! - Image preview thumbnails
//! - Export capability for debugging

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiPlugin};
use scarab_protocol::{ImageFormat as ProtocolImageFormat, ImagePlacement, TerminalMetrics};

use crate::rendering::{ImageCache, SharedImageReader};

/// Resource tracking graphics inspector state
#[derive(Resource)]
pub struct GraphicsInspectorState {
    /// Whether the inspector window is visible
    pub visible: bool,
    /// Selected image index
    pub selected_index: usize,
    /// Filter text for images
    pub filter_text: String,
    /// Sort mode for image list
    pub sort_mode: ImageSortMode,
    /// Statistics tracking
    pub stats: GraphicsStats,
}

impl Default for GraphicsInspectorState {
    fn default() -> Self {
        Self {
            visible: false,
            selected_index: 0,
            filter_text: String::new(),
            sort_mode: ImageSortMode::ById,
            stats: GraphicsStats::default(),
        }
    }
}

/// Statistics about image rendering
#[derive(Default)]
pub struct GraphicsStats {
    /// Total number of images loaded
    pub total_loaded: usize,
    /// Total memory used by images (bytes)
    pub total_memory: usize,
    /// Number of images currently visible
    pub visible_count: usize,
    /// Peak memory usage
    pub peak_memory: usize,
}

/// Sort mode for image list
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ImageSortMode {
    ById,
    ByPosition,
    BySize,
    ByProtocol,
}

/// Plugin for graphics inspector
pub struct GraphicsInspectorPlugin;

impl Plugin for GraphicsInspectorPlugin {
    fn build(&self, app: &mut App) {
        // Add egui plugin if not already present
        if !app.is_plugin_added::<EguiPlugin>() {
            app.add_plugins(EguiPlugin);
        }

        app.insert_resource(GraphicsInspectorState::default())
            .add_systems(Update, (toggle_inspector_system, render_inspector_system));
    }
}

/// System to handle toggling the inspector with Ctrl+Shift+G
fn toggle_inspector_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut state: ResMut<GraphicsInspectorState>,
) {
    // Check for Ctrl+Shift+G
    if keyboard.pressed(KeyCode::ControlLeft) || keyboard.pressed(KeyCode::ControlRight) {
        if keyboard.pressed(KeyCode::ShiftLeft) || keyboard.pressed(KeyCode::ShiftRight) {
            if keyboard.just_pressed(KeyCode::KeyG) {
                state.visible = !state.visible;
                info!(
                    "Graphics Inspector {}",
                    if state.visible { "opened" } else { "closed" }
                );
            }
        }
    }
}

/// System to render the inspector UI
fn render_inspector_system(
    mut contexts: EguiContexts,
    mut state: ResMut<GraphicsInspectorState>,
    cache: Res<ImageCache>,
    reader: Option<Res<SharedImageReader>>,
    metrics: Res<TerminalMetrics>,
) {
    if !state.visible {
        return;
    }

    let ctx = contexts.ctx_mut();

    // Update statistics
    update_stats(&mut state.stats, &cache, reader.as_deref());

    egui::Window::new("Graphics Inspector")
        .default_width(800.0)
        .default_height(600.0)
        .resizable(true)
        .show(ctx, |ui| {
            render_toolbar(ui, &mut state);
            ui.separator();

            egui::TopBottomPanel::top("stats_panel")
                .resizable(false)
                .show_inside(ui, |ui| {
                    render_stats_panel(ui, &state.stats);
                });

            ui.separator();

            egui::SidePanel::left("image_list")
                .default_width(300.0)
                .resizable(true)
                .show_inside(ui, |ui| {
                    render_image_list(ui, &mut state, &cache);
                });

            egui::CentralPanel::default().show_inside(ui, |ui| {
                render_image_details(ui, &state, &cache, &metrics);
            });
        });
}

/// Render the toolbar with controls
fn render_toolbar(ui: &mut egui::Ui, state: &mut GraphicsInspectorState) {
    ui.horizontal(|ui| {
        ui.label("Filter:");
        ui.text_edit_singleline(&mut state.filter_text);

        ui.separator();

        ui.label("Sort:");
        egui::ComboBox::from_id_salt("sort_mode")
            .selected_text(format!("{:?}", state.sort_mode))
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut state.sort_mode, ImageSortMode::ById, "By ID");
                ui.selectable_value(
                    &mut state.sort_mode,
                    ImageSortMode::ByPosition,
                    "By Position",
                );
                ui.selectable_value(&mut state.sort_mode, ImageSortMode::BySize, "By Size");
                ui.selectable_value(
                    &mut state.sort_mode,
                    ImageSortMode::ByProtocol,
                    "By Protocol",
                );
            });

        ui.separator();

        if ui.button("Clear Selection").clicked() {
            state.selected_index = 0;
        }

        if ui.button("Close [Ctrl+Shift+G]").clicked() {
            state.visible = false;
        }
    });
}

/// Render statistics panel
fn render_stats_panel(ui: &mut egui::Ui, stats: &GraphicsStats) {
    ui.horizontal(|ui| {
        ui.label(format!("Total Images: {}", stats.total_loaded));
        ui.separator();
        ui.label(format!("Visible: {}", stats.visible_count));
        ui.separator();
        ui.label(format!(
            "Memory: {}",
            format_bytes(stats.total_memory)
        ));
        ui.separator();
        ui.label(format!("Peak: {}", format_bytes(stats.peak_memory)));
    });
}

/// Render the list of images
fn render_image_list(
    ui: &mut egui::Ui,
    state: &mut GraphicsInspectorState,
    cache: &ImageCache,
) {
    ui.heading("Active Images");
    ui.separator();

    let mut placements = cache.placements.clone();

    // Apply filter
    if !state.filter_text.is_empty() {
        let filter = state.filter_text.to_lowercase();
        placements.retain(|p| {
            p.id.to_string().contains(&filter)
                || format!("{:?}", p.format).to_lowercase().contains(&filter)
        });
    }

    // Apply sort
    match state.sort_mode {
        ImageSortMode::ById => placements.sort_by_key(|p| p.id),
        ImageSortMode::ByPosition => placements.sort_by_key(|p| (p.y, p.x)),
        ImageSortMode::BySize => {
            placements.sort_by_key(|p| std::cmp::Reverse(p.width_cells * p.height_cells))
        }
        ImageSortMode::ByProtocol => placements.sort_by_key(|p| format!("{:?}", p.format)),
    }

    if placements.is_empty() {
        ui.label("No images currently loaded");
        return;
    }

    egui::ScrollArea::vertical().show(ui, |ui| {
        for (idx, placement) in placements.iter().enumerate() {
            let is_selected = idx == state.selected_index;

            let response = ui.selectable_label(
                is_selected,
                format!(
                    "#{} - {} @ ({}, {})",
                    placement.id,
                    format_protocol(placement.format),
                    placement.x,
                    placement.y
                ),
            );

            if response.clicked() {
                state.selected_index = idx;
            }

            // Show mini preview of dimensions
            if is_selected {
                ui.indent("details", |ui| {
                    ui.label(format!(
                        "{}x{} cells ({}x{} px est.)",
                        placement.width_cells,
                        placement.height_cells,
                        placement.width_cells as u32 * 10,
                        placement.height_cells as u32 * 20
                    ));
                });
            }

            ui.separator();
        }
    });
}

/// Render detailed information about the selected image
fn render_image_details(
    ui: &mut egui::Ui,
    state: &GraphicsInspectorState,
    cache: &ImageCache,
    metrics: &TerminalMetrics,
) {
    ui.heading("Image Details");
    ui.separator();

    if cache.placements.is_empty() {
        ui.label("No image selected");
        return;
    }

    if state.selected_index >= cache.placements.len() {
        ui.label("Invalid selection");
        return;
    }

    let placement = &cache.placements[state.selected_index];

    egui::Grid::new("image_details_grid")
        .num_columns(2)
        .spacing([40.0, 8.0])
        .striped(true)
        .show(ui, |ui| {
            // Basic info
            ui.label("Image ID:");
            ui.label(format!("{}", placement.id));
            ui.end_row();

            ui.label("Protocol:");
            ui.label(format_protocol(placement.format));
            ui.end_row();

            ui.separator();
            ui.separator();
            ui.end_row();

            // Position info
            ui.label("Grid Position:");
            ui.label(format!("({}, {})", placement.x, placement.y));
            ui.end_row();

            ui.label("Cell Dimensions:");
            ui.label(format!("{}x{}", placement.width_cells, placement.height_cells));
            ui.end_row();

            let (screen_x, screen_y) = metrics.grid_to_screen(placement.x, placement.y);
            ui.label("Screen Position:");
            ui.label(format!("({:.1}, {:.1}) px", screen_x, screen_y));
            ui.end_row();

            ui.separator();
            ui.separator();
            ui.end_row();

            // Memory info
            ui.label("Shared Memory Offset:");
            ui.label(format!("0x{:08X}", placement.shm_offset));
            ui.end_row();

            ui.label("Data Size:");
            ui.label(format_bytes(placement.shm_size));
            ui.end_row();

            let estimated_decoded_size = (placement.width_cells as usize * 10)
                * (placement.height_cells as usize * 20)
                * 4; // RGBA
            ui.label("Estimated Decoded Size:");
            ui.label(format_bytes(estimated_decoded_size));
            ui.end_row();
        });

    ui.separator();

    // Action buttons
    ui.horizontal(|ui| {
        if ui.button("Copy Image ID").clicked() {
            #[cfg(not(target_arch = "wasm32"))]
            {
                use arboard::Clipboard;
                if let Ok(mut clipboard) = Clipboard::new() {
                    let _ = clipboard.set_text(placement.id.to_string());
                    info!("Copied image ID {} to clipboard", placement.id);
                }
            }
        }

        if ui.button("Copy Position").clicked() {
            #[cfg(not(target_arch = "wasm32"))]
            {
                use arboard::Clipboard;
                if let Ok(mut clipboard) = Clipboard::new() {
                    let _ = clipboard.set_text(format!("{},{}", placement.x, placement.y));
                    info!(
                        "Copied position ({},{}) to clipboard",
                        placement.x, placement.y
                    );
                }
            }
        }

        if ui.button("Export Metadata").clicked() {
            export_metadata(placement);
        }
    });

    ui.separator();

    // Technical details in collapsible section
    egui::CollapsingHeader::new("Technical Details")
        .default_open(false)
        .show(ui, |ui| {
            ui.label("Raw Data:");
            ui.monospace(format!("Format: {:?}", placement.format));
            ui.monospace(format!("SHM Offset: 0x{:08X}", placement.shm_offset));
            ui.monospace(format!("SHM Size: {} bytes", placement.shm_size));
            ui.monospace(format!("Cell Width: {}", placement.width_cells));
            ui.monospace(format!("Cell Height: {}", placement.height_cells));
        });
}

/// Update statistics from current state
fn update_stats(stats: &mut GraphicsStats, cache: &ImageCache, reader: Option<&SharedImageReader>) {
    stats.total_loaded = cache.placements.len();
    stats.visible_count = cache.placements.len();

    // Calculate total memory from placements
    stats.total_memory = cache
        .placements
        .iter()
        .map(|p| p.shm_size)
        .sum();

    // Update peak memory
    if stats.total_memory > stats.peak_memory {
        stats.peak_memory = stats.total_memory;
    }

    // If reader is available, get more accurate count
    if let Some(reader) = reader {
        let active_count = reader.placements().count();
        stats.visible_count = active_count;
    }
}

/// Format protocol type as human-readable string
fn format_protocol(format: ProtocolImageFormat) -> &'static str {
    match format {
        ProtocolImageFormat::Png => "PNG",
        ProtocolImageFormat::Jpeg => "JPEG",
        ProtocolImageFormat::Gif => "GIF",
        ProtocolImageFormat::Rgba => "RGBA (Sixel)",
    }
}

/// Format byte count as human-readable string
fn format_bytes(bytes: usize) -> String {
    const KB: usize = 1024;
    const MB: usize = 1024 * KB;
    const GB: usize = 1024 * MB;

    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} bytes", bytes)
    }
}

/// Export image metadata to JSON file
fn export_metadata(placement: &ImagePlacement) {
    let metadata = serde_json::json!({
        "id": placement.id,
        "format": format!("{:?}", placement.format),
        "position": {
            "x": placement.x,
            "y": placement.y,
        },
        "dimensions": {
            "width_cells": placement.width_cells,
            "height_cells": placement.height_cells,
        },
        "shared_memory": {
            "offset": placement.shm_offset,
            "size": placement.shm_size,
        },
    });

    let filename = format!("image_{}_metadata.json", placement.id);
    if let Ok(json_str) = serde_json::to_string_pretty(&metadata) {
        if let Err(e) = std::fs::write(&filename, json_str) {
            error!("Failed to export metadata: {}", e);
        } else {
            info!("Exported metadata to {}", filename);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_bytes() {
        assert_eq!(format_bytes(512), "512 bytes");
        assert_eq!(format_bytes(1024), "1.00 KB");
        assert_eq!(format_bytes(1024 * 1024), "1.00 MB");
        assert_eq!(format_bytes(1024 * 1024 * 1024), "1.00 GB");
    }

    #[test]
    fn test_format_protocol() {
        assert_eq!(format_protocol(ProtocolImageFormat::Png), "PNG");
        assert_eq!(format_protocol(ProtocolImageFormat::Jpeg), "JPEG");
        assert_eq!(format_protocol(ProtocolImageFormat::Gif), "GIF");
        assert_eq!(format_protocol(ProtocolImageFormat::Rgba), "RGBA (Sixel)");
    }

    #[test]
    fn test_inspector_state_default() {
        let state = GraphicsInspectorState::default();
        assert!(!state.visible);
        assert_eq!(state.selected_index, 0);
        assert_eq!(state.filter_text, "");
    }
}
