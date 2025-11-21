#![no_std]
// This crate defines the data layout shared between Daemon and Client.
// It must be #[repr(C)] to ensure memory layout consistency across processes.

use bytemuck::{Pod, Zeroable};

pub const SHMEM_PATH: &str = "/scarab_shm_v1";
pub const GRID_WIDTH: usize = 200;
pub const GRID_HEIGHT: usize = 100;
pub const BUFFER_SIZE: usize = GRID_WIDTH * GRID_HEIGHT;

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct Cell {
    pub char_codepoint: u32, 
    pub fg: u32,   // RGBA
    pub bg: u32,   // RGBA
    pub flags: u8, // Bold, Italic, etc.
    pub _padding: [u8; 3], // Align to 16 bytes
}

// A double-buffered grid state living in shared memory
#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct SharedState {
    pub sequence_number: u64, // Atomic sequence for synchronization
    pub dirty_flag: u8,
    pub _padding1: [u8; 1],   // Align to u16 boundary
    pub cursor_x: u16,
    pub cursor_y: u16,
    pub _padding2: [u8; 2],   // Align to u64 boundary for cells array
    // Fixed size buffer for the "visible" screen.
    // In production, use offset pointers to a larger ring buffer.
    pub cells: [Cell; BUFFER_SIZE],
}

// Control messages (Sent via Socket/Pipe, not ShMem)
#[derive(Debug, Clone)]
pub enum ControlMessage {
    Resize { cols: u16, rows: u16 },
    Input { data: alloc::vec::Vec<u8> },
    LoadPlugin { path: alloc::string::String },
}

extern crate alloc;
