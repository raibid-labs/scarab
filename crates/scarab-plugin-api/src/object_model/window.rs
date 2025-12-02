//! Window object proxy for Fusabi scripts

use super::{ObjectError, ObjectHandle, ObjectRegistry, ObjectType, Result};
use crate::status_bar::{RenderItem, StatusBarSide};

/// Proxy for a terminal window
#[derive(Debug, Clone)]
pub struct WindowProxy {
    handle: ObjectHandle,
    /// Pending status bar updates (stored for later IPC transmission)
    pending_status: std::sync::Arc<std::sync::Mutex<Vec<(StatusBarSide, Vec<RenderItem>)>>>,
}

impl WindowProxy {
    pub fn new(handle: ObjectHandle) -> Result<Self> {
        if handle.object_type() != ObjectType::Window {
            return Err(ObjectError::type_mismatch(
                handle,
                ObjectType::Window,
                handle.object_type(),
            ));
        }
        Ok(Self {
            handle,
            pending_status: std::sync::Arc::new(std::sync::Mutex::new(Vec::new())),
        })
    }

    pub fn handle(&self) -> ObjectHandle {
        self.handle
    }

    pub fn id(&self) -> u64 {
        self.handle.id()
    }

    // Methods that will dispatch to actual implementation
    // For now, return placeholder errors

    /// Get the active pane in this window
    ///
    /// This returns the active pane from the active tab.
    ///
    /// # Arguments
    ///
    /// * `registry` - The object registry to use for navigation
    ///
    /// # Returns
    ///
    /// * `Ok(PaneProxy)` - The active pane in the active tab
    /// * `Err(ObjectError)` - If navigation fails or no active tab/pane exists
    pub fn active_pane<T>(&self, registry: &impl ObjectRegistry<T>) -> Result<PaneProxy> {
        let tab = self.active_tab(registry)?;
        tab.active_pane(registry)
    }

    /// Get the active tab in this window
    ///
    /// This method returns the first tab in the window.
    /// In a full implementation, this would query which tab is actually active.
    ///
    /// # Arguments
    ///
    /// * `registry` - The object registry to use for navigation
    ///
    /// # Returns
    ///
    /// * `Ok(TabProxy)` - The active tab in this window
    /// * `Err(ObjectError)` - If the window has no tabs or navigation fails
    pub fn active_tab<T>(&self, registry: &impl ObjectRegistry<T>) -> Result<TabProxy> {
        let tabs = self.tabs(registry)?;
        tabs.into_iter()
            .next()
            .ok_or_else(|| ObjectError::method_not_found(self.handle, "active_tab: no tabs"))
    }

    /// Get all tabs in this window
    ///
    /// # Arguments
    ///
    /// * `registry` - The object registry to use for navigation
    ///
    /// # Returns
    ///
    /// * `Ok(Vec<TabProxy>)` - All tabs in this window (may be empty)
    /// * `Err(ObjectError)` - If navigation fails
    pub fn tabs<T>(&self, registry: &impl ObjectRegistry<T>) -> Result<Vec<TabProxy>> {
        let child_handles = registry.get_children(&self.handle)?;

        child_handles
            .into_iter()
            .map(|handle| {
                if handle.object_type() != ObjectType::Tab {
                    return Err(ObjectError::type_mismatch(
                        handle,
                        ObjectType::Tab,
                        handle.object_type(),
                    ));
                }
                TabProxy::new(handle)
            })
            .collect()
    }

    /// Get window dimensions
    pub fn get_dimensions(&self) -> Result<(u32, u32)> {
        Err(ObjectError::method_not_found(
            self.handle,
            "get_dimensions",
        ))
    }

    /// Check if window is focused
    pub fn is_focused(&self) -> Result<bool> {
        Err(ObjectError::method_not_found(self.handle, "is_focused"))
    }

    /// Set right status bar content
    ///
    /// Stores the render items for later IPC transmission to the client.
    /// The items will be sent to the UI when the next status update is triggered.
    ///
    /// # Arguments
    ///
    /// * `items` - Vector of RenderItem elements to display on the right side
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Items stored successfully
    /// * `Err(ObjectError)` - If storage fails
    pub fn set_right_status(&self, items: Vec<RenderItem>) -> Result<()> {
        let mut pending = self.pending_status.lock().map_err(|_| {
            ObjectError::method_not_found(self.handle, "set_right_status: lock poisoned")
        })?;
        pending.push((StatusBarSide::Right, items));
        Ok(())
    }

    /// Set left status bar content
    ///
    /// Stores the render items for later IPC transmission to the client.
    /// The items will be sent to the UI when the next status update is triggered.
    ///
    /// # Arguments
    ///
    /// * `items` - Vector of RenderItem elements to display on the left side
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Items stored successfully
    /// * `Err(ObjectError)` - If storage fails
    pub fn set_left_status(&self, items: Vec<RenderItem>) -> Result<()> {
        let mut pending = self.pending_status.lock().map_err(|_| {
            ObjectError::method_not_found(self.handle, "set_left_status: lock poisoned")
        })?;
        pending.push((StatusBarSide::Left, items));
        Ok(())
    }

    /// Clear all status bar content
    ///
    /// Clears both left and right status bar sections.
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Status cleared successfully
    /// * `Err(ObjectError)` - If clearing fails
    pub fn clear_status(&self) -> Result<()> {
        let mut pending = self.pending_status.lock().map_err(|_| {
            ObjectError::method_not_found(self.handle, "clear_status: lock poisoned")
        })?;
        pending.push((StatusBarSide::Left, vec![]));
        pending.push((StatusBarSide::Right, vec![]));
        Ok(())
    }

    /// Drain pending status updates for IPC transmission
    ///
    /// Returns and clears all pending status updates that need to be sent to the client.
    /// This is called internally by the daemon's IPC layer.
    ///
    /// # Returns
    ///
    /// Vector of (side, items) tuples representing pending updates
    pub fn drain_pending_status(&self) -> Vec<(StatusBarSide, Vec<RenderItem>)> {
        self.pending_status
            .lock()
            .map(|mut pending| pending.drain(..).collect())
            .unwrap_or_default()
    }

    /// Show a toast notification
    pub fn toast_notification(&self, _title: &str, _message: &str) -> Result<()> {
        Err(ObjectError::method_not_found(
            self.handle,
            "toast_notification",
        ))
    }
}

// Forward declarations for cross-references
// These will be defined in their respective modules
use super::PaneProxy;
use super::TabProxy;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_window_proxy_creation() {
        let handle = ObjectHandle::new(ObjectType::Window, 1, 0);
        let proxy = WindowProxy::new(handle);
        assert!(proxy.is_ok());

        let proxy = proxy.unwrap();
        assert_eq!(proxy.id(), 1);
        assert_eq!(proxy.handle(), handle);
    }

    #[test]
    fn test_window_proxy_type_validation() {
        let pane_handle = ObjectHandle::new(ObjectType::Pane, 1, 0);
        let result = WindowProxy::new(pane_handle);

        assert!(result.is_err());
        match result {
            Err(ObjectError::TypeMismatch {
                expected, actual, ..
            }) => {
                assert_eq!(expected, ObjectType::Window);
                assert_eq!(actual, ObjectType::Pane);
            }
            _ => panic!("Expected TypeMismatch error"),
        }
    }

    #[test]
    fn test_window_proxy_methods_not_implemented() {
        let handle = ObjectHandle::new(ObjectType::Window, 1, 0);
        let proxy = WindowProxy::new(handle).unwrap();

        // Some methods should return MethodNotFound errors
        assert!(matches!(
            proxy.get_dimensions(),
            Err(ObjectError::MethodNotFound { .. })
        ));
        assert!(matches!(
            proxy.is_focused(),
            Err(ObjectError::MethodNotFound { .. })
        ));
        assert!(matches!(
            proxy.toast_notification("title", "message"),
            Err(ObjectError::MethodNotFound { .. })
        ));

        // Status methods should now work
        assert!(proxy.set_right_status(vec![]).is_ok());
        assert!(proxy.set_left_status(vec![]).is_ok());
        assert!(proxy.clear_status().is_ok());
    }

    #[test]
    fn test_window_proxy_clone() {
        let handle = ObjectHandle::new(ObjectType::Window, 1, 0);
        let proxy1 = WindowProxy::new(handle).unwrap();
        let proxy2 = proxy1.clone();

        assert_eq!(proxy1.handle(), proxy2.handle());
        assert_eq!(proxy1.id(), proxy2.id());
    }

    #[test]
    fn test_status_bar_methods() {
        let handle = ObjectHandle::new(ObjectType::Window, 1, 0);
        let proxy = WindowProxy::new(handle).unwrap();

        // Test setting left status
        let left_items = vec![RenderItem::Text("Left".to_string())];
        assert!(proxy.set_left_status(left_items).is_ok());

        // Test setting right status
        let right_items = vec![RenderItem::Text("Right".to_string())];
        assert!(proxy.set_right_status(right_items).is_ok());

        // Test draining pending status
        let pending = proxy.drain_pending_status();
        assert_eq!(pending.len(), 2);

        // After draining, should be empty
        let pending2 = proxy.drain_pending_status();
        assert_eq!(pending2.len(), 0);
    }

    #[test]
    fn test_status_bar_clear() {
        let handle = ObjectHandle::new(ObjectType::Window, 1, 0);
        let proxy = WindowProxy::new(handle).unwrap();

        // Clear status should create two pending updates (left and right)
        assert!(proxy.clear_status().is_ok());

        let pending = proxy.drain_pending_status();
        assert_eq!(pending.len(), 2);

        // Both should have empty item vectors
        for (_, items) in pending {
            assert!(items.is_empty());
        }
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
            let handle = ObjectHandle::new(ObjectType::Window, id, generation);
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
    fn test_window_navigation_to_tabs() {
        let mut registry = TestRegistry::new();

        // Create a window and tabs
        let window_handle = ObjectHandle::new(ObjectType::Window, 1, 0);
        let tab1_handle = ObjectHandle::new(ObjectType::Tab, 2, 0);
        let tab2_handle = ObjectHandle::new(ObjectType::Tab, 3, 0);

        registry.objects.insert(1, RegistryEntry::new(window_handle, TestObject));
        registry.objects.insert(2, RegistryEntry::new(tab1_handle, TestObject));
        registry.objects.insert(3, RegistryEntry::new(tab2_handle, TestObject));

        // Set window as parent of tabs
        registry.set_parent(tab1_handle, window_handle);
        registry.set_parent(tab2_handle, window_handle);

        let window_proxy = WindowProxy::new(window_handle).unwrap();
        let tabs = window_proxy.tabs(&registry).unwrap();

        assert_eq!(tabs.len(), 2);
        assert!(tabs.iter().any(|t| t.id() == 2));
        assert!(tabs.iter().any(|t| t.id() == 3));
    }

    #[test]
    fn test_window_navigation_active_tab() {
        let mut registry = TestRegistry::new();

        // Create a window and tabs
        let window_handle = ObjectHandle::new(ObjectType::Window, 1, 0);
        let tab1_handle = ObjectHandle::new(ObjectType::Tab, 2, 0);
        let tab2_handle = ObjectHandle::new(ObjectType::Tab, 3, 0);

        registry.objects.insert(1, RegistryEntry::new(window_handle, TestObject));
        registry.objects.insert(2, RegistryEntry::new(tab1_handle, TestObject));
        registry.objects.insert(3, RegistryEntry::new(tab2_handle, TestObject));

        // Set window as parent of tabs
        registry.set_parent(tab1_handle, window_handle);
        registry.set_parent(tab2_handle, window_handle);

        let window_proxy = WindowProxy::new(window_handle).unwrap();
        let active_tab = window_proxy.active_tab(&registry).unwrap();

        // Should return the first tab
        assert!(active_tab.id() == 2 || active_tab.id() == 3);
    }

    #[test]
    fn test_window_navigation_active_pane() {
        let mut registry = TestRegistry::new();

        // Create a window, tab, and pane hierarchy
        let window_handle = ObjectHandle::new(ObjectType::Window, 1, 0);
        let tab_handle = ObjectHandle::new(ObjectType::Tab, 2, 0);
        let pane_handle = ObjectHandle::new(ObjectType::Pane, 3, 0);

        registry.objects.insert(1, RegistryEntry::new(window_handle, TestObject));
        registry.objects.insert(2, RegistryEntry::new(tab_handle, TestObject));
        registry.objects.insert(3, RegistryEntry::new(pane_handle, TestObject));

        // Set hierarchy: window -> tab -> pane
        registry.set_parent(tab_handle, window_handle);
        registry.set_parent(pane_handle, tab_handle);

        let window_proxy = WindowProxy::new(window_handle).unwrap();
        let active_pane = window_proxy.active_pane(&registry).unwrap();

        assert_eq!(active_pane.id(), 3);
    }

    #[test]
    fn test_window_navigation_no_tabs() {
        let mut registry = TestRegistry::new();

        let window_handle = ObjectHandle::new(ObjectType::Window, 1, 0);
        registry.objects.insert(1, RegistryEntry::new(window_handle, TestObject));

        let window_proxy = WindowProxy::new(window_handle).unwrap();

        // No tabs should return empty vector
        let tabs = window_proxy.tabs(&registry).unwrap();
        assert!(tabs.is_empty());

        // Active tab should fail
        let result = window_proxy.active_tab(&registry);
        assert!(result.is_err());

        // Active pane should also fail
        let result = window_proxy.active_pane(&registry);
        assert!(result.is_err());
    }

    #[test]
    fn test_window_navigation_wrong_child_type() {
        let mut registry = TestRegistry::new();

        // Create a window with a pane as direct child (wrong!)
        let window_handle = ObjectHandle::new(ObjectType::Window, 1, 0);
        let pane_handle = ObjectHandle::new(ObjectType::Pane, 2, 0);

        registry.objects.insert(1, RegistryEntry::new(window_handle, TestObject));
        registry.objects.insert(2, RegistryEntry::new(pane_handle, TestObject));

        // Set window as parent of pane (should be tab)
        registry.set_parent(pane_handle, window_handle);

        let window_proxy = WindowProxy::new(window_handle).unwrap();
        let result = window_proxy.tabs(&registry);

        // Should fail with type mismatch
        assert!(matches!(result, Err(ObjectError::TypeMismatch { .. })));
    }

    #[test]
    fn test_window_navigation_complete_hierarchy() {
        let mut registry = TestRegistry::new();

        // Create complete hierarchy: window -> tab -> pane
        let window_handle = ObjectHandle::new(ObjectType::Window, 1, 0);
        let tab1_handle = ObjectHandle::new(ObjectType::Tab, 2, 0);
        let tab2_handle = ObjectHandle::new(ObjectType::Tab, 3, 0);
        let pane1_handle = ObjectHandle::new(ObjectType::Pane, 4, 0);
        let pane2_handle = ObjectHandle::new(ObjectType::Pane, 5, 0);

        registry.objects.insert(1, RegistryEntry::new(window_handle, TestObject));
        registry.objects.insert(2, RegistryEntry::new(tab1_handle, TestObject));
        registry.objects.insert(3, RegistryEntry::new(tab2_handle, TestObject));
        registry.objects.insert(4, RegistryEntry::new(pane1_handle, TestObject));
        registry.objects.insert(5, RegistryEntry::new(pane2_handle, TestObject));

        // Set hierarchy
        registry.set_parent(tab1_handle, window_handle);
        registry.set_parent(tab2_handle, window_handle);
        registry.set_parent(pane1_handle, tab1_handle);
        registry.set_parent(pane2_handle, tab2_handle);

        let window_proxy = WindowProxy::new(window_handle).unwrap();

        // Test tabs navigation
        let tabs = window_proxy.tabs(&registry).unwrap();
        assert_eq!(tabs.len(), 2);

        // Test active tab navigation
        let active_tab = window_proxy.active_tab(&registry).unwrap();
        assert!(active_tab.id() == 2 || active_tab.id() == 3);

        // Test active pane navigation (through active tab)
        let active_pane = window_proxy.active_pane(&registry).unwrap();
        assert!(active_pane.id() == 4 || active_pane.id() == 5);
    }
}
