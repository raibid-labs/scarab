# Issue #13: End-to-End Integration & Final Assembly

**Phase**: Integration
**Priority**: 🔴 Critical
**Workstream**: Integration Engineering
**Estimated Effort**: 1 week
**Assignee**: Integration Specialist Agent

---

## 🎯 Objective

Wire all components together into a working end-to-end system, resolve integration issues, and verify complete workflows from daemon to client rendering.

---

## 📋 Background

All components are individually complete:
- Phase 1: VTE, Rendering, IPC ✅
- Phase 2: VM, Interpreter, Plugins ✅
- Phase 3: Sessions, UI, Config ✅

Now we need to:
- Connect VTE output → SharedState → Rendering pipeline
- Wire UI features to terminal state
- Integrate plugins with daemon/client
- Fix API mismatches (Bevy 0.15)
- End-to-end testing

---

## ✅ Acceptance Criteria

- [ ] VTE parser updates SharedState cells
- [ ] Client renders SharedState correctly
- [ ] IPC forwards input/resize properly
- [ ] Plugins can hook terminal events
- [ ] Sessions persist across reconnects
- [ ] UI overlays work with real terminal
- [ ] Config hot-reload affects all components
- [ ] All Bevy 0.15 APIs updated
- [ ] E2E test: vim editing session
- [ ] E2E test: htop rendering
- [ ] E2E test: plugin execution
- [ ] Zero crashes in 1-hour stress test

---

## 🔧 Technical Approach

### Step 1: VTE → SharedState Integration
```rust
// scarab-daemon/src/main.rs

let mut terminal_state = vte::TerminalState::new(shared_ptr, sequence_counter.clone());

loop {
    tokio::select! {
        Ok((n, buf)) = read_pty => {
            // Parse VTE and update SharedState
            terminal_state.process_output(&buf[..n]);

            // Verify cells were updated
            unsafe {
                let state = &*shared_ptr;
                assert!(state.sequence_number > 0);
            }
        }
    }
}
```

### Step 2: SharedState → Rendering Integration
```rust
// scarab-client/src/main.rs

fn sync_grid(
    reader: ResMut<SharedMemoryReader>,
    mut renderer: ResMut<TextRenderer>,
    mut mesh_query: Query<&mut Mesh, With<TerminalGrid>>,
) {
    let shared_ptr = reader.shmem.0.as_ptr() as *const SharedState;

    unsafe {
        let state = &*shared_ptr;

        if state.sequence_number != reader.last_sequence {
            // Generate mesh from cells
            let mesh = renderer.generate_mesh(&state.cells, GRID_WIDTH, GRID_HEIGHT);

            // Update Bevy mesh
            for mut m in mesh_query.iter_mut() {
                *m = mesh.clone();
            }

            reader.last_sequence = state.sequence_number;
        }
    }
}
```

### Step 3: UI → Terminal Integration
```rust
// scarab-client/src/ui/link_hints.rs

fn detect_links(state: &SharedState) -> Vec<LinkHint> {
    let mut links = Vec::new();
    let text = extract_text_from_grid(&state.cells);

    // Regex for URLs
    let url_regex = Regex::new(r"https?://[^\s]+").unwrap();

    for mat in url_regex.find_iter(&text) {
        let (row, col) = position_to_coords(mat.start(), GRID_WIDTH);
        links.push(LinkHint {
            url: mat.as_str().to_string(),
            position: Vec2::new(col as f32, row as f32),
            hint_key: generate_hint_key(links.len()),
        });
    }

    links
}
```

### Step 4: Bevy 0.15 API Updates
```rust
// Update deprecated APIs

// OLD (Bevy 0.14)
commands.spawn(TextBundle {
    text: Text::from_section("Hello", style),
    ..default()
});

// NEW (Bevy 0.15)
commands.spawn((
    Text::from_section("Hello", style),
    Node::default(),
));

// OLD color
Color::rgba(1.0, 0.0, 0.0, 1.0)

// NEW color
Color::srgba(1.0, 0.0, 0.0, 1.0)
```

### Step 5: Plugin Integration
```rust
// scarab-daemon/src/main.rs

let mut plugin_manager = PluginManager::new()?;

// Load plugins from config
for plugin_config in config.plugins.enabled {
    plugin_manager.load_plugin(&plugin_config.path)?;
}

// Hook into PTY output
loop {
    let output = read_pty()?;

    // Let plugins process output first
    let modified_output = plugin_manager.dispatch_output(&output)?;

    // Then parse VTE
    terminal_state.process_output(&modified_output);
}
```

### Step 6: E2E Test Framework
```rust
// tests/e2e/framework.rs

pub struct TestSession {
    daemon: Child,
    client: Client,
    shared_mem: Shmem,
}

impl TestSession {
    pub async fn new() -> Result<Self> {
        // Start daemon
        let daemon = Command::new("target/release/scarab-daemon")
            .spawn()?;

        tokio::time::sleep(Duration::from_millis(100)).await;

        // Connect client
        let client = Client::connect().await?;

        // Open shared memory
        let shared_mem = ShmemConf::new()
            .os_id(SHMEM_PATH)
            .open()?;

        Ok(Self { daemon, client, shared_mem })
    }

    pub async fn send_keys(&self, keys: &str) -> Result<()> {
        self.client.send_input(keys.as_bytes()).await
    }

    pub fn get_grid_text(&self) -> String {
        let ptr = self.shared_mem.as_ptr() as *const SharedState;
        unsafe {
            extract_text_from_grid(&(*ptr).cells)
        }
    }

    pub async fn wait_for_text(&self, expected: &str) -> Result<()> {
        let start = Instant::now();
        loop {
            let text = self.get_grid_text();
            if text.contains(expected) {
                return Ok(());
            }
            if start.elapsed() > Duration::from_secs(5) {
                bail!("Timeout waiting for: {}", expected);
            }
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
    }
}
```

### Step 7: E2E Tests
```rust
// tests/e2e/terminal_workflows.rs

#[tokio::test]
async fn test_vim_editing_workflow() {
    let session = TestSession::new().await?;

    // Start vim
    session.send_keys("vim test.txt\n").await?;
    session.wait_for_text("test.txt").await?;

    // Enter insert mode and type
    session.send_keys("i").await?;
    session.send_keys("Hello, Scarab!").await?;

    // Save and quit
    session.send_keys("\x1b:wq\n").await?;

    // Verify grid cleared
    tokio::time::sleep(Duration::from_millis(200)).await;
    let text = session.get_grid_text();
    assert!(!text.contains("test.txt"));
}

#[tokio::test]
async fn test_htop_rendering() {
    let session = TestSession::new().await?;

    // Start htop
    session.send_keys("htop\n").await?;
    session.wait_for_text("CPU").await?;
    session.wait_for_text("Mem").await?;

    // Verify colored output
    let state_ptr = session.shared_mem.as_ptr() as *const SharedState;
    unsafe {
        let state = &*state_ptr;
        // Check that some cells have non-default colors
        let colored_cells = state.cells.iter()
            .filter(|c| c.fg != 0xFFFFFFFF || c.bg != 0x000000FF)
            .count();
        assert!(colored_cells > 10, "Expected colored output");
    }

    // Quit
    session.send_keys("q").await?;
}
```

---

## 📦 Deliverables

1. **Integration Code**: Wiring between all components
2. **API Updates**: Bevy 0.15 compatibility
3. **E2E Tests**: 10+ complete workflow tests
4. **Bug Fixes**: All integration issues resolved
5. **Stress Test**: 1-hour stability verification

---

## 🔗 Dependencies

- **Depends On**: All Phase 1-4 components complete
- **Critical Path**: Blocks release

---

## 📚 Resources

- [Bevy 0.15 Migration Guide](https://bevyengine.org/learn/migration-guides/0-14-to-0-15/)
- [Integration Testing Patterns](https://martinfowler.com/bliki/IntegrationTest.html)

---

## 🎯 Success Metrics

- ✅ All components wire together
- ✅ Zero crashes in stress test
- ✅ 10+ E2E tests passing
- ✅ vim, htop, git working
- ✅ Plugins execute successfully
- ✅ Performance targets maintained

---

## 🐛 Known Integration Issues

1. **Bevy 0.15 API Changes** (HIGH):
   - Text rendering API changed
   - Color types (rgba → srgba)
   - UI component structure

2. **Mesh Generation** (MEDIUM):
   - Need to connect renderer to SharedState
   - Performance optimization for large grids

3. **Plugin Execution** (LOW):
   - VM and Interpreter need daemon integration
   - Hook execution order

---

**Created**: 2025-11-21
**Labels**: `integration`, `critical`, `e2e`, `testing`, `bug-fixing`
