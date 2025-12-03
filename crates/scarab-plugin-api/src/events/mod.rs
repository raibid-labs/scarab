//! Rich Event System
//!
//! Provides granular events for plugins to hook into terminal behavior,
//! matching WezTerm's event capabilities.
//!
//! # Architecture
//!
//! The event system consists of:
//! - **EventType**: Enum of all available event types (20+ granular events)
//! - **EventArgs**: Event context passed to handlers (window, pane, tab, data)
//! - **EventHandler**: Function type for event callbacks
//! - **EventRegistry**: ⚠️ **DEPRECATED** - Legacy mutex-based dispatch (daemon only)
//!
//! # ⚠️ DEPRECATION NOTICE
//!
//! The `EventRegistry` pattern using `Arc<Mutex<EventRegistry>>` is **deprecated** in favor
//! of pure Bevy ECS events. This provides:
//! - **Lock-free**: No mutex contention or blocking
//! - **Type-safe**: Compile-time event type checking
//! - **ECS-native**: Integrates seamlessly with Bevy's parallel scheduler
//!
//! ## Migration Guide
//!
//! **Old pattern (deprecated):**
//! ```ignore
//! use scarab_plugin_api::events::{EventRegistry, EventType, EventArgs, EventResult};
//! use scarab_plugin_api::object_model::ObjectHandle;
//!
//! let mut registry = EventRegistry::new();
//!
//! // Register a handler for window creation
//! registry.register(
//!     EventType::WindowCreated,
//!     100, // priority
//!     "my-plugin",
//!     Box::new(|args| {
//!         println!("Window created: {:?}", args.window);
//!         EventResult::Continue
//!     })
//! );
//!
//! // Dispatch an event
//! let args = EventArgs::new(EventType::WindowCreated);
//! let results = registry.dispatch(&args);
//! ```
//!
//! **New pattern (Bevy ECS):**
//! ```ignore
//! use bevy::prelude::*;
//! use scarab_client::events::WindowCreatedEvent;
//!
//! // In your plugin's build() method:
//! app.add_systems(Update, handle_window_created);
//!
//! // Handler system (runs in parallel with other systems):
//! fn handle_window_created(mut events: EventReader<WindowCreatedEvent>) {
//!     for event in events.read() {
//!         println!("Window created: {:?}", event.window);
//!     }
//! }
//! ```
//!
//! ## Where to Use Each Pattern
//!
//! - **Daemon plugins**: Continue using `DaemonEventDispatcher` (wraps `EventRegistry`)
//! - **Client code**: Always use Bevy events from `scarab-client/src/events/bevy_events.rs`
//! - **New features**: Prefer Bevy events for lock-free, type-safe event handling
//!
//! See `crates/scarab-client/src/events/bevy_events.rs` for all available typed events.

mod types;
mod args;
mod registry;
mod handler;

pub use types::EventType;
pub use args::{EventArgs, EventData};
#[allow(deprecated)]
pub use registry::EventRegistry;
pub use handler::{EventHandler, EventResult, HandlerEntry};
