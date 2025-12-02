// Public modules
pub mod events;
pub mod images;
pub mod ipc;
pub mod plugin_manager;
pub mod profiling;
pub mod session;
pub mod vte;
pub mod vte_optimized;

// Re-export key types
pub use events::DaemonEventDispatcher;
