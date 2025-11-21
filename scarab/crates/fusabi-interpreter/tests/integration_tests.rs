use fusabi_interpreter::*;

#[test]
fn test_basic_arithmetic() {
    assert_eq!(eval("2 + 3").unwrap(), Value::Int(5));
    assert_eq!(eval("10 - 4").unwrap(), Value::Int(6));
    assert_eq!(eval("3 * 4").unwrap(), Value::Int(12));
    assert_eq!(eval("15 / 3").unwrap(), Value::Int(5));
    assert_eq!(eval("17 % 5").unwrap(), Value::Int(2));
}

#[test]
fn test_float_arithmetic() {
    assert_eq!(eval("2.5 + 1.5").unwrap(), Value::Float(4.0));
    assert_eq!(eval("5.0 - 2.0").unwrap(), Value::Float(3.0));
    assert_eq!(eval("2.0 * 3.0").unwrap(), Value::Float(6.0));
    assert_eq!(eval("9.0 / 3.0").unwrap(), Value::Float(3.0));
}

#[test]
fn test_comparison() {
    assert_eq!(eval("5 > 3").unwrap(), Value::Bool(true));
    assert_eq!(eval("5 < 3").unwrap(), Value::Bool(false));
    assert_eq!(eval("5 >= 5").unwrap(), Value::Bool(true));
    assert_eq!(eval("5 <= 5").unwrap(), Value::Bool(true));
    assert_eq!(eval("5 == 5").unwrap(), Value::Bool(true));
    assert_eq!(eval("5 != 3").unwrap(), Value::Bool(true));
}

#[test]
fn test_logical_operators() {
    assert_eq!(eval("true && true").unwrap(), Value::Bool(true));
    assert_eq!(eval("true && false").unwrap(), Value::Bool(false));
    assert_eq!(eval("false || true").unwrap(), Value::Bool(true));
    assert_eq!(eval("false || false").unwrap(), Value::Bool(false));
    assert_eq!(eval("!true").unwrap(), Value::Bool(false));
    assert_eq!(eval("!false").unwrap(), Value::Bool(true));
}

#[test]
fn test_let_binding() {
    let code = r#"
        let x = 42
        x
    "#;
    assert_eq!(eval(code).unwrap(), Value::Int(42));

    let code = r#"
        let x = 10
        let y = 20
        x + y
    "#;
    assert_eq!(eval(code).unwrap(), Value::Int(30));
}

#[test]
fn test_let_expression() {
    let code = "let x = 42 in x + 1";
    assert_eq!(eval(code).unwrap(), Value::Int(43));

    let code = "let x = 10 in let y = 20 in x + y";
    assert_eq!(eval(code).unwrap(), Value::Int(30));
}

#[test]
fn test_if_expression() {
    assert_eq!(eval("if true then 1 else 2").unwrap(), Value::Int(1));
    assert_eq!(eval("if false then 1 else 2").unwrap(), Value::Int(2));
    assert_eq!(eval("if 5 > 3 then 100 else 200").unwrap(), Value::Int(100));
}

#[test]
fn test_function_definition() {
    let code = r#"
        let add x y = x + y
        add 10 20
    "#;
    assert_eq!(eval(code).unwrap(), Value::Int(30));

    let code = r#"
        let double x = x * 2
        double 21
    "#;
    assert_eq!(eval(code).unwrap(), Value::Int(42));
}

#[test]
fn test_lambda() {
    let code = "(fun x -> x + 1) 41";
    assert_eq!(eval(code).unwrap(), Value::Int(42));

    let code = "(fun x y -> x * y) 6 7";
    assert_eq!(eval(code).unwrap(), Value::Int(42));
}

#[test]
fn test_higher_order_functions() {
    let code = r#"
        let apply f x = f x
        let inc x = x + 1
        apply inc 41
    "#;
    assert_eq!(eval(code).unwrap(), Value::Int(42));
}

#[test]
fn test_list_literal() {
    let code = "[1, 2, 3]";
    if let Value::List(list) = eval(code).unwrap() {
        assert_eq!(list.len(), 3);
        assert_eq!(list[0], Value::Int(1));
        assert_eq!(list[1], Value::Int(2));
        assert_eq!(list[2], Value::Int(3));
    } else {
        panic!("Expected list");
    }
}

#[test]
fn test_list_indexing() {
    let code = "[1, 2, 3][0]";
    assert_eq!(eval(code).unwrap(), Value::Int(1));

    let code = "[10, 20, 30][2]";
    assert_eq!(eval(code).unwrap(), Value::Int(30));
}

#[test]
fn test_map_literal() {
    let code = "{ x = 1; y = 2 }";
    if let Value::Map(map) = eval(code).unwrap() {
        assert_eq!(map.len(), 2);
        assert_eq!(map.get("x"), Some(&Value::Int(1)));
        assert_eq!(map.get("y"), Some(&Value::Int(2)));
    } else {
        panic!("Expected map");
    }
}

#[test]
fn test_map_field_access() {
    let code = "{ x = 42 }.x";
    assert_eq!(eval(code).unwrap(), Value::Int(42));
}

#[test]
fn test_string_operations() {
    assert_eq!(
        eval(r#""hello" + " " + "world""#).unwrap(),
        Value::String("hello world".to_string())
    );
}

#[test]
fn test_stdlib_string_functions() {
    assert_eq!(eval(r#"strlen "hello""#).unwrap(), Value::Int(5));
    assert_eq!(
        eval(r#"to_upper "hello""#).unwrap(),
        Value::String("HELLO".to_string())
    );
    assert_eq!(
        eval(r#"to_lower "WORLD""#).unwrap(),
        Value::String("world".to_string())
    );
}

#[test]
fn test_stdlib_list_functions() {
    assert_eq!(eval("length [1, 2, 3]").unwrap(), Value::Int(3));
    assert_eq!(eval("head [1, 2, 3]").unwrap(), Value::Int(1));

    let code = "reverse [1, 2, 3]";
    if let Value::List(list) = eval(code).unwrap() {
        assert_eq!(list, vec![Value::Int(3), Value::Int(2), Value::Int(1)]);
    } else {
        panic!("Expected list");
    }
}

#[test]
fn test_stdlib_math_functions() {
    assert_eq!(eval("abs -42").unwrap(), Value::Int(42));
    assert_eq!(eval("min 5 10").unwrap(), Value::Int(5));
    assert_eq!(eval("max 5 10").unwrap(), Value::Int(10));
    assert_eq!(eval("floor 3.7").unwrap(), Value::Int(3));
    assert_eq!(eval("ceil 3.2").unwrap(), Value::Int(4));
}

#[test]
fn test_type_checking_functions() {
    assert_eq!(eval("is_int 42").unwrap(), Value::Bool(true));
    assert_eq!(eval("is_int 3.14").unwrap(), Value::Bool(false));
    assert_eq!(eval(r#"is_string "hello""#).unwrap(), Value::Bool(true));
    assert_eq!(eval("is_list [1, 2, 3]").unwrap(), Value::Bool(true));
}

#[test]
fn test_recursion() {
    let code = r#"
        let factorial n =
            if n <= 1 then
                1
            else
                n * factorial (n - 1)
        factorial 5
    "#;
    assert_eq!(eval(code).unwrap(), Value::Int(120));
}

#[test]
fn test_nested_functions() {
    let code = r#"
        let outer x =
            let inner y = x + y
            inner 10
        outer 5
    "#;
    assert_eq!(eval(code).unwrap(), Value::Int(15));
}

#[test]
fn test_comments() {
    let code = r#"
        // This is a comment
        let x = 42 // Another comment
        x
    "#;
    assert_eq!(eval(code).unwrap(), Value::Int(42));
}

#[test]
fn test_complex_expression() {
    let code = r#"
        let square x = x * x
        let sum_of_squares x y = square x + square y
        sum_of_squares 3 4
    "#;
    assert_eq!(eval(code).unwrap(), Value::Int(25));
}

#[test]
fn test_error_division_by_zero() {
    let result = eval("10 / 0");
    assert!(result.is_err());
}

#[test]
fn test_error_undefined_variable() {
    let result = eval("undefined_var");
    assert!(result.is_err());
}

#[test]
fn test_error_type_mismatch() {
    let result = eval(r#"42 + "string""#);
    assert!(result.is_err());
}

#[test]
fn test_error_arity_mismatch() {
    let code = r#"
        let add x y = x + y
        add 10
    "#;
    let result = eval(code);
    assert!(result.is_err());
}
