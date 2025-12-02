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
//! - **EventRegistry**: Central dispatch system for managing handlers
//!
//! # Example
//!
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

mod types;
mod args;
mod registry;
mod handler;

pub use types::EventType;
pub use args::{EventArgs, EventData};
pub use registry::EventRegistry;
pub use handler::{EventHandler, EventResult, HandlerEntry};
