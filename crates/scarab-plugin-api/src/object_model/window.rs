//! Window object proxy for Fusabi scripts

use super::{ObjectError, ObjectHandle, ObjectType, Result};

/// Proxy for a terminal window
#[derive(Debug, Clone)]
pub struct WindowProxy {
    handle: ObjectHandle,
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
        Ok(Self { handle })
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
    pub fn active_pane(&self) -> Result<PaneProxy> {
        // TODO: Implement via IPC query
        Err(ObjectError::method_not_found(
            self.handle,
            "active_pane",
        ))
    }

    /// Get the active tab in this window
    pub fn active_tab(&self) -> Result<TabProxy> {
        Err(ObjectError::method_not_found(self.handle, "active_tab"))
    }

    /// Get all tabs in this window
    pub fn tabs(&self) -> Result<Vec<TabProxy>> {
        Err(ObjectError::method_not_found(self.handle, "tabs"))
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
    pub fn set_right_status(&self, _items: Vec<RenderItem>) -> Result<()> {
        Err(ObjectError::method_not_found(
            self.handle,
            "set_right_status",
        ))
    }

    /// Set left status bar content
    pub fn set_left_status(&self, _items: Vec<RenderItem>) -> Result<()> {
        Err(ObjectError::method_not_found(
            self.handle,
            "set_left_status",
        ))
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

// Placeholder for RenderItem (will be in status_bar module later)
#[derive(Debug, Clone)]
pub enum RenderItem {
    Text(String),
    // More variants later
}

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

        // All methods should return MethodNotFound errors
        assert!(matches!(
            proxy.active_pane(),
            Err(ObjectError::MethodNotFound { .. })
        ));
        assert!(matches!(
            proxy.active_tab(),
            Err(ObjectError::MethodNotFound { .. })
        ));
        assert!(matches!(
            proxy.tabs(),
            Err(ObjectError::MethodNotFound { .. })
        ));
        assert!(matches!(
            proxy.get_dimensions(),
            Err(ObjectError::MethodNotFound { .. })
        ));
        assert!(matches!(
            proxy.is_focused(),
            Err(ObjectError::MethodNotFound { .. })
        ));
        assert!(matches!(
            proxy.set_right_status(vec![]),
            Err(ObjectError::MethodNotFound { .. })
        ));
        assert!(matches!(
            proxy.set_left_status(vec![]),
            Err(ObjectError::MethodNotFound { .. })
        ));
        assert!(matches!(
            proxy.toast_notification("title", "message"),
            Err(ObjectError::MethodNotFound { .. })
        ));
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
    fn test_render_item_creation() {
        let item = RenderItem::Text("test".to_string());
        match item {
            RenderItem::Text(s) => assert_eq!(s, "test"),
        }
    }
}
