//! Object Model for exposing terminal state to Fusabi scripts
//!
//! This module provides handle-based proxies that allow Fusabi scripts
//! to interact with Window, Pane, and Tab objects.
//!
//! # Architecture
//!
//! The object model uses a handle-based approach similar to WezTerm's Lua API:
//!
//! 1. **Handles**: Lightweight, copyable references to objects
//! 2. **Registry**: Centralized storage for objects with lifecycle management
//! 3. **Generation Counters**: Detect stale handles after object deletion
//!
//! # Example
//!
//! ```
//! use scarab_plugin_api::object_model::{ObjectHandle, ObjectType, WindowProxy};
//!
//! // Create a handle
//! let handle = ObjectHandle::new(ObjectType::Window, 1, 0);
//!
//! // Create a proxy from the handle
//! let window = WindowProxy::new(handle).unwrap();
//!
//! // Check validity
//! assert!(handle.is_valid(0));
//! assert!(!handle.is_valid(1));
//! ```
//!
//! # Handle-Based Design
//!
//! Instead of passing raw object references to scripts, we use handles:
//!
//! - **Safety**: Scripts can't hold dangling references
//! - **Serialization**: Handles can cross process boundaries
//! - **Invalidation**: Generation counters detect deleted objects
//! - **Type Safety**: ObjectType enum prevents type confusion
//!
//! # Lifecycle Management
//!
//! Objects follow this lifecycle:
//!
//! 1. **Registration**: Object is stored, handle is returned
//! 2. **Access**: Handle is used to retrieve object from registry
//! 3. **Unregistration**: Object is removed, generation is incremented
//! 4. **Invalidation**: Old handles fail validation checks
//!
//! # Thread Safety
//!
//! All types in this module are designed to be `Send + Sync`:
//!
//! - Handles contain only primitive types
//! - Registry trait can be implemented with appropriate locking
//! - Errors are cloneable for propagation across threads

mod error;
mod handle;
mod pane;
mod registry;
mod tab;
mod window;

pub use error::{ObjectError, Result};
pub use handle::{ObjectHandle, ObjectType};
pub use pane::PaneProxy;
pub use registry::{ObjectRegistry, RegistryEntry};
pub use tab::TabProxy;
pub use window::{RenderItem, WindowProxy};
