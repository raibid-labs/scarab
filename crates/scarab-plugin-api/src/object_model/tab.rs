//! Tab object proxy for Fusabi scripts

use super::{ObjectError, ObjectHandle, ObjectType, PaneProxy, Result, WindowProxy};

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
    pub fn panes(&self) -> Result<Vec<PaneProxy>> {
        Err(ObjectError::method_not_found(self.handle, "panes"))
    }

    /// Get active pane
    pub fn active_pane(&self) -> Result<PaneProxy> {
        Err(ObjectError::method_not_found(self.handle, "active_pane"))
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
    pub fn window(&self) -> Result<WindowProxy> {
        Err(ObjectError::method_not_found(self.handle, "window"))
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
            proxy.panes(),
            Err(ObjectError::MethodNotFound { .. })
        ));
        assert!(matches!(
            proxy.active_pane(),
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
        assert!(matches!(
            proxy.window(),
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
}
