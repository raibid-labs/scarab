use bevy::prelude::*;
use scarab_protocol::{SharedState, SHMEM_PATH};
use shared_memory::ShmemConf;
use std::sync::Arc;
use scarab_client::AdvancedUIPlugin;
use scarab_client::integration::{SharedMemoryReader, SharedMemWrapper, IntegrationPlugin};

mod ipc;
use ipc::IpcPlugin;

fn main() {
    // Initialize shared memory before Bevy app starts
    let shmem = match ShmemConf::new()
        .size(std::mem::size_of::<SharedState>())
        .os_id(SHMEM_PATH)
        .open()
    {
        Ok(m) => {
            println!("Connected to shared memory at: {}", SHMEM_PATH);
            Arc::new(m)
        }
        Err(e) => {
            eprintln!("Failed to open shared memory: {}. Is the daemon running?", e);
            std::process::exit(1);
        }
    };

    // Initialize the resource from the library crate
    let reader = SharedMemoryReader {
        shmem: SharedMemWrapper(shmem),
        last_sequence: 0,
    };

    App::new()
       .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Scarab Terminal".into(),
                resolution: (1024.0, 768.0).into(),
               ..default()
            }),
           ..default()
        }))
       .add_plugins(IpcPlugin) // Add IPC support
       .add_plugins(AdvancedUIPlugin) // Add advanced UI features
       .add_plugins(IntegrationPlugin) // Add text rendering
       .insert_resource(reader)
       .add_systems(Startup, setup)
       .run();
}

fn setup(mut commands: Commands) {
    // Bevy 0.15 uses Camera2d component, Camera2dBundle is deprecated
    commands.spawn(Camera2d::default());
    // IntegrationPlugin handles spawning the TerminalGridEntity
    println!("Scarab Client Initialized with shared memory reader and IPC.");
}