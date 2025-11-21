# Issue #5: Fusabi Interpreter (Script Runtime)

**Phase**: 2B - Plugin System
**Priority**: 🟢 Medium
**Workstream**: Language/Interpreter
**Estimated Effort**: 2-3 weeks
**Assignee**: Language/Interpreter Specialist Agent

---

## 🎯 Objective

Implement an AST-based interpreter for hot-reloadable Fusabi scripts (.fsx) in the client for UI customization and overlays.

---

## 📋 Background

The client needs to run UI scripts for:
- Custom layouts and themes
- Vimium-style overlays
- Spacemacs-like menus
- Animations and transitions

Scripts must hot-reload in <100ms without Rust recompilation.

---

## ✅ Acceptance Criteria

- [ ] F# dialect parser (.fsx syntax)
- [ ] AST walker with interpreter
- [ ] Bevy UI integration (spawn entities, update components)
- [ ] File watcher for auto-reload
- [ ] Error reporting with line/column numbers
- [ ] Standard library (String, List, UI primitives)
- [ ] Hot-reload in <100ms
- [ ] Can create interactive UI overlays
- [ ] Can define key bindings
- [ ] Unit tests for language features

---

## 🔧 Technical Approach

### Step 1: Parser (nom-based)
```rust
use nom::{IResult, branch::alt, bytes::complete::tag};

#[derive(Debug)]
pub enum Expr {
    Literal(Value),
    Variable(String),
    Lambda(Vec<String>, Box<Expr>),
    Apply(Box<Expr>, Vec<Expr>),
    Let(String, Box<Expr>, Box<Expr>),
    If(Box<Expr>, Box<Expr>, Box<Expr>),
}

fn parse_expr(input: &str) -> IResult<&str, Expr> {
    alt((
        parse_literal,
        parse_variable,
        parse_lambda,
        parse_let,
        parse_if,
    ))(input)
}
```

### Step 2: Interpreter
```rust
pub struct Interpreter {
    env: Environment,
    bevy_world: &mut World,
}

impl Interpreter {
    pub fn eval(&mut self, expr: &Expr) -> Result<Value> {
        match expr {
            Expr::Literal(v) => Ok(v.clone()),
            Expr::Variable(name) => self.env.get(name),
            Expr::Lambda(params, body) => {
                Ok(Value::Closure(params.clone(), body.clone(), self.env.clone()))
            }
            Expr::Apply(func, args) => {
                let f = self.eval(func)?;
                let arg_vals: Vec<_> = args.iter().map(|a| self.eval(a)).collect()?;
                self.apply(f, arg_vals)
            }
            Expr::Let(name, val, body) => {
                let v = self.eval(val)?;
                self.env.insert(name.clone(), v);
                self.eval(body)
            }
            Expr::If(cond, then, else_) => {
                if self.eval(cond)?.as_bool()? {
                    self.eval(then)
                } else {
                    self.eval(else_)
                }
            }
        }
    }
}
```

### Step 3: Bevy Integration
```rust
// Standard library function for creating UI
fn ui_button(args: &[Value], world: &mut World) -> Result<Value> {
    let text = args[0].as_string()?;
    let callback = args[1].as_closure()?;

    let entity = world.spawn((
        ButtonBundle {
            style: Style {
                width: Val::Px(150.0),
                height: Val::Px(50.0),
                ..default()
            },
            ..default()
        },
        FusabiCallback(callback),
    ));

    Ok(Value::Entity(entity.id()))
}
```

### Step 4: Hot Reload
```rust
use notify::{Watcher, RecursiveMode};

pub fn watch_scripts(tx: mpsc::Sender<PathBuf>) {
    let mut watcher = notify::watcher(tx, Duration::from_secs(1)).unwrap();
    watcher.watch("~/.config/scarab/scripts", RecursiveMode::Recursive).unwrap();
}

fn reload_system(
    mut events: EventReader<ScriptReloadEvent>,
    mut interpreter: ResMut<Interpreter>,
) {
    for event in events.read() {
        match interpreter.reload_script(&event.path) {
            Ok(_) => info!("Reloaded script: {:?}", event.path),
            Err(e) => error!("Failed to reload: {}", e),
        }
    }
}
```

---

## 📦 Deliverables

1. **Code**: `crates/fusabi-interpreter/src/` implementation
2. **Grammar**: Fusabi language spec (EBNF)
3. **Tests**: Language feature test suite
4. **Examples**: Sample .fsx scripts
5. **Documentation**: Language guide and API reference

---

## 🔗 Dependencies

- **Depends On**: Issue #2 (Rendering) - for UI integration
- **Blocks**: Issue #8 (Advanced UI) - interpreter needed for UI scripts

---

## 📚 Resources

- [nom Parser Combinator](https://docs.rs/nom/)
- [F# Language Spec](https://fsharp.org/specs/language-spec/)
- [notify File Watcher](https://docs.rs/notify/)
- [Bevy UI Examples](https://bevyengine.org/examples/UI%20(User%20Interface)/)

---

## 🎯 Success Metrics

- ✅ Hot-reload in <100ms
- ✅ Parse 1,000 LOC in <10ms
- ✅ No Rust recompilation needed
- ✅ Useful error messages with context
- ✅ Standard library with 50+ functions

---

## 💡 Implementation Notes

### Language Subset
Focus on essential features:
- **Literals**: int, float, string, bool
- **Functions**: let-bound and lambda
- **Collections**: list, map
- **Control Flow**: if/else, match
- **Modules**: import system

Defer to Phase 3:
- Type inference (use dynamic types initially)
- Pattern matching (partial support)
- Algebraic data types

### Performance
- Cache parsed AST (only parse on change)
- Optimize common patterns (tail-call optimization)
- JIT compile hot paths (optional)

### Error Handling
```rust
pub enum Error {
    ParseError { line: u32, col: u32, msg: String },
    TypeError { expected: Type, got: Type },
    RuntimeError(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::ParseError { line, col, msg } =>
                write!(f, "Parse error at {}:{}: {}", line, col, msg),
            // ...
        }
    }
}
```

---

## 🐛 Known Issues

- F# syntax is complex (start with subset)
- Bevy World access from scripts requires careful design
- Hot-reload may lose script state (need persistence)

---

## 🎨 Example Script

```fsharp
// ~/.config/scarab/ui.fsx

// Define a command palette
let commandPalette = ui.createOverlay {
    Position = Centered
    Width = 600
    Height = 400
}

let commands = [
    { Name = "Split Vertical"; Key = "v"; Action = split_vertical }
    { Name = "New Tab"; Key = "t"; Action = new_tab }
    { Name = "Search"; Key = "/"; Action = search }
]

// Render command list
for cmd in commands do
    ui.button cmd.Name (fun () ->
        cmd.Action()
        ui.close commandPalette
    )

// Bind to leader key
keymap.bind "<Space>" (fun () ->
    ui.show commandPalette
)
```

---

**Created**: 2025-11-21
**Labels**: `phase-2`, `medium-priority`, `interpreter`, `language`, `hot-reload`
