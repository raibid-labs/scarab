//! Registry trait and types for managing object lifecycle
//!
//! The registry provides centralized management of object handles and their
//! associated data. Implementations can be in the daemon (for PTY state)
//! or client (for UI state).

use crate::object_model::{ObjectError, ObjectHandle};

/// Entry in the object registry containing the object and its handle
///
/// This struct wraps an object with its associated handle, providing
/// a unified way to store and retrieve objects from the registry.
///
/// # Type Parameters
///
/// * `T` - The type of object being stored (e.g., Window, Tab, Pane)
#[derive(Debug, Clone)]
pub struct RegistryEntry<T> {
    /// The handle for this object
    pub handle: ObjectHandle,
    /// The actual object data
    pub object: T,
}

impl<T> RegistryEntry<T> {
    /// Create a new registry entry
    ///
    /// # Arguments
    ///
    /// * `handle` - The handle for this object
    /// * `object` - The actual object data
    pub fn new(handle: ObjectHandle, object: T) -> Self {
        Self { handle, object }
    }

    /// Get the object's handle
    #[inline]
    pub fn handle(&self) -> ObjectHandle {
        self.handle
    }

    /// Get a reference to the object
    #[inline]
    pub fn object(&self) -> &T {
        &self.object
    }

    /// Get a mutable reference to the object
    #[inline]
    pub fn object_mut(&mut self) -> &mut T {
        &mut self.object
    }

    /// Consume the entry and return the object
    #[inline]
    pub fn into_object(self) -> T {
        self.object
    }

    /// Check if this entry's handle is still valid
    #[inline]
    pub fn is_valid(&self, current_generation: u32) -> bool {
        self.handle.is_valid(current_generation)
    }
}

/// Trait for managing object lifecycle in a registry
///
/// Implementations of this trait provide storage and retrieval of objects
/// by their handles, with support for generation-based invalidation.
///
/// # Type Parameters
///
/// * `T` - The type of object being managed
///
/// # Examples
///
/// ```ignore
/// // Example implementation (not compiled in doctests)
/// struct MyRegistry {
///     objects: HashMap<u64, RegistryEntry<MyObject>>,
///     next_id: u64,
///     generations: HashMap<u64, u32>,
/// }
///
/// impl ObjectRegistry<MyObject> for MyRegistry {
///     fn register(&mut self, object: MyObject) -> ObjectHandle {
///         let id = self.next_id();
///         let generation = self.generations.get(&id).copied().unwrap_or(0);
///         let handle = ObjectHandle::new(ObjectType::Window, id, generation);
///         self.objects.insert(id, RegistryEntry::new(handle, object));
///         handle
///     }
///     // ... other methods
/// }
/// ```
pub trait ObjectRegistry<T> {
    /// Register a new object and return its handle
    ///
    /// This allocates a new ID for the object and returns a handle that can
    /// be used to reference it later.
    ///
    /// # Arguments
    ///
    /// * `object` - The object to register
    ///
    /// # Returns
    ///
    /// A handle that can be used to access the object
    fn register(&mut self, object: T) -> ObjectHandle;

    /// Unregister an object by its handle
    ///
    /// This removes the object from the registry and increments the generation
    /// counter for its ID, invalidating any existing handles.
    ///
    /// # Arguments
    ///
    /// * `handle` - The handle of the object to unregister
    ///
    /// # Returns
    ///
    /// * `Ok(object)` - The removed object if successful
    /// * `Err(ObjectError)` - If the handle is invalid or not found
    fn unregister(&mut self, handle: ObjectHandle) -> Result<T, ObjectError>;

    /// Get a reference to an object by its handle
    ///
    /// # Arguments
    ///
    /// * `handle` - The handle of the object to retrieve
    ///
    /// # Returns
    ///
    /// * `Ok(&T)` - A reference to the object if found and valid
    /// * `Err(ObjectError)` - If the handle is invalid or not found
    fn get(&self, handle: ObjectHandle) -> Result<&T, ObjectError>;

    /// Get a mutable reference to an object by its handle
    ///
    /// # Arguments
    ///
    /// * `handle` - The handle of the object to retrieve
    ///
    /// # Returns
    ///
    /// * `Ok(&mut T)` - A mutable reference to the object if found and valid
    /// * `Err(ObjectError)` - If the handle is invalid or not found
    fn get_mut(&mut self, handle: ObjectHandle) -> Result<&mut T, ObjectError>;

    /// Get an object by its ID, regardless of generation
    ///
    /// This is useful for debugging or administrative operations where you
    /// need to access an object even if the handle might be stale.
    ///
    /// # Arguments
    ///
    /// * `id` - The object ID
    ///
    /// # Returns
    ///
    /// * `Some(&T)` - A reference to the object if found
    /// * `None` - If no object with this ID exists
    fn get_by_id(&self, id: u64) -> Option<&T>;

    /// Get the next available object ID
    ///
    /// This should return a unique ID that hasn't been used yet, or has been
    /// recycled after unregistration.
    ///
    /// # Returns
    ///
    /// A unique object ID
    fn next_id(&mut self) -> u64;

    /// Increment the generation counter for an object ID
    ///
    /// This is called when an object is unregistered to invalidate existing
    /// handles. The next object to use this ID will have a higher generation.
    ///
    /// # Arguments
    ///
    /// * `id` - The object ID whose generation should be incremented
    fn increment_generation(&mut self, id: u64);

    /// Get the current generation for an object ID
    ///
    /// # Arguments
    ///
    /// * `id` - The object ID
    ///
    /// # Returns
    ///
    /// The current generation counter for this ID
    fn current_generation(&self, id: u64) -> u32;

    /// Check if a handle is still valid
    ///
    /// # Arguments
    ///
    /// * `handle` - The handle to validate
    ///
    /// # Returns
    ///
    /// `true` if the handle references a valid object with the correct generation
    fn is_valid(&self, handle: ObjectHandle) -> bool {
        let current_gen = self.current_generation(handle.id());
        handle.is_valid(current_gen) && self.get_by_id(handle.id()).is_some()
    }

    /// Get all registered object IDs
    ///
    /// # Returns
    ///
    /// A vector of all currently registered object IDs
    fn all_ids(&self) -> Vec<u64>;

    /// Get the number of registered objects
    ///
    /// # Returns
    ///
    /// The count of objects currently in the registry
    fn len(&self) -> usize;

    /// Check if the registry is empty
    ///
    /// # Returns
    ///
    /// `true` if no objects are registered
    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Get the parent handle of an object
    ///
    /// This enables navigation up the object hierarchy (e.g., Pane -> Tab -> Window).
    ///
    /// # Arguments
    ///
    /// * `handle` - The handle of the object whose parent to retrieve
    ///
    /// # Returns
    ///
    /// * `Ok(Some(handle))` - The parent object's handle
    /// * `Ok(None)` - The object has no parent (e.g., Window is top-level)
    /// * `Err(ObjectError)` - If the handle is invalid or not found
    fn get_parent(&self, handle: &ObjectHandle) -> Result<Option<ObjectHandle>, ObjectError>;

    /// Get the child handles of an object
    ///
    /// This enables navigation down the object hierarchy (e.g., Window -> Tabs -> Panes).
    ///
    /// # Arguments
    ///
    /// * `handle` - The handle of the object whose children to retrieve
    ///
    /// # Returns
    ///
    /// * `Ok(Vec<ObjectHandle>)` - A vector of child object handles (may be empty)
    /// * `Err(ObjectError)` - If the handle is invalid or not found
    fn get_children(&self, handle: &ObjectHandle) -> Result<Vec<ObjectHandle>, ObjectError>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::object_model::handle::ObjectType;
    use std::collections::HashMap;

    // Simple test object
    #[derive(Debug, Clone, PartialEq)]
    struct TestObject {
        name: String,
        value: i32,
    }

    // Simple registry implementation for testing
    struct TestRegistry {
        objects: HashMap<u64, RegistryEntry<TestObject>>,
        next_id_counter: u64,
        generations: HashMap<u64, u32>,
        parents: HashMap<u64, ObjectHandle>,
        children: HashMap<u64, Vec<ObjectHandle>>,
    }

    impl TestRegistry {
        fn new() -> Self {
            Self {
                objects: HashMap::new(),
                next_id_counter: 1,
                generations: HashMap::new(),
                parents: HashMap::new(),
                children: HashMap::new(),
            }
        }

        fn set_parent(&mut self, child: ObjectHandle, parent: ObjectHandle) {
            self.parents.insert(child.id(), parent);
            self.children
                .entry(parent.id())
                .or_insert_with(Vec::new)
                .push(child);
        }
    }

    impl ObjectRegistry<TestObject> for TestRegistry {
        fn register(&mut self, object: TestObject) -> ObjectHandle {
            let id = self.next_id();
            let generation = self.current_generation(id);
            let handle = ObjectHandle::new(ObjectType::Window, id, generation);
            self.objects.insert(id, RegistryEntry::new(handle, object));
            handle
        }

        fn unregister(&mut self, handle: ObjectHandle) -> Result<TestObject, ObjectError> {
            let current_gen = self.current_generation(handle.id());
            if !handle.is_valid(current_gen) {
                return Err(ObjectError::stale_handle(handle, current_gen));
            }

            self.increment_generation(handle.id());
            self.objects
                .remove(&handle.id())
                .map(|entry| entry.into_object())
                .ok_or_else(|| ObjectError::not_found(handle))
        }

        fn get(&self, handle: ObjectHandle) -> Result<&TestObject, ObjectError> {
            let current_gen = self.current_generation(handle.id());
            if !handle.is_valid(current_gen) {
                return Err(ObjectError::stale_handle(handle, current_gen));
            }

            self.objects
                .get(&handle.id())
                .map(|entry| entry.object())
                .ok_or_else(|| ObjectError::not_found(handle))
        }

        fn get_mut(&mut self, handle: ObjectHandle) -> Result<&mut TestObject, ObjectError> {
            let current_gen = self.current_generation(handle.id());
            if !handle.is_valid(current_gen) {
                return Err(ObjectError::stale_handle(handle, current_gen));
            }

            self.objects
                .get_mut(&handle.id())
                .map(|entry| entry.object_mut())
                .ok_or_else(|| ObjectError::not_found(handle))
        }

        fn get_by_id(&self, id: u64) -> Option<&TestObject> {
            self.objects.get(&id).map(|entry| entry.object())
        }

        fn next_id(&mut self) -> u64 {
            let id = self.next_id_counter;
            self.next_id_counter += 1;
            id
        }

        fn increment_generation(&mut self, id: u64) {
            let gen = self.generations.entry(id).or_insert(0);
            *gen = gen.wrapping_add(1);
        }

        fn current_generation(&self, id: u64) -> u32 {
            self.generations.get(&id).copied().unwrap_or(0)
        }

        fn all_ids(&self) -> Vec<u64> {
            self.objects.keys().copied().collect()
        }

        fn len(&self) -> usize {
            self.objects.len()
        }

        fn get_parent(&self, handle: &ObjectHandle) -> Result<Option<ObjectHandle>, ObjectError> {
            let current_gen = self.current_generation(handle.id());
            if !handle.is_valid(current_gen) {
                return Err(ObjectError::stale_handle(*handle, current_gen));
            }

            if !self.objects.contains_key(&handle.id()) {
                return Err(ObjectError::not_found(*handle));
            }

            Ok(self.parents.get(&handle.id()).copied())
        }

        fn get_children(
            &self,
            handle: &ObjectHandle,
        ) -> Result<Vec<ObjectHandle>, ObjectError> {
            let current_gen = self.current_generation(handle.id());
            if !handle.is_valid(current_gen) {
                return Err(ObjectError::stale_handle(*handle, current_gen));
            }

            if !self.objects.contains_key(&handle.id()) {
                return Err(ObjectError::not_found(*handle));
            }

            Ok(self
                .children
                .get(&handle.id())
                .cloned()
                .unwrap_or_else(Vec::new))
        }
    }

    #[test]
    fn test_registry_entry_creation() {
        let handle = ObjectHandle::new(ObjectType::Window, 1, 0);
        let obj = TestObject {
            name: "test".to_string(),
            value: 42,
        };
        let entry = RegistryEntry::new(handle, obj.clone());

        assert_eq!(entry.handle(), handle);
        assert_eq!(entry.object(), &obj);
    }

    #[test]
    fn test_registry_entry_mutability() {
        let handle = ObjectHandle::new(ObjectType::Window, 1, 0);
        let obj = TestObject {
            name: "test".to_string(),
            value: 42,
        };
        let mut entry = RegistryEntry::new(handle, obj);

        entry.object_mut().value = 100;
        assert_eq!(entry.object().value, 100);
    }

    #[test]
    fn test_register_and_get() {
        let mut registry = TestRegistry::new();
        let obj = TestObject {
            name: "test".to_string(),
            value: 42,
        };

        let handle = registry.register(obj.clone());
        let retrieved = registry.get(handle).unwrap();

        assert_eq!(retrieved, &obj);
    }

    #[test]
    fn test_unregister() {
        let mut registry = TestRegistry::new();
        let obj = TestObject {
            name: "test".to_string(),
            value: 42,
        };

        let handle = registry.register(obj.clone());
        assert_eq!(registry.len(), 1);

        let removed = registry.unregister(handle).unwrap();
        assert_eq!(removed, obj);
        assert_eq!(registry.len(), 0);
    }

    #[test]
    fn test_stale_handle_after_unregister() {
        let mut registry = TestRegistry::new();
        let obj = TestObject {
            name: "test".to_string(),
            value: 42,
        };

        let handle = registry.register(obj.clone());
        registry.unregister(handle).unwrap();

        // Old handle should now be stale
        let result = registry.get(handle);
        assert!(matches!(result, Err(ObjectError::StaleHandle { .. })));
    }

    #[test]
    fn test_generation_increment() {
        let mut registry = TestRegistry::new();

        let obj1 = TestObject {
            name: "first".to_string(),
            value: 1,
        };
        let obj2 = TestObject {
            name: "second".to_string(),
            value: 2,
        };

        let handle1 = registry.register(obj1);
        let id = handle1.id();

        registry.unregister(handle1).unwrap();

        // The first ID's generation should have been incremented
        assert_eq!(registry.current_generation(id), 1);

        // Second object gets a new ID (simple registry doesn't reuse IDs)
        let handle2 = registry.register(obj2);
        assert_ne!(handle2.id(), id);
        assert_eq!(handle2.generation(), 0); // New ID starts at generation 0
    }

    #[test]
    fn test_get_by_id() {
        let mut registry = TestRegistry::new();
        let obj = TestObject {
            name: "test".to_string(),
            value: 42,
        };

        let handle = registry.register(obj.clone());
        let retrieved = registry.get_by_id(handle.id()).unwrap();

        assert_eq!(retrieved, &obj);
    }

    #[test]
    fn test_is_valid() {
        let mut registry = TestRegistry::new();
        let obj = TestObject {
            name: "test".to_string(),
            value: 42,
        };

        let handle = registry.register(obj);
        assert!(registry.is_valid(handle));

        registry.unregister(handle).unwrap();
        assert!(!registry.is_valid(handle));
    }

    #[test]
    fn test_all_ids() {
        let mut registry = TestRegistry::new();

        let h1 = registry.register(TestObject {
            name: "one".to_string(),
            value: 1,
        });
        let h2 = registry.register(TestObject {
            name: "two".to_string(),
            value: 2,
        });

        let mut ids = registry.all_ids();
        ids.sort();

        assert_eq!(ids.len(), 2);
        assert!(ids.contains(&h1.id()));
        assert!(ids.contains(&h2.id()));
    }

    #[test]
    fn test_len_and_is_empty() {
        let mut registry = TestRegistry::new();
        assert!(registry.is_empty());
        assert_eq!(registry.len(), 0);

        let handle = registry.register(TestObject {
            name: "test".to_string(),
            value: 42,
        });
        assert!(!registry.is_empty());
        assert_eq!(registry.len(), 1);

        registry.unregister(handle).unwrap();
        assert!(registry.is_empty());
        assert_eq!(registry.len(), 0);
    }

    #[test]
    fn test_get_mut() {
        let mut registry = TestRegistry::new();
        let obj = TestObject {
            name: "test".to_string(),
            value: 42,
        };

        let handle = registry.register(obj);

        {
            let obj_mut = registry.get_mut(handle).unwrap();
            obj_mut.value = 100;
        }

        let obj = registry.get(handle).unwrap();
        assert_eq!(obj.value, 100);
    }

    #[test]
    fn test_get_parent_none() {
        let mut registry = TestRegistry::new();
        let obj = TestObject {
            name: "root".to_string(),
            value: 1,
        };

        let handle = registry.register(obj);
        let parent = registry.get_parent(&handle).unwrap();
        assert!(parent.is_none());
    }

    #[test]
    fn test_get_parent_some() {
        let mut registry = TestRegistry::new();

        let parent_obj = TestObject {
            name: "parent".to_string(),
            value: 1,
        };
        let child_obj = TestObject {
            name: "child".to_string(),
            value: 2,
        };

        let parent_handle = registry.register(parent_obj);
        let child_handle = registry.register(child_obj);

        registry.set_parent(child_handle, parent_handle);

        let retrieved_parent = registry.get_parent(&child_handle).unwrap();
        assert_eq!(retrieved_parent, Some(parent_handle));
    }

    #[test]
    fn test_get_parent_stale_handle() {
        let mut registry = TestRegistry::new();
        let obj = TestObject {
            name: "test".to_string(),
            value: 42,
        };

        let handle = registry.register(obj);
        registry.unregister(handle).unwrap();

        let result = registry.get_parent(&handle);
        assert!(matches!(result, Err(ObjectError::StaleHandle { .. })));
    }

    #[test]
    fn test_get_children_empty() {
        let mut registry = TestRegistry::new();
        let obj = TestObject {
            name: "leaf".to_string(),
            value: 1,
        };

        let handle = registry.register(obj);
        let children = registry.get_children(&handle).unwrap();
        assert!(children.is_empty());
    }

    #[test]
    fn test_get_children_multiple() {
        let mut registry = TestRegistry::new();

        let parent_obj = TestObject {
            name: "parent".to_string(),
            value: 1,
        };
        let child1_obj = TestObject {
            name: "child1".to_string(),
            value: 2,
        };
        let child2_obj = TestObject {
            name: "child2".to_string(),
            value: 3,
        };

        let parent_handle = registry.register(parent_obj);
        let child1_handle = registry.register(child1_obj);
        let child2_handle = registry.register(child2_obj);

        registry.set_parent(child1_handle, parent_handle);
        registry.set_parent(child2_handle, parent_handle);

        let children = registry.get_children(&parent_handle).unwrap();
        assert_eq!(children.len(), 2);
        assert!(children.contains(&child1_handle));
        assert!(children.contains(&child2_handle));
    }

    #[test]
    fn test_get_children_stale_handle() {
        let mut registry = TestRegistry::new();
        let obj = TestObject {
            name: "test".to_string(),
            value: 42,
        };

        let handle = registry.register(obj);
        registry.unregister(handle).unwrap();

        let result = registry.get_children(&handle);
        assert!(matches!(result, Err(ObjectError::StaleHandle { .. })));
    }

    #[test]
    fn test_navigation_hierarchy() {
        let mut registry = TestRegistry::new();

        // Create a hierarchy: window -> tab -> pane
        let window_handle = registry.register(TestObject {
            name: "window".to_string(),
            value: 1,
        });
        let tab_handle = registry.register(TestObject {
            name: "tab".to_string(),
            value: 2,
        });
        let pane_handle = registry.register(TestObject {
            name: "pane".to_string(),
            value: 3,
        });

        registry.set_parent(tab_handle, window_handle);
        registry.set_parent(pane_handle, tab_handle);

        // Navigate up from pane to tab
        let pane_parent = registry.get_parent(&pane_handle).unwrap().unwrap();
        assert_eq!(pane_parent, tab_handle);

        // Navigate up from tab to window
        let tab_parent = registry.get_parent(&tab_handle).unwrap().unwrap();
        assert_eq!(tab_parent, window_handle);

        // Window has no parent
        let window_parent = registry.get_parent(&window_handle).unwrap();
        assert!(window_parent.is_none());

        // Navigate down from window to tab
        let window_children = registry.get_children(&window_handle).unwrap();
        assert_eq!(window_children.len(), 1);
        assert_eq!(window_children[0], tab_handle);

        // Navigate down from tab to pane
        let tab_children = registry.get_children(&tab_handle).unwrap();
        assert_eq!(tab_children.len(), 1);
        assert_eq!(tab_children[0], pane_handle);
    }
}
