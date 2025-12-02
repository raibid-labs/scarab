//! Daemon-side event dispatching
//!
//! Handles event dispatch on the daemon side, including forwarding events to clients via IPC.

mod dispatcher;

pub use dispatcher::DaemonEventDispatcher;
