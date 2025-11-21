//! FFI Bridge for Rust function calls
//!
//! Allows bytecode to call native Rust functions safely

use crate::bytecode::Value;
use std::collections::HashMap;

/// FFI function signature
pub type FfiFunction = fn(&[Value]) -> Result<Value, FfiError>;

/// FFI function registry
pub struct FfiRegistry {
    functions: HashMap<String, FfiFunction>,
}

impl FfiRegistry {
    /// Create a new registry
    pub fn new() -> Self {
        Self {
            functions: HashMap::new(),
        }
    }

    /// Register an FFI function
    pub fn register(&mut self, name: &str, func: FfiFunction) {
        self.functions.insert(name.to_string(), func);
    }

    /// Get an FFI function by name
    pub fn get(&self, name: &str) -> Option<FfiFunction> {
        self.functions.get(name).copied()
    }

    /// Check if a function is registered
    pub fn contains(&self, name: &str) -> bool {
        self.functions.contains_key(name)
    }

    /// List all registered functions
    pub fn list(&self) -> Vec<String> {
        self.functions.keys().cloned().collect()
    }
}

impl Default for FfiRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// FFI Errors
#[derive(Debug, thiserror::Error)]
pub enum FfiError {
    #[error("Invalid argument count: expected {expected}, got {got}")]
    InvalidArgCount { expected: usize, got: usize },

    #[error("Invalid argument type at index {index}: expected {expected}")]
    InvalidArgType { index: usize, expected: String },

    #[error("Function execution failed: {0}")]
    ExecutionError(String),

    #[error("Permission denied: {0}")]
    PermissionDenied(String),
}

// Standard library FFI functions

/// Print to stdout
pub fn ffi_print(args: &[Value]) -> Result<Value, FfiError> {
    if args.len() != 1 {
        return Err(FfiError::InvalidArgCount {
            expected: 1,
            got: args.len(),
        });
    }

    println!("{}", args[0]);
    Ok(Value::Unit)
}

/// String concatenation
pub fn ffi_string_concat(args: &[Value]) -> Result<Value, FfiError> {
    if args.len() != 2 {
        return Err(FfiError::InvalidArgCount {
            expected: 2,
            got: args.len(),
        });
    }

    let s1 = match &args[0] {
        Value::String(s) => s,
        _ => {
            return Err(FfiError::InvalidArgType {
                index: 0,
                expected: "String".to_string(),
            })
        }
    };

    let s2 = match &args[1] {
        Value::String(s) => s,
        _ => {
            return Err(FfiError::InvalidArgType {
                index: 1,
                expected: "String".to_string(),
            })
        }
    };

    Ok(Value::String(format!("{}{}", s1, s2)))
}

/// String contains substring
pub fn ffi_string_contains(args: &[Value]) -> Result<Value, FfiError> {
    if args.len() != 2 {
        return Err(FfiError::InvalidArgCount {
            expected: 2,
            got: args.len(),
        });
    }

    let haystack = match &args[0] {
        Value::String(s) => s,
        _ => {
            return Err(FfiError::InvalidArgType {
                index: 0,
                expected: "String".to_string(),
            })
        }
    };

    let needle = match &args[1] {
        Value::String(s) => s,
        _ => {
            return Err(FfiError::InvalidArgType {
                index: 1,
                expected: "String".to_string(),
            })
        }
    };

    Ok(Value::Bool(haystack.contains(needle.as_str())))
}

/// String length
pub fn ffi_string_len(args: &[Value]) -> Result<Value, FfiError> {
    if args.len() != 1 {
        return Err(FfiError::InvalidArgCount {
            expected: 1,
            got: args.len(),
        });
    }

    let s = match &args[0] {
        Value::String(s) => s,
        _ => {
            return Err(FfiError::InvalidArgType {
                index: 0,
                expected: "String".to_string(),
            })
        }
    };

    Ok(Value::I32(s.len() as i32))
}

/// Integer to string
pub fn ffi_i32_to_string(args: &[Value]) -> Result<Value, FfiError> {
    if args.len() != 1 {
        return Err(FfiError::InvalidArgCount {
            expected: 1,
            got: args.len(),
        });
    }

    let i = match &args[0] {
        Value::I32(i) => i,
        _ => {
            return Err(FfiError::InvalidArgType {
                index: 0,
                expected: "I32".to_string(),
            })
        }
    };

    Ok(Value::String(i.to_string()))
}

/// Math: absolute value
pub fn ffi_abs(args: &[Value]) -> Result<Value, FfiError> {
    if args.len() != 1 {
        return Err(FfiError::InvalidArgCount {
            expected: 1,
            got: args.len(),
        });
    }

    match &args[0] {
        Value::I32(i) => Ok(Value::I32(i.abs())),
        Value::I64(i) => Ok(Value::I64(i.abs())),
        Value::F32(f) => Ok(Value::F32(f.abs())),
        Value::F64(f) => Ok(Value::F64(f.abs())),
        _ => Err(FfiError::InvalidArgType {
            index: 0,
            expected: "Number".to_string(),
        }),
    }
}

/// Math: square root
pub fn ffi_sqrt(args: &[Value]) -> Result<Value, FfiError> {
    if args.len() != 1 {
        return Err(FfiError::InvalidArgCount {
            expected: 1,
            got: args.len(),
        });
    }

    match &args[0] {
        Value::F32(f) => Ok(Value::F32(f.sqrt())),
        Value::F64(f) => Ok(Value::F64(f.sqrt())),
        _ => Err(FfiError::InvalidArgType {
            index: 0,
            expected: "Float".to_string(),
        }),
    }
}

/// Create a standard library registry
pub fn create_stdlib() -> FfiRegistry {
    let mut registry = FfiRegistry::new();

    // I/O functions
    registry.register("print", ffi_print);

    // String functions
    registry.register("string_concat", ffi_string_concat);
    registry.register("string_contains", ffi_string_contains);
    registry.register("string_len", ffi_string_len);

    // Conversion functions
    registry.register("i32_to_string", ffi_i32_to_string);

    // Math functions
    registry.register("abs", ffi_abs);
    registry.register("sqrt", ffi_sqrt);

    registry
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ffi_print() {
        let result = ffi_print(&[Value::String("Hello".to_string())]);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::Unit);
    }

    #[test]
    fn test_string_concat() {
        let result = ffi_string_concat(&[
            Value::String("Hello, ".to_string()),
            Value::String("World!".to_string()),
        ]);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::String("Hello, World!".to_string()));
    }

    #[test]
    fn test_string_contains() {
        let result = ffi_string_contains(&[
            Value::String("Hello World".to_string()),
            Value::String("World".to_string()),
        ]);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::Bool(true));
    }

    #[test]
    fn test_string_len() {
        let result = ffi_string_len(&[Value::String("Hello".to_string())]);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::I32(5));
    }

    #[test]
    fn test_abs() {
        let result = ffi_abs(&[Value::I32(-42)]);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::I32(42));
    }

    #[test]
    fn test_invalid_arg_count() {
        let result = ffi_print(&[]);
        assert!(result.is_err());
        match result.unwrap_err() {
            FfiError::InvalidArgCount { expected, got } => {
                assert_eq!(expected, 1);
                assert_eq!(got, 0);
            }
            _ => panic!("Expected InvalidArgCount error"),
        }
    }

    #[test]
    fn test_registry() {
        let mut registry = FfiRegistry::new();
        registry.register("test", ffi_print);

        assert!(registry.contains("test"));
        assert!(!registry.contains("nonexistent"));

        let func = registry.get("test");
        assert!(func.is_some());
    }

    #[test]
    fn test_stdlib_creation() {
        let stdlib = create_stdlib();
        assert!(stdlib.contains("print"));
        assert!(stdlib.contains("string_concat"));
        assert!(stdlib.contains("abs"));
    }
}
