//! Pane object proxy for Fusabi scripts

use super::{ObjectError, ObjectHandle, ObjectType, Result};

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
        Err(ObjectError::method_not_found(
            self.handle,
            "get_dimensions",
        ))
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
    pub fn tab(&self) -> Result<TabProxy> {
        Err(ObjectError::method_not_found(self.handle, "tab"))
    }

    /// Get parent window
    pub fn window(&self) -> Result<WindowProxy> {
        Err(ObjectError::method_not_found(self.handle, "window"))
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
        assert!(matches!(
            proxy.tab(),
            Err(ObjectError::MethodNotFound { .. })
        ));
        assert!(matches!(
            proxy.window(),
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
}
