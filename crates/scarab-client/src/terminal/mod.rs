// Terminal-specific modules
// Handles scrollback buffer, history management, and terminal state tracking

pub mod chunks;
pub mod scrollback;

pub use chunks::{ChunkGrid, ChunkMesh, ChunkPlugin, TerminalChunk, CHUNK_HEIGHT, CHUNK_WIDTH, CHUNKS_X, CHUNKS_Y};
pub use scrollback::{ScrollbackBuffer, ScrollbackLine, ScrollbackPlugin, ScrollbackState};
