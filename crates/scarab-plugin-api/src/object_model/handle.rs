//! Object handles for type-safe references to terminal objects
//!
//! Handles provide a safe way to reference objects across the daemon-client boundary
//! and within Fusabi scripts. Each handle includes a generation counter to detect
//! stale references after object deletion/recreation.

use std::fmt;

/// Type of object this handle references
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum ObjectType {
    /// A top-level window (GUI window)
    Window = 0,
    /// A tab within a window
    Tab = 1,
    /// A pane within a tab
    Pane = 2,
    /// A multiplexer window (logical container)
    MuxWindow = 3,
    /// A multiplexer tab
    MuxTab = 4,
    /// A multiplexer pane
    MuxPane = 5,
}

impl fmt::Display for ObjectType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ObjectType::Window => write!(f, "Window"),
            ObjectType::Tab => write!(f, "Tab"),
            ObjectType::Pane => write!(f, "Pane"),
            ObjectType::MuxWindow => write!(f, "MuxWindow"),
            ObjectType::MuxTab => write!(f, "MuxTab"),
            ObjectType::MuxPane => write!(f, "MuxPane"),
        }
    }
}

/// A handle to an object managed by the object registry
///
/// Handles are lightweight, copyable references that include:
/// - `object_type`: The type of object being referenced
/// - `id`: Unique identifier for this object
/// - `generation`: Counter incremented each time an ID is reused
///
/// The generation counter prevents the ABA problem where an object is deleted
/// and a new object reuses the same ID.
///
/// # Examples
///
/// ```
/// use scarab_plugin_api::object_model::{ObjectHandle, ObjectType};
///
/// let handle = ObjectHandle::new(ObjectType::Window, 42, 1);
/// assert_eq!(handle.id(), 42);
/// assert_eq!(handle.object_type(), ObjectType::Window);
/// assert_eq!(handle.generation(), 1);
/// ```
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct ObjectHandle {
    object_type: ObjectType,
    id: u64,
    generation: u32,
}

impl ObjectHandle {
    /// Create a new object handle
    ///
    /// # Arguments
    ///
    /// * `object_type` - The type of object this handle references
    /// * `id` - Unique identifier for this object
    /// * `generation` - Generation counter for detecting stale handles
    pub const fn new(object_type: ObjectType, id: u64, generation: u32) -> Self {
        Self {
            object_type,
            id,
            generation,
        }
    }

    /// Get the object type
    #[inline]
    pub const fn object_type(&self) -> ObjectType {
        self.object_type
    }

    /// Get the object ID
    #[inline]
    pub const fn id(&self) -> u64 {
        self.id
    }

    /// Get the generation counter
    #[inline]
    pub const fn generation(&self) -> u32 {
        self.generation
    }

    /// Check if this handle is valid for the given generation
    ///
    /// Returns true if the handle's generation matches the provided generation,
    /// indicating the handle is still valid and hasn't been invalidated by
    /// object deletion/recreation.
    ///
    /// # Examples
    ///
    /// ```
    /// use scarab_plugin_api::object_model::{ObjectHandle, ObjectType};
    ///
    /// let handle = ObjectHandle::new(ObjectType::Window, 1, 5);
    /// assert!(handle.is_valid(5));
    /// assert!(!handle.is_valid(6));
    /// ```
    #[inline]
    pub const fn is_valid(&self, current_generation: u32) -> bool {
        self.generation == current_generation
    }

    /// Create a new handle with an incremented generation
    ///
    /// This is useful when an object is deleted and its ID is reused for a new object.
    #[inline]
    pub const fn next_generation(&self) -> Self {
        Self {
            object_type: self.object_type,
            id: self.id,
            generation: self.generation.wrapping_add(1),
        }
    }
}

impl fmt::Debug for ObjectHandle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "ObjectHandle({}, id={}, gen={})",
            self.object_type, self.id, self.generation
        )
    }
}

impl fmt::Display for ObjectHandle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}#{}", self.object_type, self.id)
    }
}

// Safety: ObjectHandle contains only primitive types that are Send + Sync
unsafe impl Send for ObjectHandle {}
unsafe impl Sync for ObjectHandle {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_handle_creation() {
        let handle = ObjectHandle::new(ObjectType::Window, 42, 1);
        assert_eq!(handle.object_type(), ObjectType::Window);
        assert_eq!(handle.id(), 42);
        assert_eq!(handle.generation(), 1);
    }

    #[test]
    fn test_handle_equality() {
        let h1 = ObjectHandle::new(ObjectType::Tab, 10, 1);
        let h2 = ObjectHandle::new(ObjectType::Tab, 10, 1);
        let h3 = ObjectHandle::new(ObjectType::Tab, 10, 2);
        let h4 = ObjectHandle::new(ObjectType::Pane, 10, 1);

        assert_eq!(h1, h2);
        assert_ne!(h1, h3); // Different generation
        assert_ne!(h1, h4); // Different type
    }

    #[test]
    fn test_handle_validation() {
        let handle = ObjectHandle::new(ObjectType::Window, 1, 5);
        assert!(handle.is_valid(5));
        assert!(!handle.is_valid(4));
        assert!(!handle.is_valid(6));
    }

    #[test]
    fn test_next_generation() {
        let h1 = ObjectHandle::new(ObjectType::Pane, 100, 1);
        let h2 = h1.next_generation();

        assert_eq!(h2.id(), 100);
        assert_eq!(h2.object_type(), ObjectType::Pane);
        assert_eq!(h2.generation(), 2);
        assert!(!h1.is_valid(2));
        assert!(h2.is_valid(2));
    }

    #[test]
    fn test_generation_wrapping() {
        let handle = ObjectHandle::new(ObjectType::Window, 1, u32::MAX);
        let next = handle.next_generation();
        assert_eq!(next.generation(), 0); // Wrapped around
    }

    #[test]
    fn test_handle_copy() {
        let h1 = ObjectHandle::new(ObjectType::Tab, 5, 3);
        let h2 = h1; // Should copy, not move
        assert_eq!(h1, h2);
        assert_eq!(h1.id(), h2.id()); // h1 still valid
    }

    #[test]
    fn test_object_type_display() {
        assert_eq!(ObjectType::Window.to_string(), "Window");
        assert_eq!(ObjectType::Tab.to_string(), "Tab");
        assert_eq!(ObjectType::Pane.to_string(), "Pane");
        assert_eq!(ObjectType::MuxWindow.to_string(), "MuxWindow");
        assert_eq!(ObjectType::MuxTab.to_string(), "MuxTab");
        assert_eq!(ObjectType::MuxPane.to_string(), "MuxPane");
    }

    #[test]
    fn test_handle_display() {
        let handle = ObjectHandle::new(ObjectType::Window, 42, 1);
        assert_eq!(handle.to_string(), "Window#42");
    }

    #[test]
    fn test_handle_debug() {
        let handle = ObjectHandle::new(ObjectType::Pane, 7, 3);
        let debug_str = format!("{:?}", handle);
        assert!(debug_str.contains("Pane"));
        assert!(debug_str.contains("id=7"));
        assert!(debug_str.contains("gen=3"));
    }
}
