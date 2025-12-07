//! Tab object proxy for Fusabi scripts

use super::{
    ObjectError, ObjectHandle, ObjectRegistry, ObjectType, PaneProxy, Result, WindowProxy,
};

/// Proxy for a terminal tab
#[derive(Debug, Clone)]
pub struct TabProxy {
    handle: ObjectHandle,
}

impl TabProxy {
    pub fn new(handle: ObjectHandle) -> Result<Self> {
        if handle.object_type() != ObjectType::Tab {
            return Err(ObjectError::type_mismatch(
                handle,
                ObjectType::Tab,
                handle.object_type(),
            ));
        }
        Ok(Self { handle })
    }

    pub fn handle(&self) -> ObjectHandle {
        self.handle
    }

    pub fn id(&self) -> u64 {
        self.handle.id()
    }

    /// Get tab title
    pub fn get_title(&self) -> Result<String> {
        Err(ObjectError::method_not_found(self.handle, "get_title"))
    }

    /// Set tab title
    pub fn set_title(&self, _title: &str) -> Result<()> {
        Err(ObjectError::method_not_found(self.handle, "set_title"))
    }

    /// Get all panes in this tab
    ///
    /// # Arguments
    ///
    /// * `registry` - The object registry to use for navigation
    ///
    /// # Returns
    ///
    /// * `Ok(Vec<PaneProxy>)` - All panes in this tab (may be empty)
    /// * `Err(ObjectError)` - If navigation fails
    pub fn panes<T>(&self, registry: &impl ObjectRegistry<T>) -> Result<Vec<PaneProxy>> {
        let child_handles = registry.get_children(&self.handle)?;

        child_handles
            .into_iter()
            .map(|handle| {
                if handle.object_type() != ObjectType::Pane {
                    return Err(ObjectError::type_mismatch(
                        handle,
                        ObjectType::Pane,
                        handle.object_type(),
                    ));
                }
                PaneProxy::new(handle)
            })
            .collect()
    }

    /// Get active pane
    ///
    /// This method returns the first pane in the tab.
    /// In a full implementation, this would query which pane is actually focused.
    ///
    /// # Arguments
    ///
    /// * `registry` - The object registry to use for navigation
    ///
    /// # Returns
    ///
    /// * `Ok(PaneProxy)` - The active pane in this tab
    /// * `Err(ObjectError)` - If the tab has no panes or navigation fails
    pub fn active_pane<T>(&self, registry: &impl ObjectRegistry<T>) -> Result<PaneProxy> {
        let panes = self.panes(registry)?;
        panes
            .into_iter()
            .next()
            .ok_or_else(|| ObjectError::method_not_found(self.handle, "active_pane: no panes"))
    }

    /// Switch to this tab
    pub fn activate(&self) -> Result<()> {
        Err(ObjectError::method_not_found(self.handle, "activate"))
    }

    /// Check if this tab is active
    pub fn is_active(&self) -> Result<bool> {
        Err(ObjectError::method_not_found(self.handle, "is_active"))
    }

    /// Get parent window
    ///
    /// # Arguments
    ///
    /// * `registry` - The object registry to use for navigation
    ///
    /// # Returns
    ///
    /// * `Ok(WindowProxy)` - The window containing this tab
    /// * `Err(ObjectError)` - If the tab has no parent or navigation fails
    pub fn window<T>(&self, registry: &impl ObjectRegistry<T>) -> Result<WindowProxy> {
        let parent_handle = registry
            .get_parent(&self.handle)?
            .ok_or_else(|| ObjectError::method_not_found(self.handle, "window: no parent"))?;

        if parent_handle.object_type() != ObjectType::Window {
            return Err(ObjectError::type_mismatch(
                parent_handle,
                ObjectType::Window,
                parent_handle.object_type(),
            ));
        }

        WindowProxy::new(parent_handle)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tab_proxy_creation() {
        let handle = ObjectHandle::new(ObjectType::Tab, 1, 0);
        let proxy = TabProxy::new(handle);
        assert!(proxy.is_ok());

        let proxy = proxy.unwrap();
        assert_eq!(proxy.id(), 1);
        assert_eq!(proxy.handle(), handle);
    }

    #[test]
    fn test_tab_proxy_type_validation() {
        let pane_handle = ObjectHandle::new(ObjectType::Pane, 1, 0);
        let result = TabProxy::new(pane_handle);

        assert!(result.is_err());
        match result {
            Err(ObjectError::TypeMismatch {
                expected, actual, ..
            }) => {
                assert_eq!(expected, ObjectType::Tab);
                assert_eq!(actual, ObjectType::Pane);
            }
            _ => panic!("Expected TypeMismatch error"),
        }
    }

    #[test]
    fn test_tab_proxy_methods_not_implemented() {
        let handle = ObjectHandle::new(ObjectType::Tab, 1, 0);
        let proxy = TabProxy::new(handle).unwrap();

        // All methods should return MethodNotFound errors
        assert!(matches!(
            proxy.get_title(),
            Err(ObjectError::MethodNotFound { .. })
        ));
        assert!(matches!(
            proxy.set_title("test"),
            Err(ObjectError::MethodNotFound { .. })
        ));
        assert!(matches!(
            proxy.activate(),
            Err(ObjectError::MethodNotFound { .. })
        ));
        assert!(matches!(
            proxy.is_active(),
            Err(ObjectError::MethodNotFound { .. })
        ));
    }

    #[test]
    fn test_tab_proxy_clone() {
        let handle = ObjectHandle::new(ObjectType::Tab, 1, 0);
        let proxy1 = TabProxy::new(handle).unwrap();
        let proxy2 = proxy1.clone();

        assert_eq!(proxy1.handle(), proxy2.handle());
        assert_eq!(proxy1.id(), proxy2.id());
    }

    // Test navigation methods with mock registry
    use super::super::registry::{ObjectRegistry, RegistryEntry};
    use std::collections::HashMap;

    #[derive(Clone)]
    struct TestObject;

    struct TestRegistry {
        objects: HashMap<u64, RegistryEntry<TestObject>>,
        parents: HashMap<u64, ObjectHandle>,
        children: HashMap<u64, Vec<ObjectHandle>>,
        generations: HashMap<u64, u32>,
    }

    impl TestRegistry {
        fn new() -> Self {
            Self {
                objects: HashMap::new(),
                parents: HashMap::new(),
                children: HashMap::new(),
                generations: HashMap::new(),
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
            let id = self.objects.len() as u64 + 1;
            let generation = self.current_generation(id);
            let handle = ObjectHandle::new(ObjectType::Tab, id, generation);
            self.objects.insert(id, RegistryEntry::new(handle, object));
            handle
        }

        fn unregister(&mut self, handle: ObjectHandle) -> Result<TestObject> {
            self.increment_generation(handle.id());
            self.objects
                .remove(&handle.id())
                .map(|entry| entry.into_object())
                .ok_or_else(|| ObjectError::not_found(handle))
        }

        fn get(&self, handle: ObjectHandle) -> Result<&TestObject> {
            let current_gen = self.current_generation(handle.id());
            if !handle.is_valid(current_gen) {
                return Err(ObjectError::stale_handle(handle, current_gen));
            }
            self.objects
                .get(&handle.id())
                .map(|entry| entry.object())
                .ok_or_else(|| ObjectError::not_found(handle))
        }

        fn get_mut(&mut self, handle: ObjectHandle) -> Result<&mut TestObject> {
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
            self.objects.len() as u64 + 1
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

        fn get_parent(&self, handle: &ObjectHandle) -> Result<Option<ObjectHandle>> {
            let current_gen = self.current_generation(handle.id());
            if !handle.is_valid(current_gen) {
                return Err(ObjectError::stale_handle(*handle, current_gen));
            }
            if !self.objects.contains_key(&handle.id()) {
                return Err(ObjectError::not_found(*handle));
            }
            Ok(self.parents.get(&handle.id()).copied())
        }

        fn get_children(&self, handle: &ObjectHandle) -> Result<Vec<ObjectHandle>> {
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
    fn test_tab_navigation_to_window() {
        let mut registry = TestRegistry::new();

        // Create a tab and a window
        let tab_handle = ObjectHandle::new(ObjectType::Tab, 1, 0);
        let window_handle = ObjectHandle::new(ObjectType::Window, 2, 0);

        registry
            .objects
            .insert(1, RegistryEntry::new(tab_handle, TestObject));
        registry
            .objects
            .insert(2, RegistryEntry::new(window_handle, TestObject));

        // Set window as parent of tab
        registry.set_parent(tab_handle, window_handle);

        let tab_proxy = TabProxy::new(tab_handle).unwrap();
        let window_proxy = tab_proxy.window(&registry).unwrap();

        assert_eq!(window_proxy.handle(), window_handle);
        assert_eq!(window_proxy.id(), 2);
    }

    #[test]
    fn test_tab_navigation_to_panes() {
        let mut registry = TestRegistry::new();

        // Create a tab and panes
        let tab_handle = ObjectHandle::new(ObjectType::Tab, 1, 0);
        let pane1_handle = ObjectHandle::new(ObjectType::Pane, 2, 0);
        let pane2_handle = ObjectHandle::new(ObjectType::Pane, 3, 0);

        registry
            .objects
            .insert(1, RegistryEntry::new(tab_handle, TestObject));
        registry
            .objects
            .insert(2, RegistryEntry::new(pane1_handle, TestObject));
        registry
            .objects
            .insert(3, RegistryEntry::new(pane2_handle, TestObject));

        // Set tab as parent of panes
        registry.set_parent(pane1_handle, tab_handle);
        registry.set_parent(pane2_handle, tab_handle);

        let tab_proxy = TabProxy::new(tab_handle).unwrap();
        let panes = tab_proxy.panes(&registry).unwrap();

        assert_eq!(panes.len(), 2);
        assert!(panes.iter().any(|p| p.id() == 2));
        assert!(panes.iter().any(|p| p.id() == 3));
    }

    #[test]
    fn test_tab_navigation_active_pane() {
        let mut registry = TestRegistry::new();

        // Create a tab and panes
        let tab_handle = ObjectHandle::new(ObjectType::Tab, 1, 0);
        let pane1_handle = ObjectHandle::new(ObjectType::Pane, 2, 0);
        let pane2_handle = ObjectHandle::new(ObjectType::Pane, 3, 0);

        registry
            .objects
            .insert(1, RegistryEntry::new(tab_handle, TestObject));
        registry
            .objects
            .insert(2, RegistryEntry::new(pane1_handle, TestObject));
        registry
            .objects
            .insert(3, RegistryEntry::new(pane2_handle, TestObject));

        // Set tab as parent of panes
        registry.set_parent(pane1_handle, tab_handle);
        registry.set_parent(pane2_handle, tab_handle);

        let tab_proxy = TabProxy::new(tab_handle).unwrap();
        let active_pane = tab_proxy.active_pane(&registry).unwrap();

        // Should return the first pane (implementation detail)
        assert!(active_pane.id() == 2 || active_pane.id() == 3);
    }

    #[test]
    fn test_tab_navigation_no_panes() {
        let mut registry = TestRegistry::new();

        let tab_handle = ObjectHandle::new(ObjectType::Tab, 1, 0);
        registry
            .objects
            .insert(1, RegistryEntry::new(tab_handle, TestObject));

        let tab_proxy = TabProxy::new(tab_handle).unwrap();

        // No panes should return empty vector
        let panes = tab_proxy.panes(&registry).unwrap();
        assert!(panes.is_empty());

        // Active pane should fail
        let result = tab_proxy.active_pane(&registry);
        assert!(result.is_err());
    }

    #[test]
    fn test_tab_navigation_no_parent() {
        let mut registry = TestRegistry::new();

        let tab_handle = ObjectHandle::new(ObjectType::Tab, 1, 0);
        registry
            .objects
            .insert(1, RegistryEntry::new(tab_handle, TestObject));

        let tab_proxy = TabProxy::new(tab_handle).unwrap();

        // Tab has no parent
        let result = tab_proxy.window(&registry);
        assert!(result.is_err());
    }

    #[test]
    fn test_tab_navigation_wrong_child_type() {
        let mut registry = TestRegistry::new();

        // Create a tab with a window as child (wrong!)
        let tab_handle = ObjectHandle::new(ObjectType::Tab, 1, 0);
        let window_handle = ObjectHandle::new(ObjectType::Window, 2, 0);

        registry
            .objects
            .insert(1, RegistryEntry::new(tab_handle, TestObject));
        registry
            .objects
            .insert(2, RegistryEntry::new(window_handle, TestObject));

        // Set tab as parent of window (should be pane)
        registry.set_parent(window_handle, tab_handle);

        let tab_proxy = TabProxy::new(tab_handle).unwrap();
        let result = tab_proxy.panes(&registry);

        // Should fail with type mismatch
        assert!(matches!(result, Err(ObjectError::TypeMismatch { .. })));
    }
}
