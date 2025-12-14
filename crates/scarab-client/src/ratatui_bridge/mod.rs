// Ratatui Bridge Module
//
// This module provides infrastructure for rendering Ratatui widgets in Bevy,
// allowing Scarab to leverage the rich Ratatui widget ecosystem for overlays,
// modals, and UI components without re-implementing them.
//
// # Architecture
//
// The bridge operates in three phases:
// 1. Widget rendering to Ratatui buffers (surface.rs - Task C1)
// 2. Buffer conversion to Bevy meshes/sprites (renderer.rs - Task C2)
// 3. Input event mapping from Bevy to Ratatui (input.rs - Task C3)
//
// # Usage Pattern
//
// ```ignore
// // 1. Spawn a surface entity
// commands.spawn(RatatuiSurface::new(10, 5, 80, 20));
//
// // 2. In a system, render widgets to the surface
// fn render_widget_system(
//     mut buffers: ResMut<SurfaceBuffers>,
//     query: Query<(Entity, &RatatuiSurface)>,
// ) {
//     for (entity, surface) in query.iter() {
//         if surface.visible && surface.dirty {
//             let buffer = buffers.get_or_create(entity, surface.width, surface.height);
//
//             // Render any Ratatui widget to the buffer
//             use ratatui::widgets::{Block, Borders};
//             let widget = Block::default()
//                 .title("My Widget")
//                 .borders(Borders::ALL);
//
//             widget.render(surface.rect(), buffer);
//         }
//     }
// }
//
// // 3. Handle input events for your widget
// fn handle_widget_input(
//     mut events: EventReader<SurfaceInputEvent>,
//     mut surface: Query<&mut RatatuiSurface>,
// ) {
//     for event in events.read() {
//         if let RatEvent::Key(key_event) = &event.event {
//             // Handle key input for your widget
//         }
//     }
// }
// ```
//
// # Coordinate System
//
// Surfaces use terminal grid coordinates:
// - Position (x, y): cells from top-left origin
// - Size (width, height): cells
// - Z-index: float for layer ordering (higher = on top)
//
// This matches the main terminal grid coordinate system, ensuring
// consistent positioning across the application.
//
// # Performance
//
// - Buffers are reused across frames (no per-frame allocation)
// - Surfaces track dirty state to skip unnecessary re-renders
// - Automatic cleanup when surfaces are despawned
// - Lock-free focus management for input routing
//
// # Input Handling (Task C3)
//
// The input.rs module provides:
// - Keyboard event conversion (Bevy KeyCode -> Ratatui KeyCode)
// - Mouse event conversion with grid coordinate mapping
// - Focus stack for overlay/modal priority
// - Automatic focus cleanup on surface despawn
//
// # Command Palette (Task C4)
//
// The command_palette.rs module provides a reference implementation
// of a Ratatui-based widget:
// - Searchable command list
// - Keyboard-driven navigation
// - Toggle with Ctrl+Shift+P
// - Focus management integration

mod command_palette;
mod input;
mod renderer;
mod surface;

// Re-export public API
pub use command_palette::{
    CommandPalettePlugin, CommandPaletteState, CommandPaletteSurface, CommandSelected,
    PaletteCommand,
};
pub use input::{
    bevy_to_ratatui_key, cleanup_focus, get_modifiers, handle_keyboard_input, handle_mouse_input,
    SurfaceFocus, SurfaceInputEvent,
};
pub use renderer::{cleanup_overlays, render_surfaces, SurfaceOverlay};
pub use surface::{RatatuiBridgePlugin, RatatuiSurface, SurfaceBuffers};

// Re-export commonly used Fusabi TUI types for convenience
pub use crossterm::event::{Event as RatEvent, KeyCode as RatKeyCode, KeyEvent};
pub use fusabi_tui_core::{Buffer, Color, Modifier, Rect, Style};
pub use fusabi_tui_widgets::Widget;
