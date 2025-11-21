//! Fusabi Bytecode Format (.fzb)
//!
//! This module defines the binary format for compiled Fusabi plugins.
//! Uses rkyv for zero-copy deserialization.

use rkyv::{Archive, Deserialize, Serialize};
use std::fmt;

/// Magic number for .fzb files: "FZB\0"
pub const MAGIC: [u8; 4] = *b"FZB\0";

/// Current bytecode version
pub const VERSION: u32 = 1;

/// Maximum function name length (bytes)
pub const MAX_NAME_LEN: usize = 256;

/// Maximum number of constants per module
pub const MAX_CONSTANTS: usize = 65536;

/// Maximum number of functions per module
pub const MAX_FUNCTIONS: usize = 4096;

/// Maximum bytecode size per function (bytes)
pub const MAX_FUNCTION_SIZE: usize = 1024 * 1024; // 1MB

/// Complete bytecode module
#[derive(Archive, Deserialize, Serialize, Debug, Clone)]
#[archive(check_bytes)]
pub struct FusabiBytecode {
    /// Magic identifier
    pub magic: [u8; 4],

    /// Bytecode format version
    pub version: u32,

    /// Constant pool
    pub constants: Vec<Value>,

    /// Function definitions
    pub functions: Vec<Function>,

    /// Entry point function index
    pub entry_point: u32,

    /// FFI function names (external calls)
    pub ffi_imports: Vec<String>,
}

impl FusabiBytecode {
    /// Create a new bytecode module
    pub fn new(entry_point: u32) -> Self {
        Self {
            magic: MAGIC,
            version: VERSION,
            constants: Vec::new(),
            functions: Vec::new(),
            entry_point,
            ffi_imports: Vec::new(),
        }
    }

    /// Validate bytecode integrity
    pub fn validate(&self) -> Result<(), BytecodeError> {
        // Check magic number
        if self.magic != MAGIC {
            return Err(BytecodeError::InvalidMagic);
        }

        // Check version
        if self.version != VERSION {
            return Err(BytecodeError::UnsupportedVersion(self.version));
        }

        // Check constants limit
        if self.constants.len() > MAX_CONSTANTS {
            return Err(BytecodeError::TooManyConstants);
        }

        // Check functions limit
        if self.functions.len() > MAX_FUNCTIONS {
            return Err(BytecodeError::TooManyFunctions);
        }

        // Check entry point
        if self.entry_point as usize >= self.functions.len() {
            return Err(BytecodeError::InvalidEntryPoint);
        }

        // Validate each function
        for (idx, func) in self.functions.iter().enumerate() {
            func.validate(idx)?;
        }

        Ok(())
    }

    /// Serialize to bytes using rkyv (zero-copy)
    pub fn to_bytes(&self) -> Result<Vec<u8>, BytecodeError> {
        let aligned = rkyv::to_bytes::<_, 256>(self)
            .map_err(|e| BytecodeError::SerializationError(e.to_string()))?;
        Ok(aligned.to_vec())
    }

    /// Deserialize from bytes (zero-copy)
    pub fn from_bytes(bytes: &[u8]) -> Result<&ArchivedFusabiBytecode, BytecodeError> {
        rkyv::check_archived_root::<FusabiBytecode>(bytes)
            .map_err(|e| BytecodeError::DeserializationError(e.to_string()))
    }
}

/// Function definition
#[derive(Archive, Deserialize, Serialize, Debug, Clone)]
#[archive(check_bytes)]
pub struct Function {
    /// Function name (for debugging)
    pub name: String,

    /// Parameter types
    pub params: Vec<Type>,

    /// Return type
    pub return_type: Type,

    /// Number of local variables
    pub locals: u32,

    /// Bytecode instructions
    pub bytecode: Vec<u8>,
}

impl Function {
    /// Validate function
    pub fn validate(&self, idx: usize) -> Result<(), BytecodeError> {
        // Check name length
        if self.name.len() > MAX_NAME_LEN {
            return Err(BytecodeError::NameTooLong(idx));
        }

        // Check bytecode size
        if self.bytecode.len() > MAX_FUNCTION_SIZE {
            return Err(BytecodeError::FunctionTooLarge(idx));
        }

        // Validate bytecode opcodes
        self.validate_opcodes(idx)?;

        Ok(())
    }

    fn validate_opcodes(&self, fn_idx: usize) -> Result<(), BytecodeError> {
        let mut pc = 0;
        while pc < self.bytecode.len() {
            let op = Opcode::decode(&self.bytecode[pc..])?;
            pc += op.size();

            // Validate opcode-specific constraints
            match op {
                Opcode::Load(local) | Opcode::Store(local) => {
                    if local >= self.locals {
                        return Err(BytecodeError::InvalidLocal(fn_idx, local));
                    }
                }
                _ => {}
            }
        }
        Ok(())
    }
}

/// VM Value types
#[derive(Archive, Deserialize, Serialize, Debug, Clone, PartialEq)]
#[archive(check_bytes)]
pub enum Value {
    Unit,
    Bool(bool),
    I32(i32),
    I64(i64),
    F32(f32),
    F64(f64),
    Char(char),
    String(String),
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Unit => write!(f, "()"),
            Value::Bool(b) => write!(f, "{}", b),
            Value::I32(i) => write!(f, "{}", i),
            Value::I64(i) => write!(f, "{}", i),
            Value::F32(fl) => write!(f, "{}", fl),
            Value::F64(fl) => write!(f, "{}", fl),
            Value::Char(c) => write!(f, "'{}'", c),
            Value::String(s) => write!(f, "\"{}\"", s),
        }
    }
}

impl Value {
    pub fn as_bool(&self) -> Result<bool, BytecodeError> {
        match self {
            Value::Bool(b) => Ok(*b),
            _ => Err(BytecodeError::TypeError {
                expected: "Bool",
                got: self.type_name(),
            }),
        }
    }

    pub fn as_i32(&self) -> Result<i32, BytecodeError> {
        match self {
            Value::I32(i) => Ok(*i),
            _ => Err(BytecodeError::TypeError {
                expected: "I32",
                got: self.type_name(),
            }),
        }
    }

    pub fn as_char(&self) -> Result<char, BytecodeError> {
        match self {
            Value::Char(c) => Ok(*c),
            _ => Err(BytecodeError::TypeError {
                expected: "Char",
                got: self.type_name(),
            }),
        }
    }

    pub fn type_name(&self) -> &'static str {
        match self {
            Value::Unit => "Unit",
            Value::Bool(_) => "Bool",
            Value::I32(_) => "I32",
            Value::I64(_) => "I64",
            Value::F32(_) => "F32",
            Value::F64(_) => "F64",
            Value::Char(_) => "Char",
            Value::String(_) => "String",
        }
    }
}

/// Type system
#[derive(Archive, Deserialize, Serialize, Debug, Clone, PartialEq)]
#[archive(check_bytes)]
pub enum Type {
    Unit,
    Bool,
    I32,
    I64,
    F32,
    F64,
    Char,
    String,
}

/// VM Opcodes
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Opcode {
    // Stack operations
    Push(u32),      // Push constant from pool
    Pop,            // Pop top of stack
    Dup,            // Duplicate top of stack

    // Local variables
    Load(u32),      // Load local variable
    Store(u32),     // Store to local variable

    // Control flow
    Call(u32),      // Call function by index
    CallFFI(u32),   // Call FFI function by index
    Ret,            // Return from function
    Jump(i32),      // Unconditional jump
    JumpIf(i32),    // Jump if top of stack is true
    JumpIfNot(i32), // Jump if top of stack is false

    // Arithmetic
    Add,            // Pop b, a; push a + b
    Sub,            // Pop b, a; push a - b
    Mul,            // Pop b, a; push a * b
    Div,            // Pop b, a; push a / b
    Mod,            // Pop b, a; push a % b
    Neg,            // Pop a; push -a

    // Comparison
    Eq,             // Pop b, a; push a == b
    Ne,             // Pop b, a; push a != b
    Lt,             // Pop b, a; push a < b
    Le,             // Pop b, a; push a <= b
    Gt,             // Pop b, a; push a > b
    Ge,             // Pop b, a; push a >= b

    // Logical
    And,            // Pop b, a; push a && b
    Or,             // Pop b, a; push a || b
    Not,            // Pop a; push !a

    // Special
    Nop,            // No operation
    Halt,           // Stop execution
}

impl Opcode {
    /// Decode opcode from bytecode stream
    pub fn decode(bytes: &[u8]) -> Result<Self, BytecodeError> {
        if bytes.is_empty() {
            return Err(BytecodeError::UnexpectedEndOfBytecode);
        }

        let op = match bytes[0] {
            0x00 => Opcode::Nop,
            0x01 => {
                if bytes.len() < 5 {
                    return Err(BytecodeError::UnexpectedEndOfBytecode);
                }
                let idx = u32::from_le_bytes([bytes[1], bytes[2], bytes[3], bytes[4]]);
                Opcode::Push(idx)
            }
            0x02 => Opcode::Pop,
            0x03 => Opcode::Dup,
            0x04 => {
                if bytes.len() < 5 {
                    return Err(BytecodeError::UnexpectedEndOfBytecode);
                }
                let idx = u32::from_le_bytes([bytes[1], bytes[2], bytes[3], bytes[4]]);
                Opcode::Load(idx)
            }
            0x05 => {
                if bytes.len() < 5 {
                    return Err(BytecodeError::UnexpectedEndOfBytecode);
                }
                let idx = u32::from_le_bytes([bytes[1], bytes[2], bytes[3], bytes[4]]);
                Opcode::Store(idx)
            }
            0x10 => {
                if bytes.len() < 5 {
                    return Err(BytecodeError::UnexpectedEndOfBytecode);
                }
                let idx = u32::from_le_bytes([bytes[1], bytes[2], bytes[3], bytes[4]]);
                Opcode::Call(idx)
            }
            0x11 => {
                if bytes.len() < 5 {
                    return Err(BytecodeError::UnexpectedEndOfBytecode);
                }
                let idx = u32::from_le_bytes([bytes[1], bytes[2], bytes[3], bytes[4]]);
                Opcode::CallFFI(idx)
            }
            0x12 => Opcode::Ret,
            0x13 => {
                if bytes.len() < 5 {
                    return Err(BytecodeError::UnexpectedEndOfBytecode);
                }
                let offset = i32::from_le_bytes([bytes[1], bytes[2], bytes[3], bytes[4]]);
                Opcode::Jump(offset)
            }
            0x14 => {
                if bytes.len() < 5 {
                    return Err(BytecodeError::UnexpectedEndOfBytecode);
                }
                let offset = i32::from_le_bytes([bytes[1], bytes[2], bytes[3], bytes[4]]);
                Opcode::JumpIf(offset)
            }
            0x15 => {
                if bytes.len() < 5 {
                    return Err(BytecodeError::UnexpectedEndOfBytecode);
                }
                let offset = i32::from_le_bytes([bytes[1], bytes[2], bytes[3], bytes[4]]);
                Opcode::JumpIfNot(offset)
            }
            0x20 => Opcode::Add,
            0x21 => Opcode::Sub,
            0x22 => Opcode::Mul,
            0x23 => Opcode::Div,
            0x24 => Opcode::Mod,
            0x25 => Opcode::Neg,
            0x30 => Opcode::Eq,
            0x31 => Opcode::Ne,
            0x32 => Opcode::Lt,
            0x33 => Opcode::Le,
            0x34 => Opcode::Gt,
            0x35 => Opcode::Ge,
            0x40 => Opcode::And,
            0x41 => Opcode::Or,
            0x42 => Opcode::Not,
            0xFF => Opcode::Halt,
            _ => return Err(BytecodeError::InvalidOpcode(bytes[0])),
        };

        Ok(op)
    }

    /// Encode opcode to bytes
    pub fn encode(&self) -> Vec<u8> {
        match self {
            Opcode::Nop => vec![0x00],
            Opcode::Push(idx) => {
                let mut bytes = vec![0x01];
                bytes.extend_from_slice(&idx.to_le_bytes());
                bytes
            }
            Opcode::Pop => vec![0x02],
            Opcode::Dup => vec![0x03],
            Opcode::Load(idx) => {
                let mut bytes = vec![0x04];
                bytes.extend_from_slice(&idx.to_le_bytes());
                bytes
            }
            Opcode::Store(idx) => {
                let mut bytes = vec![0x05];
                bytes.extend_from_slice(&idx.to_le_bytes());
                bytes
            }
            Opcode::Call(idx) => {
                let mut bytes = vec![0x10];
                bytes.extend_from_slice(&idx.to_le_bytes());
                bytes
            }
            Opcode::CallFFI(idx) => {
                let mut bytes = vec![0x11];
                bytes.extend_from_slice(&idx.to_le_bytes());
                bytes
            }
            Opcode::Ret => vec![0x12],
            Opcode::Jump(offset) => {
                let mut bytes = vec![0x13];
                bytes.extend_from_slice(&offset.to_le_bytes());
                bytes
            }
            Opcode::JumpIf(offset) => {
                let mut bytes = vec![0x14];
                bytes.extend_from_slice(&offset.to_le_bytes());
                bytes
            }
            Opcode::JumpIfNot(offset) => {
                let mut bytes = vec![0x15];
                bytes.extend_from_slice(&offset.to_le_bytes());
                bytes
            }
            Opcode::Add => vec![0x20],
            Opcode::Sub => vec![0x21],
            Opcode::Mul => vec![0x22],
            Opcode::Div => vec![0x23],
            Opcode::Mod => vec![0x24],
            Opcode::Neg => vec![0x25],
            Opcode::Eq => vec![0x30],
            Opcode::Ne => vec![0x31],
            Opcode::Lt => vec![0x32],
            Opcode::Le => vec![0x33],
            Opcode::Gt => vec![0x34],
            Opcode::Ge => vec![0x35],
            Opcode::And => vec![0x40],
            Opcode::Or => vec![0x41],
            Opcode::Not => vec![0x42],
            Opcode::Halt => vec![0xFF],
        }
    }

    /// Get size in bytes
    pub fn size(&self) -> usize {
        match self {
            Opcode::Push(_) | Opcode::Load(_) | Opcode::Store(_) |
            Opcode::Call(_) | Opcode::CallFFI(_) |
            Opcode::Jump(_) | Opcode::JumpIf(_) | Opcode::JumpIfNot(_) => 5,
            _ => 1,
        }
    }
}

/// Bytecode errors
#[derive(Debug, thiserror::Error)]
pub enum BytecodeError {
    #[error("Invalid magic number")]
    InvalidMagic,

    #[error("Unsupported bytecode version: {0}")]
    UnsupportedVersion(u32),

    #[error("Too many constants (max {MAX_CONSTANTS})")]
    TooManyConstants,

    #[error("Too many functions (max {MAX_FUNCTIONS})")]
    TooManyFunctions,

    #[error("Invalid entry point")]
    InvalidEntryPoint,

    #[error("Function name too long at index {0}")]
    NameTooLong(usize),

    #[error("Function too large at index {0}")]
    FunctionTooLarge(usize),

    #[error("Invalid local variable {1} in function {0}")]
    InvalidLocal(usize, u32),

    #[error("Invalid opcode: 0x{0:02X}")]
    InvalidOpcode(u8),

    #[error("Unexpected end of bytecode")]
    UnexpectedEndOfBytecode,

    #[error("Type error: expected {expected}, got {got}")]
    TypeError {
        expected: &'static str,
        got: &'static str,
    },

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Deserialization error: {0}")]
    DeserializationError(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_magic_constant() {
        assert_eq!(MAGIC, *b"FZB\0");
    }

    #[test]
    fn test_opcode_encoding() {
        let op = Opcode::Push(42);
        let bytes = op.encode();
        assert_eq!(bytes.len(), 5);
        assert_eq!(bytes[0], 0x01);

        let decoded = Opcode::decode(&bytes).unwrap();
        assert_eq!(decoded, op);
    }

    #[test]
    fn test_bytecode_serialization() {
        let mut bytecode = FusabiBytecode::new(0);
        bytecode.constants.push(Value::I32(42));
        bytecode.functions.push(Function {
            name: "main".to_string(),
            params: vec![],
            return_type: Type::I32,
            locals: 0,
            bytecode: Opcode::Push(0).encode(),
        });

        let bytes = bytecode.to_bytes().unwrap();
        let archived = FusabiBytecode::from_bytes(&bytes).unwrap();

        assert_eq!(archived.magic, MAGIC);
        assert_eq!(archived.version, VERSION);
    }

    #[test]
    fn test_value_conversions() {
        let v = Value::I32(42);
        assert_eq!(v.as_i32().unwrap(), 42);
        assert!(v.as_bool().is_err());
    }
}
