use crate::ipc::RemoteMessageEvent;
use bevy::prelude::*;
use scarab_protocol::{DaemonMessage, OverlayStyle};

/// Component to tag entities as remote overlays
#[derive(Component)]
pub struct RemoteOverlay {
    pub id: u64,
}

pub struct RemoteUiPlugin;

impl Plugin for RemoteUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, handle_remote_messages);
    }
}

fn handle_remote_messages(
    mut commands: Commands,
    mut events: EventReader<RemoteMessageEvent>,
    mut show_modal_events: EventWriter<crate::ui::command_palette::ShowRemoteModalEvent>,
    overlay_query: Query<(Entity, &RemoteOverlay)>,
    asset_server: Res<AssetServer>,
) {
    for event in events.read() {
        match &event.0 {
            DaemonMessage::DrawOverlay {
                id,
                x,
                y,
                text,
                style,
            } => {
                // Calculate pixel position (Assuming 8x16 font for now - should come from config/resources)
                let char_width = 8.0;
                let char_height = 16.0;

                // Hack: Just spawning text for now. Position will be wrong until we get window size.
                let pixel_x = *x as f32 * char_width;
                let pixel_y = *y as f32 * -char_height; // Y is down in terminal, Up in Bevy

                commands.spawn((
                    Text2d::default(),
                    Text::new(text.as_str()),
                    TextFont {
                        font: asset_server.load("fonts/FiraCode-Regular.ttf"),
                        font_size: 16.0,
                        ..default()
                    },
                    TextColor(Color::srgba_u8(
                        (style.fg >> 24) as u8,
                        (style.fg >> 16) as u8,
                        (style.fg >> 8) as u8,
                        (style.fg) as u8,
                    )),
                    Transform::from_xyz(pixel_x, pixel_y, style.z_index),
                    RemoteOverlay { id: *id },
                ));
            }
            DaemonMessage::ClearOverlays { id } => {
                for (entity, overlay) in overlay_query.iter() {
                    if let Some(target_id) = id {
                        if overlay.id == *target_id {
                            commands.entity(entity).despawn_recursive();
                        }
                    } else {
                        // Clear all
                        commands.entity(entity).despawn_recursive();
                    }
                }
            }
            DaemonMessage::ShowModal { title, items } => {
                show_modal_events.send(crate::ui::command_palette::ShowRemoteModalEvent {
                    title: title.clone(),
                    items: items.clone(),
                });
            }
            DaemonMessage::HideModal => {
                // TODO: Hide modal event
            }
            _ => {} // Ignore others
        }
    }
}
