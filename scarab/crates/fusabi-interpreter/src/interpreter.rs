use std::collections::HashMap;
use crate::ast::*;
use crate::environment::Environment;
use crate::error::{FusabiError, Result};

/// The Fusabi interpreter
pub struct Interpreter {
    env: Environment,
}

impl Interpreter {
    pub fn new() -> Self {
        let mut interpreter = Self {
            env: Environment::new(),
        };

        // Load standard library
        crate::stdlib::load_stdlib(&mut interpreter.env);

        interpreter
    }

    /// Evaluate a module (list of statements)
    pub fn eval_module(&mut self, module: &Module) -> Result<Value> {
        let mut last_value = Value::Nil;

        for statement in &module.statements {
            last_value = self.eval_statement(statement)?;
        }

        Ok(last_value)
    }

    /// Evaluate a single statement
    pub fn eval_statement(&mut self, statement: &Statement) -> Result<Value> {
        match statement {
            Statement::Expr(expr) => self.eval_expr(expr),
            Statement::Let { name, value } => {
                let val = self.eval_expr(value)?;
                self.env.define(name.clone(), val.clone());
                Ok(val)
            }
            Statement::Function { name, params, body } => {
                let func = Value::Function(Function {
                    name: name.clone(),
                    params: params.clone(),
                    body: FunctionBody::Expr(body.clone()),
                });
                self.env.define(name.clone(), func.clone());
                Ok(func)
            }
            Statement::Return(expr) => {
                if let Some(expr) = expr {
                    self.eval_expr(expr)
                } else {
                    Ok(Value::Nil)
                }
            }
        }
    }

    /// Evaluate an expression
    pub fn eval_expr(&mut self, expr: &Expr) -> Result<Value> {
        match expr {
            Expr::Literal(value) => Ok(value.clone()),

            Expr::Variable(name) => self.env.get(name),

            Expr::BinOp { op, left, right } => {
                let left_val = self.eval_expr(left)?;
                let right_val = self.eval_expr(right)?;
                self.eval_binop(*op, left_val, right_val)
            }

            Expr::UnaryOp { op, expr } => {
                let val = self.eval_expr(expr)?;
                self.eval_unaryop(*op, val)
            }

            Expr::Call { func, args } => {
                let func_val = self.eval_expr(func)?;
                let arg_vals: Result<Vec<_>> = args.iter().map(|a| self.eval_expr(a)).collect();
                let arg_vals = arg_vals?;
                self.call_function(func_val, arg_vals)
            }

            Expr::Lambda { params, body } => {
                Ok(Value::Function(Function {
                    name: "<lambda>".to_string(),
                    params: params.clone(),
                    body: FunctionBody::Expr(body.clone()),
                }))
            }

            Expr::Let { name, value, body } => {
                let val = self.eval_expr(value)?;
                self.env.push_scope();
                self.env.define(name.clone(), val);
                let result = self.eval_expr(body);
                self.env.pop_scope();
                result
            }

            Expr::If { condition, then_expr, else_expr } => {
                let cond_val = self.eval_expr(condition)?;
                if cond_val.is_truthy() {
                    self.eval_expr(then_expr)
                } else if let Some(else_expr) = else_expr {
                    self.eval_expr(else_expr)
                } else {
                    Ok(Value::Nil)
                }
            }

            Expr::ListLiteral(exprs) => {
                let values: Result<Vec<_>> = exprs.iter().map(|e| self.eval_expr(e)).collect();
                Ok(Value::List(values?))
            }

            Expr::MapLiteral(entries) => {
                let mut map = HashMap::new();
                for (key, expr) in entries {
                    let value = self.eval_expr(expr)?;
                    map.insert(key.clone(), value);
                }
                Ok(Value::Map(map))
            }

            Expr::Index { expr, index } => {
                let val = self.eval_expr(expr)?;
                let idx = self.eval_expr(index)?;
                self.eval_index(val, idx)
            }

            Expr::Field { expr, field } => {
                let val = self.eval_expr(expr)?;
                match val {
                    Value::Map(map) => {
                        map.get(field)
                            .cloned()
                            .ok_or_else(|| FusabiError::runtime_error(format!("Field not found: {}", field)))
                    }
                    _ => Err(FusabiError::type_error("map", val.type_name())),
                }
            }

            Expr::Block(statements) => {
                self.env.push_scope();
                let mut last_value = Value::Nil;
                for statement in statements {
                    last_value = self.eval_statement(statement)?;
                }
                self.env.pop_scope();
                Ok(last_value)
            }
        }
    }

    fn eval_binop(&mut self, op: BinOp, left: Value, right: Value) -> Result<Value> {
        use BinOp::*;
        match (op, &left, &right) {
            // Arithmetic
            (Add, Value::Int(l), Value::Int(r)) => Ok(Value::Int(l + r)),
            (Add, Value::Float(l), Value::Float(r)) => Ok(Value::Float(l + r)),
            (Add, Value::Int(l), Value::Float(r)) => Ok(Value::Float(*l as f64 + r)),
            (Add, Value::Float(l), Value::Int(r)) => Ok(Value::Float(l + *r as f64)),
            (Add, Value::String(l), Value::String(r)) => Ok(Value::String(format!("{}{}", l, r))),

            (Sub, Value::Int(l), Value::Int(r)) => Ok(Value::Int(l - r)),
            (Sub, Value::Float(l), Value::Float(r)) => Ok(Value::Float(l - r)),
            (Sub, Value::Int(l), Value::Float(r)) => Ok(Value::Float(*l as f64 - r)),
            (Sub, Value::Float(l), Value::Int(r)) => Ok(Value::Float(l - *r as f64)),

            (Mul, Value::Int(l), Value::Int(r)) => Ok(Value::Int(l * r)),
            (Mul, Value::Float(l), Value::Float(r)) => Ok(Value::Float(l * r)),
            (Mul, Value::Int(l), Value::Float(r)) => Ok(Value::Float(*l as f64 * r)),
            (Mul, Value::Float(l), Value::Int(r)) => Ok(Value::Float(l * *r as f64)),

            (Div, Value::Int(l), Value::Int(r)) => {
                if *r == 0 {
                    Err(FusabiError::DivisionByZero)
                } else {
                    Ok(Value::Int(l / r))
                }
            }
            (Div, Value::Float(l), Value::Float(r)) => Ok(Value::Float(l / r)),
            (Div, Value::Int(l), Value::Float(r)) => Ok(Value::Float(*l as f64 / r)),
            (Div, Value::Float(l), Value::Int(r)) => Ok(Value::Float(l / *r as f64)),

            (Mod, Value::Int(l), Value::Int(r)) => {
                if *r == 0 {
                    Err(FusabiError::DivisionByZero)
                } else {
                    Ok(Value::Int(l % r))
                }
            }

            // Comparison
            (Eq, _, _) => Ok(Value::Bool(left == right)),
            (Ne, _, _) => Ok(Value::Bool(left != right)),

            (Lt, Value::Int(l), Value::Int(r)) => Ok(Value::Bool(l < r)),
            (Lt, Value::Float(l), Value::Float(r)) => Ok(Value::Bool(l < r)),
            (Le, Value::Int(l), Value::Int(r)) => Ok(Value::Bool(l <= r)),
            (Le, Value::Float(l), Value::Float(r)) => Ok(Value::Bool(l <= r)),
            (Gt, Value::Int(l), Value::Int(r)) => Ok(Value::Bool(l > r)),
            (Gt, Value::Float(l), Value::Float(r)) => Ok(Value::Bool(l > r)),
            (Ge, Value::Int(l), Value::Int(r)) => Ok(Value::Bool(l >= r)),
            (Ge, Value::Float(l), Value::Float(r)) => Ok(Value::Bool(l >= r)),

            // Logical
            (And, _, _) => Ok(Value::Bool(left.is_truthy() && right.is_truthy())),
            (Or, _, _) => Ok(Value::Bool(left.is_truthy() || right.is_truthy())),

            _ => Err(FusabiError::runtime_error(format!(
                "Invalid binary operation: {:?} {:?} {:?}",
                left.type_name(),
                op,
                right.type_name()
            ))),
        }
    }

    fn eval_unaryop(&mut self, op: UnaryOp, val: Value) -> Result<Value> {
        match (op, val) {
            (UnaryOp::Neg, Value::Int(i)) => Ok(Value::Int(-i)),
            (UnaryOp::Neg, Value::Float(f)) => Ok(Value::Float(-f)),
            (UnaryOp::Not, v) => Ok(Value::Bool(!v.is_truthy())),
            _ => Err(FusabiError::runtime_error("Invalid unary operation")),
        }
    }

    fn eval_index(&mut self, val: Value, idx: Value) -> Result<Value> {
        match (val, idx) {
            (Value::List(list), Value::Int(i)) => {
                if i < 0 || i as usize >= list.len() {
                    Err(FusabiError::IndexOutOfBounds {
                        index: i as usize,
                        length: list.len(),
                    })
                } else {
                    Ok(list[i as usize].clone())
                }
            }
            (Value::Map(map), Value::String(key)) => {
                map.get(&key)
                    .cloned()
                    .ok_or_else(|| FusabiError::runtime_error(format!("Key not found: {}", key)))
            }
            _ => Err(FusabiError::runtime_error("Invalid index operation")),
        }
    }

    fn call_function(&mut self, func: Value, args: Vec<Value>) -> Result<Value> {
        match func {
            Value::Function(Function { name: _, params, body }) => {
                // Check arity
                if params.len() != args.len() {
                    return Err(FusabiError::ArityMismatch {
                        expected: params.len(),
                        got: args.len(),
                    });
                }

                match body {
                    FunctionBody::Expr(expr) => {
                        // Create new scope and bind parameters
                        self.env.push_scope();
                        for (param, arg) in params.iter().zip(args.iter()) {
                            self.env.define(param.clone(), arg.clone());
                        }
                        let result = self.eval_expr(&expr);
                        self.env.pop_scope();
                        result
                    }
                    FunctionBody::Native(native) => {
                        // Call native function
                        if native.arity != args.len() {
                            return Err(FusabiError::ArityMismatch {
                                expected: native.arity,
                                got: args.len(),
                            });
                        }
                        (native.func)(&args)
                    }
                }
            }
            _ => Err(FusabiError::type_error("function", func.type_name())),
        }
    }

    /// Get the environment (for testing/debugging)
    pub fn env(&self) -> &Environment {
        &self.env
    }

    /// Get mutable environment (for stdlib loading)
    pub fn env_mut(&mut self) -> &mut Environment {
        &mut self.env
    }
}

impl Default for Interpreter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse_module;

    fn eval(code: &str) -> Result<Value> {
        let module = parse_module(code)?;
        let mut interp = Interpreter::new();
        interp.eval_module(&module)
    }

    #[test]
    fn test_arithmetic() {
        assert_eq!(eval("2 + 3").unwrap(), Value::Int(5));
        assert_eq!(eval("10 - 4").unwrap(), Value::Int(6));
        assert_eq!(eval("3 * 4").unwrap(), Value::Int(12));
        assert_eq!(eval("15 / 3").unwrap(), Value::Int(5));
    }

    #[test]
    fn test_comparison() {
        assert_eq!(eval("5 > 3").unwrap(), Value::Bool(true));
        assert_eq!(eval("5 < 3").unwrap(), Value::Bool(false));
        assert_eq!(eval("5 == 5").unwrap(), Value::Bool(true));
    }

    #[test]
    fn test_let_binding() {
        assert_eq!(eval("let x = 42 in x").unwrap(), Value::Int(42));
        assert_eq!(eval("let x = 10 in let y = 20 in x + y").unwrap(), Value::Int(30));
    }

    #[test]
    fn test_if_expr() {
        assert_eq!(eval("if true then 1 else 2").unwrap(), Value::Int(1));
        assert_eq!(eval("if false then 1 else 2").unwrap(), Value::Int(2));
    }

    #[test]
    fn test_function() {
        let code = r#"
            let add x y = x + y
            add 10 20
        "#;
        assert_eq!(eval(code).unwrap(), Value::Int(30));
    }

    #[test]
    fn test_lambda() {
        let code = "(fun x -> x + 1) 41";
        assert_eq!(eval(code).unwrap(), Value::Int(42));
    }

    #[test]
    fn test_list() {
        let code = "[1, 2, 3]";
        if let Value::List(list) = eval(code).unwrap() {
            assert_eq!(list.len(), 3);
        } else {
            panic!("Expected list");
        }
    }
}
