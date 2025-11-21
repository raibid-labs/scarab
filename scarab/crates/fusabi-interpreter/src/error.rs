use std::fmt;
use thiserror::Error;

/// Errors that can occur during parsing, interpretation, or runtime
#[derive(Debug, Error)]
pub enum FusabiError {
    #[error("Parse error at line {line}, column {col}: {message}")]
    ParseError {
        line: u32,
        col: u32,
        message: String,
    },

    #[error("Type error: expected {expected}, got {got}")]
    TypeError { expected: String, got: String },

    #[error("Runtime error: {0}")]
    RuntimeError(String),

    #[error("Undefined variable: {0}")]
    UndefinedVariable(String),

    #[error("Undefined function: {0}")]
    UndefinedFunction(String),

    #[error("Arity mismatch: expected {expected} arguments, got {got}")]
    ArityMismatch { expected: usize, got: usize },

    #[error("Division by zero")]
    DivisionByZero,

    #[error("Index out of bounds: {index} (length: {length})")]
    IndexOutOfBounds { index: usize, length: usize },

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("File watcher error: {0}")]
    WatcherError(String),
}

pub type Result<T> = std::result::Result<T, FusabiError>;

impl FusabiError {
    pub fn parse_error(line: u32, col: u32, message: impl Into<String>) -> Self {
        Self::ParseError {
            line,
            col,
            message: message.into(),
        }
    }

    pub fn type_error(expected: impl Into<String>, got: impl Into<String>) -> Self {
        Self::TypeError {
            expected: expected.into(),
            got: got.into(),
        }
    }

    pub fn runtime_error(message: impl Into<String>) -> Self {
        Self::RuntimeError(message.into())
    }
}
