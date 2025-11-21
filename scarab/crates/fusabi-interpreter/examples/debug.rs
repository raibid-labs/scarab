use fusabi_interpreter::*;

fn main() {
    // Test let expression
    let code1 = "let x = 42 in x + 1";
    println!("Code: {}", code1);
    match eval(code1) {
        Ok(result) => println!("Result: {} (expected: 43)", result),
        Err(e) => println!("Error: {}", e),
    }

    // Test nested let
    let code2 = "let x = 10 in let y = 20 in x + y";
    println!("\nCode: {}", code2);
    match eval(code2) {
        Ok(result) => println!("Result: {} (expected: 30)", result),
        Err(e) => println!("Error: {}", e),
    }

    // Test nested function
    let code3 = r#"
        let outer x =
            let inner y = x + y
            inner(10)
        outer(5)
    "#;
    println!("\nCode: {}", code3);
    match eval(code3) {
        Ok(result) => println!("Result: {} (expected: 15)", result),
        Err(e) => println!("Error: {}", e),
    }

    // Simple test
    let code4 = "let x = 42 in x";
    println!("\nCode: {}", code4);
    match eval(code4) {
        Ok(result) => println!("Result: {} (expected: 42)", result),
        Err(e) => println!("Error: {}", e),
    }
}
