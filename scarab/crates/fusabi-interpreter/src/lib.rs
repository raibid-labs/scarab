// fusabi-interpreter: The Script runtime for the Client
//
// A hot-reloadable interpreter for .fsx scripts (F# subset) that enables:
// - Custom UI layouts and themes
// - Interactive overlays (Vimium-style, Spacemacs-like menus)
// - Dynamic key bindings
// - Runtime customization without Rust recompilation
//
// Features:
// - Fast parsing (<10ms for 1000 LOC)
// - Hot-reload in <100ms
// - Rich standard library (50+ functions)
// - Bevy integration for UI entity spawning
// - File watcher for automatic reloading
// - AST caching for performance

pub mod ast;
pub mod cache;
pub mod environment;
pub mod error;
pub mod interpreter;
pub mod parser;
pub mod stdlib;
pub mod watcher;

#[cfg(feature = "bevy-integration")]
pub mod bevy_integration;

// Re-exports for convenience
pub use ast::{Expr, Function, Module, Statement, Value};
pub use cache::AstCache;
pub use environment::Environment;
pub use error::{FusabiError, Result};
pub use interpreter::Interpreter;
pub use parser::parse_module;
pub use watcher::ScriptWatcher;

#[cfg(feature = "bevy-integration")]
pub use bevy_integration::{FusabiInterpreter, FusabiPlugin, ScriptReloadEvent};

/// Quick evaluation of a script string
pub fn eval(code: &str) -> Result<Value> {
    let module = parse_module(code)?;
    let mut interpreter = Interpreter::new();
    interpreter.eval_module(&module)
}

/// Parse and validate a script without executing it
pub fn parse(code: &str) -> Result<Module> {
    parse_module(code)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_eval_simple() {
        assert_eq!(eval("42").unwrap(), Value::Int(42));
        assert_eq!(eval("2 + 3").unwrap(), Value::Int(5));
        assert_eq!(eval(r#""hello""#).unwrap(), Value::String("hello".to_string()));
    }

    #[test]
    fn test_eval_function() {
        let code = r#"
            let add x y = x + y
            add 10 20
        "#;
        assert_eq!(eval(code).unwrap(), Value::Int(30));
    }

    #[test]
    fn test_eval_stdlib() {
        assert_eq!(eval("abs -42").unwrap(), Value::Int(42));
        assert_eq!(eval(r#"strlen "hello""#).unwrap(), Value::Int(5));
    }
}
