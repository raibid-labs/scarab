use bevy::prelude::*;
use scarab_protocol::{SharedState, SHMEM_PATH};
use shared_memory::ShmemConf;
use std::sync::Arc;

// Marker for the grid entity
#[derive(Component)]
struct TerminalGrid;

// Wrapper to make shared memory Send + Sync
struct SharedMemWrapper(Arc<shared_memory::Shmem>);

unsafe impl Send for SharedMemWrapper {}
unsafe impl Sync for SharedMemWrapper {}

// Resource to hold shared memory state
#[derive(Resource)]
struct SharedMemoryReader {
    shmem: SharedMemWrapper,
    last_sequence: u64,
}

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
       .insert_resource(reader)
       .add_systems(Startup, setup)
       .add_systems(Update, (sync_grid, handle_input))
       .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
    commands.spawn((
        TerminalGrid,
        SpriteBundle {
            transform: Transform::from_xyz(0.0, 0.0, 0.0),
            ..default()
        },
    ));
    println!("Scarab Client Initialized with shared memory reader.");
}

fn sync_grid(mut reader: ResMut<SharedMemoryReader>) {
    // Read SharedState from shared memory (zero-copy, lock-free)
    let shared_ptr = reader.shmem.0.as_ptr() as *const SharedState;

    unsafe {
        let state = &*shared_ptr;

        // Check if sequence number changed (atomic read)
        let current_seq = state.sequence_number;

        if current_seq != reader.last_sequence {
            // Grid has been updated by daemon
            println!(
                "Grid updated! Seq: {} -> {}, Cursor: ({}, {})",
                reader.last_sequence, current_seq, state.cursor_x, state.cursor_y
            );

            // TODO: Update texture/mesh from state.cells
            // For now, just update our tracking sequence
            reader.last_sequence = current_seq;

            // Reset dirty flag (optional, depending on your protocol)
            // Note: This is a read-only client, so we don't modify shared memory
        }
    }
}

fn handle_input(keys: Res<ButtonInput<KeyCode>>) {
    // TODO: Send input to Daemon via socket
    if keys.just_pressed(KeyCode::Space) {
        println!("Space pressed - checking for Leader Key...");
    }
}
