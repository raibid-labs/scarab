use bevy::prelude::*;
use scarab_protocol::SharedState;

// Marker for the grid entity
#[derive(Component)]
struct TerminalGrid;

fn main() {
    App::new()
       .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Scarab Terminal".into(),
                resolution: (1024.0, 768.0).into(),
               ..default()
            }),
           ..default()
        }))
       .add_systems(Startup, setup)
       .add_systems(Update, (sync_grid, handle_input))
       .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
    println!("Scarab Client Initialized.");

    // TODO: Initialize Shared Memory reader here
}

fn sync_grid() {
    // TODO: Check SharedState sequence number
    // If changed, update the texture/mesh
}

fn handle_input(keys: Res<Input<KeyCode>>) {
    // TODO: Send input to Daemon via socket
    if keys.just_pressed(KeyCode::Space) {
        println!("Space pressed - checking for Leader Key...");
    }
}
