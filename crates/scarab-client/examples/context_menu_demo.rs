//! Context Menu Demo
//!
//! This example demonstrates the context menu system in Scarab.
//!
//! Controls:
//! - Right-click: Open context menu
//! - Up/Down arrows: Navigate menu items
//! - Enter: Select menu item
//! - Escape: Close menu
//! - Mouse hover: Highlight menu items
//! - Mouse click: Select menu item
//!
//! Run with: cargo run -p scarab-client --example context_menu_demo

use bevy::prelude::*;
use bevy::window::WindowMode;
use scarab_client::context_menu::{
    ContextMenuAction, ContextMenuPlugin, DispatchContextMenuAction,
};
use scarab_client::ratatui_bridge::RatatuiBridgePlugin;
use scarab_protocol::TerminalMetrics;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Context Menu Demo".to_string(),
                resolution: (1200.0, 800.0).into(),
                mode: WindowMode::Windowed,
                ..default()
            }),
            ..default()
        }))
        .add_plugins(RatatuiBridgePlugin)
        .add_plugins(ContextMenuPlugin)
        .insert_resource(TerminalMetrics {
            cols: 120,
            rows: 40,
            cell_width: 10.0,
            cell_height: 20.0,
        })
        .add_systems(Startup, setup)
        .add_systems(Update, (handle_actions, display_instructions))
        .run();
}

/// Setup the demo scene
fn setup(mut commands: Commands) {
    // Spawn a 2D camera
    commands.spawn(Camera2dBundle::default());

    info!("Context Menu Demo started");
    info!("Right-click anywhere to open the context menu");
    info!("Use arrow keys or mouse to navigate");
}

/// Handle context menu actions and log them
fn handle_actions(mut events: EventReader<DispatchContextMenuAction>) {
    for event in events.read() {
        match &event.action {
            ContextMenuAction::Copy => {
                info!("ACTION: Copy");
            }
            ContextMenuAction::Paste => {
                info!("ACTION: Paste");
            }
            ContextMenuAction::SelectAll => {
                info!("ACTION: Select All");
            }
            ContextMenuAction::ClearSelection => {
                info!("ACTION: Clear Selection");
            }
            ContextMenuAction::Search => {
                info!("ACTION: Search");
            }
            ContextMenuAction::NewTab => {
                info!("ACTION: New Tab");
            }
            ContextMenuAction::SplitHorizontal => {
                info!("ACTION: Split Horizontal");
            }
            ContextMenuAction::SplitVertical => {
                info!("ACTION: Split Vertical");
            }
            ContextMenuAction::OpenUrl(url) => {
                info!("ACTION: Open URL - {}", url);
            }
            ContextMenuAction::CopyUrl(url) => {
                info!("ACTION: Copy URL - {}", url);
            }
            ContextMenuAction::OpenFile(path) => {
                info!("ACTION: Open File - {}", path);
            }
            ContextMenuAction::CopyPath(path) => {
                info!("ACTION: Copy Path - {}", path);
            }
            ContextMenuAction::PluginAction(id) => {
                info!("ACTION: Plugin Action - {}", id);
            }
        }
    }
}

#[derive(Component)]
struct InstructionsText;

/// Display instructions on screen
fn display_instructions(
    mut commands: Commands,
    query: Query<Entity, With<InstructionsText>>,
    asset_server: Res<AssetServer>,
) {
    // Only spawn once
    if !query.is_empty() {
        return;
    }

    let font = asset_server.load("fonts/FiraMono-Medium.ttf");

    commands.spawn((
        TextBundle::from_sections([
            TextSection::new(
                "Context Menu Demo\n\n",
                TextStyle {
                    font: font.clone(),
                    font_size: 24.0,
                    color: Color::WHITE,
                },
            ),
            TextSection::new(
                "Controls:\n",
                TextStyle {
                    font: font.clone(),
                    font_size: 20.0,
                    color: Color::rgb(0.8, 0.8, 1.0),
                },
            ),
            TextSection::new(
                "• Right-click: Open context menu\n",
                TextStyle {
                    font: font.clone(),
                    font_size: 16.0,
                    color: Color::rgb(0.7, 0.7, 0.7),
                },
            ),
            TextSection::new(
                "• Up/Down: Navigate menu items\n",
                TextStyle {
                    font: font.clone(),
                    font_size: 16.0,
                    color: Color::rgb(0.7, 0.7, 0.7),
                },
            ),
            TextSection::new(
                "• Enter: Select item\n",
                TextStyle {
                    font: font.clone(),
                    font_size: 16.0,
                    color: Color::rgb(0.7, 0.7, 0.7),
                },
            ),
            TextSection::new(
                "• Escape: Close menu\n",
                TextStyle {
                    font: font.clone(),
                    font_size: 16.0,
                    color: Color::rgb(0.7, 0.7, 0.7),
                },
            ),
            TextSection::new(
                "• Mouse: Hover and click\n\n",
                TextStyle {
                    font: font.clone(),
                    font_size: 16.0,
                    color: Color::rgb(0.7, 0.7, 0.7),
                },
            ),
            TextSection::new(
                "Actions are logged to console",
                TextStyle {
                    font,
                    font_size: 14.0,
                    color: Color::rgb(0.5, 0.5, 0.5),
                },
            ),
        ])
        .with_style(Style {
            position_type: PositionType::Absolute,
            top: Val::Px(20.0),
            left: Val::Px(20.0),
            ..default()
        }),
        InstructionsText,
    ));
}
