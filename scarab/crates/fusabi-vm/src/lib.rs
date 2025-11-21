//! Fusabi VM - AOT Runtime for Scarab Terminal
//!
//! A high-performance bytecode VM for executing compiled Fusabi plugins (.fzb)
//! with <1ms overhead.
//!
//! # Features
//!
//! - Zero-copy bytecode loading via rkyv
//! - Stack-based VM with efficient opcode dispatch
//! - FFI bridge to Rust functions
//! - Memory sandbox with 1GB limit
//! - Security enforcement (no syscalls)
//!
//! # Example
//!
//! ```no_run
//! use fusabi_vm::{FusabiVM, FusabiBytecode, Value};
//!
//! // Load bytecode
//! let bytecode_bytes = std::fs::read("plugin.fzb").unwrap();
//! let bytecode = FusabiBytecode::from_bytes(&bytecode_bytes).unwrap();
//!
//! // Create VM and register FFI functions
//! let mut vm = FusabiVM::new();
//! vm.register_ffi("print", |args| {
//!     println!("{}", args[0]);
//!     Ok(Value::Unit)
//! });
//!
//! // Execute
//! let result = vm.execute(bytecode).unwrap();
//! ```

pub mod bytecode;
pub mod ffi;
pub mod sandbox;
pub mod vm;

// Re-exports
pub use bytecode::{
    FusabiBytecode, Function, Opcode, Type, Value,
    BytecodeError, MAGIC, VERSION,
};
pub use ffi::{FfiFunction, FfiRegistry, FfiError, create_stdlib};
pub use sandbox::{Sandbox, SandboxError, SecurityPolicy, DEFAULT_MEMORY_LIMIT};
pub use vm::{FusabiVM, VmError, ExecutionStats};

/// VM builder for convenient configuration
pub struct VmBuilder {
    memory_limit: usize,
    stdlib: bool,
}

impl VmBuilder {
    /// Create a new VM builder
    pub fn new() -> Self {
        Self {
            memory_limit: DEFAULT_MEMORY_LIMIT,
            stdlib: true,
        }
    }

    /// Set memory limit
    pub fn memory_limit(mut self, limit: usize) -> Self {
        self.memory_limit = limit;
        self
    }

    /// Include/exclude standard library
    pub fn with_stdlib(mut self, enable: bool) -> Self {
        self.stdlib = enable;
        self
    }

    /// Build the VM
    pub fn build(self) -> FusabiVM {
        let mut vm = FusabiVM::new();

        if self.stdlib {
            let stdlib = create_stdlib();
            for name in stdlib.list() {
                if let Some(func) = stdlib.get(&name) {
                    vm.register_ffi(&name, func);
                }
            }
        }

        vm
    }
}

impl Default for VmBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Bytecode builder for convenient creation
pub struct BytecodeBuilder {
    bytecode: FusabiBytecode,
}

impl BytecodeBuilder {
    /// Create a new bytecode builder
    pub fn new() -> Self {
        Self {
            bytecode: FusabiBytecode::new(0),
        }
    }

    /// Add a constant
    pub fn add_constant(mut self, value: Value) -> (Self, u32) {
        let idx = self.bytecode.constants.len() as u32;
        self.bytecode.constants.push(value);
        (self, idx)
    }

    /// Add a function
    pub fn add_function(mut self, func: Function) -> (Self, u32) {
        let idx = self.bytecode.functions.len() as u32;
        self.bytecode.functions.push(func);
        (self, idx)
    }

    /// Add an FFI import
    pub fn add_ffi_import(mut self, name: &str) -> (Self, u32) {
        let idx = self.bytecode.ffi_imports.len() as u32;
        self.bytecode.ffi_imports.push(name.to_string());
        (self, idx)
    }

    /// Set entry point
    pub fn entry_point(mut self, idx: u32) -> Self {
        self.bytecode.entry_point = idx;
        self
    }

    /// Build the bytecode
    pub fn build(self) -> Result<FusabiBytecode, BytecodeError> {
        self.bytecode.validate()?;
        Ok(self.bytecode)
    }
}

impl Default for BytecodeBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vm_builder() {
        let vm = VmBuilder::new()
            .memory_limit(512 * 1024 * 1024)
            .with_stdlib(true)
            .build();

        // VM should be created successfully
        assert_eq!(vm.stats().instructions_executed, 0);
    }

    #[test]
    fn test_bytecode_builder() {
        let (builder, const_idx) = BytecodeBuilder::new()
            .add_constant(Value::I32(42));

        let (builder, _ffi_idx) = builder.add_ffi_import("print");

        let mut code = Vec::new();
        code.extend(Opcode::Push(const_idx).encode());
        code.extend(Opcode::Ret.encode());

        let func = Function {
            name: "main".to_string(),
            params: vec![],
            return_type: Type::I32,
            locals: 0,
            bytecode: code,
        };

        let (builder, func_idx) = builder.add_function(func);
        let bytecode = builder.entry_point(func_idx).build();

        assert!(bytecode.is_ok());
    }

    #[test]
    fn test_integration() {
        // Build bytecode
        let (builder, const_idx) = BytecodeBuilder::new()
            .add_constant(Value::I32(10));
        let (builder, const_idx2) = builder.add_constant(Value::I32(5));

        let mut code = Vec::new();
        code.extend(Opcode::Push(const_idx).encode());
        code.extend(Opcode::Push(const_idx2).encode());
        code.extend(Opcode::Add.encode());
        code.extend(Opcode::Ret.encode());

        let func = Function {
            name: "main".to_string(),
            params: vec![],
            return_type: Type::I32,
            locals: 0,
            bytecode: code,
        };

        let (builder, func_idx) = builder.add_function(func);
        let bytecode = builder.entry_point(func_idx).build().unwrap();

        // Serialize
        let bytes = bytecode.to_bytes().unwrap();
        let archived = FusabiBytecode::from_bytes(&bytes).unwrap();

        // Execute
        let mut vm = VmBuilder::new().build();
        let result = vm.execute(archived).unwrap();

        assert_eq!(result, Value::I32(15));
        assert!(vm.stats().instructions_executed > 0);
    }
}
