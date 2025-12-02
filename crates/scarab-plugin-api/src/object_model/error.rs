//! Error types for object model operations

use std::fmt;

use crate::object_model::{ObjectHandle, ObjectType};

/// Errors that can occur when working with the object model
#[derive(Debug, Clone, PartialEq)]
pub enum ObjectError {
    /// The requested object was not found in the registry
    NotFound {
        handle: ObjectHandle,
    },

    /// The handle refers to an object that has been deleted and recreated
    StaleHandle {
        handle: ObjectHandle,
        current_generation: u32,
    },

    /// The object exists but is not of the expected type
    TypeMismatch {
        handle: ObjectHandle,
        expected: ObjectType,
        actual: ObjectType,
    },

    /// The requested method does not exist on this object type
    MethodNotFound {
        handle: ObjectHandle,
        method_name: String,
    },

    /// Invalid argument passed to object method
    InvalidArgument {
        handle: ObjectHandle,
        method_name: String,
        argument: String,
        reason: String,
    },
}

impl ObjectError {
    /// Create a NotFound error
    pub fn not_found(handle: ObjectHandle) -> Self {
        Self::NotFound { handle }
    }

    /// Create a StaleHandle error
    pub fn stale_handle(handle: ObjectHandle, current_generation: u32) -> Self {
        Self::StaleHandle {
            handle,
            current_generation,
        }
    }

    /// Create a TypeMismatch error
    pub fn type_mismatch(handle: ObjectHandle, expected: ObjectType, actual: ObjectType) -> Self {
        Self::TypeMismatch {
            handle,
            expected,
            actual,
        }
    }

    /// Create a MethodNotFound error
    pub fn method_not_found(handle: ObjectHandle, method_name: impl Into<String>) -> Self {
        Self::MethodNotFound {
            handle,
            method_name: method_name.into(),
        }
    }

    /// Create an InvalidArgument error
    pub fn invalid_argument(
        handle: ObjectHandle,
        method_name: impl Into<String>,
        argument: impl Into<String>,
        reason: impl Into<String>,
    ) -> Self {
        Self::InvalidArgument {
            handle,
            method_name: method_name.into(),
            argument: argument.into(),
            reason: reason.into(),
        }
    }
}

impl fmt::Display for ObjectError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ObjectError::NotFound { handle } => {
                write!(f, "Object not found: {}", handle)
            }
            ObjectError::StaleHandle {
                handle,
                current_generation,
            } => {
                write!(
                    f,
                    "Stale handle: {} (generation {} is now {})",
                    handle,
                    handle.generation(),
                    current_generation
                )
            }
            ObjectError::TypeMismatch {
                handle,
                expected,
                actual,
            } => {
                write!(
                    f,
                    "Type mismatch for {}: expected {}, got {}",
                    handle, expected, actual
                )
            }
            ObjectError::MethodNotFound {
                handle,
                method_name,
            } => {
                write!(
                    f,
                    "Method '{}' not found on object {}",
                    method_name, handle
                )
            }
            ObjectError::InvalidArgument {
                handle,
                method_name,
                argument,
                reason,
            } => {
                write!(
                    f,
                    "Invalid argument '{}' for method '{}' on {}: {}",
                    argument, method_name, handle, reason
                )
            }
        }
    }
}

impl std::error::Error for ObjectError {}

/// Result type alias for object model operations
pub type Result<T> = std::result::Result<T, ObjectError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_not_found_error() {
        let handle = ObjectHandle::new(ObjectType::Window, 42, 1);
        let err = ObjectError::not_found(handle);

        match err {
            ObjectError::NotFound { handle: h } => {
                assert_eq!(h.id(), 42);
            }
            _ => panic!("Wrong error variant"),
        }

        let msg = err.to_string();
        assert!(msg.contains("not found"));
        assert!(msg.contains("Window#42"));
    }

    #[test]
    fn test_stale_handle_error() {
        let handle = ObjectHandle::new(ObjectType::Tab, 10, 2);
        let err = ObjectError::stale_handle(handle, 5);

        match err {
            ObjectError::StaleHandle {
                handle: h,
                current_generation,
            } => {
                assert_eq!(h.generation(), 2);
                assert_eq!(current_generation, 5);
            }
            _ => panic!("Wrong error variant"),
        }

        let msg = err.to_string();
        assert!(msg.contains("Stale"));
        assert!(msg.contains("generation 2"));
        assert!(msg.contains("now 5"));
    }

    #[test]
    fn test_type_mismatch_error() {
        let handle = ObjectHandle::new(ObjectType::Pane, 7, 1);
        let err = ObjectError::type_mismatch(handle, ObjectType::Tab, ObjectType::Pane);

        match err {
            ObjectError::TypeMismatch {
                expected, actual, ..
            } => {
                assert_eq!(expected, ObjectType::Tab);
                assert_eq!(actual, ObjectType::Pane);
            }
            _ => panic!("Wrong error variant"),
        }

        let msg = err.to_string();
        assert!(msg.contains("Type mismatch"));
        assert!(msg.contains("Tab"));
        assert!(msg.contains("Pane"));
    }

    #[test]
    fn test_method_not_found_error() {
        let handle = ObjectHandle::new(ObjectType::Window, 1, 1);
        let err = ObjectError::method_not_found(handle, "activate");

        match &err {
            ObjectError::MethodNotFound { method_name, .. } => {
                assert_eq!(method_name, "activate");
            }
            _ => panic!("Wrong error variant"),
        }

        let msg = err.to_string();
        assert!(msg.contains("Method"));
        assert!(msg.contains("activate"));
        assert!(msg.contains("not found"));
    }

    #[test]
    fn test_invalid_argument_error() {
        let handle = ObjectHandle::new(ObjectType::Pane, 5, 1);
        let err = ObjectError::invalid_argument(
            handle,
            "resize",
            "width",
            "must be positive",
        );

        match &err {
            ObjectError::InvalidArgument {
                method_name,
                argument,
                reason,
                ..
            } => {
                assert_eq!(method_name, "resize");
                assert_eq!(argument, "width");
                assert_eq!(reason, "must be positive");
            }
            _ => panic!("Wrong error variant"),
        }

        let msg = err.to_string();
        assert!(msg.contains("Invalid argument"));
        assert!(msg.contains("width"));
        assert!(msg.contains("resize"));
        assert!(msg.contains("must be positive"));
    }

    #[test]
    fn test_error_clone() {
        let handle = ObjectHandle::new(ObjectType::Tab, 1, 1);
        let err1 = ObjectError::not_found(handle);
        let err2 = err1.clone();
        assert_eq!(err1, err2);
    }

    #[test]
    fn test_result_type() {
        let handle = ObjectHandle::new(ObjectType::Window, 1, 1);

        let ok_result: Result<i32> = Ok(42);
        assert_eq!(ok_result.unwrap(), 42);

        let err_result: Result<i32> = Err(ObjectError::not_found(handle));
        assert!(err_result.is_err());
    }
}
