//! Fusabi Virtual Machine
//!
//! Stack-based VM with FFI support and security sandbox

use crate::bytecode::{ArchivedFusabiBytecode, Opcode, Value};
use crate::ffi::{FfiFunction, FfiRegistry};
use crate::sandbox::Sandbox;

/// Maximum stack depth
const MAX_STACK_SIZE: usize = 10_000;

/// Maximum call stack depth
const MAX_CALL_DEPTH: usize = 1_000;

/// Execution frame
#[derive(Debug, Clone)]
struct Frame {
    /// Function index
    function_idx: u32,

    /// Program counter
    pc: usize,

    /// Base pointer for locals
    base_ptr: usize,

    /// Number of locals
    num_locals: u32,
}

/// Virtual Machine state
pub struct FusabiVM {
    /// Value stack
    stack: Vec<Value>,

    /// Call stack
    call_stack: Vec<Frame>,

    /// FFI function registry
    ffi_registry: FfiRegistry,

    /// Memory sandbox
    sandbox: Sandbox,

    /// Execution statistics
    stats: ExecutionStats,
}

/// Execution statistics
#[derive(Debug, Default)]
pub struct ExecutionStats {
    pub instructions_executed: u64,
    pub ffi_calls: u64,
    pub max_stack_depth: usize,
    pub max_call_depth: usize,
}

impl FusabiVM {
    /// Create a new VM with default settings
    pub fn new() -> Self {
        Self {
            stack: Vec::with_capacity(1024),
            call_stack: Vec::with_capacity(64),
            ffi_registry: FfiRegistry::new(),
            sandbox: Sandbox::new(),
            stats: ExecutionStats::default(),
        }
    }

    /// Register an FFI function
    pub fn register_ffi(&mut self, name: &str, func: FfiFunction) {
        self.ffi_registry.register(name, func);
    }

    /// Execute bytecode from archived format (zero-copy)
    pub fn execute(&mut self, bytecode: &ArchivedFusabiBytecode) -> Result<Value, VmError> {
        // Validate bytecode
        if bytecode.magic != crate::bytecode::MAGIC {
            return Err(VmError::InvalidBytecode("Bad magic number".into()));
        }

        // Get entry point
        let entry_idx = bytecode.entry_point as usize;
        if entry_idx >= bytecode.functions.len() {
            return Err(VmError::InvalidBytecode("Invalid entry point".into()));
        }

        // Initialize execution
        self.reset();
        self.push_frame(entry_idx, 0)?;

        // Main execution loop
        while !self.call_stack.is_empty() {
            self.step(bytecode)?;
        }

        // Return final stack value or Unit
        Ok(self.stack.pop().unwrap_or(Value::Unit))
    }

    /// Reset VM state
    fn reset(&mut self) {
        self.stack.clear();
        self.call_stack.clear();
        self.sandbox.reset();
        self.stats = ExecutionStats::default();
    }

    /// Push a new call frame
    fn push_frame(&mut self, function_idx: usize, num_locals: u32) -> Result<(), VmError> {
        if self.call_stack.len() >= MAX_CALL_DEPTH {
            return Err(VmError::StackOverflow);
        }

        let base_ptr = self.stack.len();

        // Reserve space for locals
        for _ in 0..num_locals {
            self.stack.push(Value::Unit);
        }

        self.call_stack.push(Frame {
            function_idx: function_idx as u32,
            pc: 0,
            base_ptr,
            num_locals,
        });

        self.stats.max_call_depth = self.stats.max_call_depth.max(self.call_stack.len());

        Ok(())
    }

    /// Pop current call frame
    fn pop_frame(&mut self) -> Result<(), VmError> {
        let frame = self.call_stack.pop().ok_or(VmError::StackUnderflow)?;

        // Save return value if present
        let return_value = if self.stack.len() > frame.base_ptr {
            Some(self.stack.pop().unwrap())
        } else {
            None
        };

        // Remove locals from stack
        self.stack.truncate(frame.base_ptr);

        // Push return value back
        if let Some(value) = return_value {
            self.stack.push(value);
        }

        Ok(())
    }

    /// Execute one instruction
    fn step(&mut self, bytecode: &ArchivedFusabiBytecode) -> Result<(), VmError> {
        let frame = self.call_stack.last_mut().ok_or(VmError::StackUnderflow)?;
        let function = &bytecode.functions[frame.function_idx as usize];
        let bytecode_slice = &function.bytecode[frame.pc..];

        if bytecode_slice.is_empty() {
            return Err(VmError::UnexpectedEndOfBytecode);
        }

        let opcode = Opcode::decode(bytecode_slice)
            .map_err(|e| VmError::InvalidBytecode(e.to_string()))?;

        // Update program counter
        let pc_increment = opcode.size();
        frame.pc += pc_increment;

        // Execute opcode
        self.execute_opcode(opcode, bytecode)?;

        self.stats.instructions_executed += 1;
        self.stats.max_stack_depth = self.stats.max_stack_depth.max(self.stack.len());

        Ok(())
    }

    /// Execute a single opcode
    fn execute_opcode(
        &mut self,
        opcode: Opcode,
        bytecode: &ArchivedFusabiBytecode,
    ) -> Result<(), VmError> {
        match opcode {
            // Stack operations
            Opcode::Nop => {}

            Opcode::Push(idx) => {
                let value = bytecode.constants.get(idx as usize)
                    .ok_or(VmError::InvalidConstant(idx))?;
                self.push(self.archive_value_to_value(value))?;
            }

            Opcode::Pop => {
                self.pop()?;
            }

            Opcode::Dup => {
                let value = self.peek()?.clone();
                self.push(value)?;
            }

            // Local variables
            Opcode::Load(local_idx) => {
                let frame = self.call_stack.last().ok_or(VmError::StackUnderflow)?;
                if local_idx >= frame.num_locals {
                    return Err(VmError::InvalidLocal(local_idx));
                }
                let value = self.stack[frame.base_ptr + local_idx as usize].clone();
                self.push(value)?;
            }

            Opcode::Store(local_idx) => {
                let base_ptr = {
                    let frame = self.call_stack.last().ok_or(VmError::StackUnderflow)?;
                    if local_idx >= frame.num_locals {
                        return Err(VmError::InvalidLocal(local_idx));
                    }
                    frame.base_ptr
                };
                let value = self.pop()?;
                self.stack[base_ptr + local_idx as usize] = value;
            }

            // Control flow
            Opcode::Call(func_idx) => {
                let function = bytecode.functions.get(func_idx as usize)
                    .ok_or(VmError::InvalidFunction(func_idx))?;
                self.push_frame(func_idx as usize, function.locals)?;
            }

            Opcode::CallFFI(ffi_idx) => {
                let ffi_name = bytecode.ffi_imports.get(ffi_idx as usize)
                    .ok_or(VmError::InvalidFfiImport(ffi_idx))?;

                let function = self.ffi_registry.get(&ffi_name.to_string())
                    .ok_or_else(|| VmError::FfiNotFound(ffi_name.to_string()))?;

                let result = function(&self.stack)
                    .map_err(|e| VmError::FfiError(e.to_string()))?;

                self.push(result)?;
                self.stats.ffi_calls += 1;
            }

            Opcode::Ret => {
                self.pop_frame()?;
            }

            Opcode::Jump(offset) => {
                let frame = self.call_stack.last_mut().ok_or(VmError::StackUnderflow)?;
                let new_pc = (frame.pc as i32 + offset) as usize;
                frame.pc = new_pc;
            }

            Opcode::JumpIf(offset) => {
                let cond = self.pop()?.as_bool()
                    .map_err(|e| VmError::InvalidBytecode(e.to_string()))?;
                if cond {
                    let frame = self.call_stack.last_mut().ok_or(VmError::StackUnderflow)?;
                    let new_pc = (frame.pc as i32 + offset) as usize;
                    frame.pc = new_pc;
                }
            }

            Opcode::JumpIfNot(offset) => {
                let cond = self.pop()?.as_bool()
                    .map_err(|e| VmError::InvalidBytecode(e.to_string()))?;
                if !cond {
                    let frame = self.call_stack.last_mut().ok_or(VmError::StackUnderflow)?;
                    let new_pc = (frame.pc as i32 + offset) as usize;
                    frame.pc = new_pc;
                }
            }

            // Arithmetic
            Opcode::Add => self.binary_op(|a, b| match (a, b) {
                (Value::I32(x), Value::I32(y)) => Ok(Value::I32(x + y)),
                (Value::I64(x), Value::I64(y)) => Ok(Value::I64(x + y)),
                (Value::F32(x), Value::F32(y)) => Ok(Value::F32(x + y)),
                (Value::F64(x), Value::F64(y)) => Ok(Value::F64(x + y)),
                _ => Err(VmError::TypeMismatch),
            })?,

            Opcode::Sub => self.binary_op(|a, b| match (a, b) {
                (Value::I32(x), Value::I32(y)) => Ok(Value::I32(x - y)),
                (Value::I64(x), Value::I64(y)) => Ok(Value::I64(x - y)),
                (Value::F32(x), Value::F32(y)) => Ok(Value::F32(x - y)),
                (Value::F64(x), Value::F64(y)) => Ok(Value::F64(x - y)),
                _ => Err(VmError::TypeMismatch),
            })?,

            Opcode::Mul => self.binary_op(|a, b| match (a, b) {
                (Value::I32(x), Value::I32(y)) => Ok(Value::I32(x * y)),
                (Value::I64(x), Value::I64(y)) => Ok(Value::I64(x * y)),
                (Value::F32(x), Value::F32(y)) => Ok(Value::F32(x * y)),
                (Value::F64(x), Value::F64(y)) => Ok(Value::F64(x * y)),
                _ => Err(VmError::TypeMismatch),
            })?,

            Opcode::Div => self.binary_op(|a, b| match (a, b) {
                (Value::I32(x), Value::I32(y)) => {
                    if *y == 0 {
                        Err(VmError::DivisionByZero)
                    } else {
                        Ok(Value::I32(x / y))
                    }
                }
                (Value::I64(x), Value::I64(y)) => {
                    if *y == 0 {
                        Err(VmError::DivisionByZero)
                    } else {
                        Ok(Value::I64(x / y))
                    }
                }
                (Value::F32(x), Value::F32(y)) => Ok(Value::F32(x / y)),
                (Value::F64(x), Value::F64(y)) => Ok(Value::F64(x / y)),
                _ => Err(VmError::TypeMismatch),
            })?,

            Opcode::Mod => self.binary_op(|a, b| match (a, b) {
                (Value::I32(x), Value::I32(y)) => {
                    if *y == 0 {
                        Err(VmError::DivisionByZero)
                    } else {
                        Ok(Value::I32(x % y))
                    }
                }
                (Value::I64(x), Value::I64(y)) => {
                    if *y == 0 {
                        Err(VmError::DivisionByZero)
                    } else {
                        Ok(Value::I64(x % y))
                    }
                }
                _ => Err(VmError::TypeMismatch),
            })?,

            Opcode::Neg => {
                let val = self.pop()?;
                let result = match val {
                    Value::I32(x) => Value::I32(-x),
                    Value::I64(x) => Value::I64(-x),
                    Value::F32(x) => Value::F32(-x),
                    Value::F64(x) => Value::F64(-x),
                    _ => return Err(VmError::TypeMismatch),
                };
                self.push(result)?;
            }

            // Comparison
            Opcode::Eq => self.binary_op(|a, b| Ok(Value::Bool(a == b)))?,
            Opcode::Ne => self.binary_op(|a, b| Ok(Value::Bool(a != b)))?,

            Opcode::Lt => self.binary_op(|a, b| match (a, b) {
                (Value::I32(x), Value::I32(y)) => Ok(Value::Bool(x < y)),
                (Value::I64(x), Value::I64(y)) => Ok(Value::Bool(x < y)),
                (Value::F32(x), Value::F32(y)) => Ok(Value::Bool(x < y)),
                (Value::F64(x), Value::F64(y)) => Ok(Value::Bool(x < y)),
                _ => Err(VmError::TypeMismatch),
            })?,

            Opcode::Le => self.binary_op(|a, b| match (a, b) {
                (Value::I32(x), Value::I32(y)) => Ok(Value::Bool(x <= y)),
                (Value::I64(x), Value::I64(y)) => Ok(Value::Bool(x <= y)),
                (Value::F32(x), Value::F32(y)) => Ok(Value::Bool(x <= y)),
                (Value::F64(x), Value::F64(y)) => Ok(Value::Bool(x <= y)),
                _ => Err(VmError::TypeMismatch),
            })?,

            Opcode::Gt => self.binary_op(|a, b| match (a, b) {
                (Value::I32(x), Value::I32(y)) => Ok(Value::Bool(x > y)),
                (Value::I64(x), Value::I64(y)) => Ok(Value::Bool(x > y)),
                (Value::F32(x), Value::F32(y)) => Ok(Value::Bool(x > y)),
                (Value::F64(x), Value::F64(y)) => Ok(Value::Bool(x > y)),
                _ => Err(VmError::TypeMismatch),
            })?,

            Opcode::Ge => self.binary_op(|a, b| match (a, b) {
                (Value::I32(x), Value::I32(y)) => Ok(Value::Bool(x >= y)),
                (Value::I64(x), Value::I64(y)) => Ok(Value::Bool(x >= y)),
                (Value::F32(x), Value::F32(y)) => Ok(Value::Bool(x >= y)),
                (Value::F64(x), Value::F64(y)) => Ok(Value::Bool(x >= y)),
                _ => Err(VmError::TypeMismatch),
            })?,

            // Logical
            Opcode::And => self.binary_op(|a, b| match (a, b) {
                (Value::Bool(x), Value::Bool(y)) => Ok(Value::Bool(*x && *y)),
                _ => Err(VmError::TypeMismatch),
            })?,

            Opcode::Or => self.binary_op(|a, b| match (a, b) {
                (Value::Bool(x), Value::Bool(y)) => Ok(Value::Bool(*x || *y)),
                _ => Err(VmError::TypeMismatch),
            })?,

            Opcode::Not => {
                let val = self.pop()?;
                match val {
                    Value::Bool(b) => self.push(Value::Bool(!b))?,
                    _ => return Err(VmError::TypeMismatch),
                }
            }

            Opcode::Halt => {
                self.call_stack.clear();
            }
        }

        Ok(())
    }

    /// Helper for binary operations
    fn binary_op<F>(&mut self, op: F) -> Result<(), VmError>
    where
        F: FnOnce(&Value, &Value) -> Result<Value, VmError>,
    {
        let b = self.pop()?;
        let a = self.pop()?;
        let result = op(&a, &b)?;
        self.push(result)?;
        Ok(())
    }

    /// Push value to stack
    fn push(&mut self, value: Value) -> Result<(), VmError> {
        if self.stack.len() >= MAX_STACK_SIZE {
            return Err(VmError::StackOverflow);
        }
        self.stack.push(value);
        Ok(())
    }

    /// Pop value from stack
    fn pop(&mut self) -> Result<Value, VmError> {
        self.stack.pop().ok_or(VmError::StackUnderflow)
    }

    /// Peek top of stack
    fn peek(&self) -> Result<&Value, VmError> {
        self.stack.last().ok_or(VmError::StackUnderflow)
    }

    /// Convert archived value to runtime value
    fn archive_value_to_value(&self, archived: &crate::bytecode::ArchivedValue) -> Value {
        // This is a simplified conversion - in production we'd use proper rkyv deserialization
        match archived {
            crate::bytecode::ArchivedValue::Unit => Value::Unit,
            crate::bytecode::ArchivedValue::Bool(b) => Value::Bool(*b),
            crate::bytecode::ArchivedValue::I32(i) => Value::I32(*i),
            crate::bytecode::ArchivedValue::I64(i) => Value::I64(*i),
            crate::bytecode::ArchivedValue::F32(f) => Value::F32(*f),
            crate::bytecode::ArchivedValue::F64(f) => Value::F64(*f),
            crate::bytecode::ArchivedValue::Char(c) => Value::Char(*c),
            crate::bytecode::ArchivedValue::String(s) => Value::String(s.to_string()),
        }
    }

    /// Get execution statistics
    pub fn stats(&self) -> &ExecutionStats {
        &self.stats
    }
}

impl Default for FusabiVM {
    fn default() -> Self {
        Self::new()
    }
}

/// VM Errors
#[derive(Debug, thiserror::Error)]
pub enum VmError {
    #[error("Stack overflow")]
    StackOverflow,

    #[error("Stack underflow")]
    StackUnderflow,

    #[error("Invalid bytecode: {0}")]
    InvalidBytecode(String),

    #[error("Invalid constant index: {0}")]
    InvalidConstant(u32),

    #[error("Invalid function index: {0}")]
    InvalidFunction(u32),

    #[error("Invalid local variable: {0}")]
    InvalidLocal(u32),

    #[error("Invalid FFI import: {0}")]
    InvalidFfiImport(u32),

    #[error("FFI function not found: {0}")]
    FfiNotFound(String),

    #[error("FFI error: {0}")]
    FfiError(String),

    #[error("Type mismatch")]
    TypeMismatch,

    #[error("Division by zero")]
    DivisionByZero,

    #[error("Unexpected end of bytecode")]
    UnexpectedEndOfBytecode,

    #[error("Memory violation: {0}")]
    MemoryViolation(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bytecode::{FusabiBytecode, Function, Type};

    #[test]
    fn test_vm_creation() {
        let vm = FusabiVM::new();
        assert_eq!(vm.stack.len(), 0);
        assert_eq!(vm.call_stack.len(), 0);
    }

    #[test]
    fn test_simple_push_execution() {
        let mut bytecode = FusabiBytecode::new(0);
        bytecode.constants.push(Value::I32(42));

        let mut code = Vec::new();
        code.extend(Opcode::Push(0).encode());
        code.extend(Opcode::Ret.encode());

        bytecode.functions.push(Function {
            name: "main".to_string(),
            params: vec![],
            return_type: Type::I32,
            locals: 0,
            bytecode: code,
        });

        let bytes = bytecode.to_bytes().unwrap();
        let archived = FusabiBytecode::from_bytes(&bytes).unwrap();

        let mut vm = FusabiVM::new();
        let result = vm.execute(archived).unwrap();

        assert_eq!(result, Value::I32(42));
    }

    #[test]
    fn test_arithmetic() {
        let mut bytecode = FusabiBytecode::new(0);
        bytecode.constants.push(Value::I32(10));
        bytecode.constants.push(Value::I32(5));

        let mut code = Vec::new();
        code.extend(Opcode::Push(0).encode()); // 10
        code.extend(Opcode::Push(1).encode()); // 5
        code.extend(Opcode::Add.encode());     // 10 + 5
        code.extend(Opcode::Ret.encode());

        bytecode.functions.push(Function {
            name: "main".to_string(),
            params: vec![],
            return_type: Type::I32,
            locals: 0,
            bytecode: code,
        });

        let bytes = bytecode.to_bytes().unwrap();
        let archived = FusabiBytecode::from_bytes(&bytes).unwrap();

        let mut vm = FusabiVM::new();
        let result = vm.execute(archived).unwrap();

        assert_eq!(result, Value::I32(15));
    }
}
