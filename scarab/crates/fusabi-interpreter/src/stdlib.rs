use crate::ast::{Function, FunctionBody, NativeFunction, Value};
use crate::environment::Environment;
use crate::error::{FusabiError, Result};

/// Load the standard library into the environment
pub fn load_stdlib(env: &mut Environment) {
    // String functions
    register_native(env, "strlen", 1, string_length);
    register_native(env, "substr", 3, substring);
    register_native(env, "concat", 2, concat);
    register_native(env, "to_upper", 1, to_upper);
    register_native(env, "to_lower", 1, to_lower);
    register_native(env, "trim", 1, trim);
    register_native(env, "split", 2, split);
    register_native(env, "join", 2, join);

    // List functions
    register_native(env, "length", 1, list_length);
    register_native(env, "head", 1, list_head);
    register_native(env, "tail", 1, list_tail);
    register_native(env, "cons", 2, list_cons);
    register_native(env, "append", 2, list_append);
    register_native(env, "reverse", 1, list_reverse);
    register_native(env, "nth", 2, list_nth);
    register_native(env, "take", 2, list_take);
    register_native(env, "drop", 2, list_drop);

    // Map functions
    register_native(env, "keys", 1, map_keys);
    register_native(env, "values", 1, map_values);
    register_native(env, "has_key", 2, map_has_key);

    // Math functions
    register_native(env, "abs", 1, abs);
    register_native(env, "min", 2, min);
    register_native(env, "max", 2, max);
    register_native(env, "pow", 2, pow);
    register_native(env, "sqrt", 1, sqrt);
    register_native(env, "floor", 1, floor);
    register_native(env, "ceil", 1, ceil);
    register_native(env, "round", 1, round);

    // Conversion functions
    register_native(env, "to_int", 1, to_int);
    register_native(env, "to_float", 1, to_float);
    register_native(env, "to_string", 1, to_string);

    // IO functions
    register_native(env, "print", 1, print);
    register_native(env, "println", 1, println);

    // Type checking
    register_native(env, "is_nil", 1, is_nil);
    register_native(env, "is_bool", 1, is_bool);
    register_native(env, "is_int", 1, is_int);
    register_native(env, "is_float", 1, is_float);
    register_native(env, "is_string", 1, is_string);
    register_native(env, "is_list", 1, is_list);
    register_native(env, "is_map", 1, is_map);
    register_native(env, "is_function", 1, is_function);

    // Functional programming
    register_native(env, "map", 2, map_fn);
    register_native(env, "filter", 2, filter_fn);
    register_native(env, "fold", 3, fold_fn);
}

fn register_native(env: &mut Environment, name: &str, arity: usize, func: fn(&[Value]) -> Result<Value>) {
    let native = NativeFunction {
        name: name.to_string(),
        arity,
        func,
    };
    env.define(
        name.to_string(),
        Value::Function(Function {
            name: name.to_string(),
            params: (0..arity).map(|i| format!("arg{}", i)).collect(),
            body: FunctionBody::Native(native),
        }),
    );
}

// String functions
fn string_length(args: &[Value]) -> Result<Value> {
    let s = args[0].as_string()?;
    Ok(Value::Int(s.len() as i64))
}

fn substring(args: &[Value]) -> Result<Value> {
    let s = args[0].as_string()?;
    let start = args[1].as_int()? as usize;
    let end = args[2].as_int()? as usize;

    if start > s.len() || end > s.len() || start > end {
        return Err(FusabiError::runtime_error("Invalid substring indices"));
    }

    Ok(Value::String(s[start..end].to_string()))
}

fn concat(args: &[Value]) -> Result<Value> {
    let s1 = args[0].as_string()?;
    let s2 = args[1].as_string()?;
    Ok(Value::String(format!("{}{}", s1, s2)))
}

fn to_upper(args: &[Value]) -> Result<Value> {
    let s = args[0].as_string()?;
    Ok(Value::String(s.to_uppercase()))
}

fn to_lower(args: &[Value]) -> Result<Value> {
    let s = args[0].as_string()?;
    Ok(Value::String(s.to_lowercase()))
}

fn trim(args: &[Value]) -> Result<Value> {
    let s = args[0].as_string()?;
    Ok(Value::String(s.trim().to_string()))
}

fn split(args: &[Value]) -> Result<Value> {
    let s = args[0].as_string()?;
    let delimiter = args[1].as_string()?;
    let parts: Vec<Value> = s
        .split(&delimiter)
        .map(|p| Value::String(p.to_string()))
        .collect();
    Ok(Value::List(parts))
}

fn join(args: &[Value]) -> Result<Value> {
    let list = args[0].as_list()?;
    let separator = args[1].as_string()?;

    let strings: Result<Vec<String>> = list.iter().map(|v| v.as_string()).collect();
    let strings = strings?;

    Ok(Value::String(strings.join(&separator)))
}

// List functions
fn list_length(args: &[Value]) -> Result<Value> {
    let list = args[0].as_list()?;
    Ok(Value::Int(list.len() as i64))
}

fn list_head(args: &[Value]) -> Result<Value> {
    let list = args[0].as_list()?;
    if list.is_empty() {
        Err(FusabiError::runtime_error("head of empty list"))
    } else {
        Ok(list[0].clone())
    }
}

fn list_tail(args: &[Value]) -> Result<Value> {
    let list = args[0].as_list()?;
    if list.is_empty() {
        Err(FusabiError::runtime_error("tail of empty list"))
    } else {
        Ok(Value::List(list[1..].to_vec()))
    }
}

fn list_cons(args: &[Value]) -> Result<Value> {
    let elem = args[0].clone();
    let list = args[1].as_list()?;
    let mut new_list = vec![elem];
    new_list.extend_from_slice(list);
    Ok(Value::List(new_list))
}

fn list_append(args: &[Value]) -> Result<Value> {
    let list1 = args[0].as_list()?;
    let list2 = args[1].as_list()?;
    let mut new_list = list1.clone();
    new_list.extend_from_slice(list2);
    Ok(Value::List(new_list))
}

fn list_reverse(args: &[Value]) -> Result<Value> {
    let list = args[0].as_list()?;
    let mut new_list = list.clone();
    new_list.reverse();
    Ok(Value::List(new_list))
}

fn list_nth(args: &[Value]) -> Result<Value> {
    let list = args[0].as_list()?;
    let n = args[1].as_int()? as usize;

    if n >= list.len() {
        Err(FusabiError::IndexOutOfBounds {
            index: n,
            length: list.len(),
        })
    } else {
        Ok(list[n].clone())
    }
}

fn list_take(args: &[Value]) -> Result<Value> {
    let n = args[0].as_int()? as usize;
    let list = args[1].as_list()?;
    let n = n.min(list.len());
    Ok(Value::List(list[..n].to_vec()))
}

fn list_drop(args: &[Value]) -> Result<Value> {
    let n = args[0].as_int()? as usize;
    let list = args[1].as_list()?;
    if n >= list.len() {
        Ok(Value::List(vec![]))
    } else {
        Ok(Value::List(list[n..].to_vec()))
    }
}

// Map functions
fn map_keys(args: &[Value]) -> Result<Value> {
    let map = args[0].as_map()?;
    let keys: Vec<Value> = map.keys().map(|k| Value::String(k.clone())).collect();
    Ok(Value::List(keys))
}

fn map_values(args: &[Value]) -> Result<Value> {
    let map = args[0].as_map()?;
    let values: Vec<Value> = map.values().cloned().collect();
    Ok(Value::List(values))
}

fn map_has_key(args: &[Value]) -> Result<Value> {
    let map = args[0].as_map()?;
    let key = args[1].as_string()?;
    Ok(Value::Bool(map.contains_key(&key)))
}

// Math functions
fn abs(args: &[Value]) -> Result<Value> {
    match &args[0] {
        Value::Int(i) => Ok(Value::Int(i.abs())),
        Value::Float(f) => Ok(Value::Float(f.abs())),
        _ => Err(FusabiError::type_error("number", args[0].type_name())),
    }
}

fn min(args: &[Value]) -> Result<Value> {
    match (&args[0], &args[1]) {
        (Value::Int(a), Value::Int(b)) => Ok(Value::Int(*a.min(b))),
        (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a.min(*b))),
        _ => Err(FusabiError::runtime_error("min requires two numbers")),
    }
}

fn max(args: &[Value]) -> Result<Value> {
    match (&args[0], &args[1]) {
        (Value::Int(a), Value::Int(b)) => Ok(Value::Int(*a.max(b))),
        (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a.max(*b))),
        _ => Err(FusabiError::runtime_error("max requires two numbers")),
    }
}

fn pow(args: &[Value]) -> Result<Value> {
    let base = args[0].as_float()?;
    let exp = args[1].as_float()?;
    Ok(Value::Float(base.powf(exp)))
}

fn sqrt(args: &[Value]) -> Result<Value> {
    let n = args[0].as_float()?;
    Ok(Value::Float(n.sqrt()))
}

fn floor(args: &[Value]) -> Result<Value> {
    let n = args[0].as_float()?;
    Ok(Value::Int(n.floor() as i64))
}

fn ceil(args: &[Value]) -> Result<Value> {
    let n = args[0].as_float()?;
    Ok(Value::Int(n.ceil() as i64))
}

fn round(args: &[Value]) -> Result<Value> {
    let n = args[0].as_float()?;
    Ok(Value::Int(n.round() as i64))
}

// Conversion functions
fn to_int(args: &[Value]) -> Result<Value> {
    match &args[0] {
        Value::Int(i) => Ok(Value::Int(*i)),
        Value::Float(f) => Ok(Value::Int(*f as i64)),
        Value::String(s) => s
            .parse::<i64>()
            .map(Value::Int)
            .map_err(|_| FusabiError::runtime_error("Failed to parse int")),
        _ => Err(FusabiError::type_error("convertible to int", args[0].type_name())),
    }
}

fn to_float(args: &[Value]) -> Result<Value> {
    match &args[0] {
        Value::Int(i) => Ok(Value::Float(*i as f64)),
        Value::Float(f) => Ok(Value::Float(*f)),
        Value::String(s) => s
            .parse::<f64>()
            .map(Value::Float)
            .map_err(|_| FusabiError::runtime_error("Failed to parse float")),
        _ => Err(FusabiError::type_error("convertible to float", args[0].type_name())),
    }
}

fn to_string(args: &[Value]) -> Result<Value> {
    Ok(Value::String(args[0].to_string()))
}

// IO functions
fn print(args: &[Value]) -> Result<Value> {
    print!("{}", args[0]);
    Ok(Value::Nil)
}

fn println(args: &[Value]) -> Result<Value> {
    println!("{}", args[0]);
    Ok(Value::Nil)
}

// Type checking
fn is_nil(args: &[Value]) -> Result<Value> {
    Ok(Value::Bool(matches!(args[0], Value::Nil)))
}

fn is_bool(args: &[Value]) -> Result<Value> {
    Ok(Value::Bool(matches!(args[0], Value::Bool(_))))
}

fn is_int(args: &[Value]) -> Result<Value> {
    Ok(Value::Bool(matches!(args[0], Value::Int(_))))
}

fn is_float(args: &[Value]) -> Result<Value> {
    Ok(Value::Bool(matches!(args[0], Value::Float(_))))
}

fn is_string(args: &[Value]) -> Result<Value> {
    Ok(Value::Bool(matches!(args[0], Value::String(_))))
}

fn is_list(args: &[Value]) -> Result<Value> {
    Ok(Value::Bool(matches!(args[0], Value::List(_))))
}

fn is_map(args: &[Value]) -> Result<Value> {
    Ok(Value::Bool(matches!(args[0], Value::Map(_))))
}

fn is_function(args: &[Value]) -> Result<Value> {
    Ok(Value::Bool(matches!(args[0], Value::Function(_))))
}

// Functional programming (simplified - these would need closure support for full implementation)
fn map_fn(_args: &[Value]) -> Result<Value> {
    Err(FusabiError::runtime_error("map not yet implemented"))
}

fn filter_fn(_args: &[Value]) -> Result<Value> {
    Err(FusabiError::runtime_error("filter not yet implemented"))
}

fn fold_fn(_args: &[Value]) -> Result<Value> {
    Err(FusabiError::runtime_error("fold not yet implemented"))
}
