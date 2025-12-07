//! Pane object proxy for Fusabi scripts

use super::{ObjectError, ObjectHandle, ObjectRegistry, ObjectType, Result};

/// Proxy for a terminal pane
#[derive(Debug, Clone)]
pub struct PaneProxy {
    handle: ObjectHandle,
}

impl PaneProxy {
    pub fn new(handle: ObjectHandle) -> Result<Self> {
        if handle.object_type() != ObjectType::Pane {
            return Err(ObjectError::type_mismatch(
                handle,
                ObjectType::Pane,
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

    /// Send text to the pane's PTY
    pub fn send_text(&self, _text: &str) -> Result<()> {
        Err(ObjectError::method_not_found(self.handle, "send_text"))
    }

    /// Send text as paste (bracketed paste mode)
    pub fn send_paste(&self, _text: &str) -> Result<()> {
        Err(ObjectError::method_not_found(self.handle, "send_paste"))
    }

    /// Get the pane title
    pub fn get_title(&self) -> Result<String> {
        Err(ObjectError::method_not_found(self.handle, "get_title"))
    }

    /// Get current working directory
    pub fn get_current_working_dir(&self) -> Result<Option<String>> {
        Err(ObjectError::method_not_found(
            self.handle,
            "get_current_working_dir",
        ))
    }

    /// Get cursor position
    pub fn get_cursor_position(&self) -> Result<(u16, u16)> {
        Err(ObjectError::method_not_found(
            self.handle,
            "get_cursor_position",
        ))
    }

    /// Get pane dimensions in cells
    pub fn get_dimensions(&self) -> Result<(u16, u16)> {
        Err(ObjectError::method_not_found(self.handle, "get_dimensions"))
    }

    /// Get foreground process name
    pub fn get_foreground_process_name(&self) -> Result<String> {
        Err(ObjectError::method_not_found(
            self.handle,
            "get_foreground_process_name",
        ))
    }

    /// Get text from scrollback
    pub fn get_lines_as_text(&self, _start: i32, _end: i32) -> Result<String> {
        Err(ObjectError::method_not_found(
            self.handle,
            "get_lines_as_text",
        ))
    }

    /// Check if alternate screen buffer is active
    pub fn is_alt_screen_active(&self) -> Result<bool> {
        Err(ObjectError::method_not_found(
            self.handle,
            "is_alt_screen_active",
        ))
    }

    /// Check for unseen output
    pub fn has_unseen_output(&self) -> Result<bool> {
        Err(ObjectError::method_not_found(
            self.handle,
            "has_unseen_output",
        ))
    }

    /// Focus this pane
    pub fn activate(&self) -> Result<()> {
        Err(ObjectError::method_not_found(self.handle, "activate"))
    }

    /// Get parent tab
    ///
    /// # Arguments
    ///
    /// * `registry` - The object registry to use for navigation
    ///
    /// # Returns
    ///
    /// * `Ok(TabProxy)` - The tab containing this pane
    /// * `Err(ObjectError)` - If the pane has no parent or navigation fails
    pub fn tab<T>(&self, registry: &impl ObjectRegistry<T>) -> Result<TabProxy> {
        let parent_handle = registry
            .get_parent(&self.handle)?
            .ok_or_else(|| ObjectError::method_not_found(self.handle, "tab: no parent"))?;

        if parent_handle.object_type() != ObjectType::Tab {
            return Err(ObjectError::type_mismatch(
                parent_handle,
                ObjectType::Tab,
                parent_handle.object_type(),
            ));
        }

        TabProxy::new(parent_handle)
    }

    /// Get parent window
    ///
    /// # Arguments
    ///
    /// * `registry` - The object registry to use for navigation
    ///
    /// # Returns
    ///
    /// * `Ok(WindowProxy)` - The window containing this pane
    /// * `Err(ObjectError)` - If navigation fails
    pub fn window<T>(&self, registry: &impl ObjectRegistry<T>) -> Result<WindowProxy> {
        // Navigate up through tab to window
        let tab = self.tab(registry)?;
        tab.window(registry)
    }
}

// Forward declarations for cross-references
use super::TabProxy;
use super::WindowProxy;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pane_proxy_creation() {
        let handle = ObjectHandle::new(ObjectType::Pane, 1, 0);
        let proxy = PaneProxy::new(handle);
        assert!(proxy.is_ok());

        let proxy = proxy.unwrap();
        assert_eq!(proxy.id(), 1);
        assert_eq!(proxy.handle(), handle);
    }

    #[test]
    fn test_pane_proxy_type_validation() {
        let window_handle = ObjectHandle::new(ObjectType::Window, 1, 0);
        let result = PaneProxy::new(window_handle);

        assert!(result.is_err());
        match result {
            Err(ObjectError::TypeMismatch {
                expected, actual, ..
            }) => {
                assert_eq!(expected, ObjectType::Pane);
                assert_eq!(actual, ObjectType::Window);
            }
            _ => panic!("Expected TypeMismatch error"),
        }
    }

    #[test]
    fn test_pane_proxy_methods_not_implemented() {
        let handle = ObjectHandle::new(ObjectType::Pane, 1, 0);
        let proxy = PaneProxy::new(handle).unwrap();

        // All methods should return MethodNotFound errors
        assert!(matches!(
            proxy.send_text("test"),
            Err(ObjectError::MethodNotFound { .. })
        ));
        assert!(matches!(
            proxy.send_paste("test"),
            Err(ObjectError::MethodNotFound { .. })
        ));
        assert!(matches!(
            proxy.get_title(),
            Err(ObjectError::MethodNotFound { .. })
        ));
        assert!(matches!(
            proxy.get_current_working_dir(),
            Err(ObjectError::MethodNotFound { .. })
        ));
        assert!(matches!(
            proxy.get_cursor_position(),
            Err(ObjectError::MethodNotFound { .. })
        ));
        assert!(matches!(
            proxy.get_dimensions(),
            Err(ObjectError::MethodNotFound { .. })
        ));
        assert!(matches!(
            proxy.get_foreground_process_name(),
            Err(ObjectError::MethodNotFound { .. })
        ));
        assert!(matches!(
            proxy.get_lines_as_text(0, 10),
            Err(ObjectError::MethodNotFound { .. })
        ));
        assert!(matches!(
            proxy.is_alt_screen_active(),
            Err(ObjectError::MethodNotFound { .. })
        ));
        assert!(matches!(
            proxy.has_unseen_output(),
            Err(ObjectError::MethodNotFound { .. })
        ));
        assert!(matches!(
            proxy.activate(),
            Err(ObjectError::MethodNotFound { .. })
        ));
    }

    #[test]
    fn test_pane_proxy_clone() {
        let handle = ObjectHandle::new(ObjectType::Pane, 1, 0);
        let proxy1 = PaneProxy::new(handle).unwrap();
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
            let handle = ObjectHandle::new(ObjectType::Pane, id, generation);
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
    fn test_pane_navigation_to_tab() {
        let mut registry = TestRegistry::new();

        // Create a pane and a tab
        let pane_handle = ObjectHandle::new(ObjectType::Pane, 1, 0);
        let tab_handle = ObjectHandle::new(ObjectType::Tab, 2, 0);

        registry
            .objects
            .insert(1, RegistryEntry::new(pane_handle, TestObject));
        registry
            .objects
            .insert(2, RegistryEntry::new(tab_handle, TestObject));

        // Set tab as parent of pane
        registry.set_parent(pane_handle, tab_handle);

        let pane_proxy = PaneProxy::new(pane_handle).unwrap();
        let tab_proxy = pane_proxy.tab(&registry).unwrap();

        assert_eq!(tab_proxy.handle(), tab_handle);
        assert_eq!(tab_proxy.id(), 2);
    }

    #[test]
    fn test_pane_navigation_to_window() {
        let mut registry = TestRegistry::new();

        // Create a pane, tab, and window
        let pane_handle = ObjectHandle::new(ObjectType::Pane, 1, 0);
        let tab_handle = ObjectHandle::new(ObjectType::Tab, 2, 0);
        let window_handle = ObjectHandle::new(ObjectType::Window, 3, 0);

        registry
            .objects
            .insert(1, RegistryEntry::new(pane_handle, TestObject));
        registry
            .objects
            .insert(2, RegistryEntry::new(tab_handle, TestObject));
        registry
            .objects
            .insert(3, RegistryEntry::new(window_handle, TestObject));

        // Set hierarchy: pane -> tab -> window
        registry.set_parent(pane_handle, tab_handle);
        registry.set_parent(tab_handle, window_handle);

        let pane_proxy = PaneProxy::new(pane_handle).unwrap();
        let window_proxy = pane_proxy.window(&registry).unwrap();

        assert_eq!(window_proxy.handle(), window_handle);
        assert_eq!(window_proxy.id(), 3);
    }

    #[test]
    fn test_pane_navigation_no_parent() {
        let registry = TestRegistry::new();

        let pane_handle = ObjectHandle::new(ObjectType::Pane, 1, 0);
        let pane_proxy = PaneProxy::new(pane_handle).unwrap();

        // Pane has no parent
        let result = pane_proxy.tab(&registry);
        assert!(result.is_err());
    }

    #[test]
    fn test_pane_navigation_wrong_parent_type() {
        let mut registry = TestRegistry::new();

        // Create a pane with a window as direct parent (wrong!)
        let pane_handle = ObjectHandle::new(ObjectType::Pane, 1, 0);
        let window_handle = ObjectHandle::new(ObjectType::Window, 2, 0);

        registry
            .objects
            .insert(1, RegistryEntry::new(pane_handle, TestObject));
        registry
            .objects
            .insert(2, RegistryEntry::new(window_handle, TestObject));

        // Set window as parent of pane (should be tab)
        registry.set_parent(pane_handle, window_handle);

        let pane_proxy = PaneProxy::new(pane_handle).unwrap();
        let result = pane_proxy.tab(&registry);

        // Should fail with type mismatch
        assert!(matches!(result, Err(ObjectError::TypeMismatch { .. })));
    }
}
