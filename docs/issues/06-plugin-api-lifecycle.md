# Issue #6: Plugin API & Lifecycle Management

**Phase**: 2C - Plugin System
**Priority**: 🟢 Medium
**Workstream**: API Design
**Estimated Effort**: 1-2 weeks
**Assignee**: API Design Specialist Agent

---

## 🎯 Objective

Design and implement the plugin API, lifecycle management, and discovery system to enable 3rd-party plugin development.

---

## 📋 Background

With the VM and Interpreter in place, we need:
- Trait-based plugin API
- Hook system for plugins to intercept events
- Configuration loading (TOML/YAML)
- Plugin discovery and loading
- Version compatibility checking

---

## ✅ Acceptance Criteria

- [ ] Plugin trait definitions
- [ ] Hook system (pre-output, post-input, etc.)
- [ ] Configuration loading from TOML
- [ ] Plugin discovery in ~/.config/scarab/plugins
- [ ] Version compatibility checks
- [ ] Plugin isolation (errors don't crash daemon)
- [ ] Hot-reload support
- [ ] Example plugin template
- [ ] Plugin development guide
- [ ] Plugin metadata (author, version, description)

---

## 🔧 Technical Approach

### Step 1: Plugin Trait
```rust
// crates/scarab-plugin-api/src/lib.rs

pub trait Plugin: Send + Sync {
    fn name(&self) -> &str;
    fn version(&self) -> &str;

    fn on_load(&mut self, ctx: &mut PluginContext) -> Result<()> {
        Ok(())
    }

    fn on_unload(&mut self) -> Result<()> {
        Ok(())
    }

    // Hooks
    fn on_output(&mut self, line: &str) -> Result<Action> {
        Ok(Action::Continue)
    }

    fn on_input(&mut self, input: &[u8]) -> Result<Action> {
        Ok(Action::Continue)
    }

    fn on_resize(&mut self, cols: u16, rows: u16) -> Result<()> {
        Ok(())
    }
}

pub enum Action {
    Continue,      // Pass to next plugin
    Stop,          // Stop processing
    Modify(Vec<u8>), // Modify data
}
```

### Step 2: Plugin Context
```rust
pub struct PluginContext {
    pub config: Config,
    pub state: Arc<Mutex<SharedState>>,
    pub logger: Logger,
}

impl PluginContext {
    pub fn get_cell(&self, x: u16, y: u16) -> Cell {
        let state = self.state.lock().unwrap();
        state.cells[(y * GRID_WIDTH + x) as usize]
    }

    pub fn set_cell(&mut self, x: u16, y: u16, cell: Cell) {
        let mut state = self.state.lock().unwrap();
        state.cells[(y * GRID_WIDTH + x) as usize] = cell;
    }

    pub fn notify(&self, msg: &str) {
        // Send notification to client
    }
}
```

### Step 3: Plugin Manager
```rust
pub struct PluginManager {
    plugins: Vec<Box<dyn Plugin>>,
    vm: FusabiVM,
}

impl PluginManager {
    pub fn load_plugin(&mut self, path: &Path) -> Result<()> {
        let ext = path.extension().unwrap();

        let plugin: Box<dyn Plugin> = match ext.to_str() {
            Some("fzb") => {
                // Load compiled bytecode
                let bytecode = fs::read(path)?;
                Box::new(CompiledPlugin::new(&mut self.vm, bytecode)?)
            }
            Some("fsx") => {
                // Load interpreted script
                Box::new(ScriptPlugin::new(path)?)
            }
            _ => return Err(Error::UnsupportedFormat),
        };

        self.plugins.push(plugin);
        Ok(())
    }

    pub fn dispatch_output(&mut self, line: &str) -> Result<String> {
        let mut data = line.to_string();

        for plugin in &mut self.plugins {
            match plugin.on_output(&data)? {
                Action::Continue => {}
                Action::Stop => break,
                Action::Modify(new_data) => {
                    data = String::from_utf8(new_data)?;
                }
            }
        }

        Ok(data)
    }
}
```

### Step 4: Configuration
```toml
# ~/.config/scarab/plugins.toml

[[plugin]]
name = "auto-notify"
path = "~/.config/scarab/plugins/auto-notify.fzb"
enabled = true

[plugin.config]
keywords = ["ERROR", "FAIL", "PANIC"]
notification_style = "urgent"

[[plugin]]
name = "vim-mode"
path = "~/.config/scarab/plugins/vim-mode.fsx"
enabled = true
```

---

## 📦 Deliverables

1. **Code**: `crates/scarab-plugin-api/src/` API definitions
2. **Manager**: `scarab-daemon/src/plugin_manager.rs`
3. **Template**: Example plugin template repository
4. **Guide**: Plugin development guide
5. **Examples**: 3+ working example plugins

---

## 🔗 Dependencies

- **Depends On**: Issue #4 (VM) and Issue #5 (Interpreter)
- **Blocks**: Issue #8 (Advanced UI) - plugins needed for features

---

## 📚 Resources

- [Bevy Plugin System](https://bevyengine.org/learn/book/getting-started/plugins/)
- [Vim Plugin API](https://vimhelp.org/usr_41.txt.html)
- [VS Code Extension API](https://code.visualstudio.com/api)
- [Dynamic Loading in Rust](https://michael-f-bryan.github.io/rust-ffi-guide/dynamic_loading.html)

---

## 🎯 Success Metrics

- ✅ Plugin load time <10ms
- ✅ Hook overhead <1% CPU
- ✅ Support 50+ plugins simultaneously
- ✅ Zero crashes from plugin errors
- ✅ Clear error messages for plugin failures

---

## 💡 Implementation Notes

### Plugin Isolation
- Catch panics in plugin hooks
- Timeout plugin execution (1 second max)
- Memory limits per plugin
- Automatic disable on repeated failures

### Hook Types
1. **Pre-Output**: Modify text before display
2. **Post-Input**: Intercept key presses
3. **Pre-Command**: Before PTY command
4. **Post-Command**: After command completes
5. **On-Resize**: Terminal size changes
6. **On-Attach**: Client connects
7. **On-Detach**: Client disconnects

### Versioning
```toml
[plugin]
name = "my-plugin"
version = "1.0.0"
api_version = "0.1.0"  # Scarab API version
min_scarab_version = "0.1.0"
```

---

## 🐛 Known Issues

- Plugin API may need breaking changes in early versions
- Hot-reload may require careful state management
- Plugin dependencies/conflicts not handled yet

---

## 🔌 Example Plugin

```fsharp
// auto-notify.fsx - Notify on error keywords

module AutoNotify

open Scarab.PluginApi

let keywords = ["ERROR", "FAIL", "PANIC"]

let on_output (line: string) (ctx: PluginContext) =
    let hasKeyword = keywords |> List.exists (fun k -> line.Contains(k))

    if hasKeyword then
        ctx.notify(sprintf "Found keyword in output: %s" line)
        ctx.setColor(Red)

    Action.Continue

let plugin = {
    Name = "auto-notify"
    Version = "1.0.0"
    OnOutput = on_output
}
```

---

**Created**: 2025-11-21
**Labels**: `phase-2`, `medium-priority`, `plugin-api`, `extensibility`
