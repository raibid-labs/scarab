use std::fmt;
use std::collections::HashMap;

/// Runtime values in Fusabi
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Nil,
    Bool(bool),
    Int(i64),
    Float(f64),
    String(String),
    List(Vec<Value>),
    Map(HashMap<String, Value>),
    Function(Function),
    #[cfg(feature = "bevy-integration")]
    Entity(bevy::ecs::entity::Entity),
}

impl Value {
    pub fn type_name(&self) -> &str {
        match self {
            Value::Nil => "nil",
            Value::Bool(_) => "bool",
            Value::Int(_) => "int",
            Value::Float(_) => "float",
            Value::String(_) => "string",
            Value::List(_) => "list",
            Value::Map(_) => "map",
            Value::Function(_) => "function",
            #[cfg(feature = "bevy-integration")]
            Value::Entity(_) => "entity",
        }
    }

    pub fn as_bool(&self) -> crate::error::Result<bool> {
        match self {
            Value::Bool(b) => Ok(*b),
            _ => Err(crate::error::FusabiError::type_error("bool", self.type_name())),
        }
    }

    pub fn as_int(&self) -> crate::error::Result<i64> {
        match self {
            Value::Int(i) => Ok(*i),
            _ => Err(crate::error::FusabiError::type_error("int", self.type_name())),
        }
    }

    pub fn as_float(&self) -> crate::error::Result<f64> {
        match self {
            Value::Float(f) => Ok(*f),
            Value::Int(i) => Ok(*i as f64),
            _ => Err(crate::error::FusabiError::type_error("float", self.type_name())),
        }
    }

    pub fn as_string(&self) -> crate::error::Result<String> {
        match self {
            Value::String(s) => Ok(s.clone()),
            _ => Err(crate::error::FusabiError::type_error("string", self.type_name())),
        }
    }

    pub fn as_list(&self) -> crate::error::Result<&Vec<Value>> {
        match self {
            Value::List(l) => Ok(l),
            _ => Err(crate::error::FusabiError::type_error("list", self.type_name())),
        }
    }

    pub fn as_map(&self) -> crate::error::Result<&HashMap<String, Value>> {
        match self {
            Value::Map(m) => Ok(m),
            _ => Err(crate::error::FusabiError::type_error("map", self.type_name())),
        }
    }

    pub fn is_truthy(&self) -> bool {
        match self {
            Value::Nil => false,
            Value::Bool(b) => *b,
            _ => true,
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Nil => write!(f, "nil"),
            Value::Bool(b) => write!(f, "{}", b),
            Value::Int(i) => write!(f, "{}", i),
            Value::Float(fl) => write!(f, "{}", fl),
            Value::String(s) => write!(f, "{}", s),
            Value::List(l) => {
                write!(f, "[")?;
                for (i, v) in l.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", v)?;
                }
                write!(f, "]")
            }
            Value::Map(m) => {
                write!(f, "{{")?;
                for (i, (k, v)) in m.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}: {}", k, v)?;
                }
                write!(f, "}}")
            }
            Value::Function(func) => write!(f, "<function {}>", func.name),
            #[cfg(feature = "bevy-integration")]
            Value::Entity(e) => write!(f, "<entity {:?}>", e),
        }
    }
}

/// Function representation
#[derive(Debug, Clone, PartialEq)]
pub struct Function {
    pub name: String,
    pub params: Vec<String>,
    pub body: FunctionBody,
}

#[derive(Debug, Clone, PartialEq)]
pub enum FunctionBody {
    /// User-defined function with AST body
    Expr(Box<Expr>),
    /// Built-in native function
    Native(NativeFunction),
}

#[derive(Clone)]
pub struct NativeFunction {
    pub name: String,
    pub arity: usize,
    pub func: fn(&[Value]) -> crate::error::Result<Value>,
}

impl fmt::Debug for NativeFunction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("NativeFunction")
            .field("name", &self.name)
            .field("arity", &self.arity)
            .finish()
    }
}

impl PartialEq for NativeFunction {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name && self.arity == other.arity
    }
}

/// AST Expression nodes
#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    // Literals
    Literal(Value),
    Variable(String),

    // Binary operations
    BinOp {
        op: BinOp,
        left: Box<Expr>,
        right: Box<Expr>,
    },

    // Unary operations
    UnaryOp {
        op: UnaryOp,
        expr: Box<Expr>,
    },

    // Function call
    Call {
        func: Box<Expr>,
        args: Vec<Expr>,
    },

    // Lambda expression
    Lambda {
        params: Vec<String>,
        body: Box<Expr>,
    },

    // Let binding
    Let {
        name: String,
        value: Box<Expr>,
        body: Box<Expr>,
    },

    // Conditional
    If {
        condition: Box<Expr>,
        then_expr: Box<Expr>,
        else_expr: Option<Box<Expr>>,
    },

    // List literal
    ListLiteral(Vec<Expr>),

    // Map literal
    MapLiteral(Vec<(String, Expr)>),

    // List/Map access
    Index {
        expr: Box<Expr>,
        index: Box<Expr>,
    },

    // Field access (for maps)
    Field {
        expr: Box<Expr>,
        field: String,
    },

    // Block expression (sequence)
    Block(Vec<Statement>),
}

/// Binary operators
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinOp {
    // Arithmetic
    Add,
    Sub,
    Mul,
    Div,
    Mod,

    // Comparison
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,

    // Logical
    And,
    Or,

    // String
    Concat,
}

/// Unary operators
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOp {
    Neg,
    Not,
}

/// Statements
#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    // Expression statement
    Expr(Expr),

    // Let binding (top-level or in block)
    Let {
        name: String,
        value: Expr,
    },

    // Function definition
    Function {
        name: String,
        params: Vec<String>,
        body: Box<Expr>,
    },

    // Return statement
    Return(Option<Expr>),
}

/// Top-level module/script
#[derive(Debug, Clone, PartialEq)]
pub struct Module {
    pub statements: Vec<Statement>,
}

impl Module {
    pub fn new(statements: Vec<Statement>) -> Self {
        Self { statements }
    }

    pub fn empty() -> Self {
        Self {
            statements: Vec::new(),
        }
    }
}
