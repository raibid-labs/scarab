# Issue #4: Fusabi VM (AOT Runtime)

**Phase**: 2A - Plugin System
**Priority**: 🟢 Medium
**Workstream**: Compiler/VM
**Estimated Effort**: 2-3 weeks
**Assignee**: Compiler/VM Specialist Agent

---

## 🎯 Objective

Implement a high-performance bytecode VM for executing compiled Fusabi plugins (.fzb) in the daemon with <1ms overhead.

---

## 📋 Background

The daemon needs to run performance-critical plugins (output scanning, triggers, mux logic) without the overhead of an interpreter. We'll build a stack-based VM with:
- Zero-copy bytecode loading via rkyv
- FFI bridge to Rust functions
- Security sandbox (memory limits, no syscalls)
- Register optimization for hot paths

---

## ✅ Acceptance Criteria

- [ ] Bytecode format definition (.fzb schema)
- [ ] Stack-based VM with opcodes (PUSH, POP, CALL, RET, etc.)
- [ ] rkyv serialization for instant loading
- [ ] FFI bridge to Rust (call daemon functions)
- [ ] Memory allocator with limits (1GB max)
- [ ] No unsafe syscalls (sandbox enforcement)
- [ ] JIT compilation (optional, for hot loops)
- [ ] Plugin can access SharedState safely
- [ ] Benchmark: <1ms overhead vs native Rust
- [ ] Unit tests for all opcodes

---

## 🔧 Technical Approach

### Step 1: Bytecode Format
```rust
#[derive(Archive, Deserialize, Serialize)]
pub struct FusabiBytecode {
    magic: [u8; 4],  // b"FZB\0"
    version: u32,
    constants: Vec<Value>,
    functions: Vec<Function>,
    entry_point: u32,
}

#[derive(Archive, Deserialize, Serialize)]
pub struct Function {
    name: String,
    params: Vec<Type>,
    return_type: Type,
    locals: u32,
    bytecode: Vec<u8>,
}

pub enum Opcode {
    Push(u32),     // Push constant
    Pop,           // Pop stack
    Load(u32),     // Load local
    Store(u32),    // Store local
    Call(u32),     // Call function
    CallFFI(u32),  // Call Rust FFI
    Ret,           // Return
    Add, Sub, Mul, Div,
    Eq, Lt, Gt,
    Jump(i32),     // Unconditional
    JumpIf(i32),   // Conditional
}
```

### Step 2: VM Implementation
```rust
pub struct FusabiVM {
    stack: Vec<Value>,
    call_stack: Vec<Frame>,
    memory: Vec<u8>,  // Heap
    ffi_registry: HashMap<String, FfiFn>,
}

impl FusabiVM {
    pub fn execute(&mut self, bytecode: &FusabiBytecode) -> Result<Value> {
        let entry = &bytecode.functions[bytecode.entry_point as usize];

        for op in decode_opcodes(&entry.bytecode) {
            match op {
                Opcode::Push(idx) => self.stack.push(bytecode.constants[idx]),
                Opcode::CallFFI(idx) => {
                    let func = self.ffi_registry.get(idx)?;
                    let result = func(&self.stack)?;
                    self.stack.push(result);
                }
                // ... handle other opcodes
            }
        }

        Ok(self.stack.pop().unwrap())
    }
}
```

### Step 3: FFI Bridge
```rust
type FfiFn = fn(&[Value]) -> Result<Value>;

impl FusabiVM {
    pub fn register_ffi(&mut self, name: &str, func: FfiFn) {
        self.ffi_registry.insert(name.to_string(), func);
    }
}

// Example FFI function
fn ffi_update_cell(args: &[Value]) -> Result<Value> {
    let x = args[0].as_i32()?;
    let y = args[1].as_i32()?;
    let ch = args[2].as_char()?;

    unsafe {
        let state = &mut *SHARED_STATE_PTR;
        let idx = y * GRID_WIDTH + x;
        state.cells[idx].char_codepoint = ch as u32;
    }

    Ok(Value::Unit)
}
```

### Step 4: Security Sandbox
```rust
impl FusabiVM {
    fn check_memory_access(&self, ptr: usize, len: usize) -> Result<()> {
        if ptr + len > self.memory.len() {
            return Err(Error::MemoryOutOfBounds);
        }
        Ok(())
    }

    fn check_call_depth(&self) -> Result<()> {
        if self.call_stack.len() > MAX_CALL_DEPTH {
            return Err(Error::StackOverflow);
        }
        Ok(())
    }
}
```

---

## 📦 Deliverables

1. **Code**: `crates/fusabi-vm/src/` implementation
2. **Spec**: Bytecode format documentation
3. **Tests**: VM test suite with 100+ test cases
4. **Examples**: Sample .fzb plugins
5. **Benchmark**: Performance comparison vs Rust

---

## 🔗 Dependencies

- **Depends On**: Issue #1 (VTE Parser) - for integration testing
- **Blocks**: Issue #6 (Plugin API) - VM needed for plugin loading

---

## 📚 Resources

- [WebAssembly Spec](https://webassembly.github.io/spec/core/)
- [rkyv Zero-Copy](https://rkyv.org/architecture.html)
- [Cranelift JIT](https://github.com/bytecodealliance/wasmtime/tree/main/cranelift)
- [VM Design Patterns](https://craftinginterpreters.com/)

---

## 🎯 Success Metrics

- ✅ <1ms plugin execution overhead
- ✅ 1,000,000+ ops/sec throughput
- ✅ Zero-copy bytecode loading
- ✅ Memory safety (no crashes)
- ✅ <10MB VM overhead

---

## 💡 Implementation Notes

### Performance Optimization
- Use computed goto for opcode dispatch (if available)
- Inline hot FFI functions
- JIT compile hot loops with Cranelift
- Profile with `perf` and optimize bottlenecks

### Bytecode Compiler
- Phase 1: Manual assembly (for testing)
- Phase 2: F# → Bytecode compiler (separate tool)
- Use LLVM IR as intermediate step (optional)

### Security Model
- Capability-based access (plugins request permissions)
- No direct file I/O (must use FFI)
- No network access
- Time/memory limits per execution

---

## 🐛 Known Issues

- JIT compilation adds complexity (defer to Phase 3)
- FFI overhead for frequent calls (inline hot paths)
- Debugging bytecode is difficult (add debug info)

---

## 🔍 Example Plugin

```fsharp
// plugin.fsx (will be compiled to .fzb)
let onOutput (line: string) =
    if line.Contains("ERROR") then
        notify "Error detected in terminal"
        setColor Red

let onKeyPress (key: Key) =
    if key = Space && isLeaderKey() then
        showCommandPalette()
```

Compiled bytecode:
```
00: PUSH 0           ; Load "ERROR" constant
04: LOAD 0           ; Load `line` parameter
08: CALL_FFI 0       ; String.Contains
12: JUMP_IF 24       ; Skip if false
16: PUSH 1           ; Load "Error detected" constant
20: CALL_FFI 1       ; notify()
...
```

---

**Created**: 2025-11-21
**Labels**: `phase-2`, `medium-priority`, `vm`, `compiler`, `performance`
