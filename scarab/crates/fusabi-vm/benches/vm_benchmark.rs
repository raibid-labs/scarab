//! Performance benchmarks for Fusabi VM

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use fusabi_vm::*;

fn benchmark_opcode_execution(c: &mut Criterion) {
    let mut group = c.benchmark_group("opcodes");

    // Benchmark simple arithmetic
    group.bench_function("add_i32", |b| {
        let mut code = Vec::new();
        code.extend(Opcode::Push(0).encode());
        code.extend(Opcode::Push(1).encode());
        code.extend(Opcode::Add.encode());
        code.extend(Opcode::Ret.encode());

        let mut bytecode = FusabiBytecode::new(0);
        bytecode.constants = vec![Value::I32(10), Value::I32(5)];
        bytecode.functions.push(Function {
            name: "test".to_string(),
            params: vec![],
            return_type: Type::I32,
            locals: 0,
            bytecode: code,
        });

        let bytes = bytecode.to_bytes().unwrap();
        let archived = FusabiBytecode::from_bytes(&bytes).unwrap();

        b.iter(|| {
            let mut vm = FusabiVM::new();
            black_box(vm.execute(archived).unwrap());
        });
    });

    // Benchmark complex expression
    group.bench_function("complex_expression", |b| {
        // (10 + 5) * 2 - 3
        let mut code = Vec::new();
        code.extend(Opcode::Push(0).encode());
        code.extend(Opcode::Push(1).encode());
        code.extend(Opcode::Add.encode());
        code.extend(Opcode::Push(2).encode());
        code.extend(Opcode::Mul.encode());
        code.extend(Opcode::Push(3).encode());
        code.extend(Opcode::Sub.encode());
        code.extend(Opcode::Ret.encode());

        let mut bytecode = FusabiBytecode::new(0);
        bytecode.constants = vec![
            Value::I32(10),
            Value::I32(5),
            Value::I32(2),
            Value::I32(3),
        ];
        bytecode.functions.push(Function {
            name: "test".to_string(),
            params: vec![],
            return_type: Type::I32,
            locals: 0,
            bytecode: code,
        });

        let bytes = bytecode.to_bytes().unwrap();
        let archived = FusabiBytecode::from_bytes(&bytes).unwrap();

        b.iter(|| {
            let mut vm = FusabiVM::new();
            black_box(vm.execute(archived).unwrap());
        });
    });

    // Benchmark stack operations
    group.bench_function("stack_ops", |b| {
        let mut code = Vec::new();
        for _ in 0..100 {
            code.extend(Opcode::Push(0).encode());
            code.extend(Opcode::Pop.encode());
        }
        code.extend(Opcode::Push(0).encode());
        code.extend(Opcode::Ret.encode());

        let mut bytecode = FusabiBytecode::new(0);
        bytecode.constants = vec![Value::I32(42)];
        bytecode.functions.push(Function {
            name: "test".to_string(),
            params: vec![],
            return_type: Type::I32,
            locals: 0,
            bytecode: code,
        });

        let bytes = bytecode.to_bytes().unwrap();
        let archived = FusabiBytecode::from_bytes(&bytes).unwrap();

        b.iter(|| {
            let mut vm = FusabiVM::new();
            black_box(vm.execute(archived).unwrap());
        });
    });

    group.finish();
}

fn benchmark_bytecode_loading(c: &mut Criterion) {
    let mut group = c.benchmark_group("bytecode");

    // Create a moderately sized bytecode
    let mut bytecode = FusabiBytecode::new(0);
    for i in 0..1000 {
        bytecode.constants.push(Value::I32(i));
    }

    let mut code = Vec::new();
    code.extend(Opcode::Push(0).encode());
    code.extend(Opcode::Ret.encode());

    for i in 0..100 {
        bytecode.functions.push(Function {
            name: format!("func_{}", i),
            params: vec![],
            return_type: Type::I32,
            locals: 0,
            bytecode: code.clone(),
        });
    }

    let bytes = bytecode.to_bytes().unwrap();

    group.bench_function("deserialize", |b| {
        b.iter(|| {
            black_box(FusabiBytecode::from_bytes(&bytes).unwrap());
        });
    });

    group.finish();
}

fn benchmark_ffi_calls(c: &mut Criterion) {
    let mut group = c.benchmark_group("ffi");

    // Register FFI function
    let mut vm = FusabiVM::new();
    vm.register_ffi("add", |args| {
        let a = args[0].as_i32().unwrap();
        let b = args[1].as_i32().unwrap();
        Ok(Value::I32(a + b))
    });

    // Create bytecode that calls FFI
    let mut bytecode = FusabiBytecode::new(0);
    bytecode.constants = vec![Value::I32(10), Value::I32(5)];
    bytecode.ffi_imports = vec!["add".to_string()];

    let mut code = Vec::new();
    code.extend(Opcode::Push(0).encode());
    code.extend(Opcode::Push(1).encode());
    code.extend(Opcode::CallFFI(0).encode());
    code.extend(Opcode::Ret.encode());

    bytecode.functions.push(Function {
        name: "test".to_string(),
        params: vec![],
        return_type: Type::I32,
        locals: 0,
        bytecode: code,
    });

    let bytes = bytecode.to_bytes().unwrap();
    let archived = FusabiBytecode::from_bytes(&bytes).unwrap();

    group.bench_function("single_call", |b| {
        b.iter(|| {
            black_box(vm.execute(archived).unwrap());
        });
    });

    group.finish();
}

fn benchmark_throughput(c: &mut Criterion) {
    let mut group = c.benchmark_group("throughput");

    // Benchmark instruction throughput
    for size in [100, 1000, 10000].iter() {
        group.bench_with_input(
            BenchmarkId::new("instructions", size),
            size,
            |b, &size| {
                let mut code = Vec::new();
                for _ in 0..size {
                    code.extend(Opcode::Push(0).encode());
                    code.extend(Opcode::Push(1).encode());
                    code.extend(Opcode::Add.encode());
                    code.extend(Opcode::Pop.encode());
                }
                code.extend(Opcode::Push(0).encode());
                code.extend(Opcode::Ret.encode());

                let mut bytecode = FusabiBytecode::new(0);
                bytecode.constants = vec![Value::I32(1), Value::I32(2)];
                bytecode.functions.push(Function {
                    name: "test".to_string(),
                    params: vec![],
                    return_type: Type::I32,
                    locals: 0,
                    bytecode: code,
                });

                let bytes = bytecode.to_bytes().unwrap();
                let archived = FusabiBytecode::from_bytes(&bytes).unwrap();

                b.iter(|| {
                    let mut vm = FusabiVM::new();
                    black_box(vm.execute(archived).unwrap());
                });
            },
        );
    }

    group.finish();
}

fn benchmark_fibonacci(c: &mut Criterion) {
    let mut group = c.benchmark_group("fibonacci");

    // Iterative fibonacci
    let mut code = Vec::new();
    // locals: 0=n, 1=a, 2=b, 3=temp

    // a = 0
    code.extend(Opcode::Push(0).encode());
    code.extend(Opcode::Store(1).encode());

    // b = 1
    code.extend(Opcode::Push(1).encode());
    code.extend(Opcode::Store(2).encode());

    // n = 10
    code.extend(Opcode::Push(2).encode());
    code.extend(Opcode::Store(0).encode());

    // Loop start (pc = current position)
    let loop_start = code.len();

    // if n == 0: break
    code.extend(Opcode::Load(0).encode());
    code.extend(Opcode::Push(0).encode());
    code.extend(Opcode::Eq.encode());

    // Calculate jump offset to after loop (will be patched)
    let jump_patch_pos = code.len();
    code.extend(Opcode::JumpIf(0).encode()); // Will patch this

    // temp = a + b
    code.extend(Opcode::Load(1).encode());
    code.extend(Opcode::Load(2).encode());
    code.extend(Opcode::Add.encode());
    code.extend(Opcode::Store(3).encode());

    // a = b
    code.extend(Opcode::Load(2).encode());
    code.extend(Opcode::Store(1).encode());

    // b = temp
    code.extend(Opcode::Load(3).encode());
    code.extend(Opcode::Store(2).encode());

    // n = n - 1
    code.extend(Opcode::Load(0).encode());
    code.extend(Opcode::Push(1).encode());
    code.extend(Opcode::Sub.encode());
    code.extend(Opcode::Store(0).encode());

    // Jump back to loop start
    let loop_end = code.len();
    let back_jump = (loop_start as i32) - (loop_end as i32);
    code.extend(Opcode::Jump(back_jump).encode());

    // Patch forward jump
    let after_loop = code.len();
    let forward_jump = (after_loop as i32) - ((jump_patch_pos + 5) as i32);
    let jump_bytes = Opcode::JumpIf(forward_jump).encode();
    code[jump_patch_pos..jump_patch_pos + 5].copy_from_slice(&jump_bytes);

    // Return b
    code.extend(Opcode::Load(2).encode());
    code.extend(Opcode::Ret.encode());

    let mut bytecode = FusabiBytecode::new(0);
    bytecode.constants = vec![Value::I32(0), Value::I32(1), Value::I32(10)];
    bytecode.functions.push(Function {
        name: "fib".to_string(),
        params: vec![],
        return_type: Type::I32,
        locals: 4,
        bytecode: code,
    });

    let bytes = bytecode.to_bytes().unwrap();
    let archived = FusabiBytecode::from_bytes(&bytes).unwrap();

    group.bench_function("fib_10", |b| {
        b.iter(|| {
            let mut vm = FusabiVM::new();
            black_box(vm.execute(archived).unwrap());
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    benchmark_opcode_execution,
    benchmark_bytecode_loading,
    benchmark_ffi_calls,
    benchmark_throughput,
    benchmark_fibonacci,
);

criterion_main!(benches);
