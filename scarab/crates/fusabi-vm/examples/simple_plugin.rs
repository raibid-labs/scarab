//! Example: Simple Fusabi Plugin
//!
//! Demonstrates creating and executing a simple .fzb plugin

use fusabi_vm::*;

fn main() {
    println!("=== Fusabi VM Example: Simple Plugin ===\n");

    // Example 1: Basic arithmetic
    println!("Example 1: Computing (10 + 5) * 2");
    {
        let (builder, c1) = BytecodeBuilder::new().add_constant(Value::I32(10));
        let (builder, c2) = builder.add_constant(Value::I32(5));
        let (builder, c3) = builder.add_constant(Value::I32(2));

        let mut code = Vec::new();
        code.extend(Opcode::Push(c1).encode());
        code.extend(Opcode::Push(c2).encode());
        code.extend(Opcode::Add.encode());
        code.extend(Opcode::Push(c3).encode());
        code.extend(Opcode::Mul.encode());
        code.extend(Opcode::Ret.encode());

        let func = Function {
            name: "arithmetic".to_string(),
            params: vec![],
            return_type: Type::I32,
            locals: 0,
            bytecode: code,
        };

        let (builder, func_idx) = builder.add_function(func);
        let bytecode = builder.entry_point(func_idx).build().unwrap();

        let bytes = bytecode.to_bytes().unwrap();
        let archived = FusabiBytecode::from_bytes(&bytes).unwrap();

        let mut vm = FusabiVM::new();
        let result = vm.execute(archived).unwrap();

        println!("Result: {}", result);
        println!("Instructions executed: {}\n", vm.stats().instructions_executed);
    }

    // Example 2: Using FFI functions
    println!("Example 2: String operations with FFI");
    {
        let (builder, c1) = BytecodeBuilder::new()
            .add_constant(Value::String("Hello, ".to_string()));
        let (builder, c2) = builder.add_constant(Value::String("Fusabi!".to_string()));
        let (builder, ffi_idx) = builder.add_ffi_import("string_concat");

        let mut code = Vec::new();
        code.extend(Opcode::Push(c1).encode());
        code.extend(Opcode::Push(c2).encode());
        code.extend(Opcode::CallFFI(ffi_idx).encode());
        code.extend(Opcode::Ret.encode());

        let func = Function {
            name: "concat".to_string(),
            params: vec![],
            return_type: Type::String,
            locals: 0,
            bytecode: code,
        };

        let (builder, func_idx) = builder.add_function(func);
        let bytecode = builder.entry_point(func_idx).build().unwrap();

        let bytes = bytecode.to_bytes().unwrap();
        let archived = FusabiBytecode::from_bytes(&bytes).unwrap();

        let mut vm = VmBuilder::new().with_stdlib(true).build();
        let result = vm.execute(archived).unwrap();

        println!("Result: {}", result);
        println!("FFI calls: {}\n", vm.stats().ffi_calls);
    }

    // Example 3: Conditional logic
    println!("Example 3: Conditional execution");
    {
        let (builder, c_val) = BytecodeBuilder::new().add_constant(Value::I32(5));
        let (builder, c_threshold) = builder.add_constant(Value::I32(10));
        let (builder, c_low) = builder.add_constant(Value::String("Low".to_string()));
        let (builder, c_high) = builder.add_constant(Value::String("High".to_string()));

        let mut code = Vec::new();
        code.extend(Opcode::Push(c_val).encode());
        code.extend(Opcode::Push(c_threshold).encode());
        code.extend(Opcode::Lt.encode()); // val < 10?

        // If true, jump to push "Low"
        code.extend(Opcode::JumpIfNot(5).encode());
        code.extend(Opcode::Push(c_low).encode());
        code.extend(Opcode::Jump(5).encode());

        // Else push "High"
        code.extend(Opcode::Push(c_high).encode());

        code.extend(Opcode::Ret.encode());

        let func = Function {
            name: "conditional".to_string(),
            params: vec![],
            return_type: Type::String,
            locals: 0,
            bytecode: code,
        };

        let (builder, func_idx) = builder.add_function(func);
        let bytecode = builder.entry_point(func_idx).build().unwrap();

        let bytes = bytecode.to_bytes().unwrap();
        let archived = FusabiBytecode::from_bytes(&bytes).unwrap();

        let mut vm = FusabiVM::new();
        let result = vm.execute(archived).unwrap();

        println!("Result: {}\n", result);
    }

    // Example 4: Local variables
    println!("Example 4: Using local variables");
    {
        let (builder, c1) = BytecodeBuilder::new().add_constant(Value::I32(10));
        let (builder, c2) = builder.add_constant(Value::I32(20));

        let mut code = Vec::new();
        // Store 10 in local 0
        code.extend(Opcode::Push(c1).encode());
        code.extend(Opcode::Store(0).encode());

        // Store 20 in local 1
        code.extend(Opcode::Push(c2).encode());
        code.extend(Opcode::Store(1).encode());

        // Load and add
        code.extend(Opcode::Load(0).encode());
        code.extend(Opcode::Load(1).encode());
        code.extend(Opcode::Add.encode());

        // Store result in local 2
        code.extend(Opcode::Store(2).encode());

        // Return local 2
        code.extend(Opcode::Load(2).encode());
        code.extend(Opcode::Ret.encode());

        let func = Function {
            name: "locals".to_string(),
            params: vec![],
            return_type: Type::I32,
            locals: 3,
            bytecode: code,
        };

        let (builder, func_idx) = builder.add_function(func);
        let bytecode = builder.entry_point(func_idx).build().unwrap();

        let bytes = bytecode.to_bytes().unwrap();
        let archived = FusabiBytecode::from_bytes(&bytes).unwrap();

        let mut vm = FusabiVM::new();
        let result = vm.execute(archived).unwrap();

        println!("Result: {}", result);
        println!("Max stack depth: {}\n", vm.stats().max_stack_depth);
    }

    // Example 5: Save bytecode to file
    println!("Example 5: Saving bytecode to file");
    {
        let (builder, const_idx) = BytecodeBuilder::new()
            .add_constant(Value::I32(42));

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
        let bytecode = builder.entry_point(func_idx).build().unwrap();

        let bytes = bytecode.to_bytes().unwrap();

        // In a real application, you would write to a .fzb file:
        // std::fs::write("plugin.fzb", &bytes).unwrap();

        println!("Bytecode size: {} bytes", bytes.len());
        println!("Functions: {}", bytecode.functions.len());
        println!("Constants: {}", bytecode.constants.len());
    }

    println!("\n=== All examples completed successfully! ===");
}
